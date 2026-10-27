# Files: picker and drop implementation notes

The optional `files` feature provides `file_button()`, `file_input()`,
`dropzone()` and `file_attachment(info, controller)`. The finite protocol, real Web
chooser/drop/read driver, four themes, unstyled appearance and default-native
rejection have focused tests. The explicit native `blitz-files` service now also
supports FileInput/FileButton, reading and private previews; the follow-up below
keeps actual OS dialog and drag/drop acceptance separate. Immutable Web36 passed nine actual widget groups and
one classic-stream/adoption group in Chromium 148.0.7778.96, with no page errors.

The application receives opaque `FileToken` references and display metadata, never
an ambient path or a browser `File` object. Tokens carry an independently allocated
document identity, the complete generational NodeId, configuration revision and a
checked document-local serial. Sharing a RuntimeId cannot make two renderer consumers
share file read rights. A source revision change, explicit clear/removal or Owner
retirement revokes read/preview acceptance. The platform operations that already
started keep their admission permits until their Promises actually settle.

Finite domains are independent:

| Domain | Maximum |
| --- | --- |
| Current selected files per picker | 32 |
| MIME/extension filters | 32, 2048 total UTF-8 bytes |
| Filename | 1024 canonical UTF-8 bytes |
| MIME metadata | 127 ASCII bytes |
| Selection metadata | 32768 total bytes |
| File and selection byte domain | 1 GiB, configurable lower limits |
| One requested file chunk | 65536 bytes |
| Concurrent irreversible reads | 4 per document |
| Preview header read | 65536 bytes, same four admission slots |
| Preview encoded bytes | 8 MiB, configurable lower |
| Preview image area | 16 Mi pixels, configurable lower |
| Preview image axis | 16384, configurable lower |

`FileInfo.name` is display text. No API interprets it as a path. Filtering uses
canonical MIME or extensions; the MIME value is declared metadata, not a content
security check. Selection count, every original metadata string, safe integral JS
size, checked total bytes and checked metadata totals must pass before registration.
Raw FileList length is checked before cloning at most 32 handles. Consumed and
rejected picker FileLists are then cleared from the hidden input.

Preview admission is deliberately narrower than a full image decoder. It checks
static 8-bit non-interlaced PNG signatures/IHDR and CRC, scans finite pre-IDAT chunks,
and rejects APNG. It checks 8-bit grayscale/RGB JPEG frame/scan headers with bounded
segment traversal. Unknown formats, excessive metadata/header work and unsupported
modes retain metadata-only display. PNG structure/APNG ordering follow the
[W3C PNG specification](https://www.w3.org/TR/png-3/); JPEG frame and scan fields follow
[ITU-T T.81](https://www.w3.org/Graphics/JPEG/itu-t81.pdf). These are finite admission
checks, not an assertion that the entire image is well-formed. Existing Image
load/error handling remains necessary. Encoded, header, pixel and per-axis limits
are not a total browser decoder/RSS promise.

A document creates at most one blob URL per current token after successful header
admission. Clear, replacement and retirement revoke the URL. Revocation cannot undo
a browser decode or already-displayed frame. Preview URLs do not enter SSR data or
a visited resource history.

The physical picker design uses a true Button Main and hidden Picker input as
siblings under a display-contents Control root, with the text Content below Main.
`Shape.control = Some(Main)` keeps actual focus on the Button. The finite
`Shape.root_part` contract must be applied consistently to construction, plans,
serialization, classic streaming and adoption; an absent control/part remains an
error. FileInput composes a file list, independently named remove buttons and explicit
clear. Dropzone supplies actual file drop behavior and a keyboard picker path.
Uploads belong to the application.

Current evidence:

- `D:/BuildCache/sapid/file-model-tests.log`: five pure identity/filter/metadata/range
  tests passed, including same-runtime document distinction and counter exhaustion.
- `file-field-guard-tests.log`: three raw/live guard tests passed: touched-field-only
  writes, configuration pointer preservation, revision rejection, Keep/default reset,
  rollback/unwind, successful prefix and forbidden interactive placement.
- `file-preview-model-tests.log`: eight pure model/header groups passed at the first
  checkpoint. `file-jpeg-scan-baseline.log` preserves the initial failing strict
  JPEG first-scan regression; `file-jpeg-scan-tests.log` proves its correction:
  admission now requires a valid first SOS and rejects a second SOF before it.
- `file-base-check.log`, `file-base-web-check.log`, `file-base-native-check.log` and
  `file-base-clippy.log`: initial base adapters compiled with explicit unsupported
  File mapping. The next physical-shape checkpoint is proven by
  `file-html-tests.log` (24 groups, including the full all-field patch/default/plan
  oracle), `file-part-html-clippy.log`, `file-part-web-check.log` and
  `file-part-native-check.log`. Native still explicitly rejects File controls;
  Web/SSR now retain the Control root and both real sibling controls.
- `file-picker-raf-prototype.json`: Chromium 148 actually opened the native file chooser
  from an RAF host-action callback while transient user activation remained true,
  selected a named UTF-8 text file and read its exact bytes. This isolated prototype
  is not counted as completed FileInput or Dropzone acceptance.


```rust
use glory::render::{FileConfig, FilePreviewLimits};
use sapid::file_input;

file_input()
    .label("Choose documents")
    .configuration(FileConfig {
        multiple: true,
        max_files: 3,
        preview: Some(FilePreviewLimits::default()),
        ..Default::default()
    })
    .previews(true)
    .on_files(|accepted| { /* application-owned upload queue */ });
```

`files_ref` and `status_ref` are output mirrors. Writing them cannot manufacture
platform file references or replace the private accepted list. `controller_ref`
provides `FileController::{clear, remove, read, preview}`; read/preview take callbacks
with explicit errors. The `_in` variants assign query callbacks to a requesting
Scope, so removing an attachment cancels its own callback independently of the
picker. Programmatic clear/removal and accepted configuration changes are silent;
`on_files` reports successful host selections, while `on_result` also reports
cancellation or rejection. Internal state is accepted before generic observers run.
Cancellation/rejection keeps a previously accepted selection, matching file choosers.

FileInput exposes `description` and per-token `attachment` factories. Each accepted
current token gets one attachment Owner. Removing a sibling keeps the other actual
Owners. Dropzone enables real trusted drop handling and retains an intrinsic button
for keyboard choice. The default attachment shows name/MIME/size, an independently
named remove button, and optional explicit preview. PNG/JPEG previews use a real Image
with an alternative, bounded header admission and ordinary Image load/error behavior;
other formats remain metadata-only. Networking/upload cancellation belongs to the
application; these widgets revoke local selection/read acceptance only.

One actual browser Document shares a chooser admission slot, four irreversible read
slots, one pending raw-drop handle snapshot, 128 selected references, 128 KiB of
selected metadata and 1 GiB of logical selected bytes across independent consumers.
Read admission failure has no waiting-job queue. Cancelled acceptance does not refund
a Promise that has already started. A new App in the same Document cannot bypass
those physical permits. Current selections/preview URLs retire separately from
unfinished physical operations. A current chooser can remain pending until the
platform reports change/cancel; there is no claim that Owner disposal cancels the OS
dialog.

Hydration first validates the whole physical subtree without changing a dirty
FileList. It stores at most 128 candidate numeric identities. Only after complete
validation does it consume bounded raw handles, validate metadata and mint current
consumer/revision tokens. `Adopted` is first-only for a fresh control lifetime;
replayed or later adoption cannot resurrect cleared rights. SSR serializes no File
objects, file paths, selected tokens or blob URLs. The classic stream carries the
same finite Control/Main/Content/Picker shape and config patch; WASM starts a fresh
revision and document identity on adoption. Native File controls explicitly reject
before allocating their engine identity, and the accepted operation prefix remains
queryable.

Additional focused evidence:

- `file-component-six-final.log`: six component groups passed, including four themes,
  unstyled, typed-before-generic, output-only mirrors, stable attachment Owners,
  silent revision reset, cancel/rejection preservation, disabled behavior and slots.
- `file-adoption-tests.log`: three shared live-state groups passed, including
  rollback/equal-write rights, document/Owner/inert/disabled bounds and first-only
  adoption after clear.
- `file-web-model-tests.log`: nine Web helper groups passed, including four fixed
  read slots, accepted watermarks and terminal replies without early permit refund.
- `file-native-tests-final.log`: explicit native descriptor rejection before engine
  allocation with accepted-prefix bounds reply passed.
- `file-stream-tests-rebuilt.log`: the real Rust stream producer config patch,
  sibling part range and current End rekey passed.
- `file-protocol-final-clippy.log`: all-target shared render/UI/HTML/SSR strict Clippy
  passed. The current widget fixture strict log is recorded separately.


Final widget-host evidence:

- `files-browser-web36.json`, `files-stream-browser-web36.json` and
  `files-actual-web36.log`: nine widget groups plus one classic group passed.
  All four themes ran CSR and hydration with actual disk-backed OS chooser handles,
  keyboard Enter, canonical names/MIME, exact original Blob bytes, actual PNG decode,
  one current blob URL, real revocation, filtering/count/byte failures, disabled/AX,
  unstyled, retained attachment Owners, dirty neighboring editor selection and
  390px viewport checks. The final group used a trusted CDP operating-system file
  drop and rejected malformed PNG before blob URL creation. Four held platform
  Promises remained charged across clear and a new App; a fifth read never began,
  live revoked requests completed with errors, and real completion restored capacity.
  An actual intercepted chooser remained charged across Owner replacement; its late
  selection never entered the new Owner. Dirty pre-WASM FileList adoption preserved
  the actual button/editor nodes and could read the exact original bytes.
- `files-stream-browser-web35.json` also passed the original shared range/patch/End
  handoff and real chooser after adoption before the widget fixture repair.
- `files-browser-web35.json`, `files-actual-web35-original.log`, `files35-dom.log`
  preserve the first widget acceptance failure: three replacing Stack.fill calls
  removed the controls from the test page. It was corrected to one fill of three
  boxless Groups, without reducing assertions. `file-fixture-shape-pass.log` adds
  the complete SSR four-picker/controls regression. A separate runner-environment
  failure is retained as `files-browser-web35-runner-failure.json`.
- `file-focused-final.log`: seven render integration groups, three UI groups,
  all 26 HTML unit/oracle groups and the SSR File shape group passed together.
- `file-fixture-final-clippy.log`, `file-disabled-clippy.log` and
  `file-component-clippy-final.log` record strict fixture/component checks.
- Real selected-file screenshots were visually checked:
  `files-selected-green-dark.png`, `files-selected-green-dark-mobile.png`.

The cancellation branch is covered by trusted-host component/protocol tests and the
real browser cancel listener. Automated chooser interception can generate an
untrusted empty-selection change rather than an OS cancel; that synthetic event is
ignored. No manual OS-dialog cancellation or native OS picker acceptance is claimed.
Native File controls intentionally remain typed Unsupported; no ambient filesystem
fallback is installed. Upload transport and its cancellation remain application-owned.


## 2026-10-27: explicit native file service consumers

The separate `examples/native` workspace adds opt-in `files`, forwarding only to
`glory/blitz-files`. Sapid's widget/default dependency graph gains no native
transport. Applications install a `glory::blitz::files::Files` service explicitly;
normal widgets retain their existing controller and Owner lifecycle.

`examples/native/tests/file_driver.rs` passes two tests across blue/green and
light/dark. Actual FileInput/FileAttachment/FileButton consumers cover selected
CJK/emoji display names, exact chunk reading, explicit Preview activation, native
PNG header admission and decoding, CPU frames, Remove/clear, retired reads, and
preserving the previous selection on cancellation/Busy/Unavailable. The test uses
an injected picker/reader and the real engine/resource provider; it does not open
an OS chooser. Strict Clippy and the default-unavailable CopyButton regression
pass separately. Logs are `/Volumes/Data/Caches/glory/native-files-sapid-tests.log`,
`native-files-sapid-clippy.log`, and `native-files-default-sapid-tests.log`.

Glory's shared UI now revalidates a successful read/preview reply if an earlier
event in the same drained tick revoked the selected token. Native private preview
addresses are revoked together with the engine reuse-cache entry, so a later
image cannot bypass the provider by reusing cached pixels. Already delivered
pixels are not secure-erased. The genuine engine pin and all native acceptance
lock entries are synchronized; details and budgets are in
[`../glory/docs/native-files.md`](../../glory/docs/native-files.md).

Dropzone still requires native drag/drop and is explicitly rejected by that host.
Android/iOS pickers, OS chooser selection/cancellation, sandbox security-scoped
bookmarks and real assistive-technology input remain open. Existing browser and
classic-stream records above retain their original scope.
