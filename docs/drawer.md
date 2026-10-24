# Drawer

`drawer(key)` creates a named modal Dialog at one of the four physical viewport
edges selected by `DrawerSide`. Its trigger exposes expanded/controls and a
Dialog popup relationship. The panel has a stable semantic identity; title,
description, explicit name, close label and content factory provide its content.
Theme, size, class/style, unstyled and panel/trigger references remain available.

`open(source)` is read-only controlled state; `default_open` supplies local state
otherwise. Activation, Close, top-layer Escape and outside pointer requests use
the accepted state. Controlled refusal preserves the open layer and focus
constraints. Program source writes are silent. `on_dismiss` precedes
`on_open_change(false)` and generic observers. Individual Escape/outside policies
can be disabled. The default Unmount policy retires content on close;
KeepMounted preserves content identity and edits while hidden.

The browser owns top-layer presentation, modal background inertness, initial
focus, disabled skipping, Tab/Shift+Tab containment and return focus. Opening
uses an explicit initial-focus descendant or the first enabled control, falling
back to the panel. Closing restores a still-live return target. Owner retirement
removes presentation and host-introduced inert attributes.

Opting into `snap_points` or `snap_config` adds the finite host-owned gesture
protocol described in [Sheet](sheet.md). This is the same Drawer builder, with
1–8 bounded snap fractions, monotonic accepted revisions, four-edge grip capture,
mouse/touch input, keyboard snap alternatives, controlled refusal restoration,
visualViewport offsets/resize/zoom and current-gesture cancellation. Capture
starts only on the grip; content scroll, selection and editing keep their native
behavior, and a content wheel scroll does not resize the Drawer. A live Reduce
preference immediately ends an in-flight settle at its accepted endpoint.

The P0-08 roadmap scope is covered by the ordinary Drawer baseline plus this
opt-in Sheet protocol. This scope does not include a document body scroll lock,
cross-App stacking, native Layer presentation, OS touchscreen or spoken
screen-reader acceptance. Native rejects Layer and sheet-grip declarations in
preflight and direct apply; tests retain the accepted-prefix query watermark.
Those boundaries are not inferred as supported from browser results.

Evidence on unified Web15 / Chromium 148.0.7778.96:

- `examples/web/drawer-acceptance.js` passes eight four-theme CSR/hydration
  scenarios in `D:/BuildCache/sapid/drawer-browser.json`, with no errors: ordinary
  non-snap right-edge geometry, actual Chrome Dialog AX/name/modality,
  focus containment/return, Tab cycling and disabled skipping, Escape/outside,
  stable panel, default Unmount and Owner cleanup.
- `examples/web/sheet-acceptance.js` passes nine scenarios in
  `D:/BuildCache/sapid/sheet-browser.json`, with no errors. All eight themed
  CSR/hydration scenarios exercise actual Drawer instances at all four edges,
  mouse and Chromium touch capture, content wheel isolation, retained editor,
  refusal, visualViewport/resize cancellation and in-flight Reduce. The final
  scenario performs classic-stream End then actual WASM adoption with stable
  identity. Four actual Chrome AX snapshots are in `sheet-open-ax.json`.
- `tests/overlays.rs` supplies six baseline relation/policy/state/lifetime/SSR
  tests; `tests/sheet.rs` adds three snap/revision/validation/lifetime tests.
  Shared render, HTML field oracle, SSR stream and explicit native rejection
  evidence are listed in [Sheet](sheet.md). The complete native/Web gate passes
  in `D:/BuildCache/sapid/gate-snapshot15.log`.
