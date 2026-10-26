# Native action components

`action_icon("Refresh").icon(children)` is a square native Glory Button with a
required explicit accessible name and caller-supplied display content. Missing
icon content or whitespace-only names fail before creating nodes. The name is
static mount configuration so a reactive name cannot later become empty. Icon
children use existing semantic text/image/widgets; nested controls fail Glory's
shared content checks. This API does not promise a new vector icon renderer.

`close_button()` uses the same ActionIcon/native Button path, displays `×` and
defaults to the name `Close`. `name("Dismiss notification")` localizes or makes
the action specific. Activation, disabled suppression and focus use Glory's
native shared control behavior; there is no duplicated keyboard/pointer driver.

Both expose theme, size, unstyled, class, style, enabled, focus, node_ref, on and
on_press. ExtraSmall/Small/Medium/Large/ExtraLarge squares use 24/30/36/42/50 px. Class overrides append;
style overrides preserve unspecified theme declarations; unstyled removes theme
declarations while preserving explicit styles. Multiple observers run in order
while the Owner remains live. References/captures retire with the Owner.

`variant(ActionIconVariant::{Default,Filled,Light,Outline,Subtle})` selects a
portable appearance for either action. ActionIcon defaults to Default;
CloseButton defaults to Subtle. Disabled colors update with `enabled` and restore
the chosen variant when reenabled, without replacing the native button. Explicit
styles still win, including while disabled. `close_button().icon(children)`
replaces the decorative glyph while preserving its accessible name and behavior.
Square controls have zero internal padding and center their content. Tag remove
actions use the same CloseButton with smaller, size-specific dimensions.

## Connected buttons

`button_group().name("Document actions").buttons([button().label("Save"), ...])`
uses a real semantic Group and native button children in source/natural Tab order.
Only Sapid Button values enter this bounded mounted collection; the component
does not retain a backing business data source. Use `button` to append one item.
Empty groups fail before creation.

Horizontal and Vertical Orientation are supported. Joined groups have a single
outer border/radius and one separator per adjacent pair. Non-joined groups use
an 8 px gap and independent buttons. The group supplies theme/size to children.
Child explicit style declarations take precedence over connection declarations,
including reactive styles. Group enabled and each child's own enabled source are
ANDed; toggling the group cannot enable a child that explicitly disabled itself.
Loading remains an independent disable condition from the existing Button.
Group unstyled removes group-owned outer/connection/button declarations but
preserves explicit child styles. An unstyled child retains its own override.

`toolbar(true)` selects the typed Toolbar contract and Collection/RovingFocus
registry. Only one enabled, non-loading button is in the Tab order. Arrow keys,
Home and End move actual focus without activating; horizontal direction follows
`direction(Direction::Rtl)` when configured. `loop_focus(false)` keeps edge
focus in place. Tab exits the toolbar. Explicit child focus declarations are
rejected because the toolbar owns its tab stops. Plain groups retain natural
Tab order. Both modes keep structural flex/orientation/gaps when unstyled.

## Disclosure action

`burger()` is a named disclosure button with a decorative three-line Graphic that
transitions to a cross. It owns local
expanded state by default. `open(source)` is controlled: activation proposes the
opposite source value through `on_open_change`, and never mutates the supplied
source. `default_open` cannot override controlled ownership regardless of setter
order. External source changes are silent. Internal acceptance precedes typed
callbacks and generic observers; disabled activation is suppressed natively.

`labels("Open navigation", "Close navigation")` selects two nonempty localized
names; `controls(SemanticId)` links to an application-owned panel. The control's
NodeId remains stable as name, glyph and expanded metadata change. This component
does not create the panel, infer actual focus or implement routing. It uses real
expanded/pressed/controls metadata and finite host-owned bar interpolation. See
`docs/burger.md` for motion budgets, interruption and evidence.

Burger event closures use WeakScope. Local state, names, bindings, node references
and callback captures retire with its Owner; no timeouts or independent tasks are
started. There is no owning Scope captured inside its handlers.

## Evidence boundary

ButtonGroup accepts 1–128 buttons. Iterator setters consume only the remaining
allowance plus one overflow sentinel; repeated button setters retain at most 129
buttons and reject overflow before node creation. Excess capture destruction runs
outside graph borrows. Shared configuration-budget tests include million-item
lazy iterators and reactive writes from capture destructors.

`tests/actions.rs` covers names/content validation, all sizes, native Button shape,
disabled and observer order, local/controlled disclosure state and rejection,
source-silent updates, controls/SSR escaping, all themes/unstyled, Owner cleanup,
connected borders/orientations/source order, group/child disabled AND, reactive
style precedence and empty groups. Real browser and native host acceptance are
separate integration checks; these tests do not claim OS IME, spoken screen
reader output; animation has its own focused checks in `docs/burger.md`.


Integrated host checks on 2026-10-24 passed: `display-acceptance.js` ran 16
blue/green x light/dark x CSR/adopted SSR display/action cases with zero errors
on Chromium 148.0.7778.96 (`D:/BuildCache/sapid/display-browser.json`).
`examples/native/tests/display.rs` passed three real engine tests, including
four-theme surfaces/formatted text, controlled Pill refusal, disclosure/button
identity and intrinsic document updates. These checks do not establish OS IME
or spoken screen-reader acceptance. The latest unified rebuild passed the same
16 cases including the new Paper shadow and Pill content-width assertions.

`tests/toolbar.rs` adds four roving/RTL/non-looping/SSR/observer regressions.
`examples/native/tests/toolbar.rs` checks actual focus, no activation and engine
identity in all four themes. The browser action suite now checks true Toolbar
role/orientation, one Tab entry, RTL arrows, Home/End, disabled skipping, Tab exit,
vertical non-looping edges and Space activation. See also [Button](button.md)
for section slots, state colors and the five shared sizes.
