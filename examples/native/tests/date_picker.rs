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
fn native_default_picker_dialog_four_themes_has_real_popup_hit_testing_focus_return_and_pixels()
{
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(820, 650, 1.).unwrap();
        let app = host.app();
        let input = app.scope().cage(None);
        let trigger = app.scope().cage(None);
        let panel = app.scope().cage(None);
        let grid = app.scope().cage(None);
        let value = app
            .scope()
            .cage(DateSelection::Single(Some(d("2024-02-28"))));
        host.mount(
            date_picker_input("native-date", DatesProvider::new(d("2024-02-29")))
                .theme(theme.clone())
                .name("Appointment")
                .value(value)
                .node_ref(input)
                .trigger_ref(trigger)
                .panel_ref(panel)
                .calendar_ref(grid)
                .on_value_change(move |v| value.revise(|old| *old = v)),
        )
        .unwrap();
        app.settle(16).unwrap();
        let input = input.get().unwrap();
        let trigger = trigger.get().unwrap();
        let panel = panel.get().unwrap();
        let grid = grid.get().unwrap();
        let physical = host
            .renderer()
            .engine_id(input, glory::html::Part::Main)
            .unwrap();
        app.scope()
            .action(glory::render::Action::Focus(trigger))
            .unwrap();
        app.settle(16).unwrap();
        app.dispatch(trigger, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert_eq!(host.renderer().inspect(|doc| doc.top_layers().len()), 1);
        let rect = bounds(app, panel);
        assert!(rect.width >= 300. && rect.height >= 250.);
        assert!(rect.x >= 0. && rect.y >= 0.);
        let selected = day(app, d("2024-02-28"));
        let next = day(app, d("2024-02-29"));
        let cell = bounds(app, selected);
        assert!((cell.width - 36.).abs() < 0.2 && (cell.height - 36.).abs() < 0.2);
        assert!(
            cell.x >= rect.x && cell.y >= rect.y && cell.x + cell.width <= rect.x + rect.width
        );
        let hit = host.renderer().inspect(|dom| {
            dom.hit((cell.x + 18.) as f32, (cell.y + 18.) as f32)
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
        app.scope()
            .action(glory::render::Action::Focus(selected))
            .unwrap();
        app.settle(16).unwrap();
        app.dispatch(
            grid,
            Event::GridNavigate {
                from: selected,
                command: GridCommand::Right,
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        assert_eq!(query(app, Query::Focus(next)), QueryValue::Focus(true));
        app.dispatch(next, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert_eq!(value.get(), DateSelection::Single(Some(d("2024-02-29"))));
        assert_eq!(host.renderer().inspect(|doc| doc.top_layers().len()), 0);
        assert_eq!(query(app, Query::Focus(trigger)), QueryValue::Focus(true));
        assert_eq!(
            host.renderer()
                .engine_id(input, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        app.dispatch(trigger, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert_eq!(host.renderer().inspect(|doc| doc.top_layers().len()), 1);
        app.dispatch(panel, Event::Dismiss(glory::render::DismissReason::Escape))
            .unwrap();
        app.settle(16).unwrap();
        assert_eq!(query(app, Query::Focus(trigger)), QueryValue::Focus(true));
        app.clear().unwrap();
        app.pump().unwrap();
        assert_eq!(app.stats().queries, 0);
        assert_eq!(host.renderer().inspect(|doc| doc.top_layers().len()), 0);
    }
}
#[test]
fn native_range_projection_readonly_and_disabled_keep_finite_real_selection_and_accepted_state()
{
    let host = WindowApp::new();
    host.renderer().set_viewport(720, 540, 1.).unwrap();
    let app = host.app();
    let readonly = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    let value = app
        .scope()
        .cage(DateSelection::empty(DatePickerMode::Range));
    host.mount(
        date_picker("native-range", DatesProvider::new(d("2024-02-29")))
            .mode(DatePickerMode::Range)
            .value(value)
            .readonly(readonly)
            .enabled(enabled)
            .exclude_date(d("2024-02-28"))
            .on_value_change(move |v| value.revise(|old| *old = v)),
    )
    .unwrap();
    app.settle(16).unwrap();
    app.dispatch(day(app, d("2024-02-26")), Event::Activate)
        .unwrap();
    app.dispatch(day(app, d("2024-02-29")), Event::Activate)
        .unwrap();
    app.settle(16).unwrap();
    assert_eq!(value.get().format_iso(), "2024-02-26/2024-02-29");
    let cells = all(app)
        .into_iter()
        .filter(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .is_some_and(|s| s.selected == Some(true))
        })
        .count();
    assert_eq!(cells, 4);
    readonly.revise(|v| *v = true);
    app.settle(16).unwrap();
    let old = value.get();
    app.dispatch(day(app, d("2024-02-27")), Event::Activate)
        .unwrap();
    app.settle(16).unwrap();
    assert_eq!(value.get(), old);
    enabled.revise(|v| *v = false);
    app.settle(16).unwrap();
    assert!(
        app.scope()
            .action(glory::render::Action::Focus(day(app, d("2024-02-27"))))
            .is_err()
    );
    app.clear().unwrap();
}

#[test]
fn native_initially_open_range_input_retains_partial_then_closes_only_the_complete_range() {
    let host = WindowApp::new();
    host.renderer().set_viewport(760, 600, 1.).unwrap();
    let app = host.app();
    let input = app.scope().cage(None);
    let trigger = app.scope().cage(None);
    host.mount(
        date_picker_input("initial-range", DatesProvider::new(d("2024-02-29")))
            .mode(DatePickerMode::Range)
            .default_open(true)
            .node_ref(input)
            .trigger_ref(trigger),
    )
    .unwrap();
    app.settle(16).unwrap();
    assert_eq!(host.renderer().inspect(|doc| doc.top_layers().len()), 1);
    app.dispatch(day(app, d("2024-02-26")), Event::Activate)
        .unwrap();
    app.settle(16).unwrap();
    assert_eq!(host.renderer().inspect(|doc| doc.top_layers().len()), 1);
    let t = app.tree();
    let Props::Input(p) = t
        .node(input.get().unwrap())
        .unwrap()
        .descriptor
        .view()
        .props
    else {
        panic!()
    };
    assert_eq!(p.value, "2024-02-26/");
    drop(t);
    app.dispatch(day(app, d("2024-02-29")), Event::Activate)
        .unwrap();
    app.settle(16).unwrap();
    assert_eq!(host.renderer().inspect(|doc| doc.top_layers().len()), 0);
    assert_eq!(
        query(app, Query::Focus(trigger.get().unwrap())),
        QueryValue::Focus(true)
    );
    let t = app.tree();
    let Props::Input(p) = t
        .node(input.get().unwrap())
        .unwrap()
        .descriptor
        .view()
        .props
    else {
        panic!()
    };
    assert_eq!(p.value, "2024-02-26/2024-02-29");
    drop(t);
    app.clear().unwrap();
    assert_eq!(app.stats().queries, 0);
}
