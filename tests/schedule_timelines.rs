#![cfg(feature = "dates")]
use glory::prelude::*;
use glory::render::{Direction, EventKind, FocusMode, GridCommand, NodeId, Props, TableMetadata};
use sapid::prelude::*;
fn d(s: &str) -> CivilDate {
    CivilDate::parse_iso(s).unwrap()
}
fn dt(s: &str) -> LocalDateTime {
    parse_local_date_time(s).unwrap()
}
fn data(days: u16) -> ScheduleData {
    ScheduleData::new(
        ScheduleZone::Civil,
        ScheduleWindow::days(d("2024-02-26"), days).unwrap(),
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
            ScheduleEvent::new(
                ScheduleKey::new("holiday").unwrap(),
                "Holiday",
                ScheduleInterval::all_day(d("2024-03-01"), d("2024-03-02")).unwrap(),
            )
            .unwrap()
            .all_day()
            .unwrap(),
        ],
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
fn slot(app: &App, label: &str) -> NodeId {
    class(app, "sapid-schedule-slot")
        .into_iter()
        .find(|id| {
            let tree = app.tree();
            {
                let v = tree.node(*id).unwrap().descriptor.view();
                matches!(v.props,Props::Button(p)if p.label.contains(label))
                    || v.name.as_ref().is_some_and(|n| n.contains(label))
            }
        })
        .unwrap_or_else(|| panic!("missing {label}"))
}
#[test]
fn week_has_real_grid_slot_metadata_and_separate_exact_overlap_lanes() {
    let app = App::memory();
    let grid = app.scope().cage(None);
    app.mount(week_view("week", data(7)).grid_ref(grid))
        .unwrap();
    let tree = app.tree();
    let g = tree.node(grid.get().unwrap()).unwrap().descriptor.view();
    assert_eq!(g.role, Some(Role::Grid));
    assert_eq!(
        g.semantics.as_ref().unwrap().table,
        Some(TableMetadata::Table {
            rows: Some(21),
            columns: Some(8)
        })
    );
    drop(tree);
    assert_eq!(class(&app, "sapid-schedule-slot").len(), 140);
    assert_eq!(class(&app, "sapid-schedule-timeline-event").len(), 3);
    assert_eq!(class(&app, "sapid-agenda-event").len(), 1);
    let mut positions = class(&app, "sapid-schedule-timeline-event")
        .into_iter()
        .map(|id| {
            let tree = app.tree();
            let v = tree.node(id).unwrap().descriptor.view();
            (v.style.left, v.style.width, v.style.top)
        })
        .collect::<Vec<_>>();
    positions.sort_by_key(|p| match p.0 {
        Some(glory::style::Length::Px(v)) => v as i32,
        _ => 0,
    });
    assert_eq!(positions[0].0, Some(glory::style::Length::Px(424.)));
    assert_eq!(positions[1].0, Some(glory::style::Length::Px(464.)));
    assert_eq!(positions[2].0, Some(glory::style::Length::Px(504.)));
    assert!(
        positions
            .iter()
            .all(|p| p.1 == Some(glory::style::Length::Px(38.)))
    );
    assert_eq!(positions[0].2, Some(glory::style::Length::Px(96.)));
}
#[test]
fn same_tick_grid_navigation_and_activation_use_current_cursor_then_preserve_axes_at_edges() {
    let app = App::memory();
    let grid = app.scope().cage(None);
    let selected = app.scope().cage(None);
    let navigated = app.scope().cage(0usize);
    app.mount(
        week_view("cursor", data(7))
            .grid_ref(grid)
            .selection(selected)
            .on_selection_change(move |v| selected.revise(|old| *old = Some(v)))
            .on(EventKind::GridNavigate, move |_| {
                navigated.revise(|n| *n += 1)
            }),
    )
    .unwrap();
    let from = slot(&app, "Monday, February 26, 2024 08:00:00");
    let grid = grid.get().unwrap();
    app.scope()
        .batch(|| {
            for _ in 0..2 {
                app.dispatch(
                    grid,
                    Event::GridNavigate {
                        from,
                        command: GridCommand::Right,
                    },
                )
                .unwrap();
            }
            app.dispatch(from, Event::Activate).unwrap();
        })
        .unwrap();
    assert_eq!(navigated.get(), 2);
    assert!(
        matches!(selected.get(),Some(ScheduleSelection::Slot{interval,..})if interval.start()==dt("2024-02-28T08:00"))
    );
    let from = slot(&app, "Wednesday, February 28, 2024 08:00:00");
    app.dispatch(
        grid,
        Event::GridNavigate {
            from,
            command: GridCommand::Up,
        },
    )
    .unwrap();
    app.dispatch(from, Event::Activate).unwrap();
    app.flush().unwrap();
    assert!(
        matches!(selected.get(),Some(ScheduleSelection::Slot{interval,..})if interval.start()==dt("2024-02-28T08:00"))
    );
    app.scope()
        .batch(|| {
            app.dispatch(
                grid,
                Event::GridNavigate {
                    from,
                    command: GridCommand::NextPage,
                },
            )
            .unwrap();
            app.dispatch(from, Event::Activate).unwrap();
        })
        .unwrap();
    assert!(
        matches!(selected.get(),Some(ScheduleSelection::Slot{interval,..})if interval.start()==dt("2024-02-28T10:00")),
        "selection={:?}; navigation={}",
        selected.get(),
        navigated.get()
    );
    let tree = app.tree();
    assert_eq!(
        tree.node(slot(&app, "Wednesday, February 28, 2024 10:00:00"))
            .unwrap()
            .descriptor
            .view()
            .focus,
        FocusMode::TabStop
    );
}
#[test]
fn readonly_grid_still_roves_but_never_selects_and_disabled_rejects_descendant_origin() {
    let app = App::memory();
    let grid = app.scope().cage(None);
    let enabled = app.scope().cage(true);
    let selected = app.scope().cage(None);
    app.mount(
        week_view("readonly", data(7))
            .grid_ref(grid)
            .readonly(true)
            .enabled(enabled)
            .on_selection_change(move |v| selected.revise(|old| *old = Some(v))),
    )
    .unwrap();
    let grid = grid.get().unwrap();
    assert!(
        app.tree()
            .node(grid)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .readonly
    );
    let from = slot(&app, "Monday, February 26, 2024 08:00:00");
    app.dispatch(
        grid,
        Event::GridNavigate {
            from,
            command: GridCommand::Down,
        },
    )
    .unwrap();
    app.dispatch(from, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(selected.get(), None);
    let event = class(&app, "sapid-schedule-timeline-event")[0];
    assert!(!app.dispatch(event, Event::Activate).unwrap());
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(!app.tree().enabled(from));
    app.dispatch(
        grid,
        Event::GridNavigate {
            from,
            command: GridCommand::Down,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(selected.get(), None);
}
#[test]
fn day_and_week_reject_wrong_windows_and_overfull_overlap_before_mounting_children() {
    let app = App::memory();
    assert!(app.mount(day_view("wrong", data(7))).is_err());
    assert!(app.tree().roots().is_empty());
    let events = (0..17).map(|at| {
        ScheduleEvent::new(
            ScheduleKey::new(format!("overlap-{at}")).unwrap(),
            "Busy",
            ScheduleInterval::new(dt("2024-02-29T09:00"), dt("2024-02-29T10:00")).unwrap(),
        )
        .unwrap()
    });
    let busy = ScheduleData::new(
        ScheduleZone::Civil,
        ScheduleWindow::days(d("2024-02-29"), 1).unwrap(),
        events,
    )
    .unwrap();
    let app = App::memory();
    assert!(app.mount(day_view("busy", busy)).is_err());
    assert!(app.tree().roots().is_empty());
    let app = App::memory();
    let source = app.scope().cage(data(7));
    let grid = app.scope().cage(None);
    app.mount(week_view("recover", source).grid_ref(grid))
        .unwrap();
    let old = grid.get();
    source.revise(|v| *v = data(2));
    assert!(app.flush().is_err());
    assert_eq!(grid.get(), old);
    source.revise(|v| *v = data(7));
    app.flush().unwrap();
    assert_eq!(grid.get(), old);
}
#[test]
fn window_replacement_keeps_slot_shells_and_old_event_identity_cannot_select_new_data() {
    let app = App::memory();
    let source = app.scope().cage(data(7));
    let selected = app.scope().cage(None);
    app.mount(
        week_view("replace", source)
            .on_selection_change(move |v| selected.revise(|old| *old = Some(v))),
    )
    .unwrap();
    let from = slot(&app, "Monday, February 26, 2024 08:00:00");
    let old_event = class(&app, "sapid-schedule-timeline-event")[0];
    source.revise(|v| {
        *v = ScheduleData::new(ScheduleZone::Civil, v.window().shifted(1).unwrap(), []).unwrap()
    });
    app.flush().unwrap();
    assert_eq!(slot(&app, "Monday, March 4, 2024 08:00:00"), from);
    assert!(class(&app, "sapid-schedule-timeline-event").is_empty());
    assert!(!app.dispatch(old_event, Event::Activate).unwrap());
    app.dispatch(from, Event::Activate).unwrap();
    app.flush().unwrap();
    assert!(
        matches!(selected.get(),Some(ScheduleSelection::Slot{interval,..})if interval.start()==dt("2024-03-04T08:00"))
    );
}
#[test]
fn full_day_slot_budget_rtl_geometry_and_owner_refs_are_finite_and_retire() {
    let app = App::memory();
    let visible = app.scope().cage(true);
    let grid = app.scope().cage(None);
    let viewport = app.scope().cage(None);
    app.mount(glory::ui::when(visible, move || {
        week_view("whole-day", data(7))
            .time_window(ScheduleTimeWindow::new(0, 1440, 15).unwrap())
            .direction(Direction::Rtl)
            .grid_ref(grid)
            .viewport_ref(viewport)
    }))
    .unwrap();
    assert_eq!(class(&app, "sapid-schedule-slot").len(), 672);
    assert!(grid.get().is_some() && viewport.get().is_some());
    let first = slot(&app, "Monday, February 26, 2024 00:00:00");
    let tree = app.tree();
    let parent = tree.node(first).unwrap().parent.unwrap();
    assert_eq!(
        tree.node(parent).unwrap().descriptor.view().style.left,
        Some(glory::style::Length::Px(784.))
    );
    drop(tree);
    visible.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(grid.get().is_none() && viewport.get().is_none());
    assert_eq!(app.stats().queries, 0);
    assert!(!app.dispatch(first, Event::Activate).unwrap());
}
#[test]
fn day_geometry_options_are_bounded_and_short_labels_keep_full_accessible_interval() {
    use glory::style::Length;
    for width in [f64::NAN, f64::INFINITY, 95., 1025.] {
        let app = App::memory();
        assert!(
            app.mount(day_view("bad-width", data(1)).column_width(width))
                .is_err()
        );
        assert!(app.tree().roots().is_empty());
    }
    for height in [f64::NAN, 79., 2049.] {
        let app = App::memory();
        assert!(
            app.mount(day_view("bad-height", data(1)).viewport_height(height))
                .is_err()
        );
        assert!(app.tree().roots().is_empty());
    }
    let app = App::memory();
    let viewport = app.scope().cage(None);
    app.mount(
        day_view("day-width", data(1))
            .column_width(300.)
            .viewport_height(200.)
            .viewport_ref(viewport),
    )
    .unwrap();
    let first = slot(&app, "Monday, February 26, 2024 08:00:00");
    let tree = app.tree();
    let v = tree.node(first).unwrap().descriptor.view();
    assert!(matches!(v.props,Props::Button(p)if p.label=="08:00"));
    assert_eq!(
        v.name.as_deref(),
        Some("Monday, February 26, 2024 08:00:00 — 08:30:00")
    );
    assert_eq!(
        tree.node(viewport.get().unwrap())
            .unwrap()
            .descriptor
            .view()
            .style
            .height,
        Some(Length::Px(200.))
    );
    let cell = tree.node(first).unwrap().parent.unwrap();
    assert_eq!(
        tree.node(cell).unwrap().descriptor.view().style.width,
        Some(Length::Px(300.))
    );
}
