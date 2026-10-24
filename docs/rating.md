# Rating

`rating(key)` provides whole- or half-star values using actual native radio controls in a RadioGroup. The default is five stars, whole-star precision, and value zero with no selected radio. `count` is fixed at construction and limited to 1–10. `RatingPrecision::Half` permits exact half values. Finite values must be inside `0..=count` and exactly on the chosen whole/half grid; program values are rejected rather than rounded.

```rust
let score = scope.cage(2.5);
field("quality")
    .label("Quality")
    .description("Choose a score in half-star increments")
    .control(move |context| {
        rating("quality")
            .field(context)
            .precision(RatingPrecision::Half)
            .clearable(true)
            .form_name("quality")
            .value(score)
            .on_value_change(move |next| score.revise(|value| *value = next))
    })
```

`.value(...)` is a controlled, read-only source. Omitting it uses `.default_value(...)` as local state. Callbacks propose a complete numeric value; rejected controlled proposals restore the actual native radio peers. Accepted external changes are silent. Invalid initial values, counts or names create no nodes; invalid later source values retain the accepted selection and graphics. The root accessible name is limited to 512 UTF-8 bytes and validated by borrowing before cloning. Names, selected state and all registrations retire with the mounting Owner.

Each half-star hit region is a separate named radio, such as “2.5 of 5”. Two adjacent half regions clip the same ten-point star shape, producing one complete star without a gap between its halves. The original RadioGroup selection, Collection, native checked reconciliation and roving controller remain in use. Transparent native controls cover passive typed Graphics, and a separate finite FocusWithin decoration makes keyboard focus visible. There are at most 20 star regions plus one optional clear radio, and eleven path commands per star graphic. No application DOM search, raw SVG insertion or button impersonation is used.

The group has one sequential tab stop. Horizontal arrows select the adjacent whole/half value; `.direction(...)` follows the existing RadioGroup keyboard contract. Home/End reach the first/last enabled member, and the default radio navigation loops. `.clearable(true)` adds a distinct “Clear rating” radio with value zero. Clicking an already selected star does not implicitly clear it. Disabled members refuse changes and sequential focus; readonly retains native radio semantics while refusing mouse/keyboard changes. Typed value callbacks precede generic CheckedChanged observers.

Field supplies group naming, descriptions, errors, required/readonly/disabled state and theme/size. Its label names the group; it does not promise that clicking a label on a non-labelable RadioGroup View focuses a member. Use the members' real radio focus and keyboard behavior. Successful controls use the explicit bounded ASCII `.form_name(...)`, or the stable generated native group name when omitted. Field's dynamic `form_name` does not supply this native association. Every actual checked radio carries the numeric value, such as `2.5`, independently of its internal stable registration key. Form transport remains application-owned.

`unstyled()` retains the real radio controls, half-star geometry and visible focus, using neutral foreground graphics. Five sizes and all themes are supported. This baseline uses fixed star shapes and whole/half precision; it does not add arbitrary precision, custom icon renderers or a hover-preview value contract.

For passive display, choose the explicit mode:

```rust
rating("score-summary")
    .name("Quality")
    .precision(RatingPrecision::Half)
    .value(score)
    .display_only()
```

Display mode renders a named Group, for example “Quality: 2.5 of 5”, and bounded decorative star Graphics. It has no radios, checked state, tab stops, form registration or selection behavior. Values remain reactive and validated. Field/form-name registration and change callbacks are explicitly rejected in this mode. It is separate from `.readonly(true)`, which retains the interactive radio structure.

The native Blitz host currently rejects interactive Radio controls with the typed error `Unsupported("native radio groups")`, including readonly radio groups. Rating preserves that existing boundary. Native applications can use `display_only()` for verified star rendering. There is no automatic fallback to a different control or implied native radio input support.

Evidence: eleven component tests, two finite-domain model tests and the seven existing Radio tests pass. They cover whole/half selection, exact numeric submission values, explicit clear, controlled rejection, invalid recovery, callback order, Field/readonly/disabled state, wrapped navigation and visible FocusWithin decoration, names/count budgets, Owner cleanup, native-radio SSR and passive display policy. Three native tests pass: four-theme actual half-star Raster pixels, real half-region dimensions and stable identities; interactive and readonly typed rejection; and descriptor preflight/direct-apply rejection preserving the accepted prefix and query watermark. The native target was verified as `D:/BuildCache/glory-semantic-ui/target`, with one build job. Logs are `D:/BuildCache/sapid/ratings-rust.log` and `rating-native.log`; production, browser fixture and native tests pass strict Clippy.

Chromium **148.0.7778.96** passed **10/10** actual browser cases, with no page or console errors. All four themes run in CSR and hydration, covering real half-region mouse hits, visible focus, native radio keyboard/clear/endpoints, controlled peer restoration, readonly/disabled behavior, silent external changes and invalid-source recovery, exact numeric successful values, Field errors, stable unrelated editor/caret, passive display and unstyled geometry. The browser AX tree contains actual named radio nodes. Additional cases verify dirty-editor adoption and Owner teardown, and touch tapping/cancellation. The green/dark hydration screenshot was visually inspected. Reports are `D:/BuildCache/sapid/rating-browser.json` and `.log`; screenshots are `rating-{hue}-{scheme}-{csr|hydrate}.png`. The fixed same-source WASM/bindgen/SSR fixture is `D:/BuildCache/sapid/web/rating-next`, built separately from root Web18. Synthetic protocols and AX-tree inspection do not establish physical-touchscreen or screen-reader announcements.

