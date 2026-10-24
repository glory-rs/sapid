use glory::prelude::*;
use glory::render::{DragPhase, EventKind, NodeId, RangeCommand, SliderValue};
use sapid::{angle_slider, field};
use std::{cell::RefCell, rc::Rc};

fn state(app: &App, id: NodeId) -> SliderValue {
    app.tree()
        .node(id)
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
fn key(app: &App, id: NodeId, command: RangeCommand) {
    app.dispatch(id, Event::Range { from: id, command })
        .unwrap();
}
#[test]
fn endpoints_remain_distinct_and_keys_wrap_only_after_crossing_them() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    app.mount(
        angle_slider("angle")
            .default_value(359.)
            .node_ref(reference),
    )
    .unwrap();
    let id = reference.get().unwrap();
    for (command, expected) in [
        (RangeCommand::Increase, 360.),
        (RangeCommand::Increase, 1.),
        (RangeCommand::Decrease, 0.),
        (RangeCommand::Decrease, 359.),
        (RangeCommand::PageIncrease, 14.),
        (RangeCommand::Maximum, 360.),
        (RangeCommand::Minimum, 0.),
    ] {
        key(&app, id, command);
        assert_eq!(state(&app, id).range.value, expected);
        assert_eq!(
            state(&app, id).value_text,
            Some(format!("{expected} degrees"))
        );
    }
}
#[test]
fn nondivisor_steps_keep_rotational_progress_without_resnapping_to_the_old_value() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    app.mount(angle_slider("angle").step(270.).node_ref(reference))
        .unwrap();
    let id = reference.get().unwrap();
    for expected in [270., 180., 90., 360., 270.] {
        key(&app, id, RangeCommand::Increase);
        assert_eq!(state(&app, id).range.value, expected);
    }
}
#[test]
fn controlled_commit_only_acceptance_accumulates_wrapped_keys_in_one_batch() {
    let app = App::memory();
    let value = app.scope().cage(359.);
    let reference = app.scope().cage(None);
    app.mount(
        angle_slider("angle")
            .value(value)
            .node_ref(reference)
            .on_value_commit(move |next| value.revise(|v| *v = next)),
    )
    .unwrap();
    let id = reference.get().unwrap();
    app.scope()
        .batch(|| {
            key(&app, id, RangeCommand::Increase);
            key(&app, id, RangeCommand::Increase);
        })
        .unwrap();
    assert_eq!(value.get(), 1.);
    assert_eq!(state(&app, id).range.value, 1.);
}
#[test]
fn controlled_rejection_and_invalid_sources_keep_the_accepted_angle_and_typed_order() {
    let app = App::memory();
    let value = app.scope().cage(359.);
    let reference = app.scope().cage(None);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let changed = calls.clone();
    let committed = calls.clone();
    let observer = calls.clone();
    app.mount(
        angle_slider("angle")
            .value(value)
            .node_ref(reference)
            .on_value_change(move |v| changed.borrow_mut().push(format!("change:{v}")))
            .on_value_commit(move |v| committed.borrow_mut().push(format!("commit:{v}")))
            .on(EventKind::Range, move |_| {
                observer.borrow_mut().push("observer".into())
            }),
    )
    .unwrap();
    let id = reference.get().unwrap();
    key(&app, id, RangeCommand::Increase);
    assert_eq!(&*calls.borrow(), &["change:360", "commit:360", "observer"]);
    assert_eq!(value.get(), 359.);
    value.revise(|v| *v = 361.);
    assert!(app.flush().is_err());
    assert_eq!(state(&app, id).range.value, 359.);
    value.revise(|v| *v = 90.);
    app.flush().unwrap();
    assert_eq!(state(&app, id).range.value, 90.);
}
#[test]
fn field_relations_readonly_and_explicit_value_text_use_the_actual_slider() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    let readonly = app.scope().cage(true);
    app.mount(
        field("heading")
            .label("Heading")
            .description("Degrees clockwise")
            .form_name("heading")
            .readonly(readonly)
            .required(true)
            .control(move |context| {
                angle_slider("heading")
                    .field(context)
                    .default_value(90.)
                    .value_text(None)
                    .node_ref(reference)
            }),
    )
    .unwrap();
    let id = reference.get().unwrap();
    let tree = app.tree();
    let semantics = tree
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap();
    assert!(semantics.labelled_by.is_some() && semantics.described_by.is_some());
    assert!(semantics.required && semantics.readonly);
    assert_eq!(
        semantics.form_control.as_ref().unwrap().value.as_deref(),
        Some("90")
    );
    assert_eq!(state(&app, id).value_text, None);
    drop(tree);
    assert!(
        !app.dispatch(
            id,
            Event::Range {
                from: id,
                command: RangeCommand::Increase
            }
        )
        .unwrap()
    );
    readonly.revise(|v| *v = false);
    app.flush().unwrap();
    key(&app, id, RangeCommand::Increase);
    assert_eq!(state(&app, id).range.value, 91.);
}
#[test]
fn invalid_initial_angle_steps_or_value_text_create_no_nodes() {
    for widget in [
        angle_slider("bad").default_value(-1.),
        angle_slider("bad").default_value(361.),
        angle_slider("bad").step(0.),
        angle_slider("bad").step(360.),
        angle_slider("bad").page_step(360.),
        angle_slider("bad").default_value(f64::NAN),
        angle_slider("bad").value_text(Some("x".repeat(1_000_000))),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert!(app.tree().is_empty());
    }
}
#[test]
fn owner_retirement_cancels_pending_geometry_without_a_late_callback() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    let changes = app.scope().cage(0);
    app.mount(
        angle_slider("angle")
            .node_ref(reference)
            .on_value_change(move |_| changes.revise(|v| *v += 1)),
    )
    .unwrap();
    let id = reference.get().unwrap();
    app.dispatch(
        id,
        Event::Drag {
            from: id,
            phase: DragPhase::Start,
            x: 50.,
            y: 0.,
        },
    )
    .unwrap();
    assert_eq!(app.stats().queries, 1);
    app.clear().unwrap();
    assert_eq!(app.stats().queries, 0);
    assert_eq!(reference.get(), None);
    app.pump().unwrap();
    assert_eq!(changes.get(), 0);
}
#[test]
fn ssr_has_a_real_slider_and_three_bounded_decorative_graphics() {
    let html = glory::ssr::render(angle_slider("angle").default_value(360.)).unwrap();
    assert!(html.contains("role=\"slider\""));
    assert!(html.contains("aria-valuemax=\"360\""));
    assert!(html.contains("aria-valuenow=\"360\""));
    assert!(html.contains("aria-valuetext=\"360 degrees\""));
    assert_eq!(html.matches("<svg").count(), 3);
    assert!(!html.contains("role=\"separator\""));
}
