# Message, Bubble and Marker

Enable `chat` to use application-supplied conversation content. `message(key,
MessageData)` displays bounded author, body, timestamp, finite status, grouping,
alignment and an optional quotation. Time text and localized status text come
from the application. No clock, transport, model service, retry or announcement
is inferred. Explicit `.actions(...)` contains ordinary semantic controls.

Keys are stable ASCII semantic identifiers of at most 64 bytes. `message_id`
derives their collision-free target namespace. Bodies are limited to 65536
UTF-8 bytes, quotations to 4096, author/source/status labels to 128, timestamps
to 64. Validation borrows the complete candidate before accepting its clone;
invalid reactive data retains the previous complete display, reports a typed
error and can recover. Display normalization leaves the application Cage alone.

`bubble` uses real paragraph text with preserved whitespace and horizontal
overflow for long unbroken content. Its `.unstyled()` retains layout and text
semantics while removing cosmetics. Grouping hides only the stable header/footer
shells. `MessageQuote` uses Blockquote; an optional target uses a real Anchor.
`marker(target).label(...)` is a native fragment link. `.on_navigate(...)` is an
explicit application override, following the existing Link protocol.

Evidence under `D:/BuildCache/sapid`: `chat-message-tests.log` records five
passing tests for budgets, all-or-nothing rejection/recovery, stable nodes,
escaping, actual quote/link semantics, grouping and caller-owned actions in
four themes. `chat-message-strict.log` covers strict Clippy; `chat-web-check.log`
covers the WASM caller. Immutable FORMAT12 Web38 passes all twelve original
browser cases with no page or console errors (`messages-browser-web38.json`),
including four themes in CSR and adoption, literal script-looking text, stable
body updates, grouping, caller action and native fragment navigation.
`chat-message-native-tests.log` passes the real native engine test in all four
themes: nonempty message/link boxes, accepted body updates retaining their NodeId
and caller-owned action state. This is layout/protocol evidence rather than OS
screen-reader announcement acceptance.
