# MaskInput

`mask_input(MaskPattern)` builds one real single-line text input. Its public
`String` contains only the slot characters; the editor displays the formatted
value and owns its real caret, selection, focus, clipboard and preedit.

```rust
use sapid::prelude::*;

let reference = scope.cage(String::new());
let pattern = MaskPattern::parse("##-##")?;
let control = mask_input(pattern)
    .value(reference)
    .on_value_change(move |raw| reference.revise(|value| *value = raw))
    .on_commit(|raw| { /* validate or submit the raw application value */ })
    .on_reject(|reason| { /* a finite reason, without the rejected input */ });
```

Omitting `.value(...)` selects local state, initialized with `.default_value(...)`.
Controlled values remain read-only from the component's perspective: a callback
proposes a raw value, and the application explicitly accepts it. A refusal restores
the last accepted formatted value. External source changes are silent; an invalid
external source returns a typed UI error and preserves the accepted editor until a
valid value returns. Source validation borrows before cloning.

## Finite pattern and value contract

`#` accepts an ASCII digit, `A` an ASCII letter, and `*` an ASCII alphanumeric
character. Backslash escapes a literal, including `\A`, `\#`, `\*` and `\\`.
Literals must be printable ASCII. Patterns have at most 128 parts and 64 slots,
and require at least one slot. The parser checks the 256-byte source budget
before iteration or allocation. There is no regex engine or implicit locale.
Case and leading zeroes are preserved. Unicode digits, grapheme masking,
currency/date interpretation and arbitrary format functions are outside this API.

`MaskPattern::format(raw)` returns bounded `MaskedText` with `display()`,
`raw_index(display_index)` and `display_index(raw_index)`. Every index is both an
ASCII byte index and an exact UTF-16 index. Its `Debug` representation reports
lengths, without printing the value. Empty raw values display an empty editor;
partial nonempty values include literals up to the next unfilled slot. For
`(###) ###`, raw `123` displays `(123) `.

Whole unseparated raw paste takes precedence when it is itself valid. Otherwise
the parser consumes the pattern's explicit formatting literals. This matters for
digit or alphanumeric literals: `+1 (###) ###` accepts raw `123456` or formatted
`+1 (123) 456`. A wholly valid ambiguous raw value keeps all its characters.
Invalid or overlong input rejects the complete proposal; it never filters a bad
suffix, silently truncates a paste or inserts replacement business characters.

## Editing, composition and ordering

The opt-in `Semantics.input_edit` protocol captures the current program revision,
value, real UTF-16 selection and direction, the immediately preceding selection,
and a finite insert/delete/replace intent. One host edit follows `InputEdit`;
the host does not also publish the legacy `ValueChanged` path. Mask's internal
mapping and typed callback run before public `.on(EventKind::InputEdit, ...)`
observers. Other inputs keep their existing protocol.

An ordinary edit whose physical text is already canonical acknowledges the value
without assigning the native input text or resetting its caret. Formatting is
deferred until the drained host tick has completed. Only an actual formatting or
controlled restoration requests a revision-checked `SelectText` action. That
action is rejected for stale identity/revision/range, and superseded queued
selection actions cannot move a newer input's selection.

Backspace over a literal deletes the preceding raw slot; forward Delete over a
literal deletes the following raw slot. Selected ranges map to raw boundaries
before replacement. A single bounded current physical map handles consecutive
native edits before the formatted write: deleting the dash from `12-` and then
backspacing again in the same tick produces empty raw state. No previous-value
history, visited-input cache, query, task or tree scan is retained.

During composition, the host retains preedit and Mask does not format or publish
a business value. The terminal edit maps once, with a bounded current-revision
echo guard. A newer program source advances the input revision while the host
retains preedit; an old terminal edit cannot overwrite it. ASCII final values are
accepted, and an invalid final value restores the accepted editor. These are
protocol tests, not a claim of actual OS IME acceptance.

Host capture has a 256-byte component budget under the shared 4096-byte maximum.
An oversized host value becomes a small `InputEdit::Rejected` payload before
business capture clones it. `MaskRejection` contains a finite reason only.
Native `SelectText` explicitly rejects non-ASCII values rather than claiming
precise grapheme/surrogate selection. Unicode preedit observation remains possible
within the capture budget; no selection action is sent during preedit.

## Field, forms, styling and ownership

The builder supports theme, size, unstyled, class/style, name, enabled, focus,
node_ref, Field, required, readonly, invalid and form_name. Class, style and refs
belong to the real input. Unstyled preserves a usable default 240px width and
36px minimum height; explicit dimensions remain authoritative. Field label,
description and error relations target the actual editor. Readonly and disabled
controls cannot publish masked edits.

The actual HTML control's successful value is its **formatted display string**.
Register the raw business Cage explicitly with `FormState` or serialize it in an
application submit handler when raw data is required. Enter commits the raw
value before the normal ancestor Form submission; Mask never starts HTTP,
changes routes or installs a separate submit policy.

All callbacks, current drafts and echo state belong to the constructing Owner.
Retirement clears callbacks and refs, suppresses late observers/actions and does
not retain an owning Scope. Recreating a dynamic control starts from the current
application source.

## Evidence

- Pure model: 9 tests in `src/mask_model.rs`; finite parser/raw-first paste,
  exact index maps, partial values, backward selection, insertion/deletion,
  same-tick physical maps and atomic rejection. Log:
  `D:/BuildCache/sapid/mask-model-rust.log`.
- Component: 9 tests in `tests/mask_inputs.rs`; raw/display distinction,
  typed-before-generic, controlled refusal, canonical edit without selection
  action, same-tick commit, composition/old revision, Field, external budget
  recovery and callback-driven Owner retirement. Log:
  `D:/BuildCache/sapid/mask-input-rust.log`.
- Native component: 2 tests in `examples/native/tests/mask_input.rs`; four
  themes, actual editor text/selection, stable engine identity, hit targets,
  pixels, unstyled geometry, controlled refusal, readonly/disabled and cleanup.
  These drive the component through typed events and verify physical results.
  Log: `D:/BuildCache/sapid/mask-input-native.log`.
- Shared native driver: actual Backspace, insert, Enter and IME protocol test
  `native_opt_in_edit_captures_current_selection_intent_and_ime_without_the_legacy_value_path`,
  plus ASCII selection/direction/revision refusal; 36 existing native input
  regressions passed. Logs: `mask-edit-native.log`, `mask-selection-native.log`
  and `mask-edit-native-regressions.log` under `D:/BuildCache/sapid/`.
- Shared target/revision/HTML oracle tests and strict Sapid/Web/native checks
  passed. The original browser suite ran against the fixed Web26 artifact at
  `http://127.0.0.1:8794/web26` with Chromium 148.0.7778.96: **12/12 passed,
  errors `[]`**. Eight four-theme CSR/hydration cases plus same-tick physical
  maps, native clipboard/budgets/Owner retirement, dirty takeover and
  composition/revision/touch cases passed. An intercepted real input setter
  confirms canonical keyboard insertion does not assign the physical text.
  Report: `D:/BuildCache/sapid/mask-input-browser-web26.json`; log:
  `D:/BuildCache/sapid/mask-input-browser-web26.log`. Theme screenshots use
  `D:/BuildCache/sapid/mask-input-{hue}-{scheme}-{csr|hydrate}.png`; the blue
  light CSR and green dark CSR captures were visually inspected.

Actual OS IME, screen-reader announcements and non-ASCII mask selection remain
outside these claims. The default graph adds no parser/serde dependency.
