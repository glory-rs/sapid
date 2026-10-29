# OverflowList

`overflow_list(key)` packs a measured prefix of explicit current widgets and moves the remainder into the existing Popover. Its overflow entry is a real Button and Dialog; arbitrary content does not acquire Menu/MenuItem roles.

```rust
overflow_list("actions")
    .initial_visible(2)
    .item(overflow_item("home").content(|| button().label("Home")))
    .item(overflow_item("draft").width(160.).content(|| {
        text_field().name("Draft").style(
            Style::new().width(Length::Percent(100.))
        )
    }))
```

Each item has one short Owner and one widget factory invocation. Resizing, opening and closing overflow, and reordering retained keys move the same semantic range and editor. Removal retires that Owner; reinsertion creates a new identity. `.dynamic_items(keys, factory)` accepts only the bounded current registrations, rather than a business collection to enumerate or virtualize. Static `.items(iter)` consumes at most the 129th sentinel. The current limit is 128 unique, stable ASCII keys, each at most 64 bytes.

Intrinsic items are atomic, nonshrinking, nowrap inline boxes whose width must not depend on the available line width. `.width(px)` supplies a finite 0..=1,000,000 pixel box for flexible content such as an editor; percentage/fr policies are intentionally absent. An editor may use 100% inside that explicit pixel box. The container and trigger are physically measured; the capacity calculation accounts for item widths, gaps and the actual overflow entry. The overflow entry remains present while it owns actual focus, even if the content could otherwise fit without it.

Hidden items move temporarily into an inert, transparent measurement lane. The lane retains a layout box; opacity zero is not treated as hidden. Current visible/focused items stay in their presentation parent while observed. Resize of the container, visible widgets and trigger coalesces into one Owner task. `.measurement_revision(source)` or `OverflowHandle::refresh()` explicitly invalidates measurements after application changes to hidden intrinsic text/fonts. There is no observer of an arbitrary business source and no visited-item cache.

At most 128 item bounds plus container and trigger bounds are pending. Focus confirmation is serialized after the measurement batch. Closing, membership changes and Owner retirement revoke the obsolete observations. Measurements, user factories and destructors execute outside registry borrows. A matching actual FocusWithin observation authorizes moving focus to More when its editor is moved into closed overflow; the queued intention is confirmed or expired by a bounded physical Focus query.

`.initial_visible(n)` determines SSR and the first client shape before physical observation. `.visible_count(source)` controls the accepted prefix; `.on_visible_count_change` proposes measured capacity and cannot write that source. Source updates are quiet. `.open(source)` and `.on_open_change` use the same controlled rule. Local open state clears when every item fits; controlled open intent remains application-owned. KeepMounted Popover content preserves dirty values and selection. Shared Layer behavior supplies Escape, outside dismissal, keyboard focus and return focus.

All four themes and `.unstyled()` are supported; structural flex/clip/inert declarations remain in unstyled mode, while explicit root declarations preserve other theme fields. Application container styles should provide a suitable maximum width when the strip is inside a responsive page.

Native supports the default anchored, nonmodal Dialog Popover through the real Layer host. Four-theme acceptance drives its actual measurement tasks, opens and closes the engine top layer, checks anchored viewport bounds and editor hit testing, and rasterizes the themed popup. Moving the same editor from popup to the widened strip preserves engine identity, value, backward selection and actual focus; each factory runs once and clearing the component retires its Owner and pending observations. `.overflow(false)` deliberately renders an ordinary inline row. Its separate four-theme acceptance retains intrinsic text growth, a fixed 140px editor, container resize and engine identity. Native Layer limits still apply: this evidence does not cover Sheet/edge placements, Layer-local translation, OS keyboard input or platform accessibility clients.

The popup acceptance exposed an engine defect when a connected widget moved out of a closed popup into the measurement lane: the old hidden flex container retained a stale layout-child reference. The genuine `glory-rs/apace` fix at `c127fd9699a89ff8ecba0c8d8c9eccb3330627af` retires only derived references crossing the moved subtree boundary. Fork-owned regressions cover incremental/full layout, flex/grid/mixed block owners, anonymous boxes, contents, out-of-flow descendants, repeated moves and same-parent reorder, including retained editor selection and preedit. The original failures remain in `D:/BuildCache/sapid/final-native-acceptance-overflow-failure.log`, `overflow-list-native-popup-tests.log` and `overflow-engine-baseline.log`. The actual pinned native tests and strict Clippy passed in `overflow-list-native-popup-fixed.log` and `overflow-list-native-popup-clippy.log` under the same directory.

Evidence: 10 focused Rust lifecycle/budget/query tests, two finite packing unit tests, deterministic four-theme SSR, strict Clippy for component/Web/native, and two native acceptance tests each covering all four themes. Chromium 148.0.7778.96 passed all nine real browser scenarios against fixed root21, with `errors: []`; the raw report is `D:/BuildCache/sapid/overflow-list-browser-root21.json`. Every theme/CSR/adoption run measured prefix 2 initially, 1 at 240px and all 5 at 640px. The actual editor kept identity, dirty value, selection and focus through Popover-to-strip moves; narrowing transferred actual focus to More, Escape returned it, and closed content was absent from Chrome AX. Reordering kept five factories; removal/reinsertion retired/recreated one Owner. Controlled refusal, hidden-label invalidation, mobile geometry and dirty SSR adoption also passed. Four adoption screenshots were saved; the green/dark 390px screenshot was visually checked.
