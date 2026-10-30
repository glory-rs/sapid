param([switch]$Check)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
# The examples are independent Cargo workspaces; root --all does not visit them.
foreach ($target in @(
    @{ Manifest = 'Cargo.toml'; Package = 'sapid' },
    @{ Manifest = 'examples/web/Cargo.toml'; Package = 'sapid-web-example' },
    @{ Manifest = 'examples/native/Cargo.toml'; Package = 'sapid-native-acceptance' }
)) {
    $manifest = $target.Manifest
    # --all also traverses path dependencies, including the sibling Glory checkout.
    $arguments = @('fmt', '--manifest-path', (Join-Path $projectRoot $manifest), '--package', $target.Package)
    if ($Check) { $arguments += @('--', '--check') }
    & cargo @arguments
    if ($LASTEXITCODE -ne 0) { throw "Formatting failed: $manifest" }
}
