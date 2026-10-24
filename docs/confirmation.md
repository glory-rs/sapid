# Confirmation and Checkpoint

The optional `chat` feature exposes `confirmation(key)` and `checkpoint(key)`.
They render finite application-owned data with actual Group, heading, text and
Button semantics. They do not execute commands, grant permissions, manage a
server session, save a snapshot or perform a rollback.

```rust
use sapid::chat::*;

let state = scope.cage(ConfirmationData::default());
confirmation("publish-draft")
    .data(state)
    .on_decision(move |request| {
        // The application checks the key, displayed revision and its own policy.
        // Updating state is an explicit application decision.
        if request.decision == ConfirmationDecision::Reject {
            state.revise(|value| value.status = ConfirmationStatus::Rejected);
        }
    })
```

`ConfirmationData` contains title, description, literal details, application
revision and Pending/Approved/Rejected/Cancelled status. An enabled Pending
confirmation with an installed callback can propose Approve or Reject through
`ConfirmationRequest { key, revision, decision }`. The callback does not change
the displayed state. If the application ignores/refuses the request, Pending
remains displayed and the component does not claim success. Program data changes
are silent. Terminal states disable both real buttons while preserving identity.

`CheckpointData` has the same text/revision fields with Available/Current/
Restoring/Restored/Unavailable status. Only Available with an installed callback
can emit `CheckpointRequest { key, revision }`. The application may then publish
Restoring and later a real outcome. The component never invents completion or
pretends that restoring metadata reverses an external operation. A refused
restore leaves Available intact.

Keys are fixed, validated finite semantic identifiers up to 64 bytes. Revision is
an application-supplied snapshot identifier, not an authorization token or a
widget-generated monotonic counter. The receiving application decides whether a
request is still current and permitted. Current data is rechecked when the live
button callback runs. Disabled controls, inactive Owners and absent callbacks
cannot generate a request; retirement releases callback captures.

Both builders support theme, size, class, explicit style, unstyled, name,
enabled, focus, node_ref and ordinary event observers. `ConfirmationLabels` and
`CheckpointLabels` provide finite caller-supplied status/action strings. Status
is ordinary visible text, not an implicit live announcement or modal dialog.
Buttons remain actual native controls; the wrapper adds no fake interactive role.
Long literal text and action labels stay within bounded, scrollable boxes.

Each atomic data value validates before cloning/publishing to child bindings:
nonempty title at most 256 UTF-8 bytes, description 8192, details 24576, and an
independent aggregate 32768-byte budget. NUL/CR are rejected; title and labels
also reject control characters. Six labels per component each admit at most 256
bytes. Invalid reactive data preserves the last complete accepted view and
reports a typed error. No historical request list or business-source scan is kept.

Web SSR and CSR have the same deterministic shape. Hydration preserves the
existing root and controls. Native uses the ordinary supported text/button/layout
path; there are no browser-only hidden fields or native fallback widgets.
Unstyled removes theme fill while retaining column/action-wrap geometry.

Evidence currently includes seven component/model tests in
`D:/BuildCache/sapid/confirmation-tests.log`, native real engine boxes/hit tests,
focus queries and raster pixels across four themes in
`confirmation-native-tests.log`, and focused component/Web strict Clippy.
The unchanged original nine-case browser fixture passes on immutable Web43,
Chromium 148: `confirmation-browser-web43.json`; combined run log
`confirmation-rte-all-actual-web43.log`. Eight four-theme CSR/hydrate cases and
one status/native-keyboard case cover source refusal/silence, explicit
application-only status and restoration, key/revision proposals, disabled and
absent callbacks, real Chrome AX, literal text, stable roots, mobile geometry,
unstyled and Owner teardown. The green-dark mobile screenshot was inspected at
`D:/BuildCache/sapid/confirmation-green-dark-hydrate.png`.

Native semantic activation uses the shared App event route; this is not an OS
pointer/AT-client claim. Native AccessKit projection remains model-only for this
component; the real Chrome AX assertion is a separate Web axis.
