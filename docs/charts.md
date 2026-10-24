# Optional charts

Enable `sapid`'s `charts` feature and import `sapid::charts`. This adds no chart
library, JavaScript runtime, parser, serde, or host dependency to the production
graph. Drawing uses Glory's existing finite `GraphicProps`; native and HTML/SVG
consume the same admitted geometry.

```rust
use sapid::charts::*;
let data = ChartData {
    categories: vec![ChartCategory::new("jan", "January")],
    series: vec![ChartSeries::new("sales", "Sales", [Some(12.)])],
};
let view = line_chart("sales").data(data);
```

`chart`, `line_chart`, `area_chart`, `bar_chart`, and `composite_chart` return the
same configurable builder. Each named constructor selects its actual geometry;
Composite uses each `ChartSeries.kind`. `None` splits a line or filled area and
is printed as “No value” in the intrinsic table. Zero is a real value. Bar
stacks accumulate positive and negative values independently. Stacked areas
require nonnegative series and reject signed inputs with `SignedAreaStack`.

`ChartData` admits at most 128 categories, eight series, 256 bytes per label,
distinct 1–64 byte ASCII keys, and finite values within ±1e12. The iterator
constructor consumes at most the 129th sentinel. Geometry batches retain the
shared 256-command limit; a chart has at most 1,024 current point controls. It
keeps no visited-data cache. All validation precedes cloning business payloads
or publishing a replacement model.

`CartesianOptions` controls visibility, an inclusive `ChartBrush` interval,
signed stacking, and an optional explicit `ChartDomain`. Automatic scales
include zero; empty/all-zero data uses [0, 1]. Brush changes drawing, while the
default accessible table retains all admitted values. An explicit scale clips
geometry and displays the count of clipped values. Unknown hidden keys,
nonfinite domains, reversed brushes, and invalid data/options combinations
retain the previous admitted chart and report a typed error.

The legend uses real named buttons and a decorative color swatch. It proposes
new options; a controlled `options` source may refuse them. Point Click and
Previous/Next/Clear buttons provide finite sequential data access. Plot points
are programmatically focusable without introducing 1,024 tab stops. Full-key
selection and legend identities survive value updates. Controlled `selection`
only changes after application acceptance; disappearing/hidden points produce
no invented numeric value. Owner removal releases controls and callbacks.

`Legend`/`legend` is also available independently with bounded `LegendItem`
records and controlled `hidden` state. `Tooltip`/`tooltip` takes a validated
`ChartPointDetails` and uses the existing genuine anchored tooltip, description
relation and focus protocol. A missing point disables the relation. The chart
itself always provides inline point details and an intrinsic captioned data
table; colors are never the only representation of values. Applications hiding
`data_table` must supply an equivalent representation.

`ChartTheme::from_theme` supplies eight categorical colors and the theme's grid
color. `chart_theme` explicitly overrides this finite palette. Cosmetic
`unstyled` removes theme declarations; geometry, aspect ratio, data controls and
value representation remain structural. `ChartFrame` supplies the finite
coordinate system, and typed aspect ratio plus percentage placement adapts it
to actual host width. `CartesianScene::to_svg` exports validated drawing layers
without injecting business labels or markup. Axis labels, exact values and
selection remain separate scene/table data; this is not a captured frame.

Native WindowApp supports the default chart's layout, vector drawing, Click,
focus, table and sequential controls. Opt-in `pointer_selection(true)` and
genuine floating Tooltip retain the native adapter's explicit pointer/menu and
Tooltip limitations; they are not enabled by default. Neither protocol/AX
snapshots nor native raster tests establish screen-reader speech or OS IME
acceptance.

## Evidence

- `chart_model.rs`: four pure data/scale tests. `chart_scene.rs`: seven geometry,
  gap, signed-stack, brush, clipping, maximum-budget and safe export tests.
- `tests/charts.rs`: nine component tests, including standalone legend and real
  Tooltip relations, controlled refusal, paired admission, stable identities,
  empty state and repeated-update node bounds.
- `tests/button_enhancements.rs`: nine regressions, including validation inside
  keyed construction and typed rejection of invalid button children.
- `chart-strict.log` and `chart-web-strict.log`: strict library/component and
  WASM checks, with all outputs on D:.
- `charts-browser-web35.json`: **20 original reports, zero errors**, Chromium
  148.0.7778.96. Two pages × four themes × CSR/adoption, focused SSR adoption,
  390px layout, real AX table, keyboard controls, actual Tooltip, controlled
  refusal, stable keyed updates, brush, clipping, SVG export and Owner cleanup.
- `chart-native.log`: real four-theme WindowApp test with 640×320 plot bounds,
  categorical raster pixels, physical hit testing, focus, updates and cleanup.

The immutable browser source and hashes are in
`D:/BuildCache/sapid/web35-source`; the matching browser bundle is retained in
`D:/BuildCache/sapid/web/web35`. Rebuildable target caches were reclaimed after
verification. Browser JS was executed without weakening its
assertions. Mobile screenshots were visually inspected. Other plot families
remain separately tracked in the canonical inventory until their geometry,
widgets and host evidence are complete.
