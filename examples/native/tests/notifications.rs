use glory::blitz::shell::WindowApp;
use glory::render::{Event, NodeId, Query, QueryValue, Rect};
use sapid::{Theme, notification_request, toaster};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};
fn find(app: &glory::App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let n = tree.node(id).unwrap();
        if n.descriptor.view().classes.iter().any(|c| c == class) {
            return id;
        }
        pending.extend(&n.children)
    }
    panic!("missing {class}")
}
fn bounds(app: &glory::App, node: NodeId) -> Rect {
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    app.request(Query::Bounds(node), move |r| {
        *saved.borrow_mut() = Some(r.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = out.borrow_mut().take().unwrap() else {
        panic!()
    };
    rect
}
#[test]
fn persistent_notifications_have_real_themed_geometry_actions_and_stable_updates() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let out = Rc::new(RefCell::new(None));
        let saved = out.clone();
        host.mount(
            toaster("native")
                .theme(theme)
                .visible_limit(1)
                .on_ready(move |h| *saved.borrow_mut() = Some(h)),
        )
        .unwrap();
        let handle = out.borrow_mut().take().unwrap();
        let pressed = Rc::new(Cell::new(0));
        let action = pressed.clone();
        let token = handle
            .show(
                notification_request(
                    "first",
                    "Native notice",
                    "A retained message with an action.",
                )
                .action("Undo", move || action.set(action.get() + 1)),
            )
            .unwrap();
        let queued = handle
            .show(notification_request(
                "second",
                "Queued native notice",
                "Mounted after the first closes.",
            ))
            .unwrap();
        host.app().settle(16).unwrap();
        let row = find(host.app(), "sapid-notification-row");
        let main = find(host.app(), "sapid-alert");
        assert!(bounds(host.app(), main).height > 60.);
        let engine = host
            .renderer()
            .engine_id(main, glory::html::Part::Main)
            .unwrap();
        handle
            .update(&token, "Updated native notice", "Accepted body")
            .unwrap();
        host.app().settle(16).unwrap();
        assert_eq!(find(host.app(), "sapid-notification-row"), row);
        assert_eq!(
            host.renderer()
                .engine_id(main, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        let button = {
            let tree = host.app().tree();
            let mut pending = tree.roots().to_vec();
            loop {
                let id = pending.pop().unwrap();
                let n = tree.node(id).unwrap();
                if matches!(n.descriptor.view().props,glory::render::Props::Button(p)if p.label=="Undo")
                {
                    break id;
                }
                pending.extend(&n.children)
            }
        };
        host.app().dispatch(button, Event::Activate).unwrap();
        host.app().settle(16).unwrap();
        assert_eq!(pressed.get(), 1);
        assert_eq!(handle.pending().unwrap(), 1);
        assert!(!handle.dismiss(&token).unwrap());
        assert!(bounds(host.app(), find(host.app(), "sapid-alert")).height > 40.);
        handle.dismiss(&queued).unwrap();
        host.app().settle(16).unwrap();
        assert_eq!(handle.pending().unwrap(), 0);
    }
}
#[test]
fn native_timer_default_omits_precise_hover_interest_and_keeps_model_live() {
    let host = WindowApp::new();
    let clock = sapid::NotificationClock::new(|| Duration::ZERO, |_| std::future::pending());
    let saved = Rc::new(RefCell::new(None));
    let output = saved.clone();
    host.mount(
        toaster("native-timer")
            .clock(clock)
            .on_ready(move |handle| *output.borrow_mut() = Some(handle)),
    )
    .unwrap();
    let handle = saved.borrow_mut().take().unwrap();
    handle
        .show(
            notification_request(
                "timed",
                "Timed native notice",
                "Application supplies the clock.",
            )
            .timeout(Duration::from_secs(1)),
        )
        .unwrap();
    host.app().settle(8).unwrap();
    assert_eq!(handle.pending().unwrap(), 1);
    assert!(bounds(host.app(), find(host.app(), "sapid-alert")).height > 40.);
    handle.clear().unwrap();
    host.app().settle(8).unwrap();
    assert_eq!(handle.pending().unwrap(), 0);
}
