use glory::blitz::shell::WindowApp;
use glory::prelude::*;
use glory::render::{NodeId, Query, QueryValue};
use std::{cell::RefCell, rc::Rc};
#[path = "../../web/src/layout_form.rs"]
mod layout_form;
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
fn native_layout_boxes_center_wrap_and_resize_using_real_engine_geometry() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(
            sapid::stack()
                .width(Length::Px(600.0))
                .fill(layout_form::layout_form(theme)),
        )
        .unwrap();
        let app = host.app();
        app.settle(8).unwrap();
        let outer = bounds(app, find(app, "bounded-container"));
        let child = bounds(app, find(app, "container-child"));
        assert!((child.width - 480.0).abs() < 0.1);
        assert!((child.x + child.width / 2.0 - outer.x - outer.width / 2.0).abs() < 0.1);
        let first = bounds(app, find(app, "wrap-one"));
        let second = bounds(app, find(app, "wrap-two"));
        let third = bounds(app, find(app, "wrap-three"));
        assert!((first.y - second.y).abs() < 0.1);
        assert!((third.y - first.y - first.height - 10.0).abs() < 0.1);
        let center = bounds(app, find(app, "centered-box"));
        let inner = bounds(app, find(app, "center-child"));
        assert!((inner.x + inner.width / 2.0 - center.x - center.width / 2.0).abs() < 0.1);
        assert!((inner.y + inner.height / 2.0 - center.y - center.height / 2.0).abs() < 0.1);
        assert!((bounds(app, find(app, "explicit-space")).height - 24.0).abs() < 0.1);
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
}
