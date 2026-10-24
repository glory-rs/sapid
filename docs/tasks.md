# Plan, Task and Queue

Enable `chat`. Use `sapid::plan(namespace).data(source)`, `queue(namespace)` or
`task(namespace, record)` for explicit application work. Qualify `sapid::plan`
when also importing Glory's semantic planning utility. These builders do not
create a scheduler, execute tasks, retry jobs or publish a result.

`PlanData` has a nonblank title, public description and at most 128 `TaskRecord`s.
Keys are distinct ASCII identifiers of at most 64 bytes; titles/action labels are
256 bytes, descriptions 8192 bytes, and the complete model at most 128 KiB. Each
record has Pending, Running, Complete, Failed, Skipped or Cancelled status, an
optional integer percentage from 0 to 100, and an optional named application
action. All text is literal. The complete model is validated before publishing
or cloning it to child bindings. Invalid current updates preserve the complete
accepted display and return a typed error.

Plan and Queue use real ordered lists with full-key row identity. Queue order is
the supplied application order. A Task's live key cannot change; remount it to
replace its identity. Each finite row retains only its current accepted model
until the list retires its Owner. Removal cannot evaluate an absent record or
retain visited tasks. Use the windowed Terminal/TestResults widgets for large
indexed output; these finite plan models deliberately admit at most 128 tasks.

Plan progress is **processed work**, not successful work: Complete, Failed,
Skipped and Cancelled count as terminal records. The exact status remains visible
on every task. `No tasks supplied` has no fabricated progress bar. Per-task
progress is an explicit caller percentage and disappears when it is unknown.

An optional action renders a real button. It is disabled without an installed
`on_action` callback, when its action is disabled, or when the builder's `enabled`
source is false. The callback receives only `TaskRequest { task_key, action_key }`.
The application decides whether to accept and update its controlled model.
Refusal leaves status, progress and action unchanged. Widget callbacks cannot
grant authorization or perform a rollback implicitly. Callback captures and all
row/progress bindings retire with their Owner.

Common theme, size, unstyled, name, class, style, node references and generic
observers remain available. Plan/Queue namespaces are distinct semantic ASCII
identifiers of at most 40 bytes; standalone Task namespaces admit 64 bytes. The
same task keys in independent namespaces do not collide.

`tests/chat_tasks.rs` exercises finite admission, exact processed progress,
proposal refusal, explicit enablement, keyed reordering/removal, invalid-current
rollback and callback capture disposal. `examples/web/tasks.html` supplies CSR
and four-theme SSR fixtures. Its acceptance suite and the native suite check
physical controls, progress, focus, updates, pixels, empty states and cleanup.
On 2026-10-25 seven component tests and the real four-theme native test passed:
`D:/BuildCache/sapid/chat-tasks-output-final-tests.log`,
`chat-tasks-output-native-tests.log` and `chat-tasks-output-native-strict.log`.
Native acceptance checks actual geometry, hit testing, queried focus, pixels,
controlled callback acceptance/refusal, stable engine identity and retirement.

The Web42 artifact passed ten actual reports with zero errors in Chromium
148.0.7778.96: `D:/BuildCache/sapid/tasks-browser-web42-stable-key.json`. Four-theme
CSR/adoption, real ranges, keyboard proposals, source reordering, accepted/refused
updates, physical disabled controls, empty states, focused adoption, mobile and
cleanup are covered. The original frozen script incorrectly located the row by
its old title after renaming it; `output-tasks-browser-web42.log` retains that
failure. The corrected suite resolves the full semantic ID before the update,
so it checks the same physical row across title changes and reordering. The
artifact itself was unchanged. OS input and assistive-client acceptance remain
separate.
