use glory::apace::shell::WindowApp;
use glory::prelude::*;
use glory::render::{Action, NodeId, Query, QueryValue, Rect};
use glory::style::{Length, Overflow, Style};
use sapid::{AffixHorizontal, AffixOffset, AffixVertical, affix, affix_container};
use std::{cell::RefCell, rc::Rc};
fn query(app: &App, query: Query) -> QueryValue {
    let output = Rc::new(RefCell::new(None));
    let saved = output.clone();
    app.request(query, move |value| {
        *saved.borrow_mut() = Some(value.unwrap())
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
fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 0.5, "{actual} != {expected}");
}
#[test]
fn native_fixed_four_themes_preserves_viewport_offsets_during_real_scroll_resize_and_visibility() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(800, 600, 1.0).unwrap();
        let app = host.app();
        let fixed = app.scope().cage(None);
        let viewport = app.scope().cage(None);
        let input = app.scope().cage(None);
        let visible = app.scope().cage(true);
        host.mount(
            glory::ui::view()
                .node_ref(viewport)
                .style(
                    Style::new()
                        .width(Length::Px(300.0))
                        .height(Length::Px(180.0))
                        .overflow_y(Overflow::Auto),
                )
                .fill(
                    glory::ui::view()
                        .style(Style::new().height(Length::Px(1000.0)))
                        .fill(
                            affix("fixed")
                                .theme(theme)
                                .visible(visible)
                                .node_ref(fixed)
                                .offset(AffixOffset {
                                    horizontal: AffixHorizontal::Right(40.0),
                                    vertical: AffixVertical::Bottom(30.0),
                                })
                                .style(
                                    Style::new()
                                        .width(Length::Px(100.0))
                                        .height(Length::Px(40.0)),
                                )
                                .fill(
                                    glory::ui::input()
                                        .value("Native floating draft")
                                        .node_ref(input),
                                ),
                        ),
                ),
        )
        .unwrap();
        app.settle(16).unwrap();
        let id = fixed.get().unwrap();
        let editor = input.get().unwrap();
        let identity = host
            .renderer()
            .engine_id(editor, glory::html::Part::Main)
            .unwrap();
        let initial = bounds(app, id);
        close(initial.x, 660.0);
        close(initial.y, 530.0);
        app.scope()
            .action(Action::ScrollTo {
                node: viewport.get().unwrap(),
                x: 0.0,
                y: 200.0,
            })
            .unwrap();
        app.settle(16).unwrap();
        let QueryValue::Scroll { y, .. } = query(app, Query::Scroll(viewport.get().unwrap()))
        else {
            panic!("scroll")
        };
        close(y, 200.0);
        let scrolled = bounds(app, id);
        close(scrolled.x, initial.x);
        close(scrolled.y, initial.y);
        host.renderer().set_viewport(1000, 700, 1.0).unwrap();
        let resized = bounds(app, id);
        close(resized.x, 860.0);
        close(resized.y, 630.0);
        visible.revise(|v| *v = false);
        app.flush().unwrap();
        visible.revise(|v| *v = true);
        app.settle(16).unwrap();
        assert_eq!(input.get(), Some(editor));
        assert_eq!(
            host.renderer()
                .engine_id(editor, glory::html::Part::Main)
                .unwrap(),
            identity
        );
        app.clear().unwrap();
        assert!(fixed.get().is_none() && input.get().is_none());
    }
}
#[test]
fn native_absolute_affix_uses_the_explicit_relative_container() {
    let host = WindowApp::new();
    let app = host.app();
    let root = app.scope().cage(None);
    let fixed = app.scope().cage(None);
    host.mount(
        affix_container("region")
            .node_ref(root)
            .style(
                Style::new()
                    .width(Length::Px(300.0))
                    .height(Length::Px(200.0)),
            )
            .content(move |context| {
                affix("contained").within(context).node_ref(fixed).style(
                    Style::new()
                        .width(Length::Px(100.0))
                        .height(Length::Px(40.0)),
                )
            }),
    )
    .unwrap();
    let region = bounds(app, root.get().unwrap());
    let item = bounds(app, fixed.get().unwrap());
    close(item.x - region.x, 184.0);
    close(item.y - region.y, 144.0);
}
#[test]
fn native_sticky_affix_is_explicitly_rejected_and_does_not_claim_fallback_layout() {
    let host = WindowApp::new();
    let error = host
        .mount(affix_container("sticky").content(|context| affix("sticky-child").sticky(context)))
        .unwrap_err();
    assert!(
        matches!(
            error,
            glory::ui::Error::Render(glory::render::Error::UnsupportedStyle {
                requirement: glory::style::Unsupported {
                    value: Some(glory::style::Value::Sticky),
                    ..
                },
                ..
            })
        ),
        "{error:?}"
    );
}
