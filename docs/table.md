# Table

`table(key)` builds an intrinsic read-only table with `table_head`, `table_body`
and `table_foot` row groups. Use `table_row(key)`, `table_header_cell(key)`,
`table_row_header(key)` and `table_cell(key)` for explicit stable row/cell keys.
The HTML mapper emits real `table`, `caption`, `thead`, `tbody`, `tfoot`, `tr`,
`th` and `td` elements. No interactive grid role is substituted for a table.

```rust
use sapid::{table, table_body, table_cell, table_head, table_header_cell,
            table_row, table_row_header};

let roster = table("roster").caption("Project hours")
    .head(table_head().row(table_row("columns").cells([
        table_header_cell("person").text("Person"),
        table_header_cell("hours").text("Hours"),
    ])))
    .body(table_body().row(table_row("ada").cells([
        table_row_header("person").text("Ada"),
        table_cell("hours").text("12"),
    ])));
```

Rows and cells are composition configurations consumed by the table. Each group
is admitted before its content factories execute. At most 256 rows, 64 logical
columns, 4096 cells and 32 header associations per cell are supported. Iterator
constructors consume only their remaining capacity plus one rejection sentinel.
Keys are nonempty semantic ASCII names of at most 64 bytes; duplicate row keys
across groups and duplicate cell keys within a row fail before factories run.
Labels/captions are bounded to 4096 bytes and reject NUL.

`row_span` and `column_span` place cells in an explicit finite occupancy grid.
Spans must be positive, remain inside their row group, avoid occupied tracks
and fit the column budget. Each cell receives its actual zero-based coordinates;
HTML lowers them to one-based ARIA coordinates. Column headers apply to later
overlapping columns, and row headers apply to rows covered by their span.
Header relationships never point back to the cell itself. Header cells receive
`scope="col"` or `scope="row"` and data cells receive live `headers` IDs.

`.text(source)` is reactive and read-only. `.content(factory)` accepts ordinary
Glory widgets, including controlled inputs, under the cell's short Owner. Text
changes preserve the table, cell and editor identities, focus and selection.
Retirement clears node references and releases those Owners. Caption content
provides the intrinsic accessible table name. An explicit `.name(source)` takes
precedence; the shared mapper/native adapter restore caption naming after Name
Clear without rebuilding content.

Table, rows and cells support theme, size, class, independent Style overrides
and unstyled presentation. `.striped(true)` shades alternate rows and
`.bordered(true)` adds cell borders. Defaults use the active blue/green light/dark
tokens; cells retain genuine table sizing, wrapping and spans. Applications can
put a wider table inside `ScrollArea` when their columns need horizontal space.

This finite Table is the structural foundation for DataTable. Sorting, filtering,
selection, column visibility, pagination and windowed business data belong to
that separate component; Table does not scan or retain an unmounted source.
It introduces no keyboard cursor or edit protocol of its own. Ordinary inputs
inside cells retain their native behavior.

Evidence:

- `src/table/model.rs`: three grid/span/budget tests.
- `tests/tables.rs`: four component tests for four-theme SSR, associations,
  sentinel admission before factories, live text/editor identity and cleanup.
- `../glory/crates/render/tests/table.rs`: four independent target/coordinate/
  content/rollback tests. `../glory/crates/html/src/table_tests.rs` compares live
  field lowering and full-shape output, including resets without tag replacement.
- `../glory/crates/apace/src/accessibility_table_tests.rs`: actual AccessKit
  consumer caption/header/cell names, counts/coordinates and Name Clear identity.
- `examples/native/tests/tables.rs`: two real engine tests, covering four themes,
  shared column geometry, CPU pixels, focused editor identity and physical row/
  column spans. `D:/BuildCache/sapid/table-native-span.log` records both passing;
  `table-ax-native.log` records the AX test. Native checks use the genuine pinned
  engine fork recorded in Glory's `docs/engine-forks.json`.
- `examples/web/table-acceptance.js`: Web24 Chromium 148.0.7778.96 passed all 12
  reports with no errors: four themes in CSR/hydration, real browser AX, column
  and span geometry, dirty adoption/backward caret, mobile and Owner retirement.
  Results: `D:/BuildCache/sapid/table-browser-root24.json`; inspected screenshot:
  `D:/BuildCache/sapid/table-green-dark-mobile.png`.
- `../glory/crates/ssr/src/stream/table_tests.rs`: a Rust-generated deferred
  Shell/Patch/End table; the real classic receiver preserves intrinsic tags,
  caption escaping, headers and coordinates, then retires the controller and
  hands off resources. It is included in the 12 browser reports above.
- `table-sapid-strict-final.log` and `table-native-strict-final.log` record strict
  Clippy success. Test and build outputs remain under `D:/BuildCache/sapid`.
