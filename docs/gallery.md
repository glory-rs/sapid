# Gallery and explicit capability ledger

The primary documentation website is now the [Glory + Sapid showcase](showcase-site.md),
with a Rust-built home page, component navigation, search, examples and source.
This page describes the separate static capability audit gallery and its preserved fixtures.

`examples/web/gallery.html` renders all 149 canonical work items, with search,
phase/completion filters, four palettes, CSR/adoption selection and one real
mounted example. Rows without an explicitly declared fixture show no preview.
The ledger's model/SSR/Web/native/AccessKit states describe their finite
documented scope; canonical completion alone never establishes host support.

Edit `docs/capability-matrix.json` to change a task's explicit state, limitation
or preview page. `python scripts/capability-gallery.py` checks exact canonical ID
coverage, valid references and declared preview files, then generates the page
and `docs/widget-capabilities.md`. Run its `--check` form after inventory or
ledger updates. An absent fixture is a supported documentation state, rather
than an invented live implementation.

Only the current preview and latest requested URL are retained. A new selection
waits for the current WASM mount to finish before navigating, avoiding aborted
compiler requests. A selection without a preview immediately hides the old
frame while preserving its geometry until that mount completes. Loading frames
are inert and busy. One animation-frame readiness watcher has a 30-second
deadline and is cancelled on page retirement.

`D:/BuildCache/sapid/gallery-browser-next.json` records twelve original browser
cases passing with no console/page errors. That isolated static Gallery uses
fixed FORMAT12 Web38 bindings/SSR for its real examples; the three Gallery
HTML/JS/CSS files include the final navigation fix. All four palettes in CSR and
adoption operate a real NativeSelect by keyboard, test exact 149-row coverage,
filtering, explicit native rejection, empty-preview behavior and a 390px mobile
layout. Desktop/mobile captures were inspected at `gallery-next-desktop.png`
and `gallery-next-mobile.png`. Earlier Web37/Web38 failures remain recorded;
the next complete immutable snapshot repeats the same twelve assertions.
