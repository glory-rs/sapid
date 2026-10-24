# Stepper

P6-06 provides finite workflow steps with current, optional, completed, and error states. It installs no router, automatic validation, or HTTP transport.

```rust
use sapid::{step, stepper};
stepper("checkout")
    .steps([
        step("account").label("Account"),
        step("details").label("Details").optional(true),
        step("review").label("Review"),
    ])
    .default_active(0);
```

The root is a named Group. Step headers are actual Button controls inside List/ListItem semantics. Only the current header receives `Current::Step` (`aria-current="step"` on HTML and `AriaCurrent::Step` on AccessKit). Progress does not claim tab or selected semantics. Each header describes its ordinal, status, optional marker, and supplied description through a stable text relation. Orientation changes layout; keyboard activation remains native Enter/Space and Tab.

`active` is a zero-based controlled input; `default_active` supplies local state only without it. Exactly the step count represents the explicit completed stage, with no current header. `.completed(factory)` supplies its content and `.completed_label(...)` names it. Initial out-of-range state fails before nodes are created; dynamic invalid values report an error while preserving the accepted step and selection bounds. External source changes are silent. Header requests use the accepted state, including after rejected or pending source writes.

Headers are not selectable by default. `.selectable(true)` opts in to requests for earlier steps or the immediately following step. `.allow_skip(true)` additionally permits jumping beyond the next step. `.enabled(false)` disables all header buttons; each `Step.enabled` can independently disable selection. These policies affect header requests; application controls inside content retain their own enabled policy. `on_step_change(index)` proposes a change before generic activation observers. Controlled sources are never written implicitly. Selecting the current header emits no change callback. `Step.optional` describes the step and does not silently validate or skip it.

`Step.status(None)` derives Completed for steps before active, Pending otherwise. Explicit `Some(StepStatus::{Pending, Completed, Error})` overrides that derivation without changing the active step or form data. Descriptions can provide a meaningful error message and update silently. Error and current location remain independent.

Every step and completion stage has a stable named content shell. `ContentPolicy::KeepMounted` is the default: hidden content preserves local state, input identity, bindings, and tasks. `Unmount` retires child Owners when a stage closes and creates fresh children when it opens. Both policies preserve shell identity and relations. Content shells accept programmatic focus, without entering normal Tab order. If an actual focused panel closes, the widget requests focus on the newly active panel, including completion; it does not focus a disabled header or infer focus from an earlier request.

Keys and namespaces are explicit bounded ASCII tokens. At most 128 static live steps are allowed. Collection construction consumes at most 129 iterator values to reject overflow; repeated `.step(...)` calls retain the same bounded overflow sentinel. There is no source collection scan, visited-step history, or dynamic per-route registry.

## Form integration

Applications decide which registered fields belong to each validation barrier. A controlled step request can call `FormState.submit`, update the active Cage only from its valid completion token, and then complete the token. Do not advance the step before asynchronous validation finishes. Keep form values in application-owned Cages if they must survive `Unmount`; disabled/hidden field validation policy remains explicit in field registration.

```rust,ignore
stepper("account")
    .active(active)
    .selectable(true)
    .steps(steps)
    .on_step_change(move |next| {
        let version = active.try_version().unwrap();
        form.submit(move |token| {
            if token.is_current() && active.try_version().ok() == Some(version) {
                active.revise(|value| *value = next);
                token.complete(Ok(()));
            }
        }).unwrap();
    });
```

The captured active version also protects against an application moving away and returning while validation is pending. The browser fixture connects the first step to a real Form and FormState: sync errors, controlled rejection, cancellable async validation, late-result rejection, and completion all use explicit application callbacks.

## Evidence

`tests/stepper.rs` has 13 passing cases for local/controlled values, skip boundaries, accepted-state rejection, disabled controls, status/error descriptions, completion, finite construction, stable relations, Owner policies, SSR, and synchronous/cancelled asynchronous form barriers. `examples/native/tests/stepper.rs` passes two tests covering four-theme real layout, controlled rejection, actual panel focus, kept engine identities, completion, and repeated Unmount retirement/recreation with the fixed Blitz fork.

Chrome 155.0.8059.84 completed 9 reports with no errors in `D:/BuildCache/sapid/stepper-browser.json`: four-theme CSR/hydration, current/list/description semantics, optional/error/completion states, explicit skip, mouse/Enter/Space, controlled rejection, native disabled headers, sync/async FormState barriers, cancelled late-result rejection, actual panel focus, KeepMounted draft/identity, AX list, dirty hydration with caret/focus preservation, and Owner retirement. Screenshots are `stepper-*.png` in the same directory.
