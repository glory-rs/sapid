# Suggestion

`suggestion(key).label(...).value(...).source(...).on_choose(...)` is an actual
Button. Source is explicit application metadata, displayed in the action's name;
it is not a citation URL or permission to retrieve anything. `SuggestionChoice`
contains the original key, label, value and optional source. Choosing publishes
that bounded payload only. It sends no prompt, selects no model, starts no task
and executes no authorization/transport policy automatically.

`suggestion_group(key).suggestions(...)` composes a real action Toolbar with one
roving tab entry, Arrow/Home/End navigation, disabled skipping, optional looping,
explicit orientation/direction and ordinary Enter/Space activation. It reuses the
live Collection/RovingFocus registry and keeps physical button roles. An item's
callback runs before the group callback. Group enabling reaches actual button
properties; a standalone suggestion retains its independent state and callback.

There are at most 128 actions; iterator intake stops after 129 entries. Stable
keys are unique and at most 64 bytes, labels at most 256 bytes, optional sources
at most 128 bytes, each value at most 8,192 bytes, and all declaration text at most
128 KiB. Values are explicit application payloads and are not interpreted as
HTML or a command language. Labels/source names reject blank text and NUL/CR/LF.
No past choices or external source records are retained. Owner cleanup clears
callbacks, current mounted registrations and focus metadata.

`D:/BuildCache/sapid/chat-actions-tests.log` includes three Suggestion tests:
exact payload/source and item/group order without task creation, one roving entry
with disabled skipping and actual disabled guards, plus million-item intake,
duplicate keys and payload rejection before rendering. The four-theme native
consumer in `chat-actions-native-tests.log` verifies real geometry, focus movement
between actual Buttons, disabled skipping, chosen payload/source, group disabling
and raster output. It uses the native semantic event driver; OS keyboard and
screen-reader sessions are separate acceptance. Immutable FORMAT13 Web43 passes
the combined twelve-case `model-suggestions-acceptance.js` without console/page
errors (`model-suggestions-browser-web43-accepted.json`). All four themes pass
CSR/adoption, exact action payload/source, real Arrow/Home/End/Enter navigation,
disabled skipping, one tab entry, actual group disabling, independent standalone
Button, mobile/AX and Owner retirement/remount. Tests retain asynchronous host
frame boundaries; they do not infer focus from a queued intent.
