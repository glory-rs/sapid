# DateTimePicker / InlineDateTimePicker

Enable the dependency-free optional `dates` feature. Both builders require an
explicit `DateTimePolicy`. `LocalDateTime` is a civil date plus wall-clock time;
it has no implicit UTC interpretation, timezone lookup or operating-system clock.

```rust
let value = scope.cage(None::<LocalDateTime>);
date_time_picker(
    "appointment",
    DatesProvider::new(CivilDate::new(2024, 2, 29)?),
    DateTimePolicy::FixedOffset(FixedOffset::new(480)?), // minutes, UTC+08:00
)
.value(value)
.form_name("appointment")
.on_value_change(move |next| value.revise(|old| *old = next));
```

`inline_date_time_picker` composes the same real Calendar Grid and TimePicker
spinbutton editors without a popup. Its `node_ref` identifies the named Group;
`calendar_ref` and hour/minute/second refs identify the actual controls. The popup
builder's `node_ref` identifies its readonly display Input; separate trigger and
panel refs identify the real Button and host-owned Dialog. Field labels target
the display Input, or the first hour editor for the inline builder. Accessible
names and explicit successful-control names remain independent.

`DateTimeParts` and `DateTimeDraft` retain an independent bounded draft. A date
without a complete time, or a time without a date, is Incomplete. No midnight or
reference date is filled in. Complete valid pairs propose a business value in
real time. Incomplete, invalid, outside-bounds, ambiguous and gap drafts preserve
the previous complete business value. Clear explicitly proposes `None`. A
controlled refusal restores the accepted date and every segment; a Clear on an
already empty business source also clears partially entered raw segments.
`reset_revision` explicitly resets equal-value drafts and reevaluates application
resolution rules; applications must advance this counter without wrapping.
External accepted source updates and resets remain silent.

`on_draft_change` precedes `on_value_change`. `on_resolved` runs only when the
whole value proposal has actually been accepted. `on_commit` belongs to Done;
an unresolved draft instead calls `on_reject` and keeps the popup open. Done
does not initiate HTTP or submit a containing form. Escape, Close and outside
dismissal retain already accepted real-time edits; they do not roll them back.
`ContentPolicy::KeepMounted` is the popup default and retains unfinished drafts.
`Unmount` retires the child Owner and rebuilds from the accepted value on reopen.

The popup Input's successful string is exact civil ISO, for example
`2024-02-29T10:20:00`, with no `Z` or offset suffix. During a partial/gap/fold
draft that Input still contains the last accepted complete ISO value. Therefore
an application that requires a complete current draft must validate
`DateTimeDraft.status` before accepting a form submission; a successful old
control value alone is not proof that the current draft is valid. Enter in a
segment follows NumberInput's normal Commit then ancestor Form Submit ordering.
There is no nested Form or special submit interception. The inline builder adds
no synthetic hidden aggregate input; connect its typed value explicitly to the
application's FormState or transport.

`DateTimePolicy::Civil` returns a civil outcome without an instant.
`FixedOffset` uses an explicit offset in minutes. `Application` invokes the
application's synchronous rules and accepts Unique, Gap or two chronologically
ordered matching candidates. `AmbiguityPolicy::{Reject,Earlier,Later}` controls
fold resolution. Wrong candidates or reversed fold order produce InvalidResolver.
There is no installed IANA/DST database and no claim that a fixed offset models
seasonal rules. Resolvers run outside reactive-value and registry borrows.
Owner/epoch/source checks reject results superseded by reentrant application
code; no resolution history or async task is retained.

`DateTimeBounds` orders complete civil values and intersects the supplied
DatesProvider date bounds. Selecting an edge date checks its actual combined
time; it never silently clamps an editor or faults the child because the
previous day's time is outside that edge interval. Program sources outside the
accepted interval or the explicit TimeFormat precision are rejected before child
updates. TimeFormat retains the TimePicker 12/24-hour, Chinese/English and
minute/second precision rules; fractional seconds and hidden nonzero seconds
are rejected rather than truncated. Calendar always has 42 current cells and
TimeGrid remains independently capped at 288 slots. Keys are limited to 48 bytes
inline and 40 for the popup. All draft/controller state belongs to the live Owner.

Current verification:

- Six pure policy/ISO/bounds tests: `D:/BuildCache/sapid/date-time-model-rust.log`.
- Eleven component tests plus all eight TimePicker regressions:
  `D:/BuildCache/sapid/date-time-rust.log`. They cover complete/partial drafts,
  same-tick date and segment edits, equal-value reset, controlled Clear refusal,
  gap/fold/Done rejection, edge bounds, resolver and draft-callback reentry,
  source recovery, Field ISO metadata, Keep/Unmount, and resolver Owner disposal.
- Two actual native tests, including four themes:
  `D:/BuildCache/sapid/date-time-native.log`. Real Dialog layout, Calendar hit
  testing/selected accent pixels, real spinbutton geometry/Spin, focus return,
  stable display identity, gap rejection, readonly and Unmount cleanup pass.
- WASM strict, component/lib strict and SSR check pass in
  `date-time-{web-clippy,clippy,ssr-check}.log` under the same D-drive build root.

The original twelve-case browser suite passed **12/12**, errors `[]`, on Chromium
**148.0.7778.96**, fixed FORMAT-10 artifact
`http://127.0.0.1:8794/web34`, from the immutable source copy
`D:/BuildCache/sapid/web34-source`. Report:
`D:/BuildCache/sapid/date-time-picker-browser-web34.json`; sequential raw log
`D:/BuildCache/sapid/dates-browser-web34.log` also contains TimePicker 12/12 and
TimeInput 12/12 regressions. No original browser assertions were changed.
Eight theme/CSR/hydration cases cover real Calendar/Spin/Dialog and Field,
explicit ISO/offset, partial/gap/fold behavior, Done rejection, controlled Clear
refusal, disabled/readonly, Form Enter ordering, Done/Escape focus, equal-value
draft reset, AX and stable identity/selection, and unstyled geometry. Four extra
cases cover same-tick date+segments/budget, composition/Owner, dirty hydration and
touch. Blue-light CSR, green-dark hydration and the open Dialog screenshots were
visually inspected (`date-time-picker-*.png`, `date-time-picker-open-web34.png`).

After that immutable snapshot, the controller gained one additional regression:
resolver/draft callbacks that change only reset configuration cannot publish an
obsolete proposal. All twelve component tests plus eight TimePicker regressions
pass in `date-time-config-rust.log`; strict passes in `date-time-config-clippy.log`.
The guard is now included in fixed FORMAT-10 Web35, rebuilt from
`D:/BuildCache/sapid/web35-source`. The snapshot's original twelve-case browser
suite again passes 12/12 with errors `[]` on Chromium 148.0.7778.96:
`date-time-picker-browser-web35.json`; the shared sequential log
`schedule-date-time-browser-web35.log` also contains the independent Agenda suite.
Protocol/driver tests do not prove real OS IME or screen-reader announcements.

The same original twelve-case suite passes against immutable FORMAT-11 Web36,
Chromium 148.0.7778.96, errors `[]`: `date-time-picker-browser-web36.json`.
`schedule-date-time-browser-web36.log` also records independent Agenda12 and
TimePicker12 regressions from that fixed artifact.
