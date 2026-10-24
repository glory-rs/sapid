use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{
    Action, DocumentPart, NodeId, Props, Query, QueryValue, Rect, TableMetadata,
};
use std::{cell::RefCell, rc::Rc};
#[path = "../../web/src/table_form.rs"]
mod fixture;
fn query(app: &App, q: Query) -> QueryValue {
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    app.request(q, move |v| *saved.borrow_mut() = Some(v.unwrap()))
        .unwrap();
    app.pump().unwrap();
    out.borrow_mut().take().unwrap()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let QueryValue::Bounds(value) = query(app, Query::Bounds(id)) else {
        panic!("bounds")
    };
    value
}
#[test]
fn four_theme_native_tables_align_real_cells_and_preserve_editing_identity() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(860, 700, 1.).unwrap();
        host.mount(fixture::table_form(theme)).unwrap();
        let app = host.app();
        app.settle(16).unwrap();
        let tree = app.tree();
        let mut todo = tree.roots().to_vec();
        let mut cells = Vec::new();
        let mut input = None;
        let mut button = None;
        while let Some(id) = todo.pop() {
            let n = tree.node(id).unwrap();
            let d = n.descriptor.view();
            if d.classes.iter().any(|c| c == "table-span-example") {
                continue;
            }
            if let Some(TableMetadata::Cell { row, column, .. }) =
                d.semantics.as_ref().and_then(|s| s.table.as_ref())
            {
                cells.push((*row, *column, id));
            }
            if matches!(d.props, Props::Input(_)) {
                input = Some(id)
            }
            if matches!(d.props, Props::Button(_)) {
                button = Some(id)
            }
            todo.extend(&n.children);
        }
        drop(tree);
        assert_eq!(cells.len(), 15);
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        assert!(raster.pixels().as_chunks::<4>().0.iter().any(|v| v[3] > 0));
        let mut columns = Vec::new();
        for column in 0..3 {
            let rects = cells
                .iter()
                .filter(|(_, c, _)| *c == column)
                .map(|(_, _, id)| bounds(app, *id))
                .collect::<Vec<_>>();
            assert!(
                rects
                    .iter()
                    .all(|rect| rect.width > 50. && rect.height > 25.)
            );
            assert!(rects.iter().all(|rect| (rect.x - rects[0].x).abs() < 1.
                && (rect.width - rects[0].width).abs() < 1.));
            columns.push(rects[0]);
        }
        assert!(columns[0].x + columns[0].width <= columns[1].x + 1.);
        assert!(columns[1].x + columns[1].width <= columns[2].x + 1.);
        let input = input.unwrap();
        let engine = host
            .renderer()
            .engine_id(input, glory::html::Part::Main)
            .unwrap();
        app.scope().action(Action::Focus(input)).unwrap();
        app.settle(16).unwrap();
        assert_eq!(query(app, Query::Focus(input)), QueryValue::Focus(true));
        app.dispatch(button.unwrap(), glory::render::Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert_eq!(
            host.renderer()
                .engine_id(input, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        assert_eq!(query(app, Query::Focus(input)), QueryValue::Focus(true));
        let row = cells.iter().find(|(r, c, _)| *r == 1 && *c == 0).unwrap().2;
        assert_eq!(
            app.tree()
                .node(row)
                .unwrap()
                .descriptor
                .view()
                .document_part(),
            Some(DocumentPart::RowHeader)
        );
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(app.stats().queries, 0);
    }
}
#[test]
fn native_row_and_column_spans_occupy_real_grid_tracks() {
    let host = WindowApp::new();
    host.renderer().set_viewport(860, 700, 1.).unwrap();
    host.mount(fixture::span_table(sapid::Theme::blue_light()))
        .unwrap();
    let app = host.app();
    app.settle(16).unwrap();
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut cells = Vec::new();
    while let Some(id) = pending.pop() {
        let n = tree.node(id).unwrap();
        if let Some(TableMetadata::Cell { row, column, .. }) = n
            .descriptor
            .view()
            .semantics
            .as_ref()
            .and_then(|s| s.table.as_ref())
        {
            cells.push((*row, *column, id));
        }
        pending.extend(&n.children);
    }
    drop(tree);
    let cell = |row, column| {
        bounds(
            app,
            cells
                .iter()
                .find(|(r, c, _)| *r == row && *c == column)
                .unwrap()
                .2,
        )
    };
    let group = cell(0, 0);
    let left = cell(1, 0);
    let right = cell(1, 1);
    assert!((group.x - left.x).abs() < 1.);
    assert!((group.x + group.width - right.x - right.width).abs() < 1.);
    assert!(group.width > left.width + right.width);
    let span = cell(2, 0);
    let first = cell(2, 1);
    let second = cell(3, 1);
    assert!((first.x - second.x).abs() < 1.);
    assert!((span.y - first.y).abs() < 1.);
    assert!((span.y + span.height - second.y - second.height).abs() < 1.);
    assert!(span.height > first.height + second.height);
}
