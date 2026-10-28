# Finite Grid, SimpleGrid and AspectRatio

These are passive layout widgets. They use actual typed CSS grid and aspect-ratio
declarations across Record, HTML, Web/SSR and Blitz. They add no table, row, cell,
selection or interactive grid accessibility roles. Slot controls retain their
native roles, focus order and node identities.

```rust
use sapid::{aspect_ratio, grid, grid_item, simple_grid};
use glory::style::Length;
let cards = grid("cards").columns(4).gap(8.0)
    .item(grid_item("wide").span(2).fill(glory::ui::text().content("Wide")))
    .item(grid_item("small").fill(aspect_ratio(16.0 / 9.0).width(Length::Px(120.0))));
let equal = simple_grid("equal").columns(3)
    .items((0..5).map(|i| grid_item(format!("card-{i}"))));
```

Grid defaults to 12 equal fractional columns and a 12px gap. SimpleGrid defaults
to three equal fractional columns; each item occupies one automatically placed
cell. `column_tracks` and `row_tracks` accept explicit `GridTrack::{Auto,Px,
Percent,Fr}` sequences on Grid. Px/percent values must be finite and nonnegative;
fr values must be finite and positive. Each declared axis contains 1–64 tracks.
Empty lists are rejected. Existing `gap` remains one independent px declaration.
The finite schema does not accept repeat/minmax/functions, named lines, areas,
subgrid, negative line indices or implicit-track sizing declarations.

Each grid has an explicit stable namespace; item keys are unique ASCII keys
under the shared 64-byte component key contract. At most 256 items are accepted.
Iterator setters consume at most the remaining allowance plus one overflow
sentinel, retain no later items, and do not scan million-item lazy sources.
Column/row track iterators likewise consume at most 65 values. Direct typed
GridTracks vectors remain explicit author-owned values, validated at the semantic
boundary. Capture destruction occurs outside graph borrows.
`span`, `row_span`, and optional 1-based `column`/`row` configure placement. Every
span must fit the configured column count and 64 rows, and each end line must
remain within 65. Mount-time placement uses a temporary 64-row bitset, searches
the first free location in row-major order for each item in source order, and
emits explicit column/row positions. It rejects overlapping fixed placements or
items that cannot fit. This bitset is discarded after construction; no item
history or source registry persists. Data/reading order remains source order.
Each item sets min-width:0 so an equal fr track can shrink around long content.

The columns and item configuration are mount-time values. Track geometry responds
to available width through the actual engine. A shared responsive breakpoint
provider and responsive column/span props are still pending; P3-03 therefore
retains a documented partial boundary. No HTML-only media CSS is injected.
Explicit typed style overrides can deliberately change the computed layout.

`aspect_ratio(source)` accepts a read-only numeric Vary. Its preferred ratio is
finite and in [0.000001,1000000]. Width defaults to 100%; `width`, `height`, `fill`,
and typed style overrides remain available. With both axes definite, CSS's size
constraints determine the box. Minimums can exceed maximums; the engine still
applies the minimum. Intrinsic child content can also require a larger box.
No percentage-padding trick, absolute child wrapper or image fit policy is used.

An invalid initial ratio fails before node creation. Later invalid values report
a typed error, preserve the last accepted ratio and recover on a valid source
update. This validation also runs when explicit style overrides the ratio.
Updates are silent and never write the supplied source. WeakScope-owned effects,
bindings and references retire with the component Owner; slot nodes persist
through valid ratio/style changes.

All four themes, classes, names, references and explicit style overrides are
supported. Item theme/size can override inherited parent values. `unstyled`
removes component color tokens while keeping structural grid/placement/min-width
and ratio declarations, matching the existing layout components. Explicit custom
declarations retain precedence and preserve every unspecified declaration.

Glory's HTML/Record profile is v6; Blitz is v7. Five independent properties are
`GridColumns`, `GridRows`, `GridColumn`, `GridRow`, `AspectRatio`, plus the new Grid
display value. Track-list, each original track unit, placement kinds, auto/reset
and ratio requirements are independently rejectable with u64 support masks.
Optional CSS accepts finite track sequences, `none`, `auto`, `span n`,
`n / span m`, and numeric ratio or `a / b`. Every original declaration is checked
against the profile before cascade overrides. The default graph adds no parser
or platform dependency. HTML field lowering and its full-shape oracle cover new
values, explicit resets and clear-to-default from existing grid declarations.

`tests/grid_layout.rs` covers component placement/budgets, invalid creation,
stable slots, controlled ratio recovery, references/Owner retirement, themes,
unstyled output, overrides and SSR escaping. Glory style/html tests cover
numeric domains, all property/value rejections and finite CSS. Native Grid tests
verify px/percent/fr/auto tracks, spans/gaps/row positions, resets and ratio size
constraints against real engine boxes. Sapid's four-theme native fixture verifies
actual 196/298px spans within 400px, equal 128px cards, ratio height 100→200px and
stable engine identities for the ratio box and sibling input.

The browser fixture is `examples/web/src/grid_form.rs`, with its corresponding
`grid-acceptance.js`. It verifies all four themes in CSR and hydration, actual
computed grid/ratio layout, control value/caret and node identity, native AX
control presence, a narrow viewport and teardown. Browser results must be
recorded after the unified fixture build; Rust/native evidence alone does not
establish browser or assistive-technology acceptance.

On 2026-10-24, all eight blue/green × light/dark × CSR/hydration cases passed
with zero console/page errors. Four hydrated screenshots are saved under
`D:/BuildCache/sapid/grid-{blue|green}-{light|dark}-hydrate.png`.
