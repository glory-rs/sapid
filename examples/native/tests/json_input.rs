use glory::blitz::shell::WindowApp;
use glory::prelude::*;
use glory::render::{Action, Event, InputProps, NodeId, Props, Query, QueryValue};
use std::{cell::RefCell, rc::Rc};
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            return id;
        }
        pending.extend(&node.children)
    }
    panic!("missing {class}")
}
fn props(app: &App, id: NodeId) -> InputProps {
    let Props::Input(value) = app.tree().node(id).unwrap().descriptor.view().props.clone()
    else {
        panic!("input")
    };
    value
}
fn query(app: &App, value: Query) -> QueryValue {
    let output = Rc::new(RefCell::new(None));
    let result = output.clone();
    app.request(value, move |value| {
        *result.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    output.borrow_mut().take().unwrap()
}
#[test]
fn native_json_editor_four_themes_formats_and_updates_metadata_without_replacement() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let reference = app.scope().cage(None);
        host.mount(
            sapid::json_input("native")
                .theme(theme)
                .name("JSON document")
                .default_value("{\"x\":1e999,\"x\":2}")
                .node_ref(reference),
        )
        .unwrap();
        app.settle(16).unwrap();
        let id = reference.get().unwrap();
        let identity = host
            .renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap();
        let QueryValue::Bounds(rect) = query(app, Query::Bounds(id)) else {
            panic!("bounds")
        };
        assert!(rect.width > 0. && rect.height > 0.);
        app.scope().action(Action::Focus(id)).unwrap();
        app.flush().unwrap();
        assert_eq!(query(app, Query::Focus(id)), QueryValue::Focus(true));
        app.dispatch(find(app, "sapid-button"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert_eq!(props(app, id).value, "{\n  \"x\": 1e999,\n  \"x\": 2\n}");
        app.dispatch(
            id,
            Event::ValueChanged {
                value: "{".into(),
                revision: props(app, id).revision,
                composing: false,
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        assert!(
            app.tree()
                .node(id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .invalid
        );
        assert_eq!(
            host.renderer()
                .engine_id(id, glory::html::Part::Main)
                .unwrap(),
            identity
        );
        app.clear().unwrap();
        assert!(app.tree().is_empty());
        assert_eq!(app.stats().handlers, 0);
    }
}
