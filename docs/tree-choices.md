# TreeSelect and Cascader

`tree_select(key)` refines a real Button to a select-only Combobox. Its controls
relation names a real Tree popup, and focus enters actual TreeItems. It never
substitutes listbox Options or an editable input for tree navigation. Arrow keys,
Home/End, Space/Enter and bounded typeahead use Tree's shared protocol; disabled
items are skipped, focus and selection are independent, and Escape restores the
trigger. `readonly` keeps the trigger focusable and suppresses user changes.

Values are `TreeChoices` containing complete `TreePath`s, including every current
ancestor key. Keys remain independent from labels. A leaf key moved under another
parent requires an explicit new path: publish data and its new value together in
a batch. Invalid external pairs report a typed error and retain the last admitted
model/value. External valid updates are silent. Reordering siblings retains live
keys, TreeItems and successful-control identities.

```rust
use sapid::{tree_select, TreeNode, TreeKind};
let widget = tree_select("destination")
    .nodes(vec![
        TreeNode::new("src", "Source").kind(TreeKind::Branch),
        TreeNode::new("main", "Main file").parent("src"),
    ])
    .form_name("destination");
```

`TreeChoicePolicy::Leaves` is the default. `AnyNode` permits selecting a branch or
lazy node itself. Multiple choice additionally supports `DescendantLeaves`: a
branch toggles only its current enabled loaded leaves. Half-checked state is
derived from that current model, exposed as real `aria-checked="mixed"`, and is
independent from list selection metadata. Lazy descendants are not fabricated or
automatically selected after a later load. Opening a lazy disclosure invokes
`on_load_children(key)`; application Resource/request ownership and retry policy
remain explicit. Replace it with a loaded Branch and publish children to finish.

`cascader(key)` uses the same typed paths and Tree semantics in physical columns.
It selects one loaded leaf path; multiple/branch-check policies belong to TreeSelect.
Expansion is one complete ancestor chain; changing branches retires the old
child column. Up/Down/Home/End/typeahead stay in a column. Open/Close use the actual
parent/child path and respect RTL. Both pointer branch activation and the
disclosure open a current child column. A lazy column remains empty until an
explicit publication. Closed columns release their Owners.

Current limits are 2048 nodes, 64 levels, 4096 UTF-8 label bytes and 2 MiB admitted
tree text. Selection holds at most 128 complete paths. Cascader additionally
requires at most 64 siblings in each column, matching the finite Grid placement
profile. Data, selection and these budgets are checked before publishing a new
accepted state. No scrolling callback scans a business source or retains visited
keys. These components mount their finite expanded current trees; they do not
claim arbitrary source windowing.

`field(...)`, explicit names, required/readonly/invalid, controlled/default open,
controlled/default expansion, clear, themes, size and unstyled are supported.
Successful values use real readonly Inputs under a hidden structural View, one
per current accepted path, with the repeated `form_name` and slash-joined ASCII
keys. Semantic keys exclude slash, so this encoding is unambiguous. Empty choice
has no successful value. These proxies are not focus candidates. Required state
is semantic; application FormState validation remains explicit.

Current Glory supports both native intrinsic Combobox tree popups. The
2026-10-29 consumer tests cover four-theme real popup bounds, complete selected
paths, close/focus return, stable trigger identity and direct-apply/query watermarks.
OS keyboard and screen-reader client acceptance remain separate.

Current validation:

- Six pure model tests: complete/reparented paths, independent/leaf/cascade
  policies, mixed states, current lazy columns and capacity refusal.
- Seven TreeSelect component tests and six Cascader component tests cover actual
  semantics/SSR in four themes, successful full paths, controlled refusal,
  same-tick intents/navigation, source admission/recovery, physical column style,
  explicit lazy publication, readonly/disabled and Owner retirement. Ten ordinary
  Tree regressions also pass (`D:/BuildCache/sapid/tree-choices-components.log`).
  The focus loss/gain regression and all thirteen current choice tests pass in
  `D:/BuildCache/sapid/tree-choices-final-components.log`.
- Shared Render Tree six tests and HTML two differential full-shape/clear tests
  pass (`tree-choice-render.log`, `tree-choice-html.log`). Production lib strict
  Clippy passes (`tree-choice-strict.log`).
- The current native consumer/direct-apply tests pass in `native-final-v2-sync.log`.
  `tree-choices-native.log` retains the earlier rejection baseline.
- Browser acceptance passed 21 reports with errors `[]` on Chromium
  `148.0.7778.96` against fixed FORMAT-10 `http://127.0.0.1:8794/web33`:
  `D:/BuildCache/sapid/tree-choices-browser-web33.json` and its matching `.log`.
  Ten reports cover TreeSelect and eleven cover Cascader. Each has eight
  theme/CSR/hydration cases plus actual focused adoption and touch; Cascader adds
  a real three-column mobile scroll/reveal and deep-path submission case.
  Actual Tab exit/Escape return, tree focus, disabled skip, controlled refusal,
  complete FormData, key identity, lazy publication, parent half-checked state,
  RTL, readonly, browser AX, Owner retirement and popup viewport bounds are checked.
  Immutable first-party production and acceptance sources are hashed in
  `D:/BuildCache/sapid/web33-source/source-hashes.json`.

Initial actual Web31 navigation exposed item-to-item focus loss/gain closing the
popup; it now reconciles the final focus observation at the tick boundary, with
an independent regression. Web32's 20 reports passed, but visual inspection found
an eight-pixel mobile fixed-layer overflow that document scrollWidth did not
catch. Geometry now keeps a 95-percent viewport cap even in unstyled mode; Web33
adds the actual popup rectangle and deep horizontal reveal assertions. Earlier
logs and `cascader-mobile-web32-probe.json` are retained. Green dark mobile
TreeSelect/Cascader screenshots were visually inspected. Native AccessKit also
verifies unchecked/checked/mixed/clear with stable identity and no selected facet
(`tree-checked-native-ax.log`); this is separate from the current native popup consumer acceptance.

This evidence does not establish OS IME or screen-reader announcements.
