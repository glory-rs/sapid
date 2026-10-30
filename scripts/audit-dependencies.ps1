param(
    [string]$ProjectRoot = (Split-Path -Parent $PSScriptRoot),
    [string]$BuildRoot = 'D:/BuildCache/sapid/dependency-audit',
    [string]$AuditBinary = $(if ($env:SAPID_CARGO_AUDIT) { $env:SAPID_CARGO_AUDIT } else { 'cargo-audit' }),
    [string]$Database = '',
    [switch]$NoFetch
)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
# Match cargo-audit's default policy: vulnerabilities block; informational
# notices stay visible in both raw JSON and the Actions summary.
$BuildRoot = [IO.Path]::GetFullPath($BuildRoot)
if ($BuildRoot -notmatch '^D:\\BuildCache\\sapid(?:\\|$)') { throw 'Audit artifacts must stay under D:/BuildCache/sapid' }
if (-not $Database) { $Database = Join-Path $BuildRoot 'advisory-db' }
if ([IO.Path]::GetFullPath($Database) -notmatch '^D:\\BuildCache\\sapid(?:\\|$)') { throw 'Advisory database must stay under D:/BuildCache/sapid' }
$names = @('CARGO_HOME','CARGO_TARGET_DIR','TEMP','TMP')
$saved = @{}
foreach ($name in $names) { $saved[$name] = [Environment]::GetEnvironmentVariable($name, 'Process') }
$report = @{ tool = 'cargo-audit 0.22.2'; complete = $false; locks = @(); errors = @() }
Push-Location $ProjectRoot
try {
    if (-not $env:CARGO_HOME) { $env:CARGO_HOME = 'D:/BuildCache/sapid/cargo' }
    if ([IO.Path]::GetFullPath($env:CARGO_HOME) -notmatch '^D:\\BuildCache\\[^\\]+') { throw 'CARGO_HOME must stay under D:/BuildCache' }
    $env:CARGO_TARGET_DIR = Join-Path $BuildRoot 'target'
    $env:TEMP = Join-Path $BuildRoot 'tmp'; $env:TMP = $env:TEMP
    New-Item -ItemType Directory -Force -Path $BuildRoot,$env:TEMP | Out-Null
    $version = (& $AuditBinary --version | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or $version -ne $report.tool) { throw "Expected $($report.tool), got $version" }
    # Only the library root is local. Missing application locks remain errors.
    & python "$PSScriptRoot/library_lock.py" --manifest-path Cargo.toml
    if ($LASTEXITCODE -ne 0) { throw 'Audit library lock preparation failed' }
    $revision = $null
    $index = 0
    foreach ($manifest in @('Cargo.toml','examples/web/Cargo.toml','examples/native/Cargo.toml')) {
        $lock = if ($manifest -eq 'Cargo.toml') { 'Cargo.lock' } else { Join-Path (Split-Path $manifest -Parent) 'Cargo.lock' }
        $entry = @{ manifest = $manifest; lockfile = $lock; status = 'failed'; warnings = @(); vulnerabilities = @() }
        $snapshot = Join-Path $BuildRoot "audit-$index.lock"
        if (Test-Path -LiteralPath $snapshot) { Remove-Item -LiteralPath $snapshot -Force }
        try {
            $metadata = & cargo metadata --locked --manifest-path $manifest --no-deps --format-version 1 | ConvertFrom-Json
            if ($LASTEXITCODE -ne 0 -or [IO.Path]::GetFullPath($metadata.target_directory) -ne [IO.Path]::GetFullPath($env:CARGO_TARGET_DIR)) {
                throw "Unexpected metadata target for $manifest"
            }
            $entry.sha256 = (Get-FileHash -LiteralPath $lock -Algorithm SHA256).Hash
            Copy-Item -LiteralPath $lock -Destination $snapshot
            $entry.snapshot = "audit-$index.lock"
            $fetch = @()
            if ($NoFetch -or $index -gt 0) { $fetch = @('--no-fetch') }
            $raw = & $AuditBinary audit --file $lock --db $Database --json @fetch 2> (Join-Path $BuildRoot "audit-$index.stderr.log")
            $exit = $LASTEXITCODE
            $json = $raw -join "`n"
            $json | Set-Content -LiteralPath (Join-Path $BuildRoot "audit-$index.json") -Encoding utf8
            $audit = $json | ConvertFrom-Json
            $entry.report = "audit-$index.json"
            if ($null -eq $audit.vulnerabilities -or $null -eq $audit.settings -or $null -eq $audit.database) { throw 'Incomplete cargo-audit report' }
            if (@($audit.settings.ignore).Count -or @($audit.settings.target_arch).Count -or @($audit.settings.target_os).Count -or $null -ne $audit.settings.severity) {
                throw 'Advisory ignores, severity thresholds and platform filters are not permitted in this gate'
            }
            # cargo-audit deliberately omits Git metadata with --no-fetch.
            # Record the actual shared checkout revision for every scan instead.
            $currentRevision = (& git -C $Database rev-parse HEAD | Out-String).Trim()
            if ($LASTEXITCODE -ne 0 -or $currentRevision -notmatch '^[a-f0-9]{40}$') { throw 'Advisory database revision was not recorded' }
            if ($audit.database.'last-commit' -and $audit.database.'last-commit' -ne $currentRevision) { throw 'Audit report differs from database checkout' }
            if ($revision -and $revision -ne $currentRevision) { throw 'Advisory database changed between lockfile scans' }
            $revision = $currentRevision; $entry.database = $revision
            $entry.vulnerabilities = @($audit.vulnerabilities.list)
            foreach ($property in $audit.warnings.PSObject.Properties) {
                foreach ($warning in $property.Value) {
                    $entry.warnings += $warning
                    Write-Warning "$lock $($warning.package.name) $($warning.package.version): $($warning.advisory.id) $($warning.advisory.title) [$($property.Name)]"
                }
            }
            if ((Get-FileHash -LiteralPath $lock -Algorithm SHA256).Hash -ne $entry.sha256) { throw 'Lockfile changed during audit' }
            if ($exit -ne 0 -or $audit.vulnerabilities.found -or $audit.vulnerabilities.count -gt 0) {
                throw "Audit failed for $lock (exit $exit; vulnerabilities $($audit.vulnerabilities.count))"
            }
            $entry.status = 'passed'
            Write-Output "$lock`: 0 vulnerabilities; $($entry.warnings.Count) informational warnings (database $revision)"
        } catch {
            $entry.error = $_.Exception.Message
            $report.errors += "$lock`: $($entry.error)"
        }
        $report.locks += $entry
        $index++
    }
    $report.complete = $report.locks.Count -eq 3 -and $report.errors.Count -eq 0
} catch { $report.errors += $_.Exception.Message }
finally {
    New-Item -ItemType Directory -Force -Path $BuildRoot | Out-Null
    $report | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath (Join-Path $BuildRoot 'results.json') -Encoding utf8
    if ($env:GITHUB_STEP_SUMMARY) {
        "## RustSec dependency audit`nTool: $($report.tool)`n" | Add-Content -LiteralPath $env:GITHUB_STEP_SUMMARY
        foreach ($entry in $report.locks) {
            "- $($entry.lockfile): **$($entry.status)**; $($entry.vulnerabilities.Count) vulnerabilities; $($entry.warnings.Count) informational warnings." | Add-Content -LiteralPath $env:GITHUB_STEP_SUMMARY
            foreach ($warning in $entry.warnings) {
                "  - $($warning.advisory.id): $($warning.package.name) $($warning.package.version) — $($warning.advisory.title)" | Add-Content -LiteralPath $env:GITHUB_STEP_SUMMARY
            }
        }
        foreach ($failure in $report.errors) { "- **Failure:** $failure" | Add-Content -LiteralPath $env:GITHUB_STEP_SUMMARY }
    }
    Pop-Location
    foreach ($name in $names) { [Environment]::SetEnvironmentVariable($name, $saved[$name], 'Process') }
}
if (-not $report.complete) { throw "Dependency audit failed: $($report.errors -join '; ')" }
