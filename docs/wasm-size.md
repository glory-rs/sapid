# WASM size baseline

The shared Glory/Sapid matrix lives in the sibling Glory repository:
[`benchmarks/wasm-size`](../../glory/benchmarks/wasm-size/README.md).
The [full results and methodology](../../glory/docs/wasm-size-baseline.md)
include 36 applications and checked-in before/optimized JSON baselines.

It compares an empty module, an empty mounted app, text, reactive buttons,
each optional feature enabled without use, all features enabled without use,
and actual representative charts/chat/code/files/editor/JSON/date widgets.
Each representative is mounted in a browser; buttons update, the editor
accepts input, and every application is unmounted to check cleanup.

Unused Sapid and all seven unused optional features produce byte-identical
WASM to Glory text in the measured configuration. Using a widget retains its
reachable behavior and dependencies, not every exported widget. A used
feature example is a specific workload, not the size of an entire family.

The reactive Button application falls from 1,029,679 to 995,151 B
(34,528 B / 3.35% saved), and from 319,564 to 310,071 B with Brotli
(9,493 B / 2.97% saved). This includes the shared Glory mapper reduction.
The same button with all seven unused features is 995,411 B / 310,379 B
Brotli.

The first optimization isolates optional toolbar hooks from ordinary Button
construction, so standalone buttons no longer retain collection and roving
focus behavior. Public behavior and default features are preserved; toolbar
setup uses one boxed private hook. Existing action, button and toolbar tests,
the full Sapid check script, and browser fixtures validate the change.

Run from the Glory checkout, with the sibling Sapid checkout present:

```powershell
node benchmarks/wasm-size/run.cjs --label candidate --compare benchmarks/wasm-size/baseline.json --check
node benchmarks/wasm-size/browser.cjs D:/BuildCache/sapid/wasm-size/runs/candidate
```

See the runner README for pinned CLI/tool installation. All outputs and
compiler temporaries stay under `D:/BuildCache/sapid`; Cargo metadata verifies
the target path. Reports preserve source hashes, dirty Git status, features,
lockfile, WASM/JS hashes and raw/Brotli/gzip sizes. External structural/theme
CSS, fonts and images are separate assets. These are transport-size probes;
they do not measure allocator usage, rendering speed or full feature parity.
