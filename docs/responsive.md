# Measured-container breakpoints

`responsive_root(initial_width)` owns one measured View. The explicit finite
initial width determines SSR output and the first hydration shape. A live host
then reports that View's border-box width through Resize. If necessary, one
Owner-owned queued task requests one initial Bounds query. It does not poll
synchronously inside an Effect. A newer Resize takes priority over a late
initial response. SSR and other hosts without measurement retain the declared
initial width.

Default thresholds are 576, 768, 992 and 1200 CSS pixels, selecting Base, Small,
Medium, Large and ExtraLarge. `Breakpoints::new([..])` accepts four strictly
increasing positive finite thresholds, up to 1,000,000px. Width uses the same
finite bound. Equality enters the higher tier. The root measures its declared
container; it does not guess browser viewport size or read any business source.

The `.content(|context| ...)` factory runs once. `context.select(values)` creates
an Owner-owned Bond from five explicit `ResponsiveValues`. `context.styles`
validates all five styles before constructing that Bond. Pass it directly to
the ordinary layout builder's `.style(...)`; accepted tier changes patch the
existing nodes. The fixture uses this API for Center alignment, Container
padding, Space height, Grid gap and SimpleGrid gap. Grid track counts and item
placement remain explicit declarations; responsiveness does not invent or
rebuild items.

Only the current width, tier, one pending task and one pending query are retained.
No visited-width table, application-source scan or per-resize allocation of
component trees is involved. Owner retirement cancels the task/query and rejects
all later context calls. A context retained by application code keeps weak
construction access, not an owning UI Scope.

Evidence under `D:/BuildCache/sapid`: `responsive-tests.log` covers boundary
equality, invalid configuration/styles, stable nodes, queued-not-synchronous
measurement, query budgets, late response precedence and Owner cleanup.
`responsive-native-tests.log` verifies actual Center/Container/Space/Grid and
SimpleGrid updates with real native layout and unchanged semantic identities.
`responsive-browser-web36.json` and `responsive-browser-web37.json` record all
ten original browser cases passing with no page or console errors on immutable
HTML11 snapshots. All four themes run
in CSR and hydration, exercise widening/narrowing, measure the five actual
layout consumers, preserve DOM identities and verify Owner removal. A browser
viewport change separately confirms that breakpoints use the declared container.
