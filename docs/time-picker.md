# TimePicker / TimeGrid

Enable the optional dependency-free `dates` feature. `time_picker(key)` composes
real NumberInput spinbuttons for hour/minute and optional seconds. It stores one
typed `Option<LocalTime>` with a separate finite segment draft. No date, midnight
or timezone is fabricated while a segment is missing.

```rust
use sapid::prelude::*;
let time = scope.cage(Some(LocalTime::new(9, 30, 0)?));
let grid = TimeGridSpec::new(
    LocalTime::new(9, 0, 0)?, LocalTime::new(17, 0, 0)?, 1800, 4,
)?;
let picker = time_picker("appointment")
    .name("Appointment time")
    .value(time)
    .grid(grid)
    .on_value_change(move |next| time.revise(|old| *old = next));
# Ok::<(), sapid::DateError>(())
```

`TimeFormat` and `TimeBounds` have the same explicit precision/hour-cycle/locale
rules as TimeInput. Nanoseconds and seconds outside the selected precision are
rejected rather than truncated. Same-day bounds do not represent an overnight
interval. Twelve-hour mode uses an actual pressed `Afternoon (PM)` toggle
(`下午` in Chinese); false means AM. Midnight and noon both use hour 12, with
different period state. This toggle is not labelled as a radio button.

`TimeParts` holds optional hour/minute/second and the explicit period flag.
`TimePickerDraft` distinguishes Empty (all displayed segments empty), Incomplete,
Valid, Invalid, OutOfBounds, Unavailable and TooLong. Partial or invalid/outside
parts retain the last complete business time. Only a complete valid time or all
empty segments propose a new value. `on_draft_change` runs before the typed value
callback. The controller stores at most two ASCII digits per numeric segment,
checks proposals before copying, and uses NumberInput's bounded input protocol.
Rejected oversized segments restore the accepted time; no whole-day source scan
or input history is retained. Composition and caret remain owned by the actual
editors, with NumberInput's revision and terminal echo guards.

`hour_step`, `minute_step`, `second_step` default to one; admission requires
1–24 for hour and 1–59 for minute/second. They lower to real SpinValue step
semantics. Page keys use ten steps. Tab moves among separate native editors;
left/right move within the native caret. There is no implicit focus auto-advance.
An explicit segment commit pads valid numeric text to two digits. NumberInput
clamps its own numeric hour/minute/second domain on explicit commit; the aggregate
TimeBounds or unavailable-time policy does not clamp the resulting wall clock.
An invalid/outside/unavailable aggregate commit emits `on_reject`.

Controlled `value` is read-only and accepted only by the callback. Refusal restores
all accepted parts after the host tick. Invalid external precision/bounds source
updates return a typed flush error and keep accepted UI, with silent valid recovery.
`readonly` preserves focus/caret but forbids spin/period/Grid proposals; `enabled`
uses actual disabled Fieldsets for the editors. All refs, effects, callbacks and
grid members retire with their Owner. Common `node_ref` refers to the named Group;
`hour_ref`, `minute_ref`, `second_ref` refer to actual editors.

Field label/description names the containing Group, and its label targets the first
real hour Input. Required, readonly and invalid state reaches each editor; the
Field's own error relation stays on the associated first control. Explicit
`.form_name("appointment")` produces successful `appointment_hour`,
`appointment_minute`, and optional `appointment_second` numeric controls. The base
is limited to 240 bytes, before cloning, to leave room for the suffixes. There is
no hidden combined field: applications requiring canonical ISO transport should
serialize the typed `LocalTime` and validate the independent draft in the Form
submit callback. Ordinary Enter commits the segment before Form submission.

The optional grid uses `time_grid(key)` with the same TimeGridSpec. It is an actual
Grid/Row/GridCell/Button tree, and selected state belongs to the cell. Grid and
segments share the same accepted time. A valid time between grid slots remains
valid and selects no cell; the component does not snap it to the visual sampling.
The default spec is 00:00–23:30 at 30-minute steps in four columns. A spec has an
inclusive upper bound; if the step does not divide the interval, no extra final
endpoint is invented. It can contain real seconds when the format includes them.

Spec construction computes the slot count in O(1) before allocating anything and
rejects more than **288 slots**, step 0/>86400, fractional nanoseconds, reversed
same-day endpoints or column counts outside 1–12. At most 128 disabled times are
admitted, with a 129th iterator sentinel; a million-item lazy iterator is not fully
consumed. Disabled exact times deny user segment proposals and Grid endpoints;
programmatic values are still authoritative. There is no disabled-time predicate
scanning an unbounded source. The mounted cell/ref list is bounded by the current
spec and freed on Owner retirement.

Grid arrows rove actual focus and skip disabled slots. Home/End stay in a row;
Page keys move four rows; Shift+Page jumps to the first/last allowed slot. RTL
reverses horizontal commands and physical placement. Focus and selection remain
independent, and readonly retains roving focus. A single current cursor composes
several queued keys followed by activation in the same tick. No query, task,
timer, routing or transport is installed.

Validation completed before browser acceptance:

- `time_grid_model::tests`: two arithmetic admission/seconds/endpoint tests.
- `time_parts::tests`: two tests, including every second of a day in both cycles.
- Six TimeGrid and eight TimePicker component tests pass:
  `D:/BuildCache/sapid/time-picker-rust-final.log`; pure model logs are
  `time-grid-model-rust.log` and `time-parts-rust.log`.
- Two actual native tests pass (`time-picker-native.log`): four-theme physical
  spin editor geometry/focus, real Spin commands and shared Grid selection,
  real hit testing and selected accent pixels, stable editor identity; separate
  Grid disabled-skip navigation, readonly focus/selection and Owner cleanup.
- Sapid lib/component strict, WASM strict, native focused strict and SSR check
  pass in `time-picker-{clippy,web-clippy,native-clippy,ssr-check}.log`.

The joint TimePicker/TimeGrid browser suite passed **12/12**, errors `[]`, on
Chromium **148.0.7778.96** against fixed FORMAT-10
`http://127.0.0.1:8794/web33`. Production was built from the immutable source copy
`D:/BuildCache/sapid/web33-source`; later DateTime work did not enter that artifact.
Eight theme/CSR/hydration cases cover physical spinbutton/Field label/Tab, partial
and empty values, Enter padding, Spin keyboard, Form barrier and separate successful
names, controlled refusal, Grid/disabled/readonly/RTL, twelve-hour/Chinese period,
browser AX, stable caret and unstyled geometry. Four additional cases cover
same-tick three-segment edits and bounded rejection, same-tick Grid arrows plus
activation, composition/Owner, dirty SSR takeover and real touch slot/segment editing.

The initial test incorrectly treated Tab as a Commit and failed the padding
assertion. The shared host contract explicitly emits Commit on Enter, not focus
loss (`docs/number-input.md`). The corrected test retains the strong `03` padding
assertion after Enter and separately checks native Tab focus order. No production
change or forced action was needed. The failure is preserved in
`D:/BuildCache/sapid/time-browser-web33-first-tab-contract.log`; the final JSON
records this protocol correction:
`D:/BuildCache/sapid/time-picker-browser-web33.json`. The raw runner log is
`time-browser-web33.log` (also includes the independent TimeInput12 regression,
which passed errors `[]`). Screenshots are
`time-picker-{hue}-{scheme}-{csr|hydrate}.png`; blue light CSR and green dark
hydration were visually inspected.

Native typed commands/layout/pixel evidence does not establish OS IME or
screen-reader announcements.
