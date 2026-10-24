# Popover

The shared/Web placement and composition batch and the native Dialog/Popover
top-layer driver are implemented and tested. Drawer sheet, Menu/Tooltip/Listbox
layers and ContextPoint remain separate native capability boundaries.

`popover(key).placement(AnchorPlacement { side, align, gap, viewport_padding,
flip, shift })` uses four physical sides and Start/Center/End alignment. Defaults
are Bottom/Start, gap 6 CSS px, viewport padding 8 CSS px, flip and shift enabled.
Gap and padding each accept finite 0..=4096 values. Flip chooses the opposite
side only when it has less viewport overflow; shift independently clamps both
axes. Oversized panels stay at the viewport's padded starting edge. Directional
menus retain their explicit physical side and gap. The old `PopoverAlign` and
four `LayerPlacement::Anchor*` variants have been removed, with no aliases.

`trigger_content(factory)` and `close_content(factory)` supply visual children
inside the actual Buttons. `label` and `close_label` remain their accessible
names. Shared content validation rejects nested controls, including through
transparent Groups. The slots follow their constructing Owner; they do not
replace activation, focus return or controlled proposal behavior.

One current symbolic anchor is resolved through Memory's live name index and
its ancestry. An absent, unplaced, suspended, hidden, inert or explicitly
display:none anchor cannot open a physical layer. A previously valid anchor
becoming unavailable closes the host layer and queues `DismissReason::AnchorLost`.
Shared reference/direct App validation rejects that event while the anchor is
valid; restoration also discards a queued stale loss. A controlled source may
refuse the close proposal, but the physical layer stays suppressed until the
anchor recovers. A relation may bind late during construction without a premature
close callback. No business source or visited-anchor history is retained.

HTML format v6 introduced all six placement fields independently, including
defaults and in builds without CSS flags. Snapshot adoption rejects older
versions or mismatched fields before changing the root. Classic envelope remains
v2; the current HTML version is 13 after the finite rich-editor protocol. Older Start data fails before a
controller is registered or DOM edits are applied. Stream configuration carries
semantic shapes and provides no pre-WASM interaction driver.

Evidence: render's four anchor tests cover twelve placements, collision flags,
arithmetic rejection, numeric policies, late binding and ancestor/identity
lifetime. Nine Sapid overlay tests cover controlled requests, slots, content
validation and cleanup. HTML's dedicated differential oracle compares every
placement and individual field/reset with full shaping; the existing general
oracle remains in place. The anchor classic-stream producer exercises every
field patch and both version boundaries. Native `blitz/tests/hints.rs` continues
to reject unsupported Tooltip layers before facet registration and retains the successful
prefix/query watermark.

The original `examples/web/popover-placement-acceptance.js` passes all thirteen
groups on Chromium 148 against the fixed root Web21 snapshot:
`D:/BuildCache/sapid/popover-browser-root21.json`. Eight theme/CSR/hydration groups
each check all twelve actual placements, custom gap, real Button slots, input
focus, return focus and AX. Separate groups cover edge collision policies,
controlled anchor loss/recovery, touch/Owner cleanup, atomic adoption rejection
and classic stream fields/version refusal.

Independent Apps in one Document now share a weakly registered layer coordinator.
It retains at most 64 live Apps and only current layer leases. The most recent
modal owns keyboard focus, Escape/Tab routing, background inertness and viewport
scroll locking across App boundaries. Nested Apps inside that modal remain
eligible; retiring their physical parent releases disconnected layer leases.
Closing a lower modal preserves the upper modal's focus, and closing the upper
modal restores the actual previous focus in the surviving modal. Nonmodal
outside dismissal preserves the user's clicked focus target.

The first modal records viewport position and the original overflow/padding
values and priorities. The final modal restores those properties only while
they still contain the coordinator's own writes, preserving intervening author
changes. Native wheel, touch and keyboard scrolling stay inside the current
modal; capture sessions behind a new modal are cancelled. The registry stores
Weak App references and drops all document listeners when its final App retires.

The twelve original `layer-document-acceptance.js` groups pass against fixed
root Web23. They include eight theme/CSR/hydration combinations, reversed App
listener/open order, nested-root retirement, controlled close refusal, AX,
wheel/PageDown/touch, original style/scroll restoration and the 64-App boundary.
The final combined regression also runs the original Overlay, placement, Menu,
Hints, Choice, Select, Command, ContextMenu and Menubar suites: 109 reports,
`errors: []`, at `D:/BuildCache/sapid/layer-final-root23.json`.

Native now uses the genuine `glory-rs/blitz` fork at full commit
`c127fd9699a89ff8ecba0c8d8c9eccb3330627af`: separate viewport layout, ordered
paint/backdrops and reverse hit testing preserve the logical DOM/Stylo parents.
The final current-pin fork and combined native regressions pass in
`D:/BuildCache/sapid/final-engine-forks.log` and `final-native-acceptance.log`.
The earlier native introduction logs below retain their original revision evidence.
Dialog/AlertDialog Manage and nonmodal Dialog Preserve work with Center and all
twelve AnchorPlacement combinations. Modal focus/AX admission includes live
anchor-associated popups, scroll stays contained, controlled dismissal remains
a proposal, and out-of-order close transfers current focus fallback links.
Preserve does not autofocus, trap Tab or force focus return.

Fork-owned default, parallel/writing-mode and actual CPU paint tests pass through
`check-engine-forks.py --test`; the final provenance log is
`D:/BuildCache/sapid/native-top-layer-final-provenance.log`. Adapter tests cover
actual bounds, focus, wheel scrolling, lifetime and AX independently. Seven
Sapid AlertDialog/FloatingDialog/ColorInput consumer tests pass with actual raster,
anchor changes, controlled refusal and trigger return:
`D:/BuildCache/sapid/native-new-layer-consumers-final.log`. Native sheet,
ContextPoint, Menu/Tooltip/Listbox/Group Layer and Layer-local Translation remain
explicit preflight/direct-apply rejections with successful-prefix query watermarks.
