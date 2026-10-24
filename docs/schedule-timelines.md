# DayView and WeekView

Enable the optional `dates` feature. `day_view(key, data)` accepts exactly one
midnight-to-midnight civil day; `week_view` accepts exactly seven. The application
supplies the current `ScheduleData` and accepts `on_window_request` proposals.
There is no implicit clock, route, network, IANA database or automatic page load.
The Schedule recurrence, zone, half-open interval and Owner contracts apply.

```rust
week_view("work", data)
    .selection(selected)
    .on_selection_change(move |proposal| selected.revise(|v| *v = Some(proposal)))
    .time_window(ScheduleTimeWindow::new(8 * 60, 18 * 60, 30)?)
    .viewport_height(400.)
    .on_window_request(move |request| { /* application supplies the new window */ });
```

The actual Grid contains a date-heading row, a time-label column and real Button
descendants of GridCells. Slot selection is `ScheduleSelection::Slot`; selected
state belongs to GridCell, while events remain ordinary named Buttons. Timed
events use exact clipped time fractions and separate overlap lanes. Touching
half-open intervals reuse a lane. All-day events appear in real date Lists above
the timed grid. An event spanning several days has one visible segment per day,
and every segment proposes its complete original occurrence interval.

The default time extent is 08:00–18:00, thirty-minute rows. Each row is 32px;
`ScheduleTimeWindow` admits at most 96 rows (672 slots in a week). At most sixteen
overlap lanes per date are admitted before mounting or publishing source changes.
The shared occurrence cap remains 2048. Columns have a minimum width of 120px in
WeekView and 480px in DayView. `column_width(96..=1024)` changes that minimum;
overlap lanes may widen it to keep each lane at least 28px. Each event has a
minimum visible height of 18px, including shorter actual intervals; its accessible
name preserves the exact start/end. `viewport_height(80..=2048)` controls the real
scroll viewport independently of content extent. Unstyled mode retains these
functional dimensions. RTL reverses the physical date columns.

Arrow keys move within the current finite grid, preserving the other axis at its
edge. Home/End select the first/last column in that row. PageUp/PageDown move four
rows; Shift+PageUp/PageDown reach the first/last slot. Enter or Space activates a
real slot Button. Navigation requests host ScrollIntoView and focus; an ordered
current-turn cursor keeps multiple queued keys and activation consistent before
the host applies focus. Later turns start from the actual origin. No visited-slot
history, interval timer or repeated geometry query is retained.

Readonly permits slot focus/navigation and window requests but suppresses all
selection proposals. Event Buttons are disabled in readonly mode; disabled locks
all real controls. Controlled refusal preserves the supplied selection/window.
External valid data updates stay silent. Slot shells remain stable across page
data replacements; event rows use complete keys plus occurrence start and retire
with their current Owner. Retired event identities cannot select replacement
data. Generic activation/Grid observers run after internal behavior and typed
callbacks. `node_ref` targets the stable root Group; `grid_ref` and `viewport_ref`
target the physical Grid and scrolling Group.

Evidence: seven component tests plus seven Agenda regressions pass in
`D:/BuildCache/sapid/schedule-timeline-final-rust.log`. Two actual native tests
pass in `schedule-timeline-final-native.log`, covering four-theme lane rectangles,
hit testing, focus, selected pixels, real viewport reveal, readonly, stable engine
slot identity after page replacement and Owner retirement. Lib/component,
native-focused and WASM strict checks pass in
`schedule-timeline-{final-clippy,native-clippy,web-clippy}.log`.
Family59's original twelve-case browser suite passes **12/12**, errors `[]`,
against immutable FORMAT-11 Web37, Chromium148.0.7778.96. Report:
`D:/BuildCache/sapid/schedule-timeline-browser-web37.json`; raw log
`schedule-timeline-browser-web37.log` also contains independent Agenda12 and
DateTime12 regressions. The eight theme/CSR/hydration cases cover actual overlap
rectangles, Grid navigation and selection, real viewport reveal, controlled
refusal, readonly/disabled, stable keyed updates, Day/unstyled/RTL geometry,
unrelated caret and AX. Extra cases cover same-turn keys, Owner retirement,
dirty adoption and 360px touch. Screenshots were visually inspected. They exposed
verbose weekday headings wrapping beyond 32px: the subsequent source uses short
weekday + MM-DD headings, while slot names retain the full date and interval.
That short-heading change is included in immutable FORMAT-12 Web38, whose
original twelve-case suite passes12/12 again, errors `[]`, on Chromium148:
`schedule-timeline-browser-web38.json`; raw `schedule-browser-web38.log` also
contains the independent Month/Year/Mobile suite. The source additionally checks
current enabled state before delivering pending navigation focus. The final
FORMAT-12 Web39 original suite passes12/12 again, errors `[]`, in
`schedule-timeline-browser-web39.json`, with the independent Calendar12 recorded
in `schedule-browser-web39.log`. The guard source also passes the two real native
timeline tests again alongside Calendar3 in `schedule-calendar-guard-native.log`.
Browser
protocol and native driver evidence does not establish OS IME or screen-reader
announcements.
