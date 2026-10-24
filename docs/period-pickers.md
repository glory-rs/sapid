# MonthPicker / YearPicker

Enable `sapid`'s optional `dates` feature. It adds no dependency and does not enter
the default graph. `month_picker` and `year_picker` render inline physical grids;
`month_picker_input` and `year_picker_input` add a readonly successful Input and an
independent Button opening the host-owned modal Dialog.

```rust
use sapid::prelude::*;
let provider = DatesProvider::new(CivilDate::new(2024, 2, 29)?);
let value = scope.cage(MonthSelection::Single(Some(YearMonth::new(2024, 2)?)));
let picker = month_picker_input("billing-month", provider)
    .name("Billing month")
    .form_name("billing_month")
    .value(value)
    .on_value_change(move |next| value.revise(|old| *old = next));
# Ok::<(), sapid::DateError>(())
```

Use `CivilYear::new(2024)` and `YearSelection` for a year. These values are civil
periods, independent of a day, timezone or instant. `DatePeriod` is sealed to
`YearMonth` and `CivilYear`; generic `PeriodSelection<T>`, `SelectedPeriods<T>`,
`PeriodBounds<T>` and `PeriodRange<T>` share finite algorithms without admitting
arbitrary business payloads.

Both builders accept `DatePickerMode::{Single,Multiple,Range}`. Single stores an
optional period. Multiple stores at most 32 unique sorted periods in a fixed
array. The constructor consumes at most 33 iterator entries, including duplicates,
before rejecting an oversized input. Range is explicitly `Empty`, `Start` or
`Complete`; reversed endpoints are ordered. Inclusive range membership and length
are arithmetic, including the whole year 0001–9999 or month 0001-01–9999-12 domain.
There is no expanded list of every period in a range.

The successful Input value is exact ASCII ISO: `YYYY-MM` or `YYYY`, comma-separated
multiple values, `start/` for a partial range and `start/end` for a completed range.
Single empty is the empty string. No representative first day is inserted. Parsing
checks the finite text budget before splitting; formatting iterates at most 32
entries. DatesProvider date bounds project to their containing month/year, so a
partially bounded month or year remains selectable. Use explicit `exclude_value`
or `exclude_values` to deny a whole period. At most 128 exclusions are admitted;
iterators consume a 129th sentinel and reject the complete configuration. Disabled
interior periods can remain inside an inclusive selected range; exclusions govern
new endpoints. Selecting a 33rd distinct period calls `on_reject` without faulting
the App or replacing the last accepted selection.

Controlled `value` and `year` are read-only proposal sources. `on_value_change` and
`on_year_change` must write them explicitly to accept a proposal. Local defaults
work without a controlled source. `year(CivilYear)` selects the visible month year
or the decade containing that year; no reactive source is silently clamped.
Invalid external mode/bounds/page combinations preserve the accepted rendering;
later valid source recovery is silent. Actions read accepted state, and an
ephemeral bounded cursor composes same-tick arrows and activation across pages.
It is cleared after reconciliation/Owner retirement and retains no visited-period
history. Readonly keeps roving navigation and suppresses selection proposals;
disabled removes the controls from interaction.

Month has exactly 12 GridCells in four rows and three columns. Year has ten cells
in two rows and five columns, aligned to calendar decades (2020–2029). The first
decade has a noninteractive empty year-zero cell; the final view ends at 9999.
Cells expose actual `GridCell` selected state and contain real Buttons. Explicit
provider `today` marks its containing period with `Current::Date`, independently of
selection. Arrow keys move focus without selecting; Home/End use the physical row.
RTL reverses horizontal commands. PageUp/Down moves one year for Month and one
decade for Year. Shift+PageUp/Down (`PreviousLargePage` / `NextLargePage`) moves ten
years for Month and one hundred years for Year. Calendar uses the same generic
large-page commands for one year. Native/Web ignore composing, modified and nested
editor navigation; the shared origin/ancestry guard rejects retired or moved
origins. At a domain/bounds edge movement is suppressed, without wrapping.

Input wrappers support Field metadata, explicit successful `form_name`, required,
invalid, readonly/disabled, separate trigger/panel/grid refs and controlled `open`.
The display remains readonly even when selection is writable. A completed accepted
single/range closes the Dialog; a partial range or multiple selection remains open.
Controlled selection refusal leaves it open; controlled open refusal is independent.
Host Layer semantics own initial close-button focus, Tab, Escape, outside dismissal
and return focus. KeepMounted content preserves stable Input/Grid identities;
retiring the Owner clears its refs, callbacks and Layer. No HTTP, routing, timers,
serde, native date dialog or OS locale/timezone lookup is installed.

The default theme and `unstyled` preserve actual finite grid/control geometry.
Explicit style still applies. Month labels use `DatesProvider`'s supported finite
locale; year labels remain four-digit civil years. This is not general locale
calendar arithmetic or IANA/DST conversion.

Validation currently completed:

- Six `period_model::tests` include all 119,988 valid months and 9,999 years,
  checked arithmetic, exact ISO, capacity sentinel and full-domain ranges:
  `D:/BuildCache/sapid/period-model-rust.log`.
- Eight component regressions cover real rows/cells/current/selection, boundary
  empty cells, same-tick cross-year focus/activation, large pages, readonly,
  controlled invalid/recovery/refusal, disabled interior, capacity rejection,
  successful Input values and Owner cleanup:
  `D:/BuildCache/sapid/period-picker-rust.log`.
- Two actual native tests each run all four themes. They open the default real
  Dialog, query actual layout/focus, hit the selected Button, check selected accent
  raster pixels, navigate/select/close/return, preserve Input engine identity and
  clear Layer/query state on Owner retirement:
  `D:/BuildCache/sapid/period-picker-native.log`.
- Sapid lib/components and WASM strict Clippy passed:
  `period-picker-clippy.log`, `period-picker-web-clippy.log`.
- Shared generic large-page rename regression: Render Grid 3, Calendar 6 and native
  actual queued Grid keyboard 1 passed in `grid-large-page-{render,calendar,native}.log`.

The joint `period-picker-acceptance.js` passed **12/12**, errors `[]`, against
fixed FORMAT-9 `http://127.0.0.1:8794/portal-next`, Chromium **148.0.7778.96**.
Every one of eight theme/CSR/hydration cases checks both Month and Year: actual
Dialog safe focus/Tab/Escape/return, true cells and roving focus, one-year month
page and hundred-year year large page, exact ISO Field/FormData/Form Enter,
controlled refusal, readonly/disabled, multiple/range, RTL, browser AX tree and
unchanged editor identity/caret. Four further cases cover same-tick cross-year
arrows followed by activation, 33rd-year rejection and controlled page refusal,
Owner retirement/remount, dirty SSR adoption, and actual touch for both pickers.
These are **12 joint cases, not 24 separate cases**. Report and log:
`D:/BuildCache/sapid/period-picker-browser-portal-next.json` and
`D:/BuildCache/sapid/period-picker-browser-portal-next.log`. Screenshots:
`period-picker-{hue}-{scheme}-{csr|hydrate}.png`; blue light CSR and green dark
hydration were visually inspected for grid layout and popup bounds. Native
focused strict also passed (`period-picker-native-clippy.log`).

Native typed command, layout and pixel tests do not establish OS IME or
screen-reader announcements.

The enhanced original joint 12/12 suite also passed against fixed FORMAT-10
Web31 with Chromium 148.0.7778.96, errors `[]`:
`D:/BuildCache/sapid/period-picker-browser-web31.json`. The sequential runner log
is `dates-browser-web31.log` (copied as `period-picker-browser-web31.log`).
