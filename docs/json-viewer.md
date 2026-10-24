# JsonViewer

The optional `json` feature provides a bounded JSON structure model and genuine
nested disclosure lists. It shares `JsonInput`'s finite validation boundary and
never turns numbers into floating-point values.

```rust
sapid::json_viewer("settings")
    .source(json_text)
    .expanded(expanded_json_pointers)
    .search(search_text)
    .on_expanded_change(move |proposal| expanded.revise(|v| *v = proposal));
```

An empty JSON pointer names the root. Defaults expand only the root. Object
keys escape `~` and `/` as `~0` and `~1`; array segments are zero-based indices.
Decoded duplicate object keys are rejected, including escaped aliases such as
`a` and `\u0061`, so one path cannot address two current nodes. Unknown expanded
paths retain controlled intent. Explicit defaults do not override a controlled
source. User actions propose an expansion, local state accepts only when there
is no controlled source, and programmatic source changes do not emit proposals.

Closed descendants have no mounted Owners. Current retained paths preserve
their UI identity across property reorder and value changes. A closed subtree
releases its Owners, bindings and disclosure captures. Only the finite current
model and one numeric non-wrapping disclosure identity watermark are retained.
Source parsing happens before UI updates; invalid updates preserve the last
accepted tree without rewriting the business source.

Search is an exact case-sensitive substring in decoded keys or raw primitive
spellings. One current index of matches and ancestor paths is recomputed when
the accepted source/query changes. Search opens required ancestors without
changing the accepted expansion source or emitting expansion proposals. During
a nonempty search, disclosure buttons are disabled so visible matches cannot
be hidden by a contradictory collapse. Clearing search restores the accepted
expansion. Visible primitive matches receive theme highlighting.

`JsonViewLimits` permits 1..=65,536 input bytes, 1..=2,048 nodes and 1..=64
container depth (default 32). Decoded object keys have at most 256 bytes, encoded
JSON pointers 2,048 bytes and a search query 256 bytes. Expanded paths have at
most 2,048 entries and 65,536 total bytes; iterators consume at most 2,049 values
before rejecting overflow. Key/path/node budgets are checked while constructing
the current model, before mounting it. Empty or invalid JSON is rejected. Raw
string/number spellings remain available through `JsonTree::value`.

The current semantic UI uses real `ul`/`li` and `Collapsible` buttons, including
expanded/controls relationships and true keyboard activation. Four themes,
unstyled ink and explicit style/class overrides are supported. It does not
advertise `tree`/`treeitem` roles or imply a tree keyboard model that has not
been implemented. Reuse of the future P8 Tree abstraction and clipboard copy
remain pending, so P10-07 is partial. This batch does not claim those features
or general file-tree/lazy-loading behavior.

Seven Rust checks passed in `tests/json_viewer.rs`, covering lossless large
numbers, pointer escapes, duplicate keys, syntax/structural budgets, controlled
rejection, search, source reorder/recovery, bounded iterators and real Owner
retirement. Focused all-features strict Clippy passed. A native acceptance test
passed all four themes with actual disclosure/value Bounds, raster changes,
real focus, search reveal and Owner retirement. The combined Web fixture is
`code.html` with family 34. Its nine-case actual Chromium 148 suite passed fixed
`web27b`, with all four themes in CSR and hydration plus focused pre-adoption
button identity. The raw report
`D:/BuildCache/sapid/json-viewer-browser-web27b.json` has nine reports and no
errors. It checks real Space-key expansion, controlled rejection, property
reorder/retained path identity, unmounted closed descendants, search reveal and
restoration, escaped values, genuine list AX and mobile containment.

The initial Web27a run caught the shared Grid key listener requesting a DOM
Main part from a boxless ancestor. The host now crosses logical Groups while
finding the nearest boxed Grid. Original keyboard assertions passed Web27b;
the failure report/probe remain archived. Green/Dark desktop and 390px mobile
screenshots were inspected in `D:/BuildCache/sapid/code-json-green-dark-*.png`.

`copy_button(sapid::copy_button().label("Copy JSON"))` adds the shared real
CopyButton for the complete accepted JSON source, including collapsed values.
Search presentation and expansion never change those bytes. Invalid JSON updates
retain the previous copy declaration. The child has its own timer/result/status
configuration and inherits `unstyled`. Copy integration focused tests passed in
`code-json-copy-tests.log`; its separate browser suite passed fixed Web29. Reusable Tree navigation and
complete-source copy now have actual host evidence. The original nine display groups
were re-run successfully on fixed Web28 (`json-viewer-browser-web28.json`).

## Reusable Tree navigation

`tree_navigation(true)` opts into the shared `Tree` widget rather than assigning
Tree roles to the disclosure list. It uses that controller's visible preorder,
Arrow keys, Home/End, typeahead, focus reconciliation, Tree/TreeItem metadata and
keyed row Owners. JSON pointers map to numeric ASCII keys retained only for the
current source. Unicode, escaped slash/tilde and long paths keep identity when
properties reorder; removed paths leave no visited-key table. Unknown expansion
paths and current leaf intent survive proposals. Controlled expansion remains
read-only, and search forces ancestors open without writing or proposing changes.

The Tree accessible label contains the decoded property key and its value type.
Its custom content renderer preserves the complete raw value spelling, including
large numbers and leaf strings over 4 KiB, in selectable Code text with bounded
horizontal scrolling. This separates the Tree model's finite accessible label
from the displayed source without truncating business data. Tree mode permits
64 semantic levels including the root (JSON node depth 0..63); document mode
retains the original configurable JSON depth domain. An actual deeper Tree source
is rejected before publishing either its projection or copy declaration, preserving
the last accepted presentation. Model, source and expansion work are bounded to
the current 2048-node source; scrolling never traverses business data.

Ten JsonViewer tests passed in `D:/BuildCache/sapid/json-tree-tests.log`, including
true coordinates, controlled rejection, Unicode path identity, search restoration,
full large leaves and atomic depth rejection. Native Tree mode passed all four
themes with actual Bounds, raster changes, focus, property reorder and search,
alongside the previous disclosure test (`json-tree-copy-native-tests.log`).
`json-tree-web-clippy.log` and `json-tree-clippy.log` passed. The separate
`json-tree-acceptance.js` passed all nine groups on fixed Web29 with Chromium
148.0.7778.96 (`D:/BuildCache/sapid/json-tree-browser-web29.json`). It checks real
Arrow-key navigation and nested levels, closed Owner retirement, focused identity
on reorder, controlled rejection, read-only search, full escaped text, actual
complete-source clipboard writes, Chrome Tree AX and focused SSR adoption.
`code-copy-acceptance.js` passed eight additional four-theme/CSR/hydrate actual
copy groups on the same artifact (`code-copy-browser-web29.json`), including
collapsed JSON values. CopyButton nine-group regression also passed Web29.
