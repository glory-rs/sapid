param(
    [Parameter(Mandatory)][string]$Directory,
    [string]$ProjectName = 'sapid-showcase',
    [string]$BuildRoot = 'D:/BuildCache/sapid/pages-ci',
    [string]$ToolsReceiptSha256 = $env:SAPID_TOOLS_RECEIPT_SHA256
)
$ErrorActionPreference = 'Stop'
# Verify all local executable/module bytes before any tool invocation, asset
# request or Cloudflare API call. This step never installs dependencies.
$tools = & "$PSScriptRoot/install-build-tools.ps1" -BuildRoot $BuildRoot -VerifyOnly -ToolsReceiptSha256 $ToolsReceiptSha256
$BuildRoot = Split-Path $tools.NodeTools -Parent
$Directory = [IO.Path]::GetFullPath($Directory)
if (-not $env:CLOUDFLARE_API_TOKEN -or $env:CLOUDFLARE_ACCOUNT_ID -notmatch '^[a-fA-F0-9]{32}$') {
    throw 'Set CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID Actions secrets'
}
if ($ProjectName -notmatch '^[a-z0-9][a-z0-9-]{0,57}[a-z0-9]$') { throw 'Invalid Pages project name' }
foreach ($name in @('index.html','showcase.html','asset-manifest.json','asset-hashes.json','_headers')) {
    if (-not (Test-Path -LiteralPath (Join-Path $Directory $name) -PathType Leaf)) { throw "Missing deployment asset: $name" }
}
& python "$PSScriptRoot/site-assets.py" retain $Directory https://sapid.taidge.org
if ($LASTEXITCODE -ne 0) { throw 'Previous asset retention failed' }
& python "$PSScriptRoot/site-assets.py" validate $Directory
if ($LASTEXITCODE -ne 0) { throw 'Deployment asset validation failed' }
$headers = @{ Authorization = "Bearer $env:CLOUDFLARE_API_TOKEN" }
$endpoint = "https://api.cloudflare.com/client/v4/accounts/$env:CLOUDFLARE_ACCOUNT_ID/pages/projects"
try { $response = Invoke-RestMethod -Uri "$endpoint/$ProjectName" -Headers $headers }
catch {
    if ([int]$_.Exception.Response.StatusCode -ne 404) { throw }
    $body = @{ name = $ProjectName; production_branch = 'main' } | ConvertTo-Json
    $response = Invoke-RestMethod -Method Post -Uri $endpoint -Headers $headers -ContentType 'application/json' -Body $body
}
if (-not $response.success -or $response.result.production_branch -ne 'main') { throw 'Pages project must have main as its production branch' }
$env:WRANGLER_LOG_PATH = Join-Path $BuildRoot 'wrangler/logs'
$env:XDG_CONFIG_HOME = Join-Path $BuildRoot 'wrangler/config'
$env:XDG_CACHE_HOME = Join-Path $BuildRoot 'wrangler/cache'
$env:WRANGLER_SEND_METRICS = 'false'
$deployArguments = @($tools.Wrangler,'--cwd',$BuildRoot,'pages','deploy',$Directory,"--project-name=$ProjectName",'--branch=main')
if ($env:GITHUB_SHA) { $deployArguments += "--commit-hash=$env:GITHUB_SHA" }
& node @deployArguments
if ($LASTEXITCODE -ne 0) { throw 'Cloudflare Pages deployment failed' }
