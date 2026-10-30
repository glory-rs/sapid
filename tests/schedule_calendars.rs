#![cfg(feature = "dates")]
use glory::prelude::*;
use glory::render::{EventKind, FocusMode, GridCommand, NodeId, Props, TableMetadata};
use sapid::prelude::*;
fn d(s: &str) -> CivilDate {
    CivilDate::parse_iso(s).unwrap()
}
fn dt(s: &str) -> LocalDateTime {
    parse_local_date_time(s).unwrap()
}
fn data() -> ScheduleData {
    ScheduleData::new(
        ScheduleZone::Civil,
        ScheduleWindow::month(d("2024-02-01").year_month()).unwrap(),
        (0..5).map(|i| {
            ScheduleEvent::new(
                ScheduleKey::new(format!("event-{i}")).unwrap(),
                format!("Event {i}"),
                ScheduleInterval::new(dt("2024-02-28T09:00"), dt("2024-02-29T10:00")).unwrap(),
            )
            .unwrap()
        }),
    )
    .unwrap()
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
fn class(app: &App, name: &str) -> Vec<NodeId> {
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
                .any(|v| v == name)
        })
        .collect()
}
fn named(app: &App, prefix: &str) -> NodeId {
    all(app)
        .into_iter()
        .find(|id| {
            let tree = app.tree();
            let v = tree.node(*id).unwrap().descriptor.view();
            matches!(v.props, Props::Button(_))
                && v.name.as_ref().is_some_and(|n| n.starts_with(prefix))
        })
        .unwrap_or_else(|| panic!("missing {prefix}"))
}
fn button(app: &App, label: &str) -> NodeId {
    all(app).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(v)if v.label==label)).unwrap()
}
#[test]
fn exact_calendar_windows_handle_leap_lengths_and_reject_unrepresentable_exclusive_ends() {
    assert_eq!(
        ScheduleWindow::month(d("2024-02-10").year_month())
            .unwrap()
            .start()
            .date(),
        d("2024-02-01")
    );
    assert_eq!(
        ScheduleWindow::month(d("2024-02-10").year_month())
            .unwrap()
            .end()
            .date(),
        d("2024-03-01")
    );
    assert_eq!(
        ScheduleWindow::year(CivilYear::new(2024).unwrap())
            .unwrap()
            .start()
            .date()
            .days_until(d("2025-01-01")),
        366
    );
    assert!(ScheduleWindow::month(d("9999-12-01").year_month()).is_err());
    assert!(ScheduleWindow::year(CivilYear::MAX).is_err());
    let app = App::memory();
    assert!(
        app.mount(month_view(
            "bad",
            ScheduleData::new(
                ScheduleZone::Civil,
                ScheduleWindow::days(d("2024-02-01"), 28).unwrap(),
                []
            )
            .unwrap()
        ))
        .is_err()
    );
    assert!(app.tree().roots().is_empty());
}
#[test]
fn month_has_42_genuine_cells_with_exact_counts_cross_day_occurrences_and_bounded_preview() {
    let app = App::memory();
    let grid = app.scope().cage(None);
    app.mount(month_view("month", data()).grid_ref(grid))
        .unwrap();
    let tree = app.tree();
    let g = tree.node(grid.get().unwrap()).unwrap().descriptor.view();
    assert_eq!(g.role, Some(Role::Grid));
    assert_eq!(
        g.semantics.as_ref().unwrap().table,
        Some(TableMetadata::Table {
            rows: Some(7),
            columns: Some(7)
        })
    );
    drop(tree);
    assert_eq!(class(&app, "sapid-schedule-calendar-cell").len(), 42);
    assert_eq!(class(&app, "sapid-schedule-period-slot").len(), 42);
    assert_eq!(
        class(&app, "sapid-agenda-event").len(),
        6,
        "three previews on each overlapped date"
    );
    assert_eq!(class(&app, "sapid-schedule-more").len(), 2);
    let outside = named(&app, "Monday, January 29, 2024, 0 events");
    assert!(!app.tree().enabled(outside));
    let leap = named(&app, "Thursday, February 29, 2024, 5 events");
    app.dispatch(leap, Event::Activate).unwrap();
    let tree = app.tree();
    let cell = tree.node(leap).unwrap().parent.unwrap();
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
}
#[test]
fn only_one_current_disclosure_expands_and_stable_button_shell_survives_collapse() {
    let app = App::memory();
    app.mount(month_view("more", data())).unwrap();
    let a = named(&app, "Show events for 2024-02-28");
    let b = named(&app, "Show events for 2024-02-29");
    app.dispatch(a, Event::Activate).unwrap();
    assert_eq!(class(&app, "sapid-agenda-event").len(), 8);
    assert_eq!(
        app.tree()
            .node(a)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .expanded,
        Some(true)
    );
    app.dispatch(b, Event::Activate).unwrap();
    assert_eq!(class(&app, "sapid-agenda-event").len(), 8);
    assert_eq!(
        app.tree()
            .node(a)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .expanded,
        Some(false)
    );
    app.dispatch(b, Event::Activate).unwrap();
    assert_eq!(class(&app, "sapid-agenda-event").len(), 6);
    assert_eq!(named(&app, "Show events for 2024-02-29"), b);
    assert!(
        all(&app)
            .iter()
            .all(|id| app.tree().node(*id).unwrap().descriptor.view().role != Some(Role::Menu))
    );
}
#[test]
fn same_tick_navigation_activation_preserves_current_cursor_and_typed_before_generic() {
    let app = App::memory();
    let grid = app.scope().cage(None);
    let selected = app.scope().cage(None);
    let seen = app.scope().cage(None);
    app.mount(
        month_view("keys", data())
            .grid_ref(grid)
            .on_selection_change(move |v| selected.revise(|old| *old = Some(v)))
            .on(EventKind::Activate, move |_| {
                seen.revise(|v| *v = selected.get_untracked())
            }),
    )
    .unwrap();
    let from = named(&app, "Thursday, February 1, 2024, 0 events");
    let grid = grid.get().unwrap();
    app.scope()
        .batch(|| {
            for command in [GridCommand::Right, GridCommand::Right, GridCommand::Down] {
                app.dispatch(grid, Event::GridNavigate { from, command })
                    .unwrap();
            }
            app.dispatch(from, Event::Activate).unwrap();
        })
        .unwrap();
    assert!(
        matches!(selected.get(),Some(ScheduleSelection::Slot{interval,..})if interval.start().date()==d("2024-02-10")&&interval.end().date()==d("2024-02-11"))
    );
    assert_eq!(seen.get(), selected.get());
    assert_eq!(
        app.tree()
            .node(named(&app, "Saturday, February 10, 2024"))
            .unwrap()
            .descriptor
            .view()
            .focus,
        FocusMode::TabStop
    );
}
#[test]
fn calendar_requests_are_true_month_year_pages_without_duration_drift_or_implicit_accept() {
    let app = App::memory();
    let source = app.scope().cage(data());
    let requests = app.scope().cage(Vec::<ScheduleRequest>::new());
    app.mount(
        month_view("pages", source).on_window_request(move |v| requests.revise(|all| all.push(v))),
    )
    .unwrap();
    let next = button(&app, "Next schedule window");
    app.scope()
        .batch(|| {
            app.dispatch(next, Event::Activate).unwrap();
            app.dispatch(next, Event::Activate).unwrap();
        })
        .unwrap();
    assert_eq!(
        requests.with(|v| v
            .iter()
            .map(|r| r.window.start().date())
            .collect::<Vec<_>>()),
        vec![d("2024-03-01"), d("2024-04-01")]
    );
    assert_eq!(source.with(|v| v.window().start().date()), d("2024-02-01"));
    app.dispatch(next, Event::Activate).unwrap();
    assert_eq!(
        requests.with(|v| v.last().unwrap().window.start().date()),
        d("2024-03-01")
    );
    let app = App::memory();
    let request = app.scope().cage(None);
    app.mount(
        year_view(
            "year",
            ScheduleData::new(
                ScheduleZone::Civil,
                ScheduleWindow::year(CivilYear::new(2024).unwrap()).unwrap(),
                [],
            )
            .unwrap(),
        )
        .on_window_request(move |v| request.revise(|old| *old = Some(v))),
    )
    .unwrap();
    app.dispatch(button(&app, "Next schedule window"), Event::Activate)
        .unwrap();
    assert_eq!(
        request.with(|v| v.as_ref().unwrap().window.end().date()),
        d("2026-01-01")
    );
    assert_eq!(class(&app, "sapid-schedule-period-slot").len(), 12);
}
#[test]
fn mobile_focus_details_readonly_controlled_refusal_and_owner_retirement_are_separate() {
    let app = App::memory();
    let visible = app.scope().cage(true);
    let selected = app.scope().cage(None);
    let proposals = app.scope().cage(0);
    let readonly = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    let grid = app.scope().cage(None);
    app.mount(glory::ui::when(visible, move || {
        mobile_month_view("mobile", data())
            .selection(selected)
            .readonly(readonly)
            .enabled(enabled)
            .grid_ref(grid)
            .on_selection_change(move |_| proposals.revise(|v| *v += 1))
    }))
    .unwrap();
    let leap = named(&app, "Thursday, February 29, 2024");
    app.dispatch(leap, Event::FocusChanged(true)).unwrap();
    assert_eq!(class(&app, "sapid-agenda-event").len(), 5);
    app.dispatch(leap, Event::Activate).unwrap();
    assert_eq!(selected.get(), None);
    assert_eq!(proposals.get(), 1);
    readonly.revise(|v| *v = true);
    app.flush().unwrap();
    app.dispatch(leap, Event::Activate).unwrap();
    assert_eq!(proposals.get(), 1);
    assert!(app.tree().enabled(leap));
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(!app.tree().enabled(leap));
    visible.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(grid.get().is_none());
    assert!(!app.dispatch(leap, Event::Activate).unwrap());
    assert_eq!(app.stats().queries, 0);
}
#[test]
fn external_window_change_retains_slot_shells_and_clears_current_disclosure_without_proposals() {
    let app = App::memory();
    let source = app.scope().cage(data());
    let proposals = app.scope().cage(0);
    app.mount(
        month_view("update", source).on_selection_change(move |_| proposals.revise(|v| *v += 1)),
    )
    .unwrap();
    let slot = named(&app, "Monday, January 29, 2024");
    app.dispatch(named(&app, "Show events for 2024-02-29"), Event::Activate)
        .unwrap();
    source.revise(|v| {
        *v = ScheduleData::new(
            ScheduleZone::Civil,
            ScheduleWindow::month(d("2024-03-01").year_month()).unwrap(),
            [],
        )
        .unwrap()
    });
    app.flush().unwrap();
    assert_eq!(named(&app, "Monday, February 26, 2024"), slot);
    assert!(class(&app, "sapid-schedule-more").is_empty());
    assert!(class(&app, "sapid-agenda-event").is_empty());
    assert_eq!(proposals.get(), 0);
}
#[test]
fn arrow_edges_preserve_date_and_column_instead_of_clamping_to_another_cell() {
    let app = App::memory();
    let grid = app.scope().cage(None);
    let selection = app.scope().cage(None);
    app.mount(
        month_view("edges", data())
            .grid_ref(grid)
            .on_selection_change(move |v| selection.revise(|old| *old = Some(v))),
    )
    .unwrap();
    for (name, command, expected) in [
        ("Saturday, February 3, 2024", GridCommand::Up, "2024-02-03"),
        (
            "Tuesday, February 27, 2024",
            GridCommand::Down,
            "2024-02-27",
        ),
        (
            "Thursday, February 1, 2024",
            GridCommand::Left,
            "2024-02-01",
        ),
        (
            "Thursday, February 29, 2024",
            GridCommand::Right,
            "2024-02-29",
        ),
    ] {
        let from = named(&app, name);
        app.scope()
            .batch(|| {
                app.dispatch(grid.get().unwrap(), Event::GridNavigate { from, command })
                    .unwrap();
                app.dispatch(from, Event::Activate).unwrap();
            })
            .unwrap();
        assert!(selection.with(|v|matches!(v,Some(ScheduleSelection::Slot{interval,..})if interval.start().date()==d(expected))),"{name}/{command:?}: {:?}",selection.get());
    }
}
