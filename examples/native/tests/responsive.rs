use glory::blitz::shell::WindowApp;
use glory::render::{Event, NodeId, Props, Query, QueryValue, Rect};
use glory::ui::App;
use std::{cell::RefCell, rc::Rc};
#[path = "../../web/src/responsive_form.rs"]
mod fixture;
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
fn class(app: &App, name: &str) -> NodeId {
    find(app, |node| node.classes.iter().any(|class| class == name))
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
fn native_responsive_root_updates_center_container_space_and_both_grids_with_stable_nodes() {
    let host = WindowApp::new();
    host.mount(fixture::responsive_form(sapid::Theme::blue_light()))
        .unwrap();
    let app = host.app();
    app.settle(12).unwrap();
    let center = class(app, "responsive-center");
    let child = class(app, "responsive-center-child");
    let space = class(app, "responsive-space");
    let grid = class(app, "responsive-grid");
    let simple = class(app, "responsive-simple");
    let container = class(app, "responsive-container");
    let ids = [center, child, space, grid, simple, container];
    assert!((bounds(app, space).height - 10.).abs() < 0.1);
    assert!((bounds(app, child).y - bounds(app, center).y).abs() < 0.1);
    let wide = find(
        app,
        |node| matches!(node.props,Props::Button(button) if button.label=="Wide container"),
    );
    app.dispatch(wide, Event::Activate).unwrap();
    app.settle(12).unwrap();
    assert!((bounds(app, space).height - 40.).abs() < 0.1);
    assert!((bounds(app, child).y - bounds(app, center).y - 35.).abs() < 0.1);
    assert_eq!(
        app.tree().node(grid).unwrap().descriptor.view().style.gap,
        Some(16.)
    );
    assert_eq!(
        app.tree().node(simple).unwrap().descriptor.view().style.gap,
        Some(12.)
    );
    assert_eq!(
        app.tree()
            .node(container)
            .unwrap()
            .descriptor
            .view()
            .style
            .padding,
        Some(glory::style::Edges::all(16.))
    );
    for (id, name) in ids.into_iter().zip([
        "responsive-center",
        "responsive-center-child",
        "responsive-space",
        "responsive-grid",
        "responsive-simple",
        "responsive-container",
    ]) {
        assert_eq!(id, class(app, name));
    }
    assert_eq!(app.stats().queries, 0);
}
