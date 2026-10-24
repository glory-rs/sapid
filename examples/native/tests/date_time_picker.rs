use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, NodeId, Props, Query, QueryValue, RangeCommand, Rect};
use sapid::prelude::*;
use std::{cell::RefCell, rc::Rc};
fn d(s: &str) -> CivilDate {
    CivilDate::parse_iso(s).unwrap()
}
fn dt(s: &str) -> LocalDateTime {
    parse_local_date_time(s).unwrap()
}
fn query(app: &App, q: Query) -> QueryValue {
    let out = Rc::new(RefCell::new(None));
    let r = out.clone();
    app.request(q, move |v| *r.borrow_mut() = Some(v.unwrap()))
        .unwrap();
    app.pump().unwrap();
    out.borrow_mut().take().unwrap()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let QueryValue::Bounds(r) = query(app, Query::Bounds(id)) else {
        panic!()
    };
    r
}
fn all(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = todo.pop() {
        todo.extend(tree.node(id).unwrap().children.iter().copied());
        out.push(id)
    }
    out
}
fn button(app: &App, name: &str) -> NodeId {
    all(app)
        .into_iter()
        .find(|id| {
            let tree = app.tree();
            let v = tree.node(*id).unwrap().descriptor.view();
            matches!(v.props,Props::Button(p)if p.label==name)
                || matches!(v.props, Props::Button(_)) && v.name.as_deref() == Some(name)
        })
        .unwrap()
}
#[test]
fn native_date_time_dialog_four_themes_real_grid_spin_hit_pixels_and_escape_keep_value() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(900, 850, 1.).unwrap();
        let app = host.app();
        let input = app.scope().cage(None);
        let trigger = app.scope().cage(None);
        let panel = app.scope().cage(None);
        let hour = app.scope().cage(None);
        let value = app.scope().cage(Some(dt("2024-02-29T10:20")));
        let commits = app.scope().cage(0);
        host.mount(
            date_time_picker(
                "native-datetime",
                DatesProvider::new(d("2024-02-29")),
                DateTimePolicy::FixedOffset(FixedOffset::new(480).unwrap()),
            )
            .theme(theme.clone())
            .name("Appointment")
            .node_ref(input)
            .trigger_ref(trigger)
            .panel_ref(panel)
            .hour_ref(hour)
            .value(value)
            .on_value_change(move |v| value.revise(|old| *old = v))
            .on_commit(move |_| commits.revise(|n| *n += 1)),
        )
        .unwrap();
        app.settle(16).unwrap();
        let input = input.get().unwrap();
        let trigger = trigger.get().unwrap();
        let physical = host
            .renderer()
            .engine_id(input, glory::html::Part::Main)
            .unwrap();
        app.scope().action(Action::Focus(trigger)).unwrap();
        app.settle(16).unwrap();
        app.dispatch(trigger, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert_eq!(host.renderer().inspect(|doc| doc.top_layers().len()), 1);
        let rect = bounds(app, panel.get().unwrap());
        assert!(rect.width >= 300. && rect.height >= 350. && rect.x >= 0. && rect.y >= 0.);
        let selected = button(app, &DateLocale::En.date_label(d("2024-02-29")));
        let cell = bounds(app, selected);
        assert!(cell.width >= 35. && cell.height >= 30.);
        let hit = host.renderer().inspect(|doc| {
            doc.hit((cell.x + 10.) as f32, (cell.y + 10.) as f32)
                .unwrap()
                .node_id
        });
        assert_eq!(
            hit,
            host.renderer()
                .engine_id(selected, glory::html::Part::Main)
                .unwrap()
        );
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let frame = raster.frame().unwrap();
        let at = ((cell.y as usize + 5) * frame.width as usize + cell.x as usize + 5) * 4;
        for (a, b) in raster.pixels()[at..at + 4].iter().zip([
            theme.accent.0,
            theme.accent.1,
            theme.accent.2,
            theme.accent.3,
        ]) {
            assert!(a.abs_diff(b) <= 2);
        }
        let hour = hour.get().unwrap();
        let editor = bounds(app, hour);
        assert!(editor.width >= 70. && editor.height >= 30. && editor.y > cell.y);
        app.scope().action(Action::Focus(hour)).unwrap();
        app.settle(16).unwrap();
        assert_eq!(query(app, Query::Focus(hour)), QueryValue::Focus(true));
        app.dispatch(
            hour,
            Event::Spin {
                from: hour,
                command: RangeCommand::Increase,
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        assert_eq!(value.get(), Some(dt("2024-02-29T11:20")));
        app.dispatch(
            panel.get().unwrap(),
            Event::Dismiss(glory::render::DismissReason::Escape),
        )
        .unwrap();
        app.settle(16).unwrap();
        assert_eq!(value.get(), Some(dt("2024-02-29T11:20")));
        assert_eq!(commits.get(), 0);
        assert_eq!(query(app, Query::Focus(trigger)), QueryValue::Focus(true));
        app.dispatch(trigger, Event::Activate).unwrap();
        app.settle(16).unwrap();
        app.dispatch(button(app, "Done"), Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert_eq!(commits.get(), 1);
        assert_eq!(host.renderer().inspect(|doc| doc.top_layers().len()), 0);
        assert_eq!(
            host.renderer()
                .engine_id(input, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        app.clear().unwrap();
        app.pump().unwrap();
        assert_eq!(app.stats().queries, 0);
    }
}
#[test]
fn native_date_time_unmount_readonly_and_resolver_gap_do_not_replace_successful_iso() {
    let host = WindowApp::new();
    host.renderer().set_viewport(900, 850, 1.).unwrap();
    let app = host.app();
    let h = app.scope().cage(None);
    let input = app.scope().cage(None);
    let trigger = app.scope().cage(None);
    let panel = app.scope().cage(None);
    let value = app.scope().cage(Some(dt("2024-02-29T10:20")));
    let readonly = app.scope().cage(false);
    let policy = DateTimePolicy::application(AmbiguityPolicy::Reject, |local| {
        if local.time().hour() == 11 {
            LocalResolution::Gap
        } else {
            LocalResolution::Unique(ResolvedDateTime::from_fixed(
                local,
                FixedOffset::new(0).unwrap(),
            ))
        }
    });
    host.mount(
        date_time_picker("native-gap", DatesProvider::new(d("2024-02-29")), policy)
            .content_policy(ContentPolicy::Unmount)
            .value(value)
            .readonly(readonly)
            .hour_ref(h)
            .node_ref(input)
            .trigger_ref(trigger)
            .panel_ref(panel)
            .on_value_change(move |v| value.revise(|old| *old = v)),
    )
    .unwrap();
    app.settle(16).unwrap();
    assert_eq!(h.get(), None);
    app.dispatch(trigger.get().unwrap(), Event::Activate)
        .unwrap();
    app.settle(16).unwrap();
    let old = h.get().unwrap();
    app.dispatch(
        old,
        Event::Spin {
            from: old,
            command: RangeCommand::Increase,
        },
    )
    .unwrap();
    app.settle(16).unwrap();
    assert_eq!(value.get(), Some(dt("2024-02-29T10:20")));
    let tree = app.tree();
    let Props::Input(p) = tree
        .node(input.get().unwrap())
        .unwrap()
        .descriptor
        .view()
        .props
    else {
        panic!()
    };
    assert_eq!(p.value, "2024-02-29T10:20:00");
    drop(tree);
    readonly.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(
        app.dispatch(
            old,
            Event::Spin {
                from: old,
                command: RangeCommand::Increase
            }
        )
        .is_err()
    );
    app.dispatch(
        panel.get().unwrap(),
        Event::Dismiss(glory::render::DismissReason::Escape),
    )
    .unwrap();
    app.settle(16).unwrap();
    assert_eq!(h.get(), None);
    app.dispatch(trigger.get().unwrap(), Event::Activate)
        .unwrap();
    app.settle(16).unwrap();
    assert_ne!(h.get(), Some(old));
    app.clear().unwrap();
    app.pump().unwrap();
    assert_eq!(app.stats().queries, 0);
    assert_eq!(host.renderer().inspect(|doc| doc.top_layers().len()), 0);
}
