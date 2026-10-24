# MonthView, YearView and MobileMonthView

Enable `dates`. The three builders accept the same finite `ScheduleData` model,
selection policy and explicit application window-request callbacks as Schedule.
Use `ScheduleWindow::month(YearMonth)` or `ScheduleWindow::year(CivilYear)` to
construct exact half-open calendar windows. A month request advances to the next
calendar month, including February/leap boundaries; a year request advances to
the next calendar year. It never shifts by an assumed 30/365-day duration.
An unrepresentable exclusive end is rejected, including December9999 and the
full year9999. These are civil windows; no clock or time-zone rule is inferred.

```rust
month_view("month", data)
    .week_start(Weekday::Monday)
    .selection(selection)
    .on_selection_change(move |value| selection.revise(|v| *v = Some(value)))
    .on_window_request(move |request| { /* application replaces current data */ });
```

MonthView renders a real seven-column Grid with one weekday ColumnHeader row,
six date rows, exactly42 date cells and real date Buttons. Leading/trailing dates
remain visible but disabled and visually muted.
Each in-month cell summarizes every occurrence overlapping that civil day, so
an event crossing midnight appears on each touched day, while an event ending at
midnight does not appear on the following day. The date Button's full name
includes the date and exact occurrence count. At most three event rows are
initially visible per cell. A named disclosure Button expands the remaining
rows; opening another date collapses the former. It has real expanded semantics,
with no synthetic Menu role. Disclosure Buttons remain mounted while extra row
Owners retire. Event previews show the bounded title, while accessible names
preserve the complete interval; clipping does not claim ellipsis support.

YearView renders twelve actual month cells in a four-row, three-column Grid.
Each month has its exact count and the same bounded preview/disclosure policy.
Counts represent distinct current occurrences overlapping that month, rather
than adding each day of a multi-day occurrence. A month Button proposes the
complete calendar-month interval as `ScheduleSelection::Slot`.

MobileMonthView uses the same42-cell date Grid, with compact48px date Buttons,
visible nonzero counts and a separate real Heading/List for the focused day.
Changing focus changes the local detail pane; it does not implicitly accept a
controlled business selection. All current-day occurrences are available in the
detail List, up to the common2048-occurrence cap. No UI is constructed for
visited windows or off-window data.

Date/month Buttons participate in finite roving focus. Arrow keys stay within
the current grid, preserving the active date when an arrow would enter an
outside-month cell; Home/End reach row edges, PageUp/PageDown move two rows, and
Shift+PageUp/PageDown reach the first/last active period. Enter/Space proposes
the exact half-open day/month interval. Navigation uses host ScrollIntoView and
focus, with a current-turn cursor for multiple queued keys. RTL reverses physical
columns. `grid_ref` targets the physical Grid; the ordinary `node_ref` targets
the stable outer named Group. `locale` supplies finite English/Chinese names;
`week_start` controls date placement. There is no implicit system-today marker.

Readonly keeps date/month focus and local disclosure/details navigation while
locking business selection and event Buttons. Disabled locks the real controls.
Controlled sources remain read-only; callbacks propose values. Internal behavior
and typed callbacks precede generic observers. External valid data replacement
is silent, retains period Button shells, clears the current disclosure and
retires old event rows. Invalid windows/occurrence budgets reject before cloning
or publishing accepted data. Owner retirement clears refs, focus intents and
current-turn state; no timer, query registry or history is installed.

The current model caps512 events and2048 expanded occurrences. Calendar indexes
contain only current numeric memberships (at most42×2048), with each internal
event record stored once. Default visible previews are at most126 rows in a
month or36 in a year; one disclosure adds at most2048 rows and replaces its own
three previews. Mobile details contain only the current focused day's rows.
Unstyled mode preserves widths, compact row heights and focus geometry.

Eight component tests plus seven Day/Week, seven Agenda and two resource-model
regressions pass in `D:/BuildCache/sapid/schedule-calendar-guard-rust.log`.
The independent five-test resource-layout model suite is recorded separately.
Lib/component strict passes in `schedule-calendar-guard-clippy.log`; WASM strict in
`schedule-calendar-web-clippy.log`. Two actual native tests cover four themes
for all three views, real Grid/Button rectangles and hit testing, focus and
selected pixels, disclosure retirement with stable Button identity, compact
mobile sizing, readonly and Owner cleanup. Logs:
`schedule-calendar-final-native.log`, `schedule-calendar-native-clippy.log`.
The original family60 twelve-case browser suite passes12/12, errors `[]`,
against immutable FORMAT-12 Web38, Chromium148.0.7778.96. Report:
`schedule-calendar-browser-web38.json`; raw `schedule-browser-web38.log` also
contains Day/Week12. Eight theme/CSR/hydration cases cover actual Grids, counts,
previews/disclosure, keyboard selection, controlled refusal, readonly/disabled,
calendar requests, keyed identity/caret, mobile details, RTL and AX. Extra cases
cover same-tick cursors, Owner/current window, dirty hydration and360px touch.
Month/Year/Mobile screenshots were visually inspected:
`schedule-calendar-{month,year,mobile}-web38.png`.

That snapshot predates the additional arrow-edge regression and weekday-header
row. The real edge failure is retained in `schedule-calendar-edges-before-fix.log`;
the repaired source passes the eight component tests, and the original browser
suite now includes edge and generic-disable assertions for the next snapshot.
The generic-disable native regression already passed before the explicit
enabled guard, so it is not evidence of a reproduced native fault.

The immutable FORMAT-12 Web39 build includes all those changes. Its original
strengthened twelve-case suite passes12/12, errors `[]`, on Chromium148.0.7778.96:
`schedule-calendar-browser-web39.json`; `schedule-browser-web39.log` also records
Day/Week12. The unchanged case count includes new first/last-row arrow checks,
weekday ColumnHeaders, and a generic navigation observer disabling the current
root. Current-source native reruns pass Calendar3 plus Day/Week2, followed by
strict Clippy, with debug0 in the verified D-drive native target:
`schedule-calendar-guard-native.log`, `schedule-calendar-native-clippy.log`.
The final Month/Year/Mobile images were visually inspected:
`schedule-calendar-{month,year,mobile}-web39.png`. These driver/protocol checks
do not prove screen-reader announcements or OS IME.
