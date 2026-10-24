# Read-only progress

`progress(value)`, `ring_progress(value)` and `semi_circle_progress(value)` share
one controlled, read-only model. A value is a `Lotus<f64>` in the inclusive range
configured by `range(min,max)` (default 0..=100). Bounds must be finite and strictly
increasing. The minimum takes zero visual progress and the maximum takes full
progress. Even a finite range spanning -f64::MAX..=f64::MAX computes its fraction
without overflowing intermediate arithmetic. Values are rejected rather than
silently clamped.

`name(text)` provides the localized accessible name (default "Progress").
`value_text(text)` optionally supplies a localized value alternative. Names are
nonempty and at most 256 UTF-8 bytes; value text is nonempty and at most 1024.
`indeterminate(Lotus<bool>)` omits the numeric current value without inventing
zero. Its static partial shape communicates unknown progress without promising
animation. The numeric source remains finite and within range while unknown.
These widgets do not register range, pointer, keyboard or click handlers and do
not modify sources or make application callbacks. Invalid external updates
preserve the last accepted progress metadata and shape; recovery keeps identity.

The root has a real progressbar role, finite minimum/maximum and optional current
value and text. It is passive and has no explicit focus. Shared placement rules
reject interactive descendants, including moves/resumes through boxless Groups
and predeclared Plans. The independent Splitter range protocol supplies actual
interactive range actions; progress does not advertise those actions.

Linear progress uses an actual bounded fill width. Circular variants use the
shared Graphic protocol with at most six finite path commands per arc, real
cubic curves, rounded stroke caps and decorative graphics beneath the named
progress root. The semicircle reserves half a stroke below the diameter so its
round endpoints remain visible. Native rings require Glory's opt-in paint.
There is no resource transport, raw SVG string or font glyph dependency.

`size(px)` sets a circular diameter or explicit line width (16..=512).
`thickness(px)` is 1..=128 and at most half a circular diameter. `width(Length)`
and explicit Style may override the box. All four themes, custom classes/styles
and node references are supported. Unstyled retains essential geometry and
progress content while removing the themed track/frame decoration.

Evidence: five Sapid tests cover exact range fractions, all three presentations,
four themes, endpoints, bounded geometry, invalid initial values, controlled
rejection/recovery, identity and unstyled overrides. Three render tests cover
the finite numeric/text contract, prepared rollback and passive placement rules.
HTML's differential field oracle covers all progress fields and indeterminate
resets. A native AccessKit test verifies real read-only progress role, numeric
value/text and indeterminate omission without SetValue/Increment/Decrement/Click
or Focus actions. The shared host fixture is `examples/web/progress_form.rs`;
native acceptance checks geometry, rendered arc/track pixels and changing values
across four themes. The SSR stream producer uses the explicit environment
variable `GLORY_PROGRESS_STREAM_FIXTURE`. Chromium 148.0.7778.96 passed all eight
CSR/hydration runs and one classic-stream run. The actual result is
`D:/BuildCache/sapid/progress-browser.json`; hydration screenshots are stored
beside it. Native pixel/layout evidence is recorded in
`D:/BuildCache/sapid/progress-native.log`, and the complete shared protocol suite
passed in `D:/BuildCache/sapid/progress-protocol-regression.log`. These checks do
not establish screen-reader announcements.
