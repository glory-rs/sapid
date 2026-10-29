use glory::apace::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, Direction, DragPhase, NodeId, Props, Query, QueryValue, Rect};
use sapid::prelude::*;
use std::{cell::RefCell, rc::Rc};
fn d(s: &str) -> CivilDate {
    CivilDate::parse_iso(s).unwrap()
}
fn dt(s: &str) -> LocalDateTime {
    parse_local_date_time(s).unwrap()
}
fn key(s: &str) -> ScheduleKey {
    ScheduleKey::new(s).unwrap()
}
fn data(month: bool) -> ResourceScheduleData {
    ResourceScheduleData::new(
        ScheduleData::new(
            ScheduleZone::Civil,
            if month {
                ScheduleWindow::month(YearMonth::new(2024, 2).unwrap()).unwrap()
            } else {
                ScheduleWindow::days(d("2024-02-29"), 1).unwrap()
            },
            [
                ScheduleEvent::new(
                    key("planning"),
                    "Planning",
                    ScheduleInterval::new(dt("2024-02-29T09:00"), dt("2024-02-29T10:00"))
                        .unwrap(),
                )
                .unwrap()
                .resource(key("room0"))
                .permissions(SchedulePermissions {
                    select: true,
                    move_event: true,
                    resize_event: true,
                }),
                ScheduleEvent::new(
                    key("review"),
                    "Review",
                    ScheduleInterval::new(dt("2024-02-29T09:00"), dt("2024-02-29T10:00"))
                        .unwrap(),
                )
                .unwrap()
                .resource(key("room1")),
            ],
        )
        .unwrap(),
        ScheduleResources::new((0..2).map(|i| {
            ScheduleResource::new(key(&format!("room{i}")), format!("Room {i}")).unwrap()
        }))
        .unwrap(),
        ScheduleResourceWindow::new(0, 2).unwrap(),
    )
    .unwrap()
}
fn all(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut ids = tree.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = ids.pop() {
        ids.extend(tree.node(id).unwrap().children.iter().copied());
        out.push(id)
    }
    out
}
fn classes(app: &App, class: &str) -> Vec<NodeId> {
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
                .any(|s| s == class)
        })
        .collect()
}
fn button(app: &App, label: &str) -> NodeId {
    all(app).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(p) if p.label.starts_with(label))).unwrap()
}
fn query(app: &App, q: Query) -> QueryValue {
    let result = Rc::new(RefCell::new(None));
    let output = result.clone();
    app.request(q, move |v| *output.borrow_mut() = Some(v.unwrap()))
        .unwrap();
    app.pump().unwrap();
    result.borrow_mut().take().unwrap()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let QueryValue::Bounds(rect) = query(app, Query::Bounds(id)) else {
        panic!()
    };
    rect
}
#[test]
fn native_resource_axes_four_themes_actual_hit_pixels_geometry_focus_and_queued_drop() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        for axis in [ResourceAxis::Columns, ResourceAxis::Rows] {
            let host = WindowApp::new();
            host.renderer().set_viewport(900, 1050, 1.).unwrap();
            let app = host.app();
            let viewport = app.scope().cage(None);
            let edited = app.scope().cage(None);
            let selected = app.scope().cage(None);
            host.mount(
                resources_day_view("native-resources", data(false))
                    .theme(theme.clone())
                    .axis(axis)
                    .viewport_height(400.)
                    .viewport_ref(viewport)
                    .on_edit(move |e| edited.revise(|v| *v = Some(e)))
                    .on_selection_change(move |v| selected.revise(|old| *old = Some(v))),
            )
            .unwrap();
            app.settle(32).unwrap();
            let planning = button(app, "Planning");
            let review = button(app, "Review");
            let a = bounds(app, planning);
            let b = bounds(app, review);
            assert!(a.width > 100. && a.height > 15., "axis={axis:?} a={a:?}");
            if axis == ResourceAxis::Columns {
                assert!((b.x - a.x - 160.).abs() < 1. && a.y == b.y)
            } else {
                assert!((b.y - a.y - 160.).abs() < 1. && a.x == b.x)
            }
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
            assert!(
                matches!(selected.get(),Some(ScheduleSelection::Event{key:k,..})if k==key("planning"))
            );
            let card = app.tree().node(planning).unwrap().parent.unwrap();
            let handle = classes(app, "sapid-resources-drag-move")
                .into_iter()
                .find(|id| {
                    let handles = app.tree().node(*id).unwrap().parent.unwrap();
                    app.tree().node(handles).unwrap().parent == Some(card)
                })
                .unwrap();
            let r = bounds(app, handle);
            assert!(r.width > 20. && r.height >= 10., "handle={r:?}");
            let mut raster = Raster::new();
            raster.draw(host.renderer()).unwrap();
            let frame = raster.frame().unwrap();
            let at = ((r.y as usize + 2) * frame.width as usize + r.x as usize + 2) * 4;
            for (value, want) in raster.pixels()[at..at + 4].iter().zip([
                theme.accent.0,
                theme.accent.1,
                theme.accent.2,
                theme.accent.3,
            ]) {
                assert!(value.abs_diff(want) <= 3)
            }
            let identity = host
                .renderer()
                .engine_id(planning, glory::html::Part::Main)
                .unwrap();
            let start = (r.x + r.width / 2., r.y + r.height / 2.);
            app.settle(32).unwrap();
            let end = if axis == ResourceAxis::Columns {
                (start.0 + 160., start.1 + 32.)
            } else {
                (start.0 + 64., start.1 + 160.)
            };
            for (phase, point) in [
                (DragPhase::Start, start),
                (DragPhase::Move, end),
                (DragPhase::Drop, end),
            ] {
                assert!(
                    app.dispatch(
                        handle,
                        Event::Drag {
                            from: handle,
                            phase,
                            x: point.0,
                            y: point.1,
                        },
                    )
                    .unwrap()
                );
            }
            app.settle(32).unwrap();
            let edit = edited.get().unwrap();
            assert_eq!(edit.proposed.start(), dt("2024-02-29T09:30"));
            assert_eq!(edit.proposed_resource, Some(key("room1")));
            assert_eq!(app.stats().queries, 0);
            assert_eq!(
                host.renderer()
                    .engine_id(planning, glory::html::Part::Main)
                    .unwrap(),
                identity
            );
            app.scope().action(Action::Focus(planning)).unwrap();
            app.flush().unwrap();
            assert_eq!(query(app, Query::Focus(planning)), QueryValue::Focus(true));
            let QueryValue::ScrollMetrics(metrics) =
                query(app, Query::ScrollMetrics(viewport.get().unwrap()))
            else {
                panic!()
            };
            assert!(metrics.content_height > 0. && metrics.content_width > 0.);
            app.clear().unwrap();
            app.pump().unwrap();
            assert_eq!(app.stats().queries, 0);
            assert_eq!(app.stats().nodes, 0);
        }
    }
}
#[test]
fn native_resource_month_real_resource_day_grid_rtl_readonly_and_owner_release() {
    let host = WindowApp::new();
    host.renderer().set_viewport(900, 1600, 1.).unwrap();
    let app = host.app();
    let selected = app.scope().cage(0usize);
    host.mount(
        resources_month_view("native-month", data(true))
            .direction(Direction::Rtl)
            .readonly(true)
            .on_selection_change(move |_| selected.revise(|v| *v += 1)),
    )
    .unwrap();
    app.settle(32).unwrap();
    assert_eq!(classes(app, "sapid-resources-slot").len(), 58);
    let grids = all(app)
        .into_iter()
        .filter(|id| app.tree().node(*id).unwrap().descriptor.view().role == Some(Role::Grid))
        .count();
    assert_eq!(grids, 1);
    for id in classes(app, "sapid-resources-viewport") {
        let r = bounds(app, id);
        assert!(r.width > 300. && r.height > 200., "{r:?}");
    }
    let planning = button(app, "Planning");
    assert!(!app.dispatch(planning, Event::Activate).unwrap());
    app.flush().unwrap();
    assert_eq!(selected.get(), 0);
    app.clear().unwrap();
    app.pump().unwrap();
    assert_eq!(app.stats().nodes, 0);
}
#[test]
fn native_resource_month_drag_moves_one_civil_day_and_changes_resource_without_writing_source()
{
    for axis in [ResourceAxis::Columns, ResourceAxis::Rows] {
        let host = WindowApp::new();
        host.renderer().set_viewport(900, 900, 1.).unwrap();
        let app = host.app();
        let edits = app.scope().cage(None);
        host.mount(
            resources_month_view("month-edit", data(true))
                .axis(axis)
                .viewport_height(420.)
                .on_edit(move |v| edits.revise(|old| *old = Some(v))),
        )
        .unwrap();
        app.settle(32).unwrap();
        let planning = button(app, "Planning");
        app.scope()
            .action(Action::ScrollIntoView(planning))
            .unwrap();
        app.settle(32).unwrap();
        let card = app.tree().node(planning).unwrap().parent.unwrap();
        let handle = classes(app, "sapid-resources-drag-move")
            .into_iter()
            .find(|id| {
                let parent = app.tree().node(*id).unwrap().parent.unwrap();
                app.tree().node(parent).unwrap().parent == Some(card)
            })
            .unwrap();
        let rect = bounds(app, handle);
        let start = (rect.x + rect.width / 2., rect.y + rect.height / 2.);
        let end = if axis == ResourceAxis::Columns {
            (start.0 + 160., start.1 - 32.)
        } else {
            (start.0 - 64., start.1 + 160.)
        };
        for (phase, point) in [(DragPhase::Start, start), (DragPhase::Drop, end)] {
            assert!(
                app.dispatch(
                    handle,
                    Event::Drag {
                        from: handle,
                        phase,
                        x: point.0,
                        y: point.1
                    }
                )
                .unwrap()
            );
        }
        app.settle(32).unwrap();
        let value = edits.get().unwrap();
        assert_eq!(value.original.start(), dt("2024-02-29T09:00"));
        assert_eq!(value.proposed.start(), dt("2024-02-28T09:00"));
        assert_eq!(value.proposed_resource, Some(key("room1")));
        assert!(app.tree().node(planning).is_ok());
        app.clear().unwrap();
        assert_eq!(app.stats().queries, 0);
    }
}
