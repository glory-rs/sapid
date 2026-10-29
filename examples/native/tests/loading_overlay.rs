use glory::apace::{Raster, shell::WindowApp};
use glory::render::{Event, NodeId, Query, QueryValue, Rect};
use glory::style::{Color, Edges, Length, Style};
use glory::ui::App;
use std::{cell::RefCell, rc::Rc};
fn bounds(app: &App, id: NodeId) -> Rect {
    let result = Rc::new(RefCell::new(None));
    let out = result.clone();
    app.request(Query::Bounds(id), move |reply| {
        *out.borrow_mut() = Some(reply.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = result.borrow_mut().take().unwrap() else {
        panic!()
    };
    rect
}
fn pixel(raster: &Raster, x: f64, y: f64) -> [u8; 4] {
    let width = raster.frame().unwrap().width as usize;
    let offset = (y.round() as usize * width + x.round() as usize) * 4;
    raster.pixels()[offset..offset + 4].try_into().unwrap()
}
#[test]
fn native_loading_mask_has_real_alpha_geometry_hit_blocking_and_an_opaque_spinner_in_four_themes()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let loading = app.scope().cage(true);
        let clicks = app.scope().cage(0usize);
        let content_ref = app.scope().cage(None);
        let mask_ref = app.scope().cage(None);
        let root_ref = app.scope().cage(None);
        let red = Style {
            background: Some(Color(255, 0, 0, 255)),
            border_width: Some(Edges::all(0.)),
            radius: Some(0.),
            ..Style::new()
                .width(Length::Px(120.))
                .height(Length::Px(100.))
        };
        host.mount(
            sapid::loading_overlay(loading)
                .theme(theme.clone())
                .name("Editor region")
                .node_ref(root_ref)
                .content_ref(content_ref)
                .mask_ref(mask_ref)
                .color(Color(0, 0, 255, 255))
                .opacity(0.5)
                .style(Style::new().width(Length::Px(120.)))
                .fill(
                    sapid::button()
                        .theme(theme.clone())
                        .label("Region action")
                        .style(red)
                        .on_press(move || clicks.revise(|n| *n += 1)),
                ),
        )
        .unwrap();
        app.settle(16).unwrap();
        let root = root_ref.get().unwrap();
        let content = content_ref.get().unwrap();
        let mask = mask_ref.get().unwrap();
        let button = app.tree().node(content).unwrap().children[0];
        let rect = bounds(app, root);
        let mask_box = bounds(app, mask);
        assert!((rect.width - 120.).abs() < 0.1 && (rect.height - 100.).abs() < 0.1);
        assert_eq!(rect, mask_box);
        let mask_physical = host
            .renderer()
            .engine_id(mask, glory::html::Part::Main)
            .unwrap();
        let button_physical = host
            .renderer()
            .engine_id(button, glory::html::Part::Main)
            .unwrap();
        assert_eq!(
            host.renderer().inspect(|doc| doc
                .hit((rect.x + 5.) as f32, (rect.y + 5.) as f32)
                .unwrap()
                .node_id),
            mask_physical
        );
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        assert_eq!(pixel(&raster, rect.x + 5., rect.y + 5.), [127, 0, 128, 255]);
        assert_eq!(
            pixel(&raster, rect.x + 60., rect.y + 40.),
            [theme.accent.0, theme.accent.1, theme.accent.2, 255]
        );
        assert!(!app.dispatch(button, Event::Activate).unwrap());
        assert_eq!(clicks.get(), 0);
        loading.revise(|v| *v = false);
        app.flush().unwrap();
        let _ = bounds(app, root);
        raster.draw(host.renderer()).unwrap();
        assert_eq!(pixel(&raster, rect.x + 5., rect.y + 5.), [255, 0, 0, 255]);
        assert_eq!(
            host.renderer().inspect(|doc| doc
                .hit((rect.x + 5.) as f32, (rect.y + 5.) as f32)
                .unwrap()
                .node_id),
            button_physical
        );
        assert!(app.dispatch(button, Event::Activate).unwrap());
        assert_eq!(clicks.get(), 1);
        assert_eq!(
            host.renderer()
                .engine_id(mask, glory::html::Part::Main)
                .unwrap(),
            mask_physical
        );
        app.clear().unwrap();
        assert_eq!(
            (root_ref.get(), mask_ref.get(), content_ref.get()),
            (None, None, None)
        );
        assert_eq!(host.renderer().stats().nodes, 0);
    }
}
#[test]
fn native_decorative_mask_has_pixels_but_passes_real_hits_through_its_full_box() {
    let host = WindowApp::new();
    host.mount(
        sapid::overlay()
            .color(Color(0, 255, 0, 255))
            .opacity(0.5)
            .style(Style::new().width(Length::Px(120.)))
            .fill(
                glory::ui::view().style(Style {
                    background: Some(Color(255, 0, 0, 255)),
                    ..Style::new()
                        .width(Length::Px(120.))
                        .height(Length::Px(100.))
                }),
            ),
    )
    .unwrap();
    let app = host.app();
    let root = app.tree().roots()[0];
    let content = app.tree().node(root).unwrap().children[0];
    let child = app.tree().node(content).unwrap().children[0];
    let rect = bounds(app, root);
    let physical = host
        .renderer()
        .engine_id(child, glory::html::Part::Main)
        .unwrap();
    assert_eq!(
        host.renderer().inspect(|doc| doc
            .hit((rect.x + 5.) as f32, (rect.y + 5.) as f32)
            .unwrap()
            .node_id),
        physical
    );
    let mut raster = Raster::new();
    raster.draw(host.renderer()).unwrap();
    assert_eq!(pixel(&raster, rect.x + 5., rect.y + 5.), [127, 128, 0, 255]);
}
