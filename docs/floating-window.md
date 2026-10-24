# FloatingWindow

`floating_window(key)` creates a named nonmodal Dialog in the host's viewport
top layer. It has a real title bar, Close button, eight keyboard movement and
resize buttons, and captured mouse/touch movement and resize regions. It uses
the shared `LayerPlacement::Viewport` policy and `Action::RaiseLayer`; component
code does not manipulate DOM, paint order or document focus.

`WindowBounds { x, y, width, height }` is one atomic value. `.bounds(source)` and
`.open(source)` are controlled inputs; `on_bounds_change` and `on_open_change`
propose updates. Defaults apply only without those inputs. Rejected geometry
does not move the window, and same-tick accepted keyboard commands accumulate.
`WindowLimits` validates minimum/maximum dimensions before construction.
Coordinates are finite within ±1,000,000 CSS pixels and sizes within 1..=65,536.
Viewport padding is finite within 0..=4096; optional nearest-edge constraint
preserves dimensions, including when a window is larger than the viewport.

`.on_ready` supplies an Owner-bound `FloatingWindowHandle` with `command`,
`set_bounds`, `raise` and `close`. Retired handles fail. Raising preserves the
editor identity, caret and active composition and cannot cross a modal barrier.
KeepMounted is the default content policy; Unmount retires the content Owner.
Disabled windows use a real disabled Fieldset, including the content editors.
Appearance overrides preserve decoration; positioning and dimensions belong to
the bounds/limits API and conflicting style fields are rejected.

One current capture shares at most one initial Bounds query between movement
and resize. Replacing a capture cancels the old request; late old-origin events
cannot alter the new gesture. External geometry changes, refusal and Owner
retirement cancel pending work. No pointer history or business-source scan is
retained.

Web uses the document-owned Layer coordinator. Native uses the genuine pinned
engine top layer, with viewport layout, last paint/reverse hit testing and real
focus/AX admission; it preserves the logical DOM and Stylo parent chains. Shared
Viewport resolution and successful-prefix rejection rules are identical. The
Viewport was introduced in HTML8; the current protocol is HTML9 and classic
stream envelope version 2. The fixed Web28 evidence below uses HTML8.

Evidence: seven component regressions and one pure geometry model test pass
(`window-model-final2.log`, `window-model-tests.log`), with strict Clippy
(`window-strict-final.log`). Two native consumer tests cover four themes,
actual bounds, raster pixels, editor identity, geometry proposals, cancellation
and the one-query budget (`window-native-consumer-final2.log`). Ten shared native
Layer tests include Viewport and RaiseLayer editor/preedit preservation
(`window-shared-native-final.log`). Logs are in `D:/BuildCache/sapid`.

The original `examples/web/floating-window-acceptance.js` passes all thirteen
groups against fixed Web28, Chromium 148: eight theme/CSR/hydration combinations,
controlled refusal and disabled content, real layer order/modal barriers,
mouse/touch capture, viewport collision and resize, and Owner/query retirement.
The independent report is `D:/BuildCache/sapid/window-browser-web28.json`
(`errors: []`). The existing Popover thirteen-group suite also passes the same
artifact (`popover-browser-web28.json`). Browser composition protocol tests are
not evidence of OS IME or screen-reader acceptance.
