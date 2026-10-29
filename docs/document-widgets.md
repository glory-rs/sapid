# Intrinsic document components

Code, Mark, Kbd, TextList, Blockquote and DataList use explicit finite Glory
document kinds. The HTML mapper emits actual `code`, `mark`, `kbd`, `ol`, `ul`,
`li`, `blockquote`, `dl`, `dt` and `dd` elements. They do not substitute a generic
View with a similar ARIA role. Native AccessKit exposes Code/Mark, Blockquote,
List/ListItem and DescriptionList/Term/Definition roles; Kbd uses readable text
because AccessKit has no dedicated keyboard-hint role.

All components expose theme, size, unstyled, class, style and node_ref. Explicit
classes append; supplied style declarations override only themselves. Text
content is escaped and follows Glory's canonical LF/U+FFFD text mapping. Styling
does not modify business values or input selection/revisions.

## Inline text

`code().content(source)` renders inline code with a monospace face and a themed
soft surface. It does not tokenize a language or register a copy action.
`mark().content(source)` renders real marked text with independent theme colors.
Both accept reactive Vary<String> sources and patch the existing Text node.

`kbd().content(source)` renders a keyboard hint. Alternatively,
`kbd().platform(KbdPlatform::Mac).key(KbdKey::Meta)` displays `Command`.
Platform is explicit Windows/Mac/Linux mount configuration; the OS is not
detected. Supported key names include Control, Alt, Shift, Meta, Enter, Escape,
Backspace, Tab and Space. Meta becomes Windows/Command/Super; Mac Alt is Option
and Enter is Return. `content` clears a prior key selection; `key` selects a
platform name. Kbd registers no shortcut or keyboard handler.

Code/Kbd/Mark are phrasing content and may be placed inside an existing native
button without becoming nested controls. Plain document text has content rather
than an explicit accessible name. Host tag overrides cannot erase these kinds,
and changing their tag requires remounting on HTML adapters.

## Lists and description data

`text_list().ordered(true).item(text_list_item().fill(children))` builds a real
ordered list. Omit ordered for an unordered list. Nest another TextList inside a
TextListItem for nested lists. The API name distinguishes this finite mounted
document list from Glory's virtual/windowed List. It stores only the explicitly
supplied items and has no virtual business source or visited-item cache.
Each TextList or DataList accepts at most 256 items. Iterator setters consume
only the remaining allowance plus one overflow sentinel; repeated single-item
setters retain no additional values once that sentinel exists. Oversized lists
fail before creating nodes. Nested TextLists have independent local limits.

`data_list().item(data_list_item().term("Project").fill(value_widget))` builds
actual term/definition pairs. Each pair is a boxless Group, so dt/dd remain
direct physical children of dl. Terms accept reactive text; values accept
arbitrary valid semantic content, including links or independent action buttons.
DataList has no implicit selection, grid cursor, sorting or editable-value model.
Parent theme/size/unstyled settings apply to mounted list items; explicit item
theme, size and style overrides retain precedence. Optional list names do not install an overriding
generic Group role on native list elements.

Glory validates nearest boxed parents through transparent Groups: lists accept
ListItem, description lists accept Term/Definition, and those children cannot be
placed under a generic View or at the document root. Ordinary placements check
the ancestor path and the moved boxed root. Only transparent Group frontiers
need inspection. New plans apply the same rules before live placement. The fixed
document part cannot be changed by a live Semantics patch; a rejected update
rolls back without advancing stats or the accepted watermark.

## Quotations and content rows

`blockquote().fill(quote_widgets).citation(source_widgets)` creates a real
quotation with an independent attribution slot. Citation accepts ordinary source
text or an explicitly targeted link supplied by the application. The component
does not infer or fetch a source URL.

`item().title("Repository").description("Project details").media(icon)
.fill(content).actions(button)` is a passive content row with three separate
regions. It does not imply selection, click activation or a list role. Put it
inside TextListItem when list membership is intended. Child actions retain their
own native accessible names, focus and disabled behavior.

## Bounded inline Highlight

`highlight().content(text).query("Glory")` splits mount-time canonical text into
ordinary inline Text and real Mark ranges. Multiple queries use earliest match,
then the longest query for a tie. Adjacent equivalent ranges merge. Matching is
exact and case-sensitive; UTF-8 boundaries are preserved. There is no language
parser, Unicode case-fold expansion or source mutation.
Query iterators also stop at the 32-query allowance plus one overflow sentinel;
oversized configurations do not enumerate an arbitrarily large lazy iterator.

The root is a boxless Group, so Highlight may be used inside native button
content without inserting a div. Theme/class/style apply to its rendered text
ranges. Its optional node_ref is a logical Group identity and is not a physical
query/focus target. Text is limited to 64 KiB, at most 32 queries of 1–256 bytes,
and 1024 merged ranges. Budget rejection occurs before any nodes are created.

Content and queries are deliberately mount-time configuration. This component
does not repeatedly rebuild ranges on reactive updates and does not claim a
cross-host rich-text selection-preservation protocol. Existing native text
selection is not rewritten by an independent Highlight effect or event handler.

## Evidence

`tests/document.rs` covers intrinsic kinds/tags, reactive text identity, key
labels, four themes/unstyled/overrides, list nesting, orphan rejection, direct
dt/dd structure, interactive value separation, quote attribution, lifecycle,
escaping, highlight overlap/Unicode/canonical text/range budgets and passive Item
regions. `examples/web/src/document_form.rs::document_form(theme)` is shared fixture
content for Web/SSR/native integration.

Glory adds dedicated render and HTML document tests plus the existing full-shape
patch differential oracle. They verify structural targets, immutable document
parts, rollback/stats/watermarks, transparent Group Place/Resume and Plan rules,
intrinsic tags, tag-override rejection and phrasing content in native buttons.
`glory-apace`'s native AccessKit consumer test verifies real engine output for
all new document roles and keyboard-hint text. These checks do not claim spoken
screen-reader output or platform-specific text selection acceptance.

The document browser suite passed all eight blue/green × light/dark × CSR/adopted
SSR runs on 2026-10-24. It checked intrinsic tags, named list/term/definition AX
roles, dt/dd physical adjacency, original code element and text-node identity,
Unicode text selection across Mark ranges and retention during an unrelated
reactive code update, mobile overflow and teardown. Screenshots are under
`D:/BuildCache/sapid/document-*-hydrate.png`. The standalone native document test
also passed all four themes with positive real-engine bounds for the nine
display/document node categories and complete retirement.
