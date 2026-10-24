# Explicit UI environment

`theme_provider(theme)` mounts a real View and calls `.content(|context| ...)`
once under its Owner. The context carries a Theme, explicit LTR/RTL direction,
Locale and motion preference. Controls consume the supplied values explicitly:

```rust
theme_provider(theme)
    .direction(Direction::Rtl)
    .locale(Locale::arabic())
    .content(|context| {
        button()
            .theme(context.theme)
            .label(context.locale.message(LocaleMessage::Close))
    })
```

`context.provider()` preserves the current configuration for a nested provider;
its setters override selected mount-time values. `direction_provider` and
`locale_provider` are standalone constructors with default values for the other
context fields. They emit no theme background. Existing controls do not
automatically adopt a context or recolor after a provider is mounted. Direction
must also be passed to navigation builders such as RadioGroup. The shared
semantic descriptor rejects contradictory direction metadata on the same
navigation root.

`ThemeTokens` contains five size entries and fixed typography roles. Checked
immutable customization covers spacing, control height/padding/radius scale,
font size, absolute line height and optional font weight. Button/Input control
styles, Check labels, descriptions, Text and Title read these tables. Default
tokens retain the preceding styles. A Theme clone shares its immutable table;
customization copies that bounded table rather than installing a global registry.

Locale supplies twelve explicit control messages, with English, Chinese and
Arabic defaults. Each custom message is nonblank, contains no NUL/CR/LF and is
at most 1024 UTF-8 bytes; checking precedes installation. `LanguageTag` accepts
2–63 ASCII bytes, a 2–8 letter primary subtag and 1–8 alphanumeric subsequent
subtags. This is a finite syntax, not a language registry, date formatter or
automatic writing-direction detector. Application content remains application
data. Native AX exports the nearest current inherited language.

`context.transition_duration(ms)` accepts at most 2000ms and returns zero for
Reduce. NoPreference still allows the host OS to reduce motion. The environment
fixture consumes this result in a real Collapsible height transition. Existing
host-owned animation clocks and Owner cancellation remain authoritative.

Current evidence under `D:/BuildCache/sapid`: token unit/integration tests
(`theme-tokens-tests.log`), five provider/model/fixture tests
(`environment-fixture-tests.log`), three shared metadata tests
(`environment-tests.log`), complete Set/Keep/Clear HTML difference oracle
(`environment-html-tests.log`), native actual RTL flex with editor identity,
UTF-16 selection and active preedit preserved (`environment-native-tests.log`),
inherited/nested/cleared AX language (`environment-native-ax-tests.log`), and
versioned classic fields (`environment-stream-tests.log`). Immutable HTML11
Web37 passes the original eleven browser cases in all four themes and CSR/SSR
adoption (`environment-browser-web37.json`) and all five classic-stream cases
(`environment-stream-browser-web37.json`), with no page or console errors.
The dirty-editor case uses an uncontrolled default; adoption preserves its
changed value, physical identity and UTF-16 caret.
