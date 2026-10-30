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
fn slider_native_four_themes_axes_and_rtl_use_real_bounds_and_keep_identity() {
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
            let reference = app.scope().cage(None);
            host.mount(
                sapid::slider("native")
                    .theme(theme.clone())
                    .name("Native range")
                    .orientation(axis)
                    .direction(direction)
                    .default_value(25.)
                    .node_ref(reference)
                    .style(
                        Style::new()
                            .width(Length::Px(if axis == Orientation::Horizontal {
                                240.
                            } else {
                                32.
                            }))
                            .height(Length::Px(if axis == Orientation::Horizontal {
                                32.
                            } else {
                                240.
                            })),
                    ),
            )
            .unwrap();
            app.settle(16).unwrap();
            let id = reference.get().unwrap();
            let identity = host
                .renderer()
                .engine_id(id, glory::html::Part::Main)
                .unwrap();
            let rect = bounds(app, id);
            assert!(rect.width > 0. && rect.height > 0.);
            app.scope().action(Action::Focus(id)).unwrap();
            app.flush().unwrap();
            assert_eq!(query(app, Query::Focus(id)), QueryValue::Focus(true));
            let (x, y) = if axis == Orientation::Vertical {
                (rect.x + rect.width / 2., rect.y + rect.height * 0.25)
            } else {
                (rect.x + rect.width * 0.75, rect.y + rect.height / 2.)
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
            assert_eq!(
                value(app, id),
                if direction == Direction::Rtl {
                    25.
                } else {
                    75.
                }
            );
            app.dispatch(
                id,
                Event::Range {
                    from: id,
                    command: RangeCommand::PageDecrease,
                },
            )
            .unwrap();
            app.settle(16).unwrap();
            assert_eq!(
                value(app, id),
                if direction == Direction::Rtl {
                    15.
                } else {
                    65.
                }
            );
            assert_eq!(
                host.renderer()
                    .engine_id(id, glory::html::Part::Main)
                    .unwrap(),
                identity
            );
            app.clear().unwrap();
            assert!(app.tree().is_empty());
            assert_eq!(app.stats().queries, 0);
        }
    }
}
#[test]
fn slider_native_field_readonly_controlled_rejection_and_accepted_source_preserve_live_identity() {
    let host = WindowApp::new();
    let app = host.app();
    let source = app.scope().cage(40.);
    let readonly = app.scope().cage(false);
    let requested = app.scope().cage(0.);
    let reference = app.scope().cage(None);
    host.mount(
        sapid::field("amount")
            .label("Amount")
            .description("Units")
            .readonly(readonly)
            .control(move |field| {
                sapid::slider("controlled")
                    .field(field)
                    .value(source)
                    .node_ref(reference)
                    .on_value_change(move |v| requested.revise(|old| *old = v))
            }),
    )
    .unwrap();
    app.settle(16).unwrap();
    let id = reference.get().unwrap();
    let identity = host
        .renderer()
        .engine_id(id, glory::html::Part::Main)
        .unwrap();
    app.dispatch(
        id,
        Event::Range {
            from: id,
            command: RangeCommand::Increase,
        },
    )
    .unwrap();
    app.settle(16).unwrap();
    assert_eq!(requested.get(), 41.);
    assert_eq!(value(app, id), 40.);
    readonly.revise(|v| *v = true);
    app.settle(16).unwrap();
    assert!(
        !app.dispatch(
            id,
            Event::Range {
                from: id,
                command: RangeCommand::Maximum
            }
        )
        .unwrap()
    );
    assert_eq!(requested.get(), 41.);
    source.revise(|v| *v = 70.);
    app.settle(16).unwrap();
    assert_eq!(value(app, id), 70.);
    assert_eq!(
        host.renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap(),
        identity
    );
    app.clear().unwrap();
    assert!(reference.get().is_none());
}
