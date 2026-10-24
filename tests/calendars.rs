#![cfg(feature = "dates")]
use glory::prelude::*;
use glory::render::{EventKind, NodeId, Props};
use sapid::prelude::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn d(s: &str) -> CivilDate {
    CivilDate::parse_iso(s).unwrap()
}
fn p() -> DatesProvider {
    DatesProvider::new(d("2024-02-29")).today(Some(d("2024-02-29")))
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
fn class(app: &App, class: &str) -> Vec<NodeId> {
    all(app)
        .into_iter()
        .filter(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|c| c == class)
        })
        .collect()
}
fn day(app: &App, date: CivilDate) -> NodeId {
    let name = DateLocale::En.date_label(date);
    all(app)
        .into_iter()
        .find(|id| {
            let tree = app.tree();
            let view = tree.node(*id).unwrap().descriptor.view();
            matches!(view.props, Props::Button(_))
                && view.name.as_deref() == Some(name.as_str())
        })
        .unwrap_or_else(|| panic!("missing {date}"))
}
fn tab(app: &App) -> Vec<NodeId> {
    class(app, "sapid-calendar-day")
        .into_iter()
        .filter(|id| {
            app.tree().node(*id).unwrap().descriptor.view().focus == FocusMode::TabStop
        })
        .collect()
}
fn command(app: &App, root: NodeId, from: NodeId, command: GridCommand) {
    app.dispatch(root, Event::GridNavigate { from, command })
        .unwrap();
}
#[test]
fn constant_real_grid_has_day_buttons_headers_independent_selected_and_today() {
    let app = App::memory();
    let root = app.scope().cage(None);
    app.mount(
        calendar("dates", p())
            .default_value(Some(d("2024-02-28")))
            .node_ref(root),
    )
    .unwrap();
    assert_eq!(class(&app, "sapid-calendar-cell").len(), 42);
    assert_eq!(class(&app, "sapid-calendar-day").len(), 42);
    assert_eq!(class(&app, "sapid-calendar-weekdays").len(), 1);
    let id = root.get().unwrap();
    assert_eq!(
        app.tree().node(id).unwrap().descriptor.view().role,
        Some(Role::Grid)
    );
    let selected = day(&app, d("2024-02-28"));
    let today = day(&app, d("2024-02-29"));
    let tree = app.tree();
    let mut cell = tree.node(selected).unwrap().parent.unwrap();
    while tree.node(cell).unwrap().descriptor.view().role != Some(Role::GridCell) {
        cell = tree.node(cell).unwrap().parent.unwrap();
    }
    assert_eq!(
        tree.node(cell)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .selected,
        Some(true)
    );
    assert_eq!(
        tree.node(today)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .current,
        Some(Current::Date)
    );
    assert!(
        tree.node(today)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .selected
            .is_none()
    );
    drop(tree);
    assert_eq!(tab(&app), [selected]);
}
#[test]
fn same_tick_arrows_then_activation_use_the_bounded_cursor_and_do_not_select_old_physical_origin()
 {
    let app = App::memory();
    let root = app.scope().cage(None);
    let value = app.scope().cage(None);
    let month = app.scope().cage(d("2024-02-01").year_month());
    app.mount(
        calendar("dates", p())
            .value(value)
            .month(month)
            .node_ref(root)
            .on_value_change(move |v| value.revise(|old| *old = v))
            .on_month_change(move |v| month.revise(|old| *old = v)),
    )
    .unwrap();
    let grid = root.get().unwrap();
    let origin = day(&app, d("2024-02-29"));
    app.scope()
        .batch(|| {
            command(&app, grid, origin, GridCommand::Right);
            command(&app, grid, origin, GridCommand::Right);
            app.dispatch(origin, Event::Activate).unwrap();
        })
        .unwrap();
    assert_eq!(value.get(), Some(d("2024-03-02")));
    assert_eq!(month.get(), d("2024-03-01").year_month());
    assert_eq!(tab(&app), [day(&app, d("2024-03-02"))]);
}
#[test]
fn controlled_month_refusal_keeps_accepted_cells_and_external_invalid_month_recovers() {
    let app = App::memory();
    let root = app.scope().cage(None);
    let month = app.scope().cage(d("2024-02-01").year_month());
    let calls = Rc::new(RefCell::new(Vec::new()));
    let out = calls.clone();
    let bounds = DateBounds::new(Some(d("2024-02-01")), Some(d("2024-03-31"))).unwrap();
    app.mount(
        calendar("dates", p().bounds(bounds))
            .month(month)
            .node_ref(root)
            .on_month_change(move |v| out.borrow_mut().push(v)),
    )
    .unwrap();
    let grid = root.get().unwrap();
    let origin = day(&app, d("2024-02-29"));
    command(&app, grid, origin, GridCommand::NextPage);
    assert_eq!(calls.borrow().as_slice(), [d("2024-03-01").year_month()]);
    assert_eq!(tab(&app), [origin]);
    assert!(class(&app, "sapid-calendar-day").contains(&origin));
    month.revise(|v| *v = d("2025-02-01").year_month());
    assert!(app.flush().is_err());
    assert_eq!(tab(&app), [origin]);
    month.revise(|v| *v = d("2024-03-01").year_month());
    app.flush().unwrap();
    assert_eq!(tab(&app), [day(&app, d("2024-03-29"))]);
    assert_eq!(calls.borrow().len(), 1);
}
#[test]
fn weekday_exclusions_bounds_and_readonly_preserve_navigation_without_business_writes() {
    let app = App::memory();
    let root = app.scope().cage(None);
    let readonly = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    let value = app.scope().cage(None);
    app.mount(
        calendar("dates", p())
            .value(value)
            .readonly(readonly)
            .enabled(enabled)
            .disable_weekday(Weekday::Saturday)
            .disable_weekday(Weekday::Sunday)
            .exclude_date(d("2024-03-01"))
            .node_ref(root)
            .on_value_change(move |v| value.revise(|old| *old = v)),
    )
    .unwrap();
    let grid = root.get().unwrap();
    let origin = day(&app, d("2024-02-29"));
    command(&app, grid, origin, GridCommand::Right);
    assert_eq!(tab(&app), [day(&app, d("2024-03-04"))]);
    readonly.revise(|v| *v = true);
    app.flush().unwrap();
    let date = day(&app, d("2024-03-04"));
    app.dispatch(date, Event::Activate).unwrap();
    assert!(value.get().is_none());
    command(&app, grid, date, GridCommand::Down);
    assert_eq!(tab(&app), [day(&app, d("2024-03-11"))]);
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(class(&app, "sapid-calendar-day").iter().all(|id| {
        !app.tree()
            .node(*id)
            .unwrap()
            .descriptor
            .view()
            .props
            .enabled()
    }));
}
#[test]
fn finite_exclusion_admission_consumes_only_the_sentinel_and_year_edges_do_not_create_fake_dates()
 {
    let seen = Rc::new(Cell::new(0));
    let out = seen.clone();
    let iter = (0..1_000_000).map(move |_| {
        out.set(out.get() + 1);
        CivilDate::MIN
    });
    let app = App::memory();
    assert!(
        app.mount(calendar("dates", p()).exclude_dates(iter))
            .is_err()
    );
    assert_eq!(seen.get(), MAX_CALENDAR_EXCLUDED_DATES + 1);
    let app = App::memory();
    app.mount(calendar(
        "edge",
        DatesProvider::new(CivilDate::MIN).week_start(Weekday::Sunday),
    ))
    .unwrap();
    assert_eq!(class(&app, "sapid-calendar-cell").len(), 42);
    assert_eq!(class(&app, "sapid-calendar-day").len(), 41);
    assert_eq!(tab(&app), [day(&app, CivilDate::MIN)]);
}
#[test]
fn repeated_local_selection_can_deselect_in_one_tick_and_owner_retirement_stops_observers() {
    let app = Rc::new(App::memory());
    let root = app.scope().cage(None);
    let values = Rc::new(RefCell::new(Vec::new()));
    let out = values.clone();
    app.mount(
        calendar("dates", p())
            .allow_deselect(true)
            .node_ref(root)
            .on_value_change(move |v| out.borrow_mut().push(v)),
    )
    .unwrap();
    let id = day(&app, d("2024-02-29"));
    app.scope()
        .batch(|| {
            app.dispatch(id, Event::Activate).unwrap();
            app.dispatch(id, Event::Activate).unwrap();
        })
        .unwrap();
    assert_eq!(values.borrow().as_slice(), [Some(d("2024-02-29")), None]);
    app.clear().unwrap();
    assert!(root.get().is_none());
    assert_eq!(app.stats().queries, 0);
    let weak = Rc::downgrade(&app);
    let observed = Rc::new(Cell::new(0));
    let out = observed.clone();
    app.mount(
        calendar("again", p())
            .node_ref(root)
            .on_value_change(move |_| weak.upgrade().unwrap().clear().unwrap())
            .on(EventKind::Activate, move |_| out.set(out.get() + 1)),
    )
    .unwrap();
    let id = day(&app, d("2024-02-29"));
    app.dispatch(id, Event::Activate).unwrap();
    assert!(root.get().is_none());
    assert_eq!(observed.get(), 0);
}
