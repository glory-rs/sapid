use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, GridCommand, NodeId, Props, Query, QueryValue, Rect};
use sapid::prelude::*;
use std::{cell::RefCell, rc::Rc};
#[test]
fn native_grid_generic_disabling_current_owner_revokes_pending_focus_intent() {
    for view in [ScheduleView::Month, ScheduleView::Week] {
        let host = WindowApp::new();
        host.renderer().set_viewport(900, 900, 1.).unwrap();
        let app = host.app();
        let window = if view == ScheduleView::Month {
            ScheduleWindow::month(d("2024-02-01").year_month()).unwrap()
        } else {
            ScheduleWindow::days(d("2024-02-26"), 7).unwrap()
        };
        let enabled = app.scope().cage(true);
        let grid = app.scope().cage(None);
        host.mount(
            schedule("disable-during-grid", data(window))
                .view(view)
                .enabled(enabled)
                .grid_ref(grid)
                .on(glory::render::EventKind::GridNavigate, move |_| {
                    enabled.revise(|v| *v = false)
                }),
        )
        .unwrap();
        app.settle(16).unwrap();
        let first = button(
            app,
            if view == ScheduleView::Month {
                "Thursday, February 1, 2024"
            } else {
                "Monday, February 26, 2024 08:00:00"
            },
        );
        app.dispatch(
            grid.get().unwrap(),
            Event::GridNavigate {
                from: first,
                command: GridCommand::Right,
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        assert!(!enabled.get());
        assert!(!app.tree().enabled(first));
        assert_eq!(app.stats().queries, 0);
    }
}
fn d(s: &str) -> CivilDate {
    CivilDate::parse_iso(s).unwrap()
}
fn dt(s: &str) -> LocalDateTime {
    parse_local_date_time(s).unwrap()
}
fn data(window: ScheduleWindow) -> ScheduleData {
    ScheduleData::new(
        ScheduleZone::Civil,
        window,
        (0..5).map(|at| {
            ScheduleEvent::new(
                ScheduleKey::new(format!("event-{at}")).unwrap(),
                format!("Event {at}"),
                ScheduleInterval::new(dt("2024-02-28T09:00"), dt("2024-02-29T10:00")).unwrap(),
            )
            .unwrap()
        }),
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
fn native_month_year_mobile_four_themes_real_grid_hit_focus_selection_pixels_and_disclosure() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        for mode in [
            ScheduleView::Month,
            ScheduleView::Year,
            ScheduleView::MobileMonth,
        ] {
            let host = WindowApp::new();
            host.renderer().set_viewport(900, 1000, 1.).unwrap();
            let app = host.app();
            let window = if mode == ScheduleView::Year {
                ScheduleWindow::year(CivilYear::new(2024).unwrap()).unwrap()
            } else {
                ScheduleWindow::month(d("2024-02-01").year_month()).unwrap()
            };
            let selected = app.scope().cage(None);
            let grid = app.scope().cage(None);
            host.mount(
                schedule("native-calendar", data(window))
                    .view(mode)
                    .theme(theme.clone())
                    .grid_ref(grid)
                    .selection(selected)
                    .on_selection_change(move |v| selected.revise(|old| *old = Some(v))),
            )
            .unwrap();
            app.settle(16).unwrap();
            let name = if mode == ScheduleView::Year {
                "February 2024, 5 events"
            } else {
                "Thursday, February 29, 2024, 5 events"
            };
            let cell = button(app, name);
            let rect = bounds(app, cell);
            assert!(rect.width > 110. && rect.height >= 35.);
            let physical = host
                .renderer()
                .engine_id(cell, glory::html::Part::Main)
                .unwrap();
            let hit = host.renderer().inspect(|doc| {
                doc.hit((rect.x + 5.) as f32, (rect.y + 5.) as f32)
                    .unwrap()
                    .node_id
            });
            assert_eq!(hit, physical);
            app.scope().action(Action::Focus(cell)).unwrap();
            app.settle(16).unwrap();
            assert_eq!(query(app, Query::Focus(cell)), QueryValue::Focus(true));
            app.dispatch(cell, Event::Activate).unwrap();
            app.settle(16).unwrap();
            assert!(selected.with(|v|matches!(v,Some(ScheduleSelection::Slot{interval,..})if interval.start().date()==if mode==ScheduleView::Year{d("2024-02-01")}else{d("2024-02-29")})));
            let mut raster = Raster::new();
            raster.draw(host.renderer()).unwrap();
            let frame = raster.frame().unwrap();
            let at = ((rect.y as usize + 5) * frame.width as usize + rect.x as usize + 5) * 4;
            for (actual, want) in raster.pixels()[at..at + 4].iter().zip([
                theme.accent_soft.0,
                theme.accent_soft.1,
                theme.accent_soft.2,
                theme.accent_soft.3,
            ]) {
                assert!(actual.abs_diff(want) <= 2);
            }
            if mode != ScheduleView::MobileMonth {
                let more = button(
                    app,
                    if mode == ScheduleView::Year {
                        "Show events for 2024-02-01"
                    } else {
                        "Show events for 2024-02-29"
                    },
                );
                let engine = host
                    .renderer()
                    .engine_id(more, glory::html::Part::Main)
                    .unwrap();
                app.dispatch(more, Event::Activate).unwrap();
                app.settle(16).unwrap();
                let fourth = button(app, "Event 4");
                let fourth_rect = bounds(app, fourth);
                assert!(fourth_rect.width > 20. && fourth_rect.height >= 28.);
                app.dispatch(more, Event::Activate).unwrap();
                app.settle(16).unwrap();
                assert_eq!(
                    host.renderer()
                        .engine_id(more, glory::html::Part::Main)
                        .unwrap(),
                    engine
                );
            }
            app.clear().unwrap();
            app.pump().unwrap();
            assert_eq!(app.stats().queries, 0);
        }
    }
}
#[test]
fn native_mobile_grid_keyboard_updates_real_details_readonly_and_owner_retirement() {
    let host = WindowApp::new();
    host.renderer().set_viewport(360, 900, 1.).unwrap();
    let app = host.app();
    let selected = app.scope().cage(None);
    let visible = app.scope().cage(true);
    let grid = app.scope().cage(None);
    host.mount(glory::ui::when(visible, move || {
        mobile_month_view(
            "native-mobile",
            data(ScheduleWindow::month(d("2024-02-01").year_month()).unwrap()),
        )
        .readonly(true)
        .grid_ref(grid)
        .on_selection_change(move |v| selected.revise(|old| *old = Some(v)))
    }))
    .unwrap();
    app.settle(16).unwrap();
    let first = button(app, "Thursday, February 1, 2024");
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
    let leap = button(app, "Thursday, February 29, 2024");
    assert_eq!(query(app, Query::Focus(leap)), QueryValue::Focus(true));
    app.dispatch(leap, Event::Activate).unwrap();
    app.settle(16).unwrap();
    assert_eq!(selected.get(), None);
    let rect = bounds(app, leap);
    assert!(rect.width > 45. && rect.width < 55. && rect.height >= 48.);
    assert!(all(app).into_iter().any(|id|matches!(app.tree().node(id).unwrap().descriptor.view().props,Props::Button(p)if p.label.starts_with("Event 4"))));
    visible.revise(|v| *v = false);
    app.settle(16).unwrap();
    assert!(grid.get().is_none());
    assert_eq!(app.stats().queries, 0);
    assert!(!app.dispatch(leap, Event::Activate).unwrap());
}
