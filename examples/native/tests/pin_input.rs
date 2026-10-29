use glory::apace::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, NodeId, Props, Query, QueryValue, Rect};
use sapid::{PinAlphabet, input_otp, pin_input};
use std::{cell::RefCell, rc::Rc};
fn query(app: &App, query: Query) -> QueryValue {
    let reply = Rc::new(RefCell::new(None));
    let out = reply.clone();
    app.request(query, move |r| *out.borrow_mut() = Some(r.unwrap()))
        .unwrap();
    app.pump().unwrap();
    reply.borrow_mut().take().unwrap()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let QueryValue::Bounds(rect) = query(app, Query::Bounds(id)) else {
        panic!("bounds")
    };
    rect
}
#[test]
fn native_pin_has_actual_glyph_aligned_cells_real_hit_targets_focus_and_theme_pixels() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        for masked in [false, true] {
            let host = WindowApp::new();
            host.renderer().set_viewport(600, 180, 1.).unwrap();
            let app = host.app();
            let r = app.scope().cage(None);
            host.mount(
                input_otp()
                    .theme(theme.clone())
                    .default_value("000000")
                    .masked(masked)
                    .node_ref(r),
            )
            .unwrap();
            app.settle(12).unwrap();
            let id = r.get().unwrap();
            let rect = bounds(app, id);
            assert!(rect.width > 40. && rect.height >= 35.);
            let physical = host
                .renderer()
                .engine_id(id, glory::html::Part::Main)
                .unwrap();
            let advance = host.renderer().inspect(|dom| {
                let input = dom
                    .get_node(physical)
                    .unwrap()
                    .element_data()
                    .unwrap()
                    .text_input_data()
                    .unwrap();
                assert_eq!(input.is_password(), masked);
                input.display_editor().try_layout().unwrap().width()
            });
            assert!((rect.width - f64::from(advance)).abs() < 0.2);
            for index in 0..6 {
                let target = host.renderer().inspect(|dom| {
                    dom.hit(
                        (rect.x + rect.width * (index as f64 + 0.5) / 6.) as f32,
                        (rect.y + rect.height / 2.) as f32,
                    )
                    .unwrap()
                    .node_id
                });
                assert_eq!(
                    target, physical,
                    "a passive segment cannot intercept editor hits"
                );
            }
            let frame = app.tree().node(id).unwrap().parent.unwrap();
            let cells = app
                .tree()
                .node(frame)
                .unwrap()
                .children
                .iter()
                .copied()
                .filter(|child| *child != id)
                .collect::<Vec<_>>();
            assert_eq!(cells.len(), 6);
            for (index, cell) in cells.iter().enumerate() {
                let cell = bounds(app, *cell);
                assert!((cell.width * 6. - rect.width).abs() < 0.2);
                assert!((cell.x - (rect.x + rect.width * index as f64 / 6.)).abs() < 0.2);
            }
            app.scope().action(Action::Focus(id)).unwrap();
            app.flush().unwrap();
            assert_eq!(query(app, Query::Focus(id)), QueryValue::Focus(true));
            let mut raster = Raster::new();
            raster.draw(host.renderer()).unwrap();
            let offset = (((rect.y + 4.).round() as usize)
                * raster.frame().unwrap().width as usize
                + (rect.x + rect.width / 2. + 2.).round() as usize)
                * 4;
            let actual = &raster.pixels()[offset..offset + 4];
            for (a, b) in
                actual
                    .iter()
                    .zip([theme.input.0, theme.input.1, theme.input.2, theme.input.3])
            {
                assert!(
                    a.abs_diff(b) <= 2,
                    "theme background must paint behind the real editor"
                );
            }
            app.clear().unwrap();
            assert!(r.get().is_none());
            assert_eq!(app.stats().queries, 0);
        }
    }
}
#[test]
fn native_pin_unstyled_large_mono_readonly_and_controlled_restoration_keep_editor_identity() {
    let host = WindowApp::new();
    let app = host.app();
    let r = app.scope().cage(None);
    let source = app.scope().cage("aZ09".to_owned());
    let readonly = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    host.mount(
        pin_input()
            .length(4)
            .alphabet(PinAlphabet::AsciiAlphanumeric)
            .value(source)
            .readonly(readonly)
            .enabled(enabled)
            .unstyled()
            .style(Style {
                font_size: Some(24.),
                ..Style::new()
            })
            .node_ref(r),
    )
    .unwrap();
    let id = r.get().unwrap();
    let rect = bounds(app, id);
    assert!(rect.width > 50. && rect.height >= 37.);
    let physical = host
        .renderer()
        .engine_id(id, glory::html::Part::Main)
        .unwrap();
    let revision = {
        let tree = app.tree();
        let Props::Input(input) = tree.node(id).unwrap().descriptor.view().props else {
            panic!("input")
        };
        input.revision
    };
    app.dispatch(
        id,
        Event::ValueChanged {
            value: "0001".into(),
            revision,
            composing: false,
        },
    )
    .unwrap();
    app.settle(12).unwrap();
    host.renderer().inspect(|dom| {
        assert_eq!(
            dom.get_node(physical)
                .unwrap()
                .element_data()
                .unwrap()
                .text_input_data()
                .unwrap()
                .editor
                .raw_text()
                .len(),
            4
        )
    });
    assert_eq!(source.get(), "aZ09");
    readonly.revise(|v| *v = true);
    app.flush().unwrap();
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert_eq!(r.get(), Some(id));
    assert_eq!(
        host.renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap(),
        physical
    );
    app.clear().unwrap();
    assert_eq!(app.stats().queries, 0);
}
