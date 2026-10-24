# Portable shadow, wrapping and text overflow

Glory's finite style schema adds three independent optional declarations.
These declarations entered HTML/Record v5 and Blitz v6; the later grid/ratio
schema advanced profiles to HTML/Record v6 and Blitz v7; image object-fit now
advances the current profiles to HTML/Record v7 and Blitz v8. These versions
identify typed acceptance and do not promise identical pixels between engines.
Default component dependency graphs acquire no CSS parser or platform library.

`Style::box_shadow(BoxShadow::Outset(Shadow { x, y, blur, spread, color }))`
declares one outset shadow with explicit RGBA color and CSS-pixel geometry.
Offsets and spread may be signed and must be finite. Blur must be finite and
nonnegative. `box_shadow: None` leaves a stylesheet declaration in charge;
`Some(BoxShadow::None)` explicitly removes it. Inset, multiple layers and
currentColor are intentionally absent from this finite schema.

The optional Sheet parser accepts `none`, or two through four px lengths followed
by an explicit supported color: `box-shadow: 0 2px 6px -1px #0003`. Omitted blur
and spread become zero. Unitless zero is accepted. Percent lengths, inset,
comma-separated layers, omitted/unsupported colors and extra tokens fail closed.
Every original declaration is checked against the target profile even when a
later declaration overrides it. Property, ShadowNone, ShadowOutset and Px support
can each be rejected independently.

Card and Paper expose mount-time `shadow(ShadowSize::None/Small/Medium/Large)`.
The three finite tokens choose offset/blur/spread and theme-aware alpha. Custom
typed style overrides retain precedence, including reactive overrides. Default
surfaces add no shadow; unstyled removes component-owned shadow declarations.

`Style::flex_wrap(FlexWrap::NoWrap/Wrap)` declares the wrapping behavior of a flex
container. Absent leaves the stylesheet in charge; explicit NoWrap resets a
stylesheet's wrap value. It is independent of display, flex direction, shrink,
gap and dimensions. Optional CSS accepts only `nowrap` and `wrap`; wrap-reverse
is rejected. Both values and the property have independent support rejection.

HTML serializes each declaration without shorthands that reset unrelated fields.
The full-shape differential oracle covers shadow set/reset and both wrap values.
Shared Memory/adapter preflight retains its normal validation, rollback and
successful-prefix behavior.

Tests in Glory's style/html `shadow.rs` and `flex_wrap.rs` cover finite values,
numeric rejection, explicit resets, support masks, CSS parsing and serialization.
Blitz `tests/shadow.rs` records real native CPU pixels for a colored offset,
clearing the shadow, blur/spread and stable engine identity. Its wrapping test
uses real layout to verify a second fixed-width child moves to a new row and
returns to the first row after NoWrap, retaining both child identities.

`Style::text_overflow(TextOverflow::Clip/Ellipsis)` is independent of width,
whitespace and overflow clipping. Absent leaves author CSS in charge; explicit
Clip resets a stylesheet's ellipsis value. HTML/Record v5 accept both values;
the optional CSS parser accepts only `clip` and `ellipsis`. Property and both
values have independent support rejection tests and HTML differential-oracle
coverage. The pinned native engine has no text-overflow rendering path, so Blitz
v6 rejects the entire property, including explicit Clip, before model/queue/host
changes. No native ellipsis or selection behavior is fabricated.

`Display::InlineFlex` shares the existing Display property and has an independent
support value. The finite CSS parser accepts `inline-flex`; HTML serializes it
literally. The pinned native engine performs actual shrink-to-content inline
flex layout. Its geometry regression checks a 40px child fits a 40px inline flex
container within a 240px block, expands to 240px after a Flex patch, and shrinks
again without replacing the semantic node. HTML's patch oracle covers this
display value. Pill uses it by default; unrelated Flex layouts keep their
existing behavior.
