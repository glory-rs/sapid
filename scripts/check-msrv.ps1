param(
    [ValidateSet('default','all')][string]$FeatureSet = 'default',
    [string]$ProjectRoot = (Split-Path -Parent $PSScriptRoot)
)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
Push-Location $ProjectRoot
try {
    if ((Get-Content Cargo.toml -Raw) -notmatch 'rust-version\s*=\s*"1\.95"') { throw 'Update the MSRV gate when the declared MSRV changes' }
    & python "$PSScriptRoot/library_lock.py" --manifest-path Cargo.toml --toolchain 1.95.0
    if ($LASTEXITCODE) { throw 'MSRV library lock preparation failed' }
    $arguments = @('+1.95.0','check','--locked','--all-targets')
    if ($FeatureSet -eq 'all') { $arguments += '--all-features' }
    & cargo @arguments
    if ($LASTEXITCODE) { throw 'Declared MSRV does not compile' }
} finally { Pop-Location }
