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
`code.html` with family 34; its nine-case actual Web suite is pending execution.
