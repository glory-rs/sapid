use glory::apace::shell::WindowApp;
use glory::prelude::*;
use glory::render::{
    Action, Direction, DragPhase, Event, NodeId, Query, QueryValue, RangeCommand, Rect,
};
use glory::style::{Length, Style};
use std::{cell::RefCell, rc::Rc};
fn query(app: &App, query: Query) -> QueryValue {
    let output = Rc::new(RefCell::new(None));
    let result = output.clone();
    app.request(query, move |value| {
        *result.borrow_mut() = Some(value.unwrap())
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

#[test]
fn range_slider_native_four_themes_share_real_track_geometry_and_preserve_both_thumb_identities()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        for (axis, direction) in [
            (Orientation::Horizontal, Direction::Ltr),
            (Orientation::Horizontal, Direction::Rtl),
            (Orientation::Vertical, Direction::Ltr),
        ] {
            let host = WindowApp::new();
            let app = host.app();
            let root = app.scope().cage(None);
            let lower = app.scope().cage(None);
            let upper = app.scope().cage(None);
            host.mount(
                sapid::range_slider("native")
                    .theme(theme.clone())
                    .orientation(axis)
                    .direction(direction)
                    .minimum_gap(10.)
                    .node_ref(root)
                    .thumb_ref(sapid::RangeThumb::Lower, lower)
                    .thumb_ref(sapid::RangeThumb::Upper, upper)
                    .style(
                        Style::new()
                            .width(Length::Px(if axis == Orientation::Horizontal {
                                400.
                            } else {
                                36.
                            }))
                            .height(Length::Px(if axis == Orientation::Horizontal {
                                36.
                            } else {
                                240.
                            })),
                    ),
            )
            .unwrap();
            app.settle(16).unwrap();
            let track = bounds(app, root.get().unwrap());
            let low = lower.get().unwrap();
            let high = upper.get().unwrap();
            let identities = [
                host.renderer()
                    .engine_id(low, glory::html::Part::Main)
                    .unwrap(),
                host.renderer()
                    .engine_id(high, glory::html::Part::Main)
                    .unwrap(),
            ];
            for (id, target) in [(low, 40.), (high, 90.), (low, 100.)] {
                let fraction = target / 100.;
                let (x, y) = if axis == Orientation::Vertical {
                    (
                        track.x + track.width / 2.,
                        track.y + (1. - fraction) * track.height,
                    )
                } else {
                    (
                        track.x
                            + (if direction == Direction::Rtl {
                                1. - fraction
                            } else {
                                fraction
                            }) * track.width,
                        track.y + track.height / 2.,
                    )
                };
                for phase in [DragPhase::Start, DragPhase::Move, DragPhase::Drop] {
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
            }
            assert_eq!(value(app, low), 80.);
            assert_eq!(value(app, high), 90.);
            for (id, identity) in [(low, identities[0]), (high, identities[1])] {
                app.scope().action(Action::Focus(id)).unwrap();
                app.flush().unwrap();
                assert_eq!(query(app, Query::Focus(id)), QueryValue::Focus(true));
                assert_eq!(
                    host.renderer()
                        .engine_id(id, glory::html::Part::Main)
                        .unwrap(),
                    identity
                );
                let actual = bounds(app, id);
                let fraction = value(app, id) / 100.;
                let expected = if axis == Orientation::Vertical {
                    track.y + (1. - fraction) * track.height
                } else {
                    track.x
                        + (if direction == Direction::Rtl {
                            1. - fraction
                        } else {
                            fraction
                        }) * track.width
                };
                let center = if axis == Orientation::Vertical {
                    actual.y + actual.height / 2.
                } else {
                    actual.x + actual.width / 2.
                };
                assert!(
                    (center - expected).abs() < 0.1,
                    "{axis:?}/{direction:?}: center {center} expected {expected}"
                );
            }
            app.clear().unwrap();
            assert!(lower.get().is_none() && upper.get().is_none());
            assert_eq!(app.stats().queries, 0);
        }
    }
}
#[test]
fn range_slider_native_readonly_singleton_thumb_stays_focusable_and_recovers_without_replacement()
 {
    let host = WindowApp::new();
    let app = host.app();
    let lower = app.scope().cage(None);
    let upper = app.scope().cage(None);
    host.mount(
        sapid::range_slider("fixed")
            .minimum_gap(10.)
            .default_value([0., 10.])
            .thumb_ref(sapid::RangeThumb::Lower, lower)
            .thumb_ref(sapid::RangeThumb::Upper, upper),
    )
    .unwrap();
    app.settle(16).unwrap();
    let low = lower.get().unwrap();
    let high = upper.get().unwrap();
    let identity = host
        .renderer()
        .engine_id(low, glory::html::Part::Main)
        .unwrap();
    app.scope().action(Action::Focus(low)).unwrap();
    app.flush().unwrap();
    assert_eq!(query(app, Query::Focus(low)), QueryValue::Focus(true));
    assert!(
        !app.dispatch(
            low,
            Event::Range {
                from: low,
                command: RangeCommand::Increase
            }
        )
        .unwrap()
    );
    app.dispatch(
        high,
        Event::Range {
            from: high,
            command: RangeCommand::Maximum,
        },
    )
    .unwrap();
    app.settle(16).unwrap();
    app.dispatch(
        low,
        Event::Range {
            from: low,
            command: RangeCommand::Increase,
        },
    )
    .unwrap();
    app.settle(16).unwrap();
    assert_eq!(value(app, low), 1.);
    assert_eq!(
        host.renderer()
            .engine_id(low, glory::html::Part::Main)
            .unwrap(),
        identity
    );
}
