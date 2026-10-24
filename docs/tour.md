# Tour

Create an Owner-scoped `tour_context(scope, key)`, wrap current targets with
`tour_target(&context, key).fill(widget)`, and declare `tour(&context, key)` with
`tour_step(target_key, title, description)`. Targets are real programmatically
focusable View boxes. A step uses the target's stable semantic name as the
anchor and focus-return relation of a real nonmodal Dialog. Previous, Next,
Finish and Cancel are real Buttons; Escape belongs to the shared Layer host.

There are at most 64 live target registrations and 64 declared steps. Keys are
unique bounded ASCII semantic tokens; title/description each have at most 1024
canonical bytes. Iterators consume at most 65 entries, including a rejected
overflow entry; invalid declarations are rejected before cloning their content
or creating the panel. Nested targets reserve their current slot before child
construction. Registry cleanup drops entries outside its mutable borrow.
No unmounted business rows are queried or reconstructed.

`current` and `open` are controlled inputs; `on_current_change` and
`on_open_change` propose changes, including refused navigation and dismissal.
Defaults apply only without a controlled source. `on_complete` reports a Finish
request; `on_cancel` reports an explicit Gesture or host dismissal request.
These request callbacks do not imply that a controlled source accepted closing.
The Owner-bound `TourHandle` provides `next`, `previous`, `go_to`, `cancel` and
`refresh`. Retired handles fail and out-of-range steps are typed errors.

Construction starts with a hidden panel on both SSR and the adopting host. A
ready host task reveals only the current mounted target with the shared
`ScrollIntoView` action, then requests its actual Bounds. Positive live geometry
opens the anchored Layer; invisible, inert, suspended or retired targets hide
it. `on_target_lost` fires once per loss episode without rewriting controlled
open/current inputs. Re-registering that current target recovers the same panel;
`refresh` permits explicit application geometry changes to be rechecked.
Target Resize interests also refresh that current observation.

Each guide retains one current host task and at most one Bounds request. New
steps, target changes and Owner retirement cancel old work; ticket, index,
target identity and current liveness all validate before accepting a reply.
The panel and its content nodes remain stable across steps. Physical scrolling
does not modify focus/caret/preedit; focus entry and return use host Layer rules.
Native uses the same finite anchored Dialog and genuine engine top layer.

Eight component tests pass in `tour-tests.log`, including controlled refusal,
same-tick commands, exact finite declarations, million-item iterator admission,
measureless SSR shape and registry retirement. Two native tests pass in
`tour-native-tests.log`, including four themes, actual viewport reveal/anchor
collision, panel/editor identity, real focus return, target loss/recovery,
100-refresh one-query budget and pending Owner cancellation. Component/native
strict Clippy and Web WASM strict checks pass. These logs are under
`D:/BuildCache/sapid`. The original twelve-group `tour-acceptance.js` passes
against fixed HTML9 Web29, Chromium148, in `tour-browser-web29.json` with
`errors: []`. It covers eight theme/CSR/hydration combinations, actual viewport
scroll, role/name/description and focus, controlled step/close refusal, target
retirement/recovery, previous/Escape and Owner cancellation during queued work.
Popover's thirteen existing groups also pass the same artifact, including old
HTML8 rejection before mutation (`popover-browser-web29.json`). Browser protocol
checks do not establish OS IME or screen-reader acceptance.
