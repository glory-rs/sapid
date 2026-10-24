param([switch]$Web, [switch]$Native, [switch]$Serve, [string]$Bindgen = 'wasm-bindgen')
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskBuild = 'D:/BuildCache/sapid'
$saved = @{}
foreach ($name in @('CARGO_HOME', 'CARGO_TARGET_DIR', 'TEMP', 'TMP', 'PYTHONPYCACHEPREFIX', 'GLORY_DOCUMENT_STREAM_FIXTURE', 'GLORY_MENU_STREAM_FIXTURE', 'GLORY_HINTS_STREAM_FIXTURE', 'GLORY_SELECT_STREAM_FIXTURE')) {
    $saved[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
}
function Invoke-Cargo {
    param([string[]]$Arguments)
    & cargo @Arguments
    if ($LASTEXITCODE -ne 0) { throw "cargo $Arguments failed" }
}
function Confirm-Target([string]$manifest) {
    $metadata = & cargo metadata --manifest-path $manifest --no-deps --format-version 1 | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'cargo metadata failed' }
    if ([IO.Path]::GetFullPath($metadata.target_directory) -ne [IO.Path]::GetFullPath($env:CARGO_TARGET_DIR)) {
        throw "Unexpected target directory: $($metadata.target_directory)"
    }
    Write-Output "Verified target: $($metadata.target_directory)"
}
Push-Location $taskRoot
try {
    $env:CARGO_HOME = 'D:/BuildCache/glory-semantic-ui/cargo'
    $env:CARGO_TARGET_DIR = "$taskBuild/target"
    $env:TEMP = "$taskBuild/tmp"
    $env:TMP = $env:TEMP
    $env:PYTHONPYCACHEPREFIX = "$taskBuild/pycache"
    New-Item -ItemType Directory -Force -Path $env:TEMP | Out-Null
    Confirm-Target 'Cargo.toml'
    Invoke-Cargo -Arguments @('fmt', '--all', '--', '--check')
    Invoke-Cargo -Arguments @('test', '--locked')
    Invoke-Cargo -Arguments @('clippy', '--locked', '--all-targets', '--', '-D', 'warnings')
    Confirm-Target '../glory/Cargo.toml'
    Invoke-Cargo -Arguments @('test', '--locked', '--manifest-path', '../glory/Cargo.toml', '-p', 'glory-style', '-p', 'glory-core', '-p', 'glory-render', '-p', 'glory-ui', '-p', 'glory-html', '-p', 'glory-ssr', '--all-features')
    Invoke-Cargo -Arguments @('clippy', '--locked', '--manifest-path', '../glory/Cargo.toml', '-p', 'glory-style', '-p', 'glory-core', '-p', 'glory-render', '-p', 'glory-ui', '-p', 'glory-html', '-p', 'glory-ssr', '--all-features', '--all-targets', '--', '-D', 'warnings')
    if ($Native) {
        # Reuse the verified D-drive engine target; no native dependencies enter
        # Sapid's default production graph.
        $env:CARGO_TARGET_DIR = 'D:/BuildCache/glory-semantic-ui/target'
        $env:TEMP = 'D:/BuildCache/glory-semantic-ui/tmp'
        $env:TMP = $env:TEMP
        New-Item -ItemType Directory -Force -Path $env:TEMP | Out-Null
        Confirm-Target '../glory/Cargo.toml'
        Invoke-Cargo -Arguments @('test', '--locked', '--manifest-path', '../glory/Cargo.toml', '-p', 'glory-blitz', '--features', 'accesskit,raster', '--lib', '--test', 'layers', '--test', 'style_profile', '--test', 'check_modes', '--test', 'links', '--test', 'shadow', '--test', 'flex_wrap', '--test', 'inline_flex', '--test', 'grid', '--test', 'menu', '--', '--test-threads=1')
        Invoke-Cargo -Arguments @('clippy', '--locked', '--manifest-path', '../glory/Cargo.toml', '-p', 'glory-blitz', '--features', 'accesskit,raster', '--all-targets', '--', '-D', 'warnings')
        Confirm-Target 'examples/native/Cargo.toml'
        Invoke-Cargo -Arguments @('fmt', '--manifest-path', 'examples/native/Cargo.toml', '--', '--check')
        Invoke-Cargo -Arguments @('test', '--locked', '--manifest-path', 'examples/native/Cargo.toml', '--', '--test-threads=1')
        Invoke-Cargo -Arguments @('clippy', '--locked', '--manifest-path', 'examples/native/Cargo.toml', '--all-targets', '--', '-D', 'warnings')
        $env:CARGO_TARGET_DIR = "$taskBuild/target"
        $env:TEMP = "$taskBuild/tmp"
        $env:TMP = $env:TEMP
    }
    if ($Web) {
        $manifest = 'examples/web/Cargo.toml'
        Confirm-Target $manifest
        Invoke-Cargo -Arguments @('fmt', '--manifest-path', $manifest, '--', '--check')
        Invoke-Cargo -Arguments @('clippy', '--locked', '--manifest-path', $manifest, '--all-targets', '--', '-D', 'warnings')
        Invoke-Cargo -Arguments @('clippy', '--locked', '--manifest-path', $manifest, '--lib', '--target', 'wasm32-unknown-unknown', '--', '-D', 'warnings')
        if ((& $Bindgen --version).Trim() -ne 'wasm-bindgen 0.2.129') { throw 'Requires wasm-bindgen-cli 0.2.129 (pass -Bindgen for a specific executable)' }
        Invoke-Cargo -Arguments @('build', '--locked', '--manifest-path', $manifest, '--lib', '--target', 'wasm32-unknown-unknown')
        & $Bindgen "$taskBuild/target/wasm32-unknown-unknown/debug/sapid_web_example.wasm" --target web --out-dir "$taskBuild/web"
        if ($LASTEXITCODE -ne 0) { throw 'wasm-bindgen failed' }
        Copy-Item -LiteralPath examples/web/index.html -Destination "$taskBuild/web/index.html"
        Copy-Item -LiteralPath examples/web/composites.html -Destination "$taskBuild/web/composites.html"
        Copy-Item -LiteralPath examples/web/overlays.html -Destination "$taskBuild/web/overlays.html"
        Copy-Item -LiteralPath examples/web/fields.html -Destination "$taskBuild/web/fields.html"
        foreach($page in @('choices','forms','display','actions','document','toggles','layout','typography','timeline','spoiler','navigation','grid','menu','stepper','visually-hidden','avatar','shell','hints','image','buttons','scroll-area','alerts','graphics','native-select')) {Copy-Item -LiteralPath "examples/web/$page.html" -Destination "$taskBuild/web/$page.html"}
        Copy-Item -LiteralPath ../glory/crates/web/src/stream.js -Destination "$taskBuild/web/glory-stream.js"
        New-Item -ItemType Directory -Force -Path "$taskBuild/web/assets" | Out-Null
        Copy-Item -Path examples/web/assets/* -Destination "$taskBuild/web/assets"
        Copy-Item -LiteralPath examples/web/showcase.css -Destination "$taskBuild/web/showcase.css"
        Copy-Item -LiteralPath examples/web/showcase.js -Destination "$taskBuild/web/showcase.js"
        Copy-Item -LiteralPath styles/sapid.css -Destination "$taskBuild/web/sapid.css"
        Invoke-Cargo -Arguments @('run', '--locked', '--manifest-path', $manifest, '--bin', 'ssr', '--', "$taskBuild/web")
        $env:GLORY_MENU_STREAM_FIXTURE = "$taskBuild/web/menu-stream.html"
        Invoke-Cargo -Arguments @('test', '--locked', '--manifest-path', '../glory/Cargo.toml', '-p', 'glory-ssr', '--all-features', 'menu_stream_retains_intrinsic_roles_and_lowers_popup_layer_and_controlled_properties')
        $env:GLORY_DOCUMENT_STREAM_FIXTURE = "$taskBuild/web/document-stream.html"
        Invoke-Cargo -Arguments @('test', '--locked', '--manifest-path', '../glory/Cargo.toml', '-p', 'glory-ssr', '--all-features', 'document_tags_survive_stream_patch_construction_and_end_handoff')
        $env:GLORY_HINTS_STREAM_FIXTURE = "$taskBuild/web/hints-stream.html"
        Invoke-Cargo -Arguments @('test', '--locked', '--manifest-path', '../glory/Cargo.toml', '-p', 'glory-ssr', '--all-features', 'tooltip_stream_preserves_description_relation_and_passive_layer_through_end')
        $env:GLORY_SELECT_STREAM_FIXTURE = "$taskBuild/web/native-select-stream.html"
        Invoke-Cargo -Arguments @('test', '--locked', '--manifest-path', '../glory/Cargo.toml', '-p', 'glory-ssr', '--all-features', 'select_stream_uses_v4_shape_v2_envelope_and_preserves_native_selection_through_end')
        Write-Output "Browser fixture: $taskBuild/web"
        if ($Serve) { & python -m http.server 8794 --bind 127.0.0.1 --directory "$taskBuild/web" }
    }
}
finally {
    Pop-Location
    foreach ($name in $saved.Keys) {
        [Environment]::SetEnvironmentVariable($name, $saved[$name], 'Process')
    }
}
