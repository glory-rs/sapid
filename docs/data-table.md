# DataTable

`data_table(key)` combines explicit columns, sorting, filtering, row selection,
column visibility, pagination and keyed windowed rows. It uses a real semantic
Table, Row, ColumnHeader and Cell, with finite counts and global coordinates.
Sort buttons expose `aria-sort`. Inputs in cells keep their normal editing keys.

```rust
use sapid::{DataColumn, DataRecord, data_table};

let table = data_table("people")
    .columns([DataColumn::new("name", "Name"), DataColumn::new("team", "Team")])
    .local_rows(vec![DataRecord::new("ada", ["Ada", "Compiler"])])
    .height(320.0)
    .row_height(44.0);
```

Columns have stable semantic ASCII keys of 1..64 bytes, labels of at most 256
bytes, widths of 48..600 pixels and an independent `sortable` flag. There are
1..32 columns; record values follow their configured order, including hidden
columns. Current record keys must be unique. Cells admit canonical text of at
most 4096 bytes, and the current text payload is bounded to 8 MiB. Column and
record constructors consume only their capacity plus one rejection sentinel.
Invalid configurations fail before custom cell factories execute.

`local_rows(source)` admits at most 4096 current records. A source or query
change performs an explicit bounded query: Unicode lowercase substring matching
across all cells, then stable lexical String sorting, then pagination. Equal
sort values retain source order. This is plain-text sorting; numeric, locale
collation and fuzzy search require application data preparation. Scrolling
reads only the accepted page indices and keys, without cloning or rescanning
the business source. The list retains mounted rows and its bounded focus pin,
without a cache of previously visited rows.

`query(source)` and `selected(source)` are read-only controlled values. Their
callbacks propose changes; defaults apply locally only without a controlled
source. Proposals in one host tick compose in order and reconcile after the
whole tick. A refused change restores the actual filter, checkbox or sort
control. External updates do not emit callbacks. Queries use one-based pages,
page sizes of 1..4096, a single optional sort column, a filter of at most 256
bytes and an explicit hidden-column set that leaves at least one column visible.
Selection is an explicit set of at most 4096 keys. It can retain off-page keys
chosen by the application; changing page/filter does not invent or clear them.

`server_page(source)` accepts a `DataPage` containing its exact `DataRequest`,
current rows, optional total and `has_next`. These rows are authoritative;
the component does not sort or filter a server page locally. A changed request
shows the loading state until an exactly matching page arrives. An obsolete
reply cannot clear or replace the current accepted page. Known totals must
agree with row count and `has_next`; unknown totals omit the claimed total and
use the explicit next-page flag. No HTTP transport, implicit request cache or
cancellation policy is installed. Freshness between repeated identical requests
belongs to the application's Resource/request lifecycle.

The virtual list emits Table rows with global zero-based semantic coordinates,
lowered to one-based HTML ARIA coordinates. It does not emit List/ListItem
positions. A header and body use the same explicit column tracks; a narrow
viewport scrolls horizontally inside the component. Row height is fixed at
24..256 pixels; oversized content is clipped. `render_cell(column, factory)`
receives the page index Cage, stable key and reactive value under a short cell
Owner. Key-preserving sorting keeps live editor identity, draft, selection and
focus. Removed rows retire their Owners and captures. Variable-height rows,
multi-column sorting, editable spreadsheet navigation and implicit select-all
are outside this finite component contract.

Theme, size, class/style, unstyled, enabled, name, node_ref, loading/empty labels,
position, on_scroll and ListStats inspection follow the ordinary builders.
The default header controls and cells use blue/green light/dark theme tokens.

Evidence:

- `src/data_table/model.rs`: five pure tests for query stability, independent
  payload/key/shape budgets, authoritative pages and exact request echoes.
- `tests/data_tables.rs`: six tests covering four-theme bounded SSR, same-tick
  controlled refusal, keyed custom editors, admission before factories,
  selection across filters/pages and late server replies.
- Glory `crates/render/tests/table.rs`, `crates/html/src/table_tests.rs` and
  `crates/ui/tests/table_list.rs`: role-table coordinates/sort/reset validation,
  full-shape differential lowering and two finite window/overflow regressions.
- `examples/native/tests/data_tables.rs`: two actual engine tests, including
  four-theme geometry/CPU pixels, 1200 rows with bounded mounted rows, real
  scroll metrics, focused editor identity and server loading/stale reply cleanup.
  `D:/BuildCache/sapid/data-table-native-final.log` records success.
- Glory `crates/blitz/src/accessibility_table_tests.rs`: actual AccessKit
  consumer counts, cell names, sort/reset and coordinates without list positions.
  `D:/BuildCache/sapid/data-table-native-ax.log` records success.
- `examples/web/data-table-acceptance.js` and
  `examples/web/data-table-server-acceptance.js`: **20/20** reports, errors `[]`,
  Chromium 148.0.7778.96 against the fixed format-8 Web27b artifact. This covers
  four-theme CSR/hydration, actual AX, sort/filter/selection refusal, focus pin,
  caret/draft, columns, pagination, mobile horizontal scrolling, dirty takeover,
  exact server echo/loading/stale replies/unknown totals and Owner retirement.
  Report: `D:/BuildCache/sapid/data-table-browser-web27b.json`; inspected mobile
  screenshot: `D:/BuildCache/sapid/data-table-green-dark-mobile.png`.
- Glory `crates/ssr/src/stream/table_virtual_tests.rs`: Rust-generated deferred
  Shell/Patch/End preserves finite role coordinates and sort metadata, escaping
  and resource handoff. Its actual classic receiver case is included above.
- Strict Clippy: `D:/BuildCache/sapid/data-table-strict-final.log` and
  `D:/BuildCache/sapid/data-table-native-strict-final.log`. All build outputs and
  temporary files remain on D:. These tests do not claim screen-reader speech
  or allocator/RSS measurements.
