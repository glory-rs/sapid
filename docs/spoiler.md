# Measured Spoiler

`spoiler(key).content(text).max_height(120.0)` clips plain selectable text and
provides a real named disclosure button outside the clipped viewport. It accepts
reactive text up to 64KiB and positive finite mount-time height. Text remains in
one original paragraph/text node during expansion; it is not rewritten into a
truncated business value. Interactive descendants are excluded by the plain-text
API, so hidden clipped controls cannot remain in the Tab sequence.

`expanded` is controlled; `default_expanded` supplies local state only without a
source. `on_expanded_change` proposes changes before generic activation observers.
External source changes remain silent. `labels(more,less)` requires nonempty
single-line names. Common theme/size/class/style/unstyled/name/enabled/focus/ref
methods apply to the root. `viewport_ref` and `trigger_ref` identify stable parts.
Unstyled preserves structural clipping and explicit overrides.

A boxed inner view observes real content height. Overflow determines whether the
action shell is visible; content changes, font/layout changes and width changes
use the same Resize protocol. Before the first measurement SSR keeps an action
for nonempty text; it cannot predict pixels. If content shrinks while the action
is focused, the component requests focus on the stable programmatic text box
before hiding its action. There are no timers, polling loops or layout reads in
reactive patches. The only retained content is current text and its accepted
binding value, never a history of text or list rows.

Invalid source updates preserve the last accepted paragraph and can recover.
Owner cleanup retires handlers, bindings and references; late events cannot
resurrect them. `tests/spoiler.rs` verifies state, callback order, measurements,
controlled rejection, bounds/invalid input, text recovery, focus intent, SSR
escaping and retirement. `spoiler-acceptance.js` and the native fixture verify
actual geometry, element/text identity, keyboard/pointer, focus after shrink,
four themes and teardown. Actual passed host runs are recorded separately in the
D-drive reports, not inferred from test-file existence.

On 2026-10-24 the focused Chromium 148.0.7778.96 run passed all eight four-theme
CSR/hydration cases with errors=[] (`D:/BuildCache/sapid/spoiler-browser.json`).
The native four-theme geometry/identity test passed in `spoiler-native.log`.
