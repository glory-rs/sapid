use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, Event, NodeId, Props, Query, QueryValue};
use std::{cell::RefCell, rc::Rc};
fn nodes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = todo.pop() {
        let n = tree.node(id).unwrap();
        if n.descriptor.view().classes.iter().any(|c| c == class) {
            out.push(id)
        }
        todo.extend(n.children.iter().rev());
    }
    out
}
fn trigger(app: &App, label: &str) -> NodeId {
    nodes(app,"sapid-collapsible-trigger").into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(button) if button.label==label)).unwrap()
}
fn query(app: &App, query: Query) -> QueryValue {
    let reply = Rc::new(RefCell::new(None));
    let output = reply.clone();
    app.request(query, move |value| {
        *output.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    reply.borrow_mut().take().unwrap()
}
#[test]
fn native_json_disclosures_four_theme_geometry_pixels_focus_search_and_owner_retirement() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(500, 350, 1.).unwrap();
        let app = host.app();
        let query_text = app.scope().cage(String::new());
        host.mount(
            sapid::json_viewer("native")
                .theme(theme)
                .source("{\"branch\":{\"leaf\":\"😀 <script>\"},\"a\":12345678901234567890}")
                .search(query_text),
        )
        .unwrap();
        let branch = trigger(app, "branch: Object 1");
        let physical = host
            .renderer()
            .engine_id(branch, glory::html::Part::Main)
            .unwrap();
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let initial = raster.pixels().to_vec();
        app.scope().action(Action::Focus(branch)).unwrap();
        app.flush().unwrap();
        assert_eq!(query(app, Query::Focus(branch)), QueryValue::Focus(true));
        app.dispatch(branch, Event::Activate).unwrap();
        app.flush().unwrap();
        assert_eq!(nodes(app, "sapid-json-value").len(), 2);
        let QueryValue::Bounds(rect) =
            query(app, Query::Bounds(nodes(app, "sapid-json-value")[0]))
        else {
            panic!()
        };
        assert!(rect.width > 10. && rect.height > 10.);
        raster.draw(host.renderer()).unwrap();
        assert_ne!(raster.pixels(), initial);
        assert_eq!(
            host.renderer()
                .engine_id(branch, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        app.dispatch(branch, Event::Activate).unwrap();
        app.flush().unwrap();
        assert_eq!(nodes(app, "sapid-json-value").len(), 1);
        query_text.revise(|q| *q = "leaf".into());
        app.flush().unwrap();
        assert_eq!(nodes(app, "sapid-json-value").len(), 2);
        query_text.revise(|q| q.clear());
        app.flush().unwrap();
        assert_eq!(nodes(app, "sapid-json-value").len(), 1);
        app.clear().unwrap();
        assert_eq!(app.runtime_stats().tasks, 0);
        assert_eq!(app.stats().queries, 0);
    }
}
