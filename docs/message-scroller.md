# MessageScroller and Chat

Enable `chat`. `message_scroller(count, key_at, locate, render)` uses Glory's
measured windowed list, rather than mounting all messages. The application owns
its indexed source. `key_at` reads just one index; `locate` must be an indexed
key lookup. Supply `.revision(source_revision)` for nonreactive source edits.
The row renderer receives a stable semantic key and reactive current index.

The default viewport is 480px, estimated row height 96px, live-row limit 64,
overscan 2 and retained measurement budget 256. Explicit limits permit at most
256 ordinary mounted rows plus the existing focused-row pin, 1024 retained
measurements and overscan 8. Oversized viewports fail through the shared list's
typed row-budget rejection before reading their full range. Source message
data and previous visits are never retained by this wrapper.

`on_ready` exposes an Owner-bound `MessageScrollerHandle`. `status_source()`
provides a read-only reactive `{ following, unread }` value. Real physical scroll
or list-key navigation away from the tail pauses following; programmatic scroll
acknowledgements, prepend anchoring and measurements do not. While following,
accepted total/viewport changes seek the new tail. `follow_tail()` explicitly
returns there and clears unread; `pause()` preserves the current reading point.
The shared list's position acknowledgement prevents a later follow command from
being lost when its requested maximum has not changed.
Late program acknowledgements are distinguished from a user movement coalesced
with the same acknowledgement by comparing their actual coordinates. An
application issuing its own physical `ScrollTo` pauses following explicitly.
`.name(source)` supplies the actual list's bounded accessible name, defaulting
to “Messages”, independently of the surrounding conversation group.

The application calls `handle.appended(count)` only after accepting newly
appended messages. Prepending history, editing a body or rejecting a controlled
proposal must not notify appends. This explicit metadata avoids scanning a
source to guess which changes represent new messages. Multiple notifications
in one tick accumulate; overflow is a typed atomic rejection. Prepending and
changing measured content use the shared list's current key/within-row anchor.
Retired handles reject commands and reads; they retain weak construction access,
small current numeric state and no live message source.

`.name(...)` names the actual list viewport (default `Messages`).
`handle.seek(logical)` accepts finite nonnegative logical offsets up to the
shared list's `MAX_EXTENT`, clamps to the current maximum and explicitly pauses
following without changing unread. It is a program command, not a user scroll.
`follow_tail` resumes and clears unread. Invalid seeks preserve all current state.
`chat-prompt-scroller-tests.log` includes six Scroller tests with first-window
seek, clamp/refusal, tail restoration and a real Chat + Textarea composition.

`chat(scroller).title(...).prompt(...)` supplies a named conversation group and
an optional application-owned prompt region. It composes the actual windowed
scroller. It creates no transport, model service, delivery state, automatic live
announcements or command execution.

Current focused evidence: `D:/BuildCache/sapid/chat-components-tests.log`
contains nine passing component tests, including a 100-million-item indexed
source with fewer than forty initial key reads, one bounded live window,
prepend position preservation, explicit unread accumulation, overflow refusal,
Owner retirement and four-theme Chat fixture construction. Shared
`list-user-moved-tests.log` distinguishes physical user movement from seeks and
measurement commits. `chat-scroller-strict.log` and `chat-web-check.log` cover
the exported library and caller. `list-user-moved-ack-tests.log` passes all 35
shared list component regressions, including late program acknowledgements and
coalesced actual user movement. `chat-native-ack-tests.log` passes the native
message four-theme test and the real window test: constrained viewport, physical
scroll metrics, explicit pause/read position, unread and return to the visible
latest item, bounded live nodes and actual raster output.

Immutable FORMAT12 Web40 passes all twelve original browser assertions with no
console/page errors (`message-scroller-browser-web40.json`). It covers four
palettes in CSR/adoption, one million messages, bounded source reads/live rows,
real wheel departure, unread, prepend, measured height changes preserving the
viewport-top intersecting key/within-row reading anchor, explicit return and
later appends. Web39 failure reports and diagnostic coordinates remain retained:
the initial test chose a successor row rather than the documented top reading
anchor, and continued testing then exposed the real late-ack classification
fault fixed and regressed in the shared list metadata.
