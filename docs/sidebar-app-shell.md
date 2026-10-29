# Sidebar and AppShell

`Sidebar` and `AppShell` provide a finite application frame with one live navigation tree. They do not install routing, persistence, transport, or a global breakpoint service.

```rust
use sapid::{app_shell, sidebar, anchor, text};

app_shell("workspace")
    .initial_width(1024.0)
    .breakpoint(768.0)
    .sidebar(sidebar("primary").name("Workspace").content(|context| {
        anchor().theme(context.theme).label("Overview").href("/overview")
    }))
    .header(|_| text().content("Workspace header"))
    .main(|_| text().content("Main content"))
    .footer(|_| text().content("Application footer"));
```

The shell uses real Banner (`header`), Main (`main`), ContentInfo (`footer`), and Navigation (`nav`) landmarks. Header and footer exist when their content is configured; a sidebar also creates a header containing its primary trigger. The main region is always present and programmatically focusable. The visible skip link captures activation and queues the shared typed Focus action to that main region; it does not rely on a browser-only fragment jump.

Both widgets accept theme, size, unstyled, additive class/style, node reference, and generic event observers. Sidebar additionally accepts a name, navigation/drawer node references, expanded width, and rail width. Widths must be positive, finite, at most 1,000,000, and rail width cannot exceed expanded width. Default widths are 256 and 72 pixels. Collapse retains the navigation content in a clipped rail; applications choose suitable compact content with the supplied context. `unstyled` leaves colors, padding, and dock decoration to the application while retaining structural sizing and semantics.

## Responsive measurements

`ShellLayout::new(scope, initial_width, breakpoint)` is an explicit local container context. It exposes read-only `width()` and `mobile()` Bonds and a validated `observe_width(width)` host input. Widths must be finite in 0..=1,000,000; breakpoint must be positive. Mobile means width strictly below the breakpoint. An AppShell creates this context from its explicit initial configuration by default, or shares a supplied `.layout(layout)`. Its real queued Resize events supply the shell container's measured width, including the initial browser measurement. No component reads browser globals. SSR uses exactly the supplied initial width; browser adoption occurs before the first measured transition.

A standalone Sidebar requires `.layout(layout)` and an explicit outer measurement driver. It fails before creating its nodes when that context is absent. Foreign Runtime or retired layout sources are also rejected before shell construction. This local container facility is not a viewport-wide breakpoint provider. Applications needing coordinated viewport policy must supply their own shared driver rather than treating each shell's local measurement as global state.

The shared scope placement API moves the existing navigation node between its desktop dock and its mobile dialog. It preserves the original Owner, refs, factories, input values, and tasks. Content is constructed once. Mode changes do not clone a second navigation subtree. Actual FocusWithin origins guide focus restoration after moves: entering a closed mobile drawer focuses the primary trigger, and returning a focused navigation control to desktop requests focus on that same control. Focus is never inferred from an earlier focus intent.

## Controlled requests and the host drawer

`.collapsed(source)` and `.mobile_open(source)` are read-only controlled inputs. Local `.default_collapsed` and `.default_mobile_open` apply only without their corresponding source. Typed change callbacks propose state before generic Activate or Dismiss observers. External writes are silent. The desktop primary trigger proposes collapse; the mobile trigger and close button propose open/close. A rejected controlled proposal retains the accepted presentation.

Factories receive `SidebarContext` or `ShellContext` with theme, size, unstyled state, layout, and a `SidebarHandle`. The handle exposes read-only accepted collapsed/mobile/open Bonds and fallible `set_collapsed`, `set_open`, and `toggle` requests. It retains no owning Scope. Retiring the component clears its callback captures and node refs even when application code retains a handle; subsequent requests return Gone.

Mobile uses Glory's shared modal Layer with left placement, Escape, outside dismissal, focus containment, and return focus. Its initial focus is the dialog's permanently mounted named close button, which is valid before navigation is reparented. The application does not implement a separate browser focus trap. Desktop removes Layer metadata and keeps the hidden dialog shell. Adapter admission occurs before reparenting or accepted presentation updates. Apace currently rejects top layers explicitly: a desktop-to-mobile request reports that unsupported capability while retaining the accepted desktop navigation parent, identity, and state. Desktop layout, collapse, landmarks, and typed skip focus work natively. This is a documented mobile capability boundary, not a simulated native drawer.

`on_width_change` runs after an accepted responsive transition and before generic Resize observers. Repeated equal measurements do not emit a width callback. Programmatic collapse/open writes and resize observations remain separate from application persistence or routing.

## Evidence

- `tests/app_shell.rs`: 10 passing tests cover finite layout configuration and foreign/retired contexts, actual landmark/skip semantics, one-tree placement and factory/ref stability, controlled rejection/default precedence, silent source writes, shared Layer configuration, callback order, SSR initial layout, and external-handle Owner retirement.
- `examples/native/tests/app_shell.rs`: 2 passing tests cover four themes with actual native landmark geometry, collapsed engine identity, typed skip focus, and explicit Layer rejection before navigation movement with recovery to desktop.
- `examples/web/shell-acceptance.js`: 9 passing suites on Chromium 148.0.7778.96 using the unified build4 WASM/bindgen/SSR output. Four-theme CSR/hydration covers real Resize takeover, rail sizing, controlled rejection, mouse/Enter/Tab/Escape/outside behavior, host modal focus/return focus, navigation identity/drafts, separate link/disclosure behavior, actual focus across moves, dirty mobile hydration, and Owner teardown. Reports and screenshots are saved at `D:/BuildCache/sapid/shell-browser.json`, `shell-browser.log`, and `shell-*.png`. Custom accessible navigation/dialog names are checked after the browser-discovered name-consumption regression was fixed.

Browser protocol and AccessKit tree checks do not establish screen-reader announcements or OS assistive-technology acceptance. Resizable split panels, arbitrary nested application frames, off-canvas native fallback, global viewport context, persistence, and router integration are outside this finite API.
