# Sapid

- Continue the widget queue in `docs/widgets-roadmap.md`. Its canonical source
  is `docs/widget-inventory.json`; update status and evidence for each completed
  batch, run `python scripts/widget-plan.py`, then `--check`. Baseline coverage
  does not mean full upstream feature parity; do not mark empty builders done.

- Use handwritten Rust builders implementing Glory `Widget`; keep `Cage`,
  `Bond`, and `Lotus`. Host selection belongs to the application.
- The current development dependency is the sibling `../glory` checkout.
  Preserve unrelated changes there. Keep browser/SSR/native types outside
  component state and use existing semantic controls rather than HTML wrappers.
- Controlled sources are read-only inputs. Callbacks propose changes; local
  defaults apply only without a controlled source. Reconcile after the complete
  host tick so input revisions cannot discard later queued events.
- Internal behavior and typed callbacks precede generic observers. Retire
  component state, Effects, node references and captures with its Owner. Use
  WeakScope for callbacks; do not retain the owning Scope in its own handlers.
- Explicit class/style overrides must preserve component classes and unspecified
  theme declarations. Keep the Style merge exhaustive when Glory adds fields.
- Extend missing semantics in Glory and its shared mapper/adapters, rather than
  injecting raw DOM/ARIA/script behavior into individual widgets. Keep HTML's
  full-shape differential oracle and focused regressions in agreement.
- Run `./scripts/check.ps1`; `-Web -Serve` builds CSR/hydration fixtures.
  `examples/web/acceptance.js` or `scripts/browser.cjs` runs actual browser checks.
  `-Native` also runs the Glory shell/AccessKit suite and strict Clippy in its
  verified D-drive target. Composite browser checks live in
  `examples/web/composite-acceptance.js`.
  Layer browser checks live in `examples/web/overlay-acceptance.js`; all widgets
  share `src/overlay.rs` and Glory's host-owned Layer semantics. Do not implement
  modal focus, Escape or outside-click handling separately in each component.
  Browser protocol tests do not prove OS IME or screen-reader acceptance.
- All artifacts, compiler temporaries, tool caches and generated fixtures stay
  under `D:/BuildCache/sapid` (the existing D-drive Glory Cargo home is reused).
  Verify every build manifest's `cargo metadata` target directory before builds.
- Composite widget keys and item keys are explicit stable ASCII namespaces.
  Keep panel shells and their names stable across open/close and SSR adoption.
  `ContentPolicy::Unmount` retires child Owners; `KeepMounted` hides without
  suspending state/tasks. Never infer actual focus from a queued focus intent.
