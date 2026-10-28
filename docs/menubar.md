# Menubar

P4-03 has an exported builder/controller, shared Menubar role/host support and
actual browser evidence from the final same-source snapshot.

The API declares a finite set of real menu roots:

```rust
sapid::menubar("application")
    .name("Application")
    .menus([
        sapid::menubar_menu("file")
            .label("File")
            .item(sapid::menu_item("new").label("New")),
        sapid::menubar_menu("view")
            .label("View")
            .item(sapid::menu_checkbox("grid").label("Show grid")),
    ]);
```

The container exposes the real structural Menubar role and horizontal
orientation. Each root remains an intrinsic Button refined to a menuitem with a
typed Menu popup, expanded state and a stable controls relation. Its popup is
the existing real Menu tree, including intrinsic checkbox/radio items, nested
menus and cancellable submenu delay. A Toolbar or arbitrary interactive View
does not stand in for these roles.

`value(Vary<Option<String>>)` is the sole root source: None closes all roots,
Some(key) opens one declared root. `default_value` supplies only local state;
`on_value_change` proposes the whole optional key. Refused switching keeps the
old menu and roving entry. Acceptance is rechecked after callbacks and before
physical focus, including current enabled membership and Owner liveness. A
single bounded pending focus intent is reconciled against the final accepted
root after a host tick, rather than queuing focus into a root that closed later
in that tick. This intent does not claim that actual focus already moved.

Root Left/Right, Home/End and opt-in bounded Typeahead use the shared live
Collection/RovingFocus model; disabled roots are skipped and only one enabled
root is a Tab stop. Direction is explicit LTR/RTL. Down/Up opens the root at its
first/last enabled menu item. Within a root menu, forward first enters an actual
submenu; without a submenu it switches to the next root. Backward leaves a
nested menu one level at a time and switches to the previous root only at the
root menu. Escape returns to that root trigger. Host ExitMenu must make Tab and
Shift+Tab leave the complete bar boundary. Root hover does not automatically open
or switch menus in this version; click and keyboard activation are explicit.

Configuration admission stops at 32 roots plus one rejection sentinel, with
1024 total root triggers/menu entries, 256 entries per menu level and menu depth
eight. Each root/entry key is a bounded explicit ASCII key. The composed bar/root
namespace must also fit Menu's 64-byte namespace limit; long combinations fail
explicitly. Source optional keys are validated by borrowing before cloning.
Root labels use the Collection label budget. No business source, visited root
history or stale node mapping is retained. User callback captures and retired
root registrations are dropped outside registry borrows.

Sapid `src/menu/bar/tests.rs` has four passing tests for intrinsic roles and one
Tab stop, disabled skipping, Typeahead, exclusive hierarchy switching,
controlled refusal, borrowed invalid-source validation, bounded million-root
iteration and Owner capture cleanup. Glory `render/tests/menubar.rs` validates
the finite intrinsic refinement and rejects late focus to an old root;
`html/src/menu/bar/tests.rs` checks prepared patches against the full-shape
oracle, including default resets. The existing `ssr/src/stream/menu_tests.rs`
producer transports Menubar role/orientation, root marker and expanded state
through Shell/Patch/End. The original acceptance used HTML v4/classic v2;
current producers use HTML v13 with the same classic v2 envelope.

`blitz/tests/menubar.rs` now verifies finite admission and retained query/prefix
watermarks after a later unsupported control. The native connection covers root
and submenu commands, directional navigation, Typeahead and whole-bar Tab exit.
Actual four-theme Sapid consumers and current AccessKit MenuBar/MenuItem/popup
relations pass; this is a self-drawn semantic menubar, separate from platform
system-menu services and OS assistive-technology acceptance. See
[native menus](../../glory/docs/native-menus.md).

`D:/BuildCache/sapid/menubar-browser-menubar-final-rating-next.json` records the
unmodified `examples/web/menubar-acceptance.js` against the fixed combined
`D:/BuildCache/sapid/web/rating-next` WASM/SSR snapshot, served on port 8812.
Chromium 148.0.7778.96 passed all 10 groups with `errors: []`. Eight groups cover
the four themes in CSR/hydration: actual AX Menubar/Menu/MenuItem roles, custom
names, one root Tab stop, disabled skipping, Typeahead, first/last entry, nested
menus, exclusive horizontal/RTL root switching, Escape focus restoration,
forward/backward Tab exit, controlled keyboard/pointer/Escape refusal, native
checkbox echo, explicit root-hover policy, actual touchscreen activation and
Owner removal. Independent groups cover dirty adoption preserving sibling
editor identity/value/focus/selection and Rust-produced classic stream End.
The earlier snapshot using a diagnostic name adapter is not completion evidence.

Existing Menu's eight model tests and ContextMenu's four model tests remain
green; ContextMenu's full 14 browser groups also passed against the Menubar host
changes. No OS screen-reader or native Menubar interaction acceptance is claimed.
