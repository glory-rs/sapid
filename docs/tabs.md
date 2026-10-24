# Tabs

`tabs(namespace)` uses real TabList, Tab and TabPanel semantics. It supports
automatic activation after observed focus and manual activation on Enter/Space.
`orientation(Orientation::{Horizontal,Vertical})`, `direction(Direction::{Ltr,Rtl})`
and `loop_focus(false)` configure the shared keyboard navigation protocol.
Labels are reactive `Lotus<String>` values, including localized Unicode labels;
stable ASCII keys are independent from those labels.

```rust
use glory::render::{Direction, Orientation};
use sapid::{ContentPolicy, TabActivation, tab, tabs};

// `keys` is a scope-owned Cage<Vec<String>>; `selected` is Cage<String>.
let chapters = tabs("chapters")
    .name("Document chapters")
    .direction(Direction::Rtl)
    .orientation(Orientation::Horizontal)
    .activation(TabActivation::Manual)
    .content_policy(ContentPolicy::KeepMounted)
    .value(selected)
    .on_value_change(move |next| selected.revise(|value| *value = next))
    .tabs(keys, |key| {
        tab(key)
            .label(format!("Chapter {key}"))
            .trigger_content(|| glory::ui::text().content("Chapter summary"))
            .content(|| glory::ui::input().value("Editable draft"))
    });
```

`tabs(key_source, factory)` accepts at most 256 current unique keys, each a valid
ASCII semantic token of at most 64 bytes. The source is validated before cloning
or calling factories. A factory runs once for each newly admitted key under a
short Owner. Reordering retained keys preserves their configuration, trigger,
panel, editor and child Owners. Removing a key retires them; reinsertion creates
a fresh Owner. The implementation retains current configurations and
registrations, with no visited-key history. Static `.tab(...)` and dynamic
`.tabs(...)` cannot be mixed.

The trigger and panel renderers share each key's configuration. A label contains
1–256 bytes and can change without recreating the trigger or its rich text.
`trigger_content` fills the existing semantic Tab button while `label` remains
its explicit accessible name. Nested interactive trigger children are rejected
by the shared content rules. `content` is an independent body factory.

Dynamic controlled values may refer to an absent key: the source remains
unchanged and no current panel is selected. Reinsertion restores that intent
silently. Empty values also mean no controlled selection. Local values instead
repair a removed selection to the first enabled current key, falling back to
the first key when all are disabled. An empty member set is valid. Repairs,
external value changes, label changes and reordering emit no value proposal.
Invalid source updates preserve the last accepted state and report an error.

Root and item enabled states combine with AND. Current order and enabled state
determine roving navigation; all disabled means no tab stop. Navigation queues a
focus intent, and automatic selection waits for actual host focus. When the
actually focused member retires, the component hands focus to the selected live
enabled trigger or first enabled trigger. That internal repair preserves an
absent controlled value. Its one current suppression marker is bounded by an
Owner task and `Query::Focus` confirmation; accepted focus consumes it, refusal
expires it, and other focus, target retirement or Owner retirement cancels it.
A queued focus request is never treated as proof of actual focus. A renderer
without host task execution does not install this automatic-activation marker.

`KeepMounted` retains hidden body state; `Unmount` retires inactive child Owners
while preserving panel shells and label/control relationships. Internal behavior
and typed value proposals run before generic activation observers. Proposals do
not mutate controlled sources. Four themes, explicit class/style overrides and
`unstyled` follow the common Sapid builder contract; tab-list flex layout remains
structural.

Evidence:

- `tests/tabs_dynamic.rs`: ten tests cover retained identities, bounded
  factories and invalid sources, silent controlled absence/local repair,
  localized roles/direction, activation order, refused/accepted focus confirmation,
  query/task cancellation and deterministic four-theme SSR.
- Existing `tests/composites.rs` (18), `tests/composite_dynamic.rs` (8) and
  `tests/collection.rs` (4) remain green. Logs:
  `D:/BuildCache/sapid/tabs-dynamic-rust.log` and
  `D:/BuildCache/sapid/tabs-dynamic-clippy.log`.
- `examples/native/tests/tabs_dynamic.rs` loops four themes and checks actual
  native layout, retained engine editor identity, removal/reinsertion, navigation
  focus and silent controlled-absent focus repair. Passed log:
  `D:/BuildCache/sapid/tabs-dynamic-native.log`.
- `examples/web/tabs-dynamic-acceptance.js`: all eight theme × CSR/hydration
  cases and the physical-focus-refusal case passed on Chromium
  `148.0.7778.96`, using the same-source `menubar-next` WASM/SSR snapshot.
  Real Chrome AX Tab/TabList/TabPanel names, controlled-absent focus repair,
  subsequent user focus, retained editors, RTL, manual activation, empty/restore,
  cleanup and 390px layout passed. Full actual report:
  `D:/BuildCache/sapid/tabs-dynamic-browser-menubar-next.json` (`errors: []`).
  The green-dark hydration screenshot was inspected:
  `D:/BuildCache/sapid/tabs-dynamic-green-dark-hydrate.png`.

The native standalone renderer still has no input driver; `WindowApp` supplies
the queued focus/navigation protocol. These tests do not claim OS screen-reader
announcements or automatic application-wide locale selection.
