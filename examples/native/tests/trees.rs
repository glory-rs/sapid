use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, Event, NodeId, Query, QueryValue, Rect, Role, TreeCommand};
use std::{cell::RefCell, rc::Rc};
#[path = "../../web/tree_form.rs"]
mod fixture;
fn all(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut ids = vec![];
    while let Some(id) = pending.pop() {
        pending.extend(&tree.node(id).unwrap().children);
        ids.push(id);
    }
    ids
}
fn named(app: &App, name: &str) -> NodeId {
    all(app)
        .into_iter()
        .find(|id| {
            let tree = app.tree();
            let d = tree.node(*id).unwrap().descriptor.view();
            d.name.as_deref() == Some(name)
                || matches!(d.props,glory::render::Props::Button(button) if button.label==name)
        })
        .unwrap_or_else(|| panic!("missing {name}"))
}
fn class(app: &App, name: &str) -> NodeId {
    all(app)
        .into_iter()
        .find(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|class| class == name)
        })
        .unwrap_or_else(|| panic!("missing class {name}"))
}
fn query(app: &App, query: Query) -> QueryValue {
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    app.request(query, move |value| {
        *saved.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    out.borrow_mut().take().unwrap()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let QueryValue::Bounds(rect) = query(app, Query::Bounds(id)) else {
        panic!("bounds")
    };
    rect
}
fn observe(app: &App) {
    app.settle(32).unwrap();
}
fn navigate(app: &App, root: NodeId, from: NodeId, command: TreeCommand) {
    app.dispatch(root, Event::TreeNavigate { from, command })
        .unwrap();
    observe(app);
}
#[test]
fn native_four_theme_tree_has_real_indent_focus_selection_pixels_and_stable_custom_editor() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(960, 1000, 1.).unwrap();
        host.mount(fixture::tree_form(theme.clone())).unwrap();
        let app = host.app();
        observe(app);
        let root = named(app, "Project files");
        let source = named(app, "Source");
        let main = named(app, "Main file");
        let library = named(app, "Library file");
        let editor = named(app, "Tree editor");
        let source_bounds = bounds(app, source);
        let main_bounds = bounds(app, main);
        assert!(source_bounds.height >= 32. && main_bounds.height >= 36.);
        assert!(main_bounds.width > 300. && main_bounds.y > source_bounds.y);
        let label = all(app)
            .into_iter()
            .find(|id| {
                let tree = app.tree();
                let d = tree.node(*id).unwrap().descriptor.view();
                matches!(d.props,glory::render::Props::Text(p) if p.content=="Main file")
            })
            .unwrap();
        assert!(bounds(app, label).x > source_bounds.x + 20.);
        let engine = host
            .renderer()
            .engine_id(editor, glory::html::Part::Main)
            .unwrap();
        app.scope().action(Action::Focus(editor)).unwrap();
        app.flush().unwrap();
        observe(app);
        assert_eq!(query(app, Query::Focus(editor)), QueryValue::Focus(true));
        app.dispatch(editor, Event::Activate).unwrap();
        observe(app);
        assert_eq!(query(app, Query::Focus(editor)), QueryValue::Focus(true));
        app.dispatch(named(app, "Reorder tree files"), Event::Activate)
            .unwrap();
        observe(app);
        assert_eq!(named(app, "Tree editor"), editor);
        assert_eq!(
            host.renderer()
                .engine_id(editor, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        assert_eq!(query(app, Query::Focus(editor)), QueryValue::Focus(true));
        app.scope().action(Action::Focus(main)).unwrap();
        app.flush().unwrap();
        observe(app);
        navigate(app, root, main, TreeCommand::Previous);
        assert_eq!(query(app, Query::Focus(library)), QueryValue::Focus(true));
        navigate(app, root, library, TreeCommand::Select);
        let selected = bounds(app, library);
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let frame = raster.frame().unwrap();
        let offset = ((selected.y + 3.).round() as usize * frame.width as usize
            + (selected.x + 12.).round() as usize)
            * 4;
        let expected = theme.accent_soft;
        for (a, b) in raster.pixels()[offset..offset + 4]
            .iter()
            .zip([expected.0, expected.1, expected.2, expected.3])
        {
            assert!(a.abs_diff(b) <= 2);
        }
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(app.stats().queries, 0);
    }
}
#[test]
fn native_tree_reveal_scrolls_current_viewport_lazy_loads_and_retires_descendant_owners() {
    let host = WindowApp::new();
    host.renderer().set_viewport(920, 1200, 1.).unwrap();
    host.mount(fixture::tree_form(sapid::Theme::green_dark()))
        .unwrap();
    let app = host.app();
    observe(app);
    let root = named(app, "Project files");
    let docs = named(app, "Documents");
    navigate(app, root, docs, TreeCommand::Open);
    let last = named(app, "Document 95");
    app.scope().action(Action::Focus(docs)).unwrap();
    app.flush().unwrap();
    navigate(app, root, docs, TreeCommand::Next);
    navigate(app, root, named(app, "Document 00"), TreeCommand::Last);
    assert_eq!(
        query(app, Query::Focus(named(app, "Remote files"))),
        QueryValue::Focus(true)
    );
    let QueryValue::ScrollMetrics(metrics) = query(app, Query::ScrollMetrics(root)) else {
        panic!("metrics")
    };
    assert!(metrics.y > 1000.);
    let remote = named(app, "Remote files");
    navigate(app, root, remote, TreeCommand::Open);
    app.dispatch(named(app, "Publish lazy files"), Event::Activate)
        .unwrap();
    observe(app);
    assert!(bounds(app, named(app, "Loaded file")).height >= 32.);
    app.dispatch(named(app, "Collapse all tree branches"), Event::Activate)
        .unwrap();
    observe(app);
    assert!(app.tree().node(last).is_err());
    assert_eq!(
        app.tree().node(root).unwrap().descriptor.view().role,
        Some(Role::Tree)
    );
    app.dispatch(named(app, "Toggle tree Owner"), Event::Activate)
        .unwrap();
    observe(app);
    assert!(app.tree().node(root).is_err());
    assert_eq!(app.stats().queries, 0);
    assert!(app.tree().node(class(app, "tree-selected")).is_ok());
}
