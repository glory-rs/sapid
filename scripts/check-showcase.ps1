param(
    [switch]$Serve,
    [switch]$Browser,
    [switch]$Release,
    [string]$BuildRoot = 'D:/BuildCache/sapid',
    [string]$Bindgen = 'D:/BuildCache/codmin-glory/tools/bin/wasm-bindgen.exe'
)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$projectRoot = Split-Path -Parent $PSScriptRoot
$BuildRoot = [IO.Path]::GetFullPath($BuildRoot)
if ($BuildRoot -notmatch '^D:\\BuildCache\\[^\\]+') { throw 'BuildRoot must be inside D:/BuildCache' }
$showcaseBuild = "$BuildRoot/showcase"
$showcaseTarget = "$BuildRoot/target"
$savedEnvironment = @{}
$environmentNames = @('CARGO_HOME','CARGO_TARGET_DIR','CARGO_BUILD_JOBS','CARGO_PROFILE_DEV_DEBUG','CARGO_PROFILE_TEST_DEBUG','CARGO_INCREMENTAL','TEMP','TMP','PYTHONPYCACHEPREFIX','SAPID_SHOWCASE_BASE','SAPID_PLAYWRIGHT','SAPID_SHOWCASE_MEDIA')
foreach ($name in $environmentNames) { $savedEnvironment[$name] = [Environment]::GetEnvironmentVariable($name, 'Process') }
function Invoke-ShowcaseCargo {
    param([string[]]$Arguments)
    & cargo @Arguments
    if ($LASTEXITCODE -ne 0) { throw "showcase cargo failed: $($Arguments -join ' ')" }
}
Push-Location $projectRoot
try {
    if (-not $env:CARGO_HOME) { $env:CARGO_HOME = 'D:/BuildCache/glory-semantic-ui/cargo' }
    if ([IO.Path]::GetFullPath($env:CARGO_HOME) -notmatch '^D:\\BuildCache\\[^\\]+') { throw 'CARGO_HOME must be on D drive' }
    $env:CARGO_TARGET_DIR = $showcaseTarget
    $env:CARGO_BUILD_JOBS = '1'
    $env:CARGO_PROFILE_DEV_DEBUG = '0'
    $env:CARGO_PROFILE_TEST_DEBUG = '0'
    $env:CARGO_INCREMENTAL = '0'
    $env:TEMP = "$BuildRoot/tmp"
    $env:TMP = $env:TEMP
    $env:PYTHONPYCACHEPREFIX = "$BuildRoot/pycache"
    New-Item -ItemType Directory -Force -Path $env:TEMP | Out-Null
    # A clean output prevents removed assets or obsolete HTML from leaking into a release.
    if (Test-Path -LiteralPath $showcaseBuild) { Remove-Item -LiteralPath $showcaseBuild -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $showcaseBuild | Out-Null
    $mediaDirectory = & python scripts/site-assets.py prepare $showcaseBuild examples/web/assets
    if ($LASTEXITCODE -ne 0) { throw 'Media fingerprint failed' }
    $env:SAPID_SHOWCASE_MEDIA = "/$mediaDirectory"
    $metadata = & cargo metadata --locked --manifest-path examples/web/Cargo.toml --no-deps --format-version 1 | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'showcase metadata failed' }
    if ([IO.Path]::GetFullPath($metadata.target_directory) -ne [IO.Path]::GetFullPath($showcaseTarget)) { throw "Unexpected showcase target: $($metadata.target_directory)" }
    Write-Output "Showcase target verified: $($metadata.target_directory)"
    & python scripts/showcase-catalog.py --check
    if ($LASTEXITCODE -ne 0) { throw 'showcase catalog is stale' }
    Invoke-ShowcaseCargo -Arguments @('fmt','--manifest-path','examples/web/Cargo.toml','--','--check')
    Invoke-ShowcaseCargo -Arguments @('run','--locked','--manifest-path','examples/web/Cargo.toml','--bin','theme_audit')
    Invoke-ShowcaseCargo -Arguments @('test','--locked','--manifest-path','examples/web/Cargo.toml','--test','showcase_site')
    Invoke-ShowcaseCargo -Arguments @('test','--locked','--manifest-path','examples/web/Cargo.toml','--lib')
    Invoke-ShowcaseCargo -Arguments @('clippy','--locked','--manifest-path','examples/web/Cargo.toml','--all-targets','--','-D','warnings')
    Invoke-ShowcaseCargo -Arguments @('clippy','--locked','--manifest-path','examples/web/Cargo.toml','--lib','--target','wasm32-unknown-unknown','--','-D','warnings')
    if ((& $Bindgen --version).Trim() -ne 'wasm-bindgen 0.2.129') { throw 'showcase requires wasm-bindgen-cli 0.2.129' }
    $profileName = if ($Release) { 'release' } else { 'debug' }
    $buildArguments = @('build','--locked','--manifest-path','examples/web/Cargo.toml','--lib','--target','wasm32-unknown-unknown')
    if ($Release) { $buildArguments += '--release' }
    Invoke-ShowcaseCargo -Arguments $buildArguments
    & $Bindgen "$showcaseTarget/wasm32-unknown-unknown/$profileName/sapid_web_example.wasm" --target web --out-dir $showcaseBuild
    if ($LASTEXITCODE -ne 0) { throw 'showcase wasm-bindgen failed' }
    Copy-Item -LiteralPath examples/web/site.css -Destination "$showcaseBuild/site.css"
    Copy-Item -LiteralPath styles/sapid.css -Destination "$showcaseBuild/sapid.css"
    # Keep bindgen snippets and the relative WASM URL in one immutable asset graph.
    $assetDirectory = & python scripts/site-assets.py bundle $showcaseBuild
    if ($LASTEXITCODE -ne 0) { throw 'Asset fingerprint failed' }
    $generateArguments = @('run','--locked','--manifest-path','examples/web/Cargo.toml','--bin','showcase_ssr')
    # This native build tool is not published. Reuse its validated dev build;
    # the browser bundle above still uses the requested release profile.
    Invoke-ShowcaseCargo -Arguments ($generateArguments + @('--',$showcaseBuild,$assetDirectory))
    if (Test-Path -LiteralPath deploy/_headers) { Copy-Item -LiteralPath deploy/_headers -Destination "$showcaseBuild/_headers" }
    & python scripts/site-assets.py validate $showcaseBuild
    if ($LASTEXITCODE -ne 0) { throw 'Asset validation failed' }
    $oversized = @(Get-ChildItem -LiteralPath $showcaseBuild -Recurse -File | Where-Object Length -GT (25 * 1024 * 1024))
    if ($Release -and $oversized.Count) { throw "Cloudflare Pages assets exceed 25 MiB: $($oversized.Name -join ', ')" }
    $buildFiles = Get-ChildItem -LiteralPath $showcaseBuild -File -Recurse | Where-Object Name -ne 'asset-hashes.json'
    $assetHashes = @($buildFiles | Sort-Object FullName | ForEach-Object { @{ path=[IO.Path]::GetRelativePath($showcaseBuild,$_.FullName);sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash;bytes=$_.Length } })
    $assetHashes | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath "$showcaseBuild/asset-hashes.json"
    Write-Output "Showcase built separately at $showcaseBuild (old 8794 fixtures were not touched)"
    if ($Serve -or $Browser) {
        $listener = Get-NetTCPConnection -LocalAddress 127.0.0.1 -LocalPort 8795 -State Listen -ErrorAction SilentlyContinue
        if (-not $listener) {
            $server = Start-Process -FilePath 'python' -ArgumentList @('-m','http.server','8795','--bind','127.0.0.1','--directory',$showcaseBuild) -WindowStyle Hidden -PassThru -RedirectStandardOutput 'D:/BuildCache/sapid/showcase-server.log' -RedirectStandardError 'D:/BuildCache/sapid/showcase-server-errors.log'
            $server.Id | Set-Content -LiteralPath 'D:/BuildCache/sapid/showcase-server.pid'
        }
        $serverReady = $false
        for ($attempt=0; $attempt -lt 30; $attempt++) {
            try {
                $response = Invoke-WebRequest -Uri 'http://127.0.0.1:8795/showcase.html' -TimeoutSec 2
                if ($response.Content -notlike '*Sapid showcase*') { throw 'port 8795 belongs to another service' }
                $serverReady = $true
                break
            } catch { Start-Sleep -Milliseconds 100 }
        }
        if (-not $serverReady) { throw 'showcase server did not become ready on 8795' }
        Write-Output 'Showcase URL: http://127.0.0.1:8795/'
    }
    if ($Browser) {
        $env:SAPID_SHOWCASE_BASE = 'http://127.0.0.1:8795'
        $env:SAPID_PLAYWRIGHT = Join-Path (Split-Path -Parent $projectRoot) 'glory/tests/playwright/node_modules/playwright'
        & node scripts/showcase-browser.cjs
        if ($LASTEXITCODE -ne 0) { throw 'showcase browser acceptance failed' }
        & node scripts/theme-studio-browser.cjs
        if ($LASTEXITCODE -ne 0) { throw 'theme studio browser acceptance failed' }
        & node scripts/palette-order-browser.cjs
        if ($LASTEXITCODE -ne 0) { throw 'palette order browser acceptance failed' }
        & "$PSScriptRoot/check-theme-export.ps1"
        & node scripts/showcase-navigation-browser.cjs
        if ($LASTEXITCODE -ne 0) { throw 'showcase navigation acceptance failed' }
        & node scripts/showcase-style-browser.cjs
        if ($LASTEXITCODE -ne 0) { throw 'showcase style acceptance failed' }
        & node scripts/mantine-api-browser.cjs
        if ($LASTEXITCODE -ne 0) { throw 'Mantine API browser acceptance failed' }
    }
} finally {
    Pop-Location
    foreach ($name in $environmentNames) { [Environment]::SetEnvironmentVariable($name, $savedEnvironment[$name], 'Process') }
}
