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
fn period(app: &App, name: &str) -> NodeId {
    all(app)
        .into_iter()
        .find(|id| {
            let tree = app.tree();
            let v = tree.node(*id).unwrap().descriptor.view();
            matches!(v.props, Props::Button(_)) && v.name.as_deref() == Some(name)
        })
        .unwrap()
}

#[test]
fn native_month_dialog_four_themes_uses_real_grid_hit_pixels_keyboard_and_focus_return() {
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
        let value = app.scope().cage(MonthSelection::Single(Some(
            YearMonth::new(2024, 2).unwrap(),
        )));
        host.mount(
            month_picker_input("native-date", DatesProvider::new(d("2024-02-29")))
                .theme(theme.clone())
                .name("Appointment")
                .value(value)
                .node_ref(input)
                .trigger_ref(trigger)
                .panel_ref(panel)
                .grid_ref(grid)
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
        assert!(rect.width >= 300. && rect.height >= 180.);
        assert!(rect.x >= 0. && rect.y >= 0.);
        let selected = period(app, "February 2024");
        let next = period(app, "March 2024");
        let cell = bounds(app, selected);
        assert!(cell.width >= 75. && cell.height >= 30.);
        assert!(cell.x >= rect.x && cell.y >= rect.y && cell.x + cell.width <= rect.x + rect.width);
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
        assert_eq!(
            value.get(),
            MonthSelection::Single(Some(YearMonth::new(2024, 3).unwrap()))
        );
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
fn native_year_dialog_four_themes_uses_real_grid_hit_pixels_keyboard_and_focus_return() {
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
            .cage(YearSelection::Single(Some(CivilYear::new(2024).unwrap())));
        host.mount(
            year_picker_input("native-date", DatesProvider::new(d("2024-02-29")))
                .theme(theme.clone())
                .name("Appointment")
                .value(value)
                .node_ref(input)
                .trigger_ref(trigger)
                .panel_ref(panel)
                .grid_ref(grid)
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
        assert!(rect.width >= 300. && rect.height >= 120.);
        assert!(rect.x >= 0. && rect.y >= 0.);
        let selected = period(app, "2024");
        let next = period(app, "2025");
        let cell = bounds(app, selected);
        assert!(cell.width >= 45. && cell.height >= 30.);
        assert!(cell.x >= rect.x && cell.y >= rect.y && cell.x + cell.width <= rect.x + rect.width);
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
        assert_eq!(
            value.get(),
            YearSelection::Single(Some(CivilYear::new(2025).unwrap()))
        );
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
