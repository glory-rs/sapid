# Finite typed plot families

Enable `charts` and import `sapid::charts`. Eighteen constructors select distinct
geometry on the common `Plot` builder: `pie_chart`, `donut_chart`,
`radial_bar_chart`, `scatter_chart`, `bubble_chart`, `candlestick_chart`,
`radar_chart`, `funnel_chart`, `sparkline`, `heatmap`, `matrix_chart`,
`waffle_chart`, `bars_list`, `bullet_chart`, `gauge_chart`, `treemap`,
`sunburst_chart` and `sankey_chart`. Drawing uses the existing portable vector
protocol; no chart runtime, parser, serde or JavaScript chart library is added.
The optional feature pins the pure Rust `libm` 0.2.16 trigonometric primitives
so native SSR and WASM construct identical paths rather than depending on each
platform's last-bit differences in the system sine/cosine functions.

```rust
use sapid::charts::*;
let data = PlotData::values([
    ValueDatum::new("north", "North", 3.),
    ValueDatum::new("south", "South", 7.),
]);
let chart = donut_chart("regions").data(data);
```

`PlotData` is typed by schema. Values, XY/size points, OHLC candles, matrix
axes/cells, target ranges/bands, parent/leaf hierarchies and DAG flows are
different records; incompatible chart/data pairs return `ChartError::Schema`.
Keys are distinct ASCII identifiers of at most 64 bytes; labels are canonical
text of at most 256 bytes and values are finite within ±1e12. Widget namespaces
are at most 48 bytes, reserving space for deterministic table identifiers.
Admission checks counts and all cross-field constraints before cloning or
publishing a replacement. Invalid reactive updates preserve the accepted view.

The default record cap is 128. Radial bars admit 16; Radar and BarsList 32;
Bullet 16 targets and Gauge one, with eight ordered bands each. Matrices have
at most 32 rows, 32 columns and 1,024 uniquely addressed cells. Hierarchies have
128 current nodes and depth 32, with weights only on leaves and totals derived
from visible descendants. Sankey has 128 current nodes, 256 edges and DAG depth
32. Cycles, missing endpoints/parents, duplicate cells, invalid OHLC bounds,
negative sizes/weights and unordered ranges fail explicitly. The widget keeps
only the admitted current records and live keyed controls.

Pie/Donut compute actual angular proportions. Zero totals create no fabricated
slices or percentages. Radial bars use concentric value arcs; Radar uses radial
axes and true missing-value breaks; Funnel uses distinct trapezoids and
Sparkline preserves disconnected spans. Bubble area, rather than radius,
follows size. Candles expose open/high/low/close and up/down/unchanged values.
Heatmap uses continuous color; Matrix also scales cell area; both include
numeric labels, a five-stop numeric color legend and a full exact table. Waffle
uses deterministic largest-remainder allocation of 100 cells while retaining
the original values. BarsList, Bullet and Gauge expose read-only progress
ranges; targets and band boundaries remain exact table values. Treemap uses
alternating proportional subdivisions, Sunburst weighted angular depth bands,
and Sankey uses explicit DAG ranks and conserved incoming/outgoing flow ports.
Sankey zero flows omit ribbons. These are bounded deterministic layouts, not
force simulations or a general-purpose graph editor.

`PlotOptions` controls current-key visibility and applicable explicit XY/value
domains. Unknown keys and irrelevant scale options are rejected. Geometry
clips explicit scales and discloses the clipped count; exact source values
remain in the table. Legend buttons propose visibility changes. Point buttons
and Previous/Next/Clear provide keyboard access without adding every point to
the tab order. Treemap/Sunburst additionally navigate parent/first-child keys.
Controlled `options` and `selection` can refuse proposals; source updates are
silent. Stable business keys preserve live point/legend identities.

Scatter/Bubble can explicitly sample an indexed source:

```rust
let data = PlotData::sample_points(source_len, 128, |index| point_at(index))?;
let chart = scatter_chart("sample").data(data);
```

This reads at most 128 selected indices, including the endpoints when at least
two points are requested. It never scans or clones the source. Sampling metadata
is validated and the table states “128 of N sampled”; it does not claim to
represent all unsampled values. Ordinary finite snapshots are not silently
sampled. Tables chunk at 128 rows under the existing intrinsic table budget,
with deterministic unique IDs and all admitted values represented. Applications
turning `data_table(false)` off must provide an equivalent value representation.

`ChartFrame` supplies the coordinate system and responsive aspect ratio;
`chart_theme` changes a finite palette. `unstyled` keeps structural geometry and
controls. Native defaults support vector drawing, Click, focus and tables.
Opt-in raw pointer selection retains the native adapter's explicit pointer
limitation and is off by default. Screen-reader speech, OS IME and force-layout
acceptance are not inferred from semantic or raster evidence.

## Evidence

- `plot-core-final-tests.log`: 17 pure admission/geometry tests, including
  adversarial generated-key collisions, maximum matrix/graph budgets, true
  gaps/zero handling, sampling callback bounds, area scaling, hierarchy totals
  and conserved flow thickness.
- `plot-component-tests.log`: six tests exercise all 18 widgets, intrinsic exact
  tables, typed paired admission, controlled refusal, stable identities,
  hierarchy navigation, progress roles, 1,024-cell tables, sampling and cleanup.
- `plot-native-tests.log`: all 18 kinds in four real WindowApp themes, with
  640×320 bounds, palette raster pixels, physical hit tests, focus, stable-key
  updates, empty source replacement and full Owner retirement.
- `plot-strict.log` and `plot-web-strict.log`: strict Rust/component and WASM
  compilation. Browser fixtures are six pages (families 68–73).
- `plots-browser-web39.json` and `plots-lightbox-browser-web39.log`: **60
  original reports, errors `[]`**, Chromium148.0.7778.96. Six families × four
  themes × CSR/SSR adoption, actual AX tables/ranges, keyboard access, controlled
  refusal, current-key identity, source/empty replacement, sampling disclosure,
  hierarchy navigation and Owner cleanup; six focused adoptions and six 390px
  mobile views. Mobile screenshots were visually inspected.
- `plots-lightbox-native-final-tests.log` repeats the complete 18-kind ×
  four-theme native test after the portable-math change;
  `plots-lightbox-native-strict.log` is green with warnings denied.

Immutable source/hashes are retained in `D:/BuildCache/sapid/web39-source`, and
the matching browser artifact is `D:/BuildCache/sapid/web/web39`. The Web38
original failure and `plot-hydration-differences38.json` are retained: system
trigonometry differed in one final coordinate bit between SSR and WASM. The
fixed pure Rust math removes that mismatch without relaxing hydration or test
assertions. Subsequent small visual cleanup removes redundant clipped matrix
scale headings, wraps dense navigation controls and rounds displayed pie
percentages to two decimals while preserving original exact numeric values.
That cleanup is captured in immutable Web40 and passes the same original
**60 reports, errors `[]`** (`plots-browser-web40.json` and
`chat-evidence-plots-browser-web40.log`), including all six mobile screenshots.
The final matrix and dense hierarchy controls were visually inspected again.
