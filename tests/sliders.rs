use glory::prelude::*;
use glory::render::{DragPhase, EventKind, NodeId, RangeCommand};
use sapid::{field, slider};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn range(app: &App, node: NodeId) -> glory::render::SliderValue {
    app.tree()
        .node(node)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .slider
        .clone()
        .unwrap()
}
fn key(app: &App, node: NodeId, command: RangeCommand) {
    app.dispatch(
        node,
        Event::Range {
            from: node,
            command,
        },
    )
    .unwrap();
}
#[test]
fn local_range_steps_pages_endpoints_and_same_tick_intents_accumulate() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    app.mount(
        slider("local")
            .default_value(30.0)
            .min(20.0)
            .max(80.0)
            .step(0.5)
            .page_step(5.0)
            .node_ref(reference),
    )
    .unwrap();
    let node = reference.get().unwrap();
    app.scope()
        .batch(|| {
            for _ in 0..3 {
                key(&app, node, RangeCommand::Increase)
            }
        })
        .unwrap();
    assert_eq!(range(&app, node).range.value, 31.5);
    key(&app, node, RangeCommand::PageIncrease);
    assert_eq!(range(&app, node).range.value, 36.5);
    key(&app, node, RangeCommand::Maximum);
    assert_eq!(range(&app, node).range.value, 80.0);
    key(&app, node, RangeCommand::Increase);
    assert_eq!(range(&app, node).range.value, 80.0);
    key(&app, node, RangeCommand::Minimum);
    assert_eq!(range(&app, node).range.value, 20.0);
}
#[test]
fn controlled_rejection_preserves_ui_and_callbacks_precede_observers() {
    let app = App::memory();
    let value = app.scope().cage(25.0);
    let reference = app.scope().cage(None);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let changed = calls.clone();
    let commit_calls = calls.clone();
    let observed = calls.clone();
    app.mount(
        slider("controlled")
            .value(value)
            .node_ref(reference)
            .on_value_change(move |value| changed.borrow_mut().push(format!("change:{value}")))
            .on_value_commit(move |value| commit_calls.borrow_mut().push(format!("commit:{value}")))
            .on(EventKind::Range, move |_| {
                observed.borrow_mut().push("observer".into())
            }),
    )
    .unwrap();
    let node = reference.get().unwrap();
    key(&app, node, RangeCommand::Increase);
    assert_eq!(value.get(), 25.0);
    assert_eq!(range(&app, node).range.value, 25.0);
    assert_eq!(&*calls.borrow(), &["change:26", "commit:26", "observer"]);
    value.revise(|value| *value = 40.0);
    app.flush().unwrap();
    assert_eq!(range(&app, node).range.value, 40.0);
    assert_eq!(calls.borrow().len(), 3);
}
#[test]
fn accepting_controlled_callbacks_accumulate_in_one_batch_and_pending_external_values_stay_unaccepted()
 {
    let app = App::memory();
    let value = app.scope().cage(10.0);
    let reference = app.scope().cage(None);
    app.mount(
        slider("accepted")
            .value(value)
            .node_ref(reference)
            .on_value_change(move |next| value.revise(|value| *value = next)),
    )
    .unwrap();
    let node = reference.get().unwrap();
    app.scope()
        .batch(|| {
            key(&app, node, RangeCommand::Increase);
            key(&app, node, RangeCommand::Increase)
        })
        .unwrap();
    assert_eq!(value.get(), 12.0);
    value.revise(|value| *value = 90.0);
    key(&app, node, RangeCommand::Increase);
    assert_eq!(value.get(), 13.0);
}
#[test]
fn field_relations_readonly_disabled_and_successful_value_use_real_slider_metadata() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    let readonly = app.scope().cage(true);
    let enabled = app.scope().cage(true);
    app.mount(
        field("amount")
            .label("Amount")
            .description("Choose units")
            .form_name("amount")
            .required(true)
            .readonly(readonly)
            .enabled(enabled)
            .control(move |context| {
                slider("amount-range")
                    .field(context)
                    .node_ref(reference)
                    .default_value(35.0)
            }),
    )
    .unwrap();
    let node = reference.get().unwrap();
    let semantics = app
        .tree()
        .node(node)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .as_ref()
        .clone();
    assert!(semantics.labelled_by.is_some());
    assert!(semantics.described_by.is_some());
    assert!(semantics.required && semantics.readonly);
    assert_eq!(semantics.form_control.unwrap().value.as_deref(), Some("35"));
    assert!(
        !app.dispatch(
            node,
            Event::Range {
                from: node,
                command: RangeCommand::Increase
            }
        )
        .unwrap()
    );
    assert_eq!(range(&app, node).range.value, 35.0);
    readonly.revise(|value| *value = false);
    app.flush().unwrap();
    key(&app, node, RangeCommand::Increase);
    assert_eq!(range(&app, node).range.value, 36.0);
    enabled.revise(|value| *value = false);
    app.flush().unwrap();
    assert!(
        app.dispatch(
            node,
            Event::Range {
                from: node,
                command: RangeCommand::Increase
            }
        )
        .is_err()
    );
    assert_eq!(
        app.tree().node(node).unwrap().descriptor.view().focus,
        glory::render::FocusMode::Programmatic
    );
}
#[test]
fn invalid_dynamic_configuration_retains_accepted_range_and_requests_use_it() {
    let app = App::memory();
    let value = app.scope().cage(30.0);
    let step = app.scope().cage(2.0);
    let min = app.scope().cage(0.0);
    let reference = app.scope().cage(None);
    let proposed = Rc::new(Cell::new(0.0));
    let output = proposed.clone();
    app.mount(
        slider("invalid")
            .value(value)
            .step(step)
            .min(min)
            .node_ref(reference)
            .on_value_change(move |value| output.set(value)),
    )
    .unwrap();
    let node = reference.get().unwrap();
    step.revise(|step| *step = f64::NAN);
    assert!(app.flush().is_err());
    key(&app, node, RangeCommand::Increase);
    assert_eq!(proposed.get(), 32.0);
    assert_eq!(range(&app, node).step, 2.0);
    step.revise(|step| *step = 1.0);
    min.revise(|min| *min = 50.0);
    assert!(app.flush().is_err());
    assert_eq!(range(&app, node).range.min, 0.0);
    min.revise(|min| *min = 0.0);
    app.flush().unwrap();
    assert_eq!(range(&app, node).step, 1.0);
}
#[test]
fn invalid_initial_bounds_steps_values_and_large_value_text_create_no_nodes() {
    for widget in [
        slider("bad").min(2.0).max(1.0),
        slider("bad").step(0.0),
        slider("bad").default_value(f64::NAN),
        slider("bad").max(1e13),
        slider("bad").value_text(Some("x".repeat(1_000_000))),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert!(app.tree().roots().is_empty());
    }
}
#[test]
fn a_pending_geometry_request_is_canceled_on_invalid_source_or_owner_retirement() {
    let app = App::memory();
    let value = app.scope().cage(20.0);
    let reference = app.scope().cage(None);
    app.mount(slider("pending").value(value).node_ref(reference))
        .unwrap();
    let node = reference.get().unwrap();
    app.dispatch(
        node,
        Event::Drag {
            from: node,
            phase: DragPhase::Start,
            x: 10.0,
            y: 10.0,
        },
    )
    .unwrap();
    assert_eq!(app.stats().queries, 1);
    value.revise(|value| *value = f64::NAN);
    assert!(app.flush().is_err());
    assert_eq!(app.stats().queries, 0);
    app.pump().unwrap();
    assert_eq!(range(&app, node).range.value, 20.0);
    value.revise(|value| *value = 20.0);
    app.flush().unwrap();
    app.dispatch(
        node,
        Event::Drag {
            from: node,
            phase: DragPhase::Start,
            x: 10.0,
            y: 10.0,
        },
    )
    .unwrap();
    assert_eq!(app.stats().queries, 1);
    app.clear().unwrap();
    assert_eq!(app.stats().queries, 0);
    assert_eq!(reference.get(), None);
    app.pump().unwrap();
}
#[test]
fn ssr_exports_a_custom_slider_with_explicit_snapshot_metadata_and_no_separator_role() {
    let html = glory::ssr::render(
        field("amount")
            .label("Amount")
            .form_name("amount")
            .readonly(true)
            .control(|context| {
                slider("amount")
                    .field(context)
                    .default_value(25.0)
                    .value_text(Some("Twenty five units".into()))
            }),
    )
    .unwrap();
    assert!(html.contains("role=\"slider\""));
    assert!(html.contains("aria-valuenow=\"25\""));
    assert!(html.contains("aria-valuetext=\"Twenty five units\""));
    assert!(html.contains("data-gly-control-name=\"amount\""));
    assert!(!html.contains("role=\"separator\""));
    assert!(!html.contains("type=\"range\""));
}

#[test]
fn rejected_cursor_cannot_be_confirmed_by_later_external_writes_of_the_same_number() {
    let app = App::memory();
    let value = app.scope().cage(25.);
    let requested = app.scope().cage(0.);
    let reference = app.scope().cage(None);
    app.mount(
        slider("cursor")
            .value(value)
            .node_ref(reference)
            .on_value_change(move |v| requested.revise(|old| *old = v)),
    )
    .unwrap();
    let id = reference.get().unwrap();
    key(&app, id, RangeCommand::Increase);
    assert_eq!(requested.get(), 26.);
    value.revise(|v| *v = 26.);
    key(&app, id, RangeCommand::Increase);
    assert_eq!(requested.get(), 26.);
    assert_eq!(range(&app, id).range.value, 26.);
}
#[test]
fn accepted_cursor_is_invalidated_even_by_same_value_external_writes_before_the_next_event() {
    let app = App::memory();
    let value = app.scope().cage(25.);
    let requested = app.scope().cage(0.);
    let reference = app.scope().cage(None);
    app.mount(
        slider("cursor")
            .value(value)
            .node_ref(reference)
            .on_value_change(move |v| {
                requested.revise(|old| *old = v);
                value.revise(|old| *old = v)
            }),
    )
    .unwrap();
    let id = reference.get().unwrap();
    app.scope()
        .batch(|| {
            key(&app, id, RangeCommand::Increase);
            value.revise(|v| *v = 26.);
            key(&app, id, RangeCommand::Increase)
        })
        .unwrap();
    assert_eq!(requested.get(), 26.);
    assert_eq!(value.get(), 26.);
}

#[test]
fn commit_callback_acceptance_and_same_value_rewrites_accumulate_keyboard_intents() {
    for change_accepts in [false, true] {
        let app = App::memory();
        let value = app.scope().cage(25.);
        let reference = app.scope().cage(None);
        app.mount(
            slider("commit-cursor")
                .value(value)
                .node_ref(reference)
                .on_value_change(move |next| {
                    if change_accepts {
                        value.revise(|old| *old = next)
                    }
                })
                .on_value_commit(move |next| value.revise(|old| *old = next)),
        )
        .unwrap();
        let id = reference.get().unwrap();
        app.scope()
            .batch(|| {
                key(&app, id, RangeCommand::Increase);
                key(&app, id, RangeCommand::Increase)
            })
            .unwrap();
        assert_eq!(value.get(), 27.);
        assert_eq!(range(&app, id).range.value, 27.);
    }
}
