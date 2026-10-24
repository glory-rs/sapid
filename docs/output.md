# Terminal and TestResults

Enable the `chat` feature. `terminal(namespace, count, key_at, locate, read)` displays
read-only application output; `test_results(namespace, counts, key_at, locate, read)`
displays explicit application test facts. Neither builder launches a command or a
test runner. The namespace is a distinct, nonempty semantic identifier, at most 40
bytes, for each output window.

The source stays in the application. `key_at` and `locate` must be indexed lookups;
`read` supplies only the requested record, for example through `Cage::with` on one
indexed item. A billion logical records need not be allocated. The component reads
only the current window and the list's one retained focused row. Defaults are 64
mounted rows, 256 retained height measurements and two overscan rows, with explicit
bounded overrides. No visited-record cache, source scan or complete log clone is
used. Current keys are located again before reading an updated row, so a prepend
cannot temporarily read the previous position under the wrong key.

`LogRecord` has a numeric key, stdout/stderr, a textual level and `LogPayload`.
`Text` admits at most 8192 UTF-8 bytes per current record. CRLF/CR become LF, tabs
and line breaks remain, and other controls including ANSI escape are visibly
escaped. Markup is literal semantic text and URLs do not become automatic links.
`Redacted` contains no original payload and shows `[redacted]`; callers redact
secrets before passing them to the widget. This is an output viewer, not an ANSI
terminal emulator.

`TestCounts` explicitly supplies passed, failed, skipped and running totals. Their
checked sum is the indexed record count, at most one billion. The component does
not scan the source to derive or independently verify these facts. Each `TestCase`
has an application status, a nonblank 256-byte name, optional exact elapsed
milliseconds (at most seven days), and the same bounded output payload. Details
use a real Collapsible with keyboard and expanded metadata; closed details are
unmounted. Empty counts show `No tests supplied`. There are no Run/Retry actions.

`revision` notifies a structural source change. Reads may also subscribe directly
to the indexed reactive fields. A record must return the mounted key; invalid
current text, keys, names or times report a typed error and retain the previous
complete accepted row. The application updates count, key lookup and revision
atomically when inserting or removing records.

Both windows use MessageScroller's measured window and physical scroll protocol.
First pauses following and seeks to zero; Newest explicitly resumes following;
Pause preserves the reading position. `on_ready` exposes the same operations via
`OutputHandle`. Its weak Owner-bound controller retains no application source or
read callback, and all methods fail after disposal. Actual scroll movement pauses
or resumes following according to proximity to the tail. Height updates use the
existing visible-key reading anchor.

The fixture is `examples/web/output.html`, with CSR and four-theme SSR adoption.
`tests/chat_output.rs` covers control projection/redaction, overflow, bounded
billion-record reads, current source updates, malformed-record rollback, prepend
ordering, independent namespaces, actual test disclosures and Owner cleanup.
`examples/native/tests/chat_output.rs` and `examples/web/output-acceptance.js`
contain real-host acceptance checks. On 2026-10-25 the eight component tests and
the four-theme native test passed, including real geometry/hit testing, focus,
raster pixels, billion-record windows and Owner cleanup. Logs are
`D:/BuildCache/sapid/chat-tasks-output-final-tests.log`,
`chat-tasks-output-native-tests.log` and `chat-tasks-output-native-strict.log`.

The immutable Web42 original suite passed ten reports with zero errors in
Chromium 148.0.7778.96: `D:/BuildCache/sapid/output-browser-web42.json`. This covers
four themes in CSR and adoption, keyboard controls, actual test disclosures,
stable/prepended keys, source updates, bounded billion-record work, focused SSR
adoption, mobile layout and retirement. Earlier Web41 failure/diagnostics remain
in `output-browser-web41.log` and `output-browser-web41-storage-bound.log`.
They exposed a real count/key subscription ordering fault, now covered by the
fixture clearing regression. The earlier storage assertion also confused sparse
index nodes with retained measurements; Web42 separately checks at most 64
measurements and the sparse index's bounded node count. A separate mobile probe
confirms convergence to the real tail, rather than treating an intermediate
measurement frame as a completed scroll. No OS-device or screen-reader acceptance
is inferred.
