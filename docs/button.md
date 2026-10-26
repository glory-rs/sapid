# Button

`ButtonVariant` also supports `Light`, `Outline` and `Subtle`, alongside Primary,
Secondary and Danger. These are typed theme appearances, with the same native
button, loading/disabled suppression, callback order and style-override rules.
They do not require showcase-only CSS or custom application color calculations.

`button().label("Save")` builds a real Glory Button. Primary, Secondary and
Danger variants use the four supplied themes. ExtraSmall, Small, Medium, Large
and ExtraLarge share control minimum heights of 24/30/36/42/50 px; ActionIcon,
CloseButton and Burger use the corresponding square dimensions. Text still
determines the final button width and can increase its height.

`left_section(children)` and `right_section(children)` place caller-supplied
inline text/images around a separately bound label. Structural inline flex,
centering and an 8 px gap also survive `unstyled()`. `fill(children)` inserts
additional content after the label. Repeated slot setters replace that slot.
The shared Button content validator rejects nested controls and block content.
No icon renderer or spinner is implied by these slots.

Section buttons require a nonempty label or an explicit nonempty accessible
`name`. Their automatic name follows the accepted label and excludes decorative
section text. Invalid reactive labels preserve the previous accepted label/name
and report an error; a later valid update recovers without recreating text or
button nodes. An explicit name permits an intentionally empty visual label.

`loading(source)` sets busy metadata and disables activation independently of
`enabled(source)`. Disabled theme colors are portable typed declarations, so
the same state is visible without the optional stylesheet and in the native
engine. Explicit style declarations take precedence. External updates do not
invoke press callbacks. The optional stylesheet adds hover/focus-visible and
reduced-motion behavior; applications supply localized loading text or icons.

Every `on_press` callback runs before generic Activate observers regardless of
setter order. Each callback rechecks Owner liveness; retirement skips remaining
callbacks. This ordering also applies to ActionIcon, CloseButton and Burger.
Browser/native controls supply keyboard and pointer activation, rather than
duplicating those drivers in Sapid.

`tests/button_enhancements.rs` has eight focused tests, including failed builds,
reactive rejection/recovery, five sizes, typed state colors, escaping, callback
order and retirement. Existing actions (14) and toolbar (4) tests still pass.
`examples/native/tests/button_enhancements.rs` checks all four themes with real
engine boxes, exactly one 8 px section gap, loading, label updates and engine
identity. The live native AccessKit busy true/false/absent reset test passes.
`examples/web/buttons-acceptance.js` passed eight CSR/adopted-SSR cases on
Chromium 148.0.7778.96 with actual AX busy state, disabled suppression,
Space activation, stable identities and mobile overflow checks. Reports and
screenshots are under `D:/BuildCache/sapid/buttons-browser.json` and
`buttons-*-hydrate.png`. These checks do not establish spoken screen-reader
announcements or OS input-method acceptance.
