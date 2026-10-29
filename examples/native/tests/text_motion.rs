use glory::apace::{Config, Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{
    Action, MotionPreference, NodeId, Query, QueryValue, Rect, TranslationKind,
};
use std::{cell::RefCell, rc::Rc};
fn observe(host: &WindowApp) {
    for _ in 0..10 {
        host.observe(&std::task::Context::from_waker(std::task::Waker::noop()))
            .unwrap();
    }
}
fn nodes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut result = Vec::new();
    while let Some(id) = todo.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            result.push(id);
        }
        todo.extend(node.children.iter().rev());
    }
    result
}
fn query(app: &App, query: Query) -> QueryValue {
    let result = Rc::new(RefCell::new(None));
    let reply = result.clone();
    app.request(query, move |value| {
        *reply.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    result.borrow_mut().take().unwrap()
}
fn bounds(app: &App, node: NodeId) -> Rect {
    let QueryValue::Bounds(rect) = query(app, Query::Bounds(node)) else {
        panic!()
    };
    rect
}
#[test]
fn four_theme_marquee_and_reels_use_real_layout_clock_clip_pixels_and_retained_focus() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::with_config(Config {
            motion: MotionPreference::NoPreference,
            ..Default::default()
        });
        host.renderer().set_viewport(420, 220, 1.).unwrap();
        let app = host.app();
        let animated = app.scope().cage(true);
        let value = app.scope().cage(1234.25);
        let editor = app.scope().cage(None);
        host.mount(
            glory::ui::col().fill((
                sapid::marquee()
                    .theme(theme.clone())
                    .text("A finite native marquee status moving through a clipped viewport")
                    .animated(animated)
                    .period_millis(1200u32)
                    .style(
                        Style::new()
                            .width(Length::Px(160.))
                            .max_width(glory::style::Limit::Percent(100.)),
                    ),
                sapid::rolling_number().theme(theme).value(value),
                glory::ui::input()
                    .node_ref(editor)
                    .value("Kept native editor"),
            )),
        )
        .unwrap();
        observe(&host);
        let viewport = nodes(app, "sapid-marquee-viewport")[0];
        let first = nodes(app, "sapid-marquee-first")[0];
        let track = nodes(app, "sapid-marquee-track")[0];
        assert!(bounds(app, first).width > bounds(app, viewport).width);
        assert!(matches!(
            app.tree()
                .node(track)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_deref()
                .unwrap()
                .translation
                .unwrap()
                .kind,
            TranslationKind::Loop { .. }
        ));
        app.scope()
            .action(Action::Focus(editor.get().unwrap()))
            .unwrap();
        app.flush().unwrap();
        observe(&host);
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let initial = raster.pixels().to_vec();
        let writes = app.runtime_stats().source_writes;
        host.renderer().advance_time(0.2).unwrap();
        raster.draw(host.renderer()).unwrap();
        assert_ne!(raster.pixels(), initial);
        assert_eq!(app.runtime_stats().source_writes, writes);
        animated.revise(|old| *old = false);
        app.flush().unwrap();
        observe(&host);
        let reels = nodes(app, "sapid-rolling-number-reel");
        let reel = reels[2];
        let before = bounds(app, reel).y;
        let physical = host
            .renderer()
            .engine_id(reel, glory::html::Part::Main)
            .unwrap();
        raster.draw(host.renderer()).unwrap();
        let first = raster.pixels().to_vec();
        value.revise(|old| *old = 1298.75);
        app.flush().unwrap();
        observe(&host);
        assert_eq!(nodes(app, "sapid-rolling-number-reel"), reels);
        host.renderer().advance_time(0.35).unwrap();
        raster.draw(host.renderer()).unwrap();
        let middle = raster.pixels().to_vec();
        let y = bounds(app, reel).y;
        assert!(
            y < before && y > before - 144.,
            "intermediate reel y={y},previous={before}"
        );
        assert_ne!(middle, first);
        host.renderer().advance_time(0.9).unwrap();
        raster.draw(host.renderer()).unwrap();
        assert!((bounds(app, reel).y - (before - 144.)).abs() < 0.05);
        assert_ne!(raster.pixels(), middle);
        assert_eq!(
            query(app, Query::Focus(editor.get().unwrap())),
            QueryValue::Focus(true)
        );
        assert_eq!(
            host.renderer()
                .engine_id(reel, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        assert_eq!(value.get(), 1298.75);
        app.clear().unwrap();
        assert_eq!(app.stats().queries, 0);
        assert_eq!(app.runtime_stats().tasks, 0);
    }
}
#[test]
fn native_default_reduce_is_static_and_unstyled_retains_finite_presentation() {
    let host = WindowApp::new();
    host.renderer().set_viewport(240, 120, 1.).unwrap();
    let app = host.app();
    let value = app.scope().cage(12.);
    host.mount(
        glory::ui::col().fill((
            sapid::marquee()
                .unstyled()
                .text("A longer static reduced-motion accessible status")
                .style(Style::new().width(Length::Px(120.))),
            sapid::rolling_number().unstyled().value(value),
        )),
    )
    .unwrap();
    observe(&host);
    let mut raster = Raster::new();
    raster.draw(host.renderer()).unwrap();
    assert!(!host.renderer().inspect(|doc| doc.is_animating()));
    let before = raster.pixels().to_vec();
    value.revise(|value| *value = 98.);
    app.flush().unwrap();
    observe(&host);
    raster.draw(host.renderer()).unwrap();
    assert_ne!(before, raster.pixels());
    assert!(!host.renderer().inspect(|doc| doc.is_animating()));
    host.renderer().set_viewport(140, 120, 1.).unwrap();
    observe(&host);
    assert_eq!(app.stats().queries, 0);
    app.clear().unwrap();
    assert_eq!(app.runtime_stats().tasks, 0);
}
