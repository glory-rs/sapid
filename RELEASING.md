# Releases

The 2026-10-30 version bumps are unpublished development versions. Do not publish
or create release tags for this change. Library, example and benchmark locks are
local ignored files; generate and validate them without adding them to Git.


Push an annotated `v<version>` tag after updating manifests, local resolutions and reviewed `releases/<version>.md` notes.
The release workflow first resolves the requested tag to a commit SHA. Repository checks and the publisher both check out that frozen SHA before publishing the dependency-ordered `release.json` allowlist to crates.io and creating a GitHub Release. A push must still resolve to the event's commit; a tag that moves after validation is rejected before publication.
Configure the repository Actions secret `CARGO_REGISTRY_TOKEN`. Never commit credentials.
Run `python scripts/release.py` with the approved Cargo artifact directory to validate the plan locally.
Windows Cargo homes, build output and compiler temporaries belong under `D:/BuildCache`.

If workflow environment setup needs repair after a tag was pushed, fix the workflow on main and run the Release workflow manually with that existing tag. The tag is resolved once, and both validation and publication use the resulting SHA. Release tooling and browser acceptance sources come from the immutable workflow revision, so repaired validation can also publish an older product tag. The publisher still requires the tag's exact version and commit. Do not move the tag or overwrite a published crate version.

Browser acceptance scripts are independently versioned by the workflow commit. Release builds and Rust/native/DSL checks use the frozen product SHA. The browser harness runs against those exact built fixtures; its own source hashes, fixture hashes and product/harness revisions are preserved as CI artifacts and attached to the GitHub Release. This permits correcting test-driver assumptions without changing the released product source or weakening its assertions. CI runs the browser suites serially to avoid contention between their layout and focus checks.

Successful checks return an exact attempt-specific artifact name and a SHA-256 digest of its provenance. Failed or partial browser runs use a separate artifact name. Before Cargo metadata or any registry operation, the publisher verifies that provenance digest, the complete report digest, `complete: true`, no errors, the entire suite list and every suite source hash against the workflow checkout. Provenance must identify this repository and workflow run, the frozen product SHA, the Glory repository/pinned SHA and the acceptance SHA; the publisher's local product, Glory and harness checkouts must agree. CI also verifies the report's fixture hashes against the actual built files before recording successful provenance.

The root library `Cargo.lock` is local development state. Successful checks
copy their actual root resolution into `library-Cargo.lock` in the same acceptance
artifact and bind its SHA-256 in provenance. After validating the complete
source/evidence/tag gate, the publisher restores that exact lock only when its
root lock is missing, before any Cargo metadata, packaging or publishing command.
An existing root lock must match byte for byte; historical tags with a tracked
lock keep their original file. A missing artifact, changed lock or missing lock
digest fails the gate instead of generating a new release resolution. After
updating this evidence format, rerun checks to obtain the required lock artifact
and digest, including when publishing an older product tag.

A publication retry may reuse the exact successful artifact from an earlier attempt of the same workflow run when all source identities and digests still match. The attempt number remains in provenance; artifacts from another run, future attempt, failed suite, changed source or changed tag fail the gate. Evidence is downloaded and checked before any registry upload, and the remote tag is checked again immediately before every upload attempt, including retries after a registry rate-limit wait. Direct local `--publish` without the workflow's frozen identities and successful evidence is rejected. Local plan validation remains `python scripts/release.py`.

Release attachments use `browser-results-run-<run_id>-attempt-<publication_attempt>.json` and
`browser-provenance-run-<run_id>-attempt-<publication_attempt>.json`, alongside the
checked `library-Cargo-run-<run_id>-attempt-<publication_attempt>.lock`. A manual rerun of an
existing tag adds new evidence instead of overwriting earlier reports or colliding with
the original unversioned attachments. The provenance still records the successful
validation attempt when a later publication attempt reuses its evidence. The
`--attach-acceptance` helper revalidates the same gate before uploading those files.

The static fixture server uses HTTP/1.1 persistent connections and each complete suite receives a fresh browser process. This bounds networking and browser resources over the full matrix: Windows CI previously failed to load pages or WASM with Winsock's `ERR_NO_BUFFER_SPACE`. All modes and identity/focus assertions within each suite still share their original browser lifetime. Do not hide infrastructure failures by dropping suites or converting missing-page failures into passes.

## Acceptance evidence

`D:/BuildCache/...` paths in the capability and verification records identify the
original machine-local runs; those files are not part of a Git checkout. For a
release, use the assets on the repository's [GitHub Releases](https://github.com/sapid-ui/sapid/releases)
page. Its browser results and provenance JSON assets include product/harness
revisions and source/fixture hashes. Before release,
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
