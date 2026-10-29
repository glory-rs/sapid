#[path = "../../web/src/grid_form.rs"]
mod grid_form;
use glory::apace::shell::WindowApp;
use glory::render::{Event, NodeId, Props, Query, QueryValue, Rect};
use glory::style::{Length, Style};
use glory::ui::App;
use std::{cell::RefCell, rc::Rc};
fn find(app: &App, predicate: impl Fn(glory::render::DescriptorView<'_>) -> bool) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if predicate(node.descriptor.view()) {
            return id;
        }
        pending.extend(node.children.iter().copied());
    }
    panic!("missing node")
}
fn class(app: &App, class: &str) -> NodeId {
    find(app, |node| node.classes.iter().any(|value| value == class))
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
fn near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 0.1, "{actual} != {expected}");
}
#[test]
fn native_grid_spans_equal_cards_and_controlled_ratio_use_real_layout_and_stable_slots() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(
            glory::ui::view()
                .style(Style::new().width(Length::Px(400.0)))
                .fill(grid_form::grid_form(theme)),
        )
        .unwrap();
        let app = host.app();
        app.settle(16).unwrap();
        let main = class(app, "grid-main");
        let main_rect = bounds(app, main);
        let wide = bounds(app, class(app, "grid-wide"));
        let tail = bounds(app, class(app, "grid-tail"));
        near(main_rect.width, 400.0);
        near(wide.width, 196.0);
        near(tail.width, 298.0);
        assert!(tail.y > wide.y);
        let simple = class(app, "grid-simple");
        let children = app.tree().node(simple).unwrap().children.clone();
        for child in &children {
            near(bounds(app, *child).width, 128.0);
        }
        assert!(bounds(app, children[3]).y > bounds(app, children[0]).y);
        let ratio = class(app, "grid-ratio");
        near(bounds(app, ratio).height, 100.0);
        near(bounds(app, class(app, "grid-fixed-ratio")).height, 40.0);
        let input = class(app, "grid-draft");
        let physical = host
            .renderer()
            .engine_id(input, glory::html::Part::Main)
            .unwrap();
        let ratio_physical = host
            .renderer()
            .engine_id(ratio, glory::html::Part::Main)
            .unwrap();
        let change = find(
            app,
            |node| matches!(&node.props,Props::Button(button)if button.label=="Change ratio"),
        );
        app.dispatch(change, Event::Activate).unwrap();
        app.settle(16).unwrap();
        near(bounds(app, ratio).height, 200.0);
        assert_eq!(
            host.renderer()
                .engine_id(input, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        assert_eq!(
            host.renderer()
                .engine_id(ratio, glory::html::Part::Main)
                .unwrap(),
            ratio_physical
        );
        app.clear().unwrap();
        assert!(app.tree().is_empty());
        assert_eq!(app.stats().handlers, 0);
    }
}
