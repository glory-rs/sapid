use glory::apace::shell::WindowApp;
use glory::prelude::*;
use glory::render::{Action, NodeId, Query, QueryValue, Rect};
use sapid::{TourHandle, tour, tour_context, tour_step, tour_target};
use std::{cell::RefCell, rc::Rc};
fn settle(host: &WindowApp) {
    for _ in 0..8 {
        host.observe(&std::task::Context::from_waker(std::task::Waker::noop()))
            .unwrap();
    }
}
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    while let Some(id) = todo.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            return id;
        }
        todo.extend(&node.children);
    }
    panic!("{class}")
}
fn bounds(app: &App, node: NodeId) -> Rect {
    let result = Rc::new(RefCell::new(None));
    let out = result.clone();
    app.request(Query::Bounds(node), move |value| {
        *out.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = result.borrow_mut().take().unwrap() else {
        panic!("bounds")
    };
    rect
}
fn focused(app: &App, node: NodeId) -> bool {
    let result = Rc::new(RefCell::new(None));
    let out = result.clone();
    app.request(Query::Focus(node), move |value| {
        *out.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    result.borrow_mut().take().unwrap() == QueryValue::Focus(true)
}
#[test]
fn native_tour_reveals_current_real_target_anchors_dialog_and_preserves_identity_across_steps() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(900, 700, 1.).unwrap();
        let app = host.app();
        let ctx = tour_context(app.scope(), "native").unwrap();
        let saved = Rc::new(RefCell::new(None::<TourHandle>));
        let out = saved.clone();
        let editor = app.scope().cage(None);
        host.mount(
            view().fill((
                tour_target(&ctx, "first")
                    .fill(input().name("Kept editor").node_ref(editor).value("Draft")),
                view().style(Style::new().height(Length::Px(1100.))),
                tour_target(&ctx, "second").fill(button().label("Final target")),
                tour(&ctx, "intro")
                    .theme(theme)
                    .default_open(true)
                    .steps([
                        tour_step("first", "Write", "Edit this draft"),
                        tour_step("second", "Finish", "Continue below the fold"),
                    ])
                    .on_ready(move |handle| *out.borrow_mut() = Some(handle)),
            )),
        )
        .unwrap();
        let panel = find(app, "sapid-tour");
        assert!(
            app.tree()
                .node(panel)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .hidden
        );
        settle(&host);
        assert!(
            !app.tree()
                .node(panel)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .hidden
        );
        let physical = host
            .renderer()
            .engine_id(panel, glory::html::Part::Main)
            .unwrap();
        let editor = editor.get().unwrap();
        let editor_physical = host
            .renderer()
            .engine_id(editor, glory::html::Part::Main)
            .unwrap();
        let handle = saved.borrow().clone().unwrap();
        handle.next().unwrap();
        app.flush().unwrap();
        settle(&host);
        assert_eq!(
            host.renderer()
                .engine_id(panel, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        assert_eq!(
            host.renderer()
                .engine_id(editor, glory::html::Part::Main)
                .unwrap(),
            editor_physical
        );
        let target_id = app
            .tree()
            .node(panel)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .layer
            .as_ref()
            .unwrap()
            .anchor
            .clone()
            .unwrap();
        let target = app.tree().named(&target_id).unwrap();
        let target_bounds = bounds(app, target);
        let panel_bounds = bounds(app, panel);
        assert!(
            target_bounds.y >= 0. && target_bounds.y < 700.,
            "actual reveal: {target_bounds:?}"
        );
        assert!(
            panel_bounds.width > 0.
                && panel_bounds.y >= 0.
                && panel_bounds.y + panel_bounds.height <= 701.,
            "actual collision: {panel_bounds:?}"
        );
        handle.cancel().unwrap();
        app.flush().unwrap();
        settle(&host);
        assert!(
            app.tree()
                .node(panel)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .hidden
        );
        assert!(
            focused(app, target),
            "real host return focus reaches the final target"
        );
        app.scope().action(Action::Focus(target)).unwrap();
        app.flush().unwrap();
        app.clear().unwrap();
        assert_eq!(app.stats().queries, 0);
        assert!(handle.refresh().is_err());
        assert!(ctx.is_empty());
    }
}

#[test]
fn native_target_retirement_recovery_and_pending_owner_cancellation_keep_one_current_query() {
    let host = WindowApp::new();
    host.renderer().set_viewport(900, 700, 1.).unwrap();
    let app = host.app();
    let ctx = tour_context(app.scope(), "lifetime").unwrap();
    let present = app.scope().cage(false);
    let open = app.scope().cage(true);
    let lost = app.scope().cage(0usize);
    let saved = Rc::new(RefCell::new(None::<TourHandle>));
    let out = saved.clone();
    let targets = ctx.clone();
    host.mount(
        view().fill((
            when(present, move || {
                tour_target(&targets, "target").fill(button().label("Live target"))
            }),
            tour(&ctx, "guide")
                .open(open)
                .step(tour_step(
                    "target",
                    "Current target",
                    "Only live boxes are queried",
                ))
                .on_target_lost(move |_| lost.revise(|v| *v += 1))
                .on_ready(move |h| *out.borrow_mut() = Some(h)),
        )),
    )
    .unwrap();
    let panel = find(app, "sapid-tour");
    let handle = saved.borrow().clone().unwrap();
    settle(&host);
    assert_eq!(lost.get(), 1);
    assert_eq!(ctx.len(), 0);
    assert!(open.get());
    present.revise(|v| *v = true);
    app.flush().unwrap();
    settle(&host);
    assert!(
        !app.tree()
            .node(panel)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
    let engine = host
        .renderer()
        .engine_id(panel, glory::html::Part::Main)
        .unwrap();
    present.revise(|v| *v = false);
    app.flush().unwrap();
    settle(&host);
    assert_eq!(lost.get(), 2);
    assert!(
        app.tree()
            .node(panel)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
    for _ in 0..100 {
        handle.refresh().unwrap();
        assert!(app.stats().queries <= 1);
    }
    settle(&host);
    assert_eq!(lost.get(), 2);
    present.revise(|v| *v = true);
    app.flush().unwrap();
    settle(&host);
    assert_eq!(
        host.renderer()
            .engine_id(panel, glory::html::Part::Main)
            .unwrap(),
        engine
    );
    handle.refresh().unwrap();
    app.clear().unwrap();
    settle(&host);
    assert_eq!(app.stats().queries, 0);
    assert!(ctx.is_empty());
    assert!(handle.refresh().is_err());
}
