# Releases

Push an annotated `v<version>` tag after updating manifests, lockfiles and reviewed `releases/<version>.md` notes.
The release workflow runs the repository checks before publishing the dependency-ordered `release.json` allowlist to crates.io and creating a GitHub Release.
Configure the repository Actions secret `CARGO_REGISTRY_TOKEN`. Never commit credentials.
Run `python scripts/release.py` with the approved Cargo artifact directory to validate the plan locally.
Windows Cargo homes, build output and compiler temporaries belong under `D:/BuildCache`.

If workflow environment setup needs repair after a tag was pushed, fix the workflow on main and run the Release workflow manually with that existing tag. Both validation and publication check out the requested immutable tag; the publisher still requires its exact version and commit. Do not move the tag or overwrite a published crate version.

Browser acceptance scripts are independently versioned by the workflow commit. Release builds and Rust/native/DSL checks still use the immutable product tag. The browser harness runs against those exact built fixtures; its own source hashes, fixture hashes and product/harness revisions are preserved as CI artifacts and attached to the GitHub Release. This permits correcting test-driver assumptions without changing the released product source or weakening its assertions. CI runs the browser suites serially to avoid contention between their layout and focus checks.
