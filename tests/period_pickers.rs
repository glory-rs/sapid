#![cfg(feature = "dates")]
use glory::prelude::*;
use glory::render::{NodeId, Props};
use sapid::prelude::*;
use std::{cell::Cell, rc::Rc};
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

fn year(v: u16) -> CivilYear {
    CivilYear::new(v).unwrap()
}
fn month(v: u8) -> YearMonth {
    YearMonth::new(2024, v).unwrap()
}
fn button(app: &App, name: &str) -> NodeId {
    all(app)
        .into_iter()
        .find(|id| {
            let t = app.tree();
            let v = t.node(*id).unwrap().descriptor.view();
            matches!(v.props, Props::Button(_)) && v.name.as_deref() == Some(name)
        })
        .unwrap_or_else(|| panic!("missing {name}"))
}
fn selected(app: &App) -> usize {
    class(app, "sapid-period-cell")
        .into_iter()
        .filter(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .selected
                == Some(true)
        })
        .count()
}
#[test]
fn month_grid_is_twelve_real_buttons_in_four_rows_with_typed_month_selection() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    app.mount(
        month_picker("months", p())
            .default_value(MonthSelection::Single(Some(month(2))))
            .node_ref(reference),
    )
    .unwrap();
    assert_eq!(class(&app, "sapid-period-button").len(), 12);
    assert_eq!(class(&app, "sapid-period-cell").len(), 12);
    assert_eq!(selected(&app), 1);
    let t = app.tree();
    let root = t.node(reference.get().unwrap()).unwrap().descriptor.view();
    assert_eq!(root.role, Some(Role::Grid));
    assert_eq!(
        root.semantics.as_ref().unwrap().table,
        Some(glory::render::TableMetadata::Table {
            rows: Some(4),
            columns: Some(3)
        })
    );
    drop(t);
    let feb = button(&app, "February 2024");
    assert_eq!(
        app.tree().node(feb).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    assert_eq!(
        app.tree()
            .node(feb)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .current,
        Some(Current::Date)
    );
}
#[test]
fn year_grid_uses_a_real_ten_year_view_and_never_invents_year_zero_or_ten_thousand() {
    for (date, expected, buttons) in [
        (CivilDate::MIN, "0001–0009", 9),
        (CivilDate::MAX, "9990–9999", 10),
    ] {
        let app = App::memory();
        let reference = app.scope().cage(None);
        app.mount(year_picker("years", DatesProvider::new(date)).node_ref(reference))
            .unwrap();
        assert_eq!(class(&app, "sapid-period-cell").len(), 10);
        assert_eq!(class(&app, "sapid-period-button").len(), buttons);
        assert_eq!(
            app.tree()
                .node(reference.get().unwrap())
                .unwrap()
                .descriptor
                .view()
                .name
                .as_deref(),
            Some(expected)
        );
        assert!(all(&app).iter().all(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .name
                .as_deref()
                != Some("0000")
        }));
    }
}
#[test]
fn same_tick_month_cross_year_navigation_then_activation_uses_the_current_cursor() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    let value = app.scope().cage(MonthSelection::Single(Some(month(12))));
    let page = app.scope().cage(year(2024));
    app.mount(
        month_picker("cursor", p())
            .value(value)
            .year(page)
            .node_ref(reference)
            .on_year_change(move |v| page.revise(|old| *old = v))
            .on_value_change(move |v| value.revise(|old| *old = v)),
    )
    .unwrap();
    let from = button(&app, "December 2024");
    let grid = reference.get().unwrap();
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
    assert_eq!(page.get(), year(2025));
    assert_eq!(value.get().format_iso(), "2025-02");
    assert_eq!(
        app.tree()
            .node(button(&app, "February 2025"))
            .unwrap()
            .descriptor
            .view()
            .focus,
        FocusMode::TabStop
    );
}
#[test]
fn year_page_large_page_and_row_navigation_are_finite_and_readonly_does_not_write() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    let page = app.scope().cage(year(2024));
    let value = app.scope().cage(YearSelection::Single(Some(year(2024))));
    let readonly = app.scope().cage(false);
    app.mount(
        year_picker("nav", p())
            .value(value)
            .year(page)
            .readonly(readonly)
            .node_ref(reference)
            .on_year_change(move |v| page.revise(|old| *old = v))
            .on_value_change(move |v| value.revise(|old| *old = v)),
    )
    .unwrap();
    let grid = reference.get().unwrap();
    let from = button(&app, "2024");
    app.dispatch(
        grid,
        Event::GridNavigate {
            from,
            command: GridCommand::NextLargePage,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(page.get(), year(2124));
    let from = button(&app, "2124");
    app.dispatch(
        grid,
        Event::GridNavigate {
            from,
            command: GridCommand::LastInRow,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(
        app.tree().node(from).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    readonly.revise(|v| *v = true);
    app.flush().unwrap();
    app.dispatch(from, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(value.get().format_iso(), "2024");
}
#[test]
fn controlled_year_refusal_and_invalid_external_page_keep_accepted_cells_then_recover_silently() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    let page = app.scope().cage(year(2024));
    let calls = app.scope().cage(0usize);
    let provider =
        p().bounds(DateBounds::new(Some(d("2024-03-15")), Some(d("2025-02-04"))).unwrap());
    app.mount(
        month_picker("controlled", provider)
            .year(page)
            .node_ref(reference)
            .on_year_change(move |_| calls.revise(|n| *n += 1)),
    )
    .unwrap();
    let root = reference.get().unwrap();
    let from = button(&app, "December 2024");
    app.dispatch(
        root,
        Event::GridNavigate {
            from,
            command: GridCommand::Right,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(calls.get(), 1);
    assert!(class(&app, "sapid-period-button").contains(&from));
    page.revise(|v| *v = year(2030));
    assert!(app.flush().is_err());
    assert!(class(&app, "sapid-period-button").contains(&from));
    page.revise(|v| *v = year(2025));
    app.flush().unwrap();
    assert!(
        app.tree()
            .node(button(&app, "February 2025"))
            .unwrap()
            .descriptor
            .view()
            .props
            .enabled()
    );
    assert_eq!(calls.get(), 1);
}
#[test]
fn range_and_multiple_use_real_selected_cells_with_disabled_interior_and_nonfault_capacity() {
    let app = App::memory();
    let value = app
        .scope()
        .cage(MonthSelection::empty(DatePickerMode::Range));
    app.mount(
        month_picker("range", p())
            .mode(DatePickerMode::Range)
            .exclude_value(month(2))
            .value(value)
            .on_value_change(move |v| value.revise(|old| *old = v)),
    )
    .unwrap();
    app.scope()
        .batch(|| {
            app.dispatch(button(&app, "January 2024"), Event::Activate)
                .unwrap();
            app.dispatch(button(&app, "March 2024"), Event::Activate)
                .unwrap();
        })
        .unwrap();
    assert_eq!(value.get().format_iso(), "2024-01/2024-03");
    assert_eq!(selected(&app), 3);
    let app = App::memory();
    let rejects = app.scope().cage(0usize);
    let full = SelectedYears::try_from_iter((2000..2032).map(year)).unwrap();
    app.mount(
        year_picker("full", p())
            .mode(DatePickerMode::Multiple)
            .default_value(YearSelection::Multiple(full))
            .default_year(year(2032))
            .on_reject(move |_| rejects.revise(|n| *n += 1)),
    )
    .unwrap();
    app.dispatch(button(&app, "2032"), Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(rejects.get(), 1);
}
#[test]
fn readonly_period_inputs_use_exact_month_and_year_successful_values_and_real_dialogs() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    let panel = app.scope().cage(None);
    app.mount(
        month_picker_input("month-input", p())
            .mode(DatePickerMode::Range)
            .default_open(true)
            .form_name("month")
            .node_ref(reference)
            .panel_ref(panel),
    )
    .unwrap();
    app.dispatch(button(&app, "March 2024"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert!(
        !app.tree()
            .node(panel.get().unwrap())
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
    app.dispatch(button(&app, "January 2024"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    let t = app.tree();
    let view = t.node(reference.get().unwrap()).unwrap().descriptor.view();
    let Props::Input(v) = view.props else {
        panic!()
    };
    assert_eq!(v.value, "2024-01/2024-03");
    assert!(view.readonly());
    drop(t);
    let app = App::memory();
    let reference = app.scope().cage(None);
    app.mount(
        year_picker_input("year-input", p())
            .default_value(YearSelection::Single(Some(year(2024))))
            .node_ref(reference),
    )
    .unwrap();
    let t = app.tree();
    let Props::Input(v) = t
        .node(reference.get().unwrap())
        .unwrap()
        .descriptor
        .view()
        .props
    else {
        panic!()
    };
    assert_eq!(v.value, "2024");
}
#[test]
fn finite_exclusion_admission_disabled_and_owner_retirement_do_not_leave_live_refs() {
    let app = App::memory();
    let seen = Rc::new(Cell::new(0));
    let out = seen.clone();
    let widget = year_picker("budget", p()).exclude_values((0..1_000_000).map(move |_| {
        out.set(out.get() + 1);
        year(2024)
    }));
    assert_eq!(seen.get(), MAX_EXCLUDED_PERIODS + 1);
    assert!(app.mount(widget).is_err());
    let app = App::memory();
    let visible = app.scope().cage(true);
    let enabled = app.scope().cage(true);
    let reference = app.scope().cage(None);
    app.mount(glory::ui::when(visible, move || {
        year_picker("owner", p())
            .enabled(enabled)
            .node_ref(reference)
    }))
    .unwrap();
    let from = button(&app, "2024");
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(!app.dispatch(from, Event::Activate).unwrap());
    visible.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(reference.get().is_none());
    assert!(!app.dispatch(from, Event::Activate).unwrap());
}
