# NavigationProgress

`navigation_progress().on_ready(|handle| { ... })` provides an application-owned
command handle once the bar is mounted. `start()` returns an opaque token and
shows indeterminate progress. `set(&token, percent)` accepts finite values in
0..=100; `indeterminate` removes its numeric value. `finish` stores 100 and
immediately hides the retained bar; `cancel` hides it without a numeric value.

Only one operation is current. Starting again revokes the previous token.
Commands using revoked or completed tokens return `Ok(false)` without a write.
A token from another controller is an error, even if its generation matches.
Generation allocation is checked and never wraps. Owner retirement invalidates
all handles; the handle retains no owning Scope. There is no operation history,
timer, automatic increment, route interceptor or network interceptor.

The bar reuses Progress's real progressbar semantics and finite typed width.
Indeterminate state omits `aria-valuenow`; its retained parent has busy state
and is hidden while idle. The default accessible name is “Navigation progress”.
Name, theme, class, style, unstyled, node_ref and thickness are configurable.
Placement is normal layout; applications can compose it with Affix for viewport
positioning. Cancellation never replaces the bar or unrelated editor nodes.

Application code owns its asynchronous work and cancellation; revoking a token
prevents publication but does not cancel an arbitrary future on its behalf.
No completion delay or automatic animation is promised.

Evidence: `tests/navigation_progress.rs` covers actual semantic transitions,
stale/foreign tokens, invalid percentages, repeated cancellation, retirement
and 512 bounded restart cycles. `examples/native/tests/navigation_progress.rs`
checks actual 40%/75% widths and engine identity through cancel/restart in four
themes. `examples/web/navigation-progress-acceptance.js` covers CSR/hydration,
real geometry, stale publication, busy resets, mobile and retained editors.
The Web9 Chromium 148 run passed all eight four-theme CSR/hydration cases with
no errors (`D:/BuildCache/sapid/navigation-progress-browser.json`). The combined
Rust/native/Web gate is `D:/BuildCache/sapid/gate-snapshot9.log`.
