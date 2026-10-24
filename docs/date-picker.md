# DatePicker / DatePickerInput

Enable Sapid's optional `dates` feature. It adds no dependency. Dates are civil
Gregorian values; providers explicitly supply the reference date, optional today,
locale, week start and bounds. No browser clock, timezone, HTTP or serialization
framework is inferred.

```rust
use sapid::prelude::*;
let provider = DatesProvider::new(CivilDate::new(2024, 2, 29)?);
let selection = scope.cage(DateSelection::empty(DatePickerMode::Range));
date_picker("travel", provider)
    .mode(DatePickerMode::Range)
    .value(selection)
    .on_value_change(move |next| selection.revise(|old| *old = next));
```

`DatePickerMode::{Single, Multiple, Range}` is fixed per builder. The controlled
`DateSelection` must retain that mode. Defaults become local state only when no
controlled source exists. `.month` and `.on_month_change` separately control the
visible month. `.default_month`, `.direction`, `.disable_weekday`, `.exclude_date`
and `.exclude_dates` have the Calendar policies. `.allow_deselect(true)` enables
single-date clearing; multiple selection always toggles the activated day.

`SelectedDates` is a sorted unique set in a private fixed array with capacity 32.
Iterator admission consumes at most 33 items, including duplicate inputs.
`DateSelection::Multiple` stores this finite value rather than a `Vec`.
Activating a 33rd distinct date calls `.on_reject(DateSelectionError::TooManyDates)`
and preserves the accepted set without faulting the App. Invalid external mode
or bounds changes reject and keep the previous rendered selection until recovery.

Ranges have explicit `DateRangeSelection::{Empty, Start, Complete}` states.
The first activation starts a range; the second orders both endpoints; activating
a completed range starts another. A completed inclusive range uses constant-time
membership tests and stores two dates. Endpoints must be selectable. **Disabled
interior dates are allowed and remain selected but disabled**; no hidden full-span
scan is performed. Bounds validate both endpoints.

The inline component reuses Calendar's real 42 GridCells and real day Buttons.
Cell selection, physical focus and `Current::Date` remain independent. Multiple
and range modes set typed Grid multiselection, mapped to HTML
`aria-multiselectable` and AccessKit's multiselectable flag. It does not make a
Button into a selected option. Arrow, Home/End and Page/Shift-Page navigation keep
the Calendar protocol. Same-tick activations compose against one live proposal
cursor; no action history is retained. Typed callbacks run before generic observers.

## Input and popup

`date_picker_input(key, provider)` displays the exact ISO selection in a real
readonly text Input with an independent, named Dialog trigger. It supports the
same three modes, Field relationships, required/invalid/disabled state,
controlled/local selection, controlled/local `.open`, `.on_open_change`, and
`.trigger_ref`, `.panel_ref`, `.calendar_ref` plus the ordinary Input `.node_ref`.
Keys are stable ASCII namespaces of at most 57 bytes; the derived Dialog key
must fit the common 64-byte namespace budget.

The readonly display remains selectable and successful. `.readonly(true)` also
locks calendar selection while keeping navigation available. This picker does
not parse arbitrary typed dates; editable date drafts belong to DateInput.
A `Field` supplies its accessible label/description/error and successful name;
`.form_name` explicitly overrides only the successful name. No ComboBox/Listbox
role is fabricated for the display.

Successful values use an explicit locale-independent spelling:

| Mode | Empty | Value |
| --- | --- | --- |
| Single | empty string | `2024-02-29` |
| Multiple | empty string | sorted `2024-02-26,2024-02-29` |
| Range | empty string | partial `2024-02-26/`, complete `2024-02-26/2024-02-29` |

`DateSelection::format_iso` and `parse_iso` implement this bounded contract. A
partial range is distinguishable from an empty range; application validation can
require `is_complete()`. Submission and routing remain application decisions.

The shared host Layer owns modal focus containment, Escape, outside dismissal
and return focus. The explicit initial target is the named close Button. The
panel shell and calendar are kept mounted so closing preserves month and child
identities. A successfully accepted complete single/range selection proposes
closing; multiple selection stays open. Selection refusal leaves the dialog open.
Open refusal is independent from selection refusal. Retiring the Owner removes
all callbacks, references and the physical Layer. The Native host supports this
default Dialog and is tested; no unsupported-layer fallback is substituted.

## Evidence

- Five pure selection-model tests: fixed iterator admission and sorted unique
  membership, capacity rejection, partial/reverse/restarted ranges, exact ISO and
  mode/bounds. `D:/BuildCache/sapid/date-selection-rust.log`.
- Nine component tests in `tests/date_pickers.rs`: same-tick multiple selection,
  typed-before-generic ordering, disabled interior policy, controlled refusal and
  invalid recovery, 33rd-date nonfault rejection, readonly/disabled/Owner cleanup,
  real readonly Input/Dialog, stable shells, Field relationships, open refusal,
  admission budgets, and explicit range/multiple Input open/text policies. Calendar's six existing tests pass unchanged.
  `D:/BuildCache/sapid/date-picker-rust-final.log` (Calendar regression in
  `date-picker-rust.log`).
- Shared Grid target/rollback tests, all eighteen HTML tests including field-delta
  oracle and reset cases, and native AX multiselection on/off passed:
  `date-grid-rust.log`, `date-grid-html.log`, `date-grid-native-ax.log` in the same
  build root.
- Three actual native component tests in `examples/native/tests/date_picker.rs`
  passed. Four themes open the default physical Dialog, check top-layer entries,
  actual day bounds/hit/selected pixels, real focus, Grid navigation, accepted
  value/close/return focus, stable Input engine identity, reopen/Escape and Owner
  cleanup. The second checks range, disabled interior, readonly and disabled
  rejection; the third checks an initially open Range Input, partial text,
  completed-range close and focus return.
  `D:/BuildCache/sapid/date-picker-native-final.log`.
- Sapid lib/components, Web WASM and native focused strict Clippy passed:
  `date-input-clippy.log` (includes the final DatePicker tests), `date-picker-web-clippy.log`,
  `date-input-native-clippy.log` (includes final DatePicker native tests). SSR bin check passed.
- The original browser suite passed **12/12**, errors `[]`, with Chromium
  **148.0.7778.96** against fixed FORMAT-8 Web28 at
  `http://127.0.0.1:8794/web28`. Eight theme/CSR/hydration combinations cover
  native keyboard focus, Tab/Escape/return, readonly Field and successful FormData,
  Form Enter, controlled refusal, range/multiple selection, disabled/readonly,
  real browser Dialog/Grid/Button AX tree and stable input/draft caret. Four
  additional cases cover same-tick multi/range actions, 33rd-date nonfault
  rejection, controlled open/Owner cleanup and remount, dirty SSR takeover, and
  actual touch popup/range. Report:
  `D:/BuildCache/sapid/date-picker-browser-web28.json`; log:
  `D:/BuildCache/sapid/date-picker-browser-web28.log`.
  The initial script incorrectly tried clicking an inert trigger after a
  controlled-open Owner remounted; its failure is retained in
  `date-picker-browser-web28-first.log`. The final assertion instead requires
  that the fresh Owner honors the still-true open source, then closes by Escape.
  No production change or forced pointer action was needed. Screenshots are
  `date-picker-{hue}-{scheme}-{csr|hydrate}.png`; blue light CSR and green dark
  CSR were visually inspected.

The same original 12/12 suite also passed against FORMAT-9 Web29, errors `[]`,
with Chromium 148.0.7778.96: `date-picker-browser-web29.json` and
`date-picker-browser-web29.log` under the D build root.

Native component tests drive typed host commands and query physical layout,
focus, hit testing and raster output. They do not establish OS keyboard/IME or
screen-reader announcement acceptance. IANA/DST and absolute instant conversion
remain separate from civil date selection.
