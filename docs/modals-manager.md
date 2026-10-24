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
one permanently closed host boundary so a native host rejecting Layer fails at
manager mount, before it can admit an imperative request. Native top layers
remain explicitly unsupported. Cross-App modal stacks remain outside this API.

Evidence: `tests/modals.rs` covers FIFO construction, stable metadata updates,
safe decisions, queued removal, capacity, invalid/foreign tokens, callback
reentry and construction context, Owner cleanup, triggerless title validation
and bounded repeated disposal. `examples/native/tests/modals.rs` checks early
Layer refusal and a usable application afterwards. The browser fixture
`examples/web/modals-acceptance.js` passed all eight four-theme CSR/hydration
cases on Chromium 148.0.7778.96 with no errors. It checks real focus, FIFO final
restoration, queued confirmations/cancellation, retained editors and mobile
layout against the corrected Web11 host (`modals-browser-web11.json`). The
shared host carries a closing Layer's restoration target into a replacement
Layer, so the end of a confirmation queue restores its original application
trigger. The initial failing Web10 run is not completion evidence.
