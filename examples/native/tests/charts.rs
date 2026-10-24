use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, NodeId, Props, Query, QueryValue, Rect};
use sapid::charts::*;
use std::{cell::RefCell, rc::Rc};
fn all(app: &App) -> Vec<NodeId> {
    let t = app.tree();
    let mut pending = t.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = pending.pop() {
        pending.extend(&t.node(id).unwrap().children);
        out.push(id);
    }
    out
}
fn find(app: &App, class: &str) -> NodeId {
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
                .any(|c| c == class)
        })
        .unwrap()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let r = Rc::new(RefCell::new(None));
    let out = r.clone();
    app.request(Query::Bounds(id), move |v| {
        *out.borrow_mut() = Some(v.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = r.borrow_mut().take().unwrap() else {
        panic!()
    };
    rect
}
#[test]
fn four_theme_cartesian_charts_have_real_native_geometry_vector_pixels_hit_controls_and_cleanup()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let d = ChartData {
            categories: vec![
                ChartCategory::new("a", "January"),
                ChartCategory::new("b", "February"),
            ],
            series: vec![ChartSeries::new("s", "Revenue", [Some(2.), Some(4.)])],
        };
        host.mount(
            bar_chart("native")
                .theme(theme.clone())
                .data(d)
                .style(Style::new().width(Length::Px(640.))),
        )
        .unwrap();
        app.settle(16).unwrap();
        let plot = find(app, "sapid-chart-plot");
        let rect = bounds(app, plot);
        assert!(
            (rect.width - 640.).abs() < 0.5 && (rect.height - 320.).abs() < 0.5,
            "{rect:?}"
        );
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let color = ChartTheme::from_theme(&theme).series[0];
        assert!(
            raster
                .pixels()
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| **p == [color.0, color.1, color.2, color.3])
                .count()
                > 1000
        );
        let control = find(app, "sapid-chart-point");
        let physical = host
            .renderer()
            .engine_id(control, glory::html::Part::Main)
            .unwrap();
        let c = bounds(app, control);
        let hit = host.renderer().inspect(|dom| {
            dom.hit((c.x + c.width / 2.) as f32, (c.y + c.height / 2.) as f32)
                .unwrap()
                .node_id
        });
        assert_eq!(hit, physical);
        app.scope().action(Action::Focus(control)).unwrap();
        app.settle(16).unwrap();
        app.dispatch(control, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(all(app).iter().any(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Text(p)if p.content.contains("Revenue: "))));
        assert_eq!(
            host.renderer()
                .engine_id(control, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        let hide = all(app)
            .into_iter()
            .find(|id| {
                app.tree()
                    .node(*id)
                    .unwrap()
                    .descriptor
                    .view()
                    .name
                    .as_deref()
                    == Some("Hide Revenue")
            })
            .unwrap();
        app.dispatch(hide, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(all(app).iter().all(|id| {
            !app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|c| c == "sapid-chart-point")
        }));
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(host.renderer().stats().nodes, 0);
    }
}
