use glory::blitz::shell::WindowApp;
use glory::prelude::*;
use glory::render::{Action, Event, Navigation, NodeId, Props, Query, QueryValue};
use std::{cell::RefCell, rc::Rc};
#[path = "../../web/actions_form.rs"]
mod fixture;
fn button(app: &App, label: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if matches!(node.descriptor.view().props,Props::Button(p) if p.label==label) {
            return id;
        }
        pending.extend(node.children.iter().copied())
    }
    panic!("missing {label}")
}
fn focused(app: &App, id: NodeId) -> bool {
    let result = Rc::new(RefCell::new(None));
    let out = result.clone();
    app.request(Query::Focus(id), move |value| {
        *out.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Focus(value) = result.borrow_mut().take().unwrap() else {
        panic!("focus")
    };
    value
}
#[test]
fn native_toolbar_moves_actual_focus_without_activation_and_retains_engine_buttons() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(fixture::actions_form(theme)).unwrap();
        let app = host.app();
        app.settle(8).unwrap();
        let first = button(app, "Toolbar first");
        let last = button(app, "Toolbar last");
        let physical = host
            .renderer()
            .engine_id(first, glory::html::Part::Main)
            .unwrap();
        app.scope().action(Action::Focus(first)).unwrap();
        app.settle(8).unwrap();
        assert!(focused(app, first));
        for (from, command, to) in [
            (first, Navigation::Next, last),
            (last, Navigation::Next, first),
            (first, Navigation::Last, last),
        ] {
            app.dispatch(from, Event::Navigate { from, command })
                .unwrap();
            app.settle(8).unwrap();
            assert!(focused(app, to));
        }
        assert_eq!(
            host.renderer()
                .engine_id(first, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        let tree = app.tree();
        let mut pending = tree.roots().to_vec();
        let mut found = false;
        while let Some(id) = pending.pop() {
            let node = tree.node(id).unwrap();
            if matches!(node.descriptor.view().props,Props::Text(p) if p.content=="Toolbar activations: 0")
            {
                found = true;
            }
            pending.extend(node.children.iter().copied())
        }
        assert!(found);
        drop(tree);
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
}
