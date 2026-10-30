param(
    [Parameter(Mandatory)][string]$BuildRoot,
    [switch]$VerifyOnly,
    [string]$ToolsReceiptSha256 = $env:SAPID_TOOLS_RECEIPT_SHA256,
    [string]$ArchivePath
)
$ErrorActionPreference = 'Stop'
$source = Split-Path $PSScriptRoot -Parent
$manifest = Get-Content -LiteralPath (Join-Path $source 'deploy/build-tools.json') -Raw | ConvertFrom-Json
$packages = Join-Path $source 'deploy/tools'

function Assert-CachePath([string]$Path) {
    $absolute = [IO.Path]::GetFullPath($Path)
    if ($absolute -ine 'D:\BuildCache\sapid' -and -not $absolute.StartsWith('D:\BuildCache\sapid\', [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Build tools and caches must stay under D:/BuildCache/sapid'
    }
    $parent = $absolute
    while ($parent -and $parent -ne 'D:\') {
        if (Test-Path -LiteralPath $parent) {
            if ((Get-Item -LiteralPath $parent -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) {
                throw "Build cache must not traverse a reparse point: $parent"
            }
        }
        $parent = Split-Path $parent -Parent
    }
    return $absolute
}
function Assert-Hash([string]$Path, [string]$Expected, [string]$Label) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw "Missing $Label`: $Path" }
    if ($Expected -notmatch '^[a-f0-9]{64}$' -or (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $Expected) {
        throw "$Label SHA-256 mismatch; refusing cached or downloaded tool"
    }
}
function Get-ModuleHashes([string]$Root) {
    $modules = Assert-CachePath (Join-Path $Root 'node_modules')
    if (-not (Test-Path -LiteralPath $modules -PathType Container)) { throw 'Missing local Node tools; run install-build-tools.ps1 without credentials first' }
    $files = [ordered]@{}
    foreach ($item in Get-ChildItem -LiteralPath $modules -Force -Recurse | Sort-Object FullName) {
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Node tool cache contains a reparse point' }
        if (-not $item.PSIsContainer) {
            $relative = [IO.Path]::GetRelativePath($Root, $item.FullName).Replace('\', '/')
            $files[$relative] = (Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    }
    if (-not $files.Count) { throw 'Missing local Node tool files' }
    return $files
}
function Assert-Packages([string]$Root) {
    $expected = (Get-Content -LiteralPath (Join-Path $packages 'package.json') -Raw | ConvertFrom-Json).dependencies
    foreach ($name in @('wrangler', 'playwright')) {
        $metadata = Join-Path $Root "node_modules/$name/package.json"
        if (-not (Test-Path -LiteralPath $metadata -PathType Leaf)) { throw "Missing local $name package" }
        $installed = Get-Content -LiteralPath $metadata -Raw | ConvertFrom-Json
        if ($installed.name -cne $name -or $installed.version -cne $expected.$name) { throw "Local $name version differs from the locked tool" }
    }
    foreach ($entry in @('node_modules/wrangler/bin/wrangler.js', 'node_modules/playwright/cli.js')) {
        if (-not (Test-Path -LiteralPath (Join-Path $Root $entry) -PathType Leaf)) { throw "Missing local tool entrypoint: $entry" }
    }
}
function Assert-NodeVersion {
    $version = & node --version
    if ($LASTEXITCODE -or $version -notmatch '^v([0-9]+)\.' -or [int]$Matches[1] -lt $manifest.node_min_major) {
        throw "Build tools require Node >= $($manifest.node_min_major)"
    }
}

$BuildRoot = Assert-CachePath $BuildRoot
$tools = Assert-CachePath (Join-Path $BuildRoot 'tools')
$nodeTools = Assert-CachePath (Join-Path $BuildRoot 'node-tools')
$binding = $manifest.wasm_bindgen
if ($binding.package -cne "wasm-bindgen-$($binding.version)-x86_64-pc-windows-msvc") { throw 'Invalid pinned wasm-bindgen package' }
$archive = Join-Path $tools "$($binding.package).tar.gz"
$bindgen = Join-Path $tools "$($binding.package)/wasm-bindgen.exe"
$receiptPath = Join-Path $nodeTools 'tools-receipt.json'
$lockHash = (Get-FileHash -LiteralPath (Join-Path $packages 'package-lock.json') -Algorithm SHA256).Hash.ToLowerInvariant()
if (Test-Path -LiteralPath $tools) {
    foreach ($item in Get-ChildItem -LiteralPath $tools -Force -Recurse) {
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'wasm-bindgen cache contains a reparse point' }
    }
}
if (-not $VerifyOnly) {
    foreach ($name in @('CLOUDFLARE_API_TOKEN','CARGO_REGISTRY_TOKEN','NPM_TOKEN','NODE_AUTH_TOKEN')) {
        if ([Environment]::GetEnvironmentVariable($name)) { throw 'Install build tools in a stage without deployment or registry credentials' }
    }
    New-Item -ItemType Directory -Force $tools,$nodeTools | Out-Null
    $temporary = Assert-CachePath (Join-Path $BuildRoot 'tmp')
    New-Item -ItemType Directory -Force $temporary | Out-Null
    $env:TEMP = $env:TMP = $temporary
    $env:npm_config_cache = Assert-CachePath (Join-Path $BuildRoot 'npm')
}
if (Test-Path -LiteralPath $archive) {
    Assert-Hash $archive $binding.archive_sha256 'wasm-bindgen archive'
} elseif ($VerifyOnly) {
    throw 'Missing cached wasm-bindgen archive'
} else {
    $download = Join-Path $tools 'bindgen-download.tar.gz'
    if ($ArchivePath) {
        Assert-Hash $ArchivePath $binding.archive_sha256 'wasm-bindgen archive'
        Copy-Item -LiteralPath $ArchivePath -Destination $download
    } else {
        Invoke-WebRequest -Uri $binding.archive_url -OutFile $download
    }
    Assert-Hash $download $binding.archive_sha256 'wasm-bindgen archive'
    Move-Item -LiteralPath $download -Destination $archive
}
if (Test-Path -LiteralPath $bindgen) {
    Assert-Hash $bindgen $binding.executable_sha256 'wasm-bindgen executable'
} elseif ($VerifyOnly) {
    throw 'Missing cached wasm-bindgen executable'
} else {
    & tar -xzf $archive -C $tools
    if ($LASTEXITCODE) { throw 'wasm-bindgen extraction failed' }
    Assert-Hash $bindgen $binding.executable_sha256 'wasm-bindgen executable'
}
if ($VerifyOnly) {
    Assert-Hash $receiptPath $ToolsReceiptSha256 'Node tools receipt'
    $receipt = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json -AsHashtable
    if ($receipt.schema -ne 1 -or $receipt.package_lock_sha256 -cne $lockHash) { throw 'Node tools receipt differs from the committed lockfile' }
    Assert-Packages $nodeTools
    $actual = Get-ModuleHashes $nodeTools
    if ((ConvertTo-Json -InputObject $actual -Depth 5 -Compress) -cne (ConvertTo-Json -InputObject $receipt.files -Depth 5 -Compress)) {
        throw 'Node tool cache SHA-256 inventory mismatch; refusing local tools'
    }
    Assert-NodeVersion
} else {
    # npm ci removes its old node_modules. Check the entire target before npm
    # can mutate it, and refuse links that could redirect writes outside cache.
    foreach ($item in Get-ChildItem -LiteralPath $nodeTools -Force -Recurse) {
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Node tool cache contains a reparse point' }
    }
    Assert-NodeVersion
    Copy-Item -LiteralPath (Join-Path $packages 'package.json') -Destination (Join-Path $nodeTools 'package.json')
    Copy-Item -LiteralPath (Join-Path $packages 'package-lock.json') -Destination (Join-Path $nodeTools 'package-lock.json')
    & npm ci --prefix $nodeTools --ignore-scripts --no-audit --no-fund | Out-Host
    if ($LASTEXITCODE) { throw 'Locked Node tool installation failed' }
    Assert-Packages $nodeTools
    @{ schema = 1; package_lock_sha256 = $lockHash; files = (Get-ModuleHashes $nodeTools) } |
        ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $receiptPath -Encoding utf8
    $ToolsReceiptSha256 = (Get-FileHash -LiteralPath $receiptPath -Algorithm SHA256).Hash.ToLowerInvariant()
}
[pscustomobject]@{
    Bindgen = $bindgen
    NodeTools = $nodeTools
    Wrangler = (Join-Path $nodeTools 'node_modules/wrangler/bin/wrangler.js')
    Playwright = (Join-Path $nodeTools 'node_modules/playwright')
    ReceiptSha256 = $ToolsReceiptSha256
}
