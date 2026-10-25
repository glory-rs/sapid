# Cloudflare Pages

The public showcase is https://sapid.taidge.org/ and runs as the `sapid-showcase`
Direct Upload Pages project. Its custom domain belongs to the Cloudflare-managed
`taidge.org` zone in the same account as Glory.

Pushes to `main` run `.github/workflows/deploy-showcase.yml`: check out Sapid and
the full Glory revision in `dependencies.json` as sibling directories, validate
the showcase, build release WASM and static entry/SSR documents, run Chromium
acceptance, and upload to Pages. `workflow_dispatch` can rerun production.

Repository Actions secrets:

- `CLOUDFLARE_ACCOUNT_ID`: selected account's 32-character ID.
- `CLOUDFLARE_API_TOKEN`: Account / Cloudflare Pages / Edit, scoped to that account.

The token is provided only to the deployment step. No DNS permission is needed
for routine uploads. Configure `sapid.taidge.org` in the Pages project's Custom
domains screen; keep its CNAME pointing to the project's actual pages.dev host.

Rust 1.98.0, wasm-bindgen 0.2.129, Playwright 1.60.0, and Wrangler 4.152.0 are
pinned. All tools, targets, npm/browser caches, temporary files and output stay
under `D:/BuildCache` on the Windows runner (build root `sapid-pages-ci`, browser
reports under `sapid`). Cargo's actual output
directory is checked before compilation. Individual assets must fit Pages'
25 MiB limit. HTML revalidates; content-addressed asset directories are immutable.

For an isolated local release build:

```powershell
./scripts/check-showcase.ps1 -Release -BuildRoot D:/BuildCache/sapid-pages-local -Bindgen D:/path/to/wasm-bindgen.exe
```

Publish only that build's `showcase` directory. Query-based guide and theme URLs
are supported by `showcase.html`; representative `ssr-*.html` documents exercise
adoption and do not imply server rendering for all 149 guides. DNS and TLS must
be active before the custom domain is considered ready.
