# Sapid

[Showcase](https://sapid.taidge.org/) · [GitHub repository](https://github.com/sapid-ui/sapid)

Pushes to `main` validate and publish the showcase through Cloudflare Pages.
See [deployment configuration](deploy/README.md) for the pinned Glory revision,
required Actions secrets, release build and custom-domain setup.

Mantine 全组件 API/功能对照与实施记录见 [181 项差异报告](docs/mantine-api-comparison.md)，
完整 Props 与本地入口见 [API 索引](docs/mantine-api-surface.md)。组件清单的 done 不表示上游 API 全兼容。

Composable widgets for Glory, written with Rust builders and the public
`Cage` / `Bond` / `Vary` reactive model. Applications select their host; Sapid
does not install a browser runtime, stylesheet, font loader, or native engine.

The frozen catalogue covers **268 upstream entries in 149 completed work items**.
Each item records its finite implementation, tests and adapter boundaries in the
[roadmap](docs/widgets-roadmap.md) and [capability ledger](docs/widget-capabilities.md).
Completion does not imply every upstream feature or every platform is supported.

Final commands, host boundaries and run evidence are collected in the
[verification record](docs/widgets-verification.md).

The library includes typed controls and forms; layout, typography and themes;
menus, layers and navigation; tables, trees and indexed choices; dates and
resource schedules; accessible charts; files, carousel and lightbox; a finite
rich document editor; and application-supplied chat, questionnaire, task and
trace displays. Applications retain ownership of data, authorization, transport
and execution. Builders use real semantic controls and the existing Glory host
contracts.

Default builds keep the core widget graph small. Optional features are explicit:

| Feature | Widgets / scope |
|---|---|
| `json` | Bounded JsonInput / JsonViewer and explicit JSON parsing |
| `code` | CodeBlock / CodeHighlight |
| `dates` | Civil date/time controls, calendars and resource schedules |
| `charts` | Cartesian and specialized charts, tables and bounded sampling; portable `libm` geometry |
| `files` | Permission- and budget-bound file controls |
| `editor` | Finite rich documents, Web editing and classic streams; native editing is explicitly rejected |
| `chat` | Messages, indexed output, prompts, questionnaires, model/suggestion controls, plans, confirmations and public traces; includes `files` |

Browser, native and AccessKit acceptance are recorded separately. Native facets
that are unavailable return typed errors; browser composition tests do not imply
OS IME or screen-reader acceptance.

The [Sapid showcase](docs/showcase-site.md) provides a Mantine-inspired home page,
149 widget guides, live Rust examples, search, themes, source code and a
Button property playground. The page and its controls are built with Glory +
Sapid; the HTML entry only boots WASM. Build and open it locally:

```powershell
./scripts/check-showcase.ps1 -Serve -Browser
# http://127.0.0.1:8795/
```

The separate [capability audit gallery](examples/web/gallery.html) preserves
the detailed host evidence and original acceptance fixtures.

It targets the current semantic Glory checkout in the sibling `../glory`
directory. Publishing and a standalone versioned Glory dependency are pending.
The current integration follows Glory's new `main` branch: reactive property
sources use `Vary<T>`, and native examples pin the same engine revisions as Glory.
The deployment manifest locks the exact tested Glory commit. Sapid builders
continue to implement `glory::ui::Widget`; Glory's separate `display` HTML API
does not replace the semantic controls used for input, SSR and native hosts.

Full widget comparison and completed work records:
[widgets roadmap](docs/widgets-roadmap.md), with a
[machine-readable inventory](docs/widget-inventory.json) covering the Mantine
and shadcn official catalogues. Run `python scripts/widget-plan.py --check` to
validate the source-to-task coverage and generated report.

```rust
use glory::prelude::*;
use sapid::{button, checkbox, text_field};

let app = App::memory();
app.mount_with(|| {
    let name = app.scope().cage(String::new());
    let notifications = app.scope().cage(false);
    col().fill((
        text_field()
            .name("Username")
            .value(name)
            .on_value_change(move |value| name.revise(|current| *current = value)),
        checkbox()
            .label("Receive notifications")
            .checked(notifications)
            .on_checked_change(move |checked| {
                notifications.revise(|current| *current = checked);
            }),
        button().label("Save").on_press(|| println!("Save requested")),
    ))
})?;
# Ok::<(), glory::ui::Error>(())
```

## Widgets

| Widget | Configuration |
|---|---|
| `button()` | `label`, `fill`, `variant(Primary/Secondary/Danger/Light/Outline/Subtle)`, `loading`, `on_press` |
| `text_field()` | `value` / `default_value`, `purpose`, `multiline`, `on_value_change`, `on_commit` |
| `textarea()` | TextField state/events and Field integration; `rows`, `min_rows`, `max_rows`, `autosize` |
| `password_input()` | TextField state/events and Field integration; `visible` / `default_visible`, `on_visibility_change`, `visibility_labels` |
| `input_group()` | One primary `input(widget)`, ordered `start(children)` / `end(children)` slots |
| `input_addon()` / `input_section()` | Noninteractive `content` / `fill(children)` with independent control children |
| `checkbox()` | `label`, `checked` / `default_checked`, `on_checked_change` |
| `field(key)` | `label`, `description`, `error`, `required`, `readonly`, `invalid`, `control(factory)` |
| `label(id, control_id)` | `content`, `required`; native control activation |
| `description(id)` / `field_error(id)` | `content`; empty content hides the stable shell |
| `fieldset(key)` | `legend`, `disabled`, `fill`; native grouping and inherited disabled state |
| `collapsible(key)` | `label`, `open` / `default_open`, `on_open_change`, `content`, `content_policy` |
| `accordion(key)` | `item(accordion_item(key))`, `open` / `default_open`, `multiple`, `collapsible`, `heading_level`, `regions`, `loop_focus`, `content_policy` |
| `tabs(key)` | `tab(tab(key))`, `value` / `default_value`, `on_value_change`, `orientation`, `activation`, `loop_focus`, `content_policy` |

Further APIs and exact acceptance boundaries:

- [Forms and requests](docs/form-widgets.md), [validation and serialization](docs/forms-state.md).
- [TextField and Checkbox state, themes and evidence](docs/basic-controls.md).
- [Mixed checks, switches, radios and segmented values](docs/choices-implementation.md), [toggle buttons](docs/toggles.md).
- [Action buttons and connected groups](docs/action-widgets.md), [tags and surfaces](docs/display-widgets.md).
- [Intrinsic document content](docs/document-widgets.md), [finite number formatting](docs/number-formatter.md).
- [Layout boxes](docs/layout-widgets.md), [portable shadow and wrapping](docs/portable-shadow-wrap.md).
- [Grid and ratio](docs/grid-layout.md), [typography](docs/typography.md), [links](docs/anchor.md).
- [Timeline and separators](docs/timeline-separator.md), [Spoiler](docs/spoiler.md), [navigation](docs/navigation.md).
- [Stepper](docs/stepper.md), [VisuallyHidden](docs/visually-hidden.md), [mounted collections](docs/collection-implementation.md).
- [Button sections, sizes and loading](docs/button.md), [menus](docs/menu-implementation.md), [hints](docs/hints-implementation.md).
- [Native selection](docs/native-select-implementation.md), [alerts, dialogs and skeletons](docs/feedback-widgets.md), [progress](docs/progress.md).
- [Custom Select](docs/select-implementation.md), [Spinner and motion](docs/spinner.md), [NavigationProgress](docs/navigation-progress.md), [modal queues](docs/modals-manager.md).
- [Indicators and icons](docs/indicators-icons.md), [Sidebar and AppShell](docs/sidebar-app-shell.md), [scrolling](docs/scroll-area.md), [Splitter](docs/splitter.md).
- [Affix positioning](docs/affix.md).
- [Adapter capabilities and actual host evidence](docs/adapter-capabilities.md).

Input controls support `name`, `enabled`, `focus`, `node_ref`, `on`, `theme`,
`size`, `class`, `style`, and `unstyled`. Dynamic values use `Vary` where the
underlying declaration is reactive; theme, size, focus and multiline are mount
configuration. A TextField needs a Field label or accessible `name`; Checkbox's `label` uses
Glory's associated native checkbox label. `name` explicitly overrides that name.

### State and events

- `value(...)` and `checked(...)` are controlled even when given literal values.
  User events propose changes through callbacks. They never implicitly write a
  supplied Cage. The external source decides the displayed value.
- Omit those methods to use widget-owned state. `default_value(...)` and
  `default_checked(...)` initialize it once. A default does not override an
  explicitly controlled source, regardless of configuration order.
- External source changes do not invoke user-change callbacks. Rejected native
  edits/toggles are restored after the complete host tick, including a literal
  controlled value. All same-revision input events delivered by a host tick reach
  callbacks before reconciliation allocates a new input revision. Hosts may
  coalesce uninterrupted native input observations before delivering that tick.
- TextField change callbacks include composing input. `on_commit` receives the
  host's final text and updates local state, without synthesizing another change
  callback. Observe `EventKind::Composition` for composition-aware validation.
- Internal state updates happen before the typed change/commit callback; generic
  `.on(...)` observers follow it. Repeated `.on` / `.on_press` observers append
  in registration order. Singular `on_value_change`, `on_commit`, and
  `on_checked_change` setters replace the previous typed callback. Remaining
  observers are skipped if an earlier callback disposes the widget Owner.
- Disabled controls use Glory's native enabled semantics. Button `loading(true)`
  also disables activation; applications provide loading labels/status content.
- User text follows Glory's canonicalization, revision and composition rules.
  Single-line newlines are rejected. Multiline values normalize CRLF/CR to LF;
  NUL becomes U+FFFD. Rendering does not silently rewrite the supplied source.
- State, observers, bindings and event captures retire with the widget Owner.
  `node_ref` points to the semantic control and is cleared on retirement.

### Specialized inputs and composition

```rust
use glory::prelude::*;
use sapid::{field, textarea, password_input, input_group, input_addon, text_field};

let app = App::memory();
app.mount(col().fill((
    field("bio").label("Biography").control(|context| {
        textarea().field(context).autosize(true).min_rows(2).max_rows(6)
    }),
    field("secret").label("Password").control(|context| password_input().field(context)),
    field("website").label("Website").control(|context| {
        input_group().start(input_addon().content("https://"))
            .input(text_field().field(context))
    }),
)))?;
# Ok::<(), glory::ui::Error>(())
```

Textarea row counts are positive `u16` mount configuration (default: three).
`min_rows` defaults to `rows`; `max_rows` is optional and must not precede the
minimum. Sizes use the effective typed line height, padding and borders; explicit
height/min/max style overrides win. Unstyled inputs use the size's line-height
fallback when none is supplied, so specify a typed line height when replacing it.
Autosize uses Glory's typed `FieldSizing::Content`, including soft wrapping,
shrinkage and later width/font changes, with scrolling after the maximum. It
requires a browser supporting CSS `field-sizing: content`; Chromium acceptance
is recorded. SSR and stream share the same CSS serialization. Blitz explicitly
rejects field sizing in both preflight and direct apply; fixed rows remain usable.
No measurement script, hidden duplicate editor, or value rewrite is installed.
`examples/native` keeps native widget layout/identity/refusal tests outside the
default dependency graph; `scripts/check.ps1 -Native` runs them with the engine.

PasswordInput starts masked. Its named button toggles the existing input's
purpose without changing text revisions. Visibility is independently controlled
by `visible(...)`, or widget-owned when omitted; external updates are silent.
`visibility_labels(show, hide)` provides nonempty localized action names. The
input's `enabled`/Field disabled state also disables the visibility button.
Read-only values may still be revealed. Input callbacks, names, classes, styles,
focus and node references address the input; the toggle retains its own native
keyboard position. Revealing intentionally makes the value visible, including
to accessibility clients. Avoid embedding real secrets in SSR snapshots.

InputGroup is an ordered layout, with no synthetic input role or focus target.
Pass FieldControl to its primary input. Addons remain text and never become an
implicit name; InputSection can contain real buttons with their own names and
normal Tab order. Use Fieldset for group disabling. Theme/size/unstyled configure
each supplied part independently; no ambient child mutation is performed.

### Appearance

Default tokens are portable typed Glory styles. Choose `Theme::blue_light()`,
`Theme::blue_dark()`, `Theme::green_light()` or `Theme::green_dark()`, customize
public token fields, and choose `Size::{ExtraSmall, Small, Medium, Large, ExtraLarge}`. `light()` and
`dark()` select the blue variants. Each hue exposes ten palette steps, while
semantic tokens independently select surfaces, readable text, accents, errors,
borders and disabled states. Green's filled-button color is darker than its
display palette to keep white normal-size text above a 4.5:1 contrast ratio.
Theme selection is mount configuration; the gallery switches themes by navigating
to a fresh fixture while preserving the current gallery page and theme across
sidebar navigation. Native focus indicators remain host-owned.
Checkbox retains the host's native checkbox appearance; its theme styles the
label container/text rather than drawing a replacement checkbox.

`class(...)` appends user classes to stable `sapid-button`, `sapid-text-field`,
or `sapid-checkbox` classes. `style(...)` overrides only explicitly supplied
declarations, preserving other tokens. Dynamic removal restores token defaults.
`unstyled()` removes theme declarations while preserving native semantics,
widget classes and explicit user styles. No raw CSS or JavaScript is injected
by a widget.

Web applications can explicitly load `styles/sapid.css` for focus rings, hover,
required indicators and native checkbox accents. It uses the same four theme
classes as Rust and respects reduced motion and `unstyled()`. The documentation
gallery uses Mantine-inspired spacing, neutral surfaces, sidebar navigation,
preview cards and API tables. Its stylesheet and scaffolding belong to the host.

## Fields and form groups

```rust
use glory::prelude::*;
use sapid::{field, fieldset, text_field, checkbox, Theme};

let app = App::memory();
app.mount_with(|| {
    let email = app.scope().cage(String::new());
    let error = app.scope().cage(String::new());
    fieldset("account")
        .legend("Account details")
        .theme(Theme::green_dark())
        .fill((
            field("email")
                .label("Email address")
                .description("Used for account notifications.")
                .error(error)
                .required(true)
                .theme(Theme::green_dark())
                .control(move |context| text_field()
                    .field(context)
                    .value(email)
                    .on_value_change(move |next| email.revise(|value| *value = next))),
            field("updates")
                .description("Optional product news.")
                .theme(Theme::green_dark())
                .control(|context| checkbox().field(context).label("Receive updates")),
        ))
})?;
# Ok::<(), glory::ui::Error>(())
```

`FieldControl` explicitly connects exactly one control to stable label, help and
error IDs. It carries the Field's theme, size and state; it installs no ambient
context. Pass the same theme explicitly to each Field inside a Fieldset. A Field
requires a control factory. Keys use the same bounded ASCII namespace rules as
composite widgets. Label/description/error shells retain identity across text
and state changes; absent parts are omitted at construction. An explicit control
`name(...)` overrides its Field naming relation. Checkbox's own label remains
clickable when an external Field label supplies its accessible name.

`required`, `readonly` and `invalid` also work directly on TextField/Checkbox.
Field and control flags combine with OR. A nonempty Field error implies invalid
and adds its error relation alongside the help description. Clearing it removes
the relation and hides the error shell. `invalid(true)` can express invalid state
without error text. These states do not validate business data or submit a form.
Metadata patches preserve input identity, value revision, caret and composition.

Readonly controls remain focusable and programmatically writable. Native editing
and checked-change proposals are suppressed, while text selection/copy remain
available. A disabled Fieldset blocks descendant editing, activation and focus;
reenabling it preserves each control's own enabled state and draft. Legend is
noninteractive, so this API does not expose HTML's first-legend control exception.
`required` is a native/accessible state, with no cross-host validation engine.

Standalone `label(label_id, control_id)`, `description(id)` and `field_error(id)`
builders use Glory `SemanticId`s. Give the target an ID and `labelled_by` /
`described_by` / `error_message` through typed `Semantics`. Native HTML labels
focus text inputs and toggle checkboxes; the Blitz shell performs the same
association. Native AccessKit exports required/read-only/invalid and live
label/help/error relations, plus inherited disabled state.

## Composite widgets

```rust
use glory::prelude::*;
use sapid::{accordion, accordion_item, collapsible, tab, tabs, text_field};

let app = App::memory();
app.mount(col().fill((
    collapsible("advanced-settings")
        .label("Advanced settings")
        .content(|| text_field().name("Additional notes")),
    accordion("help")
        .item(accordion_item("shipping")
            .label("How does shipping work?")
            .content(|| text().content("Shipping information")))
        .item(accordion_item("returns")
            .label("Can I return an item?")
            .content(|| text().content("Return information"))),
    tabs("account-settings")
        .name("Account settings")
        .tab(tab("profile").label("Profile")
            .content(|| text_field().name("Display name")))
        .tab(tab("security").label("Security")
            .content(|| text_field().name("Password").purpose(Purpose::Password))),
)))?;
# Ok::<(), glory::ui::Error>(())
```

Widget namespaces and item keys are explicit ASCII tokens (letters, digits,
hyphens and underscores, 1–64 bytes). Choose different namespaces for widgets
sharing a document, including widgets in independent Apps, and use the same keys
on server and client. Length-prefixed generated IDs keep different keys and
parts distinct. Glory rejects duplicate live names within an App before creating
a node; names retire with their semantic nodes. There is no global ID counter or
visited-widget registry. Labels can contain arbitrary supported Unicode text.

Each panel shell stays mounted with a stable ID and a typed naming/control
relation, even when its content is closed. `ContentPolicy::Unmount` lazily mounts
content when opened and releases its Owner, inputs, subscriptions and tasks when
closed. This is the default for Collapsible and Accordion.
`ContentPolicy::KeepMounted` constructs content once, preserving drafts and node
identity while hidden. This is the default for Tabs. Hidden kept content cannot
receive ordinary semantic events or focus, but its state, bindings and tasks
remain alive. Hiding is distinct from semantic Suspend and from disposing an
Owner. No exit-animation lifetime is implied.

Collapsible has a native button trigger. Enter/Space toggles it. Its `open(...)`
source follows the same controlled-state rules as TextField: callbacks propose,
and the source decides the final state. `trigger_ref` and `panel_ref` expose
semantic identities without DOM handles. When an observed focused panel closes,
it requests focus on the trigger after the visibility/removal commit. Closing a
panel whose focus was outside does not request focus.

Accordion accepts statically configured, uniquely keyed items with `label`,
`enabled`, `content`, `trigger_ref`, and `panel_ref`. Single-open mode is the
default; `multiple(true)` permits several open keys. `open(...)` takes
`Vary<Vec<String>>`; `default_open(["key"])` initializes local state. Unknown,
duplicate or disallowed selections produce typed errors and preserve the last
valid UI. `collapsible(false)` requires one open item for a nonempty collection;
its sole open header remains focusable with semantic activation disabled.
Headers are headings at level 3 by default. `heading_level(1..=6)` changes that
level; `regions(true)` explicitly marks panels as regions, rather than creating
many landmarks by default.

Accordion Up/Down, Home/End move focus among enabled headers without opening
panels; `loop_focus(false)` stops at the ends. All enabled headers retain their
native Tab stops. Descendant inputs/buttons keep their ordinary keyboard
behavior. A root's `enabled(false)` disables its triggers; arbitrary controls
inside panels retain their own enabled state.

Tabs supports `TabActivation::{Automatic, Manual}` and
`Orientation::{Horizontal, Vertical}`. The default is horizontal/automatic with
wrapping. Left/Right (horizontal) or Up/Down (vertical), Home/End navigate enabled
tabs. Perpendicular arrows, modified keys, composing keys, Tab and editor keys
remain native. Automatic selection follows actual focus observations, not a
successful focus request. Manual navigation moves only focus; Enter/Space
activates. Rejected controlled proposals leave selection unchanged while allowing
focus movement. Focus plus activation for the same tab in one host tick produces
one proposal. External source changes do not invoke change callbacks.

There is one Tab stop among enabled tab triggers. Leaving the list restores that
stop to the selected enabled tab; otherwise it falls back to an enabled member.
Panels are focusable Tab stops when visible. A programmatically selected disabled
tab may keep its panel visible; disabling it never silently rewrites a controlled
value. When switching away from a focused panel, focus restoration targets the
new selection's enabled tab, or its panel when that tab is disabled (focusing a
different tab would otherwise trigger automatic activation). The default local selection is the first enabled
item, or the first item when all are disabled. Empty tab collections and unknown
selected keys are rejected. Items are configured at mount time; dynamic item
registration/reordering and virtual data sources are outside this release.

Composite `.on(Activate, ...)` observers follow internal trigger behavior and
typed callbacks. Navigate observers follow internal navigation. Other generic
observers belong to the root. Item factories run under their content Owner.

The interaction rules are based on WAI's [Disclosure](https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/),
[Accordion](https://www.w3.org/WAI/ARIA/apg/patterns/accordion/) and
[Tabs](https://www.w3.org/WAI/ARIA/apg/patterns/tabs/) patterns.

## Dialog, Drawer and Popover

```rust
use glory::prelude::*;
use sapid::{dialog, drawer, popover, text_field, DrawerSide};

let app = App::memory();
app.mount(col().fill((
    dialog("account-editor")
        .label("Edit account")
        .title("Account settings")
        .description("Update your account name")
        .content(|| text_field().name("Account name")),
    drawer("navigation")
        .label("Navigation")
        .title("Pages")
        .side(DrawerSide::Left),
    popover("quick-edit")
        .label("Quick edit")
        .title("Options"),
)))?;
# Ok::<(), glory::ui::Error>(())
```

All three use one shared layer implementation. Dialog and Drawer are modal;
Popover defaults to a nonmodal dialog and can opt into `.modal(true)`.
The trigger exposes expanded/controls/has-popup semantics. Panel IDs remain
stable; titles and optional descriptions supply naming relations. `.name(...)`
explicitly overrides the title/trigger naming relation. All panels use the
semantic Dialog role; Popover is intended for interactive dialog-like content,
not a menu or tooltip with different keyboard behavior.

Use `.open(source).on_open_change(callback)` for controlled state, or
`.default_open(bool)` for local state. Escape and primary pointer-down outside
request closure only for the top layer. `.dismiss_on_escape(false)` and
`.dismiss_on_outside(false)` disable individual requests; a disabled Escape
request still stops at the top layer. `.on_dismiss(...)` receives the finite
Escape/Outside reason, before `on_open_change(false)` and generic Dismiss
observers. Trigger toggles and the built-in close button call `on_open_change`.
Controlled rejection leaves the layer open and retains its focus constraints.
`.close_label(...)` customizes the close button; `.show_close(false)` omits it.

The browser adapter coordinates layers across Apps in the same Document and uses
the HTML manual popover top layer,
preserving the semantic/DOM ancestry and delegated events. Modal layers make
outside branches inert, constrain actual focus and cycle Tab/Shift+Tab. Opening
focuses `.initial_focus(SemanticId)` when it resolves to an enabled focusable
descendant, otherwise the first enabled tab stop, then the panel itself.
Closing restores the trigger or `.return_focus(SemanticId)` when focus still
belongs to the closing layer. A nonmodal outside click that focuses another
control keeps that new focus. Inert attributes introduced by the host and open
layers are cleaned up on closure or App teardown; pre-existing inert attributes
are preserved. Focus actions remain intents, with native events reporting the
actual outcome. Composition and repeated Escape keys do not dismiss layers.

Drawer supports Left/Right/Top/Bottom viewport edges and optional finite Sheet
snap points with captured pointer/touch grips, viewport cancellation and reduced
motion. See [Drawer evidence](docs/drawer.md) and [Sheet](docs/sheet.md). Popover uses
`.placement(AnchorPlacement { side, align, gap, viewport_padding, flip, shift })`
with four physical sides and Start/Center/End alignment. Flip and shift are
independent collision policies. The default is Bottom/Start with a 6 px gap and
8 px viewport padding. Scroll/resize placement work coalesces at the host read
boundary. `trigger_content` and `close_content` customize the actual Buttons
while retaining their accessible names and control behavior. See
[placement and native evidence](docs/popover.md). `.class/.style/.theme/.size/.unstyled` configure the
panel; placement and modality remain host semantics. The default content policy
is Unmount; KeepMounted preserves factory content and its tasks while hidden.
Titles, descriptions, the close button and panel shell have permanent Owners.

A document-owned weak registry coordinates current layer leases across Apps.
The active modal controls Escape/Tab routing, focus, background inertness and
viewport scroll locking. Closing layers out of order and retiring nested Apps
preserve the surviving modal's constraints; final cleanup restores the host's
original scroll/style state. SSR emits stable inert markup; physical top-layer
presentation begins after successful browser adoption. Classic streams carry
finite HTML shapes without a pre-WASM interaction driver.

Native Dialog/AlertDialog and nonmodal Dialog Preserve use the genuine pinned
Blitz fork's viewport layout, top-layer painting and hit testing while retaining
logical ancestry and editor identity. Native Drawer/Sheet, Menu/Tooltip/Listbox
layers and ContextPoint remain explicit capability rejections. These boundaries
are checked before mutation and preserve accepted-prefix query watermarks;
[Popover evidence](docs/popover.md) records the exact supported placements,
focus/AX behavior and real native consumer tests.

The focus rules follow WAI's [modal dialog pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/);
top-layer presentation uses the [HTML popover contract](https://html.spec.whatwg.org/multipage/popover.html).

## Development and verification

```powershell
./scripts/check.ps1
./scripts/check.ps1 -Web -Serve
./scripts/check.ps1 -Web -Native -Bindgen <path-to-wasm-bindgen-0.2.129.exe>
```

Build output and compiler temporaries go to `D:/BuildCache/sapid`; Cargo home is
the existing D-drive Glory cache. Both manifests are checked with `cargo metadata`
before building. The Web example needs `wasm32-unknown-unknown` and
`wasm-bindgen-cli 0.2.129`. It serves CSR and real SSR adoption at
`http://127.0.0.1:8794/` and `/hydrate.html`.
Composite examples are at `/composites.html` and `/composites-hydrate.html`.
Layer examples are at `/overlays.html` and `/overlays-hydrate.html`.
Fields and four-theme examples are at `/fields.html?theme=blue&scheme=light` and
`/fields-hydrate-{blue,green}-{light,dark}.html`. The basic, composite and overlay
CSR pages accept the same `theme`/`scheme` parameters; corresponding
`hydrate-{hue}-{scheme}.html`, `composites-hydrate-{hue}-{scheme}.html` and
`overlays-hydrate-{hue}-{scheme}.html` snapshots match their Rust themes.
Pass `-Bindgen <path-to-wasm-bindgen.exe>` when that version is not on PATH.

`examples/web/{acceptance,composite-acceptance,overlay-acceptance,field-acceptance}.js` are async Playwright functions accepting a Page.
Run it against that server. It verifies native activation, accepted/rejected
controlled edits/toggles, local state, same-tick edits, caret preservation,
composition protocol, hydration drafts/identity, and teardown. The build script
does not run browser acceptance automatically. Rust integration tests cover the
semantic model, stale/future revisions, event composition, styling, error
propagation, SSR escaping and repeated Owner cleanup.

With an installed Playwright package, `node scripts/browser.cjs` runs acceptance
in an independent headless browser and saves its JSON report and screenshot to
`D:/BuildCache/sapid`. Set `SAPID_PLAYWRIGHT` to an existing Playwright package
directory if it is not resolvable as `playwright`; install any new tooling/cache
under the D-drive build root. Native Rust tools and the Web example both pass
strict Clippy with warnings denied.

Glory's `Scope::set_input_value`
helper allocates revisions from the live node, and `Scope::report_error` forwards
custom widget Effect/handler failures to the next App flush. These helpers have
independent regression tests in Glory.
The shared HTML mapper also treats explicit `Checked(Set/Clear)` as a control
property synchronization barrier, including equal values. This restores rejected
native toggles; `Checked(Keep)` and metadata-only patches do not write the value.

Glory also provides optional typed `Semantics` metadata: bounded stable IDs,
controls/labelled-by relations, expanded/selected states, hidden panels,
activation-disabled triggers, and explicit navigation boundaries/membership.
`Role::{TabList, TabPanel, Heading}` are structural View roles; `Semantics::tab`
refines a Button while retaining its native activation behavior. Metadata uses a
field transaction with rollback; independent patches do not rewrite control
values. Reactive `focus(...)` supports roving Tab stops. Web and native shell
share finite orientation-to-navigation mapping; AccessKit exports live semantic
states and resolves named relations only to exported targets. Symbolic relations
may refer forward or to absent nodes at the lower Glory layer; Sapid's stable
panel shells avoid dangling controls. Snapshot IDs are independent of NodeId and
canonical rekeying, and the existing HTML edit representation is retained.

The completed inventory and per-widget evidence are the current verification
record. `scripts/browser.cjs` registers 120 acceptance suites; registration is
separate from a successful run, whose JSON records every completed suite,
report and error. The check script runs the HTML patch differential oracle,
strict Clippy and WASM checks/build. Native engine/AX tests remain separate from
browser tests and from displayed-window or OS-client acceptance.

The earlier 2026-10-24 baseline of 55 Sapid tests, 21 Chromium groups, 210 Glory
tests and 46 native tests is historical; it does not describe the completed
catalogue or the final verification totals.

Field regressions verify native label focus/checkbox activation, independent
required/read-only/invalid state, combined help/error descriptions, inherited
disabling, program writes, stable values/revisions, Owner cleanup, SSR escaping,
and pre-adoption drafts/caret. The four-theme browser matrix also checks actual
Chromium AX data, search, theme navigation and mobile overflow. An additional
matrix mounts all three older galleries in four themes with CSR and SSR adoption
(24 cases), checks their actual control colors, and follows theme-preserving links.
Native probes
exercise engine pointer/key handling and a valid AccessKit consumer tree.

Local evidence lives under `D:/BuildCache/sapid`. Each widget document links
its exact immutable snapshot, acceptance JSON, native logs and screenshots;
`browser-results.json` records the final runner's current output. Earlier Field
logs and gallery screenshots remain historical evidence. All native build output stays in the verified D-drive Glory
target. The default normal dependency graph still contains only Glory's
facade/core/render/style/ui packages.

Motion support and reduced-motion boundaries are recorded per widget; no
universal animation behavior is inferred from catalogue completion. Browser
protocol tests do not establish actual OS IME or screen-reader acceptance. The native tests use the
Glory document/shell and AccessKit consumer without claiming displayed windows,
new Windows UIA client acceptance, or screen-reader speech for Sapid.


### Theme definitions

`BLUE` and `GREEN` are `ThemeDefinition` values, each with complete base colors
and optional independent `PaletteOverrides` for light/dark modes. No colors are
created or substituted by `Theme::new`.

```rust
use sapid::{BLUE, ColorScheme, PaletteOverrides, Theme, ThemeDefinition};
use glory::style::Color;

let brand = ThemeDefinition::new("brand", BLUE.base).with_dark(PaletteOverrides {
    canvas: Some(Color(12, 14, 18, 255)),
    foreground: Some(Color(240, 241, 242, 255)),
    ..PaletteOverrides::NONE
});
let (scheme, colors) = brand.default(); // Light and the complete base palette
assert_eq!(scheme, ColorScheme::Light);
assert_eq!(brand.light(), Some(colors)); // no light override is necessary
let dark_colors = brand.dark().expect("dark is supported");
assert_eq!(dark_colors.surface, colors.surface); // omitted fields inherit base
let theme = Theme::new(&brand, ColorScheme::Dark).unwrap();
assert_eq!(theme.canvas, dark_colors.canvas);
```

The default mode is always available, even when its override is absent. A
non-default mode is supported only when its override is `Some`; an empty
`PaletteOverrides` explicitly supports that mode using the unchanged base.
`with_default_scheme(ColorScheme::Dark)` can make the base a dark-only theme.
`definition.light()`/`.dark()` return `Option<Palette>`; unsupported modes return
`None`. `definition.default()` returns `(ColorScheme, Palette)` with the default
mode's overrides applied. Resolving one mode never changes the base or the other
mode. `Theme::new(&definition, scheme)` rejects unsupported modes;
`Theme::from_default(&definition)` uses the definition's default mode.

Theme names are 1–64 ASCII lowercase letters, digits or hyphens, starting with a
letter. Use distinct names for distinct CSS themes. All colors, including canvas,
surface, text, borders, disabled/danger states and focus colors, belong to the
base palette and its overrides. No neutral colors are implicit in the resolver.

Widget colors use typed styles. For Web/native CSS states, install the resolved
theme's `stylesheet()?` alongside `styles/sapid.css` in the outer host; widgets
never inject CSS. `cargo run --example palette_css` regenerates the shipped
rules; `-- --check` verifies them. Palette selection remains mount-time configuration.


The [Theme Studio](http://127.0.0.1:8795/showcase.html?widget=theme-studio) generates
and edits complete palettes, including per-index shades/series and independent
light/dark overrides. Save local previews to try them across the showcase, copy
compilable Rust definitions, or reopen a saved theme for editing. The palette
contains 34 semantic color roles, ten shades and eight series colors; optional
array overrides use `[Option<Color>; 10]` / `[Option<Color>; 8]`. See
[theme definitions and audit](docs/theme-definitions.md#theme-studio-and-color-audit).


The public showcase uses Rust client routing with shareable URLs and browser
back/forward support. Theme Studio fills the viewport and retains one unsaved
workspace while browsing or changing the site's palette/scheme. Saved preview
payloads from the earlier schema are backed up and upgraded on load.

The [WASM size baseline](docs/wasm-size.md) compares unused features and actual widget use against Glory, with reproducible builds and browser checks.
