# Affix

`affix(key)` positions one stable subtree with the
host's typed layout; it does not run a measurement or scroll-following loop.

The default viewport mode uses `Position::Fixed` with a 16px right/bottom offset.
`AffixOffset` chooses exactly one horizontal side (`Left` or `Right`) and one
vertical side (`Top` or `Bottom`). Values are finite pixels within +/-1,000,000.
Offsets and visibility accept `Vary` sources. Applications may derive offsets
from an explicit `ShellLayout` or another host-driven source; Affix does not
install an implicit global breakpoint service. SSR uses those same explicit
initial values, then normal bindings apply the application's measured updates.

`affix_container(key).content(|context| ...)` creates a real relative scroll
container. `.within(context)` uses absolute positioning in that container;
the item scrolls with its content. `.sticky(context)` keeps its normal flow slot
and uses actual CSS sticky constraints within the scroll container; its default
offset is top/left zero. Sticky constrains the item's natural position rather
than relocating it to a viewport corner. Native's style profile rejects Sticky
explicitly until the engine implements it; HTML support does not widen native
support. Contexts must be live construction ancestors in the same runtime.

`.visible(false)` hides the wrapper while retaining its children, Owners, values
and references. It does not create routing, persistence, observers of every DOM
node, or background polling. The component owns `position` and all four inset
declarations; explicit styles can set its other declarations. The container owns
its relative positioning while width, height, overflow and decoration remain
customizable. Unstyled mode retains those structural layout requirements.

Five component tests cover accepted snapshots, visibility/Owner lifecycle,
context validation, structural overrides and finite budgets. Three native tests
pass: four themes preserve right/bottom offsets through actual ancestor scrolling
and viewport resizing, hidden editors retain engine identity, container absolute
positioning uses measured geometry, and Sticky is explicitly rejected. Evidence:
`D:/BuildCache/sapid/affix-native.log`. The shared foundation adds two ancestry,
two style/parser, one HTML positioning and one native percentage/signed/Auto
inset layout test; the complete style suite and HTML field-delta oracle pass.
Chromium 148.0.7778.96 passed all nine reports on root build9: four-theme
CSR/hydration, page scrolling and real sticky constraints, explicit Resize-driven
responsive offsets, rejected invalid sources, retained hidden editors, dirty
identity/value/selection adoption and Owner retirement. Evidence is in
`D:/BuildCache/sapid/affix-browser.json`, `affix-browser.log`, and `affix-*.png`.
The green/dark hydrated screenshot was inspected for readable controls and both
container and viewport positions. Native Sticky remains an explicit limitation.
