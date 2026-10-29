use glory::apace::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, NodeId, Props, Query, QueryValue, Rect};
use sapid::charts::*;
use std::{cell::RefCell, rc::Rc};
#[path = "../../fixtures/plot_samples.rs"]
mod samples;
fn all(app: &App) -> Vec<NodeId> {
    let t = app.tree();
    let mut p = t.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = p.pop() {
        p.extend(&t.node(id).unwrap().children);
        out.push(id);
    }
    out
}
fn class(app: &App, c: &str) -> Vec<NodeId> {
    all(app)
        .into_iter()
        .filter(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|v| v == c)
        })
        .collect()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let out = Rc::new(RefCell::new(None));
    let result = out.clone();
    app.request(Query::Bounds(id), move |v| {
        *result.borrow_mut() = Some(v.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(r) = out.borrow_mut().take().unwrap() else {
        panic!()
    };
    r
}
#[test]
fn eighteen_plots_in_four_themes_render_native_vectors_hit_focus_keep_identity_and_retire() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        for kind in samples::KINDS {
            let host = WindowApp::new();
            let app = host.app();
            let data = app.scope().cage(samples::sample(kind));
            host.mount(
                plot("native", kind)
                    .name(samples::title(kind))
                    .theme(theme.clone())
                    .data(data)
                    .style(Style::new().width(Length::Px(640.))),
            )
            .unwrap_or_else(|e| panic!("{kind:?} {e:?}"));
            app.settle(16).unwrap();
            let frame = class(app, "sapid-plot-drawing")[0];
            let r = bounds(app, frame);
            assert!(
                (r.width - 640.).abs() < 0.5 && (r.height - 320.).abs() < 0.5,
                "{kind:?} {r:?}"
            );
            let mut raster = Raster::new();
            raster.draw(host.renderer()).unwrap();
            let palette = ChartTheme::from_theme(&theme);
            let colored = raster
                .pixels()
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| palette.series.iter().any(|c| **p == [c.0, c.1, c.2, c.3]))
                .count();
            assert!(colored > 100, "{kind:?}: {colored}");
            let mark = class(app, "sapid-plot-point")[0];
            let physical = host
                .renderer()
                .engine_id(mark, glory::html::Part::Main)
                .unwrap();
            let b = bounds(app, mark);
            let hit = host.renderer().inspect(|dom| {
                dom.hit((b.x + b.width / 2.) as f32, (b.y + b.height / 2.) as f32)
                    .unwrap()
                    .node_id
            });
            assert_eq!(hit, physical, "{kind:?}");
            app.scope().action(Action::Focus(mark)).unwrap();
            app.settle(16).unwrap();
            app.dispatch(mark, Event::Activate).unwrap();
            app.settle(16).unwrap();
            assert!(all(app).iter().any(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Text(p)if p.content.contains(": "))));
            data.revise(samples::update);
            app.settle(16).unwrap();
            assert_eq!(
                host.renderer().engine_id(mark, glory::html::Part::Main),
                Ok(physical),
                "{kind:?}"
            );
            data.revise(|d| *d = samples::empty(kind));
            app.settle(16).unwrap();
            assert!(class(app, "sapid-plot-point").is_empty());
            app.clear().unwrap();
            assert_eq!(app.stats().nodes, 0);
            assert_eq!(host.renderer().stats().nodes, 0);
        }
    }
}
