# Layout boxes

`box_()`, `flex()`, `stack()`, `group()`, `center()`, `container()` and `space()`
compose semantic View boxes. `sapid::Group` is a flex row with wrapping and a
12px gap; `glory::ui::group()` remains boxless. No layout helper introduces an
implicit accessible role or keyboard behavior. A supplied name follows Glory's
named View group rule.

All builders accept `theme`, `size`, `unstyled`, reactive `class`/`style`, `name`,
`node_ref`, `fill`, `width`, `height` and `padding`. Flex/Stack/Group additionally
accept finite `gap`, `align`, `justify`, `wrap(FlexWrap::{NoWrap,Wrap})`; Flex
accepts `direction`. The gap and direction are mount configuration; reactive
Style can update any supplied declaration without replacing children. Unstyled
preserves structural layout while removing Container's cosmetic padding.

Center aligns children on both axes. Container uses a real centered inner box,
100% available width, a default 960px cap and 16px horizontal padding.
`max_width(Limit)` changes the cap; `fluid()` explicitly removes it. The cap
shrinks with the parent's available width. Space defaults to zero by zero and
accepts explicit px/percent dimensions. Percent height needs a definite containing
height according to the selected engine. Breakpoint providers and responsive
spacing token selection remain part of the P0 theme infrastructure.

Invalid mount dimensions/gaps/caps are rejected before creating nodes. Explicit
Style overrides replace only their supplied fields; clearing an override restores
the configured layout. No source list, window data or visited history is retained.

`tests/layout.rs` covers boxed Group, defaults, independent style merging,
reactive identity/Owner cleanup, unstyled, invalid configuration and SSR escaping.
`examples/web/src/layout_form.rs` is shared by CSR/SSR/native verification;
`layout-acceptance.js` verifies real center/max-width/wrap/gaps/resize/identity,
four themes and teardown. Actual run results belong in the D-drive acceptance
reports. `examples/native/tests/layout.rs` checks real engine geometry in all four
themes. A present test file is not a claim that the host run has passed.
