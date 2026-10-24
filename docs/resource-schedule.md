# Resource schedules

The optional `dates` feature exports `resources_schedule`, `resources_day_view`,
`resources_week_view`, and `resources_month_view`. They use one admitted
`ResourceScheduleData`: the current `ScheduleData`, up to 64 registered resources,
and an explicit `ScheduleResourceWindow` containing at most eight registrations.
Events must name a registered resource. Off-window registered events remain in
the application data; the widget retains only the current rendered window.

```rust
use sapid::prelude::*;

let window = ScheduleWindow::days(CivilDate::new(2024, 2, 29)?, 1)?;
let resources = ScheduleResources::new([
    ScheduleResource::new(ScheduleKey::new("room-a")?, "Room A")?,
    ScheduleResource::new(ScheduleKey::new("room-b")?, "Room B")?,
])?;
let data = ResourceScheduleData::new(
    ScheduleData::new(ScheduleZone::Civil, window, [])?,
    resources,
    ScheduleResourceWindow::new(0, 2)?,
)?;
let widget = resources_day_view("rooms", data)
    .axis(ResourceAxis::Columns)
    .time_window(ScheduleTimeWindow::new(480, 1080, 30)?)
    .viewport_height(400.)
    .on_window_request(|request| {
        // Application loads/accepts request.window + request.resource_window.
    })
    .on_edit(|proposal| {
        // Application checks authorization, conflicts and recurrence policy,
        // then supplies a new coherent ResourceScheduleData if accepted.
    });
```

The application chooses civil, fixed-offset, or application-resolved zone
semantics. An application zone key does not install an IANA database or DST
rules. No clock, network transport, route, resource loader, or implicit model
write is installed by these builders.

## Windows and layout

Day and Week require exact midnight-to-midnight one-day/seven-day windows.
Month requires an exact `ScheduleWindow::month`. The month constructor defaults
to resource rows; the other constructors default to resource columns.
`ResourceAxis::{Columns,Rows}` transposes real geometry, Grid rows/columns and
keyboard coordinates. RTL reverses the physical horizontal coordinate.

There is one scrolling viewport and one canvas. Day/Week resource-major columns
combine current resources and dates. Time rows are configured by
`ScheduleTimeWindow` (at most 96), with at most 2,048 actual slots overall.
Overlapping timed events use at most 16 lanes per resource/date and at most 4,096
segments. All-day Day/Week events use a separate actual list. Equal-time events
on distinct resources never consume each other's overlap lanes.

Month is a single resource-by-civil-day Grid (at most 8 × 31 slots). A month bar
occupies every civil day its half-open interval intersects. Separate same-day
events receive separate lanes even when their precise times do not overlap;
this prevents the day projection from painting bars on top of one another.
At most 16 projected overlap lanes are admitted. Exact times remain in event
names, selections and edit proposals, including seconds/nanoseconds.

The pure `layout_timeline` and `layout_month` APIs return primitive current
indices and fractions. They reject a complete over-budget configuration before
any partial layout is published. There is no visited-date/resource cache or
scan of application history.

`column_width` is finite 96–1,024px (default 160); it expands to keep overlap
lanes at least 28px wide. `viewport_height` is finite 80–2,048px (default 520).
Time rows are 32px; transposed time columns are 64px. Compact event bars have a
32px minimum extent; their full exact interval is available in the accessible
name and editor. `unstyled()` preserves these functional dimensions and the
scrolling viewport. Explicit common styles apply to the outer component.

The pager proposes dates and resource slices independently through one
`ResourceScheduleRequest`, including the zone and a non-wrapping revision.
Request refusal preserves the admitted window. Multiple requests in a host turn
use one current-turn cursor. The resource stride defaults to the initial page
count. Use `.resource_page_size(1..=8)` when initially mounting a partially filled
final page. A final short page does not shrink the return stride.

`grid_ref` and `viewport_ref` expose actual nodes. Same-size resource-page
changes keep numeric slot shells. A changed resource count or month day count
retires the old Frame Owner, refs, captures and pending measurements. There is
no owning Scope retained in a callback.

## Selection, keyboard and editing

Actual GridCells carry `selected`; their descendant buttons retain genuine
Button semantics. Arrow keys move one physical cell, Home/End move within the
row, Page keys move four rows, and large-page commands choose the first/last
finite slot. The current-turn navigation cursor handles several queued keys and
activation in order. Focus reveal uses typed host actions. Readonly retains
date/resource navigation while suppressing business selections and edits;
disabled reaches actual controls and requests.

Selection is controlled with `.selection(...)` or local through
`.default_selection(...)`. `on_selection_change` receives an exact event
occurrence or a half-open slot with the resource key. External changes are
silent. Choosing a slot/event never changes the authoritative event data.

Providing `on_edit` enables the editor and independent move/start/end drag
handles. A selection button is separate from each boxed View capture area.
Select, move and resize permissions are independent. Events without select
permission can still expose an authorized editor. Keyboard equivalents use
named actual buttons for earlier/later, previous/next resource and each
endpoint. All-day keyboard edits advance whole civil days; other keyboard
edits use the configured minute step.

Pointer/touch edits propose on Drop. Day/Week snap the time-axis displacement to
the configured step; Month snaps to civil days. The perpendicular axis selects
the current visible resource/date column. Move retains full duration; resize
fixes the opposite endpoint and resource. An all-day result must retain midnight
endpoints. Crossed endpoints, exhausted civil dates, unavailable resource moves
and stale/unauthorized proposals are quietly refused.

One Frame owns one gesture, a non-wrapping ticket and at most one Bounds query.
Moves coalesce into that live gesture, without per-pointer history. Accepted
data changes, readonly/disabled, canvas resize, viewport scroll, Cancel and
Owner retirement revoke pending geometry; late replies cannot publish. The
query uses the live canvas and accepted model, with no DOM/source lookup in the
component. The shared host capture supplies touch-action and pointer capture;
the widget does not inject a DOM listener.

Keyboard editing has a 64-intent FIFO per turn. Additional intents beyond that
accepted prefix are ignored. Each proposal runs before its corresponding generic
Activate observer, with structural admission allowed between intents. Accepted
edits use the latest accepted occurrence as their original. A controlled refusal
can accumulate same-kind proposals in the current turn; switching edit kind
starts from the accepted occurrence. Invalid boundary attempts do not consume
the cursor or discard subsequent valid intents. The cursor is cleared after the
turn, so refusal never becomes a hidden persistent write.

`ScheduleData::validate_edit` checks the current occurrence, resource membership,
permissions, duration/opposite endpoint and all-day midnights. Applications choose
instance-versus-series recurrence edits, overlap/conflict policy, persistence,
authorization and acceptance. A successful proposal is not an authorization or
an applied data mutation.

## Evidence

- Pure resource/window/layout/edit tests: 8 in
  `tests/schedule_resource_views.rs`; component/protocol tests: 11 in
  `tests/resource_schedules.rs`. Includes both axes, month projection/lane cap,
  all-day invariants, current-turn budgets, permission independence, request
  stride, accepted/refused editing, late query cancellation and Owner retirement.
  Combined regressions also cover existing Agenda7, Day/Week7 and Calendar8:
  `D:/BuildCache/sapid/resource-schedule-final-rust.log`. The final full-card
  hit-area regression is in `resource-schedule-full-hit-rust.log`.
- Focused strict Clippy:
  `D:/BuildCache/sapid/resource-schedule-ui-clippy.log`.
- Native actual tests: 3 in `examples/native/tests/resource_schedule.rs`,
  `D:/BuildCache/sapid/resource-schedule-full-hit-native.log`. Four themes and both axes
  verify real engine geometry, hit testing, raster colors, focus, scroll metrics,
  query-backed queued Drop and controlled refusal. Separate tests cover the
  actual month Grid, RTL/readonly/cleanup and month cross-day/resource Drop.
  These are native renderer/input-protocol tests, not physical OS-device or
  screen-reader acceptance.
- The four-theme CSR/hydration fixture and original 12-group script are
  `resource-schedule.html`, `src/resource_schedule_form.rs`, and
  `resource-schedule-acceptance.js`. The immutable Web41 and final strengthened
  Web42 scripts each passed all 12 groups, with `errors: []`, in Chromium
  148.0.7778.96. Final reports are
  `D:/BuildCache/sapid/resource-schedule-browser-web42.json` and the matching
  `.log`; screenshots are in `resource-schedule-web42-shots`. The green-dark
  CSR and narrow touch screenshots were visually checked. Web42 additionally
  verifies that a non-editable event's real button fills its card. Actual mouse,
  CDP touch/drop/cancel, controlled refusal, same-tick keyboard ordering,
  resource paging, Owner retirement, dirty adoption, editor caret and both axes
  run against the same frozen source/artifact. These checks do not establish
  physical touchscreen or screen-reader acceptance.
- Final full-matrix replay exposed a test-readiness race in the ordered-edit
  case: the real Edit click can return before its queued Rust frame enables
  Move later. A subsequent programmatic click on that still-disabled button
  correctly emits nothing. The unchanged artifact reproduced this in 3/40
  natural runs, all with `disabled=true`, `edits=0`, `seen=0`; the 37 enabled
  runs all produced the expected two proposals. The original failure and
  progress are retained as `final-browser-before-resource-fix.{log,json}`;
  detailed observations are `final-resource-precondition-probe.{log,json}`.
  The test now awaits the actual enabled control, keeps both clicks in one JS
  turn, and observes terminal edit counts instead of assuming three RAFs.
  FIFO order, controlled refusal, the 10:00 proposal, generic `seen=2` and the
  unchanged accepted 09:00 event remain mandatory. No production/WASM/JS/SSR
  artifact changed. The corrected original 12-group suite passed on that same
  artifact (`final-resource-corrected-recheck.{log,json}`), followed by 40/40
  successful controlled-edit probes (`final-resource-corrected-precondition-probe`).
