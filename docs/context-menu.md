# ContextMenu

P4-02 has component, shared protocol, native connection and actual browser evidence
against the unified root Web17 snapshot.

`context_menu(key).target(widget)` mounts a named Group region around an ordinary
widget and a real Menu tree. The target keeps its intrinsic child controls;
neither the region nor a button is reclassified as a menu trigger. The region is
a Tab stop and has no `aria-haspopup` or `aria-expanded`. `target_name` names the
region; `label` names its menu. `target_ref` and `node_ref` expose the region and
panel respectively. The existing Menu item, checkbox, radio, separator, submenu,
direction, loop, typeahead and optional hover-delay APIs are shared.

Right-click captures a finite `Event::ContextMenu` request containing its exact
boxed origin, source and viewport CSS-pixel point. Coordinates must be finite.
The host clamps the menu inside the current viewport and repositions on its
usual layout/viewport turns. Keyboard ContextMenu and Shift+F10 use the actual
focused control's bottom-left box; composition, repeats and unrelated modifiers
remain native. Both keydown and the browser's subsequent contextmenu event check
active composition, so Shift+F10 cannot bypass the guard through its native
follow-up event. Host listeners enqueue requests without entering application code.

`editor_policy(ContextEditPolicy::Native)` is the default: Input, Textarea,
Select and native editable context commands stay with the browser, for pointer
and keyboard requests. `Override` explicitly admits those origins. Editor value,
selection and composition remain the real editor's state. Touch long-press is
always native in this version; there is no implicit timer or synthetic right
click. Unsubscribed regions do not cancel browser context behavior.

Only the nearest declared context region can handle a request. Shared Memory
validation checks runtime/generation, live placement, physical boxed origin and
ancestry. An outer region cannot accept a forged event through a nearer context
boundary. Retired, suspended, hidden or inert origins cannot reopen a menu, and
boxless Groups cannot be physical origins. Direct App dispatch and queued host
requests follow these same checks.

`open` is controlled input and `on_open_change` proposes changes. A rejected
opening keeps the previous point and visibility; a rejected closing leaves the
menu present. Accepted context requests update only the private current point.
No source history or source scan is retained. Budgets remain 1024 total menu
entries, 256 per level and depth eight, with bounded iterator admission. Target
children belong to a real child Scope under the region and retire with it.

The shared nonmodal Menu Layer manages focus and dismissal. Escape and selecting
a closing action return to the actual previously focused control, including an
editor descendant, without requiring an invented node name. Tab closes the menu
tree and exits the complete context target region in its direction; the host's
bounded mounted-tree traversal performs that action. Keyboard hierarchy, native
checkbox/radio state, pointer gaps and cancellable submenu delays are the same
Menu mechanisms, including Owner cancellation.

`LayerPlacement::ContextPoint` carries the finite point and retains a distinct
named region anchor. Only a Menu Layer may use it. HTML uses the existing v4
shape and classic v2 finite attribute schema. Default/reset policy changes and
point updates use prepared-field lowering with the full-shape differential
oracle. Classic streaming transports declarations and point updates; it does not
implement a second pre-WASM menu input/focus driver. Adoption starts the actual
Web driver from the declared live state.

Apace admits context regions, nearest-boundary/editor policy and finite mouse/pen or
keyboard requests. ContextPoint Menu layers use actual viewport geometry and
preserve parent modal/focus-scope boundaries. Native Tab exit uses the current
context region as a boundary, excluding its descendants and all Menu trees;
controlled close and exit can commit in one frame. The native tests retain the
accepted-prefix/query contract when a later unsupported control is rejected.
Actual four-theme Sapid context consumers, CPU frames and AX models are covered
in [native menus](../../glory/docs/native-menus.md); OS input and screen-reader
acceptance remain separate. Touch long-press is unchanged.

Current evidence: Sapid `tests/context_menu.rs` has four passing component,
controlled refusal, observer order, Owner and iterator-budget tests. Existing `tests/menu.rs`
has eight passing regressions. Glory `render/tests/context_menu.rs` has two
passing finite-policy/payload/ancestry/retirement/rollback tests;
`html/src/context_menu_tests.rs` has one passing policy/point/reset full-shape
oracle; `ssr/src/stream/menu_tests.rs` verifies policy/point Shell/Patch/End;
`apace/tests/context_menu.rs` verifies explicit rejection before facets and the
accepted query watermark. Web fixture strict Clippy passes. The independent
browser fixture and suite have completed actual acceptance.

`D:/BuildCache/sapid/context-menu-browser-root17.json` records Chromium
148.0.7778.96: 14 groups passed, `errors: []`. Eight groups cover four themes in
CSR/hydration, true right-click viewport placement, actual AX group/menu/item
roles, disabled skipping, real checkbox/radio, keyboard submenu hierarchy,
ContextMenu/Shift+F10, actual prior descendant focus, forward/backward Tab exits,
controlled opening/closing refusal, disabled regions, narrow viewport and Owner
cleanup. Independent groups cover nearest nested regions, explicit native/custom
editor policy with selection preservation, native touch policy, browser IME
protocol including the native follow-up contextmenu event, dirty adoption with
editor identity/value/focus/selection, and Rust-produced classic stream patches.
The open green-dark hydration screenshot was visually inspected. Touch policy is
a finite PointerEvent protocol test; it does not claim OS long-press UI or actual
screen-reader announcements. CDP composition is not OS IME acceptance.
