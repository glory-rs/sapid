param(
    [switch]$Browser,
    [switch]$Native,
    [string]$Bindgen = 'D:/BuildCache/glory-v2-dioxus-20261028/tools/wasm-bindgen-0.2.129-x86_64-pc-windows-msvc/wasm-bindgen.exe'
)
$ErrorActionPreference = 'Stop'
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
Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    $env:CARGO_HOME = 'D:/BuildCache/glory-semantic-ui/cargo'
    $env:CARGO_TARGET_DIR = 'D:/BuildCache/sapid/target'
    $env:TEMP = 'D:/BuildCache/sapid/tmp'; $env:TMP = $env:TEMP
    $env:CARGO_INCREMENTAL = '0'; $env:CARGO_PROFILE_DEV_DEBUG = '0'; $env:CARGO_PROFILE_TEST_DEBUG = '0'; $env:CARGO_BUILD_JOBS = '2'
    New-Item -ItemType Directory -Force -Path $env:TEMP | Out-Null
    Confirm-Target 'Cargo.toml'
    Invoke-Cargo -Arguments @('test','--locked','--test','dsl')
    Invoke-Cargo -Arguments @('clippy','--locked','--example','dsl','--','-D','warnings')
    if ($Browser) {
        $manifest = 'examples/web/Cargo.toml'
        Confirm-Target $manifest
        Invoke-Cargo -Arguments @('clippy','--locked','--manifest-path',$manifest,'--lib','--target','wasm32-unknown-unknown','--','-D','warnings')
        Invoke-Cargo -Arguments @('build','--locked','--manifest-path',$manifest,'--lib','--target','wasm32-unknown-unknown')
        $env:SAPID_DSL_SITE = 'D:/BuildCache/sapid/dsl'
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
        $env:CARGO_TARGET_DIR = 'D:/BuildCache/glory-semantic-ui/target'
        $env:TEMP = 'D:/BuildCache/glory-semantic-ui/tmp'; $env:TMP = $env:TEMP
        Confirm-Target 'examples/native/Cargo.toml'
        Invoke-Cargo -Arguments @('test','--locked','--manifest-path','examples/native/Cargo.toml','--test','mixed_dsl','--','--test-threads=1')
    }
} finally {
    Pop-Location
    foreach ($key in $saved.Keys) { [Environment]::SetEnvironmentVariable($key, $saved[$key], 'Process') }
}
