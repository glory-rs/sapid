use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, Event, NodeId, Query, QueryValue, RangeCommand, Rect};
use glory::style::Color;
use std::{cell::RefCell, rc::Rc};
fn bounds(app: &App, node: NodeId) -> Rect {
    let out = Rc::new(RefCell::new(None));
    let result = out.clone();
    app.request(Query::Bounds(node), move |value| {
        *result.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = out.borrow_mut().take().unwrap() else {
        panic!("bounds")
    };
    rect
}
fn classes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut ids = Vec::new();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            ids.push(id)
        }
        pending.extend(&node.children)
    }
    ids
}
fn pixel(raster: &Raster, x: f64, y: f64) -> [u8; 4] {
    let offset = (y as usize * raster.frame().unwrap().width as usize + x as usize) * 4;
    raster.pixels()[offset..offset + 4].try_into().unwrap()
}
fn close(actual: [u8; 4], expected: [u8; 4]) {
    for (a, b) in actual.into_iter().zip(expected) {
        assert!(a.abs_diff(b) <= 2, "{actual:?} != {expected:?}")
    }
}
#[test]
fn native_hue_and_alpha_four_themes_have_real_color_pixels_and_shared_control_identity() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let hue = app.scope().cage(None);
        let alpha = app.scope().cage(None);
        let color = app.scope().cage(Color(100, 150, 200, 255));
        host.mount(
            sapid::stack().gap(16.).fill((
                sapid::hue_slider("hue")
                    .theme(theme.clone())
                    .default_value(180.)
                    .node_ref(hue),
                sapid::alpha_slider("alpha")
                    .theme(theme)
                    .color(color)
                    .default_value(0.5)
                    .node_ref(alpha),
            )),
        )
        .unwrap();
        app.settle(16).unwrap();
        let id = hue.get().unwrap();
        let identity = host
            .renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap();
        app.scope().action(Action::Focus(id)).unwrap();
        app.flush().unwrap();
        app.dispatch(
            id,
            Event::Range {
                from: id,
                command: RangeCommand::Increase,
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        assert_eq!(
            app.tree()
                .node(id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .slider
                .as_ref()
                .unwrap()
                .range
                .value,
            181.
        );
        let mut segments = classes(app, "sapid-color-slider-segment");
        segments.sort_by(|a, b| {
            let ar = bounds(app, *a);
            let br = bounds(app, *b);
            ar.y.partial_cmp(&br.y)
                .unwrap()
                .then_with(|| ar.x.partial_cmp(&br.x).unwrap())
        });
        assert_eq!(segments.len(), 184);
        let hue_sample = segments[30];
        let hue_rect = bounds(app, hue_sample);
        let Some(Color(r, g, b, a)) = app
            .tree()
            .node(hue_sample)
            .unwrap()
            .descriptor
            .view()
            .style
            .background
        else {
            panic!("hue")
        };
        let alpha_start = bounds(app, segments[120]);
        let alpha_end = bounds(app, *segments.last().unwrap());
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        close(
            pixel(&raster, hue_rect.x + hue_rect.width / 2., hue_rect.y + 0.5),
            [r, g, b, a],
        );
        close(
            pixel(
                &raster,
                alpha_start.x + alpha_start.width / 2.,
                alpha_start.y + 0.5,
            ),
            [192, 192, 192, 255],
        );
        close(
            pixel(
                &raster,
                alpha_end.x + alpha_end.width / 2.,
                alpha_end.y + 0.5,
            ),
            [100, 150, 200, 255],
        );
        color.revise(|old| *old = Color(200, 50, 100, 3));
        app.settle(16).unwrap();
        raster.draw(host.renderer()).unwrap();
        close(
            pixel(
                &raster,
                alpha_end.x + alpha_end.width / 2.,
                alpha_end.y + 0.5,
            ),
            [200, 50, 100, 255],
        );
        assert_eq!(
            host.renderer()
                .engine_id(id, glory::html::Part::Main)
                .unwrap(),
            identity
        );
        let rect = bounds(app, alpha.get().unwrap());
        assert!(rect.width > 0. && rect.height > 0.);
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(app.stats().queries, 0);
    }
}
