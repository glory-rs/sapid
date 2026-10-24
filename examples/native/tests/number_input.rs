use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, NodeId, Props, Query, QueryValue, RangeCommand, Rect};
use sapid::{Decimal, NumberValue, number_input};
use std::{cell::RefCell, rc::Rc};
fn d(value: &str) -> Decimal {
    Decimal::parse(value).unwrap()
}
fn query(app: &App, value: Query) -> QueryValue {
    let reply = Rc::new(RefCell::new(None));
    let out = reply.clone();
    app.request(value, move |v| *out.borrow_mut() = Some(v.unwrap()))
        .unwrap();
    app.pump().unwrap();
    reply.borrow_mut().take().unwrap()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let QueryValue::Bounds(v) = query(app, Query::Bounds(id)) else {
        panic!()
    };
    v
}
fn text(app: &App, id: NodeId) -> String {
    let tree = app.tree();
    let Props::Input(v) = tree.node(id).unwrap().descriptor.view().props else {
        panic!()
    };
    v.value.clone()
}
#[test]
fn native_number_four_themes_have_real_editor_focus_pixels_and_exact_same_tick_steps() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(700, 300, 1.).unwrap();
        let app = host.app();
        let source = app.scope().cage(NumberValue::Decimal(d("0.1")));
        let reference = app.scope().cage(None);
        host.mount(
            number_input()
                .name("Exact amount")
                .theme(theme.clone())
                .value(source)
                .step(d("0.1"))
                .node_ref(reference)
                .on_value_change(move |v| source.revise(|old| *old = v)),
        )
        .unwrap();
        app.settle(16).unwrap();
        let id = reference.get().unwrap();
        let identity = host
            .renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap();
        let rect = bounds(app, id);
        assert!(rect.width >= 179. && rect.height >= 35.);
        app.scope().action(Action::Focus(id)).unwrap();
        app.flush().unwrap();
        assert_eq!(query(app, Query::Focus(id)), QueryValue::Focus(true));
        app.scope()
            .batch(|| {
                for _ in 0..2 {
                    app.dispatch(
                        id,
                        Event::Spin {
                            from: id,
                            command: RangeCommand::Increase,
                        },
                    )
                    .unwrap();
                }
            })
            .unwrap();
        assert_eq!(source.get(), NumberValue::Decimal(d("0.3")));
        assert_eq!(text(app, id), "0.3");
        assert_eq!(
            host.renderer()
                .engine_id(id, glory::html::Part::Main)
                .unwrap(),
            identity
        );
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let offset = ((rect.y + rect.height / 2.).round() as usize
            * raster.frame().unwrap().width as usize
            + (rect.x + rect.width - 9.).round() as usize)
            * 4;
        let actual = &raster.pixels()[offset..offset + 4];
        let expected = [theme.input.0, theme.input.1, theme.input.2, theme.input.3];
        for (a, b) in actual.iter().zip(expected) {
            assert!(a.abs_diff(b) <= 2, "{actual:?} {expected:?}");
        }
        app.clear().unwrap();
        assert!(reference.get().is_none());
        assert_eq!(app.stats().queries, 0);
    }
}
#[test]
fn native_number_disabled_readonly_and_unstyled_geometry_keep_the_same_input() {
    let host = WindowApp::new();
    let app = host.app();
    let reference = app.scope().cage(None);
    let active = app.scope().cage(true);
    let readonly = app.scope().cage(false);
    host.mount(
        number_input()
            .name("Plain exact amount")
            .unstyled()
            .enabled(active)
            .readonly(readonly)
            .default_value(d("2"))
            .node_ref(reference),
    )
    .unwrap();
    app.settle(16).unwrap();
    let id = reference.get().unwrap();
    let identity = host
        .renderer()
        .engine_id(id, glory::html::Part::Main)
        .unwrap();
    let rect = bounds(app, id);
    assert!(rect.width >= 179. && rect.height >= 35.);
    readonly.revise(|v| *v = true);
    app.settle(16).unwrap();
    app.scope().action(Action::Focus(id)).unwrap();
    app.flush().unwrap();
    assert_eq!(query(app, Query::Focus(id)), QueryValue::Focus(true));
    assert!(
        app.dispatch(
            id,
            Event::Spin {
                from: id,
                command: RangeCommand::Increase
            }
        )
        .is_err()
    );
    assert_eq!(text(app, id), "2");
    readonly.revise(|v| *v = false);
    active.revise(|v| *v = false);
    app.settle(16).unwrap();
    assert!(app.scope().action(Action::Focus(id)).is_err());
    active.revise(|v| *v = true);
    app.settle(16).unwrap();
    assert_eq!(
        host.renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap(),
        identity
    );
    app.dispatch(
        id,
        Event::Spin {
            from: id,
            command: RangeCommand::Decrease,
        },
    )
    .unwrap();
    assert_eq!(text(app, id), "1");
}
