#[path = "../../web/src/toolbar_form.rs"]
mod fixture;
use glory::blitz::shell::WindowApp;
use glory::render::{Event, Navigation, NodeId, Props, Query, QueryValue, Rect};
use std::{cell::RefCell, rc::Rc};
fn button(app: &glory::App, label: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if matches!(node.descriptor.view().props,Props::Button(p)if p.label==label) {
            return id;
        }
        pending.extend(&node.children)
    }
    panic!("missing {label}")
}
fn bounds(app: &glory::App, node: NodeId) -> Rect {
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    app.request(Query::Bounds(node), move |value| {
        *saved.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(value) = out.borrow_mut().take().unwrap() else {
        panic!()
    };
    value
}
#[test]
fn grouped_toolbar_native_layout_and_selection_updates_keep_engine_identities() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(fixture::toolbar_form(theme)).unwrap();
        let app = host.app();
        app.settle(16).unwrap();
        let cut = button(app, "Cut draft");
        let undo = button(app, "Undo draft");
        let copy = button(app, "Copy draft");
        let identity = host
            .renderer()
            .engine_id(cut, glory::html::Part::Main)
            .unwrap();
        let first = bounds(app, cut);
        let next = bounds(app, undo);
        assert!(first.width > 40. && first.height >= 30.);
        assert!(next.x > first.x + first.width);
        app.dispatch(
            cut,
            Event::Navigate {
                from: cut,
                command: Navigation::Next,
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        assert_eq!(
            app.tree().node(undo).unwrap().descriptor.view().focus,
            glory::render::FocusMode::TabStop
        );
        app.dispatch(button(app, "Toggle copy enabled"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert!(
            matches!(app.tree().node(copy).unwrap().descriptor.view().props,Props::Button(p)if p.enabled)
        );
        app.dispatch(button(app, "Select three rows"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        let archive = button(app, "Archive selected");
        assert!(bounds(app, archive).width > 60.);
        let retained = host
            .renderer()
            .engine_id(archive, glory::html::Part::Main)
            .unwrap();
        app.dispatch(button(app, "Accept clear selection"), Event::Activate)
            .unwrap();
        app.dispatch(button(app, "Clear selection"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert!(!app.tree().active(archive));
        app.dispatch(button(app, "Select three rows"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert_eq!(
            host.renderer()
                .engine_id(archive, glory::html::Part::Main)
                .unwrap(),
            retained
        );
        assert_eq!(
            host.renderer()
                .engine_id(cut, glory::html::Part::Main)
                .unwrap(),
            identity
        );
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
}
