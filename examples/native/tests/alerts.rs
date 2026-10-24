use glory::blitz::shell::WindowApp;
use glory::prelude::*;
use glory::render::{Event, NodeId, Query, QueryValue};
use std::{cell::RefCell, rc::Rc};
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let n = tree.node(id).unwrap();
        if n.descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            return id;
        }
        pending.extend(&n.children);
    }
    panic!("missing {class}")
}
fn height(app: &App, id: NodeId) -> f64 {
    let result = Rc::new(RefCell::new(None));
    let reply = result.clone();
    app.request(Query::Bounds(id), move |value| {
        *reply.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = result.borrow_mut().take().unwrap() else {
        panic!("bounds")
    };
    rect.height
}
#[test]
fn native_inline_alerts_have_real_layout_controlled_refusal_and_stable_message_nodes() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let message = host
            .app()
            .scope()
            .cage("Stored on this device.".to_string());
        let visible = host.app().scope().cage(true);
        host.mount(
            sapid::alert("native-notice")
                .theme(theme)
                .title("Changes saved")
                .content(message)
                .visible(visible)
                .closable(true),
        )
        .unwrap();
        let app = host.app();
        app.settle(12).unwrap();
        let root = find(app, "sapid-alert");
        let text = find(app, "sapid-alert-content");
        let engine = host
            .renderer()
            .engine_id(text, glory::html::Part::Main)
            .unwrap();
        assert!(height(app, root) > 60.0);
        app.dispatch(find(app, "sapid-alert-close"), Event::Activate)
            .unwrap();
        app.settle(12).unwrap();
        assert!(
            !app.tree()
                .node(root)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .hidden
        );
        message.revise(|value| *value = "Updated message without recreating the alert.".into());
        app.settle(12).unwrap();
        assert_eq!(
            host.renderer()
                .engine_id(text, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        visible.revise(|value| *value = false);
        app.settle(12).unwrap();
        assert!(
            app.tree()
                .node(root)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .hidden
        );
        visible.revise(|value| *value = true);
        app.settle(12).unwrap();
        assert_eq!(find(app, "sapid-alert"), root);
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
}
#[test]
fn native_alert_dialog_refuses_unsupported_layer_before_leaving_a_partial_component() {
    let host = WindowApp::new();
    let before = host.app().stats();
    assert!(
        host.mount(
            sapid::alert_dialog("native-decision")
                .title("Remove draft?")
                .description("This action removes the local draft.")
        )
        .is_err()
    );
    assert!(host.app().tree().is_empty());
    assert_eq!(host.app().stats().handlers, before.handlers);
    assert!(
        host.mount(sapid::floating_dialog("native-notice").title("Background sync"))
            .is_err()
    );
    assert!(host.app().tree().is_empty());
    assert_eq!(host.app().stats().handlers, before.handlers);
    host.mount(sapid::button().label("Recovery")).unwrap();
    host.app().settle(8).unwrap();
}
#[test]
fn native_skeleton_keeps_explicit_geometry_and_decorative_engine_identity_when_loading_changes()
{
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let loading = host.app().scope().cage(true);
        host.mount(
            sapid::skeleton()
                .theme(theme)
                .circle(40.0)
                .name("Loading profile")
                .loading(loading),
        )
        .unwrap();
        let app = host.app();
        app.settle(12).unwrap();
        let root = find(app, "sapid-skeleton");
        let graphic = find(app, "sapid-skeleton-placeholder");
        let engine = host
            .renderer()
            .engine_id(graphic, glory::html::Part::Main)
            .unwrap();
        assert!((height(app, root) - 40.0).abs() < 0.1);
        assert!((height(app, graphic) - 40.0).abs() < 0.1);
        loading.revise(|value| *value = false);
        app.settle(12).unwrap();
        assert!(
            app.tree()
                .node(root)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .hidden
        );
        loading.revise(|value| *value = true);
        app.settle(12).unwrap();
        assert_eq!(
            host.renderer()
                .engine_id(graphic, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        assert!((height(app, graphic) - 40.0).abs() < 0.1);
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
}
