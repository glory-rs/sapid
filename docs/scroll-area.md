# ScrollArea and Scroller

`scroll_area(key)` creates a real focusable scrolling View with one stable content wrapper. The default axis is vertical, keyboard step is 40 logical pixels, viewport height is 240 pixels, and width is 100%. Explicit styles can override dimensions. `Scrollbars::{Auto, Always}` use the host's real `overflow:auto` and `overflow:scroll` policies. Scrollbars retain their browser/platform appearance; this API does not simulate a draggable custom thumb or hide native scrollbars through untyped CSS.

```rust
use sapid::scroll_area;
use glory::style::{Length, Style};

scroll_area("documents")
    .name("Documents")
    .style(Style::new().height(Length::Px(320.0)))
    .content(|context| {
        // context.scroll is a fallible, Owner-scoped ScrollHandle.
        sapid::text().theme(context.theme).content("Document content")
    });
```

The content factory runs once. Viewport/content refs, descendant input identities, selection, and child Owners remain stable while scrolling. The component does not enumerate an application collection or virtualize its data. Use Glory's windowed list for large dynamic collections; message anchoring and bottom-follow policy belong to the later MessageScroller task.

## Observations and requests

`ScrollHandle::metrics()` returns a read-only Bond of the most recent actual `ScrollMetrics`, initially None. The shared query records offset x/y, viewport width/height, and content width/height at one accepted watermark. All six values must be finite; dimensions must be nonnegative. Content may be smaller than the viewport. Negative RTL/elastic offsets remain intact in observations. Browser reads use actual `scrollLeft/Top`, `clientWidth/Height`, and `scrollWidth/Height`. Apace reads its resolved engine's corresponding client/content extent methods. No content-wrapper measurement is substituted for the host's complete overflow range.

`refresh`, `scroll_to`, `scroll_by`, and `command` are fallible explicit requests. The current shared assignment contract is nonnegative; negative RTL/elastic positions are observable, while commands from that domain report Unsupported. Horizontal commands have an LTR assignment boundary. This API does not claim full RTL scroll-origin normalization. Values must be finite, absolute requests nonnegative, and commands cannot use excluded axes. The configured step must be positive, finite, and at most 1,000,000.

Each pending batch stores at most 128 finite intents and starts one metrics query. Additional commands join that FIFO until its response arrives. Each command clamps against the previous command's result, preserving boundary behavior; opposite directions are not collapsed into a sum. The batch emits one final typed ScrollTo assignment. Metrics publish only actual host observations, not predicted assignment coordinates. Resize and Scroll observations refresh metrics, and content Resize requests fresh complete host extents. There is no visited-node history or retained business-data collection.

`on_scroll_change((x,y))` runs before generic Scroll observers. `on_metrics_change` observes the asynchronous complete metrics reply; it is separate from immediate event notification. `on_ready` exposes the handle after construction. Clearing the component cancels its Owner-scoped requests, clears refs and callback captures, and makes retained handles return Gone. Late responses cannot restore retired state.

## Keyboard, nesting, and Scroller

Shared `Semantics.scroll_keyboard` opts into `ScrollAxes::{Vertical, Horizontal, Both}`. With keyboard enabled, the viewport is a normal Tab stop. Hosts consume Arrow keys, Home/End, and PageUp/PageDown only when that exact viewport owns actual focus. Descendant editors, buttons, links, and nested viewports retain their own input behavior. Modified keys, composition, Space, Enter, Tab, and printable characters are not reinterpreted as scroll commands. Repeated arrow/page keys remain valid. Home/End and Page keys use the vertical axis for Both, or the explicitly selected single axis. Disabling `.keyboard(false)` leaves a programmatically focusable viewport and native pointer/wheel scrolling.

Nested viewports use ordinary host scrolling and wheel chaining. There is no automatic overscroll containment or interception of ancestor wheel events. The shared ScrollKeyboard event targets only its opted-in physical origin; collection Orientation and roving-focus rules remain independent.

`scroller(key)` reuses the same viewport and adds actual named Previous/Next scroll-page buttons. It defaults to horizontal; `.axis(Orientation::Vertical)` changes both button requests and viewport axis. `.area(|area| ...)` configures dimensions, appearance, refs, observation callbacks, and other ScrollArea properties; the Scroller owns its axis. Buttons derive native disabled boundaries from actual metrics and page by one measured client viewport. No route or carousel selection state is implied.

SSR records finite markup and event interests, returns asynchronous Unsupported for physical metrics queries, and invents no measurements. Browser hydration adopts the same viewport/content nodes; subsequent real observations supply metrics.

## Evidence

- Shared render: 3 tests validate all six numbers, negative offsets, small content, result kind, semantic targets/axes, and independent key mapping.
- Shared UI: a dedicated query test covers Owner cancellation, invalid result kinds/NaN, accepted query watermarks after later commits, and duplicate replies.
- Shared HTML differential oracle covers all three keyboard axes and resets; synchronous SSR and stream conformance cover the new query and event target rules.
- Shared Apace: actual client/content/offset query test and actual queued viewport-only keyboard/nested-editor test pass.
- Sapid: 4 integration tests and one ordered-clamping unit test pass; native 2 tests cover four themes with actual geometry, ordered assignments, descendant editor identity, real Scroller buttons, and disabled end boundaries.
- `examples/web/scroll-area-acceptance.js`: 9 passing suites on Chromium 148.0.7778.96 against unified Web6, including the final four-theme fixture. Real CSR/hydration checks cover actual metrics and page boundaries, Arrow/Page/Home/End, FIFO edge behavior, nested wheel/keyboard targeting, native descendant editing, actual Scroller buttons, input identity/drafts, callback order, dirty hydration scroll/selection retention, and Owner teardown. Results are saved at `D:/BuildCache/sapid/scroll-area-browser.json`, `scroll-area-browser.log`, and `scroll-area-*.png`; dark and light screenshots were visually inspected.
