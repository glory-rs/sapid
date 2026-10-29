use glory::apace::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, DragPhase, NodeId, PathCommand, Props, Query, QueryValue, Rect};
use glory::style::{Length, Style};
use std::{cell::RefCell, rc::Rc};

fn query(app: &App, query: Query) -> QueryValue {
    let output = Rc::new(RefCell::new(None));
    let reply = output.clone();
    app.request(query, move |value| {
        *reply.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    output.borrow_mut().take().unwrap()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let QueryValue::Bounds(rect) = query(app, Query::Bounds(id)) else {
        panic!("bounds")
    };
    rect
}
fn value(app: &App, id: NodeId) -> f64 {
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
        .value
}
fn drag(app: &App, id: NodeId, phase: DragPhase, x: f64, y: f64) {
    app.dispatch(
        id,
        Event::Drag {
            from: id,
            phase,
            x,
            y,
        },
    )
    .unwrap();
    app.settle(16).unwrap();
}
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|v| v == class) {
            return id;
        }
        pending.extend(&node.children);
    }
    panic!("missing {class}")
}
#[test]
fn native_angle_cardinals_center_and_non_square_geometry_keep_live_identity_in_four_themes() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let reference = app.scope().cage(None);
        host.mount(
            sapid::angle_slider("native-angle")
                .theme(theme)
                .node_ref(reference)
                .style(
                    Style::new()
                        .width(Length::Px(240.))
                        .height(Length::Px(120.)),
                ),
        )
        .unwrap();
        app.settle(16).unwrap();
        let id = reference.get().unwrap();
        let rect = bounds(app, id);
        assert!((rect.width - 240.).abs() < 0.1 && (rect.height - 120.).abs() < 0.1);
        let physical = host
            .renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap();
        app.scope().action(Action::Focus(id)).unwrap();
        app.flush().unwrap();
        assert_eq!(query(app, Query::Focus(id)), QueryValue::Focus(true));
        let cx = rect.x + rect.width / 2.;
        let cy = rect.y + rect.height / 2.;
        for (x, y, expected) in [
            (cx, rect.y, 0.),
            (rect.x + rect.width, cy, 90.),
            (cx, rect.y + rect.height, 180.),
            (rect.x, cy, 270.),
        ] {
            drag(app, id, DragPhase::Start, x, y);
            assert_eq!(value(app, id), expected);
            drag(app, id, DragPhase::Move, cx, cy);
            assert_eq!(
                value(app, id),
                expected,
                "undefined center changed direction"
            );
            drag(app, id, DragPhase::Drop, cx, cy);
            assert_eq!(value(app, id), expected);
        }
        let hand = find(app, "sapid-angle-slider-hand");
        let tree = app.tree();
        let Props::Graphic(graphic) = tree.node(hand).unwrap().descriptor.view().props else {
            panic!("graphic")
        };
        let PathCommand::Line(end) = graphic.commands[1] else {
            panic!("hand endpoint")
        };
        assert!((end.x - 10.).abs() < 1e-8 && (end.y - 50.).abs() < 1e-8);
        drop(tree);
        assert_eq!(
            host.renderer()
                .engine_id(id, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        app.clear().unwrap();
        assert!(reference.get().is_none());
        assert_eq!(app.stats().queries, 0);
    }
}
#[test]
fn native_angle_graphics_paint_the_rotating_hand_and_thumb_in_four_themes() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let reference = app.scope().cage(None);
        host.mount(
            sapid::angle_slider("pixels")
                .theme(theme.clone())
                .default_value(90.)
                .node_ref(reference),
        )
        .unwrap();
        app.settle(16).unwrap();
        let id = reference.get().unwrap();
        let rect = bounds(app, id);
        let mut raster = Raster::default();
        raster.draw(host.renderer()).unwrap();
        let image = raster.frame().unwrap();
        let x = (rect.x + rect.width * 0.7).round() as usize;
        let y = (rect.y + rect.height / 2.).round() as usize;
        let offset = (y * image.width as usize + x) * 4;
        assert_eq!(
            &raster.pixels()[offset..offset + 4],
            &[
                theme.accent.0,
                theme.accent.1,
                theme.accent.2,
                theme.accent.3
            ]
        );
        app.clear().unwrap();
    }
}
