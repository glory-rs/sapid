use glory::blitz::shell::WindowApp;
use glory::prelude::*;
use glory::render::{Event, LinkOpen, NodeId, Query, QueryValue};
use std::{cell::RefCell, rc::Rc};
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let n = tree.node(id).unwrap();
        if n.descriptor.view().classes.iter().any(|c| c == class) {
            return id;
        }
        pending.extend(n.children.iter().copied());
    }
    panic!("missing {class}")
}
fn bounds(app: &App, id: NodeId) -> glory::render::Rect {
    let response = Rc::new(RefCell::new(None));
    let output = response.clone();
    app.request(Query::Bounds(id), move |value| {
        *output.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = response.borrow_mut().take().unwrap() else {
        panic!("bounds")
    };
    rect
}
#[test]
fn text_title_and_current_context_anchor_use_real_native_layout_and_keep_identity() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let content = app.scope().cage("Before".to_owned());
        let href = app.scope().cage("/before".to_owned());
        let count = app.scope().cage(0usize);
        host.mount(
            sapid::typography()
                .theme(theme)
                .style(Style::new().width(Length::Px(420.0)))
                .content(move |context| {
                    (
                        context.title(2).class("title").content("Native typography"),
                        context.text().class("body").content(content),
                        context
                            .anchor()
                            .class("anchor")
                            .href(href)
                            .label("Read documentation")
                            .style(
                                Style::new()
                                    .display(Display::Block)
                                    .width(Length::Px(220.0)),
                            )
                            .on_navigate(move |request| {
                                assert_eq!(request.open, LinkOpen::Current);
                                count.revise(|v| *v += 1)
                            }),
                    )
                }),
        )
        .unwrap();
        app.settle(12).unwrap();
        for class in ["title", "body", "anchor"] {
            let rect = bounds(app, find(app, class));
            assert!(rect.width > 0.0 && rect.height > 0.0, "{class}: {rect:?}");
        }
        let body = find(app, "body");
        let anchor = find(app, "anchor");
        let body_engine = host
            .renderer()
            .engine_id(body, glory::html::Part::Main)
            .unwrap();
        let anchor_engine = host
            .renderer()
            .engine_id(anchor, glory::html::Part::Main)
            .unwrap();
        content.revise(|v| *v = "After".into());
        href.revise(|v| *v = "/after".into());
        app.settle(12).unwrap();
        assert_eq!(
            host.renderer()
                .engine_id(body, glory::html::Part::Main)
                .unwrap(),
            body_engine
        );
        assert_eq!(
            host.renderer()
                .engine_id(anchor, glory::html::Part::Main)
                .unwrap(),
            anchor_engine
        );
        assert_eq!(count.get(), 0);
        app.dispatch(anchor, Event::Activate).unwrap();
        assert_eq!(count.get(), 1);
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
}
#[test]
fn unsupported_native_ellipsis_and_new_link_context_are_explicit_and_atomic() {
    let host = WindowApp::new();
    assert!(
        host.mount(sapid::text().truncate(true).content("Full text"))
            .is_err()
    );
    assert!(host.app().tree().is_empty());
    let host = WindowApp::new();
    assert!(
        host.mount(
            sapid::anchor()
                .href("https://example.test")
                .label("External")
                .open_in(LinkOpen::NewContext)
        )
        .is_err()
    );
    assert!(host.app().tree().is_empty());
    let host = WindowApp::new();
    host.mount(
        sapid::text()
            .truncate(false)
            .content("Supported native text"),
    )
    .unwrap();
    assert!(!host.app().tree().is_empty());
}

#[test]
fn rejected_native_context_update_keeps_accepted_navigation_request() {
    let host = WindowApp::new();
    let app = host.app();
    let open = app.scope().cage(LinkOpen::Current);
    let last = Rc::new(RefCell::new(None));
    let output = last.clone();
    host.mount(
        sapid::anchor()
            .class("anchor")
            .href("/accepted")
            .label("Read")
            .open_in(open)
            .on_navigate(move |request| *output.borrow_mut() = Some(request)),
    )
    .unwrap();
    let node = find(app, "anchor");
    let physical = host
        .renderer()
        .engine_id(node, glory::html::Part::Main)
        .unwrap();
    open.revise(|v| *v = LinkOpen::NewContext);
    assert!(app.flush().is_err());
    app.dispatch(node, Event::Activate).unwrap();
    assert_eq!(last.borrow().as_ref().unwrap().open, LinkOpen::Current);
    assert_eq!(last.borrow().as_ref().unwrap().href, "/accepted");
    assert_eq!(
        host.renderer()
            .engine_id(node, glory::html::Part::Main)
            .unwrap(),
        physical
    );
}
