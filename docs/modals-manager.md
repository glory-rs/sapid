# ModalsManager

`modals_manager(key).on_ready(|handle| { ... })` owns a local FIFO queue. Its
default capacity is 16, configurable to 1..=32. `modal_request(title, description)`
opens a Dialog; `confirmation_request` opens an AlertDialog with a safe Cancel
initial focus. Both can supply a content factory and an `on_result` callback.
Only the first request mounts content or opens a Layer. Queued requests retain
bounded request configuration, never an editor tree or an owning Scope.

`open` returns an opaque manager-specific token. `update` changes the accepted
title and description in place, preserving the content Owner, editor and node
identities. `close` can remove either the current or a queued request.
`close_all` first removes the old queue, then invokes its callbacks; a callback
may enqueue a new request without that request being accidentally removed.
Old tokens return false, foreign tokens are errors and numeric identities never
wrap. Titles are limited to 1024 UTF-8 bytes, descriptions to 16384 and action
labels to 256; empty or noncanonical CR/NUL text is rejected before admission.

Results distinguish Closed, Confirmed, Cancelled and Dismissed(reason). A result
is delivered at most once, after removal, without queue or Cage borrows. It runs
untracked under the manager's Owner, including when `close` is called directly
from application code. Retiring that Owner drops pending captures without
calling application code and invalidates retained handles/tokens. No completed
request history or global singleton is retained.

The manager uses the existing Dialog/AlertDialog controlled `open` and host
Layer implementation. Triggerless dialogs require an independent title/name;
anchored Popovers cannot omit their trigger. Layer focus, Escape, modal input
exclusion and final focus restoration remain host responsibilities. There is
one permanently closed host boundary so a host rejecting Layer fails at manager
mount, before it can admit an imperative request. The current native adapter
supports this manager's centered Dialog and AlertDialog Layers, including safe
Cancel focus and FIFO final return to the original application opener. Other
native Layer facets keep their own explicit support limits; support for this
queue does not imply Menu, Tooltip, sheet or ContextPoint support. Cross-App
modal stacks remain outside this API.

Evidence: `tests/modals.rs` covers FIFO construction, stable metadata updates,
safe decisions, queued removal, capacity, invalid/foreign tokens, callback
reentry and construction context, Owner cleanup, triggerless title validation
and bounded repeated disposal. `examples/native/tests/modals.rs` now checks
actual native FIFO Dialog→AlertDialog in four themes: real bounds, top-layer
hit testing and raster pixels, one current content factory, metadata updates
preserving the input engine identity and actual selection, safe Cancel focus,
at-most-once outcomes, final original-opener focus and retired-handle/capture
cleanup. Native semantic activation uses the App route, not an OS pointer or
screen-reader client. The browser fixture
`examples/web/modals-acceptance.js` passed all eight four-theme CSR/hydration
cases on Chromium 148.0.7778.96 with no errors. It checks real focus, FIFO final
restoration, queued confirmations/cancellation, retained editors and mobile
layout against the corrected Web11 host (`modals-browser-web11.json`). The
shared host carries a closing Layer's restoration target into a replacement
Layer, so the end of a confirmation queue restores its original application
trigger. The initial failing Web10 run is not completion evidence.

The former native rejection assertion became stale after native top layers were
implemented. Its original final-gate failure remains in
`D:/BuildCache/sapid/final-check-native-web.log`. The replacement strong test
also exposed a real native FIFO focus-handoff defect; its pre-fix failure is
preserved in `modals-native-current-tests.log`. The adapter now inherits only
the closing managed top Layer's live local fallback when disposal has removed
the focused control. It retains exact implicit NodeId lifetimes, rejects
retired/foreign, hidden/inert, disabled and suspended targets, and keeps no
closed-layer history. Three focused adapter regressions additionally cover
current nested managed scopes, Preserve exclusion and explicit return targets.
Post-fix focused evidence under `D:/BuildCache/sapid`:
`modals-native-handoff-tests.log` (3), `modals-native-current-tests-fixed.log`
(1 actual four-theme test), `modals-native-handoff-clippy.log` and
`modals-native-consumer-clippy.log` (strict). These focused reports supplement,
rather than replace, the following complete native gate.
