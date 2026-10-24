use glory::prelude::*;
use glory::render::{DragPhase, Event, EventKind, NodeId, RangeCommand, SliderValue};
use sapid::{RangeThumb, range_slider};
use std::{cell::RefCell, rc::Rc};
fn range(app: &App, id: NodeId) -> SliderValue {
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
fn local_pair_clamps_neighbours_keeps_tab_order_and_unfreezes_singleton_thumb() {
    let app = App::memory();
    let lower = app.scope().cage(None);
    let upper = app.scope().cage(None);
    app.mount(
        range_slider("local")
            .minimum_gap(10.)
            .thumb_ref(RangeThumb::Lower, lower)
            .thumb_ref(RangeThumb::Upper, upper),
    )
    .unwrap();
    let low = lower.get().unwrap();
    let high = upper.get().unwrap();
    assert_eq!(range(&app, low).range.max, 65.);
    key(&app, low, RangeCommand::Maximum);
    assert_eq!(range(&app, low).range.value, 65.);
    assert_eq!(range(&app, high).range.min, 75.);
    key(&app, low, RangeCommand::Minimum);
    key(&app, high, RangeCommand::Minimum);
    assert_eq!(range(&app, low).range.min, 0.);
    assert_eq!(range(&app, low).range.max, 0.);
    assert!(
        app.tree()
            .node(low)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .readonly
    );
    assert_eq!(
        app.tree().node(low).unwrap().descriptor.view().focus,
        glory::render::FocusMode::TabStop
    );
    assert!(
        !app.dispatch(
            low,
            Event::Range {
                from: low,
                command: RangeCommand::Increase
            }
        )
        .unwrap()
    );
    key(&app, high, RangeCommand::Maximum);
    assert!(
        !app.tree()
            .node(low)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .readonly
    );
    assert_eq!(lower.get(), Some(low));
    assert_eq!(upper.get(), Some(high));
}
#[test]
fn controlled_commit_only_two_thumb_events_accumulate_without_overwriting_the_other_value() {
    let app = App::memory();
    let value = app.scope().cage([25., 75.]);
    let lower = app.scope().cage(None);
    let upper = app.scope().cage(None);
    app.mount(
        range_slider("controlled")
            .value(value)
            .minimum_gap(10.)
            .thumb_ref(RangeThumb::Lower, lower)
            .thumb_ref(RangeThumb::Upper, upper)
            .on_value_commit(move |next| value.revise(|old| *old = next)),
    )
    .unwrap();
    app.scope()
        .batch(|| {
            key(&app, lower.get().unwrap(), RangeCommand::Increase);
            key(&app, upper.get().unwrap(), RangeCommand::Decrease)
        })
        .unwrap();
    assert_eq!(value.get(), [26., 74.]);
}
#[test]
fn controlled_rejection_and_invalid_source_use_accepted_bounds_and_typed_order() {
    let app = App::memory();
    let value = app.scope().cage([25., 75.]);
    let lower = app.scope().cage(None);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let changes = calls.clone();
    let committed = calls.clone();
    let observers = calls.clone();
    app.mount(
        range_slider("rejected")
            .value(value)
            .minimum_gap(10.)
            .thumb_ref(RangeThumb::Lower, lower)
            .on_value_change(move |v| changes.borrow_mut().push(format!("change:{v:?}")))
            .on_value_commit(move |v| committed.borrow_mut().push(format!("commit:{v:?}")))
            .on(EventKind::Range, move |_| {
                observers.borrow_mut().push("observer".into())
            }),
    )
    .unwrap();
    let low = lower.get().unwrap();
    key(&app, low, RangeCommand::Increase);
    assert_eq!(value.get(), [25., 75.]);
    assert_eq!(
        &*calls.borrow(),
        &["change:[26.0, 75.0]", "commit:[26.0, 75.0]", "observer"]
    );
    value.revise(|v| *v = [90., 10.]);
    assert!(app.flush().is_err());
    assert_eq!(range(&app, low).range.value, 25.);
    key(&app, low, RangeCommand::Maximum);
    assert_eq!(calls.borrow().get(3).unwrap(), "change:[65.0, 75.0]");
    value.revise(|v| *v = [20., 80.]);
    app.flush().unwrap();
    assert_eq!(range(&app, low).range.value, 20.);
}
#[test]
fn invalid_initial_pairs_gaps_steps_and_unbounded_names_create_no_nodes() {
    for widget in [
        range_slider("bad").default_value([90., 10.]),
        range_slider("bad").minimum_gap(101.),
        range_slider("bad").minimum_gap(-1.),
        range_slider("bad").step(0.),
        range_slider("bad").labels(["x".repeat(1_000_000), "Maximum".into()]),
        range_slider("bad").value_text([Some("x".repeat(1_000_000)), None]),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert!(app.tree().is_empty());
    }
}
#[test]
fn field_uses_two_real_controls_with_explicit_repeated_snapshot_names() {
    let app = App::memory();
    let low = app.scope().cage(None);
    let high = app.scope().cage(None);
    let readonly = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    app.mount(
        sapid::field("price")
            .label("Price")
            .description("Budget")
            .required(true)
            .form_name("price")
            .readonly(readonly)
            .enabled(enabled)
            .control(move |field| {
                range_slider("price")
                    .field(field)
                    .labels(["Minimum price".into(), "Maximum price".into()])
                    .thumb_ref(RangeThumb::Lower, low)
                    .thumb_ref(RangeThumb::Upper, high)
            }),
    )
    .unwrap();
    for (id, expected) in [(low.get().unwrap(), "25"), (high.get().unwrap(), "75")] {
        let tree = app.tree();
        let node = tree.node(id).unwrap();
        let meta = node.descriptor.view().semantics.as_ref().unwrap();
        assert!(meta.required && meta.labelled_by.is_none() && meta.described_by.is_some());
        assert!(
            node.descriptor
                .view()
                .name
                .as_ref()
                .is_some_and(|name| name.ends_with("price"))
        );
        assert_eq!(meta.form_control.as_ref().unwrap().name, "price");
        assert_eq!(
            meta.form_control.as_ref().unwrap().value.as_deref(),
            Some(expected)
        );
    }
    readonly.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(
        !app.dispatch(
            low.get().unwrap(),
            Event::Range {
                from: low.get().unwrap(),
                command: RangeCommand::Increase
            }
        )
        .unwrap()
    );
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(high.get().unwrap())
            .unwrap()
            .descriptor
            .view()
            .focus,
        glory::render::FocusMode::Programmatic
    );
}
#[test]
fn pending_track_geometry_is_canceled_on_domain_changes_and_owner_retirement() {
    let app = App::memory();
    let low = app.scope().cage(None);
    let gap = app.scope().cage(10.);
    app.mount(
        range_slider("requests")
            .minimum_gap(gap)
            .thumb_ref(RangeThumb::Lower, low),
    )
    .unwrap();
    let id = low.get().unwrap();
    app.dispatch(
        id,
        Event::Drag {
            from: id,
            phase: DragPhase::Start,
            x: 10.,
            y: 10.,
        },
    )
    .unwrap();
    assert_eq!(app.stats().queries, 1);
    gap.revise(|v| *v = 20.);
    app.flush().unwrap();
    assert_eq!(app.stats().queries, 0);
    app.pump().unwrap();
    app.dispatch(
        id,
        Event::Drag {
            from: id,
            phase: DragPhase::Start,
            x: 10.,
            y: 10.,
        },
    )
    .unwrap();
    assert_eq!(app.stats().queries, 1);
    app.clear().unwrap();
    assert_eq!(app.stats().queries, 0);
    assert!(low.get().is_none());
    app.pump().unwrap();
}
#[test]
fn ssr_has_two_named_sliders_with_neighbour_limits_and_no_separator() {
    let html = glory::ssr::render(
        range_slider("ssr")
            .minimum_gap(10.)
            .labels(["Lower amount".into(), "Upper amount".into()]),
    )
    .unwrap();
    assert_eq!(html.matches("role=\"slider\"").count(), 2);
    assert!(html.contains("aria-label=\"Lower amount\""));
    assert!(html.contains("aria-valuemax=\"65\""));
    assert!(html.contains("aria-valuemin=\"35\""));
    assert!(!html.contains("role=\"separator\""));
}

#[test]
fn decimal_gap_boundaries_round_inwards_and_never_publish_a_too_small_distance() {
    let app = App::memory();
    let lower = app.scope().cage(None);
    let upper = app.scope().cage(None);
    app.mount(
        range_slider("decimal")
            .min(0.)
            .max(1.)
            .step(0.1)
            .minimum_gap(0.1)
            .default_value([0.2, 0.7])
            .thumb_ref(RangeThumb::Lower, lower)
            .thumb_ref(RangeThumb::Upper, upper),
    )
    .unwrap();
    let low = lower.get().unwrap();
    let high = upper.get().unwrap();
    key(&app, low, RangeCommand::Maximum);
    let lower = value(&app, low);
    let upper = value(&app, high);
    assert!(upper - lower >= 0.1, "{lower}/{upper}");
    assert!(lower <= 0.6);
    key(&app, high, RangeCommand::Minimum);
    assert!(value(&app, high) - value(&app, low) >= 0.1);
}
fn value(app: &App, id: NodeId) -> f64 {
    range(app, id).range.value
}
