use glory::apace::{Config, Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{
    Action, GraphicProps, MotionPreference, NodeId, Props, Query, QueryValue, Rect,
};
use std::{cell::RefCell, rc::Rc};
fn observe(host: &WindowApp) {
    for _ in 0..8 {
        host.observe(&std::task::Context::from_waker(std::task::Waker::noop()))
            .unwrap();
    }
}
fn nodes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut found = Vec::new();
    while let Some(id) = todo.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            found.push(id)
        }
        todo.extend(node.children.iter().rev());
    }
    found
}
fn value(app: &App, query: Query) -> QueryValue {
    let result = Rc::new(RefCell::new(None));
    let reply = result.clone();
    app.request(query, move |value| {
        *reply.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    result.borrow_mut().take().unwrap()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let QueryValue::Bounds(rect) = value(app, Query::Bounds(id)) else {
        panic!("bounds")
    };
    rect
}
fn drawing(app: &App, id: NodeId) -> GraphicProps {
    let tree = app.tree();
    let Props::Graphic(graphic) = &tree.node(id).unwrap().descriptor.view().props else {
        panic!("graphic")
    };
    graphic.clone()
}
#[test]
fn four_theme_real_bounds_drive_finite_painted_motion_without_touching_actual_editor_focus() {
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
        host.renderer().set_viewport(420, 160, 1.).unwrap();
        let app = host.app();
        let active = app.scope().cage(Some("one".to_owned()));
        let input = app.scope().cage(None);
        let theme_content = theme.clone();
        host.mount(
            sapid::floating_indicator("native")
                .theme(theme)
                .active(active)
                .duration(200u16)
                .style(Style::new().width(Length::Px(400.)))
                .content(move |context| {
                    glory::ui::view().fill((
                        glory::ui::row()
                            .style(
                                Style::new()
                                    .display(Display::Flex)
                                    .flex_direction(FlexDirection::Row)
                                    .gap(12.),
                            )
                            .fill((
                                sapid::floating_indicator_target(context.clone(), "one")
                                    .class("native-target-one")
                                    .fill(
                                        sapid::button()
                                            .theme(theme_content.clone())
                                            .label("One")
                                            .style(Style::new().width(Length::Px(80.))),
                                    ),
                                sapid::floating_indicator_target(context, "two")
                                    .class("native-target-two")
                                    .fill(
                                        sapid::button()
                                            .theme(theme_content.clone())
                                            .label("Two")
                                            .style(Style::new().width(Length::Px(120.))),
                                    ),
                            )),
                        glory::ui::input()
                            .node_ref(input)
                            .value("Retained native focus"),
                    ))
                }),
        )
        .unwrap();
        observe(&host);
        let shape = nodes(app, "sapid-floating-indicator-shape")[0];
        let area = nodes(app, "sapid-floating-area")[0];
        let first = nodes(app, "native-target-one")[0];
        let second = nodes(app, "native-target-two")[0];
        let initial = drawing(app, shape);
        let target = bounds(app, first);
        let origin = bounds(app, area);
        assert_eq!(
            initial.commands[0],
            glory::render::PathCommand::Move(glory::render::GraphicPoint::new(
                target.x - origin.x,
                target.y - origin.y
            ))
        );
        assert!(target.width >= 79. && target.height > 20.);
        let physical = host
            .renderer()
            .engine_id(shape, glory::html::Part::Main)
            .unwrap();
        app.scope()
            .action(Action::Focus(input.get().unwrap()))
            .unwrap();
        app.flush().unwrap();
        observe(&host);
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let before = raster.pixels().to_vec();
        active.revise(|value| *value = Some("two".into()));
        app.flush().unwrap();
        observe(&host);
        assert_eq!(
            value(app, Query::Focus(input.get().unwrap())),
            QueryValue::Focus(true)
        );
        let writes = app.runtime_stats().source_writes;
        let next = drawing(app, shape);
        let target = bounds(app, second);
        assert_eq!(
            next.commands[0],
            glory::render::PathCommand::Move(glory::render::GraphicPoint::new(
                target.x - origin.x,
                target.y - origin.y
            ))
        );
        host.renderer().advance_time(0.1).unwrap();
        raster.draw(host.renderer()).unwrap();
        let middle = raster.pixels().to_vec();
        assert_ne!(middle, before);
        host.renderer().advance_time(0.21).unwrap();
        raster.draw(host.renderer()).unwrap();
        assert_ne!(raster.pixels(), before);
        assert_ne!(raster.pixels(), middle);
        assert_eq!(app.runtime_stats().source_writes, writes);
        assert!(!host.renderer().inspect(|document| document.is_animating()));
        assert_eq!(
            host.renderer()
                .engine_id(shape, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        app.clear().unwrap();
        assert_eq!(app.stats().queries, 0);
        assert_eq!(app.runtime_stats().tasks, 0);
    }
}

#[test]
fn default_native_reduce_is_static_and_hiding_retires_current_motion_membership() {
    let host = WindowApp::new();
    host.renderer().set_viewport(320, 100, 1.).unwrap();
    let app = host.app();
    let active = app.scope().cage(Some("one".to_owned()));
    let visible = app.scope().cage(true);
    host.mount(
        sapid::floating_indicator("reduce")
            .active(active)
            .visible(visible)
            .duration(200u16)
            .style(Style::new().width(Length::Px(300.)))
            .content(|context| {
                glory::ui::row()
                    .style(
                        Style::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Row)
                            .gap(20.),
                    )
                    .fill((
                        sapid::floating_indicator_target(context.clone(), "one").fill(
                            glory::ui::view().style(
                                Style::new().width(Length::Px(60.)).height(Length::Px(36.)),
                            ),
                        ),
                        sapid::floating_indicator_target(context, "two").fill(
                            glory::ui::view().style(
                                Style::new().width(Length::Px(100.)).height(Length::Px(36.)),
                            ),
                        ),
                    ))
            }),
    )
    .unwrap();
    observe(&host);
    let mut raster = Raster::new();
    raster.draw(host.renderer()).unwrap();
    let initial = raster.pixels().to_vec();
    active.revise(|value| *value = Some("two".into()));
    app.flush().unwrap();
    observe(&host);
    raster.draw(host.renderer()).unwrap();
    assert_ne!(raster.pixels(), initial);
    assert!(!host.renderer().inspect(|document| document.is_animating()));
    host.renderer()
        .set_motion_preference(MotionPreference::NoPreference);
    active.revise(|value| *value = Some("one".into()));
    app.flush().unwrap();
    observe(&host);
    assert!(host.renderer().inspect(|document| document.is_animating()));
    visible.revise(|value| *value = false);
    app.flush().unwrap();
    observe(&host);
    assert!(!host.renderer().inspect(|document| document.is_animating()));
    active.revise(|value| *value = Some("two".into()));
    visible.revise(|value| *value = true);
    app.flush().unwrap();
    observe(&host);
    assert!(!host.renderer().inspect(|document| document.is_animating()));
    app.clear().unwrap();
    assert_eq!(app.runtime_stats().tasks, 0);
    assert_eq!(app.stats().queries, 0);
}
