use glory::blitz::{Raster, shell::WindowApp};
use glory::render::{NodeId, Query, QueryValue, Rect};
use glory::style::Color;
use std::{cell::RefCell, rc::Rc};
fn bounds(app: &glory::App, node: NodeId) -> Rect {
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    app.request(Query::Bounds(node), move |v| {
        *saved.borrow_mut() = Some(v.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = out.borrow_mut().take().unwrap() else {
        panic!()
    };
    rect
}
fn pixel(raster: &Raster, x: f64, y: f64) -> [u8; 4] {
    let width = raster.frame().unwrap().width as usize;
    let offset = (y as usize * width + x as usize) * 4;
    raster.pixels()[offset..offset + 4].try_into().unwrap()
}
fn close(actual: [u8; 4], expected: [u8; 4]) {
    for (a, b) in actual.into_iter().zip(expected) {
        assert!(a.abs_diff(b) <= 1, "{actual:?} != {expected:?}")
    }
}
#[test]
fn rgba_preview_has_real_checker_alpha_pixels_and_updates_same_native_graphic() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let color = app.scope().cage(Color(255, 0, 0, 128));
        let reference = app.scope().cage(None);
        host.mount(
            sapid::color_swatch(color)
                .theme(theme)
                .interactive(true)
                .node_ref(reference),
        )
        .unwrap();
        app.settle(12).unwrap();
        let root = reference.get().unwrap();
        let drawing = app.tree().node(root).unwrap().children[1];
        let rect = bounds(app, drawing);
        assert!(rect.width >= 28. && rect.width <= 32.);
        let identity = host
            .renderer()
            .engine_id(drawing, glory::html::Part::Main)
            .unwrap();
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        close(
            pixel(
                &raster,
                rect.x + rect.width * 0.125,
                rect.y + rect.height * 0.125,
            ),
            [224, 96, 96, 255],
        );
        close(
            pixel(
                &raster,
                rect.x + rect.width * 0.375,
                rect.y + rect.height * 0.125,
            ),
            [255, 127, 127, 255],
        );
        let hit = host.renderer().inspect(|doc| {
            doc.hit(
                (rect.x + rect.width / 2.) as f32,
                (rect.y + rect.height / 2.) as f32,
            )
            .unwrap()
            .node_id
        });
        assert_eq!(
            hit,
            host.renderer()
                .engine_id(root, glory::html::Part::Main)
                .unwrap()
        );
        color.revise(|v| *v = Color(0, 0, 255, 0));
        app.settle(12).unwrap();
        raster.draw(host.renderer()).unwrap();
        close(
            pixel(
                &raster,
                rect.x + rect.width * 0.125,
                rect.y + rect.height * 0.125,
            ),
            [192, 192, 192, 255],
        );
        close(
            pixel(
                &raster,
                rect.x + rect.width * 0.375,
                rect.y + rect.height * 0.125,
            ),
            [255, 255, 255, 255],
        );
        assert_eq!(
            host.renderer()
                .engine_id(drawing, glory::html::Part::Main)
                .unwrap(),
            identity
        );
        app.clear().unwrap();
        assert!(reference.get().is_none());
        assert_eq!(host.renderer().stats().nodes, 0);
    }
}
