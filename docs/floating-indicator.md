# FloatingIndicator

`floating_indicator(key)` draws a passive outline around the current registered widget. It does not select, focus, reorder or activate a control. Actual buttons, editors and other children retain their own roles and behavior; the target wrapper is an unnamed generic View and the Graphic has no accessible alternative.

```rust
floating_indicator("workspaces")
    .active(active_key)
    .content(|context| {
        row().fill((
            floating_indicator_target(context.clone(), "projects")
                .fill(button().label("Projects")),
            floating_indicator_target(context, "teams")
                .fill(button().label("Teams")),
        ))
    })
```

The current registry holds at most 128 targets with unique 1..=64 byte ASCII keys. It does not enumerate an application's business source. Removing a target retires its registration; the same key later receives a new non-recycled registration ticket. The current active key can be absent, in which case the decoration is hidden. Pending replies belong only to the current ticket/target and are revoked on replacement or retirement.

One Owner task coalesces refreshes; one batch requests exactly two boxes: the zero-padding/border measurement origin and current target. The component subtracts their physical viewport origins and paints the resulting local rectangle. Zero-sized/hidden boxes suppress the outline, while nonfinite geometry or local coordinates outside +/-1,000,000px reject explicitly. `Scope::contains_node` checks current staged semantic ancestry, including external Place, instead of trusting the target's original construction Scope. A target moved outside its area is hidden and rejected.

The root and target Resize interests refresh dimensions. `.measurement_revision(source)` or `FloatingIndicatorContext::refresh()` invalidates position after application-owned reordering, nested scrolling or direction changes that keep box sizes unchanged. The component does not add browser listeners or maintain a history of visited targets.

The outline is a fixed five-command decorative Graphic using the shared finite `GraphicTransition`. `.duration(millis)` accepts 0..=500ms; zero and `.animated(false)` clear current motion. First observation and reappearance position immediately; subsequent accepted geometry can interpolate, interrupt and reverse. Browser Reduce changes and the native host's motion preference use the existing renderer policy. Native defaults to Reduce, and explicit native clock advancement paints intermediate paths without per-frame Cage writes. No OS preference or display completion is inferred.

The four themes supply an accent stroke. `.color(source)` overrides it independently and `.thickness(px)` accepts finite 0.000001..=16px. `.unstyled()` keeps registration, measurement and pointer-transparent geometry; callers supply `.color` for visible ink. User root styles preserve unspecified theme declarations. The internal absolute marker and fixed-size origin isolate coordinate policy from outer padding/borders.

SSR has no physical observation, so it emits the same hidden, unmeasured marker as the first client shape. Adoption validates this shape before initial measurement; dirty editor values, focus and selection remain owned by the original controls.

Evidence: five component lifecycle/finite/query/ancestry tests, two local-coordinate model tests, four shared staged ancestry tests, and strict component/Web/native Clippy. Two real native tests cover four-theme geometry, raster interpolation, engine identity, actual editor focus, the default Reduce policy and hidden/reappearance motion cancellation (`D:/BuildCache/sapid/floating-native-final.log`). All nine browser scenarios passed in Chromium 148.0.7778.96 on the same-source `layer-next` snapshot: four-theme CSR/adoption, intermediate paths, reversal, live Reduce, editor identity/selection/focus, current target retirement/re-registration, physical direction changes, mobile bounds and dirty pre-adoption input. The initial SSR marker is hidden and agrees with the first client shape; measurement begins after adoption. Raw results: `D:/BuildCache/sapid/floating-indicator-browser-layer-next.json`; representative mobile image: `D:/BuildCache/sapid/floating-indicator-green-dark-hydrate.png`. These checks establish renderer motion and actual browser/native focus, not OS preference synchronization or screen-reader announcements.
