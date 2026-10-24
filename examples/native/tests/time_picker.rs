use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, InputProps, NodeId, Props, Query, QueryValue, Rect};
use sapid::prelude::*;
use std::{cell::RefCell, rc::Rc};
fn t(v: &str) -> LocalTime {
    LocalTime::parse_iso(v).unwrap()
}
fn query(app: &App, q: Query) -> QueryValue {
    let reply = Rc::new(RefCell::new(None));
    let out = reply.clone();
    app.request(q, move |v| *out.borrow_mut() = Some(v.unwrap()))
        .unwrap();
    app.pump().unwrap();
    reply.borrow_mut().take().unwrap()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let QueryValue::Bounds(rect) = query(app, Query::Bounds(id)) else {
        panic!()
    };
    rect
}
fn props(app: &App, id: NodeId) -> InputProps {
    let tree = app.tree();
    let Props::Input(p) = tree.node(id).unwrap().descriptor.view().props else {
        panic!()
    };
    p.clone()
}
fn all(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut result = vec![];
    while let Some(id) = todo.pop() {
        todo.extend(tree.node(id).unwrap().children.iter().copied());
        result.push(id)
    }
    result
}
fn button(app: &App, name: &str) -> NodeId {
    all(app)
        .into_iter()
        .find(|id| {
            let tree = app.tree();
            let view = tree.node(*id).unwrap().descriptor.view();
            matches!(view.props,Props::Button(p)if p.label==name)
        })
        .unwrap()
}
#[test]
fn native_time_segments_four_themes_have_real_spin_editors_geometry_focus_and_shared_grid_value()
 {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(800, 650, 1.).unwrap();
        let app = host.app();
        let h = app.scope().cage(None);
        let m = app.scope().cage(None);
        let s = app.scope().cage(None);
        let value = app.scope().cage(Some(t("09:30:00")));
        host.mount(
            time_picker("native-segments")
                .theme(theme.clone())
                .format(TimeFormat::new(
                    HourCycle::H24,
                    TimePrecision::Second,
                    DateLocale::En,
                ))
                .hour_ref(h)
                .minute_ref(m)
                .second_ref(s)
                .value(value)
                .grid(TimeGridSpec::new(t("09:00"), t("10:30"), 1800, 2).unwrap())
                .on_value_change(move |v| value.revise(|old| *old = v)),
        )
        .unwrap();
        app.settle(16).unwrap();
        let h = h.get().unwrap();
        let m = m.get().unwrap();
        let s = s.get().unwrap();
        let hr = bounds(app, h);
        let mr = bounds(app, m);
        let sr = bounds(app, s);
        assert!(hr.width >= 71. && hr.height >= 35.);
        assert!(mr.x >= hr.x + hr.width && sr.x >= mr.x + mr.width);
        let physical = host
            .renderer()
            .engine_id(h, glory::html::Part::Main)
            .unwrap();
        app.scope().action(Action::Focus(h)).unwrap();
        app.settle(16).unwrap();
        assert_eq!(query(app, Query::Focus(h)), QueryValue::Focus(true));
        app.dispatch(
            h,
            Event::Spin {
                command: glory::render::RangeCommand::Increase,
                from: h,
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        assert_eq!(value.get(), Some(t("10:30:00")));
        assert_eq!(props(app, h).value, "10");
        let slot = button(app, "09:00:00");
        let rect = bounds(app, slot);
        assert!(rect.width >= 89. && rect.height >= 35.);
        let hit = host.renderer().inspect(|dom| {
            dom.hit(
                (rect.x + rect.width / 2.) as f32,
                (rect.y + rect.height / 2.) as f32,
            )
            .unwrap()
            .node_id
        });
        assert_eq!(
            hit,
            host.renderer()
                .engine_id(slot, glory::html::Part::Main)
                .unwrap()
        );
        app.dispatch(slot, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert_eq!(value.get(), Some(t("09:00:00")));
        assert_eq!(props(app, m).value, "00");
        assert_eq!(props(app, s).value, "00");
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let frame = raster.frame().unwrap();
        let at = ((rect.y as usize + 5) * frame.width as usize + rect.x as usize + 5) * 4;
        for (a, b) in raster.pixels()[at..at + 4].iter().zip([
            theme.accent.0,
            theme.accent.1,
            theme.accent.2,
            theme.accent.3,
        ]) {
            assert!(a.abs_diff(b) <= 2);
        }
        assert_eq!(
            host.renderer()
                .engine_id(h, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        app.clear().unwrap();
        assert_eq!(app.stats().queries, 0);
    }
}
#[test]
fn native_time_grid_queued_navigation_skips_disabled_and_readonly_keeps_focus_without_value_change()
 {
    let host = WindowApp::new();
    host.renderer().set_viewport(800, 500, 1.).unwrap();
    let app = host.app();
    let root = app.scope().cage(None);
    let ro = app.scope().cage(false);
    let value = app.scope().cage(Some(t("09:00")));
    host.mount(
        time_grid("native-grid")
            .spec(TimeGridSpec::new(t("09:00"), t("11:30"), 1800, 3).unwrap())
            .node_ref(root)
            .value(value)
            .readonly(ro)
            .disable_time(t("09:30"))
            .on_value_change(move |v| value.revise(|old| *old = v)),
    )
    .unwrap();
    app.settle(16).unwrap();
    let from = button(app, "09:00");
    let target = button(app, "10:00");
    app.scope().action(Action::Focus(from)).unwrap();
    app.settle(16).unwrap();
    app.dispatch(
        root.get().unwrap(),
        Event::GridNavigate {
            from,
            command: GridCommand::Right,
        },
    )
    .unwrap();
    app.settle(16).unwrap();
    assert_eq!(query(app, Query::Focus(target)), QueryValue::Focus(true));
    assert_eq!(value.get(), Some(t("09:00")));
    ro.revise(|v| *v = true);
    app.settle(16).unwrap();
    app.dispatch(target, Event::Activate).unwrap();
    app.settle(16).unwrap();
    assert_eq!(value.get(), Some(t("09:00")));
    ro.revise(|v| *v = false);
    app.settle(16).unwrap();
    app.dispatch(target, Event::Activate).unwrap();
    app.settle(16).unwrap();
    assert_eq!(value.get(), Some(t("10:00")));
    app.clear().unwrap();
    assert!(root.get().is_none());
    assert_eq!(app.stats().queries, 0);
}
