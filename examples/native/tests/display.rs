use glory::blitz::shell::WindowApp;
use glory::prelude::*;
use glory::render::{Event, NodeId, Props, Query, QueryValue};
use std::{cell::RefCell, rc::Rc};

#[path = "../../web/actions_form.rs"]
mod actions_form;
#[path = "../../web/display_form.rs"]
mod display_form;
#[path = "../../web/document_form.rs"]
mod document_form;

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
fn themed_surfaces_pills_and_formatted_text_use_native_layout_and_stable_identity() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(
            col()
                .style(Style::new().width(Length::Px(600.0)))
                .fill(display_form::display_form(theme)),
        )
        .unwrap();
        let app = host.app();
        app.settle(8).unwrap();
        for class in [
            "sapid-card",
            "sapid-paper",
            "sapid-empty-state",
            "number-us",
            "number-eur",
            "number-percent",
        ] {
            let rect = bounds(app, find(app, class));
            assert!(rect.width > 0.0 && rect.height > 0.0, "{class}: {rect:?}");
        }
        let formatter = find(app, "number-us");
        assert!(
            matches!(&app.tree().node(formatter).unwrap().descriptor.view().props.clone(),Props::Text(text) if text.content=="1,234.50")
        );
        let controlled = find(app, "controlled-pill");
        let physical = host
            .renderer()
            .engine_id(controlled, glory::html::Part::Main)
            .unwrap();
        let tree = app.tree();
        let remove = tree
            .node(controlled)
            .unwrap()
            .children
            .iter()
            .copied()
            .find(|id| {
                tree.node(*id)
                    .unwrap()
                    .descriptor
                    .view()
                    .classes
                    .iter()
                    .any(|c| c == "sapid-pill-remove")
            })
            .unwrap();
        drop(tree);
        app.dispatch(remove, Event::Activate).unwrap();
        app.settle(8).unwrap();
        assert!(
            !app.tree()
                .node(controlled)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .hidden
        );
        assert_eq!(
            host.renderer()
                .engine_id(controlled, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        let local = find(app, "local-pill");
        let tree = app.tree();
        let remove = tree
            .node(local)
            .unwrap()
            .children
            .iter()
            .copied()
            .find(|id| {
                tree.node(*id)
                    .unwrap()
                    .descriptor
                    .view()
                    .classes
                    .iter()
                    .any(|c| c == "sapid-pill-remove")
            })
            .unwrap();
        drop(tree);
        app.dispatch(remove, Event::Activate).unwrap();
        app.settle(8).unwrap();
        assert!(
            app.tree()
                .node(local)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .hidden
        );
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
}

#[test]
fn disclosure_and_named_actions_preserve_native_button_identity_and_metadata() {
    let host = WindowApp::new();
    host.mount(actions_form::actions_form(sapid::Theme::green_dark()))
        .unwrap();
    let app = host.app();
    app.settle(8).unwrap();
    for class in [
        "sapid-action-icon",
        "sapid-close-button",
        "sapid-burger",
        "sapid-button-group",
    ] {
        assert!(bounds(app, find(app, class)).width > 0.0, "{class}");
    }
    let local = find(app, "local-burger");
    let physical = host
        .renderer()
        .engine_id(local, glory::html::Part::Main)
        .unwrap();
    for open in [true, false] {
        app.dispatch(local, Event::Activate).unwrap();
        app.settle(8).unwrap();
        assert_eq!(
            app.tree()
                .node(local)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .expanded,
            Some(open)
        );
        assert_eq!(
            host.renderer()
                .engine_id(local, glory::html::Part::Main)
                .unwrap(),
            physical
        );
    }
    let controlled = find(app, "controlled-burger");
    app.dispatch(controlled, Event::Activate).unwrap();
    app.settle(8).unwrap();
    assert_eq!(
        app.tree()
            .node(controlled)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .expanded,
        Some(false)
    );
    app.clear().unwrap();
    assert!(app.tree().is_empty());
}

#[test]
fn document_widgets_keep_intrinsic_structure_and_inline_content_in_native_engine() {
    let host = WindowApp::new();
    host.mount(
        col()
            .style(Style::new().width(Length::Px(600.0)))
            .fill(document_form::document_form(sapid::Theme::blue_light())),
    )
    .unwrap();
    let app = host.app();
    app.settle(8).unwrap();
    for class in [
        "sapid-text-list",
        "sapid-blockquote",
        "sapid-data-list",
        "sapid-code",
        "sapid-kbd",
        "sapid-mark",
        "sapid-item",
    ] {
        let rect = bounds(app, find(app, class));
        assert!(rect.width > 0.0 && rect.height > 0.0, "{class}: {rect:?}");
    }
    let code = find(app, "document-code");
    let physical = host
        .renderer()
        .engine_id(code, glory::html::Part::Main)
        .unwrap();
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut update = None;
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if matches!(&node.descriptor.view().props.clone(),Props::Button(b) if b.label=="Update code source")
        {
            update = Some(id);
            break;
        }
        pending.extend(node.children.iter().copied());
    }
    drop(tree);
    app.dispatch(update.unwrap(), Event::Activate).unwrap();
    app.settle(8).unwrap();
    assert!(
        matches!(&app.tree().node(code).unwrap().descriptor.view().props.clone(),Props::Text(text) if text.content=="let sapid = 42;")
    );
    assert_eq!(
        host.renderer()
            .engine_id(code, glory::html::Part::Main)
            .unwrap(),
        physical
    );
    app.clear().unwrap();
    assert!(app.tree().is_empty());
}
