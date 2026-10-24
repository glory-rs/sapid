# Marquee and RollingNumber

Both components use the finite shared `TranslationMotion` presentation. They
have no application frame task and do not modify focus, selection, value
sources or scroll position. One static formatted/plain text supplies the
accessible content; every moving visual copy is inert and absent from both
Chrome AX and the native AccessKit tree. They do not create a live region.

```rust
marquee()
    .text(status)
    .period_millis(8000u32)
    .paused(paused)
    .animated(running);

rolling_number()
    .value(amount)
    .locale(NumberLocale::DeDe)
    .format(NumberFormat::Currency(Currency::Eur))
    .duration_millis(500u16)
    .paused(paused);
```

`Marquee` accepts plain text of at most 4096 UTF-8 bytes. It maintains two
visual copies only while the first copy exceeds the current viewport width.
A short value stays static and hides the duplicate. An inner zero-padding
viewport owns clipping, while root styles retain borders, padding, colors and
size overrides. Left and Right are physical directions. Gap is finite
0..=4096px, and the measured cycle must be at most 1,000,000px.

One Owner task coalesces Resize and `.measurement_revision` refreshes. Each
current batch requests exactly two Bounds, for the viewport and first copy;
source replacement, stop and Owner retirement revoke older replies/tasks. It
does not scan a business collection or retain old values. Initial SSR and
client shapes have an unmeasured static first copy. Motion begins after
adoption and measurement; measureless hosts keep that static presentation.

Marquee periods are 16..=60,000ms. `.paused(true)` retains the current phase;
`.animated(false)` cancels it, returns to the static first copy and hides the
duplicate. Starting again begins a new loop. Replacing text first removes old
measured motion and measures the accepted new value. Resize can change whether
motion is needed. Explicit refresh handles application-owned position/font
changes that do not produce a Resize observation.

`RollingNumber` reuses NumberFormatter's exact finite formatting implementation:
EnUs/DeDe/ZhCn, Decimal/Percent and USD/EUR/CNY, grouping and fraction budgets
0 <= min <= max <= 18. Nonfinite values, percent overflow, more than 64 formatted
characters or more than 24 numeric digits reject before rendering. Reactive
rejection preserves the previous accepted visual and accessible value while
leaving the external source unchanged.

Each current formatted position has a short keyed Owner. Numeric positions have
ten fixed rows (0 through 9), a monospace `1ch` cell and 24px line geometry;
punctuation/sign/currency positions are static text. Surviving positions retain
their reel identity. Removing a position or switching its digit/literal kind
retires its previous Owner. New digits first appear at their accepted value;
later targets interpolate from current ink. Duration accepts 0..=2000ms;
`.animated(false)` uses immediate target placement. The static formatted
alternative changes directly to the accepted value and never rolls or repeats.

Both components support all four themes, size, class/style overrides, node
references, generic semantic observers and `.unstyled()`. Unstyled retains the
finite clipping/reel geometry and accessible text. Numeric reel geometry uses
its own monospace cells; root styling controls surrounding typography and ink.

Browser Reduce changes immediately settle To reels to their target and Loop
tracks to their resting start. Native defaults to Reduce and accepts an
explicit Config/renderer preference; OS preference synchronization is not
inferred. Native uses its existing document clock, and `advance_time` updates
real paint and visual revision without Cage writes. Bounds include finite
current translations on the node and its semantic ancestors, with scroll
compensated only once. Arbitrary author CSS transform geometry is outside this
typed promise.

Evidence: six component tests plus eleven unchanged NumberFormatter regressions
(`D:/BuildCache/sapid/text-motion-rust.log`); two native tests, including actual
four-theme dimensions, raster intermediate frames, clip, stable engine identity,
default Reduce, unstyled, resize and real editor focus
(`D:/BuildCache/sapid/text-motion-native.log`). The shared facet additionally has
three phase tests, two render tests, two HTML full-shape oracle tests, synchronous
and stream SSR tests, four real native translation tests and one native AX test.

Chromium 148.0.7778.96 passed all nine same-source web26 component scenarios:
four-theme CSR/adoption, bidirectional movement, pause/stop, actual intermediate
reels, keyed sign/length changes, live Reduce, one accessible text per component,
short/long text, viewport resize, mobile overflow, retained editor value/focus/
UTF-16 selection and dirty pre-adoption input. Raw report:
`D:/BuildCache/sapid/text-motion-browser-web26.json`. The shared facet's separate
nine browser scenarios also passed, including Rust classic stream animation
after End and cleanup of the ready root:
`D:/BuildCache/sapid/translation-browser-web26.json`. Representative images
`text-motion-green-dark-hydrate.png` and `translation-green-dark-hydrate.png`
were visually inspected. Browser AX and headless native focus/raster evidence
do not establish screen-reader speech or OS motion preference integration.
