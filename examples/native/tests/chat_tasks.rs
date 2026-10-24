#[path = "../../web/tasks_form.rs"]
mod fixture;
use glory::{
    blitz::{Raster, shell::WindowApp},
    prelude::*,
    render::{Action, NodeId, Props, Query, QueryValue, Rect},
};
use std::{cell::RefCell, rc::Rc};
fn descendants(app: &App, roots: Vec<NodeId>) -> Vec<NodeId> {
    let t = app.tree();
    let mut pending = roots;
    let mut out = vec![];
    while let Some(id) = pending.pop() {
        pending.extend(&t.node(id).unwrap().children);
        out.push(id)
    }
    out
}
fn all(app: &App) -> Vec<NodeId> {
    descendants(app, app.tree().roots().to_vec())
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
                .any(|c| c == name)
        })
        .collect()
}
fn button(app: &App, name: &str) -> NodeId {
    all(app).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(p)if p.label==name)).unwrap()
}
fn check(app: &App, name: &str) -> NodeId {
    all(app).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Check(p)if p.label==name)).unwrap()
}
fn content(app: &App) -> String {
    all(app)
        .into_iter()
        .filter_map(
            |id| match app.tree().node(id).unwrap().descriptor.view().props {
                Props::Text(p) => Some(p.content.clone()),
                _ => None,
            },
        )
        .collect::<Vec<_>>()
        .join("|")
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
#[test]
fn plans_tasks_and_queues_have_native_four_theme_progress_actions_focus_keyed_updates_and_pixels()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(1100, 1800, 1.).unwrap();
        host.mount(fixture::tasks_form(theme.clone())).unwrap();
        let app = host.app();
        app.settle(24).unwrap();
        let plan = class(app, "plan-primary")[0];
        let action=descendants(app,vec![plan]).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(p)if p.label=="Request task cancellation")).unwrap();
        let r = bounds(app, action);
        assert!(r.width > 40. && r.height > 10. && r.y < 1800., "{r:?}");
        let physical = host
            .renderer()
            .engine_id(action, glory::html::Part::Main)
            .unwrap();
        let hit = host.renderer().inspect(|dom| {
            dom.hit((r.x + r.width / 2.) as f32, (r.y + r.height / 2.) as f32)
                .unwrap()
                .node_id
        });
        assert_eq!(hit, physical);
        app.scope().action(Action::Focus(action)).unwrap();
        app.settle(16).unwrap();
        assert_eq!(query(app, Query::Focus(action)), QueryValue::Focus(true));
        app.dispatch(action, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(content(app).contains("Application task requests: 1"));
        assert!(content(app).contains("Running · Prepare change"));
        app.dispatch(button(app, "Update current task"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert!(content(app).contains("Updated preparation task"));
        assert_eq!(
            host.renderer()
                .engine_id(action, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        app.dispatch(
            check(app, "Enable task actions"),
            Event::CheckedChanged(false),
        )
        .unwrap();
        app.settle(16).unwrap();
        assert!(
            matches!(app.tree().node(action).unwrap().descriptor.view().props,Props::Button(p)if !p.enabled)
        );
        app.dispatch(
            check(app, "Enable task actions"),
            Event::CheckedChanged(true),
        )
        .unwrap();
        app.dispatch(
            check(app, "Accept task proposals"),
            Event::CheckedChanged(true),
        )
        .unwrap();
        app.settle(16).unwrap();
        app.dispatch(action, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(content(app).contains("Cancelled · Updated preparation task"));
        assert!(content(app).contains("Processed 2 of 3 tasks"));
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let c = theme.surface;
        assert!(
            raster
                .pixels()
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| **p == [c.0, c.1, c.2, c.3])
                .count()
                > 1000
        );
        app.dispatch(button(app, "Empty supplied tasks"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert!(class(app, "sapid-plan-progress").is_empty());
        assert_eq!(class(app, "sapid-task").len(), 1);
        app.dispatch(button(app, "Toggle task owner"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert!(class(app, "sapid-task").is_empty());
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(host.renderer().stats().nodes, 0);
    }
}
