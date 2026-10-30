param(
    [switch]$Browser,
    [switch]$Native,
    [string]$Bindgen = 'D:/BuildCache/glory-v2-dioxus-20261028/tools/wasm-bindgen-0.2.129-x86_64-pc-windows-msvc/wasm-bindgen.exe',
    [string]$BuildRoot = 'D:/BuildCache/sapid'
)
$ErrorActionPreference = 'Stop'
$taskBuild = [IO.Path]::GetFullPath($BuildRoot)
$saved = @{}
foreach ($key in @('CARGO_HOME','CARGO_TARGET_DIR','TEMP','TMP','CARGO_INCREMENTAL','CARGO_PROFILE_DEV_DEBUG','CARGO_PROFILE_TEST_DEBUG','CARGO_BUILD_JOBS','SAPID_DSL_SITE')) {
    $saved[$key] = [Environment]::GetEnvironmentVariable($key, 'Process')
}
function Invoke-Cargo([string[]]$Arguments) {
    & cargo @Arguments
    if ($LASTEXITCODE -ne 0) { throw "cargo $Arguments failed" }
}
function Confirm-Target([string]$manifest) {
    $metadata = & cargo metadata --manifest-path $manifest --no-deps --format-version 1 | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0 -or [IO.Path]::GetFullPath($metadata.target_directory) -ne [IO.Path]::GetFullPath($env:CARGO_TARGET_DIR)) { throw 'Unexpected Cargo target directory' }
    Write-Output "Verified target: $($metadata.target_directory)"
}
function Set-DslBuildPaths([string]$Root, [switch]$Native) {
    $absolute = [IO.Path]::GetFullPath($Root)
    if ($absolute -notmatch '^D:\\BuildCache\\sapid(?:\\|$)') { throw 'BuildRoot must be inside D:/BuildCache/sapid' }
    if (-not $env:CARGO_HOME) { $env:CARGO_HOME = 'D:/BuildCache/glory-semantic-ui/cargo' }
    if ([IO.Path]::GetFullPath($env:CARGO_HOME) -notmatch '^D:\\BuildCache\\[^\\]+') { throw 'CARGO_HOME must be on D drive' }
    $env:CARGO_TARGET_DIR = Join-Path $absolute $(if ($Native) { 'native-target' } else { 'target' })
    $env:TEMP = Join-Path $absolute 'tmp'
    $env:TMP = $env:TEMP
    $env:SAPID_DSL_SITE = Join-Path $absolute 'dsl'
    # Each stage must stand alone on a fresh runner. In particular, MSVC link
    # cannot create its lnk*.tmp files when the selected TEMP directory is absent.
    New-Item -ItemType Directory -Force -Path $env:CARGO_TARGET_DIR,$env:TEMP | Out-Null
}
Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    Set-DslBuildPaths -Root $taskBuild
    $env:CARGO_INCREMENTAL = '0'; $env:CARGO_PROFILE_DEV_DEBUG = '0'; $env:CARGO_PROFILE_TEST_DEBUG = '0'; $env:CARGO_BUILD_JOBS = '2'
    Confirm-Target 'Cargo.toml'
    & python "$PSScriptRoot/library_lock.py" --manifest-path Cargo.toml
    if ($LASTEXITCODE -ne 0) { throw 'Library lock preparation failed' }
    Invoke-Cargo -Arguments @('test','--locked','--test','dsl')
    Invoke-Cargo -Arguments @('clippy','--locked','--example','dsl','--','-D','warnings')
    if ($Browser) {
        $manifest = 'examples/web/Cargo.toml'
        Confirm-Target $manifest
        Invoke-Cargo -Arguments @('clippy','--locked','--manifest-path',$manifest,'--lib','--target','wasm32-unknown-unknown','--','-D','warnings')
        Invoke-Cargo -Arguments @('build','--locked','--manifest-path',$manifest,'--lib','--target','wasm32-unknown-unknown')
        New-Item -ItemType Directory -Force -Path $env:SAPID_DSL_SITE | Out-Null
        if ((& $Bindgen --version).Trim() -ne 'wasm-bindgen 0.2.129') { throw 'Expected wasm-bindgen 0.2.129' }
        & $Bindgen "$env:CARGO_TARGET_DIR/wasm32-unknown-unknown/debug/sapid_web_example.wasm" --target web --out-dir $env:SAPID_DSL_SITE
        if ($LASTEXITCODE -ne 0) { throw 'wasm-bindgen failed' }
        Copy-Item -LiteralPath styles/sapid.css -Destination "$env:SAPID_DSL_SITE/sapid.css"
        Invoke-Cargo -Arguments @('run','--locked','--manifest-path',$manifest,'--bin','dsl-ssr','--',$env:SAPID_DSL_SITE)
        node examples/web/dsl-acceptance.cjs
        if ($LASTEXITCODE -ne 0) { throw 'DSL browser acceptance failed' }
    }
    if ($Native) {
        Set-DslBuildPaths -Root $taskBuild -Native
        $env:CARGO_BUILD_JOBS = '1'
        Confirm-Target 'examples/native/Cargo.toml'
        Invoke-Cargo -Arguments @('test','--locked','--manifest-path','examples/native/Cargo.toml','--test','mixed_dsl','--','--test-threads=1')
    }
} finally {
    Pop-Location
    foreach ($key in $saved.Keys) { [Environment]::SetEnvironmentVariable($key, $saved[$key], 'Process') }
}
