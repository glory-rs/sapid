#[path = "../../web/src/graphics_form.rs"]
mod graphics_form;
use glory::apace::shell::WindowApp;
use glory::render::{Event, NodeId, Props, Query, QueryValue, Rect};
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
    panic!("missing {class}")
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
        panic!()
    };
    rect
}
#[test]
fn marker_anchors_real_graphics_and_reactive_identity_work_in_all_native_themes() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(graphics_form::graphics_form(theme)).unwrap();
        let app = host.app();
        app.settle(16).unwrap();
        let wrapper = find(app, "graphics-indicator");
        let marker = app.tree().node(wrapper).unwrap().children[1];
        let wrap = bounds(app, wrapper);
        let mark = bounds(app, marker);
        assert!((mark.width - 20.0).abs() < 0.1 && (mark.height - 20.0).abs() < 0.1);
        assert!(
            (mark.x + 10.0 - wrap.x - wrap.width).abs() < 0.1,
            "{mark:?} {wrap:?}"
        );
        assert!((mark.y + 10.0 - wrap.y).abs() < 0.1);
        let icon = find(app, "graphics-dynamic");
        let graphic = app.tree().node(icon).unwrap().children[0];
        let icon_bounds = bounds(app, icon);
        let graphic_bounds = bounds(app, graphic);
        assert!((icon_bounds.width - 36.0).abs() < 0.1);
        assert!((graphic_bounds.width - 21.6).abs() < 0.1);
        let physical = host
            .renderer()
            .engine_id(graphic, glory::html::Part::Main)
            .unwrap();
        host.renderer().inspect(|doc| {
            assert!(
                doc.get_node(physical)
                    .unwrap()
                    .element_data()
                    .unwrap()
                    .custom_widget_data()
                    .is_some()
            )
        });
        let update = {
            let tree = app.tree();
            let root = tree.node(tree.roots()[0]).unwrap();
            root.children[2]
        };
        app.dispatch(update, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(
            matches!(app.tree().node(graphic).unwrap().descriptor.view().props,Props::Graphic(props) if props.alternative.as_deref()==Some("Closed"))
        );
        assert_eq!(
            host.renderer()
                .engine_id(graphic, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        let toggle = {
            let tree = app.tree();
            tree.node(tree.roots()[0]).unwrap().children[3]
        };
        app.dispatch(toggle, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(app.tree().node(marker).unwrap().descriptor.view().hidden());
        app.dispatch(toggle, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(!app.tree().node(marker).unwrap().descriptor.view().hidden());
        app.clear().unwrap();
        assert_eq!(host.renderer().stats().nodes, 0);
    }
}
