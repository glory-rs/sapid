# RichTextEditor

Enable Sapid's dependency-free `editor` feature. `rich_text_editor()` is a real
multiline contenteditable control on Web, with a finite document value and a
separate, genuine button toolbar. It does not accept HTML strings.

```rust
use sapid::prelude::*;
use std::rc::Rc;

let document = scope.cage(Rc::new(RichDocument::default()));
rich_text_editor()
    .name("Article editor")
    .document(document)
    .on_document_change(move |next| document.revise(|value| *value = next))
```

`default_document` selects uncontrolled operation; `document` selects controlled
operation. A proposal becomes accepted only when the controlled source accepts
that exact document. Refusal restores the accepted physical value even if the
source emits no mutation. External source changes are silent and clear undo/redo.
`readonly` preserves focus and selection; `enabled(false)`, ancestor Fieldset
and inert policy prevent editing. `toolbar(false)` removes the toolbar. The four
themes, size, style/class and unstyled options use the ordinary Sapid contract;
unstyled retains required editor geometry.

`controller_ref` exposes an Owner-bound `EditorController`. `format` and `insert`
query the actual host selection; only one current query is retained and a newer
command cancels it. Explicit `format_at`/`insert_at` accept validated UTF-16
ranges. `undo`, `redo`, and uncontrolled `reset` operate on accepted documents.
The `document`, `selection`, `composing` and `history` outputs are Vary handles.
Selection is the last observed range, not a claim that pixels were displayed.
A retired controller returns a typed error and cannot affect a replacement Owner.

The default toolbar supplies bold/italic/code, paragraph/heading/quote and undo/
redo. Ctrl/Meta+B, I, backtick and Z/Y/Shift+Z use the same finite commands.
Safe links use `RichFormat::Link`. Clipboard paste is plain text; pasted HTML,
images, tables, embedded widgets and arbitrary style declarations are unsupported.
Inline formatting applies to a nonempty selection; `insert` accepts explicit
marks. Persistent typing-format toggles are outside this finite surface.
Native browser editing supplies deletion/grapheme behavior; the Rust model does
not replace the platform's input method with scalar-index editing.

## Document and history limits

`RichDocument` has paragraph, heading 1–6 and quote blocks. Each run contains
canonical text and independent bold, italic, code and optional http/https/mailto
link marks. Text and links are escaped when serialized. Blocks, runs, text and
metadata have independent limits:

- 1–128 blocks and at most 2048 runs across the current document.
- At most 65536 UTF-8 text bytes, 2048 bytes per link, 131072 aggregate link bytes.
- UTF-16 anchor/focus ranges must stay at scalar boundaries; backward selection
  is preserved. Paragraph separators are block boundaries, never run newlines.
- At most 32 accepted undo/redo snapshots combined and 2 MiB of logical document/
  selection bookkeeping. This is a payload budget, not an allocator or RSS bound.
- Host projection admits at most 16384 DOM nodes and 16 inline nesting levels.

Adjacent runs with equal marks merge. Empty blocks have one unmarked empty run.
Invalid host grammar or an over-budget draft is rejected with `RichRejected`
and the last accepted finite document is restored. It does not produce a success
callback or undo entry. Preedit and selection-only observations do not enter
history; a completed accepted composition creates one entry. A newer program
source wins over a late composition completion.

## Platform and protocol boundary

Web uses one stable DIV textbox with `aria-multiline`, actual DOM selection,
composition and finite document nodes. Host listeners enqueue observations;
projection and geometry reads precede private repairs. Equal canonical source
echoes acknowledge revisions without reconstructing the document. Program
revisions, query watermarks, stale replies and Owner retirement follow the shared
Input lifetime contract. No `execCommand`, raw HTML insertion or per-frame Cage
is involved.

SSR emits the same finite document and metadata. Hydration preserves the main
control identity and can adopt a bounded dirty pre-WASM document and selection.
Classic HTML format 13 transports typed RichDocument AST properties through
Start/Create/Patch/End. It preserves dirty composing text until End can settle,
then releases the controller. Classic rendering does not install business editing
callbacks or an undo manager. Format 12 is rejected before host mutation.

Native currently rejects RichEditor at descriptor preflight, direct create,
CreatePlan and related action/query boundaries. There is no textarea fallback.
The pure document/controller tests are not native editing evidence. Actual OS
IME, screen-reader announcements, Firefox and Safari acceptance remain untested;
Chromium CDP composition tests establish only the recorded Chromium host scope.

## Evidence

- Pure document 4 and shared revision/guard/event/context 3 tests:
  `D:/BuildCache/sapid/rich-render-tests-final.log`.
- Full HTML library 30 tests, including all patch variants/reset differential
  oracle and inherited-disabled RichEditor serialization:
  `D:/BuildCache/sapid/rich-html-tests-final.log`.
- Native explicit-rejection 1 test and strict Clippy: `rich-native-tests.log` and
  `rich-native-final-clippy.log`; no native editing is claimed.
- Shared render/UI/HTML/SSR allfeatures/alltargets strict: `rich-shared-final-clippy.log`.
- Sapid controller/component 6 tests cover accepted history, controlled refusal,
  source silence, composition, finite budgets, formatting and four-theme SSR: `rich-component-tests-final.log`.
- Classic Rust producer 1: `rich-stream-tests.log`; actual Chromium 148 classic
  11 cases: `rich-editor-stream-browser-rich-classic41.json`. This independent
  artifact uses the immutable Web41 classic script and a separately generated
  Rust fixture. It does not replace the frozen Web41 product.
- Immutable Web41 original suite exposed a text-run boundary affinity difference
  in exact backward selection. The original failure is preserved in
  `rich-editor-browser-web41.json`; the fixed driver passes the unchanged
  original nine cases in immutable Web42 (`rich-editor-browser-web42.json`).
- Immutable Web42 actual Chromium148: original four-theme CSR/hydrate and
  dirty-adoption/IME/retirement 9, classic13 11 and real clipboard/grapheme 2 all
  pass without weakened assertions. Reports: `rich-editor-browser-web42.json`,
  `rich-editor-stream-browser-web42.json`, `rich-editor-extra-browser-web42.json`;
  combined log: `rich-editor-all-actual-web42.log`.
- Final immutable Web43 repeats the unchanged original 9 and classic 11, plus
  3 extras: actual plain-vs-HTML clipboard paste, native grapheme deletion and
  dispatched raw `beforeinput` historyUndo/historyRedo command routing. All
  reports have zero errors: `rich-editor-browser-web43.json`,
  `rich-editor-stream-browser-web43.json`, `rich-editor-extra-browser-web43.json`;
  combined log: `confirmation-rte-all-actual-web43.log`. The third extra checks
  the raw input route; it does not claim an OS menu was automated.
