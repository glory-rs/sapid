# Accordion

`accordion(namespace).items(keys, factory)` mounts a bounded dynamic keyed
sequence. `keys` is `Lotus<Vec<String>>`; `factory` returns an `AccordionItem`
with the same key. Static `.item(...)` remains available and cannot be mixed
with `.items(...)`. A sequence may be empty and contains at most 256 unique
ASCII semantic keys of at most 64 bytes. Keys validate before cloning or calling
configuration factories. Static builders retain at most 256 entries and one
rejection sentinel; `default_open(iter)` consumes at most 257 values.

The configuration factory runs once for each newly admitted key under a short
Owner. Reordering a surviving key retains that Owner, trigger, panel, body and
editing state. Removing it retires those nodes and captures; reinsertion creates
a fresh identity. Only current configurations and mounted Collection entries
remain. Factory execution and capture destruction occur outside graph and
registry borrows. A failed new configuration preserves the last accepted
collection, and reentrant source writes schedule another reconciliation.

`AccordionItem::trigger_content(factory)` supplies rich content inside the real
Button. Its `label` remains the accessible name, controls/expanded relationship
and heading semantics stay on the existing nodes, and nested interactive
children are rejected by the shared content rules. `content(factory)` is the
independent panel body. KeepMounted retains its body while hidden; Unmount
recreates body Owners on reopening. Body factories therefore follow that policy,
independently from the once-per-admission configuration factory.

`open(source)` remains read-only controlled state and callbacks are proposals.
A controlled key temporarily absent from the collection remains business intent:
it displays no open panel and resumes when that key is reinserted, without a
callback or source write. Local state drops removed keys; a required local
accordion silently chooses the first enabled remaining key (or first remaining
key if all are disabled). Membership changes and external source writes are
silent. Internal acceptance precedes typed and generic observers.

Collection traversal reads current explicit row indices, so Arrow/Home/End
navigation follows reordered membership and skips disabled controls. It changes
focus without opening panels. `direction(Direction)` publishes the shared
direction metadata; labels and rich/body content can use live localized sources.
Root enabled state combines with each item's enabled state. A removed member
that actually held focus hands focus to a live enabled trigger after keyed
reconciliation; outside focus is not inferred from a previous focus intent.
No available remaining control means no invented focus destination.

Four themes, size, root class/style overrides, unstyled, named regions, heading
level, looping and content policies retain their existing behavior. Trigger and
panel references clear with their Owners. The same `trigger_content` capability
is available on static Tab and Collapsible triggers; optional Collapsible height
motion remains a separate unfinished feature.

Focused evidence is in `D:/BuildCache/sapid/composite-dynamic-rust.log`: eight
tests cover bounded iterators and capture retirement, current ordered Collection,
dynamic reorder/editor identity, repeated Owner/graph return to baseline,
configuration failure/reentrancy, local/controlled selection repair, rich trigger
names/relations/content policies and deterministic four-theme SSR.
`composite-dynamic-clippy.log` records strict Clippy. Existing composite and
Collection regressions also pass. Native four-theme layout/identity and actual
focus queries are in `examples/native/tests/accordion_dynamic.rs`; focused
browser coverage is `examples/web/accordion-dynamic-acceptance.js`.
`D:/BuildCache/sapid/accordion-dynamic-browser-root17.json` records eight passing
four-theme CSR/hydration scenarios against unified Web17 on Chromium
148.0.7778.96, with no errors. They verify real Chrome Button/Region AX,
localization, callback ordering, disabled/current-order keyboard navigation,
retained editor/panel/trigger identities, once-per-admission factories,
controlled refusal/absent intent, focused removal handoff, local repair,
Unmount, mobile and cleanup. Four `accordion-{hue}-{scheme}-hydrate.png`
screenshots record the fixture; green/dark was visually inspected. Native
`accordion-dynamic-native.log` also proves that later membership removal does not
steal focus after actual focus has moved outside. These are layout and AX-tree
checks, not spoken screen-reader acceptance.
