use glory::prelude::*;
use glory::render::{Current, EventKind, NodeId, Props};
use sapid::{ContentPolicy, StepStatus, step, stepper};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn nodes(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut result = Vec::new();
    while let Some(id) = pending.pop() {
        pending.extend(tree.node(id).unwrap().children.iter().copied());
        result.push(id)
    }
    result
}
fn button(app: &App, label: &str) -> NodeId {
    nodes(app).into_iter().find(|id|matches!(&app.tree().node(*id).unwrap().descriptor.view().props.clone(),Props::Button(p) if p.label==label)).unwrap_or_else(||panic!("missing {label}"))
}
fn active(app: &App) -> Option<NodeId> {
    nodes(app).into_iter().find(|id| {
        app.tree()
            .node(*id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .is_some_and(|s| s.current == Some(Current::Step))
    })
}
fn steps() -> Vec<sapid::Step> {
    vec![
        step("identity").label("Identity"),
        step("address").label("Address"),
        step("review").label("Review"),
    ]
}
#[test]
fn default_headers_are_nonselectable_and_current_step_is_independent_of_tabs() {
    let app = App::memory();
    app.mount(stepper("setup").steps(steps())).unwrap();
    assert_eq!(active(&app), Some(button(&app, "1. Identity")));
    for label in ["1. Identity", "2. Address", "3. Review"] {
        assert!(!app.dispatch(button(&app, label), Event::Activate).unwrap());
    }
    for id in nodes(&app) {
        if let Some(s) = &app.tree().node(id).unwrap().descriptor.view().semantics {
            assert!(!s.tab);
            assert!(s.selected.is_none());
        }
    }
}
#[test]
fn local_state_supports_back_next_and_explicit_skipping_before_observers() {
    let app = App::memory();
    let trace = Rc::new(RefCell::new(Vec::new()));
    let a = trace.clone();
    let b = trace.clone();
    let skip = app.scope().cage(false);
    app.mount(
        stepper("setup")
            .steps(steps())
            .selectable(true)
            .allow_skip(skip)
            .on_step_change(move |index| a.borrow_mut().push(index.to_string()))
            .on(EventKind::Activate, move |_| {
                b.borrow_mut().push("observed".into())
            }),
    )
    .unwrap();
    assert!(
        !app.dispatch(button(&app, "3. Review"), Event::Activate)
            .unwrap()
    );
    app.dispatch(button(&app, "2. Address"), Event::Activate)
        .unwrap();
    assert_eq!(active(&app), Some(button(&app, "2. Address")));
    assert_eq!(*trace.borrow(), vec!["1", "observed"]);
    app.dispatch(button(&app, "1. Identity"), Event::Activate)
        .unwrap();
    skip.revise(|v| *v = true);
    app.flush().unwrap();
    app.dispatch(button(&app, "3. Review"), Event::Activate)
        .unwrap();
    assert_eq!(active(&app), Some(button(&app, "3. Review")));
}
#[test]
fn controlled_rejection_defaults_and_external_updates_are_silent() {
    let app = App::memory();
    let value = app.scope().cage(0);
    let calls = Rc::new(Cell::new(0));
    let out = calls.clone();
    app.mount(
        stepper("setup")
            .active(value)
            .default_active(2)
            .steps(steps())
            .selectable(true)
            .on_step_change(move |index| {
                assert_eq!(index, 1);
                out.set(out.get() + 1)
            }),
    )
    .unwrap();
    let ids = nodes(&app);
    app.dispatch(button(&app, "2. Address"), Event::Activate)
        .unwrap();
    assert_eq!(value.get(), 0);
    assert_eq!(calls.get(), 1);
    value.revise(|v| *v = 2);
    app.flush().unwrap();
    assert_eq!(nodes(&app), ids);
    assert_eq!(active(&app), Some(button(&app, "3. Review")));
    assert_eq!(calls.get(), 1);
}
#[test]
fn rejected_and_pending_sources_do_not_change_the_accepted_jump_bounds() {
    let app = App::memory();
    let value = app.scope().cage(0);
    let calls = Rc::new(Cell::new(0));
    let out = calls.clone();
    app.mount(
        stepper("setup")
            .active(value)
            .steps(steps())
            .selectable(true)
            .on_step_change(move |index| out.set(index)),
    )
    .unwrap();
    value.revise(|v| *v = 99);
    assert!(app.flush().is_err());
    assert!(
        !app.dispatch(button(&app, "3. Review"), Event::Activate)
            .unwrap()
    );
    app.dispatch(button(&app, "2. Address"), Event::Activate)
        .unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(active(&app), Some(button(&app, "1. Identity")));
    value.revise(|v| *v = 2);
    assert!(
        !app.dispatch(button(&app, "3. Review"), Event::Activate)
            .unwrap()
    );
}
#[test]
fn per_step_and_root_disabling_affect_real_header_controls() {
    let app = App::memory();
    let enabled = app.scope().cage(true);
    app.mount(
        stepper("setup")
            .enabled(enabled)
            .selectable(true)
            .allow_skip(true)
            .steps([step("one"), step("two").enabled(false), step("three")]),
    )
    .unwrap();
    assert!(
        !app.dispatch(button(&app, "2. two"), Event::Activate)
            .unwrap()
    );
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(
        !app.dispatch(button(&app, "3. three"), Event::Activate)
            .unwrap()
    );
}
#[test]
fn explicit_status_overrides_derived_completion_and_exposes_descriptive_error_text() {
    let app = App::memory();
    let status = app.scope().cage(Some(StepStatus::Error));
    app.mount(stepper("setup").default_active(1).steps([
        step("one").status(status).description("Email rejected"),
        step("two").optional(true),
    ]))
    .unwrap();
    let captions = || {
        nodes(&app)
            .into_iter()
            .filter_map(
                |id| match &app.tree().node(id).unwrap().descriptor.view().props {
                    Props::Text(p) => Some(p.content.clone()),
                    _ => None,
                },
            )
            .collect::<Vec<_>>()
    };
    assert!(
        captions()
            .iter()
            .any(|text| text.contains("Error · Email rejected"))
    );
    assert!(captions().iter().any(|text| text.contains("Optional")));
    status.revise(|v| *v = None);
    app.flush().unwrap();
    assert!(
        captions()
            .iter()
            .any(|text| text.contains("Completed · Email rejected"))
    );
}
#[test]
fn completion_stage_is_explicit_and_has_no_current_header() {
    let app = App::memory();
    let value = app.scope().cage(0);
    app.mount(
        stepper("setup")
            .active(value)
            .steps(steps())
            .completed(|| glory::ui::text().content("Workflow saved")),
    )
    .unwrap();
    let ids = nodes(&app);
    value.revise(|v| *v = 3);
    app.flush().unwrap();
    assert!(active(&app).is_none());
    assert_eq!(ids, nodes(&app));
    value.revise(|v| *v = 0);
    app.flush().unwrap();
    assert_eq!(active(&app), Some(button(&app, "1. Identity")));
}
#[test]
fn content_policy_keeps_state_or_retires_owners_explicitly() {
    for (policy, created_at_close, dropped_at_close) in [
        (ContentPolicy::KeepMounted, 2, 0),
        (ContentPolicy::Unmount, 1, 1),
    ] {
        let app = App::memory();
        let value = app.scope().cage(0);
        let created = Rc::new(Cell::new(0));
        let drops = Rc::new(Cell::new(0));
        let a = created.clone();
        let b = drops.clone();
        let make = move || {
            let a = a.clone();
            let b = b.clone();
            glory::ui::component(move |scope| {
                a.set(a.get() + 1);
                let b = b.clone();
                scope.on_cleanup(move || b.set(b.get() + 1))?;
                glory::ui::input().value("draft").show_in(scope)
            })
        };
        let first = make.clone();
        app.mount(
            stepper("setup")
                .active(value)
                .content_policy(policy)
                .steps([step("one").content(first), step("two").content(make)]),
        )
        .unwrap();
        value.revise(|v| *v = 1);
        app.flush().unwrap();
        assert_eq!(
            created.get(),
            if policy == ContentPolicy::Unmount {
                2
            } else {
                created_at_close
            }
        );
        assert_eq!(drops.get(), dropped_at_close);
        app.clear().unwrap();
        assert_eq!(created.get(), drops.get());
    }
}
#[test]
fn panel_refs_are_programmatic_focus_targets_and_reset_on_retirement() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    app.mount(stepper("setup").step(step("one").panel_ref(reference)))
        .unwrap();
    let panel = reference.get().unwrap();
    assert_eq!(
        app.tree().node(panel).unwrap().descriptor.view().focus,
        glory::render::FocusMode::Programmatic
    );
    app.clear().unwrap();
    assert!(reference.get().is_none());
}
#[test]
fn initial_invalid_values_duplicate_keys_and_constructor_overflow_create_no_nodes() {
    let app = App::memory();
    assert!(app.mount(stepper("empty")).is_err());
    assert!(
        app.mount(stepper("invalid").default_active(2).step(step("one")))
            .is_err()
    );
    assert!(
        app.mount(stepper("dupe").steps([step("one"), step("one")]))
            .is_err()
    );
    let count = Rc::new(Cell::new(0));
    let out = count.clone();
    let oversized = stepper("bounded").steps((0..1_000_000).map(move |i| {
        out.set(out.get() + 1);
        step(format!("step{i}"))
    }));
    assert_eq!(count.get(), 129);
    assert!(app.mount(oversized).is_err());
    assert!(app.tree().roots().is_empty());
}
#[test]
fn ssr_uses_real_list_buttons_current_step_relations_and_no_tab_roles() {
    let html = glory::ssr::render(stepper("setup").steps(steps()).selectable(true)).unwrap();
    assert_eq!(html.matches("aria-current=\"step\"").count(), 1);
    assert!(html.contains("role=\"list\""));
    assert_eq!(html.matches("role=\"listitem\"").count(), 3);
    assert!(html.contains("aria-describedby="));
    assert!(!html.contains("aria-selected"));
    assert!(!html.contains("role=\"tab"));
}
#[test]
fn form_validation_is_an_explicit_application_barrier_for_step_requests() {
    use sapid::{FieldOptions, FormState, SubmitStatus};
    let app = App::memory();
    let active = app.scope().cage(0);
    let value = app.scope().cage(String::new());
    let form = FormState::new(app.scope());
    form.register(
        app.scope(),
        "email",
        value,
        FieldOptions::default().validate(|value: &String| {
            if value.contains('@') {
                vec![]
            } else {
                vec!["Email required".into()]
            }
        }),
    )
    .unwrap();
    let submit = form.clone();
    app.mount(
        stepper("form")
            .active(active)
            .selectable(true)
            .steps([step("email"), step("review")])
            .on_step_change(move |next| {
                submit
                    .submit(move |token| {
                        if token.is_current() {
                            active.revise(|v| *v = next);
                            token.complete(Ok(()));
                        }
                    })
                    .unwrap();
            }),
    )
    .unwrap();
    app.dispatch(button(&app, "2. review"), Event::Activate)
        .unwrap();
    assert_eq!(active.get(), 0);
    assert_eq!(form.state().get().submit, SubmitStatus::Invalid);
    value.revise(|v| *v = "alex@example.test".into());
    app.dispatch(button(&app, "2. review"), Event::Activate)
        .unwrap();
    assert_eq!(active.get(), 1);
    assert_eq!(form.state().get().submit, SubmitStatus::Succeeded);
}
#[test]
fn cancelled_async_form_validation_cannot_advance_a_step() {
    use glory::core::executor::Executor;
    use sapid::{FieldOptions, FormState};
    let executor = Rc::new(Executor::new(4));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let active = app.scope().cage(0);
    let value = app.scope().cage("old".to_owned());
    let form = FormState::new(app.scope());
    form.register(
        app.scope(),
        "email",
        value,
        FieldOptions::default().validate_async(|_| Box::pin(async { vec![] })),
    )
    .unwrap();
    let submit = form.clone();
    app.mount(
        stepper("form")
            .active(active)
            .selectable(true)
            .steps([step("email"), step("review")])
            .on_step_change(move |next| {
                submit
                    .submit(move |token| {
                        if token.is_current() {
                            active.revise(|v| *v = next);
                            token.complete(Ok(()));
                        }
                    })
                    .unwrap();
            }),
    )
    .unwrap();
    app.dispatch(button(&app, "2. review"), Event::Activate)
        .unwrap();
    assert_eq!(active.get(), 0);
    value.revise(|v| *v = "new".into());
    assert!(executor.poll_one());
    app.flush().unwrap();
    assert_eq!(active.get(), 0);
    app.dispatch(button(&app, "2. review"), Event::Activate)
        .unwrap();
    assert!(executor.poll_one());
    app.flush().unwrap();
    assert_eq!(active.get(), 1);
}
