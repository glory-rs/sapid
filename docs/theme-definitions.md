# Theme definitions

A `ThemeDefinition` owns a complete base `Palette`, a default `ColorScheme`, and
optional independent `PaletteOverrides` for Light and Dark. A resolved `Theme`
contains the actual values consumed by widgets and CSS serialization. No neutral,
danger, disabled, background or text colors are synthesized in its constructor.

```rust
use glory::style::Color;
use sapid::{BLUE, ColorScheme, PaletteOverrides, Theme, ThemeDefinition};

// Copying an existing base is explicit; a fully handwritten Palette is also valid.
let brand = ThemeDefinition::new("brand", BLUE.base).with_dark(PaletteOverrides {
    canvas: Some(Color(12, 14, 18, 255)),
    surface: Some(Color(20, 22, 26, 255)),
    input: Some(Color(24, 26, 30, 255)),
    foreground: Some(Color(240, 241, 242, 255)),
    ..PaletteOverrides::NONE
});
let (scheme, colors) = brand.default();
assert_eq!(scheme, ColorScheme::Light);
assert_eq!(brand.light(), Some(colors));
let dark = brand.dark().expect("supported dark mode");
assert_eq!(dark.border, colors.border); // None inherits only the base
let theme = Theme::new(&brand, ColorScheme::Dark).unwrap();
assert_eq!(theme.canvas, dark.canvas);
```

The default mode is implicitly supported when its override is absent. An alternate
mode is unsupported unless its override is Some; `Some(PaletteOverrides::NONE)`
explicitly supports an unchanged copy of the base. A definition whose default is
Dark can omit Dark overrides and remain Dark-only. Unsupported requests return
None from `.light()`/`.dark()`/`.resolve()`, or an error from `Theme::new`; they never
silently use the other mode. `.default()` always returns a concrete palette.

Both mode overrides apply directly to base, not on top of each other. Transparent
colors are real Some values and replace the base exactly. Resolution returns
owned Copy data and never mutates the definition or another resolved instance.
`Theme::from_default(&definition)` honors the default mode and its overrides.

The base provides all 34 semantic color roles, ten shades and eight chart series colors. Every
role and individual array index has an independent optional override. Typography tokens and radius remain
separate widget configuration. Applications install resolved `Theme::stylesheet()`
output beside the structural Sapid CSS in the outer host; widgets do not inject
CSS. All color roles, shades and series serialize to CSS variables, including page canvas
and the danger fill/text/soft states. `cargo run --example palette_css` generates
the shipped Blue/Green rules; `-- --check` verifies synchronization.

Theme names are CSS-safe ASCII identifiers as documented in the README. Use
separate names for definitions with different CSS data. The preset source is
[src/theme/presets.rs](../src/theme/presets.rs); definitions and overrides are
in [definition.rs](../src/theme/definition.rs) and [palette.rs](../src/theme/palette.rs).
The core logic has no browser/HTTP/runtime engine dependencies.

Verification covers independently overridden modes, all base colors, all overrides,
transparent replacements, implicit and explicit mode support, Dark defaults,
unsupported-mode rejection, real widget styles and generated CSS. Four built-in
contrast checks and the browser's four-mode theme/navigation checks remain in use.

## 2026-10-25 Theme Studio verification

- The full `scripts/check.ps1` default/all-feature tests, doctests, strict Clippy
  and selected Glory gates passed during this rewrite. Log:
  `D:/BuildCache/sapid/studio-library-gate.log`.
- Final schema/backdrop regressions passed 22 palette, swatch and overlay tests;
  the suggestion-group fix passed four additional suggestion tests. Final
  all-feature library Clippy passed. Logs: `studio-backdrop-check.log` and
  `studio-suggestions-check.log` under `D:/BuildCache/sapid`.
- `scripts/check-showcase.ps1 -Serve -Browser` passed 17 Rust tests, host/wasm32
  strict Clippy, WASM build, 20 representative SSR pages, 182 baseline Chromium
  checks and 14 Theme Studio checks. It also compiled the exact clipboard export
  in a separate D-drive project and asserted its seed, base and dark override
  values. Log: `D:/BuildCache/sapid/studio-showcase-final-gate.log`.
- The fast `theme_audit` gate mounts all 149 compiled demos under both green/dark
  and an arbitrary named custom/light theme. No styled widget retains a foreign
  theme class (298 mounts). Report: `D:/BuildCache/sapid/theme-default-audit-final.json`.
- `node scripts/audit-showcase-themes.cjs` additionally visited all 149 actual
  browser routes in green/dark. No unintended default blue/light widget or wrong
  page canvas remained. Report: `D:/BuildCache/sapid/dark-theme-audit.json`.
- The browser flow covers generation, exact/incomplete RGBA editing, inherited
  values, preserved sample state, local save/reload/edit/remove, header selection
  and copy actions, custom modal backdrop, unsupported modes, SSR adoption with
  saved themes, malformed storage preservation and 320/360/390px overflow checks.
- Native example `--all-targets` compilation passed after the palette schema
  expansion. Log: `D:/BuildCache/sapid/studio-native-final-check.log`. This is not
  new OS input or assistive-technology acceptance.
- Current asset manifest SHA256: `54ee6ec6670ddd5108b37f5ada4f33eb13b6d66e6d728f7b9d498cd7fbdd9071`.
- Screenshots: `theme-studio-final-desktop.png`, `theme-studio-dark.png`,
  `theme-studio-mobile.png` and `showcase-dark-fixed.png` in `D:/BuildCache/sapid`.

## Theme Studio and color audit

Open `showcase.html?widget=theme-studio`. This is a Glory + Sapid Rust widget,
including the color inspector, exact RGBA inputs, inheritance controls and preview.
The host module only handles browser storage, stylesheet installation and navigation.

- Generate ten Oklab tones from a seed (index 6 retains the exact seed), with bounded
  chroma reduction to the sRGB gamut. Generation replaces accent roles and retains
  neutral colors. Every output can subsequently be edited.
- Edit the complete Base palette or independent Light/Dark overrides. Each shade
  and chart series index can inherit independently. The default mode always exists;
  an alternate mode can be explicitly enabled or omitted.
- Hex inputs accept six or eight digits. Incomplete input remains visible, retains
  the last valid preview, and disables Save/Copy until corrected. The shared color
  inspector edits hue/saturation/value/alpha for the selected role.
- Preview real buttons, inputs, checks and status alerts. Theme updates preserve
  the sample's controlled values. Four resolved contrast observations are shown;
  they assume opaque backgrounds and are not an accessibility certification.
- Save up to eight previews in this browser's local storage under
  `sapid.theme-studio.v1`. The payload is version 2 and bounded to 128 KiB; malformed data
  is reported and preserved. Saved themes are available in the top-right palette
  picker. The selected theme gets Copy theme code and Edit theme actions. Remove
  an existing preview from its editor to reclaim a slot.
- Copy emits a complete `const ThemeDefinition`, with every base color and only
  explicit overrides. There is no dependency on private showcase helpers. Runtime
  saved themes use `Theme::from_palette` with an owned, validated identifier; the
  resolver never synthesizes colors and no string is leaked.

The audit moved success/warning text and surfaces, string syntax highlighting,
chart series, shadows, modal backdrops, transparency checkerboards, slider/selection markers and
switch thumbs into Palette. Website badges, category marks and shadows also read
these roles. Showcase layout containers explicitly receive the selected theme;
they cannot reintroduce a default blue/light variable scope.

Transparent paint used to clear a surface, hue/saturation spectra, selected color
DATA and analytic black/white endpoints used to generate data remain calculations,
not theme chrome. Preset literals live only in `src/theme/presets.rs`; the CSS
preset block is generated from those definitions.

The page host installs the active resolved palette and paints `html`/`body` as well
as the site root. CSS, WASM loader and WASM binary are published together under an
immutable content-addressed `assets-<hash>/` directory before HTML is replaced.
This prevents a newer page from mixing an older cached stylesheet with its token
schema. Mixed cached assets are a plausible cause of the reported white chrome;
the original stale response was not recoverable, so this is not asserted as its
only possible cause.

Design references: [Mantine colors generator](https://mantine.dev/colors-generator/),
[Oklab's original matrices and explanation](https://bottosson.github.io/posts/oklab/),
and [WCAG text contrast](https://www.w3.org/TR/WCAG22/#contrast-minimum).


The payload version 1 preview schema without `backdrop` is upgraded explicitly at
storage admission, using preset Palette data for only the missing new role. The
original bytes are preserved under `sapid.theme-studio.v1.backup` before writing
version 2. Unknown fields, unrelated missing roles and malformed data still fail
validation without replacement. This migration does not synthesize colors in the
theme resolver.

Client routing keeps one application-owned editor workspace across guide visits
and site theme changes. It preserves incomplete hex input, selected layer, color
inspector state and sample values; it does not keep a cache of visited views.
Saved preview IDs remain attached to that workspace, so repeated saves update an
existing preview. An explicit document reload resets unsaved workspace state.
