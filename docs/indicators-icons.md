# Indicator and ThemeIcon

`indicator().fill(children)` places a passive dot around independent content.
`IndicatorPosition` covers the nine top/middle/bottom and left/center/right
anchors. The explicit square diameter is finite, 4..=128 pixels (default 12).
The center of the marker meets the selected edge/corner. The caller reserves
space around boundary markers; the content and marker keep separate identities.

An unnamed dot has no image, status or interactive role. `label(text)` adds an
ordinary text label (nonempty, at most 32 UTF-8 bytes), and `name(text)` optionally
names its passive group (nonempty, at most 256 bytes). No live announcement is
inferred. `visible(Lotus<bool>)` hides only the marker, preserving child controls
and their values. Invalid controlled text leaves the last accepted presentation
in place and reports an error; recovery preserves identities without writing the
source. There are no implicit callbacks. Root and marker node references are
available separately. Labels use the caller's chosen fixed diameter.

`theme_icon(Icon::{Check,Close,Info,ArrowRight})` draws finite vector geometry in
a themed passive frame. The icon source may be reactive. `name(text)` provides
the actual graphic's accessible image alternative; omitting it makes the graphic
decorative. The surrounding frame stays generic, and the component adds no click
or keyboard behavior. Use an actual Button/ActionIcon for an interactive icon.
`ThemeIconVariant::{Soft,Solid,Outline}`, all five shared sizes and all four themes
provide their normal tokens. `color(Color)` explicitly overrides path color.

Both components preserve custom classes and explicit Style precedence. Unstyled
Indicator retains its essential anchor geometry and omits theme decoration;
unstyled ThemeIcon retains real path content and omits frame decoration.
ThemeIcon uses Glory's typed Graphic protocol, documented in
`../../glory/docs/finite-graphics.md`. Native rendering requires Glory's opt-in paint
feature and its supported graphic box declarations; there is no resource loader,
SVG string parser, font glyph dependency or application DOM injection.

Evidence: `tests/indicator.rs` has five cases covering all nine anchors and four
themes, initial budgets, passive semantics, independent controls, controlled
rejection/recovery, identity and unstyled overrides. `tests/theme_icon.rs` has
three cases covering all four paths, three variants, four themes, names,
decoration, updates and invalid source recovery. The shared host fixture is
`examples/web/src/graphics_form.rs`; native geometry and engine widget installation
are checked by `examples/native/tests/graphics.rs` (four themes passed). Chromium
148.0.7778.96 passed eight CSR/hydration cases and one classic-stream case; the
actual report is `D:/BuildCache/sapid/graphics-browser.json`. Hydration screenshots
are stored beside it. The stream producer test supports the explicit environment
variable `GLORY_GRAPHICS_STREAM_FIXTURE`; its browser test checks actual SVG
creation, path patches, alternatives and End/controller release. This does not
establish screen-reader announcements.
