# Component and adapter evidence

This matrix records implemented finite contracts, not full parity with every
upstream option. Record/SSR validate structure; browser tests validate actual DOM,
keyboard/pointer input, hydration and Chrome AX snapshots. Native engine tests
validate layout and controlled state; AccessKit tests validate semantic exports.
None of these imply OS IME, screen-reader speech or untested devices.

| Component family | Web / SSR / classic stream | Native Blitz / AccessKit |
|---|---|---|
| TextField, fixed Textarea, PasswordInput, Field, Fieldset, InputGroup | Existing input/field suites, dirty hydration and composition protocol | Actual layout/editor identity; password raw/display protocol shared by Glory |
| Autosizing Textarea | Browser field-sizing:content; fixed rows remain explicit | FieldSizing rejected before allocation and direct apply |
| Form and native submit/reset buttons | Actual form/type/name/value, queued requests, Enter, no implicit network or early reset | Shell Enter/pointer requests queue until host tick; editor identity/raw text preserved |
| FormState | Owner-scoped model, synchronous/async cancellation and bounded snapshots | Same host-independent model; files explicitly unsupported |
| MixedCheckbox | Real checkbox indeterminate after adoption/stream; no-JS HTML retains semantic state | Explicit rejection; no fake boolean value |
| CheckboxGroup and Switch | Native checked/role, controlled refusal, keyboard and label activation | Real checkbox engine checked; independent Switch AX role |
| RadioGroup / SegmentedControl | Native radio/radiogroup, single Tab, arrows/Home/End, controlled whole-group restoration; stream dirty peers protected | Radio explicitly rejected until a native one-way selection driver exists |
| Toggle / ToggleGroup | Native button and aria-pressed, roving groups; nine themed/dirty-adoption cases passed | Button identity/geometry and AccessKit toggled state; no native fake checkbox |
| Chip / ChipGroup | Actual Checkbox or Radio according to selection mode; shared Field and controlled restoration | Checkbox supported; single-select Radio explicitly rejected |
| Collection / RovingFocus / Typeahead | Bounded live Owner registrations, RTL, disabled skipping, Unicode prefix/repeat/timeout; ten choice reports passed | Actual RTL key driver supported; character-search metadata/listeners explicitly rejected |
| Badge, Pill, Card, Paper, EmptyState, NumberFormatter | Four-theme CSR/hydration, controlled pill removal and stable shell, headings/actions/number text | Four-theme real layout, native button action and stable engine identity |
| Code, Kbd, Mark, Highlight, text lists, Blockquote, DataList, Item | Intrinsic tags/roles; dt/dd structure; cross-Mark text selection retained during unrelated text updates | Real layout and dedicated intrinsic AccessKit roles |
| Box/Flex/Stack/layout Group/Center/Container/Space | Shared finite style, responsive available width, actual wrapping/center in eight cases | Four-theme real engine bounds/wrapping/gaps/center |
| Grid / SimpleGrid / AspectRatio | Finite tracks, spans/offsets/auto placement and positive ratio, shared CSS and field oracle; eight themed cases passed | Four-theme real track/span/ratio geometry and input/engine identity; breakpoint provider remains pending |
| Text / Title / Typography / Anchor | Semantic text variants/headings, real ellipsis, current/new link context and explicit navigation requests; nine cases passed | Four-theme text/current-context layout and callbacks; ellipsis/new context explicitly rejected |
| Separator / Timeline | Passive separator axes and actual ordered list/progress text, stable keyed children | Four-theme geometry, child input identity and intrinsic AccessKit roles |
| Spoiler | Actual Resize height drives clipping/disclosure, controlled refusal and focus after shrink; eight cases passed | Four-theme real text/viewport geometry and original engine identity |
| Breadcrumbs / Pagination / NavLink | Current::Page separate from selection, bounded folding/page window, link/disclosure separation; nine cases passed | Four-theme real controls/current metadata; Unmount hidden-child disposal regression passed after immutable engine fix |
| Stepper | Current::Step, explicit selection policy and FormState validation; focused browser run pending | KeepMounted focus/identity and repeated Unmount Owner retirement passed in four themes |
| VisuallyHidden | Semantic plain text with zero clipped out-of-flow box; focused browser AX run pending | Zero real box/unchanged flow, text identity and AccessKit name/live description passed |
| Card/Paper shadow | Finite single outset shadow or explicit none; shared CSS/patch mapping | CPU raster pixels verified for offset, blur/spread and clearing |
| Dialog/Drawer/Popover | Existing host-owned Layer and hydration suites | Layer explicitly rejected; native top layer remains planned |

Plain SSR cannot encode the DOM-only indeterminate property. Hydration validates
the whole subtree before adopting it; classic stream initializes live controls.
Radio uses native name for both mutual exclusion and successful-control name;
conflicting metadata is rejected. FormState serialization is an explicit bounded
snapshot in registration-key order, not automatic DOM FormData equivalence.

Current focused evidence files live under `D:/BuildCache/sapid`: `forms-browser.json`
(10 reports), `choices-browser-results.json` (10 reports), `display-browser.json`
(16 reports), `display-native.log`, `layout-native.log`, and `toggles-native.log`.
Document screenshots are `document-*-hydrate.png`; its suite runs eight themed
cases. The unified `browser-results.json` is regenerated only after all suites
run against one current build. Older reports describe the build they actually
ran and must not be treated as evidence for newer additions.

Rust evidence is versioned alongside each family: `tests/form_state.rs`,
`tests/forms.rs`, `tests/choices.rs`, `tests/radio.rs`, `tests/segmented_control.rs`,
`tests/toggles.rs`, `tests/display.rs`, `tests/actions.rs`, `tests/document.rs`,
`tests/number_formatter.rs`, `tests/layout.rs` and `examples/native/tests/`.
Shared Glory regressions cover descriptor validation, transactional prepare,
preflight/direct apply, successful-prefix query watermarks, field/full-shape
mapping and native AX/host boundaries. The roadmap keeps partial features such
as toolbar policy, Burger animation, native radio and general motion explicit.

The 2026-10-24 `scripts/check.ps1 -Native` gate completed successfully in
`D:/BuildCache/sapid/current-native-gate.log`: Sapid tests/docs/strict Clippy,
shared style/render/core/ui/html/SSR tests, Glory native semantic/profile/AX
tests and all native component fixtures. The separate fork provenance gate
`engine-pins-check.log` verified all six immutable engine packages and ran
Taffy 185 tests, Blitz 100 tests and the parallel profile/style/drop matrices.
These reports cover that source snapshot; later component batches require their
own focused evidence and another integration gate.
