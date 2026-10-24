# NavigationMenu

NavigationMenu extends the existing NavLink baseline with a real Navigation
landmark, actual links and separate disclosure buttons. Its floating content is
a physical `View` with `Role::Group`, not a logical boxless Group, menu, dialog,
menuitem or selection widget. Current-page metadata remains independent from
highlighting and browser visited state. The application owns routing.

The API uses explicit `NavigationMenuContext` registration:

```rust,ignore
navigation_menu("primary").content(|context| (
    navigation_menu_link("home")
        .context(context.clone()).label("Home").href("/"),
    navigation_menu_panel("products")
        .context(context).label("Products")
        .content(|children| (
            navigation_menu_link("editor")
                .context(children).label("Editor").href("/editor"),
        )),
));
```

Contexts allow normal Glory conditional/keyed children to register only while
their Owners are live. Registration is finite, sibling keys are distinct, and
retirement removes captures and entries without retaining a visited-key history.
An open path contains at most eight bounded keys and is validated before cloning.
Controlled sources remain read-only; callbacks propose paths before observers.
Rejected dynamic sources leave interaction callbacks using accepted state.

Header direction keys and Home/End traverse actual links and disclosure buttons;
popup direction keys traverse their own mounted level. Links keep their native
Tab order. Down/Up on a header disclosure enters its first/last link. A nested
forward command opens children; a backward command closes to the parent trigger.
Escape/outside dismissal uses the host Layer path. Leaving through ordinary Tab
closes floating content without commandeering focus elsewhere.

Nonmodal Group layers preserve host focus: hover opens without moving focus,
keyboard entry requests a specific live link after the panel is accepted. A
finite navigation-panel command protocol keeps this behavior separate from menu
roles. Native hosts continue to reject unsupported floating Layers explicitly;
the closed navigation controls still have a meaningful native baseline.

Hover delays, when enabled, use one Owner-owned cancellable task and only the
current bounded intent. Repeated moves over that same intent do not restart its
timer. External path changes, retirement and obsolete member identities revoke
late publication. User factories, callbacks and captured destructors run outside
collection/controller registry borrows.

The registered graph has at most 128 live entries and eight panel levels. Each
key uses 1–64 ASCII token bytes and labels have a 4096-byte budget. The default
path receiver consumes at most nine entries; reactive paths are checked before
cloning. The path is a sequence of mounted panel keys, not URL segments. An
unknown dynamic path is rejected; applications must repair a controlled path
when they retire one of its referenced panels. No source collection is scanned.

Every panel shell and its contents stay mounted while closed. This preserves
editor values, selection and refs. `NavigationMenuContext::open()` exposes the
accepted visibility as a read-only Bond. Applications can explicitly put a
Glory conditional inside the factory if they want shorter child Owners; closed
or retired items are never retained in a visited-key cache. Contexts must belong
to the actual construction ancestor and live Runtime.

`open_path` is controlled and read-only; `default_open_path` creates local state.
`on_open_path_change` proposes interaction paths. External updates remain
silent, and rejected proposals do not move focus. Item `on_navigate` reads the
currently accepted link descriptor and runs before its generic Activate
observers. Providing that callback asks the Web host to prevent default link
navigation; without it, the href retains ordinary browser navigation. No router,
network request or persistent selection is installed. Current page/step uses
`.current(Some(Current::Page/Step))`; there is no `aria-selected` on links.

Validation recorded so far:

- `tests/navigation_menu.rs`: 13 passing tests for real roles/SSR, finite defaults,
  controlled rejection, stable shells, accepted href callbacks, Owner retirement,
  one cancellable hover task with late-wake rejection, same-tick focus transfer,
  and focused-member retirement without a terminal event.
- `examples/native/tests/navigation_menu.rs`: 2 passing tests, including actual
  focus and stable engine identity across all four themes. Opening a floating
  Group Layer returns Unsupported and leaves the closed descriptor/identity.
- Shared render protocol: 3 tests; HTML mapping: 1 test; native command queue:
  1 test; HTML patch oracle: 13822 accepted / 27306 rejected cases at this
  protocol checkpoint. Native key input only queues commands and does not run
  business callbacks from the driver.
- Sapid lib and Web WASM strict Clippy pass. Chromium 148.0.7778.96 on root
  Web14 passed all 9 real browser cases with `errors: []`: four themes in CSR
  and hydration plus dirty hydration/Owner retirement. Every theme also opens
  parent and child in one source update, checks actual adjacent placement,
  resizes the viewport, and checks alignment again. The host computes pending
  ancestor deltas in dependency order before any positioning writes; reused
  NodeId slots do not determine that order.


Logs are under `D:/BuildCache/sapid/navigation-menu-rust.log` and
`D:/BuildCache/sapid/navigation-menu-native.log`. Browser JSON/log are
`navigation-menu-browser.json` / `navigation-menu-browser.log`, with four-theme
`navigation-menu-*.png` screenshots in that build directory. The old Web12
focus failure remains separately archived; Web14 contains the tested fix.
These protocol/native checks do
not claim physical keyboard devices, screen-reader announcements or native
floating-layer support.


FocusWithin is a per-origin observation. The component aggregates loss/gain
within a host tick before deciding whether focus left the navigation. A
registered focused item retiring clears only its current registration token;
old NodeIds and same-key replacements cannot inherit that focus. Embedded custom
controls can live inside panels. For a dynamically retired custom control, put
`track_focusable` inside the same conditional branch, pass its actual node_ref,
and let the adapter own the widget:

```rust,ignore
let input_ref = scope.cage(None);
let shown = scope.cage(true);
navigation_menu_panel("tools").context(context).label("Tools")
    .content(move |children| glory::ui::when(shown, move ||
        children.track_focusable(input_ref,
            glory::ui::input().node_ref(input_ref).value("Draft"))));
```

This adapter adds no physical wrapper or navigation item. It consumes one slot
from the shared 128-live-registration budget and removes that token when its
Owner retires, even if the host emits no terminal focus event. A replacement
with the same reference cannot inherit the old generational origin. Static
custom controls may omit the adapter; independently retiring such an untracked
control requires the application to close the path itself. No global tree scan,
visited-focus history or polling service is installed.
