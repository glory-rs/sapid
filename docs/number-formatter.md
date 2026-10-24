# NumberFormatter

`number_formatter().value(source)` displays a read-only `Lotus<f64>` value.
The source remains numeric and is never modified by formatting. The component
uses one ordinary semantic Text node, with no editable control, live region,
network request, clock or implicit browser internationalization API.

Locales are explicit finite `NumberLocale::{EnUs,DeDe,ZhCn}` values. The default
is EnUs; no OS/browser/user locale is inferred. This is a small documented
formatting set rather than a complete CLDR implementation. en-US and zh-CN use
`,` groups and `.` decimals; de-DE uses `.` groups and `,` decimals.

`NumberFormat::Decimal` defaults to 0–3 fractional digits. `Percent` multiplies
by 100 and defaults to 0 fractional digits. `Currency(Currency::Usd/Eur/Cny)`
defaults to 2 fractional digits. Currency symbols are `$` for USD in en-US,
`US$` elsewhere; `€` for EUR; `¥` for CNY in zh-CN and `CN¥` elsewhere. de-DE
places the symbol after the number with a nonbreaking space. Other locales put
the symbol first. de-DE also separates a percent sign with a nonbreaking space.
Negative signs precede the entire formatted amount; negative zero is retained.

`min_fraction_digits` and `max_fraction_digits` require
`0 <= min <= max <= 18`. Setting only a minimum raises the default maximum as
needed. Setting only a maximum below the format's default minimum is rejected;
set both to opt into fewer currency digits. Trailing zeroes are removed down to
the minimum. Rounding is Rust's fixed decimal formatting of the supplied binary
f64; exact halfway cases use ties-to-even. For exact monetary calculations,
the application should keep an appropriate decimal model and choose its own
conversion policy before passing a display value.

Non-finite values, invalid fraction budgets and percent scaling overflow return
typed errors before initial creation. A later invalid reactive value reports a
typed UI error and preserves the last accepted text/NodeId. A subsequent valid
update recovers normally. Rendering never writes a sanitized value back into
the business source. Formatted text is patched only when it changes.

Finite f64 has at most 309 fixed integer digits. The maximum fraction budget is
18, so formatting allocation/work is bounded per scalar, including f64::MAX.
There is no source collection scan or cache of previously formatted numbers.
Bindings use WeakScope and retire with their Owner; node_ref is cleared on
retirement. `theme`, `size`, `unstyled`, `class`, `style` and `node_ref` match the
other display widgets. Classes append, supplied style declarations override
only themselves, and unstyled preserves explicit overrides.

`tests/number_formatter.rs` verifies all finite locales/currencies, percent
spacing, rounding/fraction bounds, negative zero/extremes, non-finite and overflow
rejection, reactive recovery/source preservation, stable identity/no redundant
patches, all four themes/unstyled, Owner cleanup and SSR output. Actual browser
hydration/AX and native acceptance are recorded by integration host checks.


Integrated host checks on 2026-10-24 passed: `display-acceptance.js` ran 16
blue/green x light/dark x CSR/adopted SSR display/action cases with zero errors
on Chromium 148.0.7778.96 (`D:/BuildCache/sapid/display-browser.json`).
`examples/native/tests/display.rs` passed three real engine tests, including
four-theme surfaces/formatted text, controlled Pill refusal, disclosure/button
identity and intrinsic document updates. These checks do not establish OS IME
or spoken screen-reader acceptance. New shadow assertions require the later
unified browser rebuild, which is tracked separately.
