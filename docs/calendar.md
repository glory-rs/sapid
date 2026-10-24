# Calendar and MiniCalendar

Enable the optional `dates` feature. Both builders use the explicit
[`DatesProvider` model](dates.md); there is no browser clock, locale lookup,
implicit UTC conversion or date-library dependency in the default graph.

```rust
use sapid::prelude::*;

let reference = CivilDate::parse_iso("2024-02-29")?;
let provider = DatesProvider::new(reference).today(Some(reference));
let selected = scope.cage(None);
let month = scope.cage(reference.year_month());
let widget = calendar("booking-calendar", provider)
    .value(selected)
    .month(month)
    .on_value_change(move |date| selected.revise(|value| *value = date))
    .on_month_change(move |next| month.revise(|value| *value = next));
```

Omit `.value` or `.month` independently to use local state. `.default_value` and
`.default_month` initialize local state only. A controlled callback proposes the
next value or visible month; the application explicitly accepts it. A refused
month retains the accepted cells, roving date and physical focus. External
changes are silent. A visible month outside provider bounds or a selected value
outside those bounds returns an error and preserves the accepted snapshot until
the source becomes valid.

`mini_calendar` returns the same builder with an extra-small default size and a
compact English month label. It keeps all six date rows and the same interaction
contract. The provider's reference date determines the initial visible month,
clamped to its explicit bounds. Today is a separate optional value.

## Real grid and bounded editing state

The semantic structure is a named Grid View, seven Row Views (one weekday header
plus six date rows), seven ColumnHeaders and 42 GridCells. Real Button controls
inside represent valid dates. Dates before year 1 or after year 9999 leave an
empty cell; they do not wrap into a fictional date. Grid/TableMetadata counts and
coordinates agree in HTML and AccessKit. `aria-selected` belongs to GridCell;
today's actual date Button independently has `aria-current="date"`.

Exactly one enabled day has a roving Tab stop when the visible month contains a
selectable day. Other days remain programmatically focusable; native disabled
days cannot receive focus or activation. The focused date and selected value are
separate. Arrow navigation does not implicitly select a day. Enter, Space,
pointer and touch use the real Button activation protocol. `allow_deselect(true)`
allows selecting the current value again to clear it.

The mounted rows, cell shells and semantic root remain stable as the month
changes. Their dates and names update. Buttons in impossible edge cells mount or
retire in their own branch Owners. A single current navigation cursor handles
multiple keys and activation before the host applies focus; it never selects the
old physical origin after a pending move. The cursor is cleared at reconciliation
and never retains visited dates, queries or tasks.

## Keyboard, bounds and disabled days

| Key | Intent |
|---|---|
| Left / Right | Previous / next date; reversed for explicit RTL |
| Up / Down | Same weekday in the preceding / following week |
| Home / End | First / last available day in the current configured week |
| PageUp / PageDown | Previous / next month, preserving the day where valid |
| Shift+PageUp / Shift+PageDown | Previous / next year |
| Enter / Space | Activate the focused real day Button |

Month/year navigation clamps a missing day such as February 31 to the nearest
allowed day in that target month, with forward tie preference. Header Buttons
also provide previous/next month and year without moving focus into the grid.
Header browsing may enter an entirely excluded month so its navigation controls
remain available. Navigation outside the supported civil range or min/max bounds
does nothing.

`disable_weekday(day)` adds a finite weekday bit. `exclude_date` and
`exclude_dates` accept at most 366 explicit entries; iterator admission consumes
at most the limit plus one sentinel and rejects the complete over-budget
configuration. Accepted exclusions are sorted and deduplicated. Day traversal
checks at most 366 days; month/year moves check at most 31 candidates. A fully
disabled destination or exhausted search leaves focus unchanged. There is no
scan of the complete calendar, business collection or prior navigation history.

`GridNavigate` is an independent finite two-dimensional protocol. It does not
change Glory's linear collection Orientation rules. Only a real focused Button
inside its nearest opted-in Grid produces grid commands. Nested Input controls
retain native caret keys. Composition, Alt, Ctrl and Meta preserve host policy;
Shift is recognized only for the explicit year commands. Moved, retired,
foreign, disabled or nearer-grid origins cannot target the previous Grid.

## Presentation, readonly and lifecycle

Theme, size, unstyled, class/style, name, enabled, focus and node_ref use the
ordinary builder API. The public ref and name belong to the semantic Grid;
class/style belong to its outer layout. RTL mirrors the physical column layout
and keyboard direction explicitly. English and simplified Chinese labels and
week-start order come from the provider. No automatic locale negotiation occurs.
The layout has fixed bounded cells based on control size; explicit outer widths
do not invent a responsive breakpoint policy.

Readonly keeps day focus and month/year browsing, sets the real Grid's
`aria-readonly`, and suppresses selection proposals. Disabled propagates to all
actual header/day Buttons. Unstyled removes theme paint while retaining usable
cell and row geometry. Focus border changes track actual physical focus; a
roving Tab stop alone does not paint a fake focus ring.

Calendar is a civil selection surface, without an HTML successful input or
implicit form submission. Applications may register the selected Cage with
FormState. DatePicker/Input integration is a separate component. Root and branch
Owners retire refs, callbacks and current cursor state. No owning Scope, input
history, pending query or task is kept after retirement.

## Evidence

- Six component tests in `tests/calendars.rs`: finite roles/counts,
  selection/current/focus separation, same-tick arrows plus activation,
  controlled month refusal, invalid external recovery, weekday/date exclusions,
  readonly, disabled, iterator sentinel, year edge, deselection and callback
  Owner retirement. `D:/BuildCache/sapid/calendar-rust.log`.
- Seven shared date-model tests and Cargo/strict evidence are in `dates.md`.
- Shared grid tests: three render target/key/identity cases plus Current::Date
  target/reset, all eighteen HTML tests including full field-delta oracle,
  actual native key queue/modifier/editor isolation and Grid/GridCell AX
  selection/coordinates/current. `grid-render-tests.log`, `grid-html-tests.log`
  and `grid-native-tests.log` under `D:/BuildCache/sapid/`.
- Native component: two tests in `examples/native/tests/calendar.rs` passed,
  including four-theme real 42-cell bounds, selected pixels, hit targets,
  physical focus, controlled values, stable engine shell identity, RTL compact
  columns, readonly and finite year-edge layout. Log:
  `D:/BuildCache/sapid/calendar-native.log`. Focused native strict Clippy was
  subsequently re-run successfully in `calendar-native-clippy-final.log`.
- The original four-theme CSR/hydration browser suite passed **12/12**, errors
  `[]`, with Chromium 148.0.7778.96 against the fixed format-8 Web27b artifact at
  `http://127.0.0.1:8794/web27b`. It includes actual Arrow, Enter and Space
  through boxless `when` Groups, independent focus/selection/today, controlled
  month/value refusal, disabled/readonly, locale/week-start, RTL, same-tick
  navigation plus activation, nested editor/modifier/IME isolation, external
  recovery, Owner retirement, dirty editor takeover and touch. Report:
  `D:/BuildCache/sapid/calendar-browser-web27b.json`; log:
  `D:/BuildCache/sapid/calendar-browser-web27b.log`. Screenshots are
  `calendar-{hue}-{scheme}-{csr|hydrate}.png` under the same build root;
  blue light CSR and green dark CSR captures were visually inspected.

Actual screen-reader announcement timing, OS IME, IANA timezone and DST handling
are outside these claims. Calendar itself never changes an absolute instant.
