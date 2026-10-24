# Schedule / AgendaView

Enable optional dependency-free `dates`. `schedule(key, data)` currently builds
the same genuine Agenda view as `agenda_view(key, data)`: a named Group, real
date Headings, unordered Lists/ListItems and ordinary event Buttons. It does not
pretend a date list is a time-grid or a resource timeline. Day/Week/Month/Year,
MobileMonth and resource views are being developed separately.

```rust
let data = scope.cage(ScheduleData::new(
    ScheduleZone::Civil,
    ScheduleWindow::days(CivilDate::new(2024, 2, 26)?, 7)?,
    events,
)?);
agenda_view("work", data)
    .name("Work agenda")
    .on_window_request(move |request| {
        // Application owns loading, cancellation and response sequencing.
        // Supply a new bounded ScheduleData after accepting this request.
    })
    .on_selection_change(move |selection| {
        // Application owns routing/details; nothing navigates implicitly.
    });
```

The model's half-open intervals, explicit zones, recurrence/exception rules,
conflicts, resource/edit permissions and finite budgets are documented in
[schedule-model.md](schedule-model.md). The component accepts at most 2048 current
occurrences. Invalid/overflow source updates are rejected before publishing child
changes, preserving the prior UI; correcting the source recovers silently.
Nothing queries or clones an application's complete historical event collection.
The input is already one finite authoritative request window.

Events are grouped in current start/end order. An event beginning before the
window is listed under the first visible overlapping date; its text still shows
its actual date/time interval. All-day text shows the inclusive final date while
the model retains an exclusive midnight boundary. Ordinary timed events display
seconds and any supplied fractional seconds; no display precision is silently
lost. Resource keys are shown explicitly. `locale` controls the supplied finite
date-heading locale, not an operating-system timezone or a general translation
service.

Selection is local unless `.selection(...)` supplies a controlled source.
`on_selection_change` proposes a bounded `ScheduleSelection::Event`; a controlled
source is read-only to the component. Accepted selection has visible text and
border feedback. Event Buttons retain their true Button role; they do not acquire
false `aria-selected` or `aria-pressed`. Internal selection runs before public
generic Activate observers. `readonly` prevents event selection while allowing
page requests. `enabled(false)` disables both actual row controls and navigation.
Event-specific `SchedulePermissions::select` is checked before every proposal.

Next/Previous propose an adjacent equal-duration half-open `ScheduleWindow` and
the explicit current zone, with a non-wrapping Owner-local request revision.
Several queued requests in one host turn advance an ordered finite cursor; a
later turn starts from the accepted window. Requests do not optimistically
replace data, initiate HTTP or install a route. The application accepts them by
updating its data source. Async loaders must apply their own cancellation/latest
response policy, for example using the existing keyed Resource controller.
Domain-edge controls are disabled instead of wrapping or faulting the App.

The root and header shells remain stable. Date groups key by date; event rows key
by complete event key and actual occurrence start, retaining physical event
identity across title/interval updates in the same group. Callbacks recheck the
current accepted key/candidate/interval and permission. Old rows and captures
retire with their Owners; only the current window remains live. An explicit
off-window controlled selection can remain in business state without a fabricated
visible row. Unrelated editor values, caret and selection are not rewritten by
window changes. Theme/style overrides and unstyled mode preserve functional
geometry; header controls wrap on narrow viewports.

Evidence so far:

- Seven component tests pass in `D:/BuildCache/sapid/schedule-rust.log`: genuine
  Lists/Headings, selection ordering/refusal, keyed identity/current callback,
  same-tick request cursor, overflow rejection/recovery, readonly/disabled,
  domain edges and Owner retirement.
- Two actual native tests pass in `schedule-native-final.log`, including four themes:
  physical layout, real hit testing and focus, accepted selected border pixels,
  keyed engine identity/focus after data updates; separate request/empty/readonly
  and Owner cleanup. This rerun includes final wrapping/list-padding/row geometry;
  focused native strict also passes in `schedule-native-clippy.log`.
- Lib/component strict, WASM strict and SSR check pass in
  `schedule-{clippy,web-clippy,ssr-check}.log`.

The family-58 original twelve-case suite passed **12/12**, errors `[]`, on
Chromium **148.0.7778.96** against fixed FORMAT-10
`http://127.0.0.1:8794/web35`, from `D:/BuildCache/sapid/web35-source`. Report:
`D:/BuildCache/sapid/schedule-browser-web35.json`; raw sequential log
`schedule-date-time-browser-web35.log` also includes DateTime's separate 12/12.
The snapshot script was executed without modifying its assertions. Eight
theme/CSR/hydration cases cover Lists/Headings/Buttons and actual AX, ordered leap
week recurrence/all-day presentation, Space activation and typed ordering,
controlled refusal, stable keyed updates, readonly/disabled and accepted request
windows, empty/Chinese headings, caret and unstyled geometry. Extra cases cover
same-tick request revisions, Owner/current-window lifetime, dirty hydration and
360px touch navigation. Blue-light CSR, green-dark hydration and mobile touch
screenshots were visually inspected (`D:/BuildCache/sapid/schedule-*.png`).
Native driver and browser protocol checks do not prove screen-reader
announcements or OS IME.

The original twelve-case Agenda suite also passes against immutable FORMAT-11
Web36 on Chromium 148.0.7778.96, errors `[]`:
`D:/BuildCache/sapid/schedule-browser-web36.json`. The sequential raw log
`schedule-date-time-browser-web36.log` includes DateTime12 and TimePicker12.
