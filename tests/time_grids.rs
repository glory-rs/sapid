#![cfg(feature = "dates")]
use glory::prelude::*;
use glory::render::{NodeId, Props};
use sapid::prelude::*;
use std::{cell::Cell, rc::Rc};
fn t(s: &str) -> LocalTime {
    LocalTime::parse_iso(s).unwrap()
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
            matches!(view.props, Props::Button(_)) && view.name.as_deref() == Some(name)
        })
        .unwrap()
}
fn selected(app: &App) -> usize {
    all(app)
        .into_iter()
        .filter(|id| {
            let tree = app.tree();
            let v = tree.node(*id).unwrap().descriptor.view();
            v.role == Some(Role::GridCell)
                && v.semantics
                    .as_ref()
                    .is_some_and(|s| s.selected == Some(true))
        })
        .count()
}
#[test]
fn real_time_grid_rows_cells_buttons_selected_and_off_step_focus_are_distinct() {
    let app = App::memory();
    let r = app.scope().cage(None);
    app.mount(
        time_grid("slots")
            .node_ref(r)
            .default_value(Some(t("03:15"))),
    )
    .unwrap();
    let tree = app.tree();
    let root = tree.node(r.get().unwrap()).unwrap().descriptor.view();
    assert_eq!(root.role, Some(Role::Grid));
    assert_eq!(
        root.semantics.as_ref().unwrap().table,
        Some(glory::render::TableMetadata::Table {
            rows: Some(12),
            columns: Some(4)
        })
    );
    drop(tree);
    assert_eq!(
        all(&app)
            .into_iter()
            .filter(
                |id| app.tree().node(*id).unwrap().descriptor.view().role == Some(Role::GridCell)
            )
            .count(),
        48
    );
    assert_eq!(selected(&app), 0);
    assert_eq!(
        app.tree()
            .node(button(&app, "03:30"))
            .unwrap()
            .descriptor
            .view()
            .focus,
        FocusMode::TabStop
    );
}
#[test]
fn same_tick_commands_then_activation_follow_one_bounded_cursor() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let value = app.scope().cage(Some(t("00:00")));
    app.mount(
        time_grid("batch")
            .node_ref(r)
            .value(value)
            .on_value_change(move |v| value.revise(|old| *old = v)),
    )
    .unwrap();
    let from = button(&app, "00:00");
    app.scope()
        .batch(|| {
            for _ in 0..2 {
                app.dispatch(
                    r.get().unwrap(),
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
    assert_eq!(value.get(), Some(t("01:00")));
    assert_eq!(selected(&app), 1);
}
#[test]
fn disabled_slots_roving_rows_large_pages_and_readonly_obey_actual_grid_rules() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let calls = app.scope().cage(0usize);
    let ro = app.scope().cage(false);
    app.mount(
        time_grid("nav")
            .node_ref(r)
            .readonly(ro)
            .disable_time(t("00:30"))
            .on_value_change(move |_| calls.revise(|n| *n += 1)),
    )
    .unwrap();
    let from = button(&app, "00:00");
    app.dispatch(
        r.get().unwrap(),
        Event::GridNavigate {
            from,
            command: GridCommand::Right,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(button(&app, "01:00"))
            .unwrap()
            .descriptor
            .view()
            .focus,
        FocusMode::TabStop
    );
    assert!(
        !app.tree()
            .node(button(&app, "00:30"))
            .unwrap()
            .descriptor
            .view()
            .props
            .enabled()
    );
    app.dispatch(
        r.get().unwrap(),
        Event::GridNavigate {
            from: button(&app, "01:00"),
            command: GridCommand::LastInRow,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(button(&app, "01:30"))
            .unwrap()
            .descriptor
            .view()
            .focus,
        FocusMode::TabStop
    );
    app.dispatch(
        r.get().unwrap(),
        Event::GridNavigate {
            from: button(&app, "01:30"),
            command: GridCommand::NextLargePage,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(button(&app, "23:30"))
            .unwrap()
            .descriptor
            .view()
            .focus,
        FocusMode::TabStop
    );
    ro.revise(|v| *v = true);
    app.flush().unwrap();
    app.dispatch(button(&app, "23:30"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert_eq!(calls.get(), 0);
}
#[test]
fn controlled_refusal_invalid_source_and_recovery_preserve_last_accepted_selection() {
    let app = App::memory();
    let value = app.scope().cage(Some(t("09:00")));
    let calls = app.scope().cage(0usize);
    app.mount(
        time_grid("controlled")
            .bounds(TimeBounds::new(Some(t("09:00")), Some(t("17:00"))).unwrap())
            .value(value)
            .on_value_change(move |_| calls.revise(|n| *n += 1)),
    )
    .unwrap();
    app.dispatch(button(&app, "10:00"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert_eq!(value.get(), Some(t("09:00")));
    assert_eq!(calls.get(), 1);
    assert_eq!(selected(&app), 1);
    value.revise(|v| *v = Some(t("18:00")));
    assert!(app.flush().is_err());
    assert_eq!(selected(&app), 1);
    value.revise(|v| *v = Some(t("11:00")));
    app.flush().unwrap();
    assert_eq!(selected(&app), 1);
    assert_eq!(calls.get(), 1);
}
#[test]
fn disabled_iterator_admission_stops_at_sentinel_before_creating_controls() {
    let count = Rc::new(Cell::new(0));
    let observed = count.clone();
    let values = (0..1_000_000).map(move |_| {
        observed.set(observed.get() + 1);
        t("00:00")
    });
    let widget = time_grid("budget").disable_times(values);
    assert_eq!(count.get(), 129);
    let app = App::memory();
    assert!(app.mount(widget).is_err());
    assert_eq!(app.stats().nodes, 0);
    let bad = App::memory();
    assert!(
        bad.mount(
            time_grid("precision")
                .spec(TimeGridSpec::new(t("00:00:01"), t("00:00:30"), 1, 4).unwrap())
        )
        .is_err()
    );
}
#[test]
fn owner_retirement_clears_grid_ref_and_has_no_retained_query_registry() {
    let app = App::memory();
    let visible = app.scope().cage(true);
    let r = app.scope().cage(None);
    app.mount(glory::ui::when(visible, move || {
        time_grid("owner").node_ref(r)
    }))
    .unwrap();
    let old = r.get().unwrap();
    visible.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(r.get().is_none());
    assert_eq!(app.stats().queries, 0);
    assert!(!app.dispatch(old, Event::Activate).unwrap());
    visible.revise(|v| *v = true);
    app.flush().unwrap();
    assert_ne!(r.get(), Some(old));
    app.clear().unwrap();
    assert_eq!(app.stats().queries, 0);
}
