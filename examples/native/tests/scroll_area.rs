use glory::apace::shell::WindowApp;
use glory::prelude::*;
use glory::render::{Event, Query, QueryValue, ScrollCommand, ScrollMetrics, ScrollMovement};
use glory::style::{Length, Style};
use sapid::{ScrollHandle, scroll_area, scroller};
use std::{cell::RefCell, rc::Rc};
fn metrics(app: &App, node: glory::render::NodeId) -> ScrollMetrics {
    let result = Rc::new(RefCell::new(None));
    let saved = result.clone();
    app.request(Query::ScrollMetrics(node), move |value| {
        *saved.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::ScrollMetrics(metrics) = result.borrow_mut().take().unwrap() else {
        panic!("metrics")
    };
    metrics
}
#[test]
fn native_scroll_area_four_themes_reads_real_extents_and_applies_ordered_queued_intents() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let viewport = app.scope().cage(None);
        let input = app.scope().cage(None);
        let output = Rc::new(RefCell::new(None::<ScrollHandle>));
        let saved = output.clone();
        host.mount(
            scroll_area("native")
                .theme(theme)
                .node_ref(viewport)
                .name("Native documents")
                .style(
                    Style::new()
                        .width(Length::Px(300.0))
                        .height(Length::Px(180.0)),
                )
                .on_ready(move |handle| *saved.borrow_mut() = Some(handle))
                .content(move |_| {
                    glory::ui::view()
                        .style(Style::new().height(Length::Px(1000.0)))
                        .fill(glory::ui::input().value("Kept editor").node_ref(input))
                }),
        )
        .unwrap();
        app.settle(16).unwrap();
        let viewport = viewport.get().unwrap();
        let input = input.get().unwrap();
        let engine = host
            .renderer()
            .engine_id(input, glory::html::Part::Main)
            .unwrap();
        let initial = metrics(app, viewport);
        assert!(initial.content_height >= 1000.0);
        assert!(initial.viewport_height <= 180.0);
        let handle = output.borrow().as_ref().unwrap().clone();
        handle.scroll_by(0.0, -100.0).unwrap();
        handle.scroll_by(0.0, 50.0).unwrap();
        app.settle(16).unwrap();
        assert_eq!(metrics(app, viewport).y, 50.0);
        app.dispatch(
            viewport,
            Event::ScrollKeyboard {
                from: viewport,
                command: ScrollCommand {
                    axis: Orientation::Vertical,
                    movement: ScrollMovement::PagePositive,
                },
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        assert_eq!(metrics(app, viewport).y, 50.0 + initial.viewport_height);
        assert_eq!(
            host.renderer()
                .engine_id(input, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        app.clear().unwrap();
        assert!(handle.refresh().is_err());
    }
}
#[test]
fn native_scroller_page_buttons_use_real_metrics_and_respect_end_boundaries() {
    let host = WindowApp::new();
    let app = host.app();
    let viewport = app.scope().cage(None);
    host.mount(
        scroller("pages")
            .axis(Orientation::Vertical)
            .area(move |area| {
                area.node_ref(viewport).style(
                    Style::new()
                        .width(Length::Px(250.0))
                        .height(Length::Px(150.0)),
                )
            })
            .content(|_| glory::ui::view().style(Style::new().height(Length::Px(750.0)))),
    )
    .unwrap();
    app.settle(16).unwrap();
    let ids = app.tree().roots().to_vec();
    let previous = ids[0];
    let next = ids[1];
    let viewport = viewport.get().unwrap();
    assert!(!app.tree().enabled(previous));
    assert!(app.tree().enabled(next));
    app.dispatch(next, Event::Activate).unwrap();
    app.settle(16).unwrap();
    assert_eq!(metrics(app, viewport).y, 150.0);
    assert!(app.tree().enabled(previous));
    for _ in 0..8 {
        if !app.tree().enabled(next) {
            break;
        }
        app.dispatch(next, Event::Activate).unwrap();
        app.settle(16).unwrap();
    }
    assert_eq!(metrics(app, viewport).y, 600.0);
    assert!(!app.tree().enabled(next));
    app.dispatch(previous, Event::Activate).unwrap();
    app.settle(16).unwrap();
    assert_eq!(metrics(app, viewport).y, 450.0);
    app.clear().unwrap();
}
