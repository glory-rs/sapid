# HueSlider and AlphaSlider

`hue_slider(key)` uses a finite 0..=360 degree HSV hue domain (step 1 by default).
`alpha_slider(key)` uses 0..=1 opacity (step 0.01). Both reuse the actual Slider
controller, semantic role, keyboard commands, captured pointer/touch input, RTL,
query coalescing/cancellation, controlled cursor, Field relations and Owner
lifetime. They retain f64 channel precision; preview quantization never writes
back to the source. Hue endpoints are both red, with ordinary clamped Slider
endpoints; use AngleSlider for circular keyboard wrap behavior.

```rust
use sapid::{alpha_slider, hue_slider};
use glory::style::Color;

hue_slider("hue").default_value(120.123456789).step(0.25);
alpha_slider("opacity").color(Color(100, 150, 200, 255)).default_value(0.5);
```

Controlled `.value` sources remain read-only inputs. `.on_value_change` proposes
the new number, followed by `.on_value_commit` at the shared Slider boundary.
Local defaults accept changes; controlled applications decide whether to accept
either callback. Same-tick commit-only acceptance retains the shared cursor.
Invalid program sources leave the last accepted control and preview intact;
later valid values recover. A Field labels the real Slider and retains required,
readonly, disabled, error and explicit successful-snapshot metadata. These custom
sliders use the documented Slider snapshot attributes; they do not contribute
native browser FormData entries. Keys require at most 58 ASCII bytes, leaving
space for independent hue/alpha semantic namespaces.

The trail is a bounded passive visual: 120 HSV color samples for hue, 64 alpha
samples over a genuine 16×4 white/gray checker for opacity. Samples are explicit
typed boxes, so Web/SSR/Blitz share the same finite representation without author
CSS gradients or a new host paint promise. This is a sampled trail rather than a
continuous gradient shader; pointer geometry and channel precision remain
continuous and independent of sample count. Alpha ignores the supplied Color's
alpha and keeps its RGB. The thumb previews the current quantized RGBA8 value.
`unstyled()` retains the color data and geometry while removing theme decoration.

The pure `Hsva` model validates hue 0..=360 and saturation/value/alpha 0..=1,
rejecting nonfinite coordinates. Conversion uses HSV over sRGB channel values;
it is not a perceptual or linear-light color space. `to_rgba` quantizes only the
preview. `from_rgba` preserves every RGBA8 channel on round trip, including RGB
under zero alpha; achromatic colors decode with hue zero. A picker preserving a
chosen achromatic hue should keep its original HSV model. `parse_hex_color`
accepts exactly #rgb/#rgba/#rrggbb/#rrggbbaa; oversized input is refused before
scanning or allocating. Formatting produces lowercase long RGB or RGBA hex.

Evidence includes 6 pure-model tests, 7 component tests and one native test
spanning all four themes, actual color/checker pixels, focus, typed keyboard
input, base-RGB updates, real bounds and engine identity. Chromium 148.0.7778.96
passed all 10 Web18 acceptance scenarios: four themes in CSR and hydration,
controlled refusal, commit-only acceptance, capture outside the track, RTL,
readonly/disabled behavior, dirty editor adoption and genuine touch input.
The archived report is `D:/BuildCache/sapid/color-sliders-browser-web18.json`.

An actual browser geometry regression exposed an unstyled zero-height Slider.
The shared geometry now supplies missing default dimensions while preserving
explicit `Auto` and custom dimensions. Three model geometry tests, one native
test over five sizes, and browser width/height and no-overlap assertions cover
this repair. The pre-fix failure is retained in
`D:/BuildCache/sapid/color-sliders-unstyled-web17-before-fix.log`.
