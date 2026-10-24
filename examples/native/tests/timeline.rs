#[path = "../../web/src/timeline_form.rs"]
mod timeline_form;
use glory::blitz::shell::WindowApp;
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
    panic!("missing fixture node")
}
fn class(app: &App, name: &str) -> NodeId {
    find(app, |node| node.classes.iter().any(|value| value == name))
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
fn native_lines_have_real_axes_and_progress_preserves_mounted_items_and_input() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(
            glory::ui::col()
                .style(Style::new().width(Length::Px(600.0)))
                .fill(timeline_form::timeline_form(theme)),
        )
        .unwrap();
        let app = host.app();
        app.settle(16).unwrap();
        let horizontal = bounds(app, class(app, "sapid-separator-horizontal"));
        let vertical = bounds(app, class(app, "sapid-separator-vertical"));
        assert!(
            (horizontal.width - 240.0).abs() < 0.1 && (horizontal.height - 1.0).abs() < 0.1,
            "{horizontal:?}"
        );
        assert!(
            (vertical.width - 1.0).abs() < 0.1 && (vertical.height - 32.0).abs() < 0.1,
            "{vertical:?}"
        );
        let timeline = class(app, "sapid-timeline");
        let list = bounds(app, timeline);
        assert!(list.width > 0.0 && list.height > 0.0);
        let items = app.tree().node(timeline).unwrap().children.clone();
        let field = class(app, "timeline-draft");
        let engine_field = host
            .renderer()
            .engine_id(field, glory::html::Part::Main)
            .unwrap();
        let advance = find(app, |node| {
            node.name.as_deref() == Some("Advance timeline")
                || matches!(&node.props, Props::Button(button) if button.label == "Advance timeline")
        });
        app.dispatch(advance, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert_eq!(app.tree().node(timeline).unwrap().children, items);
        assert_eq!(
            host.renderer()
                .engine_id(field, glory::html::Part::Main)
                .unwrap(),
            engine_field
        );
        let current = class(app, "sapid-timeline-current");
        assert_eq!(current, items[2]);
        app.clear().unwrap();
        assert!(app.tree().is_empty());
        assert_eq!(app.stats().handlers, 0);
    }
}
