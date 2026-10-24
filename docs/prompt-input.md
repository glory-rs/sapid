# PromptInput

`prompt_input(key)` composes one real Form, a labelled multiline Textarea,
submit/cancel Buttons, optional current FileInput and application-supplied text
suggestions. Enable the `chat` feature. It installs no model service, transport,
clipboard reader or upload policy.

The default `PromptSubmitMode::ButtonOnly` preserves native multiline Enter.
`EnterToSend` opts into the shared keyboard command; Shift+Enter remains a line
break and composing Enter is ignored. `on_send(PromptSubmission)` runs in an
Owner task after the complete host tick and accepted bindings. It can issue a
real query barrier, including `FileController::read_in`. It receives accepted
canonical text and current document-owned file metadata/tokens. Sending never
clears text, files or busy state. Cancel calls only `on_cancel`; the application
owns cancellation, authorization and transport. Suggestions propose text through
the same controlled update path and never submit it.

Use `.value(source).on_value_change(...)` for controlled editing. A refusal keeps
the accepted source; callbacks cannot make the editor authoritative. `.busy`,
`.readonly` and `.enabled` gate submission and editing. The queued send rechecks
current state and live active/enabled/inert ancestry before publication; Owner
retirement cancels the task. Repeated requests while one send is queued coalesce.
Hidden or disabled keep-mounted ancestors cannot publish a late send.

Text is limited to 65,536 canonical UTF-8 bytes, including NUL replacement
expansion. At most sixteen suggestions are accepted, each at most 8,192 bytes
and 32,768 bytes in total; iterator intake stops at the first over-budget item.
Labels are nonblank and at most 128 bytes. The successful control name is at
most 256 bytes. Invalid external source values are checked by borrow before any
clone. Empty text with no current files emits `PromptRejection::Empty`.

`.attachments(FileConfig)` opts into the actual finite file protocol. Selection
metadata contains opaque live document tokens, never filesystem paths. The
application chooses whether and how much to read, within the host read budget.
Native supports the ordinary multiline form and explicitly rejects the optional
file configuration; it does not emulate a native file picker.

Current evidence: `D:/BuildCache/sapid/chat-prompt-scroller-tests.log` passes six
Prompt tests and six Scroller tests. Prompt coverage includes all edits in one
drained tick, controlled refusal, composition/busy/readonly checks, bounded
million-item intake, canonical byte accounting, suggestion/cancel policy,
post-flush query barriers, retired tasks and late hidden/inert/disabled ancestry.
`chat-prompt-native-tests.log` passes a four-theme actual native multiline layout,
accepted text echo, actual ASCII selection/focus retention, queued send,
composition refusal and raster test, plus explicit optional-file rejection.
Native event injection verifies the adapter protocol, not an OS IME session.
Immutable FORMAT13 Web41 passes all fourteen browser cases without console/page
errors (`prompt-browser-web41-final.json`, Chromium 148). This includes all four
themes in CSR/adoption, actual three-line editor geometry, intrinsic multiline
AX, exact FormData, sending without caret/focus/text replacement, controlled
refusal, ordinary/Shift Enter, browser IME, busy/readonly/cancel policy, suggestion
proposals, actual file chooser + four-byte Blob read from `on_send`, dirty
adoption and Owner cleanup. The script hash is
`b155324ca94d61d76fec1c022edda95123f890c324c95f97cb840e2b4a5f1b93`.
The first two test-only failures remain recorded: the initial script assumed a
`rows` attribute despite Textarea's existing CSS-height contract, and used
`Send` instead of the declared default `Send message` label. The corrected test
checks real three-line content height; the frozen source/artifact are unchanged.
