# Slider

`slider(key)` is a single custom numeric slider with a real Slider role. It shares the finite range command and captured pointer protocol with Splitter, while retaining its own role, axis and value semantics. It does not install routing, HTTP or a native HTML form submission policy.

```rust
let amount = scope.cage(25.0);
field("amount")
    .label("Amount")
    .description("Choose units")
    .form_name("amount")
    .control(move |field| {
        slider("amount")
            .field(field)
            .min(0.0).max(100.0).step(0.5).page_step(10.0)
            .value(amount)
            .on_value_change(move |next| amount.revise(|value| *value = next))
    })
```

`.value(...)` is controlled and read only to the component. Omitting it uses `.default_value(...)` as local state. Initial and later program values must be finite and inside `min..=max`, with `min < max`; invalid initial configuration creates no nodes, and an invalid later source preserves the last accepted descriptor. Source updates are silent. A rejected controlled request leaves the displayed value and source unchanged. Typed change and commit callbacks precede generic Range/Drag observers.

The default range is 0–100, initial value 0, step 1, page step ten steps. Bounds and steps are limited to absolute 10^12, with positive finite steps. The optional nonblank value text is limited to 1024 UTF-8 bytes and is checked by borrowing before cloning. Pointer and keyboard proposals snap to the nearest step from the minimum, permitting both endpoints; program values can remain off-grid. Values are ordinary `f64`, not exact decimal quantities. A tiny step that cannot change the representable value emits no fictitious numeric progress.

Horizontal Left/Right follow `.direction(Direction::Ltr/Rtl)`. Vertical Up increases and Down decreases, independently of text direction. Home/End reach bounds; PageUp/PageDown use the page step. The other axis, editable origins, composition and modifier chords retain their ordinary behavior. The component owns one tab stop; readonly keeps focus while refusing changes, and disabled is removed from sequential focus. Field labels, descriptions, required, readonly, invalid, errors, theme and size are carried by the actual slider node.

Pointer down focuses the slider and starts a gesture immediately, so clicking the track changes the value. The host captures the pointer and routes release to the original slider even outside its bounds. Horizontal RTL reverses the coordinate; vertical ranges increase upwards. One live Bounds request and a bounded start/latest intent record serve each gesture. Move and final release resample current geometry, while intervening moves coalesce. Resize, invalid or external model configuration, disable/readonly and Owner retirement cancel the pending request. Late replies cannot mutate a later gesture. Cancellation never emits a commit. The semantic host contract supplies `touch-action:none` for the slider; it does not add a general application style escape hatch.

Track, fill and thumb are passive typed View visuals. There are no independently focusable children. `unstyled()` removes the theme decoration while retaining required geometry and behavior. All five sizes are supported. `.style(...)` can set explicit dimensions; a zero-size unmeasurable track rejects pointer geometry rather than inventing a value.

Form names and accessible names remain separate. The custom slider exports `FormControl` snapshot metadata through Field, with the current numeric text when no explicit value was supplied. HTML uses `data-gly-control-name/value`, rather than pretending that a div is a native successful input. Native `FormData` does not automatically include this custom control: connect the explicit FormState/successful-control snapshot policy when submitting. Files are not a Slider capability.

Evidence: 8 Sapid Rust tests cover local and controlled same-tick proposals, callback order, invalid-source recovery, Field relations and readonly/disabled behavior, request cancellation, Owner retirement and SSR mapping. Two native component tests exercise all four themes with horizontal LTR/RTL and vertical real Bounds, Focus, pointer-coordinate mapping, accepted/rejected models and stable engine identity. Four shared render tests cover finite payloads, exact origins, readonly cancellation, passive descendants and keyboard equivalence. The native input driver test exercises queued RTL/Page keyboard input and captured Mouse/Finger start, move, release and cancellation. A separate AccessKit test verifies the Slider role, range, numeric step, value text, required state, readonly action policy and presentational descendants. The full HTML field-delta differential oracle passes with Slider metadata and its explicit control snapshot fields.

The standalone browser acceptance suite passed **10/10**, with no console/page errors, on Chromium **148.0.7778.96**. This includes four themes × CSR/hydration, mouse track clicking and release outside bounds, Field label focus/errors, controlled rejection, RTL/vertical keys, Page/Home/End, readonly/disabled state, stable unrelated editor selection, explicit snapshot attributes and typed callback order. Additional cases verify a dirty editor before hydration with Owner cleanup, and real Chromium CDP touch start/move/end/cancel without scrolling the page, changing editor selection or committing a cancellation. This is actual browser/synthetic native-driver input evidence; it does not claim physical touchscreen or screen-reader announcements.

The independent Web build is `D:/BuildCache/sapid/slider-next`, generated from the same fixture source with a D-drive-only probe crate while the unified root output remained frozen. Reports are `D:/BuildCache/sapid/slider-browser.json` and `.log`; four-theme screenshots are `slider-{hue}-{scheme}-{csr|hydrate}.png`. The final green/dark hydration screenshot was visually inspected. Production, browser fixture, native acceptance test and shared native library/tests pass strict Clippy. The unified build will record its separate same-source rerun rather than replacing this independent provenance.
