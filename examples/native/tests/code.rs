use glory::blitz::{Config, Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, NodeId, Query, QueryValue};
use std::{cell::RefCell, rc::Rc};
fn ids(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut out = Vec::new();
    while let Some(id) = todo.pop() {
        let n = tree.node(id).unwrap();
        if n.descriptor.view().classes.iter().any(|c| c == class) {
            out.push(id)
        }
        todo.extend(n.children.iter().rev());
    }
    out
}
fn query(app: &App, query: Query) -> QueryValue {
    let reply = Rc::new(RefCell::new(None));
    let receive = reply.clone();
    app.request(query, move |result| {
        *receive.borrow_mut() = Some(result.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    reply.borrow_mut().take().unwrap()
}
#[test]
fn four_theme_native_code_has_real_multiline_geometry_colors_scroll_and_identity() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::with_config(Config::default());
        host.renderer().set_viewport(360, 260, 1.).unwrap();
        let app = host.app();
        let source = app
            .scope()
            .cage("fn main() {\n  let count = 42; // 😀\n}\n".to_string());
        let input = app.scope().cage(None);
        host.mount(
            glory::ui::col().fill((
                sapid::code_highlight()
                    .theme(theme)
                    .source(source)
                    .line_numbers(true)
                    .highlighted_lines([2])
                    .style(Style::new().width(Length::Px(260.))),
                glory::ui::input()
                    .node_ref(input)
                    .value("Native code draft"),
            )),
        )
        .unwrap();
        let first = ids(app, "sapid-code-token")[0];
        let physical = host
            .renderer()
            .engine_id(first, glory::html::Part::Main)
            .unwrap();
        let QueryValue::Bounds(body) =
            query(app, Query::Bounds(ids(app, "sapid-code-source")[0]))
        else {
            panic!()
        };
        assert!(body.height >= 50., "multiline height {}", body.height);
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let before = raster.pixels().to_vec();
        app.scope()
            .action(Action::Focus(input.get().unwrap()))
            .unwrap();
        app.flush().unwrap();
        source
            .revise(|s| *s = format!("fn other() {{\n  // {}\n}}\n", "long text ".repeat(50)));
        app.flush().unwrap();
        assert_eq!(
            host.renderer()
                .engine_id(first, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        let viewport = ids(app, "sapid-code-scroll")[0];
        let QueryValue::ScrollMetrics(metrics) = query(app, Query::ScrollMetrics(viewport))
        else {
            panic!()
        };
        assert!(
            metrics.content_width > metrics.viewport_width + 100.,
            "{metrics:?}"
        );
        assert_eq!(
            query(app, Query::Focus(input.get().unwrap())),
            QueryValue::Focus(true)
        );
        raster.draw(host.renderer()).unwrap();
        assert_ne!(raster.pixels(), before);
        app.clear().unwrap();
        assert_eq!(app.runtime_stats().tasks, 0);
        assert_eq!(app.stats().queries, 0);
    }
}
