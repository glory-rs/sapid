# Display surfaces and tags

`badge()`, `pill()`, `card()`, `paper()` and `empty_state()` implement Glory
`Widget` directly. They use semantic Text/View/Button nodes and portable typed
Style declarations. They do not inject HTML, script or adapter options.

## Tags

`badge().content(source).variant(BadgeVariant::Soft)` is passive text. Soft,
Solid and Outline use the selected theme's accent/background/foreground/border
tokens. A badge does not silently become a live status announcement.

`pill().content("Rust").removable("Remove Rust")` has a passive label and a
separate native button displaying `×` with the supplied accessible action name.
Names must be nonempty. Omit `removable` for a passive pill. `enabled(false)`
disables the removal button, while preserving readable content.
The root uses typed `Display::InlineFlex`: it fits its label/action content in a
normal block parent instead of painting a full-width tag. An explicit display
override retains precedence; unstyled output adds no component display value.

Removal hides local pills by default and calls `on_remove()` before registered
`on_remove_press` observers. `visible(source)` makes visibility controlled:
removal only calls the callback, which proposes dismissal to the application.
Defaults never override a controlled source regardless of setter order. External
visibility changes are silent. Hidden pills retain their node identities and
Owners; this is deliberate so controlled rejection/reappearance does not rebuild
content. Applications that need disposal can remove the widget in their parent
collection or conditional owner. Hidden pills have disabled removal controls.

`node_ref` identifies the root, `remove_ref` the independent native button.
References and callback captures retire with the component Owner. Event closures
use WeakScope. They retain no owning Scope, visited-key history, tasks or timers.

## Surfaces

`paper().fill(children)` provides a surface with padding/radius/theme colors.
Its border is opt-in with `bordered(true)`.

`card().title("Settings").description("Manage your preferences")` provides
ordered `header`, title, description, body (`fill`) and `footer` slots. Its border
defaults to enabled. The title is heading content (`heading_level(3)` by default),
and description is paragraph content. Levels outside 1–6 fail before creating
nodes. Header/body/footer remain stable ordinary container shells. Both surfaces
are passive by default; links and buttons in slots retain native behavior and
natural Tab order. `name` explicitly names a surface as a semantic group.

Both surfaces accept `shadow(ShadowSize::Small/Medium/Large)` using explicit
theme-aware finite outset-shadow tokens. The default leaves author shadow styles
in charge; `shadow(ShadowSize::None)` explicitly clears them. `unstyled` removes
component shadow declarations, while an explicit `Style::box_shadow` override
retains precedence. Typed single-layer shadows are supported by Web/SSR and the
native CPU painter; inset/multiple shadows remain outside the finite contract.

## Empty content

`empty_state().title("No messages").description("Start a conversation")`
provides illustration, heading, description, body and actions slots in that order.
The heading defaults to level 2. The illustration is caller-supplied: use a
decorative image only when its meaning is already conveyed by the title. Actions
are ordinary semantic controls. EmptyState emits no unsolicited alert or status
role and does not infer loading from empty data.

## Appearance and support

All five components provide `theme`, `size`, `unstyled`, `class`, `style` and
`node_ref`. Explicit classes append to component classes. Style overrides replace
only supplied declarations; absent values preserve theme declarations. `unstyled`
removes all component-owned theme declarations, including Card title/description
and the Pill button; explicit overrides and child widgets keep their own styles.

The nodes and style declarations are supported by Record, SSR/stream, Web and
Blitz. Headless/SSR tests establish semantic shape, escaping, controlled state,
disabled behavior, node stability and Owner cleanup. They do not establish
browser pixel layout, hydration/AX acceptance, OS input or screen-reader speech.
The integration fixture is `examples/web/src/display_form.rs`; browser/native
evidence must be recorded separately after actual host checks.

## Verification

`tests/display.rs` covers reactive content/classes/style, all Badge variants,
passive semantics, local/controlled Pill removal, disabled controls, names,
callback order, identity retention, reference/capture retirement, Card slot order,
heading validation, border/theme/unstyled declarations, EmptyState ordering and
SSR escaping/native button/heading shapes.


Integrated host checks on 2026-10-24 passed: `display-acceptance.js` ran 16
blue/green x light/dark x CSR/adopted SSR display/action cases with zero errors
on Chromium 148.0.7778.96 (`D:/BuildCache/sapid/display-browser.json`).
`examples/native/tests/display.rs` passed three real engine tests, including
four-theme surfaces/formatted text, controlled Pill refusal, disclosure/button
identity and intrinsic document updates. These checks do not establish OS IME
or spoken screen-reader acceptance. The later unified rebuild also passed all
16 display/action cases with actual Paper shadow and Pill inline flex
shrink-to-content assertions. Native `tests/pill.rs` verifies the pill fits its
content within a 400px block parent for every theme. Updated screenshots are
`D:/BuildCache/sapid/display-{blue|green}-{light|dark}.png`.
