# Resizable / Splitter

`splitter(key)` owns two stable panes and one focusable separator. A horizontal
layout uses a vertical separator line; a vertical layout uses a horizontal line.
The line carries finite `aria-valuemin`, `aria-valuemax`, `aria-valuenow`, and
`aria-controls`. Plain `Separator` remains passive.

```rust
use glory::prelude::*;
use sapid::splitter;

let percentage = scope.cage(50.0);
splitter("editor")
    .value(percentage)
    .min(10.0)
    .max(90.0)
    .on_value_change(move |next| percentage.revise(|value| *value = next))
    .first(|| glory::ui::input().name("Document"))
    .second(|| glory::ui::text().content("Preview"));
```

Without `.value`, `.default_value(50.0)` supplies local state. Controlled sources
are read-only: change callbacks propose a percentage and the application chooses
whether to write it back. External changes are silent. `.on_value_commit` runs
after a keyboard adjustment or completed pointer drag. Typed callbacks precede
generic Range/Drag observers. Same-tick local key events accumulate; rejected
external source values leave callbacks using the accepted configuration.

Percentages must stay within `0..=100`, with finite `min < max` and an inclusive
current value. `.step` is positive and at most 100; `.thickness` is positive and
at most 128 pixels. Defaults are min10, max90, step1 and thickness6. Bounds are
reactive. Invalid configurations return errors before creating nodes; invalid
later updates preserve accepted descriptors. This component has two panes;
nested splitters compose additional regions.

The arrow keys perpendicular to the separator adjust by one step; Home/End select
the configured boundaries. Modifier and composition keys retain their host
meaning. Disabled handles use programmatic focus mode, leave the Tab sequence,
and reject physical focus or range actions. At an allowed 0/100 endpoint the
collapsed pane is hidden while its Owner and children remain mounted. Focus in a
newly hidden pane moves to an enabled handle. `.first_ref`, `.second_ref`, and
`.handle_ref` expose stable live nodes and clear on retirement.

The inner layout reserves the handle's actual thickness, then divides the
remaining extent between panes. Outer padding and borders do not enter that
measurement. Panes have zero minimum flex extent and retain their child nodes,
editor values and engine identities while resizing. Application CSS can still
change a child's overflow policy; the pane wrapper clips overflowing content.

Pointer capture belongs to the original handle even when release occurs over a
sibling. One live geometry request serves a drag; moves waiting for geometry
coalesce to the latest absolute point. Cancellation and Owner retirement remove
the request and its captures immediately; late results cannot publish. Each drag
has a non-wrapping ticket, with no visited-pointer or historical geometry cache.
HTML's typed splitter contract includes `touch-action:none`. Native Finger
pointer protocol simulation exercises the real engine input path; it does not
establish physical-device or screen-reader acceptance.

Validation currently includes six Sapid component tests; two native component
tests covering four themes, both axes, measured layout conservation, controlled
requests, disabled behavior and editor identity; three shared render tests; two
shared native tests covering queued keyboard, mouse/Finger capture/cancel and
AccessKit numeric actions. The HTML full-shape versus field-delta oracle passes
with splitter range set/reset metadata. `examples/web/splitter-acceptance.js`
passed all ten checks on root build8 with Chromium 148.0.7778.96: four-theme
CSR/hydration, dirty editor identity/value/selection takeover, Owner retirement,
and actual CDP touch Start/Move/End/Cancel. Touch capture preserved page scroll
and the editor selection; cancellation produced no commit. Reports and screenshots
are in `D:/BuildCache/sapid/splitter-browser.json`, `splitter-browser.log`, and
`splitter-*.png`; native evidence is in `splitter-native.log`. The green/dark
hydrated screenshot was inspected for readable content and handle placement.
