# Burger

`burger()` is a real named Button with synchronized accepted expanded and pressed
states. Its decorative Graphic contains exactly three Move/Line pairs, morphing
between parallel bars and a cross (the center line collapses at the intersection).
It uses no font glyph, injected SVG source, independent pointer/key listener,
per-frame Cage or component animation task.

`open(source)` remains read-only controlled state; `default_open` supplies local
state otherwise. Rejected proposals preserve both semantic state and the current
path. External source changes are silent. Local acceptance precedes
`on_open_change`, `on_press` and generic observers. Disabled activation remains
native. `labels(open, close)` supplies nonempty localized names and `controls(id)`
associates an application-owned panel; Burger does not create that panel or infer
actual focus.

`animated(source)` defaults true; `duration_millis(value)` defaults 180 and accepts
0–500. Explicit no motion and a live reduced-motion preference immediately cancel
an in-flight transition at the accepted endpoint. Browser paths interpolate in
the host; interrupted changes continue from current geometry. Native uses the
existing finite host clock and current motion membership. Its explicit default
Reduce paints the endpoint; NoPreference permits transitions. Native OS motion
preference discovery is outside the current host contract.

All five sizes, four themes, class/style overrides, unstyled, focus and node_ref
remain available. The themed control centers a 12/15/18/21/25 px drawing in a
24/30/36/42/50 px button. Explicit user style colors override theme colors and
also color the path. Disabled theme colors are typed declarations. Unstyled
retains the path, semantics and explicit styles while omitting theme framing.
The Button, Graphic and physical path retain identity through state/motion
changes. Names, sources, callbacks, references and transition state retire with
the Owner.

`tests/actions.rs` passes 14 tests, including the real path/state and existing
controlled rejection, disabled suppression, source silence, four themes,
unstyled, observers, stable references and lifetime behavior. Shared render
`tests/graphic_transition.rs` covers finite duration/topology and raw/live rollback.
The HTML full-shape oracle includes compatible transitions, invalid duration and
clear/reset. The classic-stream test checks path/config content patch through End.

`D:/BuildCache/sapid/graphic-transition-native.log` records two real raster tests:
intermediate line geometry, interrupted sampling, equal/Keep writes, final frame
and idle clock behavior, live Reduce, engine identity, ancestor clipping and
premultiplied group-opacity pixels, hidden cancellation and disposal. Native
component four-theme pixel checks are in `examples/native/tests/burger.rs`.
`graphic-transition-interruption-prototype.json` records Chromium 148.0.7778.96
actual CSS d interpolation, continuous reversal and immediate mid-flight Reduce.
The complete component browser report remains pending unified Web15 and
`examples/web/burger-acceptance.js`; the prototype is not a substitute for it.
