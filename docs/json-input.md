# JsonInput

Enable `sapid = { ..., features = ["json"] }` to use `json_input`, `validate_json`
and `format_json`. The default dependency graph has no JSON parser or serde.
The optional parser is pinned to serde_json 1.0.154 with borrowed raw values.

```rust
use sapid::{field, json_input};

field("payload")
    .label("Payload")
    .description("JSON request document")
    .form_name("payload")
    .control(|field| json_input("payload").field(field).default_value("{}"));
```

The editor is a real multiline Textarea. Syntax errors remain editable drafts,
with a separate bounded message linked to the editor. Field labels, descriptions,
required/readonly/disabled states, successful-control names and external errors
remain on the editor. An external Field error takes precedence in the error
relation. Parsing and metadata updates preserve editor identity, value revisions,
selection and composition. Error announcements are opt-in with
`.announce_errors(true)`; this does not establish screen-reader acceptance.

`.value(Vary<String>)` is controlled: `.on_value_change` proposes edits and
formatting; only the application updates that source. `.default_value` uses local
state. Typed change callbacks precede generic input observers. `.on_format`
observes a successful formatting proposal, including a proposal the controlled
application refuses. `.on_format_error` observes bounded-output failures; those
failures preserve the valid original draft and do not mark its syntax invalid.
The next accepted draft clears that formatting message. `.on_validation` emits
changes between Empty, Valid(kind), and Invalid(error), including initial state.

The optional formatter uses PrettyTwo, PrettyFour or Compact whitespace. It
retains key order, duplicate members, string escape spelling and exact number
lexemes, including integers larger than JavaScript's safe range and `1e999`.
It validates borrowed raw syntax before editing whitespace and never decodes a
business object into a floating-point or map representation. It is a bounded
JSON text editor, with no schema validator, syntax highlighting or AST history.

`JsonLimits` defaults to 65,536 UTF-8 bytes and depth 64. Applications may lower
both limits; maximum supported depth is 128. Byte checks precede source cloning;
a quote/escape-aware depth scan precedes parsing. Output growth is checked at
every append. Malformed non-JSON whitespace remains a syntax error, while an
empty or JSON-whitespace-only draft is Empty. Component values use Glory's
canonical LF/U+FFFD text rules; program values containing CR/NUL are refused.
Invalid program updates preserve the last accepted editor state and may recover
with a later valid bounded source. Limits bound payloads rather than process RSS.

Formatting is unavailable for empty/invalid drafts, disabled/readonly Fields and
active composition. The event callback rechecks the current composition state,
including events queued before a visual update. Default fixed rows work on Web
and Blitz. Explicit `.style` overrides apply to the editor and preserve absent
theme declarations. `.unstyled()` retains behavior and removes theme decoration.
All callbacks, references and validation Effects retire with the component Owner.

Rust evidence: `tests/json_model.rs` (4 tests) covers lossless tokens, nesting,
input/output budgets and malformed syntax; `tests/json_input.rs` (7 tests)
covers drafts, controlled refusal/acceptance, callback ordering, composition,
Field metadata, output failure recovery, oversized program sources, SSR escaping
and Owner cleanup. Browser acceptance lives in
`examples/web/json-input-acceptance.js`; native acceptance lives in
`examples/native/tests/json_input.rs`. Host results are recorded in the widget
inventory only after executing the corresponding built artifact.

The fixed Web16 artifact passed 9 actual Chromium 148.0.7778.96 cases: four
themes × CSR/hydration and a separate dirty-editor adoption case, with no
console/page errors. The matrix exercises real keyboard formatter activation,
controlled refusal, syntax/output failure recovery, Field errors without caret
reset, composition protocol, actual multiline AX, readonly/disabled, mobile
width and Owner retirement. Results: `D:/BuildCache/sapid/json-browser-web16.json`.
Native four-theme real layout, focus, formatting, error updates and engine
identity passed in `D:/BuildCache/sapid/json-native.log`. These checks do not
establish OS IME or screen-reader announcements.
