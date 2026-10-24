# NumberInput

`number_input()` is an actual single-line text input with intrinsic `spinbutton`
semantics. Its sibling Decrease/Increase controls are real buttons. It shares
Glory's ordinary input revisions, selection, composition and Field associations;
there is no hidden second business value, default HTTP action, serde dependency,
locale discovery, timer, query or DOM access in the component.

```rust
use glory::prelude::*;
use sapid::{Decimal, NumberValue, field, number_input};

let value = scope.cage(NumberValue::Decimal(Decimal::parse("0.1")?));
field("amount")
    .label("Amount")
    .form_name("amount")
    .control(move |context| {
        number_input()
            .field(context)
            .value(value)
            .min(Some(Decimal::parse("0").unwrap()))
            .max(Some(Decimal::parse("10").unwrap()))
            .step(Decimal::parse("0.1").unwrap())
            .on_value_change(move |next| value.revise(|old| *old = next))
            .on_commit(move |next| value.revise(|old| *old = next))
    })
    .show_in(scope)?;
```

The public builder provides `value`/`default_value`, optional reactive `min`/`max`,
reactive `step`/`page_step`, `locale`, `grouping`, `controls`, `field`, `form_name`,
`required`, `readonly`, `invalid`, and the common theme/size/unstyled/style/name/
enabled/focus/node_ref/event methods. `node_ref` addresses the actual input.
`controls(false)` omits the buttons and retains keyboard and accessibility input.
Defaults are empty, no explicit bounds, step 1, page step 10, EnUs and no grouping.
`min(None)`/`max(None)` deliberately omit the corresponding accessibility bound.

## Exact values and drafts

`Decimal` uses a normalized coefficient and scale, with at most 18 fractional
digits and absolute value at most 10^18. Construction, comparison, stepping and
clamping use bounded i128 arithmetic, including alignment of fractional scales.
`coefficient()`, `scale()`, `parse()`, `format()` and `to_f64()` are explicit;
`to_f64()` is a lossy accessibility coordinate, never the stepping algorithm.
Even when two exact values have the same f64 coordinate, adjustment availability
comes from the exact model.

`NumberValue` has `Empty`, `Decimal(Decimal)` and `Text(String)` variants. Every
user edit becomes `Text`, including a complete `1.00`, so trailing zeroes, signs,
grouping and caret positions survive an accepted echo. Program writes and steps
can use `Decimal`. `NumberValue::parse(locale)` and `parse_number(text, locale)`
return `NumberParse::{Empty, Complete, Incomplete { candidate }, Invalid}`.
A bare sign, trailing group and trailing decimal remain drafts; they are never
silently converted to zero. Incomplete/invalid drafts have no `aria-valuenow`.
They retain their exact text in `aria-valuetext` and carry `aria-invalid`.

Input is limited to 128 UTF-8 bytes, with no NUL, CR or LF. Initial and reactive
source text is validated by borrowing before cloning. An oversized host edit
invokes `on_reject(NumberError)`, restores the accepted text after the whole host
tick, and is not cloned into generic observers. Invalid program configurations
return a typed UI error and retain the last accepted snapshot; recovery updates
the existing input. No historical drafts, visited inputs or requests are cached.

Locales are explicit `NumberLocale::{EnUs, DeDe, ZhCn}`. EnUs/ZhCn use `.` decimal
and `,` grouping; DeDe uses `,` decimal and `.` grouping. Grouping is strictly
three digits after an initial one-to-three digit group. Only ASCII digits and an
optional initial sign are accepted. Exponents, whitespace, other digit sets and
more than 18 fractional digits are rejected. `grouping(true)` formats Decimal
values; it does not rewrite each raw Text edit. This finite locale contract does
not promise full CLDR/Intl formatting, arbitrary currency or exponent input.

## Editing, stepping and ownership

ArrowUp/ArrowDown, PageUp/PageDown and known-bound Home/End issue independent
`Event::Spin` requests only on the focused spin input. Ordinary text inputs and
Left/Right/Delete/caret shortcuts retain normal behavior. Unknown-bound Home/End
remain editor caret operations. Composing and modified keys do not enter the
Spin branch. Composition drafts publish once on terminal composition; actual OS
IME, virtual keyboard layouts and screen-reader announcements are not claimed by
the protocol tests.

Increase/decrease uses the exact parsed candidate, or zero for an empty/invalid
draft, then adds the configured step and clamps to explicit bounds and the exact
decimal domain. A direction never reverses to recover an outside value: an input
above max can decrease into range, while Increase is unavailable. Bounds never
silently mutate an external source. Outside values stay visible and invalid.
An explicit Commit proposes the clamped Decimal for a complete number or a
trailing decimal with a candidate; empty stays empty and an invalid/bare sign
stays its draft. Host Commit follows the normal Input Enter protocol. For single-line inputs in
a Form, hosts queue ValueChanged, Commit and Keyboard Submit in that order, so
typed commit callbacks update the business value before the Form snapshot.
NumberInput does not consume the ancestor Form submission. DOM `change` alone
is not a Glory Commit.

Adjustment availability is an AX/button advertisement, not a stale keyboard
permission: all writable keyboard intents are queued and evaluated in order
against the confirmed current cursor. From min, Up then Down in one tick returns
to min; a no-op Down followed by Up still advances once. Disabled/readonly and
exact-origin checks remain mandatory.

Controlled sources are read-only to the widget. `on_value_change` receives edits
and step proposals; `on_commit` receives explicit commits and completed steps.
Applications decide which to accept. A commit-only acceptance also advances the
same-tick cursor. Refused requests use the current accepted value on the next
intent. Two accepted requests in one host tick accumulate in order. External
changes are silent. Typed callbacks run before generic observers, and retirement
during a callback prevents later callbacks. Callback captures are cleared outside
registry/graph borrows on Owner retirement.

A stable borderless Fieldset provides actual inherited disabled behavior for
input and buttons, with no conflicting sequential Input-enabled/Spin-metadata
updates. Readonly preserves editor focus/selection and successful form values,
while withholding adjustment and editing. Field supplies the actual input's
label, description, required state, errors, readonly/disabled state and successful
name; an explicit `form_name` can override the field name. Successful values are
the displayed raw text. Native form submission/normalization, persistence and
server validation remain application policies. File inputs are not supported.

## Shared semantic contract

`Semantics::spinbutton: Option<SpinValue>` refines the intrinsic role of a
single-line Input with Text/Number purpose. Explicit `Role::SpinButton` on a View
is rejected. `SpinValue` has optional finite min/max/value coordinates, positive
finite step/page_step, bounded value_text and explicit can_increase/can_decrease.
Outside coordinates require invalid metadata; disabled/readonly inputs cannot
advertise adjustment. `Event::Spin { from, command: RangeCommand }` validates the
exact physical origin and finite commands independently of Slider/Splitter.
Endpoint commands require known bounds.

HTML maps these fields to actual input ARIA metadata. Clearing metadata removes
the intrinsic role and numeric attributes without changing input identity/value.
Both full shaping and the field-local lowering oracle cover this transition.
AccessKit exports SpinButton, exact value text, optional numeric coordinates,
step and only available adjustment actions. Numeric SetValue issues Spin; String
SetValue retains the ordinary text editing path. f64 SetValue is necessarily a
lossy external accessibility request; applications needing exact program values
should write Decimal directly. Native keyboard input queues requests and does
not execute application callbacks in the foreign listener.

The semantic choices follow [ARIA spinbutton](https://www.w3.org/TR/wai-aria/#spinbutton)
and the [APG spinbutton pattern](https://www.w3.org/WAI/ARIA/apg/patterns/spinbutton/).
These references describe the intended semantics; executable evidence is below.

## Evidence

- `tests/number_inputs.rs`: 15 passed. Exact and same-tick stepping, complete raw
  lexemes/drafts/empty/commit, controlled refusal and callback order, commit-only
  cursor, bounded source and host edit refusal, invalid-domain recovery, Field
  relations, composition protocol and retirement during a callback.
- `src/number_model.rs`: 6 passed through Cargo, including exact scale/domain,
  strict explicit locales, overflow and million-byte rejection before clone.
- Glory render Spin model 3 and target tests 3 passed; ordinary Input/Range target
  isolation, malformed origins, numeric availability and collapsed f64 coordinates.
- Glory HTML Spin integration 1 passed and **all 13 HTML library tests passed**,
  including the extended full-shape/field delta oracle and Clear/no-op roles.
- Glory native foundation 2 passed: actual queued keys/IME-modifier isolation,
  unchanged editor, SpinButton AX, optional numeric values/exact text and queued
  numeric actions. `D:/BuildCache/sapid/spin-native.log`.
- Shared native Enter order regression 1 passed: Input Commit before Form Submit,
  unchanged textarea/IME behavior. `D:/BuildCache/sapid/number-enter-native.log`.
- `examples/native/tests/number_input.rs`: 2 passed, covering actual four-theme
  editor geometry, focus, sampled raster colors, identity, exact same-tick steps,
  readonly/disabled recovery, unstyled dimensions, retirement and zero queries.
  `D:/BuildCache/sapid/number-input-native.log`.
- Sapid lib/NumberInput tests and Web WASM strict Clippy passed. Actual build
  targets were checked with Cargo metadata and are on D.
- Fixed **Web21 actual 12/12 passed**, Chromium 148, errors[], including real
  Enter clamp followed by normal Form submission and terminal composition.
  `D:/BuildCache/sapid/number-input-browser-root21.json`.
- A strengthened same-tick boundary probe then found Up/Down being filtered by
  old availability metadata; the before-fix result is preserved in
  `D:/BuildCache/sapid/number-input-web21-cursor-before-fix.json`. The source and
  new 15th component regression are fixed. The same original 12 browser reports
  include the stronger Up/Down assertion at the minimum in every theme.
  **Fixed Web22 actual 12/12 passed**, Chromium 148, errors[];
  `D:/BuildCache/sapid/number-input-browser-root22.json`.

There is no claim of physical touch-device/OS IME or screen-reader acceptance.
The native component tests and driver/AX tests exercise the real native adapter;
headless App event tests alone are not presented as native keyboard acceptance.
