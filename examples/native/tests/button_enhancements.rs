#[path = "../../web/src/button_form.rs"]
mod fixture;
use glory::blitz::shell::WindowApp;
use glory::render::{Event, NodeId, Props, Query, QueryValue, Rect};
use glory::ui::App;
use std::{cell::RefCell, rc::Rc};
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            return id;
        }
        pending.extend(&node.children);
    }
    panic!("missing {class}");
}
fn button(app: &App, label: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if matches!(node.descriptor.view().props,Props::Button(p) if p.label==label) {
            return id;
        }
        pending.extend(&node.children);
    }
    panic!("missing {label}");
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
fn native_section_layout_sizes_loading_and_live_labels_preserve_controls() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(fixture::button_form(theme)).unwrap();
        let app = host.app();
        app.settle(12).unwrap();
        let id = find(app, "section-button");
        let label = {
            let tree = app.tree();
            *tree
                .node(id)
                .unwrap()
                .children
                .iter()
                .find(|child| {
                    tree.node(**child)
                        .unwrap()
                        .descriptor
                        .view()
                        .classes
                        .iter()
                        .any(|class| class == "sapid-button-label")
                })
                .unwrap()
        };
        let engine = host
            .renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap();
        assert!(bounds(app, find(app, "button-xs")).height >= 24.0);
        assert!(bounds(app, find(app, "button-xl")).height >= 50.0);
        let text_ids = {
            let tree = app.tree();
            let mut all = Vec::new();
            let mut pending = tree.node(id).unwrap().children.to_vec();
            while let Some(id) = pending.pop() {
                let n = tree.node(id).unwrap();
                if matches!(n.descriptor.view().props, Props::Text(_)) {
                    all.push(id);
                }
                pending.extend(&n.children);
            }
            all
        };
        let mut rects = text_ids
            .iter()
            .map(|id| bounds(app, *id))
            .collect::<Vec<_>>();
        rects.sort_by(|a, b| a.x.total_cmp(&b.x));
        assert_eq!(rects.len(), 3);
        assert!(
            (rects[1].x - rects[0].x - rects[0].width - 8.0).abs() < 1.0,
            "{rects:?}"
        );
        assert!(
            (rects[2].x - rects[1].x - rects[1].width - 8.0).abs() < 1.0,
            "{rects:?}"
        );
        app.dispatch(button(app, "Toggle button loading"), Event::Activate)
            .unwrap();
        app.settle(12).unwrap();
        assert_eq!(
            app.tree()
                .node(id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .busy,
            Some(true)
        );
        assert!(
            matches!(app.tree().node(id).unwrap().descriptor.view().props,Props::Button(p) if !p.enabled)
        );
        app.dispatch(button(app, "Update button label"), Event::Activate)
            .unwrap();
        app.settle(12).unwrap();
        assert!(
            matches!(app.tree().node(label).unwrap().descriptor.view().props,Props::Text(p) if p.content=="Save changes")
        );
        assert_eq!(
            app.tree()
                .node(id)
                .unwrap()
                .descriptor
                .view()
                .name
                .as_deref(),
            Some("Save changes")
        );
        assert_eq!(
            host.renderer()
                .engine_id(id, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
}
