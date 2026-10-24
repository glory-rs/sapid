# DateInput

Enable the optional `dates` feature. `date_input(key, provider)` uses one real
editable Input for civil dates and, by default, a separate named Button opening
the shared host Dialog/Calendar. It has no browser clock, transport, IANA/DST,
serde or parser dependency. `.with_picker(false)` keeps only the editable control.

```rust
use sapid::prelude::*;
let provider = DatesProvider::new(CivilDate::new(2024, 2, 29)?);
let value = scope.cage(None);
date_input("appointment", provider)
    .value(value)
    .on_value_change(move |date| value.revise(|old| *old = date));
```

The committed value is `Option<CivilDate>`. A draft has bounded text and a
separate `DateDraftStatus`: Empty, Incomplete, Valid, Invalid, OutOfBounds or
TooLong. Empty proposes `None`; a valid in-bounds date proposes `Some(date)`.
An incomplete, invalid or outside-bounds draft remains visible and leaves the
last committed business value unchanged. Outside dates are diagnosed, never
implicitly clamped. Explicit commit of an invalid/incomplete draft calls
`.on_reject` and keeps the text. Field validation can separately require a
complete draft before the application submits it.

The default parser accepts exact ASCII `YYYY-MM-DD`. Its finite prefixes such as
`2024-` are incomplete. It does not trim whitespace, infer a current year or
reinterpret invalid leap dates. `.parser(Fn(&str) -> DateParse)` may additionally
accept application spellings. `DateParse::{Empty, Incomplete, Date, Invalid}`
contains no unbounded diagnostics; bounds are checked independently after the
application parser. Text is limited to **64 UTF-8 bytes**, and the host uses the
opt-in InputEdit capture budget before creating an application String. Oversize
captures restore the accepted ISO value and emit a small TooLong diagnostic
without calling the parser or faulting the App.

Custom valid spellings remain as native text while editing. Explicit commit
normalizes them to ISO. Controlled values are read-only inputs: callbacks propose
civil dates, and a refused valid proposal reconciles to the accepted ISO value
at the end of the host tick. Invalid/intermediate text remains an independent
draft. The single pending draft records its current business baseline so a
same-tick valid edit followed by an incomplete edit does not lose that final
text. A later external business value wins silently and resets the display.
No visited-value or edit history is retained.

`.on_draft_change(DateInputDraft)` observes bounded text plus its status;
`.on_value_change(Option<CivilDate>)` observes valid/empty proposals;
`.on_commit(Option<CivilDate>)` observes valid explicit commits;
`.on_reject(DateDraftStatus)` observes rejected commits or host captures. Internal
state and these typed callbacks precede public generic InputEdit/Commit observers.
The opt-in control does not separately deliver legacy ValueChanged callbacks.
Ordinary Form Enter remains available after the Input commit protocol.

Composition/preedit is owned by the host. It is not parsed or formatted until a
terminal noncomposing capture. One current revision/text echo rejects the native
terminal duplicate. Program changes during composition publish a new program
revision, so an old terminal cannot overwrite the new value. This is protocol
evidence, not a claim about actual OS IME acceptance.

The ordinary theme/size/unstyled/style/class/name/enabled/focus/node_ref and
Field APIs apply to the real editor. `.required`, `.readonly`, `.invalid` and
`.form_name` are explicit. The Field's label, description, error, required,
readonly and disabled state are retained. Invalid/outside/oversize drafts attach
a stable actual error target when the Field has no prior error target. The
native successful value is the displayed draft; an accepted explicit commit
uses ISO. Applications that serialize the committed model can do so explicitly
through FormState and validate the independent draft status. No default HTTP
submission or routing is installed.

The optional calendar shares the same committed value and provider bounds.
`.picker_label`, `.picker_title` and `.picker_close_label` configure its names.
The host manages focus containment, safe initial close focus, Escape, outside
click and return focus. Choosing an accepted date updates the same editor and
closes; controlled refusal leaves the popup open. The panel is kept mounted.
The namespace is explicit stable ASCII, at most 57 bytes. Owner retirement drops
state, pending text, callback captures, refs and the physical Layer. Default
native Dialog support is used; it is not replaced by a hidden fallback.

## Evidence

- Three pure bounded draft/parser tests in `src/date_draft.rs` passed:
  prefixes/leap syntax, independent bounds/empty proposals, and a million-byte
  rejection before the custom parser. `D:/BuildCache/sapid/date-draft-rust.log`.
- Eleven component tests in `tests/date_inputs.rs` passed: intermediate/invalid/
  valid/empty text, bounds and rejected commit, custom spelling and callback
  order, same-tick controlled refusal, nonfault oversize rejection, composition
  echo and superseded revision, silent external recovery, readonly/disabled/
  Owner cleanup, Field and one-editor default Dialog, and same-tick valid then
  incomplete draft retention, and Calendar draft notification before its civil
  value callback, and direct million-byte Commit rejection before copying the
  pending draft. `D:/BuildCache/sapid/time-date-input-rust.log` (also runs TimeInput).
- Two native component tests passed, including four-theme actual editor text,
  native selection/caret query, bounds/hit/raster, stable engine identity,
  default physical Dialog selection/focus return, readonly/disabled and small
  unstyled geometry. `D:/BuildCache/sapid/date-input-native.log`.
- Latest dates lib/tests, WASM fixture and native focused strict Clippy passed:
  `date-input-clippy.log`, `date-input-web-clippy.log`,
  `date-input-native-clippy.log` under `D:/BuildCache/sapid/`.
- Fixed FORMAT-9 Web29 passed the original **12/12** browser groups, errors
  `[]`, on Chromium 148.0.7778.96: four-theme CSR/hydration, input/commit/Form,
  Calendar, custom parser, same-tick/budget, composition/revision/Owner, dirty
  takeover and touch. `D:/BuildCache/sapid/date-input-browser-web29.json` and
  `date-input-browser-web29.log`. DatePicker also re-passed 12/12 against this
  artifact (`date-picker-browser-web29.json`). Blue light CSR and green dark CSR
  screenshots were visually inspected. That inspection found a Calendar-choice
  draft callback omission: the internal draft became Valid but the application
  could retain its previous draft diagnostic. The source emits the selected
  ISO/Valid draft before its civil value callback, with the tenth component
  regression. The strengthened original **12/12** suite then passed against
  fixed FORMAT-9 Web30, errors `[]`, on the same Chromium version:
  `D:/BuildCache/sapid/date-input-browser-web30.json` and
  `date-input-browser-web30.log`. This includes an explicit published Valid
  draft check after Calendar selection. DatePicker also re-passed 12/12 against
  Web30 (`date-picker-browser-web30.json`).

Native tests drive the widget's typed edit protocol and inspect actual host
editor/layout/focus/hit/pixels. Shared native InputEdit key/IME driver evidence is
recorded with MaskInput. These tests do not establish OS IME or screen-reader
announcement acceptance. DateInput never silently turns a civil date into UTC.

The original strengthened browser suite passed again **12/12**, errors `[]`,
against fixed FORMAT-10 Web31 on Chromium 148.0.7778.96:
`D:/BuildCache/sapid/date-input-browser-web31.json`. The raw sequential runner
log is `dates-browser-web31.log` (copied as `date-input-browser-web31.log`). The
new direct application million-byte Commit guard is proved by the eleventh Rust
component test; the browser fixture does not expose a direct Glory Commit probe.
