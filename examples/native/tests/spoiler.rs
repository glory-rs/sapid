use glory::apace::shell::WindowApp;
use glory::prelude::*;
use glory::render::{Event, NodeId, Props, Query, QueryValue};
use std::{cell::RefCell, rc::Rc};
#[path = "../../web/src/spoiler_form.rs"]
mod spoiler_form;
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
        pending.extend(node.children.iter().copied());
    }
    panic!("missing {class}")
}
fn descendant(app: &App, root: NodeId, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = vec![root];
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
        pending.extend(node.children.iter().copied());
    }
    panic!("missing {class}")
}
fn bounds(app: &App, id: NodeId) -> glory::render::Rect {
    let result = Rc::new(RefCell::new(None));
    let out = result.clone();
    app.request(Query::Bounds(id), move |value| {
        *out.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = result.borrow_mut().take().unwrap() else {
        panic!("bounds")
    };
    rect
}
#[test]
fn native_text_measurements_drive_expansion_and_keep_the_original_text_identity() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(
            sapid::stack()
                .width(Length::Px(480.0))
                .fill(spoiler_form::spoiler_form(theme)),
        )
        .unwrap();
        let app = host.app();
        app.settle(8).unwrap();
        let root = find(app, "local-spoiler");
        let viewport = descendant(app, root, "sapid-spoiler-viewport");
        let text = descendant(app, root, "sapid-spoiler-text");
        let trigger = descendant(app, root, "sapid-spoiler-trigger");
        let physical = host
            .renderer()
            .engine_id(text, glory::html::Part::Main)
            .unwrap();
        assert!((bounds(app, viewport).height - 80.0).abs() < 0.1);
        assert!(bounds(app, text).height > 80.0);
        app.settle(8).unwrap();
        app.dispatch(trigger, Event::Activate).unwrap();
        app.settle(8).unwrap();
        assert!(bounds(app, viewport).height > 80.0);
        assert!(
            matches!(&app.tree().node(text).unwrap().descriptor.view().props.clone(),Props::Text(value) if value.content.len()>500)
        );
        assert_eq!(
            host.renderer()
                .engine_id(text, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        app.dispatch(trigger, Event::Activate).unwrap();
        app.settle(8).unwrap();
        assert!((bounds(app, viewport).height - 80.0).abs() < 0.1);
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
}
