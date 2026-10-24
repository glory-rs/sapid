# Navigation

`Breadcrumbs`, `Pagination`, and `NavLink` are handwritten semantic widgets. They install no router, history policy, HTTP transport, or automatic destination fetching. NavigationMenu popup, menu keyboard behavior, and nested menu placement remain a separate unfinished roadmap feature.

```rust
use sapid::{breadcrumb_item, breadcrumbs, pagination, nav_link};
breadcrumbs("trail").items([
    breadcrumb_item("home", "Home").href("/"),
    breadcrumb_item("docs", "Documentation"),
]);
pagination("results").default_page(1).total(Some(120));
nav_link("docs").label("Documentation").href("/docs").current(true);
```

All three support theme, size, unstyled, additive class/style, accessible name, node reference, and event observers. Widget namespaces and breadcrumb keys are explicit bounded ASCII tokens. There is no generic focus or disabled-link promise on their structural containers. Pagination exposes `enabled`, mapped to every actual native button. Links retain native link semantics and keyboard behavior.

## Current location

Shared Glory `Current::{Page, Step}` is independent from selection, toggle state, and browser visit history. `Semantics.current` is supported only on Link, Button, or Text. HTML emits `aria-current="page"` or `"step"`; clearing removes it. AccessKit receives its corresponding `AriaCurrent` property. These widgets never assign `aria-selected` to links. `Anchor.current(...)` also exposes the shared finite location metadata.

## Breadcrumbs

Items have a stable key, reactive label, and optional destination. Destination items are actual links; absent destinations are actual text. The root is a named Navigation landmark containing List and ListItem semantics. The last key is current by default; `.current(source)` is controlled and an empty string explicitly clears it. Unknown keys are rejected while preserving the last accepted current state.

`max_items` defaults to 5 and must be at least 2. Collections are static and limited to 128 live items; construction consumes at most 129 iterator entries to detect and reject overflow. Longer paths show the first item, a named ordinary expand/collapse button, and the last `max_items - 2` items. A current item in the middle also remains visible, so the current-location exception may add one item to that presentation limit. This is a disclosure of the path; it does not claim menu semantics. Label and current updates preserve item and link identities.

`expanded` is controlled; `default_expanded` only supplies local state without a controlled source. `on_expanded_change` proposes changes before generic activation observers. External changes are silent. Collapsed middle items remain mounted in stable hidden shells; their Owners remain live. If closing the path would hide the actual focused middle item, the widget requests focus on the disclosure button. No focus is inferred merely from a queued focus request.

## Pagination

Pages are positive `u64` values. Known totals must be positive and contain the current page. Initial invalid state fails before creating nodes. Dynamic invalid page/total state reports a typed UI error while keeping the last accepted state. Every callback derives its proposed page from that accepted state, including after rejected source writes and before a pending source update is rendered. There is no silent clamping or mutation of a controlled source.

`page(source)` controls the page; `default_page` supplies the local initial page only without that source. `on_page_change` proposes a page before generic activation observers. Selecting the current page produces no change callback. Directional buttons have explicit First/Previous/Next/Last page names. Known boundaries disable the corresponding actual native buttons. `enabled(false)` disables every page control. Programmatic source changes are silent.

`total(None)` means unknown, independently from `.has_next(source)`. Unknown totals expose one current page plus the four directional controls, with Last disabled. Next depends only on explicit `has_next`, and is also disabled at `u64::MAX`. There is no inference or prefetch of subsequent pages.

`pagination_window` builds the finite snapshot in O(siblings + boundaries), with both budgets at most 8. It iterates only three bounded ranges and inserts gap markers between disjoint ranges, including at `u64::MAX`. At most 35 entries are returned, independent of total or navigation history. The component creates exactly 35 numeric/gap slots and four directional buttons once. Only the visible window participates in layout, focus, and accessibility; hidden slots retain no visited-page state. Physical slot identity remains stable as its displayed page changes. The current button uses `Current::Page` and themed emphasis.

## NavLink

NavLink contains an actual Anchor. Its optional ordinary disclosure button is a separate control: following the link never toggles children, and expanding children never follows the link. `.current(true)` claims the current page. `.highlighted(true)` changes presentation only. Browser visited styling remains browser-owned.

`on_navigate` uses Anchor's captured request path: the browser prevents native navigation before queuing the event, then the application receives the accepted destination. Without that callback the ordinary link retains browser default navigation. NavLink itself installs no route changes. Typed navigation/expansion callbacks run before generic activation observers.

`.children(factory)` creates a stable named panel shell related to its disclosure and link. `ContentPolicy::Unmount` is the default: closing retires child Owners, bindings, tasks, and state, and reopening constructs new children. `KeepMounted` hides children without retiring their state/tasks. Closing a panel that contains actual focus requests focus on the disclosure button. Controlled `expanded` proposals may be rejected; defaults never override controlled state.

## Evidence

`tests/navigation.rs` has 15 passing cases for bounded windows and overflow, initial/dynamic validation, accepted callbacks, local/controlled behavior, native disabled props, current semantics, stable breadcrumb folding, separate NavLink requests, Owner policies, retirement, and SSR. Shared `render/tests/current.rs` and HTML's full-shape differential oracle verify typed target rejection/reset and field-local mapping. Blitz's `accessibility_current_tests.rs` verifies Page/Step and reset independently from selection.

Chrome 155.0.8059.84 completed 9 browser reports with no errors in `D:/BuildCache/sapid/navigation-browser.json`: four-theme CSR/hydration, mouse/Enter/Space, fold focus restoration, controlled rejection, native disabled controls, finite huge/unknown totals, actual AX navigation landmarks, child retirement, and focused adoption. Screenshots are `navigation-*.png` in the same D-drive directory.

`examples/native/tests/navigation.rs` passes both native tests: four-theme real-engine layout/identity/disabled checks and a minimal hidden-panel/input-disposal regression. The latter exposed a stale layout-child cache, fixed in the genuine Blitz fork pinned at `32ffee48…`; the active regression now passes without bypasses. Evidence is `D:/BuildCache/sapid/navigation-native-fixed.log`.
