use glory::apace::shell::WindowApp;
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
fn focused(app: &App, id: NodeId) -> bool {
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    app.request(Query::Focus(id), move |value| {
        *saved.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    out.borrow_mut().take().unwrap() == QueryValue::Focus(true)
}
#[test]
fn native_alert_dialog_uses_safe_focus_controlled_refusal_real_pixels_and_return() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(800, 600, 1.).unwrap();
        let app = host.app();
        let open = app.scope().cage(false);
        let outside = app.scope().cage(None);
        let panel = app.scope().cage(None);
        let calls = app.scope().cage(0);
        host.mount(
            glory::ui::view().fill((
                sapid::button().label("Outside").node_ref(outside),
                sapid::alert_dialog("native-decision")
                    .theme(theme)
                    .title("Remove draft?")
                    .description("This action removes the local draft.")
                    .open(open)
                    .panel_ref(panel)
                    .on_confirm(move || calls.revise(|v| *v += 1)),
            )),
        )
        .unwrap();
        app.settle(16).unwrap();
        app.scope()
            .action(glory::render::Action::Focus(outside.get().unwrap()))
            .unwrap();
        app.settle(16).unwrap();
        open.revise(|v| *v = true);
        app.settle(16).unwrap();
        let panel = panel.get().unwrap();
        let cancel = find(app, "sapid-layer-close");
        let confirm = find(app, "sapid-alert-dialog-confirm");
        assert!(height(app, panel) > 100.);
        assert!(focused(app, cancel));
        let engine = host
            .renderer()
            .engine_id(panel, glory::html::Part::Main)
            .unwrap();
        let mut raster = glory::apace::Raster::new();
        raster.draw(host.renderer()).unwrap();
        assert!(raster.pixels().as_chunks::<4>().0.iter().any(|p| p[3] > 0));
        app.dispatch(confirm, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert_eq!(calls.get(), 1);
        assert!(open.get());
        assert_eq!(
            host.renderer()
                .engine_id(panel, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        open.revise(|v| *v = false);
        app.settle(16).unwrap();
        assert!(focused(app, find(app, "sapid-alert-dialog-trigger")));
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(app.stats().queries, 0);
    }
}
#[test]
fn native_floating_dialog_preserves_background_focus_and_releases_its_owner() {
    let host = WindowApp::new();
    let app = host.app();
    let open = app.scope().cage(false);
    let outside = app.scope().cage(None);
    let panel = app.scope().cage(None);
    host.mount(
        glory::ui::view().fill((
            sapid::button().label("Background").node_ref(outside),
            sapid::floating_dialog("notice")
                .title("Background sync")
                .open(open)
                .panel_ref(panel),
        )),
    )
    .unwrap();
    app.settle(16).unwrap();
    app.scope()
        .action(glory::render::Action::Focus(outside.get().unwrap()))
        .unwrap();
    app.settle(16).unwrap();
    open.revise(|v| *v = true);
    app.settle(16).unwrap();
    assert!(height(app, panel.get().unwrap()) > 50.);
    assert!(focused(app, outside.get().unwrap()));
    open.revise(|v| *v = false);
    app.settle(16).unwrap();
    assert!(focused(app, outside.get().unwrap()));
    app.clear().unwrap();
    assert_eq!(app.stats().handlers, 0);
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
