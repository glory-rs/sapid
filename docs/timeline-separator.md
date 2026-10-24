# Timeline and Separator

`timeline("release")` renders an actual ordered list. Explicit stable ASCII item
keys identify each list item; `title`, `description`, `timestamp`, `marker` and
`fill` provide separate content slots. Timestamps are ordinary text supplied by
the application. The component does not parse dates or sort the input.

```rust
use sapid::{timeline, timeline_item};
let release = timeline("release")
    .active("build")
    .item(timeline_item("design").title("Design"))
    .item(timeline_item("build").title("Build").timestamp("2026-10-23"))
    .item(timeline_item("ship").title("Ship"));
```

`progress(Lotus<TimelineProgress>)` accepts `Pending`, `Active(key)` or `Complete`.
Items before the active item read “Completed”, the active item reads “Current”,
and following items read “Upcoming”. `state_labels` replaces these three text
alternatives. The current status has theme accent color and stronger weight;
state classes supplement the readable text. There is no invented
interactive role, implicit focus movement or progress callback. Changes patch
only item classes and status text; item, slot and embedded control identities
remain stable. Controlled sources remain read-only. An unknown dynamic active
key returns a typed error, retains the last accepted state and can recover after
a valid update. The effect and references retire with the component Owner.

Namespaces and keys follow the shared 64-byte ASCII component key contract.
Duplicate keys, more than 256 items, invalid heading levels and empty or
over-256-byte status labels fail before creating nodes. The item array is an
explicit finite configuration rather than a virtual or asynchronously loaded
data source. No visited-item history is retained.
Iterator setters consume only the remaining item allowance plus one overflow
sentinel. Repeated item calls retain at most 257 values; extra captures drop
outside graph borrows rather than growing an invalid configuration indefinitely.

`separator()` is a passive accessible boundary. It uses a real separator role,
horizontal or vertical orientation, an optional name, and no tab stop or input
actions. Horizontal lines default to 100% width by 1px; vertical lines default to
1px by 24px. `length` and positive finite `thickness` set the physical dimensions.
`decorative(true)` uses an unnamed generic view with no separator metadata;
combining a decorative line with a name is rejected. An adjustable, focusable
splitter belongs to the later interaction component.

Both components support all four themes, unstyled output and explicit style and
class overrides. `unstyled` removes the component declarations while preserving
intrinsic list/separator semantics; custom declarations still apply. Timeline
items inherit the parent theme and size unless overridden.

Tests in `tests/timeline.rs` (9) and `tests/separator.rs` (4) cover structure,
validation, controlled progress, recovery, identity, silence, teardown, themes,
unstyled output and SSR. Glory render/HTML tests cover finite separator metadata,
placement, noninteractive targets and full-shape patch equivalence. The native
fixture in `examples/native/tests/timeline.rs` checks actual line dimensions and
stable embedded input identity through progress changes across all four themes.
AccessKit represents the passive separator using its `Splitter` role with axis
metadata and no focus, click or adjustment actions.

The browser fixture and acceptance script are `examples/web/src/timeline_form.rs`
and `examples/web/timeline-acceptance.js`. They check CSR/hydration, real `ol/li`
tags, status alternatives, both line axes, AX roles, control value/caret and node
identity, a narrow viewport and unmount cleanup. Browser AX structure does not
establish screen-reader announcement or platform assistive-technology acceptance.

On 2026-10-24, all eight blue/green × light/dark × CSR/hydration browser cases
passed with zero console/page errors. Four hydrated full-page screenshots are
under `D:/BuildCache/sapid/timeline-{blue|green}-{light|dark}-hydrate.png`. Native
line dimensions, identity and teardown passed across all four themes; the
AccessKit separator orientation/no-action regression passed as well.
