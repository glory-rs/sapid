# Cloudflare Pages

The public showcase is https://sapid.taidge.org/ and runs as the `sapid-showcase`
Direct Upload Pages project. Its custom domain belongs to the Cloudflare-managed
`taidge.org` zone in the same account as Glory.

Pushes to `main` run `.github/workflows/deploy-showcase.yml`: check out Sapid and
the full Glory revision in `dependencies.json` as sibling directories, validate
the showcase, build release WASM and static entry/SSR documents, run Chromium
acceptance, and upload to Pages. `workflow_dispatch` can rerun production.
Pull requests run the same build and all nine showcase acceptance suites, but
the deploy step is disabled and receives no Cloudflare credentials. Each PR has
its own concurrency group so it cannot replace a production run.

Repository Actions secrets:

- `CLOUDFLARE_ACCOUNT_ID`: selected account's 32-character ID.
- `CLOUDFLARE_API_TOKEN`: Account / Cloudflare Pages / Edit, scoped to that account.

The token is provided only to the deployment step. No DNS permission is needed
for routine uploads. Configure `sapid.taidge.org` in the Pages project's Custom
domains screen; keep its CNAME pointing to the project's actual pages.dev host.

Rust 1.98.0, wasm-bindgen 0.2.129, Playwright 1.64.1, and Wrangler 4.152.0 are
pinned. Node 22 or newer is required; CI selects Node 24 before installing tools.
All tools, targets, npm/browser caches, temporary files and output stay under
`D:/BuildCache/sapid` on the Windows runner (build root `pages-ci`). Cargo's actual output
directory is checked before compilation. Individual assets must fit Pages'
25 MiB limit. HTML revalidates; content-addressed asset directories are immutable.

`build-tools.json` pins the official Windows wasm-bindgen archive SHA-256 and
the extracted executable SHA-256. `scripts/install-build-tools.ps1` verifies both
on every installation and verification, including cache hits. A damaged archive
or executable fails before extraction or execution; it is never silently reused
or replaced. The archive digest is the official release asset's SHA-256; the
executable digest was derived from that verified archive.

`deploy/tools/package-lock.json` fixes the complete Wrangler/Playwright dependency
tree and registry integrity hashes. Installation runs `npm ci --ignore-scripts`
in the external build cache without deployment or registry credentials. It
records every installed module file's SHA-256 in `tools-receipt.json`; CI retains
the receipt digest separately in `SAPID_TOOLS_RECEIPT_SHA256`. Build, acceptance
and deployment verify the cached tools against this trusted digest and the
committed lockfile before using them. Missing tools, modified transitive modules
and replaced receipts fail verification. Preserve the digest returned by the
installer outside the mutable tool cache for local use as well.

The deployment step invokes the verified local Wrangler entrypoint through Node.
It never runs `npx` or installs packages while holding the Cloudflare token, and
it verifies local tools before asset-retention requests or Cloudflare API calls.
Wrangler logs, configuration, cache and working files remain inside the build
root. Production browser acceptance uses the same
`scripts/showcase-acceptance.cjs` suite runner as quality CI, including the actual
theme export compilation; its reports are retained beside the deployment assets.

For an isolated local release build:

```powershell
$buildRoot = 'D:/BuildCache/sapid/pages-local'
$tools = ./scripts/install-build-tools.ps1 -BuildRoot $buildRoot
$env:SAPID_TOOLS_RECEIPT_SHA256 = $tools.ReceiptSha256
$env:SAPID_PLAYWRIGHT = $tools.Playwright
$env:PLAYWRIGHT_BROWSERS_PATH = "$buildRoot/browsers"
node (Join-Path $tools.Playwright 'cli.js') install chromium
./scripts/install-build-tools.ps1 -BuildRoot $buildRoot -VerifyOnly
./scripts/check-showcase.ps1 -Release -BuildRoot $buildRoot -Bindgen $tools.Bindgen
node scripts/showcase-acceptance.cjs --directory "$buildRoot/showcase" --output "$buildRoot/acceptance"
```

Publish only that build's `showcase` directory. Query-based guide and theme URLs
are supported by `showcase.html`; representative `ssr-*.html` documents exercise
adoption and do not imply server rendering for all 149 guides. DNS and TLS must
be active before the custom domain is considered ready.
