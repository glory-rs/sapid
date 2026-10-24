use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{
    Action, Event, NodeId, Props, Query, QueryValue, Rect, Role, TableMetadata,
};
use std::{cell::RefCell, rc::Rc};
#[path = "../../web/data_table_form.rs"]
mod fixture;
fn all(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut out = Vec::new();
    while let Some(id) = todo.pop() {
        let n = tree.node(id).unwrap();
        out.push(id);
        todo.extend(&n.children);
    }
    out
}
fn named(app: &App, name: &str) -> NodeId {
    all(app)
        .into_iter()
        .find(|id| {
            let tree = app.tree();
            let d = tree.node(*id).unwrap().descriptor.view();
            d.name.as_deref() == Some(name)
                || matches!(d.props,Props::Button(p) if p.label==name)
        })
        .unwrap_or_else(|| panic!("missing {name}"))
}
fn query(app: &App, q: Query) -> QueryValue {
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    app.request(q, move |v| *saved.borrow_mut() = Some(v.unwrap()))
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
fn observe(host: &WindowApp) {
    for _ in 0..12 {
        host.observe(&std::task::Context::from_waker(std::task::Waker::noop()))
            .unwrap();
    }
}
#[test]
fn four_theme_native_data_tables_have_real_tracks_bounded_windows_and_focused_row_identity() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(860, 950, 1.).unwrap();
        host.mount(fixture::data_table_form(theme)).unwrap();
        observe(&host);
        let app = host.app();
        let editor = named(app, "Note for r0000");
        let engine = host
            .renderer()
            .engine_id(editor, glory::html::Part::Main)
            .unwrap();
        app.scope().action(Action::Focus(editor)).unwrap();
        observe(&host);
        assert_eq!(query(app, Query::Focus(editor)), QueryValue::Focus(true));
        let table = named(app, "Project people");
        let mut cells = Vec::new();
        for id in all(app) {
            let tree = app.tree();
            let d = tree.node(id).unwrap().descriptor.view();
            if let Some(TableMetadata::Cell { row, column, .. }) =
                d.semantics.as_ref().and_then(|s| s.table.as_ref())
            {
                cells.push((*row, *column, id));
            }
        }
        for column in 1..4 {
            let header = bounds(
                app,
                cells
                    .iter()
                    .find(|(r, c, _)| *r == 0 && *c == column)
                    .unwrap()
                    .2,
            );
            let row = bounds(
                app,
                cells
                    .iter()
                    .find(|(r, c, _)| *r == 1 && *c == column)
                    .unwrap()
                    .2,
            );
            assert!((header.x - row.x).abs() < 1.);
            assert!((header.width - row.width).abs() < 1.);
            assert!(row.height >= 44.);
        }
        assert!(bounds(app, table).height > 260.);
        let list_viewport = all(app)
            .into_iter()
            .find(|id| {
                app.tree()
                    .node(*id)
                    .unwrap()
                    .descriptor
                    .view()
                    .style
                    .overflow_y
                    == Some(glory::style::Overflow::Auto)
            })
            .unwrap();
        app.scope()
            .action(Action::ScrollTo {
                node: list_viewport,
                x: 0.,
                y: 8800.,
            })
            .unwrap();
        observe(&host);
        let live = all(app)
            .into_iter()
            .filter(|id| {
                app.tree().node(*id).unwrap().descriptor.view().role == Some(Role::Row)
            })
            .count();
        assert!(live <= 13, "unbounded mounted rows: {live}");
        assert_eq!(
            host.renderer()
                .engine_id(editor, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        assert_eq!(query(app, Query::Focus(editor)), QueryValue::Focus(true));
        let QueryValue::ScrollMetrics(metrics) =
            query(app, Query::ScrollMetrics(list_viewport))
        else {
            panic!("metrics")
        };
        assert!(metrics.y > 1000.);
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        assert!(raster.pixels().as_chunks::<4>().0.iter().any(|p| p[3] > 0));
        app.dispatch(named(app, "Sort by Name"), Event::Activate)
            .unwrap();
        observe(&host);
        assert_eq!(
            host.renderer()
                .engine_id(editor, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(app.stats().queries, 0);
    }
}
