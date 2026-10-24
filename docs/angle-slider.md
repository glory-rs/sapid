# AngleSlider

`angle_slider(key)` is a single accessible Slider with a circular degree display and a real two-dimensional pointer mapping. Zero is at the top and degrees increase clockwise. The finite domain is `0..=360`: 0 and 360 are distinct numeric endpoints at the same visual position. Values wrap only after crossing an endpoint, so Home reaches 0, End reaches 360, and successive increases from 359 produce 360 then 1.

```rust
let heading = scope.cage(90.0);
field("heading")
    .label("Heading")
    .description("Degrees clockwise from the top")
    .form_name("heading")
    .control(move |context| {
        angle_slider("heading")
            .field(context)
            .value(heading)
            .step(1.0)
            .page_step(15.0)
            .on_value_change(move |next| heading.revise(|value| *value = next))
    })
```

Omitting `.value(...)` uses local `.default_value(...)` state. Controlled sources remain read only to the component; callbacks propose values and the application chooses whether to accept them. Same-tick accepted proposals accumulate, including when acceptance happens only in the commit callback. Invalid initial configuration creates no nodes; invalid later values or steps preserve the accepted descriptor and cancel the current gesture. External valid changes are silent. Typed change/commit callbacks precede generic observers.

The default value is 0, step 1 and page step 15 degrees. Values must be finite within the domain, and both steps must be finite, positive and below 360. Keyboard increments preserve the requested rotational delta even when the step does not divide a full turn: step 270 yields 0 → 270 → 180. Pointer proposals snap to the nearest step from zero, with both endpoints permitted. Program values may remain off-grid. These are `f64` quantities, without a decimal-exactness promise.

Left/Right use the horizontal Slider keyboard contract and respect `.direction(Direction::Ltr/Rtl)`. PageUp/PageDown rotate by the page step, and Home/End select the endpoints. Up/Down are not an additional angle-specific contract. Readonly retains the real tab stop and refuses changes; disabled removes sequential focus. Default accessible value text is `"90 degrees"`; `.value_text(None)` explicitly clears it and `.value_text(Some(...))` supplies an application label. Explicit value text is limited to 1024 UTF-8 bytes and validated before cloning.

Pointer direction comes from `atan2(dx, -dy)` around the current measured center, using both coordinates rather than the track's horizontal fraction. Clicking or touching the exact center preserves the most recent valid direction in that gesture, or the current value when the gesture has no direction yet. Captured move/release outside the circle still supplies an angle. RTL changes keyboard direction, while the physical clockwise dial remains clockwise. The shared controller keeps one pending Bounds query and a bounded start/latest gesture record, resamples live bounds on subsequent input, and cancels requests on invalid configuration, resize, disable/readonly or Owner retirement. Late responses cannot revive a canceled gesture. Cancellation emits no commit. The shared host supplies `touch-action:none`.

Three passive typed Graphics draw the ring, hand and thumb. Each has at most six path commands. Their common square coordinate system scales uniformly and centers in a non-square box, preserving the circular shape. The root keeps the actual Slider role and focus; decorative graphics are presentational to accessibility. Five sizes, themes, class/style overrides and Field name/description/error/required/readonly metadata use the same accepted control state as Slider. `unstyled()` retains the default diameter; absent dimensions recover defaults, while explicit `Auto` and custom dimensions remain intact. No native range input, implicit form submission, routing or HTTP is installed. Field form metadata is an explicit successful-control snapshot, rather than automatic inclusion in browser `FormData`.

Evidence: eight Sapid integration tests and three mapping tests pass, covering endpoints, wrapping, nondivisor steps, same-tick commit-only acceptance, controlled rejection, invalid recovery, Field relations, explicit text clearing, budgets, Owner request cleanup and SSR. Two native tests pass across all four themes: real Bounds and non-square geometry drive all four cardinal directions and the undefined center, preserving focus and engine identity; Raster pixels verify the rotating hand is actually painted. Three additional shared component geometry tests cover all five sizes, both linear axes and dynamic dimensions; a native geometry test verifies the unstyled range family reserves real stack space without overlapping the next editor. Native evidence is saved in `D:/BuildCache/sapid/angle-geometry-native.log`. Production, browser fixture and these native tests pass strict Clippy. The shared Slider keyboard/pointer/AX evidence remains applicable to the reused protocol.

The unified **Web18** snapshot passed **10/10** actual browser cases on Chromium **148.0.7778.96**, with no page/console errors. Eight cases cover all four themes in CSR and hydration, exact degree endpoints and wrapping, nondivisor steps, RTL keys, a same-frame commit-only pair from 359 to 1, controlled rejection, all cardinal pointer directions, center preservation, captured release outside the circle, fresh measurements after a typed aspect-ratio resize, Field errors/readonly/disabled, successful snapshot metadata and stable unrelated editor/caret. Extra cases verify dirty-editor hydration and Owner cleanup, plus Chromium CDP touch start/move/end/cancel without scroll or selection changes or a cancellation commit. Reports are `D:/BuildCache/sapid/angle-slider-browser-root18.json` and `.log`; screenshots are `angle-slider-{hue}-{scheme}-{csr|hydrate}.png`. The green/dark hydration screenshot was visually inspected. This does not claim physical-touchscreen or screen-reader announcement acceptance.
