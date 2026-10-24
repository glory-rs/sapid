use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, NodeId, Props, Query, QueryValue, Rect};
use sapid::prelude::*;
use std::{cell::RefCell, rc::Rc};
fn d(s: &str) -> CivilDate {
    CivilDate::parse_iso(s).unwrap()
}
fn dt(s: &str) -> LocalDateTime {
    parse_local_date_time(s).unwrap()
}
fn data() -> ScheduleData {
    ScheduleData::new(
        ScheduleZone::Civil,
        ScheduleWindow::days(d("2024-02-26"), 7).unwrap(),
        [
            ScheduleEvent::new(
                ScheduleKey::new("planning").unwrap(),
                "Planning",
                ScheduleInterval::new(dt("2024-02-29T09:00"), dt("2024-02-29T10:00")).unwrap(),
            )
            .unwrap(),
            ScheduleEvent::new(
                ScheduleKey::new("review").unwrap(),
                "Review",
                ScheduleInterval::new(dt("2024-03-01T10:00"), dt("2024-03-01T11:00")).unwrap(),
            )
            .unwrap(),
        ],
    )
    .unwrap()
}
fn query(app: &App, q: Query) -> QueryValue {
    let out = Rc::new(RefCell::new(None));
    let r = out.clone();
    app.request(q, move |v| *r.borrow_mut() = Some(v.unwrap()))
        .unwrap();
    app.pump().unwrap();
    out.borrow_mut().take().unwrap()
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
    all(app).into_iter().find(|id|{let tree=app.tree();matches!(tree.node(*id).unwrap().descriptor.view().props,Props::Button(p)if p.label.starts_with(name))}).unwrap()
}
#[test]
fn native_agenda_four_themes_real_lists_hit_focus_selection_pixels_and_keyed_identity() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(900, 650, 1.).unwrap();
        let app = host.app();
        let source = app.scope().cage(data());
        let selected = app.scope().cage(None);
        host.mount(
            agenda_view("native-agenda", source)
                .theme(theme.clone())
                .selection(selected)
                .on_selection_change(move |v| selected.revise(|old| *old = Some(v))),
        )
        .unwrap();
        app.settle(16).unwrap();
        let id = button(app, "Planning");
        let physical = host
            .renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap();
        let rect = bounds(app, id);
        assert!(rect.width > 180. && rect.height >= 32. && rect.y > 30.);
        let hit = host.renderer().inspect(|doc| {
            doc.hit(
                (rect.x + rect.width / 2.) as f32,
                (rect.y + rect.height / 2.) as f32,
            )
            .unwrap()
            .node_id
        });
        assert_eq!(hit, physical);
        app.scope().action(Action::Focus(id)).unwrap();
        app.settle(16).unwrap();
        assert_eq!(query(app, Query::Focus(id)), QueryValue::Focus(true));
        app.dispatch(id, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(
            matches!(selected.get(),Some(ScheduleSelection::Event{key,..})if key.as_str()=="planning")
        );
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let frame = raster.frame().unwrap();
        let rect = bounds(app, id);
        let at = ((rect.y as usize + rect.height as usize / 2) * frame.width as usize
            + rect.x as usize
            + 1)
            * 4;
        for (a, b) in raster.pixels()[at..at + 4].iter().zip([
            theme.accent.0,
            theme.accent.1,
            theme.accent.2,
            theme.accent.3,
        ]) {
            assert!(a.abs_diff(b) <= 2);
        }
        source.revise(|v| {
            *v = ScheduleData::new(
                ScheduleZone::Civil,
                v.window(),
                [ScheduleEvent::new(
                    ScheduleKey::new("planning").unwrap(),
                    "Changed planning",
                    ScheduleInterval::new(dt("2024-02-29T09:00"), dt("2024-02-29T11:00"))
                        .unwrap(),
                )
                .unwrap()],
            )
            .unwrap()
        });
        app.settle(16).unwrap();
        assert_eq!(button(app, "Changed planning"), id);
        assert_eq!(
            host.renderer()
                .engine_id(id, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        assert_eq!(query(app, Query::Focus(id)), QueryValue::Focus(true));
        app.clear().unwrap();
        app.pump().unwrap();
        assert_eq!(app.stats().queries, 0);
    }
}
#[test]
fn native_agenda_window_requests_readonly_empty_state_and_owner_retirement() {
    let host = WindowApp::new();
    host.renderer().set_viewport(900, 650, 1.).unwrap();
    let app = host.app();
    let source = app.scope().cage(data());
    let readonly = app.scope().cage(true);
    let root = app.scope().cage(None);
    let visible = app.scope().cage(true);
    let requests = app.scope().cage(0usize);
    host.mount(glory::ui::when(visible, move || {
        schedule("native-owner", source)
            .node_ref(root)
            .readonly(readonly)
            .on_window_request(move |request| {
                requests.revise(|n| *n += 1);
                source.revise(|v| {
                    *v = ScheduleData::new(request.zone, request.window, []).unwrap()
                });
            })
    }))
    .unwrap();
    app.settle(16).unwrap();
    assert!(
        !app.dispatch(button(app, "Planning"), Event::Activate)
            .unwrap()
    );
    app.dispatch(button(app, "Next schedule window"), Event::Activate)
        .unwrap();
    app.settle(16).unwrap();
    assert_eq!(requests.get(), 1);
    assert_eq!(source.with(|v| v.window().start().date()), d("2024-03-04"));
    let has_empty=all(app).into_iter().any(|id|{let tree=app.tree();matches!(tree.node(id).unwrap().descriptor.view().props,Props::Text(p)if p.content=="No events in this window")});
    assert!(has_empty);
    let old = root.get().unwrap();
    visible.revise(|v| *v = false);
    app.settle(16).unwrap();
    assert_eq!(root.get(), None);
    assert!(!app.dispatch(old, Event::Activate).unwrap());
    assert_eq!(app.stats().queries, 0);
}
