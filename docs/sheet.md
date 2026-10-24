# Sheet and Drawer gestures

Drawer can opt into a host-owned sheet with finite snap geometry:

```rust
use sapid::{drawer, DrawerSnap, DrawerSnaps, DrawerSide};
let snaps = DrawerSnaps::new([
    DrawerSnap::new(0.25, "Compact preview"),
    DrawerSnap::new(0.5, "Half preview"),
    DrawerSnap::new(1.0, "Full preview"),
]).max_extent(640.0).settle_millis(180).dismissible(true);
let panel = drawer("preview")
    .title("Preview")
    .side(DrawerSide::Bottom)
    .snap_config(snaps)
    .default_snap(1);
```

`snap_points(iter)` is a convenience for static points and the default
640 px/180 ms/dismissible configuration. `snap_config(Lotus<DrawerSnaps>)` permits
validated program changes. `snap_index(Lotus<usize>)` is read-only controlled
state; `default_snap` supplies local state otherwise. `on_snap_change` proposes
an index. Internal acceptance precedes that callback and generic Sheet observers.
The existing open source/callback controls gesture dismissal; `on_dismiss`
receives `DismissReason::Gesture`. A rejected controlled proposal restores the
accepted physical extent after the complete host turn, even with zero mutations.
Program source writes never emit proposal callbacks.

There are 1–8 strictly increasing finite fractions in (0,1], a finite maximum
extent in 128–4096 logical pixels, an in-range current index and a settle period
in 0–500 ms. Each explicitly localized snap label is nonempty and at most 256
UTF-8 bytes. The iterator constructor consumes at most nine values, retaining
the ninth only to reject excess cardinality at build. Reactive vectors validate
cardinality and labels before cloning. Invalid updates retain the accepted
configuration without changing the external source.

Extent is `min(visual viewport extent, max_extent) * fraction`; tiny and zero
viewports stay finite. On release, the nearest configured snap wins (ties choose
the first); a dismissible release below half the first snap requests close.
The host places all four edges within the current visual viewport, including
its offset. Sheet owns fixed positioning and the dimension along its snap axis;
an explicit non-fixed position is rejected. Existing content, theme, class,
style, unstyled, panel/trigger references and KeepMounted policy remain available.
The inner edge reserves at least 24 px for its passive grip, including unstyled.

Every configured snap has a real named toggle button exposing the accepted
pressed state. The ordinary Close action is required; Escape retains Drawer
policy. The visual grip has no fabricated slider/separator role or keyboard
focus. Pointer capture starts only there, with intrinsic touch-action:none;
content scrolling, selection and editing keep native behavior. Dragging the
content itself is intentionally outside this gesture contract.

Raw listeners only retain current pointer coordinates. The RAF read phase reads
initial physical extent and visualViewport before private geometry writes. One
current gesture and one pending restoration are retained; no per-frame Cage,
application task or source history is created. Current program configuration,
resize/visualViewport changes, inherited inert, hidden/suspended state, Window
blur, pointer cancellation and Owner retirement revoke an old gesture. A
captured revision cannot act on a later accepted configuration.

Revisions are validated in shared render/UI, not merely component state. Older
events are silently discarded; future events are errors. Equal revision requires
equal snap configuration and edge. Actual accepted configuration/index changes
allocate a new revision; equal writes do not. A live sheet declaration cannot be
removed and reintroduced with an old counter; dispose its node to retire it.
Settling uses finite host transitions and follows live prefers-reduced-motion.
Native Layer and sheet-grip descriptors retain an explicit Unsupported boundary.
No native gesture/OS touch or assistive-technology announcement is claimed.

Rust evidence includes three Sapid tests, two shared render tests, the HTML
field/full-shape oracle, native early/direct rejection with successful-prefix
query watermark, and a classic-stream config/revision patch followed by End.
The complete render/UI/HTML/SSR full-feature suite and strict Clippy pass in
`D:/BuildCache/sapid/sheet-protocol.log` and `sheet-protocol-clippy.log`.
Component/native evidence is `sheet-rust.log` and `sheet-native.log`; production
WASM strict Clippy is `sheet-web-clippy.log`. Browser acceptance passes nine actual scenarios against unified Web14 on Chromium
148.0.7778.96, with no errors: `D:/BuildCache/sapid/sheet-browser.json`. Each of the
eight four-theme CSR/hydration scenarios checks all four physical edges, captured
mouse drags and real Chromium touch input, controlled snap/close refusal, editor
identity, content wheel isolation, program revision cancellation, viewport resize
and visualViewport zoom, Window blur/inert cancellation, live reduced motion and
real intermediate settling geometry. The ninth executes classic stream End then
real WASM adoption with stable identity and finite physical extent.

`sheet-open-ax.json` separately records four real Chrome AX snapshots: the Dialog
name and Full snap button pressed state. Four `sheet-{hue}-{scheme}-open.png`
screenshots show the opened panel; green/dark was visually inspected. The eight
main scenarios also check cleanup and mobile overflow. Browser protocol/touch
input injection is not an actual OS touchscreen or screen-reader client claim.
