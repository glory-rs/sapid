use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, Current, NodeId, Query, QueryValue, Rect, ScrollMetrics};
use std::{cell::RefCell, rc::Rc};
fn settle(host: &WindowApp) {
    for _ in 0..8 {
        host.observe(&std::task::Context::from_waker(std::task::Waker::noop()))
            .unwrap();
    }
}
fn value(app: &App, query: Query) -> QueryValue {
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    app.request(query, move |v| *saved.borrow_mut() = Some(v.unwrap()))
        .unwrap();
    app.pump().unwrap();
    out.borrow_mut().take().unwrap()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let QueryValue::Bounds(r) = value(app, Query::Bounds(id)) else {
        panic!("bounds")
    };
    r
}
fn metrics(app: &App, id: NodeId) -> ScrollMetrics {
    let QueryValue::ScrollMetrics(r) = value(app, Query::ScrollMetrics(id)) else {
        panic!("metrics")
    };
    r
}
#[test]
fn four_theme_native_index_uses_actual_scroll_geometry_and_preserves_focus_and_engine_nodes() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let ctx = sapid::toc_context(app.scope(), "native").unwrap();
        let viewport = app.scope().cage(None);
        let heading = app.scope().cage(None);
        let nav = app.scope().cage(None);
        let draft = app.scope().cage(None);
        let label = app.scope().cage("Middle".to_owned());
        let order = app.scope().cage(1u32);
        let t = theme.clone();
        host.mount(
            sapid::stack().fill((
                sapid::text_field()
                    .theme(theme.clone())
                    .name("Kept draft")
                    .node_ref(draft)
                    .default_value("Caret"),
                sapid::table_of_contents(ctx.clone(), "nav")
                    .theme(theme.clone())
                    .node_ref(nav),
                sapid::toc_document(ctx.clone(), "doc")
                    .theme(theme)
                    .offset(6.)
                    .node_ref(viewport)
                    .style(
                        Style::new()
                            .width(Length::Px(320.))
                            .height(Length::Px(140.)),
                    )
                    .content(move |ctx| {
                        glory::ui::col().fill((
                            sapid::toc_heading(ctx.clone(), "a", 1)
                                .content("Start")
                                .theme(t.clone())
                                .order(0),
                            glory::ui::view().style(Style {
                                height: Some(Length::Px(260.)),
                                shrink: Some(0.),
                                ..Style::new()
                            }),
                            sapid::toc_heading(ctx.clone(), "b", 3)
                                .content(label)
                                .theme(t.clone())
                                .order(order)
                                .node_ref(heading),
                            glory::ui::view().style(Style {
                                height: Some(Length::Px(260.)),
                                shrink: Some(0.),
                                ..Style::new()
                            }),
                            sapid::toc_heading(ctx, "c", 2)
                                .content("Last")
                                .theme(t.clone())
                                .order(2),
                            glory::ui::view().style(Style {
                                height: Some(Length::Px(200.)),
                                shrink: Some(0.),
                                ..Style::new()
                            }),
                        ))
                    }),
            )),
        )
        .unwrap();
        settle(&host);
        let viewport = viewport.get().unwrap();
        let heading = heading.get().unwrap();
        let draft = draft.get().unwrap();
        let current = ctx.current(app.scope());
        assert_eq!(
            current.get(),
            Some(ctx.entries(app.scope()).get()[0].id.clone())
        );
        let entry = ctx.entries(app.scope()).get()[1].clone();
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        assert!(raster.pixels().as_chunks::<4>().0.iter().any(|p| p[3] > 0));
        let link = {
            let tree = app.tree();
            let mut todo = tree.roots().to_vec();
            let mut found = None;
            while let Some(id) = todo.pop() {
                let node = tree.node(id).unwrap();
                if let glory::render::Props::Link(link) = node.descriptor.view().props
                    && link.target == format!("#{}", entry.id.as_str())
                {
                    found = Some(id);
                    break;
                }
                todo.extend(&node.children);
            }
            found.unwrap()
        };
        app.scope().action(Action::Focus(link)).unwrap();
        settle(&host);
        order.revise(|v| *v = 4);
        settle(&host);
        assert_eq!(value(app, Query::Focus(link)), QueryValue::Focus(true));
        let engine = host
            .renderer()
            .engine_id(heading, glory::html::Part::Main)
            .unwrap();
        app.scope().action(Action::Focus(draft)).unwrap();
        settle(&host);
        assert_eq!(value(app, Query::Focus(draft)), QueryValue::Focus(true));
        ctx.reveal(&entry.id).unwrap();
        settle(&host);
        let moved = metrics(app, viewport);
        assert!(moved.y > 200., "{moved:?}");
        let v = bounds(app, viewport);
        let h = bounds(app, heading);
        assert!((h.y - v.y - 6.).abs() < 2., "{h:?} {v:?}");
        assert_eq!(value(app, Query::Focus(draft)), QueryValue::Focus(true));
        ctx.refresh().unwrap();
        settle(&host);
        assert_eq!(current.get(), Some(entry.id));
        label.revise(|v| *v = "Middle revised".into());
        settle(&host);
        assert_eq!(
            host.renderer()
                .engine_id(heading, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        let tree = app.tree();
        let mut todo = tree.roots().to_vec();
        let mut count = 0;
        while let Some(id) = todo.pop() {
            let n = tree.node(id).unwrap();
            if n.descriptor
                .view()
                .semantics
                .as_ref()
                .is_some_and(|s| s.current == Some(Current::Location))
            {
                count += 1
            }
            todo.extend(&n.children);
        }
        drop(tree);
        assert_eq!(count, 1);
        app.clear().unwrap();
        settle(&host);
        assert!(ctx.is_empty());
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(app.stats().queries, 0);
    }
}
