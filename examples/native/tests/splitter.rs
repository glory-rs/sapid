use glory::blitz::shell::WindowApp;
use glory::prelude::*;
use glory::render::{Action, Event, NodeId, Query, QueryValue, RangeCommand, Rect};
use glory::style::{Length, Style};
use sapid::splitter;
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
#[test]
fn splitter_native_four_themes_conserve_both_axes_and_keep_pane_editor_identity() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        for axis in [Orientation::Horizontal, Orientation::Vertical] {
            let host = WindowApp::new();
            let app = host.app();
            let first = app.scope().cage(None);
            let second = app.scope().cage(None);
            let handle = app.scope().cage(None);
            let editor = app.scope().cage(None);
            host.mount(
                splitter("native")
                    .theme(theme.clone())
                    .axis(axis)
                    .name("Resize native panes")
                    .first_ref(first)
                    .second_ref(second)
                    .handle_ref(handle)
                    .style(
                        Style::new()
                            .width(Length::Px(if axis == Orientation::Horizontal {
                                606.0
                            } else {
                                300.0
                            }))
                            .height(Length::Px(if axis == Orientation::Vertical {
                                606.0
                            } else {
                                300.0
                            })),
                    )
                    .first(move || glory::ui::input().node_ref(editor).value("Kept editor"))
                    .second(|| glory::ui::text().content("Stable second pane")),
            )
            .unwrap();
            app.settle(16).unwrap();
            let first = first.get().unwrap();
            let second = second.get().unwrap();
            let handle = handle.get().unwrap();
            let editor = editor.get().unwrap();
            let identity = host
                .renderer()
                .engine_id(editor, glory::html::Part::Main)
                .unwrap();
            let length = |rect: Rect| {
                if axis == Orientation::Horizontal {
                    rect.width
                } else {
                    rect.height
                }
            };
            assert!((length(bounds(app, first)) - 300.0).abs() < 0.1);
            assert!((length(bounds(app, second)) - 300.0).abs() < 0.1);
            assert!((length(bounds(app, handle)) - 6.0).abs() < 0.1);
            app.scope().action(Action::Focus(handle)).unwrap();
            app.flush().unwrap();
            assert_eq!(query(app, Query::Focus(handle)), QueryValue::Focus(true));
            app.dispatch(
                handle,
                Event::Range {
                    from: handle,
                    command: RangeCommand::Set(25.0),
                },
            )
            .unwrap();
            app.settle(16).unwrap();
            assert!((length(bounds(app, first)) - 150.0).abs() < 0.1);
            assert!((length(bounds(app, second)) - 450.0).abs() < 0.1);
            let total = length(bounds(app, first))
                + length(bounds(app, second))
                + length(bounds(app, handle));
            assert!((total - 606.0).abs() < 0.1);
            assert_eq!(
                host.renderer()
                    .engine_id(editor, glory::html::Part::Main)
                    .unwrap(),
                identity
            );
            app.dispatch(
                handle,
                Event::Range {
                    from: handle,
                    command: RangeCommand::Maximum,
                },
            )
            .unwrap();
            app.settle(16).unwrap();
            assert!((length(bounds(app, first)) - 540.0).abs() < 0.1);
            app.clear().unwrap();
            assert!(app.tree().is_empty());
        }
    }
}
#[test]
fn splitter_native_controlled_rejection_and_disabled_focus_requests_keep_value() {
    let host = WindowApp::new();
    let app = host.app();
    let source = app.scope().cage(40.0);
    let enabled = app.scope().cage(true);
    let handle = app.scope().cage(None);
    let requested = app.scope().cage(0.0);
    host.mount(
        splitter("controlled")
            .value(source)
            .enabled(enabled)
            .handle_ref(handle)
            .on_value_change(move |value| requested.revise(|old| *old = value))
            .style(Style::new().width(Length::Px(606.0))),
    )
    .unwrap();
    app.settle(16).unwrap();
    let handle = handle.get().unwrap();
    app.dispatch(
        handle,
        Event::Range {
            from: handle,
            command: RangeCommand::Increase,
        },
    )
    .unwrap();
    assert_eq!(requested.get(), 41.0);
    assert_eq!(source.get(), 40.0);
    enabled.revise(|value| *value = false);
    app.settle(16).unwrap();
    assert!(app.scope().action(Action::Focus(handle)).is_err());
    assert!(!app.tree().enabled(handle));
    app.clear().unwrap();
}
