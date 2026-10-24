# TextField and Checkbox

Both controls have local state when their controlled setter is omitted. A literal
`value` or `checked` still selects controlled mode: a user event proposes a
change, and the supplied source decides the accepted value. External source
updates do not invoke change callbacks. Typed callbacks run before generic
observers; retiring the registering Owner prevents subsequent callbacks.

TextField requires an accessible name or a Field label. Its `required`,
`readonly` and `invalid` declarations combine with Field metadata; Fieldset
disabled state combines with its own `enabled`. Checkbox uses its intrinsic
label and supports the same Field metadata. Boolean Checkbox, MixedCheckbox
and CheckboxGroup have separate contracts; a mixed value is never silently
reduced to a boolean. FormControl supplies explicit successful name/value data.

Disabled TextField and Checkbox use typed theme colors in every host.
Application styles override these defaults, and `unstyled` removes visual
defaults. State changes preserve node/control identity and do not rewrite
editor values, revisions, selection or preedit. Readonly remains independently
focusable; disabled controls cannot activate or accept edits.

Text edits follow Glory's canonical LF/U+FFFD and monotonic revision protocol.
Composition updates retain editor state, obsolete revisions are discarded,
and controlled refusal restores the accepted value at the host boundary.
Selection offsets use UTF-16. These protocol tests do not claim physical OS
IME or screen-reader speech acceptance.

Evidence: `tests/widgets.rs`, `tests/fields.rs`, `tests/inputs.rs`,
`tests/forms.rs` and `tests/basic_control_state.rs`; native input/field fixtures;
browser basic, field, input, choices and form suites. Combined gate snapshot 8
and `D:/BuildCache/sapid/browser-snapshot8-results.json` cover these contracts
in the same four-theme Web/SSR/hydration build. Native mixed checks and radio
selection remain explicitly rejected as recorded in `adapter-capabilities.md`.
