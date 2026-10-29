# Component and adapter evidence

The current per-work-item source is the [explicit capability ledger](widget-capabilities.md),
generated with the [interactive gallery](../examples/web/gallery.html) from
`docs/capability-matrix.json` and the canonical inventory. It distinguishes tested,
model-only, rejected and pending states for all 149 work items. The family table
and gate history below preserve earlier snapshots; newer native Layer and other
capabilities supersede their older refusals only where the explicit ledger and
canonical evidence say so.
The [2026-10-29 Glory main migration](glory-main-migration-2026-10-29.md)
records the current native consumer regressions and superseded refusal assertions.

This matrix records implemented finite contracts, not full parity with every
upstream option. Record/SSR validate structure; browser tests validate actual DOM,
keyboard/pointer input, hydration and Chrome AX snapshots. Native engine tests
validate layout and controlled state; AccessKit tests validate semantic exports.
None of these imply OS IME, screen-reader speech or untested devices.

| Component family | Web / SSR / classic stream | Native Apace / AccessKit |
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
| Stepper | Current::Step, explicit selection policy and FormState validation; nine browser reports passed | KeepMounted focus/identity and repeated Unmount Owner retirement passed in four themes |
| VisuallyHidden | Semantic plain text with zero clipped out-of-flow box; eight browser/AX reports passed | Zero real box/unchanged flow, text identity and AccessKit name/live description passed |
| Button / ActionIcon / ButtonGroup | Five sizes, real section gaps, busy/loading and disabled typed styles, toolbar roving keys | Four-theme geometry, stable identity, real keyboard focus and busy AX reset |
| Menu / DropdownMenu | Bounded submenu, checks/radios, pointer/focus, keyboard, dirty hydration and classic stream | Layer and precise pointer interests explicitly rejected |
| Tooltip / HoverCard | Owner-bound delays, shared description, joint pointer/focus presence; hydration/stream | Layer and precise pointer interests explicitly rejected |
| Avatar / Image / BackgroundImage | Real revision-specific image decode and fallback, stale completions ignored | Actual PNG resources, per-consumer generations, cancellation and cache boundaries |
| Indicator / ThemeIcon | Nine positions, finite native SVG paths and decorative/named graphic separation | CPU raster layout, clipping, group opacity and real pixels; requires paint |
| NativeSelect | Real select/option/optgroup, controlled single/multiple selection, FormData, hydration/stream | Explicit rejection before mutation; native select driver remains pending |
| Select | Real button combobox and owned Option/Listbox, independent activity/selection, read-only/controlled restoration, typeahead, rapid same-tick keys, touch and local reveal; Web11 eleven reports passed | Choice metadata and RevealOption explicitly rejected |
| Progress / RingProgress / SemiCircleProgress | Read-only numeric/indeterminate semantics, finite line/arcs, stream resets | Four-theme real raster arcs/track, stable identity, numeric AccessKit state |
| Alert / AlertDialog / FloatingDialog / Skeleton | Live regions, explicit modal cancellation focus, nonmodal natural Tab, busy decorative placeholders | Alert/Skeleton real geometry; all Layer variants explicitly rejected |
| Sidebar / AppShell | Actual Resize contexts, responsive navigation reparenting, retained editors | Four-theme layout; modal navigation refuses unsupported Layer before moving |
| ScrollArea / Scroller | Actual metrics, sequential bounded commands, keyboard and controlled editor preservation | Actual scroll metrics, viewport keyboard input and native geometry |
| Splitter | Conserved pane size, controlled ratio, range keyboard, captured mouse/touch, cancel | Four-theme layout, range AX actions and real keyboard driver |
| Affix | Viewport Fixed, explicit container Absolute/Sticky and reactive offsets; nine reports passed | Actual scroll/resize Fixed and container Absolute; Sticky explicitly rejected |
| Spinner / Loader | Real finite SVG rotation, dynamic reduced-motion, named busy/status or decorative; nine reports passed | Explicit motion preference, shared renderer clock, CPU rotation pixels, hidden pause/Resume and cleanup |
| NavigationProgress | Application start/set/finish/cancel, revoked stale tokens, stable editor; eight reports passed | Four-theme actual 40%/75% widths and engine identity through restart |
| ModalsManager | Bounded FIFO with one active Dialog/AlertDialog, stable metadata updates, queued cancellation, once-only results; eight Web11 reports passed | Layer rejected at manager mount before admission; app remains usable |
| Notifications / Toaster | Bounded FIFO, same-key replacement, one explicit host timer, hover/focus/manual pause and action/close; ten Web13 reports passed | Four-theme actual layout/actions/identity; hover opt-in unsupported, default persistent/timer available |
| Sheet | Four edges, captured mouse/touch snap/dismiss, revision guards, controlled refusal, viewport cancellation, dynamic motion and stream; nine Web14 reports passed | Layer/sheet explicitly rejected with accepted-prefix/query watermark preserved |
| NavigationMenu | Real landmark/links/disclosure groups, nesting/hover/controlled behavior, dynamic focus lifecycle and nested layer positioning; nine Web14 reports passed | Closed header/NavLink supported; opening Group Layer explicitly rejected |
| Toolbar / ActionBar | Cross-group roving keys, RTL/vertical/nonloop, source-controlled selection actions, stable controls; eight Web14 reports passed | Four-theme actual grouped geometry, typed navigation and engine identity |
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
as Burger animation, native radio and general motion explicit.

The 2026-10-24 `scripts/check.ps1 -Native` gate completed successfully in
`D:/BuildCache/sapid/current-native-gate.log`: Sapid tests/docs/strict Clippy,
shared style/render/core/ui/html/SSR tests, Glory native semantic/profile/AX
tests and all native component fixtures. The separate fork provenance gate
`engine-pins-check.log` verified all six immutable engine packages and ran
Takia 185 tests, Apace 100 tests and the parallel profile/style/drop matrices.
These reports cover that source snapshot; later component batches require their
own focused evidence and another integration gate.

The later combined `scripts/check.ps1 -Native -Web` snapshot 8 passed in
`D:/BuildCache/sapid/gate-snapshot8.log` (976 passed-report sum across 238 test
result reports, including doctests and repeated targets). Its matching
`browser-snapshot8-results.json` records Chromium 148.0.7778.96, all 31 suites,
264 reports and an empty errors array. That snapshot covers the completed
families through Splitter. `engine-image-pins-check.log` verifies the newer
immutable image-observation engine pin with 106 Apace tests and 185 Takia tests.

Snapshot 9's combined gate passed with 1019 passed-report sum across 261 test
result reports (`gate-snapshot9.log`). Its browser run stopped at the custom
Select typeahead defect; `browser-snapshot9-partial.json` records 29 completed
suites and is explicitly incomplete. The corrected Select and modal focus
handoff passed focused Web11 runs in `select-browser-web11.json` and
`modals-browser-web11.json`. Affix, Spinner and NavigationProgress have their
focused Web9 reports above. These are separate snapshots; a later unified run
must cover subsequent shared inert/navigation work before claiming integration.
The corrected Web11 unified run then passed all 36 suites, 309 reports and no
errors in `D:/BuildCache/sapid/browser-web11-results.json`. It includes Select,
ModalsManager, Affix, Spinner and NavigationProgress against one fixed build.

Snapshot 10's complete combined gate passed with 1085 passed-report sum across
281 result reports (`gate-snapshot10.log`, including repeated targets/doctests).
Its Web12 browser run passed 38 suites/330 reports before stopping on a
NavigationMenu focus defect; `browser-web12-partial.json` is incomplete.
Focused Combobox 12/12 (`combobox-browser-web12.json`) and LoadingOverlay 9/9
(`loading-overlay-browser.json`) passed against that same fixed build. The
NavigationMenu correction requires a subsequent browser build.

Unified Web14 passed all 43 suites and 378 reports, errors=[] in
`D:/BuildCache/sapid/browser-web14-results.json`. It includes Notifications,
Toolbar/ActionBar, Sheet, NavigationMenu and the first Auto/MultiSelect matrix.
The later stable listbox-name audit, MultiSelect bulk commands and mid-settle
motion-preference switch need their subsequent focused/unified snapshots.

The next full gate passed in `gate-snapshot15.log`: 1163 passed-report sum across
309 result reports, including repeated targets and doctests. Native linking
uses one Cargo build job after the first parallel link attempt exhausted MSVC
memory. The earlier failed attempts remain separate logs. Web15 focused reports
cover the later component changes; `burger-browser.json` and
`color-swatch-browser-web15.json` each record nine passing actual scenarios.
