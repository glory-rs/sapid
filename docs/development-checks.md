# Development checks

CI uses stable Rust 1.98.0, including its rustfmt and Clippy components. The
repository rustfmt configuration uses stable options only. Generated showcase
sources use the same configuration as ordinary Rust sources.

Format all three independent Cargo workspaces before sending a change:

```powershell
./scripts/format.ps1
./scripts/format.ps1 -Check
python scripts/showcase-catalog.py --check
```

`cargo fmt --all` at the repository root does not visit the separate Web and
Native example workspaces, but does traverse local path dependencies. The
helper targets each Sapid package explicitly so it cannot reformat a sibling
Glory checkout. `scripts/check.ps1` checks all three before starting tests,
even when host-specific compilation is not requested. Its formatter regression
introduces drift in an isolated Native package and verifies both detection and
repair while leaving an intentionally unformatted sibling dependency untouched.

The full host gate remains:

```powershell
./scripts/check.ps1 -Web -Native -Bindgen <wasm-bindgen-0.2.129-executable>
```

Keep build outputs, Cargo caches, temporary files and browser profiles under
`D:/BuildCache/sapid`, as described in `AGENTS.md`. A passing format check does
not replace behavior tests, browser acceptance or Native host validation.

`-BuildRoot` isolates both `target` and `native-target` beneath that directory;
it does not share a native build lock with another checkout's validation run.
Both manifests are checked with Cargo metadata before compilation. Downloaded
dependencies continue to use the existing D-drive Cargo home.
