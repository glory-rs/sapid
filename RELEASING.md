# Releases

Push an annotated `v<version>` tag after updating manifests, lockfiles and reviewed `releases/<version>.md` notes.
The release workflow runs the repository checks before publishing the dependency-ordered `release.json` allowlist to crates.io and creating a GitHub Release.
Configure the repository Actions secret `CARGO_REGISTRY_TOKEN`. Never commit credentials.
Run `python scripts/release.py` with the approved Cargo artifact directory to validate the plan locally.
Windows Cargo homes, build output and compiler temporaries belong under `D:/BuildCache`.

If workflow environment setup needs repair after a tag was pushed, fix the workflow on main and run the Release workflow manually with that existing tag. Both validation and publication check out the requested immutable tag; the publisher still requires its exact version and commit. Do not move the tag or overwrite a published crate version.

Browser acceptance scripts are independently versioned by the workflow commit. Release builds and Rust/native/DSL checks still use the immutable product tag. The browser harness runs against those exact built fixtures; its own source hashes, fixture hashes and product/harness revisions are preserved as CI artifacts and attached to the GitHub Release. This permits correcting test-driver assumptions without changing the released product source or weakening its assertions. CI runs the browser suites serially to avoid contention between their layout and focus checks.

The static fixture server uses HTTP/1.1 persistent connections and each complete suite receives a fresh browser process. This bounds networking and browser resources over the full matrix: Windows CI previously failed to load pages or WASM with Winsock's `ERR_NO_BUFFER_SPACE`. All modes and identity/focus assertions within each suite still share their original browser lifetime. Do not hide infrastructure failures by dropping suites or converting missing-page failures into passes.

## Acceptance evidence

`D:/BuildCache/...` paths in the capability and verification records identify the
original machine-local runs; those files are not part of a Git checkout. For a
release, use the assets on the repository's [GitHub Releases](https://github.com/sapid-ui/sapid/releases)
page. The workflow attaches `browser-results.json` and `browser-provenance.json`,
including product/harness revisions and source/fixture hashes. Before release,
the same browser reports are available in the run's browser-acceptance Actions
artifacts, whose exact names are shown by the checks. Download artifacts needed for a review while they remain
available, and keep the reports and their provenance together.
An uploaded report can describe a failed or interrupted run: check `complete`,
the recorded errors and the source/fixture hashes before treating it as passing
evidence. Preserve failed reports separately when rerunning a check.

The release publisher also attaches `registry-consumer.json` and its lockfile
after `scripts/verify-registry.py` tests an isolated crates.io-only consumer.
This checks packaged DSL/SSR/native dependencies independently of the pinned
sibling source build. The local path dependency, CI Glory revision in
`deploy/dependencies.json`, and published exact version serve these separate
development and distribution checks.

Keep accessibility evidence scoped to what was exercised: browser keyboard and
Chrome AX checks, native layout/input regressions and AccessKit exports are
separate records. OS IME, screen-reader speech and physical-device acceptance
require their own dated platform/tool versions, steps and results; automated
protocol checks alone do not establish those outcomes.
