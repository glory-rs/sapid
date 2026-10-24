use glory::prelude::*;
use glory::render::{Event, NodeId, Props, RangeCommand};
use glory::style::{Color, Length};
use sapid::{alpha_slider, hue_slider};
use std::{cell::RefCell, rc::Rc};
fn value(app: &App, id: NodeId) -> f64 {
    app.tree()
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .slider
        .as_ref()
        .unwrap()
        .range
        .value
}
fn key(app: &App, id: NodeId, command: RangeCommand) {
    app.dispatch(id, Event::Range { from: id, command })
        .unwrap();
}
fn classes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut ids = Vec::new();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            ids.push(id)
        }
        pending.extend(&node.children)
    }
    ids
}
#[test]
fn hue_local_same_tick_proposals_keep_fractional_precision_and_true_slider_role() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    app.mount(
        hue_slider("hue")
            .default_value(120.25)
            .step(0.25)
            .node_ref(reference),
    )
    .unwrap();
    let id = reference.get().unwrap();
    app.scope()
        .batch(|| {
            key(&app, id, RangeCommand::Increase);
            key(&app, id, RangeCommand::Increase)
        })
        .unwrap();
    assert_eq!(value(&app, id), 120.75);
    assert_eq!(
        app.tree().node(id).unwrap().descriptor.view().role,
        Some(glory::render::Role::Slider)
    );
    assert!(matches!(
        app.tree().node(id).unwrap().descriptor.view().props,
        Props::View
    ));
    assert_eq!(classes(&app, "sapid-color-slider-segment").len(), 120);
    key(&app, id, RangeCommand::Maximum);
    app.flush().unwrap();
    assert_eq!(value(&app, id), 360.);
    key(&app, id, RangeCommand::Increase);
    app.flush().unwrap();
    assert_eq!(value(&app, id), 360.);
}
#[test]
fn alpha_controlled_rejection_and_commit_only_acceptance_follow_shared_cursor() {
    let app = App::memory();
    let source = app.scope().cage(0.25);
    let accept = app.scope().cage(false);
    let reference = app.scope().cage(None);
    let log = Rc::new(RefCell::new(Vec::new()));
    let changes = log.clone();
    let commits = log.clone();
    app.mount(
        alpha_slider("alpha")
            .value(source)
            .step(0.01)
            .node_ref(reference)
            .on_value_change(move |v| changes.borrow_mut().push(("change", v)))
            .on_value_commit(move |v| {
                commits.borrow_mut().push(("commit", v));
                if accept.get_untracked() {
                    source.revise(|old| *old = v)
                }
            }),
    )
    .unwrap();
    let id = reference.get().unwrap();
    key(&app, id, RangeCommand::Increase);
    app.flush().unwrap();
    assert_eq!(source.get(), 0.25);
    assert_eq!(value(&app, id), 0.25);
    assert_eq!(*log.borrow(), [("change", 0.26), ("commit", 0.26)]);
    accept.revise(|old| *old = true);
    app.scope()
        .batch(|| {
            key(&app, id, RangeCommand::Increase);
            key(&app, id, RangeCommand::Increase)
        })
        .unwrap();
    assert!((source.get() - 0.27).abs() < 1e-12);
    assert_eq!(value(&app, id), source.get());
}
#[test]
fn alpha_trail_keeps_real_transparency_checker_and_rgb_source_updates() {
    let app = App::memory();
    let color = app.scope().cage(Color(10, 20, 30, 0));
    let source = app.scope().cage(0.123456789);
    let reference = app.scope().cage(None);
    app.mount(
        alpha_slider("alpha")
            .color(color)
            .value(source)
            .node_ref(reference)
            .unstyled(),
    )
    .unwrap();
    assert_eq!(value(&app, reference.get().unwrap()), 0.123456789);
    assert_eq!(source.get(), 0.123456789);
    assert_eq!(classes(&app, "sapid-alpha-checker-cell").len(), 64);
    let mut segments = classes(&app, "sapid-color-slider-segment");
    let left = |id: &NodeId| match app.tree().node(*id).unwrap().descriptor.view().style.left {
        Some(Length::Percent(v)) => v,
        _ => panic!("left"),
    };
    segments.sort_by(|a, b| left(a).partial_cmp(&left(b)).unwrap());
    assert_eq!(
        app.tree()
            .node(segments[0])
            .unwrap()
            .descriptor
            .view()
            .style
            .background,
        Some(Color(10, 20, 30, 0))
    );
    assert_eq!(
        app.tree()
            .node(*segments.last().unwrap())
            .unwrap()
            .descriptor
            .view()
            .style
            .background,
        Some(Color(10, 20, 30, 255))
    );
    color.revise(|old| *old = Color(40, 50, 60, 77));
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(*segments.last().unwrap())
            .unwrap()
            .descriptor
            .view()
            .style
            .background,
        Some(Color(40, 50, 60, 255))
    );
    assert_eq!(classes(&app, "sapid-color-slider-segment").len(), 64);
}
#[test]
fn field_label_readonly_disabled_and_snapshot_stay_on_actual_control() {
    let app = App::memory();
    let readonly = app.scope().cage(true);
    let reference = app.scope().cage(None);
    app.mount(
        sapid::field("opacity")
            .label("Layer opacity")
            .description("Alpha channel")
            .form_name("opacity")
            .required(true)
            .readonly(readonly)
            .control(move |field| {
                alpha_slider("alpha")
                    .field(field)
                    .node_ref(reference)
                    .default_value(0.5)
            }),
    )
    .unwrap();
    let id = reference.get().unwrap();
    let tree = app.tree();
    let descriptor = tree.node(id).unwrap().descriptor.view();
    let semantics = descriptor.semantics.as_ref().unwrap();
    assert!(semantics.labelled_by.is_some());
    assert!(descriptor.name.is_none());
    assert!(semantics.readonly && semantics.required);
    assert_eq!(semantics.form_control.as_ref().unwrap().name, "opacity");
    assert_eq!(
        semantics.form_control.as_ref().unwrap().value.as_deref(),
        Some("0.5")
    );
    drop(tree);
    key(&app, id, RangeCommand::Increase);
    app.flush().unwrap();
    assert_eq!(value(&app, id), 0.5);
    readonly.revise(|old| *old = false);
    app.flush().unwrap();
    key(&app, id, RangeCommand::Increase);
    app.flush().unwrap();
    assert_eq!(value(&app, id), 0.51);
    assert_eq!(reference.get(), Some(id));
}
#[test]
fn invalid_sources_retain_last_accepted_preview_and_configuration_is_bounded() {
    for bad in [f64::NAN, -0.1, 360.1] {
        let app = App::memory();
        assert!(app.mount(hue_slider("bad").default_value(bad)).is_err());
        assert_eq!(app.stats().nodes, 0)
    }
    assert!(App::memory().mount(alpha_slider("x".repeat(59))).is_err());
    let app = App::memory();
    let source = app.scope().cage(200.);
    let reference = app.scope().cage(None);
    app.mount(hue_slider("recover").value(source).node_ref(reference))
        .unwrap();
    let id = reference.get().unwrap();
    source.revise(|old| *old = f64::INFINITY);
    assert!(app.flush().is_err());
    assert_eq!(value(&app, id), 200.);
    source.revise(|old| *old = 123.456789);
    app.flush().unwrap();
    assert_eq!(value(&app, id), 123.456789);
    assert_eq!(reference.get(), Some(id));
}
#[test]
fn owner_retirement_releases_fixed_trails_references_and_captures() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    let capture = Rc::new(());
    let weak = Rc::downgrade(&capture);
    let mounted = app
        .scope()
        .mount_with(move || {
            hue_slider("owned")
                .node_ref(reference)
                .on_value_change(move |_| {
                    let _ = &capture;
                })
        })
        .unwrap();
    assert!(reference.get().is_some());
    drop(mounted);
    app.flush().unwrap();
    assert!(weak.upgrade().is_none());
    assert_eq!(reference.get(), None);
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(app.stats().handlers, 0);
}

#[test]
fn ssr_emits_actual_slider_numbers_and_bounded_color_trails() {
    let html = glory::ssr::render(hue_slider("ssr").default_value(123.456789)).unwrap();
    assert!(html.contains("role=\"slider\""));
    assert!(html.contains("aria-valuenow=\"123.456789\""));
    assert!(html.contains("aria-valuemax=\"360\""));
    assert_eq!(
        html.matches("class=\"sapid-color-slider-segment\"").count(),
        120
    );
    assert!(!html.contains("sapid-slider-fill"));
}
