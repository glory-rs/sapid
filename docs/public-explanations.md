# Public explanations

The optional `chat` feature exports `reasoning` and `chain_of_thought`. Both
display a supplied `PublicExplanation` through the existing real Collapsible.
`reasoning` uses unordered steps; `chain_of_thought` uses an intrinsic ordered
list. They have no model service, inference callback or access to hidden model
reasoning. The application supplies text that is already appropriate to show.

```rust
use sapid::chat::*;
let view = reasoning("result").data(PublicExplanation {
    summary: "About this result".into(),
    body: "A public summary supplied by the application.".into(),
    steps: vec![ExplanationStep::new("review", "Review", "Public status")],
});
```

Summary labels are nonblank and at most 256 UTF-8 bytes, body text at most
16384 bytes, and each step has a distinct 1–64 byte ASCII key, a 256-byte label
and an 8192-byte summary. At most 64 current steps and 65536 total text/key bytes
are admitted. `ExplanationStatus` is explicit Pending/Active/Complete/Failed;
optional elapsed time is supplied in milliseconds, at most seven days. No
clock or completion status is inferred. Canonical text rejects NUL and CR.
Validation borrows the whole finite candidate before accepting its clone;
invalid reactive data retains the preceding complete display.

The disclosure defaults closed and unmounts its content. `open` is controlled,
`default_open` supplies local initial state, and `on_open_change` proposes a
change which the application may refuse. `content_policy` explicitly opts into
retained hidden content. The trigger is a genuine named button with expanded
and controls relations. Full step keys preserve live identities on updates.
Source changes are silent; Owner removal retires content, observers and
callbacks. `theme`, `size` and `unstyled` keep the existing finite appearance
rules; unstyled text still preserves whitespace and bounded overflow.

`tests/chat_evidence.rs` exercises both list forms, literal script-looking text,
elapsed/status data, controlled refusal, keyed updates, all-or-nothing recovery,
budgets and Owner cleanup. `chat-evidence-tests.log` records seven combined
explanation/citation tests passing on the current shared implementation;
`chat-evidence-strict.log` denies warnings. `chat-evidence-native-tests.log`
contains the real four-theme WindowApp case for actual bounds, hit tests, focus,
disclosure, source updates and raster surfaces; focused strict checks are in
`chat-evidence-native-strict.log`. The temporary baseline-only development
log is explicitly not current integration evidence. Browser family96 uses
`public-explanations.html` and the original `chat-evidence-acceptance.js`;
immutable FORMAT12 Web40 passes all **20 combined original reports**, errors
`[]`, Chromium148.0.7778.96 (`chat-evidence-browser-web40.json`, raw
`chat-evidence-plots-browser-web40.log`). Four themes × CSR/adoption, actual
keyboard disclosure, literal markup, controlled refusal, keyed updates,
disabled and Owner lifecycle, two focused SSR controls and two mobile layouts
are covered. Mobile screenshots were visually inspected. Native
layout/protocol evidence does not establish screen-reader speech or OS IME.
