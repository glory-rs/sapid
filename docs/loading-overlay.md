# Overlay and LoadingOverlay

`overlay().fill(content)` adds a decorative local shade over retained content.
Its mask has no accessible role/name, busy state or interaction controller and
uses typed pointer-events None. The content remains operable with pointer,
keyboard and assistive technology. `visible(Vary<bool>)` hides only the shade.

`loading_overlay(loading).fill(content)` is an explicit busy region. The same
controlled source drives the parent's busy metadata, the retained content's
inert metadata and the mask's visibility in one flush. The independent spinner
remains outside the inert content and has a localized `label`, default "Loading".
`name` optionally names the content region. It suppresses input and accessibility
exposure while preserving current values, owners and identities; it does not
cancel resources/tasks, trap focus, request modal behavior or restore focus.
Unblocking does not focus the previous editor automatically. Queries and
programmatic scrolling remain available.

The wrapper supplies a positioned containing block. Provide content or explicit
dimensions; the mask does not contribute height. Explicit Static positioning is
rejected. Relative, Absolute and Fixed use the host's typed positioning contract.
The mask fills the region using zero top/right/bottom/left offsets. Region and
mask references are available through node_ref/content_ref/mask_ref. Theme,
unstyled, custom root class/style, color and opacity are supported. Opacity is
finite in 0..=1 and applies only to the shade's RGBA8 alpha, preserving the
spinner's full opacity; color alpha and opacity compose with byte rounding.
Unstyled omits the default surface tint, retains geometry and blocking behavior,
and paints the spinner using the theme foreground. Explicit color still applies.
Names and status labels are nonempty and at most 256 UTF-8 bytes. Invalid initial
values create no nodes; invalid controlled names/style position retain the last
accepted UI without rewriting the source.

Blocking uses Glory's shared inert protocol, with real HTML inert, ancestor input
guards and native focus/Tab/editor/AccessKit behavior. Web top-layer elements also
receive effective inert themselves, preserving the region boundary when a nested
dialog is already open. Automatic modal background restoration preserves semantic
inert. Native layers keep their existing explicit unsupported boundary; the
native loading region itself supports ordinary controls and real graphics.
Spinner animation follows the host motion preference, including the native
static Reduce default. This is not a generic wheel/scroll lock.

Four component tests cover source silence, coordinated busy/inert/visibility,
retained identity, decorative behavior, all four themes/unstyled, budgets and
invalid update recovery. `examples/native/tests/loading_overlay.rs` passes two
tests including all four themes. It measures actual 120×100 region/mask geometry,
real half-alpha shade pixels, a fully opaque spinner, native mask hit blocking,
pointer-transparent decorative pixels and restored hits on the same button.
Evidence is `D:/BuildCache/sapid/loading-overlay-rust.log` and
`D:/BuildCache/sapid/loading-overlay-native.log`. Glory's focused native tests
separately prove caret/edit/Tab suppression, AX branch retirement/new IDs,
unchanged engine identity and permitted programmatic scrolling. Shared tests,
the field oracle and strict Clippy pass.


`examples/web/src/loading_overlay_form.rs` and
`examples/web/loading-overlay-acceptance.js` pass all nine actual browser scenarios
on Chromium 148.0.7778.96 against unified Web12, with no browser errors:
`D:/BuildCache/sapid/loading-overlay-browser.json`. They include real pointer
passthrough, queued activation suppression, keyboard/focus and Chrome AX exclusion,
independent modal restoration, retained top-layer inert, mobile layout, actual
stream End and WASM hydration without replacing the layer. The four hydration
screenshots are `loading-overlay-{blue,green}-{light,dark}-hydrate.png` under the
same build root; the green/dark screenshot was visually inspected.
