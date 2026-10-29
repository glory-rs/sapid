# ColorPicker and ColorInput

`color_picker(key)` edits one finite `Hsva` value. A 16×16 sampled saturation /
brightness plane supports immediate mouse and touch capture, including release
outside the plane. Saturation and Brightness are also genuine keyboard Sliders;
Hue and optional Opacity reuse the existing color sliders. The plane itself has
no invented two-dimensional accessibility role or extra Tab stop. Pressing it
preserves an adjacent editor's actual focus and selection. RTL mirrors its
saturation axis; brightness still increases upwards.

```rust
use sapid::{color_picker, color_input, Hsva};
color_picker("accent").default_value(Hsva {
    hue: 210., saturation: 0.6, value: 0.8, alpha: 0.5,
});
color_input("accent-hex").default_value("#6496c880");
```

Picker `.value(...)` is a read-only controlled source. `.on_value_change` and
`.on_value_commit` propose complete `Hsva` values; local defaults accept changes.
Refusal leaves the source and rendered channels intact. Same-tick accepted edits
read the live source, preserving changes to the other channels. Program writes
are silent; invalid/nonfinite values preserve the accepted tree and return an
error until a later valid value recovers. Coordinates retain f64 precision;
RGBA8 conversion quantizes only visual previews. Hue zero and 360 remain distinct
numeric endpoints, and achromatic values retain the supplied hue.

The plane contains exactly 256 passive typed color boxes, independent of pointer
history. Its side is finite in 128..=512 pixels, default 240. This is a sampled
sRGB HSV surface; it does not claim a continuous gradient shader, a perceptual
color space or gamut conversion. One pending geometry query and one current
start/latest pointer batch belong to the live gesture. Cancellation, disabling
and Owner retirement revoke pending work. No visited-color cache is retained.
`unstyled()` preserves physical geometry, color content and keyboard alternatives.

`color_input(key)` has one String source. It preserves incomplete and invalid
hex drafts and links a parse error to the real single-line TextField without
resetting that control. Accepted forms are exactly #rgb, #rgba, #rrggbb and
#rrggbbaa; drafts have a 256-byte canonical single-line budget. Empty text stays
empty. `.on_validation` reports Empty, Valid(RGBA8) or Invalid(ColorError), while
business changes remain proposals through `.on_value_change`. Field naming,
descriptions, required/readonly/disabled state and successful-control name belong
to the actual editor. A Field's own error takes precedence over the parse message.

The default Choose color button opens the shared Popover with its existing
focus, dismissal and restoration rules. Palette changes propose lowercase long
RGBA hex; `.show_alpha(false)` emits RGB hex. Converting back to an editable hex
string intentionally quantizes channels to RGBA8. An invalid draft leaves the
palette at its last valid color. Active composition or readonly/disabled state
blocks palette changes. Controlled `.open` uses `.on_open_change` proposals.

Native Apace supports the standalone picker, its real Slider alternatives and
ColorInput's default anchored Popup through the finite Dialog/Popover layer
facet. Four-theme native tests verify actual anchor geometry, palette changes,
CPU pixels, retained editor identity and closing. Explicit `.show_picker(false)`
continues to select the hex-editor-only mode and rejects simultaneous popup
configuration. The standalone ColorPicker has
no native form entry; ColorInput submits the real editor value through Field.

Keys are stable ASCII, at most 48 bytes for Picker and 46 for Input, reserving
space for internal channel/popup namespaces. Five sizes affect the shared channel
controls; plane geometry is configured independently with `.plane_size(...)`.
Names and class/style overrides preserve the established component conventions.

Evidence: eleven component regressions pass, including controlled refusal and
commit-only acceptance, exact cross-channel updates, invalid recovery, linked
errors, Field state, composition and Owner cleanup. Native tests cover four-theme
actual plane pixels, two-axis native bounds/mapping, channel updates, stable
engine identities, the real hex editor and explicit top-layer refusal. Shared
DragCapture has two render regressions, an HTML full-shape differential oracle,
a native queued Mouse/Finger focus/capture/cancel test and a classic stream
Shell/Patch/End test. The fixed Web19 artifact passed eleven actual Chromium 148.0.7778.96 scenarios,
including four themes in CSR/hydration, two-axis mouse capture, controlled
refusal, readonly/disabled and composition policy, actual AX roles, dirty editor
adoption, CDP touch/cancellation and classic stream final metadata. The report is
`D:/BuildCache/sapid/color-picker-browser-web19.json`. Visual review then found
that a 128-pixel plane retained 240-pixel channel widths and that the indicator
needed centering. Both repairs passed strengthened actual native bounds checks and all eleven
browser scenarios against the fixed `command-next` WASM/SSR artifact; see
`D:/BuildCache/sapid/color-picker-geometry-native20.log` and
`D:/BuildCache/sapid/color-picker-browser-geometry20.json`. The initially omitted
DragCapture classic support page was generated with its Rust v5 producer before
the final complete replay; no assertion or production behavior was removed.
The final green/dark hydration screenshot was visually reviewed: 128-pixel
channels now fit their plane, and the indicator centers on the actual value.
Browser protocol and AX-tree inspection do not establish OS IME or screen-reader
announcements.

The native consumer migration passed all seven alerts/color tests in
`D:/BuildCache/sapid/native-new-layer-consumers-final.log`. It does not extend
native support to Menu/Tooltip/Listbox, Sheet or ContextPoint layer facets.
