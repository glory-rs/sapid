use glory::apace::shell::WindowApp;
use glory::render::{Event, NodeId, Props, Query, QueryValue, Rect};
use glory::ui::App;
use std::{cell::RefCell, rc::Rc};
#[path = "../../web/src/messages_form.rs"]
mod fixture;
#[path = "../../web/src/message_scroller_form.rs"]
#[allow(dead_code)]
mod scroller_fixture;
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
fn bounds(app: &App, id: NodeId) -> Rect {
    let output = Rc::new(RefCell::new(None));
    let reply = output.clone();
    app.request(Query::Bounds(id), move |value| {
        *reply.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(bounds) = output.borrow_mut().take().unwrap() else {
        panic!("bounds")
    };
    bounds
}
#[test]
fn native_message_body_quote_grouping_and_application_action_have_actual_layout() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(fixture::messages_form(theme)).unwrap();
        let app = host.app();
        app.settle(12).unwrap();
        let body = find(
            app,
            |node| matches!(node.props,Props::Text(text) if text.content=="A public answer 🙂\nSecond line"),
        );
        let original = bounds(app, body);
        assert!(original.width > 40. && original.height > 20.);
        let marker = find(
            app,
            |node| matches!(node.props,Props::Link(link) if link.target=="#sapid-message-8_question"),
        );
        assert!(bounds(app, marker).width > 20.);
        let complete = find(
            app,
            |node| matches!(node.props,Props::Button(button) if button.label=="Complete message"),
        );
        app.dispatch(complete, Event::Activate).unwrap();
        app.settle(12).unwrap();
        assert_eq!(
            body,
            find(
                app,
                |node| matches!(node.props,Props::Text(text) if text.content=="Completed answer 🙂")
            )
        );
        assert!(bounds(app, body).height > 10.);
        let retry = find(
            app,
            |node| matches!(node.props,Props::Button(button) if button.label=="Application retry"),
        );
        app.dispatch(retry, Event::Activate).unwrap();
        app.settle(12).unwrap();
        find(
            app,
            |node| matches!(node.props,Props::Text(text) if text.content=="Retries 1"),
        );
    }
}

#[test]
fn native_message_window_has_real_scrolling_bounded_rows_and_returns_to_latest() {
    use glory::render::{Action, VirtualPart};
    let host = WindowApp::new();
    let controller = host.app().scope().cage(None);
    host.mount(scroller_fixture::message_scroller_form_with_controller(
        sapid::Theme::blue_light(),
        Some(controller),
    ))
    .unwrap();
    let observe = || {
        for _ in 0..8 {
            host.observe(&std::task::Context::from_waker(std::task::Waker::noop()))
                .unwrap();
        }
    };
    observe();
    let app = host.app();
    let root = find(app, |node| {
        node.options.get::<VirtualPart>() == Some(&VirtualPart::Root)
    });
    assert!((bounds(app, root).height - 320.).abs() < 1.);
    let physical = Rc::new(RefCell::new(0.));
    let reply = physical.clone();
    app.request(Query::ScrollMetrics(root), move |value| {
        let QueryValue::ScrollMetrics(metrics) = value.unwrap() else {
            panic!("metrics")
        };
        *reply.borrow_mut() = metrics.y;
    })
    .unwrap();
    observe();
    assert!(*physical.borrow() > 1000.);
    controller.get().unwrap().pause().unwrap();
    app.scope()
        .action(Action::ScrollTo {
            node: root,
            x: 0.,
            y: *physical.borrow() - 600.,
        })
        .unwrap();
    observe();
    find(
        app,
        |node| matches!(node.props,Props::Text(text) if text.content=="following false; unread 0"),
    );
    let append = find(
        app,
        |node| matches!(node.props,Props::Button(button) if button.label=="Append five"),
    );
    app.dispatch(append, Event::Activate).unwrap();
    observe();
    find(
        app,
        |node| matches!(node.props,Props::Text(text) if text.content=="following false; unread 5"),
    );
    let follow = find(
        app,
        |node| matches!(node.props,Props::Button(button) if button.label=="Follow latest"),
    );
    app.dispatch(follow, Event::Activate).unwrap();
    observe();
    find(
        app,
        |node| matches!(node.props,Props::Text(text) if text.content=="following true; unread 0"),
    );
    let last = find(app, |node| {
        node.semantics.as_ref().is_some_and(|semantics| {
            semantics
                .id
                .as_ref()
                .is_some_and(|id| id.as_str() == "sapid-message-11_row-1001004")
        })
    });
    let view = bounds(app, root);
    let row = bounds(app, last);
    assert!(
        row.y < view.y + view.height && row.y + row.height > view.y,
        "{row:?} {view:?}"
    );
    assert!(app.stats().nodes < 1500);
    let mut raster = glory::apace::Raster::new();
    raster.draw(host.renderer()).unwrap();
    assert!(
        raster
            .pixels()
            .as_chunks::<4>()
            .0
            .iter()
            .any(|pixel| pixel[3] > 0)
    );
}
