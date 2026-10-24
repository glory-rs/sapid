use glory::blitz::shell::WindowApp;
use glory::prelude::*;
use glory::render::{Action, Navigation, NodeId, Props, Query, QueryValue};
use sapid::{navigation_menu, navigation_menu_link, navigation_menu_panel};
use std::{cell::RefCell, rc::Rc};
fn control(app: &App, label: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if match &node.descriptor.view().props {
            Props::Link(p) => p.label == label,
            Props::Button(p) => p.label == label,
            _ => false,
        } {
            return id;
        }
        pending.extend(node.children.iter().copied())
    }
    panic!("missing {label}")
}
fn focused(app: &App, node: NodeId) -> bool {
    let reply = Rc::new(RefCell::new(None));
    let out = reply.clone();
    app.request(Query::Focus(node), move |result| {
        *out.borrow_mut() = Some(result.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let result = reply.borrow_mut().take().unwrap();
    result == QueryValue::Focus(true)
}
#[test]
fn native_closed_navigation_moves_actual_focus_and_preserves_engine_controls_in_four_themes() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(navigation_menu("native-nav").theme(theme).content(|ctx| {
            (
                navigation_menu_link("home")
                    .context(ctx.clone())
                    .label("Home")
                    .href("/home"),
                navigation_menu_panel("products")
                    .context(ctx.clone())
                    .label("Products")
                    .content(|ctx| {
                        navigation_menu_link("editor")
                            .context(ctx)
                            .label("Editor")
                            .href("/editor")
                    }),
                navigation_menu_panel("disabled")
                    .context(ctx)
                    .label("Disabled")
                    .enabled(false),
            )
        }))
        .unwrap();
        let app = host.app();
        app.settle(16).unwrap();
        let home = control(app, "Home");
        let products = control(app, "Products");
        let engine = host
            .renderer()
            .engine_id(products, glory::html::Part::Main)
            .unwrap();
        app.scope().action(Action::Focus(home)).unwrap();
        app.settle(16).unwrap();
        assert!(focused(app, home));
        app.dispatch(
            home,
            Event::Navigate {
                from: home,
                command: Navigation::Next,
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        assert!(focused(app, products));
        app.dispatch(
            products,
            Event::Navigate {
                from: products,
                command: Navigation::Next,
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        assert!(focused(app, home));
        assert_eq!(
            host.renderer()
                .engine_id(products, glory::html::Part::Main)
                .unwrap(),
            engine
        );
    }
}
#[test]
fn native_group_layer_rejection_keeps_closed_descriptor_and_engine_identity() {
    let host = WindowApp::new();
    host.mount(navigation_menu("native-reject").content(|ctx| {
        navigation_menu_panel("products")
            .context(ctx)
            .label("Products")
            .content(|ctx| {
                navigation_menu_link("editor")
                    .context(ctx)
                    .label("Editor")
                    .href("/editor")
            })
    }))
    .unwrap();
    let app = host.app();
    app.settle(16).unwrap();
    let trigger = control(app, "Products");
    let engine = host
        .renderer()
        .engine_id(trigger, glory::html::Part::Main)
        .unwrap();
    assert!(app.dispatch(trigger, Event::Activate).is_err());
    assert_eq!(
        app.tree()
            .node(trigger)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .expanded,
        Some(false)
    );
    assert_eq!(
        host.renderer()
            .engine_id(trigger, glory::html::Part::Main)
            .unwrap(),
        engine
    );
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
}
