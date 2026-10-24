use glory::blitz::shell::WindowApp;
use glory::render::{Query, QueryValue, Rect};
use std::{cell::RefCell, rc::Rc};
fn bounds(app: &glory::App, id: glory::render::NodeId) -> Rect {
    let result = Rc::new(RefCell::new(None));
    let saved = result.clone();
    app.request(Query::Bounds(id), move |value| {
        *saved.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = result.borrow_mut().take().unwrap() else {
        panic!()
    };
    rect
}
#[test]
fn application_progress_uses_real_geometry_and_retains_host_identity_across_cancel_restart() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let reference = app.scope().cage(None);
        let saved = Rc::new(RefCell::new(None));
        let out = saved.clone();
        host.mount(
            sapid::navigation_progress()
                .theme(theme)
                .thickness(8.)
                .style(glory::style::Style::new().width(glory::style::Length::Px(300.)))
                .node_ref(reference)
                .on_ready(move |handle| *out.borrow_mut() = Some(handle)),
        )
        .unwrap();
        let handle = saved.borrow_mut().take().unwrap();
        app.settle(16).unwrap();
        let root = reference.get().unwrap();
        let bar = app.tree().node(root).unwrap().children[0];
        let fill = app.tree().node(bar).unwrap().children[0];
        let token = handle.start().unwrap();
        handle.set(&token, 40.).unwrap();
        app.settle(16).unwrap();
        assert!((bounds(app, bar).width - 300.).abs() < 0.1);
        assert!((bounds(app, fill).width - 120.).abs() < 0.1);
        let engine = host
            .renderer()
            .engine_id(fill, glory::html::Part::Main)
            .unwrap();
        handle.cancel(&token).unwrap();
        app.settle(16).unwrap();
        let token = handle.start().unwrap();
        handle.set(&token, 75.).unwrap();
        app.settle(16).unwrap();
        assert!((bounds(app, fill).width - 225.).abs() < 0.1);
        assert_eq!(
            host.renderer()
                .engine_id(fill, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        handle.finish(&token).unwrap();
        app.settle(16).unwrap();
        assert!(
            app.tree()
                .node(root)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .hidden
        );
    }
}
