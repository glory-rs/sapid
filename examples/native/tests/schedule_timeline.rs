use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, GridCommand, NodeId, Props, Query, QueryValue, Rect};
use sapid::prelude::*;
use std::{cell::RefCell, rc::Rc};
fn d(s: &str) -> CivilDate {
    CivilDate::parse_iso(s).unwrap()
}
fn dt(s: &str) -> LocalDateTime {
    parse_local_date_time(s).unwrap()
}
fn data() -> ScheduleData {
    ScheduleData::new(
        ScheduleZone::Civil,
        ScheduleWindow::days(d("2024-02-26"), 7).unwrap(),
        [
            ScheduleEvent::new(
                ScheduleKey::new("a").unwrap(),
                "Planning",
                ScheduleInterval::new(dt("2024-02-29T09:00"), dt("2024-02-29T10:00")).unwrap(),
            )
            .unwrap(),
            ScheduleEvent::new(
                ScheduleKey::new("b").unwrap(),
                "Review",
                ScheduleInterval::new(dt("2024-02-29T09:30"), dt("2024-02-29T10:30")).unwrap(),
            )
            .unwrap(),
            ScheduleEvent::new(
                ScheduleKey::new("c").unwrap(),
                "Interview",
                ScheduleInterval::new(dt("2024-02-29T09:45"), dt("2024-02-29T10:30")).unwrap(),
            )
            .unwrap(),
        ],
    )
    .unwrap()
}
fn query(app: &App, q: Query) -> QueryValue {
    let r = Rc::new(RefCell::new(None));
    let out = r.clone();
    app.request(q, move |v| *out.borrow_mut() = Some(v.unwrap()))
        .unwrap();
    app.pump().unwrap();
    r.borrow_mut().take().unwrap()
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
            {
                let v = tree.node(*id).unwrap().descriptor.view();
                matches!(v.props,Props::Button(p)if p.label==name||p.label.starts_with(name))
                    || v.name.as_ref().is_some_and(|n| n.starts_with(name))
            }
        })
        .unwrap()
}
#[test]
fn native_week_timeline_four_themes_actual_three_lanes_hit_pixels_keyboard_reveal_and_focus() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(1000, 800, 1.).unwrap();
        let app = host.app();
        let grid = app.scope().cage(None);
        let viewport = app.scope().cage(None);
        let selected = app.scope().cage(None);
        host.mount(
            week_view("native-week", data())
                .theme(theme.clone())
                .grid_ref(grid)
                .viewport_ref(viewport)
                .selection(selected)
                .on_selection_change(move |v| selected.revise(|old| *old = Some(v))),
        )
        .unwrap();
        app.settle(16).unwrap();
        let planning = button(app, "Planning");
        let review = button(app, "Review");
        let interview = button(app, "Interview");
        let a = bounds(app, planning);
        let b = bounds(app, review);
        let c = bounds(app, interview);
        assert!(a.width >= 37. && a.width <= 39. && a.height >= 63. && a.height <= 65.);
        assert!((b.x - a.x - 40.).abs() < 0.1 && (c.x - b.x - 40.).abs() < 0.1);
        assert!((b.y - a.y - 32.).abs() < 0.1);
        let hit = host.renderer().inspect(|doc| {
            doc.hit((a.x + 5.) as f32, (a.y + 5.) as f32)
                .unwrap()
                .node_id
        });
        assert_eq!(
            hit,
            host.renderer()
                .engine_id(planning, glory::html::Part::Main)
                .unwrap()
        );
        app.dispatch(planning, Event::Activate).unwrap();
        app.settle(16).unwrap();
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let frame = raster.frame().unwrap();
        let at = ((a.y as usize + 3) * frame.width as usize + a.x as usize + 3) * 4;
        for (x, y) in raster.pixels()[at..at + 4].iter().zip([
            theme.accent.0,
            theme.accent.1,
            theme.accent.2,
            theme.accent.3,
        ]) {
            assert!(x.abs_diff(y) <= 2);
        }
        let first = button(app, "Monday, February 26, 2024 08:00:00");
        app.scope().action(Action::Focus(first)).unwrap();
        app.settle(16).unwrap();
        app.dispatch(
            grid.get().unwrap(),
            Event::GridNavigate {
                from: first,
                command: GridCommand::NextLargePage,
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        let last = button(app, "Sunday, March 3, 2024 17:30:00");
        assert_eq!(query(app, Query::Focus(last)), QueryValue::Focus(true));
        let QueryValue::ScrollMetrics(metrics) =
            query(app, Query::ScrollMetrics(viewport.get().unwrap()))
        else {
            panic!()
        };
        assert!(metrics.y > 100.);
        let visible = bounds(app, viewport.get().unwrap());
        let last_rect = bounds(app, last);
        assert!(
            last_rect.y >= visible.y
                && last_rect.y + last_rect.height <= visible.y + visible.height + 0.5
        );
        app.dispatch(last, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(
            matches!(selected.get(),Some(ScheduleSelection::Slot{interval,..})if interval.start()==dt("2024-03-03T17:30"))
        );
        app.clear().unwrap();
        app.pump().unwrap();
        assert_eq!(app.stats().queries, 0);
    }
}
#[test]
fn native_timeline_readonly_focus_owner_cleanup_and_slot_identity_after_page_data() {
    let host = WindowApp::new();
    host.renderer().set_viewport(1000, 800, 1.).unwrap();
    let app = host.app();
    let visible = app.scope().cage(true);
    let source = app.scope().cage(data());
    let readonly = app.scope().cage(true);
    let grid = app.scope().cage(None);
    let viewport = app.scope().cage(None);
    let selected = app.scope().cage(None);
    host.mount(glory::ui::when(visible, move || {
        week_view("native-replace", source)
            .grid_ref(grid)
            .viewport_ref(viewport)
            .readonly(readonly)
            .on_selection_change(move |v| selected.revise(|old| *old = Some(v)))
    }))
    .unwrap();
    app.settle(16).unwrap();
    let first = button(app, "Monday, February 26, 2024 08:00:00");
    let physical = host
        .renderer()
        .engine_id(first, glory::html::Part::Main)
        .unwrap();
    app.scope().action(Action::Focus(first)).unwrap();
    app.settle(16).unwrap();
    app.dispatch(first, Event::Activate).unwrap();
    app.settle(16).unwrap();
    assert_eq!(selected.get(), None);
    assert_eq!(query(app, Query::Focus(first)), QueryValue::Focus(true));
    source.revise(|value| {
        *value = ScheduleData::new(ScheduleZone::Civil, value.window().shifted(1).unwrap(), [])
            .unwrap()
    });
    app.settle(16).unwrap();
    let next = button(app, "Monday, March 4, 2024 08:00:00");
    assert_eq!(next, first);
    assert_eq!(
        host.renderer()
            .engine_id(next, glory::html::Part::Main)
            .unwrap(),
        physical
    );
    assert_eq!(query(app, Query::Focus(next)), QueryValue::Focus(true));
    visible.revise(|v| *v = false);
    app.settle(16).unwrap();
    assert!(grid.get().is_none() && viewport.get().is_none());
    assert_eq!(app.stats().queries, 0);
    assert!(!app.dispatch(first, Event::Activate).unwrap());
}
