use glory::blitz::shell::WindowApp;
use glory::prelude::*;
use glory::render::{NodeId, Query, QueryValue};
use std::{cell::RefCell, rc::Rc};
#[path = "../../web/src/visually_hidden_form.rs"]
mod fixture;
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut ids = tree.roots().to_vec();
    while let Some(id) = ids.pop() {
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
        ids.extend_from_slice(&node.children)
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
    let QueryValue::Bounds(value) = result.borrow_mut().take().unwrap() else {
        panic!("bounds")
    };
    value
}
#[test]
fn hidden_text_has_zero_native_box_without_moving_flow_and_patches_in_place() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(fixture::visually_hidden_form(theme)).unwrap();
        let app = host.app();
        app.settle(8).unwrap();
        let hidden = find(app, "hidden-detail");
        let before = host
            .renderer()
            .engine_id(hidden, glory::html::Part::Main)
            .unwrap();
        let rect = bounds(app, hidden);
        assert_eq!((rect.width, rect.height), (0.0, 0.0));
        assert!((bounds(app, find(app, "hidden-row")).height - 40.0).abs() < 0.01);
        let tree = app.tree();
        let update=tree.roots().iter().copied().flat_map(|id|tree.node(id).unwrap().children.clone()).find(|id|matches!(&tree.node(*id).unwrap().descriptor.view().props.clone(),glory::render::Props::Button(p) if p.label=="Update hidden detail")).unwrap();
        drop(tree);
        app.dispatch(update, glory::render::Event::Activate)
            .unwrap();
        app.settle(8).unwrap();
        assert_eq!(
            host.renderer()
                .engine_id(hidden, glory::html::Part::Main)
                .unwrap(),
            before
        );
        assert_eq!(bounds(app, hidden).width, 0.0);
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
}
