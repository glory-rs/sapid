# FocusScope

`focus_scope().fill(children)` declares focus policy around ordinary visible
content. Web and interactive native hosts perform actual focus traversal;
Memory and SSR retain the declaration without claiming input capability.

```rust
use sapid::{focus_scope, FocusScopeConfig};

focus_scope()
    .active(active)
    .loop_tab(true)
    .contain_focus(true)
    .auto_focus(true)
    .restore_focus(true)
    .fill(content)
```

`.configuration(Lotus<FocusScopeConfig>)` accepts all five flags together and
optional named `initial_focus` and `return_focus` relations. Individual builder
setters override the corresponding configuration field. Controlled inputs remain
read-only. Inactive scopes keep their content and ordinary document Tab order.
Changing policy preserves nodes, values, selections and component Owners.

The default active policy automatically focuses the first eligible control and
loops Tab in both directions. `contain_focus(false)` permits explicit focus
outside. With `loop_tab(false)`, an uncontained boundary allows document Tab
exit; a contained boundary keeps focus on its current edge control. Named
initial targets resolve only within the current scope. Unresolved relations can
resolve after mounting; they do not require construction order tricks.

This is ordinary programmatically focusable View semantics. It adds no Dialog
role, backdrop, top-layer painting, background inert state, scroll lock or
Escape dismissal. Background accessibility and activation remain available;
Focus actions outside the current containing scope are refused. Current
anchored popups retain their own keyboard handling. A modal Layer takes priority
over an unrelated focus scope.

Web scopes share the existing document-owned Layer coordinator across Apps.
Nested scopes take priority over their enclosing scope. Deactivation and Owner
retirement restore focus only when focus belongs to the closing scope; closing
an unfocused scope preserves the current outside focus. Return paths transfer
through out-of-order and simultaneous nested retirement, retaining only current
leases and live identities.

Each App accepts at most 1024 declared scopes, including inactive, hidden and
suspended declarations. Each restricted scope subtree accepts at most 4096
current semantic nodes, including Groups and suspended descendants. Shared
creation, Plan, placement and prepared metadata checks enforce the same limits.
Rejected moves and dropped/unwound field guards preserve counts, statistics and
the accepted query watermark. Ordinary plans without scopes allocate no scope
size buffer; same-scope moves walk ancestors without scanning their subtree.

HTML10 serializes five explicit boolean policy fields and two optional bounded
semantic relations. Set/Keep/Clear use the full-shape differential oracle.
Classic envelope2 carries the same fields and versioned Start/End; it validates
finite policy values. Actual focus policy begins with the live WASM host after
complete adoption, preserving inactive dirty editors.

Evidence so far under `D:/BuildCache/sapid`:

- `focus-model-tests.log`: five shared count, Plan, lifecycle, guard and prefix
  watermark regressions.
- `focus-html-tests.log`: every policy field and reset against the full-shape
  oracle.
- `focus-stream-tests.log`: classic field patches and current Start/End format.
- `focus-sapid-tests.log`: three component/SSR tests, including four themes and
  unchanged controlled source values.
- `focus-native-tests.log`: five real native focus/Tab/return tests, including
  nested Owner retirement, associated popup precedence and disabled candidates.
- `focus-native-ax-tests.log`: background AX and Click stay available while
  Focus actions respect containment, without modal metadata.
- `focus-web-strict.log` and `focus-native-strict.log`: strict host compilation.

`focus-browser-web33.json` records the original 13 browser cases passing on the
immutable FORMAT10 artifact. `focus-stream-browser-web34.json` records four
same-source classic-stream cases, including invalid policy/relation and old
format refusal without dirty-editor mutation. The strengthened 15-case Web34
run passed its behavior assertions but caught two synchronous focus-listener
reentry errors; that failed report is retained. The listener now uses an
immutable closure. `focus-browser-web35.json` verifies all 15 original cases
on the immutable HTML10 Web35 artifact, including both cross-App cases, with
no console or page errors. `focus-stream-browser-web35.json` repeats all four
classic cases on that same artifact. The current protocol is HTML11; its
independent Environment upgrade does not change these focus policy fields.
Actual OS IME, screen-reader announcements and platform AT client acceptance
require their own evidence; protocol tests do not establish them.
