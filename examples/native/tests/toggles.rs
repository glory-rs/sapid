use glory::blitz::shell::WindowApp;
use glory::prelude::*;
use glory::render::{Action, Event, FocusMode, NodeId, Query, QueryValue};
use sapid::{ToggleMode, toggle, toggle_group, toggle_item};
use std::{cell::RefCell, rc::Rc};

fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            return id;
        }
        pending.extend(node.children.iter().copied());
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
        panic!("bounds expected")
    };
    rect
}
#[test]
fn toggles_preserve_native_engine_identity_and_controlled_values_in_four_themes() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let value = host.app().scope().cage(false);
        host.mount(
            col().style(Style::new().width(Length::Px(480.0))).fill((
                toggle()
                    .theme(theme.clone())
                    .class("controlled")
                    .label("Bold")
                    .pressed(value),
                toggle_group("native-format")
                    .theme(theme)
                    .mode(ToggleMode::Multiple)
                    .default_value(["italic"])
                    .items([
                        toggle_item("italic").class("italic").label("Italic"),
                        toggle_item("underline")
                            .class("underline")
                            .label("Underline"),
                    ]),
            )),
        )
        .unwrap();
        let app = host.app();
        app.settle(12).unwrap();
        let id = find(app, "controlled");
        let physical = host
            .renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap();
        assert!(bounds(app, id).width > 0.0 && bounds(app, id).height > 0.0);
        app.dispatch(id, Event::Activate).unwrap();
        app.settle(12).unwrap();
        assert!(
            !app.tree()
                .node(id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .pressed
                .unwrap()
        );
        value.revise(|v| *v = true);
        app.settle(12).unwrap();
        assert!(
            app.tree()
                .node(id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .pressed
                .unwrap()
        );
        assert_eq!(
            host.renderer()
                .engine_id(id, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        let underline = find(app, "underline");
        app.scope().action(Action::Focus(underline)).unwrap();
        app.settle(12).unwrap();
        assert_eq!(
            app.tree().node(underline).unwrap().descriptor.view().focus,
            FocusMode::TabStop
        );
        assert!(
            !app.tree()
                .node(underline)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .pressed
                .unwrap()
        );
        app.dispatch(underline, Event::Activate).unwrap();
        app.settle(12).unwrap();
        assert!(
            app.tree()
                .node(underline)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .pressed
                .unwrap()
        );
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
}
