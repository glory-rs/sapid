use glory::apace::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{NodeId, Props, Query, QueryValue, Rect};
use sapid::prelude::*;
use std::{cell::RefCell, rc::Rc};
fn d(value: &str) -> CivilDate {
    CivilDate::parse_iso(value).unwrap()
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
    let mut pending = tree.roots().to_vec();
    let mut result = Vec::new();
    while let Some(id) = pending.pop() {
        pending.extend(tree.node(id).unwrap().children.iter().copied());
        result.push(id);
    }
    result
}
fn day(app: &App, date: CivilDate) -> NodeId {
    let name = DateLocale::En.date_label(date);
    all(app)
        .into_iter()
        .find(|id| {
            let tree = app.tree();
            let v = tree.node(*id).unwrap().descriptor.view();
            matches!(v.props, Props::Button(_)) && v.name.as_deref() == Some(name.as_str())
        })
        .unwrap()
}
#[test]
fn native_calendar_four_themes_have_actual_42_cell_geometry_selection_pixels_focus_and_stable_shells()
 {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(720, 540, 1.).unwrap();
        let app = host.app();
        let value = app.scope().cage(Some(d("2024-02-28")));
        let month = app.scope().cage(d("2024-02-01").year_month());
        let reference = app.scope().cage(None);
        host.mount(
            calendar(
                "actual",
                DatesProvider::new(d("2024-02-29")).today(Some(d("2024-02-29"))),
            )
            .theme(theme.clone())
            .name("Actual calendar")
            .value(value)
            .month(month)
            .node_ref(reference)
            .on_value_change(move |v| value.revise(|old| *old = v))
            .on_month_change(move |v| month.revise(|old| *old = v)),
        )
        .unwrap();
        app.settle(16).unwrap();
        let grid = reference.get().unwrap();
        let selected = day(app, d("2024-02-28"));
        let next = day(app, d("2024-02-29"));
        let rect = bounds(app, selected);
        let right = bounds(app, next);
        assert!((rect.width - 36.).abs() < 0.2 && (rect.height - 36.).abs() < 0.2);
        assert!((right.x - rect.x - 38.).abs() < 0.2);
        let cells = all(app)
            .into_iter()
            .filter(|id| {
                app.tree().node(*id).unwrap().descriptor.view().role == Some(Role::GridCell)
            })
            .collect::<Vec<_>>();
        assert_eq!(cells.len(), 42);
        let physical = host
            .renderer()
            .engine_id(grid, glory::html::Part::Main)
            .unwrap();
        let physical_day = host
            .renderer()
            .engine_id(selected, glory::html::Part::Main)
            .unwrap();
        let hit = host.renderer().inspect(|dom| {
            dom.hit(
                (rect.x + rect.width / 2.) as f32,
                (rect.y + rect.height / 2.) as f32,
            )
            .unwrap()
            .node_id
        });
        assert_eq!(hit, physical_day);
        app.scope()
            .action(glory::render::Action::Focus(selected))
            .unwrap();
        app.flush().unwrap();
        assert_eq!(query(app, Query::Focus(selected)), QueryValue::Focus(true));
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let offset = (((rect.y + 5.).round() as usize)
            * raster.frame().unwrap().width as usize
            + (rect.x + 5.).round() as usize)
            * 4;
        for (a, b) in raster.pixels()[offset..offset + 4].iter().zip([
            theme.accent.0,
            theme.accent.1,
            theme.accent.2,
            theme.accent.3,
        ]) {
            assert!(a.abs_diff(b) <= 2);
        }
        app.dispatch(
            grid,
            Event::GridNavigate {
                from: selected,
                command: GridCommand::Down,
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        assert_eq!(month.get(), d("2024-03-01").year_month());
        let focused = day(app, d("2024-03-06"));
        assert_eq!(query(app, Query::Focus(focused)), QueryValue::Focus(true));
        assert_eq!(value.get(), Some(d("2024-02-28")));
        app.dispatch(focused, Event::Activate).unwrap();
        assert_eq!(value.get(), Some(d("2024-03-06")));
        assert_eq!(
            host.renderer()
                .engine_id(grid, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        app.clear().unwrap();
        assert!(reference.get().is_none());
        assert_eq!(app.stats().queries, 0);
    }
}
#[test]
fn native_mini_rtl_boundary_and_readonly_calendar_keep_real_buttons_and_finite_layout() {
    let host = WindowApp::new();
    host.renderer().set_viewport(600, 350, 1.).unwrap();
    let app = host.app();
    let root = app.scope().cage(None);
    let readonly = app.scope().cage(true);
    let value = app.scope().cage(None);
    host.mount(
        mini_calendar(
            "rtl",
            DatesProvider::new(d("2024-02-29")).week_start(Weekday::Sunday),
        )
        .direction(glory::render::Direction::Rtl)
        .readonly(readonly)
        .value(value)
        .node_ref(root)
        .on_value_change(move |v| value.revise(|old| *old = v)),
    )
    .unwrap();
    let sunday = day(app, d("2024-02-25"));
    let monday = day(app, d("2024-02-26"));
    let a = bounds(app, sunday);
    let b = bounds(app, monday);
    assert!((a.width - 24.).abs() < 0.2);
    assert!(a.x > b.x);
    app.dispatch(monday, Event::Activate).unwrap();
    assert!(value.get().is_none());
    app.scope()
        .action(glory::render::Action::Focus(monday))
        .unwrap();
    app.flush().unwrap();
    app.dispatch(
        root.get().unwrap(),
        Event::GridNavigate {
            from: monday,
            command: GridCommand::Right,
        },
    )
    .unwrap();
    app.settle(12).unwrap();
    assert_eq!(
        query(app, Query::Focus(day(app, d("2024-02-27")))),
        QueryValue::Focus(true)
    );
    app.clear().unwrap();
    host.mount(
        mini_calendar("edge", DatesProvider::new(CivilDate::MAX))
            .unstyled()
            .node_ref(root),
    )
    .unwrap();
    let rect = bounds(app, root.get().unwrap());
    assert!(rect.width >= 179. && rect.height > 150.);
    let count = all(app)
        .into_iter()
        .filter(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|c| c == "sapid-calendar-day")
        })
        .count();
    assert!(count < 42);
    app.clear().unwrap();
    assert_eq!(app.stats().queries, 0);
}
