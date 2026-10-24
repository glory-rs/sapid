#[path = "../../web/src/avatar_form.rs"]
mod avatar_form;
use glory::blitz::shell::WindowApp;
use glory::render::{Event, NodeId, Query, QueryValue, Rect};
use glory::ui::App;
use std::{cell::RefCell, rc::Rc};
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut nodes = tree.roots().to_vec();
    while let Some(id) = nodes.pop() {
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
        nodes.extend(&node.children);
    }
    panic!("missing {class}");
}
fn bounds(app: &App, id: NodeId) -> Rect {
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
    rect
}
#[test]
fn text_avatars_have_real_native_diameters_overlap_and_patch_without_recreation() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(avatar_form::avatar_form(theme)).unwrap();
        let app = host.app();
        app.settle(16).unwrap();
        let dynamic = find(app, "avatar-dynamic");
        let physical = host
            .renderer()
            .engine_id(dynamic, glory::html::Part::Main)
            .unwrap();
        let rect = bounds(app, dynamic);
        assert!((rect.width - 36.0).abs() < 0.1 && (rect.height - 36.0).abs() < 0.1);
        assert!((bounds(app, find(app, "avatar-square")).width - 42.0).abs() < 0.1);
        assert!((bounds(app, find(app, "avatar-default")).width - 30.0).abs() < 0.1);
        let group = find(app, "avatar-team");
        let children = app.tree().node(group).unwrap().children.clone();
        assert_eq!(children.len(), 3);
        let first = bounds(app, children[0]);
        let second = bounds(app, children[1]);
        assert!(second.x < first.x + first.width && second.x > first.x);
        let button = {
            let tree = app.tree();
            let root = tree.roots()[0];
            *tree.node(root).unwrap().children.last().unwrap()
        };
        app.dispatch(button, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert_eq!(
            app.tree()
                .node(dynamic)
                .unwrap()
                .descriptor
                .view()
                .name
                .as_deref(),
            Some("Katherine Johnson")
        );
        assert_eq!(
            host.renderer()
                .engine_id(dynamic, glory::html::Part::Main)
                .unwrap(),
            physical
        );
    }
}
