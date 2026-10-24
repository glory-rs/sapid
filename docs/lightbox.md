# Lightbox

`lightbox("photos").images(images)` displays one current semantic Image in the
existing managed Dialog Layer. The closed gallery creates no Image or resource
consumer. Opening uses the Layer's modal focus management, Escape/outside
dismissal and focus return. Closing retires image content; removing the Owner
also retires the dialog, controls and callbacks.

`lightbox_image(key, source, alternative).caption(text)` creates a record.
Snapshots admit at most 128 distinct 1–64 byte ASCII keys, source strings within
the existing Image 4096-byte limit, nonblank alternatives up to 1024 bytes and
captions up to 2048 bytes. The combined metadata budget is 256 KiB. Sources are
canonical single-line text. All records and the current selection are checked
before cloning or publishing a replacement; invalid updates retain the accepted
model. This metadata budget does not measure downloaded/decoded image bytes.
Host resource configuration still controls origins, transports, decoded sizes,
cancellation and caches. No ambient native HTTP/filesystem transport is added.

`LightboxView { current, zoom }` controls a complete view proposal. `None` chooses
the first current image, including truthful empty state for an empty snapshot.
An explicit key must belong to the supplied snapshot. Applications changing
both should publish them in one batch. `LightboxZoom` is Fit/Two/Three/Four;
the viewport actually scrolls the larger image at 200–400%. Previous/Next and
Zoom in/out/Fit are real named buttons with physical disabled boundaries. The
focusable Image navigation region supports Previous/Next/First/Last commands.
Exact alternatives, captions, current count and zoom remain textual. Failed
decoding keeps the image alternative and displays a failure message.

`view` and `open` are controlled sources; `default_view`/`default_open` supply
uncontrolled initial values. `on_view_change` and `on_open_change` propose
changes and permit refusal. Source changes never emit proposals. Repeated
navigation reuses the one current Image identity, and all source/status events
retain the shared Image revision/lifetime checks. No preload or visited-image
registry is installed. An application can theme the builder or use `unstyled`;
viewport sizing, scrolling and focus behavior remain structural.

Evidence is recorded under `D:/BuildCache/sapid`. `lightbox-component-tests.log`
contains six tests for modal lifetime, keyboard commands, 400%/Fit geometry,
controlled refusal, atomic admission, empty snapshots, silent sources, bounded
metadata and repeated-navigation cleanup. `lightbox-strict.log` and
`lightbox-web-strict.log` check production/components and WASM. Actual Native
and browser acceptance use `examples/native/tests/lightboxes.rs` and the
original 12-case `examples/web/lightbox-acceptance.js`. Immutable FORMAT12 Web39
passes all **12 original reports, errors `[]`**, in Chromium148.0.7778.96:
`lightbox-browser-web39.json` / `plots-lightbox-browser-web39.log`. The suite
covers four-theme CSR/adoption, real decoded and failed images, scrollable zoom,
root keyboard commands, actual focus return, controlled refusal, silent sources,
Owner retirement, focused SSR trigger adoption and mobile geometry. The mobile
screenshot was visually inspected. `plots-lightbox-native-final-tests.log`
passes the real four-theme WindowApp case: injected resource ingestion changes
actual pixels, next/zoom retain the engine image identity, bounds really grow,
failure text appears and closing retires pending consumers. Native focused
strict checks pass in `plots-lightbox-native-strict.log`. Source and hashes are
under `D:/BuildCache/sapid/web39-source`. Native fixtures inject a test provider;
this is not a
production transport or screen-reader acceptance claim.
