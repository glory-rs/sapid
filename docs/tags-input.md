# TagsInput and PillsInput

P4-09 has model, shared protocol, native refusal and actual browser evidence.
The final browser report uses the unified root Web16 snapshot; the earlier
isolated `tags-next` report remains useful diagnostic evidence.

Both controls use a real ordinary single-line Input and dynamic removable tag
views. PillsInput changes the tag shape; it shares the controller and successful
value behavior. Neither control invents a Combobox/Listbox role or a popup when
there is no suggestion list. Left/Right, Home/End, Space and Tab remain native
editor behavior. Empty Enter retains native form submission. Configured nonempty
Enter/comma/semicolon commits and empty Backspace removal use an explicit finite
host policy. Composition suspends tokenization and reconciliation of preedit.

The public controlled state is deliberately atomic:

```rust
let state = scope.cage(sapid::TagsInputState {
    values: vec!["Rust".into()],
    search: String::new(),
});
sapid::tags_input("skills")
    .state(state)
    .on_change(move |proposal| state.revise(|value| *value = proposal));
```

`on_change(TagsInputState)` proposes the complete pair. Supplied sources are
read-only to the widget. `default_value(Vec<String>)` and `default_search` supply
local initial state; they never take ownership away from an explicit source.
`on_commit(Vec<String>)` observes the newly accepted tag batch and
`on_remove(String)` observes accepted removal. They run after `on_change` only
when the source equals the whole proposal and its Owner remains live. A source
refusal keeps both values and draft, emits no accepted observer, and preserves
the actual editor/caret. No application callback runs in a pure Binding.

`duplicate_policy` is `Reject` (default), `Ignore`, or `Allow`, comparing complete
case-sensitive values. Insertion trims surrounding Unicode whitespace and skips
empty tokens. Allow retains duplicate occurrences with distinct live keyed rows;
removal rechecks the current value/occurrence before proposing a change. External
state must already satisfy the selected policy. `max_tags` is 1–256. Rejected
user batches report `TagRejection` through `on_reject` without publishing earlier
tokens or clearing the draft. A rejected later pasted token therefore cannot
partially accept an earlier token.

`separators(glory::render::TagSeparators)` selects Enter alone, Enter/comma, or
Enter/comma/semicolon. Pasted line breaks also separate tags. Delimited paste
replaces the current UTF-16 selection, then tokenizes the complete resulting
draft as one proposal. Ordinary non-delimited paste stays native editing. The
shared payload validation rejects unordered/out-of-range selections and offsets
inside a surrogate pair. Host clipboard admission checks JS UTF-16 length before
Rust UTF-8 copying; the conversion itself is consequently bounded. Clipboard
budget refusal emits a body-free finite rejection, preserving the accepted state
and editor selection. A subsequent admitted paste remains usable; the rejected
body never enters Rust component state or a persistent host fault.

Limits are 256 accepted tags, 256 UTF-8 bytes per tag, 64 KiB total accepted tag
text, 4096 bytes of canonical single-line draft, and 16 KiB per clipboard body.
The Source Vec length and string lengths are checked by borrowing before cloning
accepted state. Dynamic rows retain current values/occurrences only, with no
visited values or external source scan. The 256-tag model stays below 1100 live
semantic nodes. Callback captures, rows, editor references and proxy Owners
retire with the component Owner, and destructors run outside graph borrows.

FormData uses one real native Select with `multiple`, containing each accepted
free business value and selecting all of them. There is no known-option-key
requirement. The hidden proxy is rebuilt when tag values change; the real editor
retains its identity. Editing the draft alone keeps the same proxy. Readonly
keeps successful values; disabled omits them. The visible search is unnamed.
`form_name` or Field supplies the successful name. Required tags are validated by
Field/FormState against `state.values`; the focusable Input exposes ARIA
required/error metadata, while neither empty draft nor hidden proxy has native
`required`.

Glory's finite `Semantics.tags_input`, `TagSeparators`, `Event::Tags` and
`TagsCommand` describe only this opted-in editor. Ordinary Inputs remain
unaffected. Exact origin, Input revision, finite text and UTF-16 offsets are
shared validation. App ignores old revisions, and all same-revision edits and
tag commits in one drained host tick run before dirty reconciliation. HTML's
prepared-field lowering retains its full-shape difference oracle and keeps
value/revision untouched on policy changes. This batch used HTML v4/classic
stream v2 without new edit operations. Current producers follow HTML v13 with
the classic v2 envelope; the original snapshot evidence below retains its version.

Blitz explicitly rejects tags policy in descriptor preflight and direct apply
before registering facets. Plain native Inputs remain supported. The rejection
test preserves successful-prefix query watermarks. No native tags input or AX
support, OS IME, or screen-reader acceptance is claimed.

Current evidence:

- `src/tags_input.rs`: 12 focused model tests, including atomic refusal/observer
  order, duplicate modes, UTF-16 replacement, whole-batch rejection, composition,
  readonly/disabled, same-tick revisions, huge sources, max live rows, successful
  values, Pills appearance, stable editor/proxy identities, body-free host budget
  rejection/recovery and Owner retirement.
- Glory `render/tests/tags.rs`: three payload/policy/routing/rollback tests.
  `html/src/tags_tests.rs`: prepared-field/full-shape oracle, required/default
  resets and preservation of a large unrelated editor value/revision.
- Glory `blitz/tests/tags.rs`: one passing explicit refusal/plain Input/prefix/query
  test, run against the verified native D-drive target.
- Sapid library/tests and current WASM fixture compile with strict Clippy.
  `examples/web/src/tags_form.rs`, `tags.html` and `tags-acceptance.js` are prepared
  for four-theme CSR/hydration, real clipboard, IME protocol, pointer/touch,
  FormData, Field validation, dirty adoption and Owner acceptance.
- `D:/BuildCache/sapid/tags-browser-tags-next.json`: Chromium 148.0.7778.96,
  14 actual browser groups passed, `errors: []`. Eight groups cover the four
  themes in CSR/hydration. Independent groups cover real clipboard and UTF-16
  replacement, a million-character clipboard rejection followed by successful
  recovery, browser IME protocol, dirty adoption with focus/caret, touch removal,
  and classic stream plus dirty pre-WASM capture. The green-dark hydration
  screenshot was visually inspected. This does not establish OS IME or actual
  screen-reader acceptance.
- `D:/BuildCache/sapid/tags-browser-web16.json`: the same 14 groups passed again
  against the final unified Web16 WASM/SSR/classic-stream snapshot, `errors: []`.
