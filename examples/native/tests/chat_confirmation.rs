#[path = "../../web/src/confirmation_form.rs"]
mod fixture;
use glory::{
    blitz::{Raster, shell::WindowApp},
    prelude::*,
    render::{Action, NodeId, Props, Query, QueryValue, Rect},
};
use std::{cell::RefCell, rc::Rc};
fn nodes(app: &App, roots: Vec<NodeId>) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = roots;
    let mut out = vec![];
    while let Some(id) = pending.pop() {
        pending.extend(&tree.node(id).unwrap().children);
        out.push(id)
    }
    out
}
fn class(app: &App, name: &str) -> NodeId {
    nodes(app, app.tree().roots().to_vec())
        .into_iter()
        .find(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|class| class == name)
        })
        .unwrap()
}
fn button(app: &App, label: &str) -> NodeId {
    nodes(app,app.tree().roots().to_vec()).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(p)if p.label==label)).unwrap()
}
fn check(app: &App, label: &str) -> NodeId {
    nodes(app,app.tree().roots().to_vec()).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Check(p)if p.label==label)).unwrap()
}
fn content(app: &App) -> String {
    nodes(app, app.tree().roots().to_vec())
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
fn query(app: &App, query: Query) -> QueryValue {
    let result = Rc::new(RefCell::new(None));
    let out = result.clone();
    app.request(query, move |value| *out.borrow_mut() = Some(value.unwrap()))
        .unwrap();
    app.pump().unwrap();
    result.borrow_mut().take().unwrap()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let QueryValue::Bounds(value) = query(app, Query::Bounds(id)) else {
        panic!()
    };
    value
}
#[test]
fn confirmation_and_checkpoint_native_four_themes_have_real_geometry_focus_pixels_and_application_owned_state()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(1100, 2400, 1.).unwrap();
        host.mount(fixture::confirmation_form(theme.clone()))
            .unwrap();
        let app = host.app();
        app.settle(24).unwrap();
        let primary = class(app, "confirm-primary");
        let approve=nodes(app,vec![primary]).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(p)if p.label=="Approve request")).unwrap();
        let rect = bounds(app, approve);
        assert!(
            rect.width > 60. && rect.height >= 20. && rect.y < 2400.,
            "{rect:?}"
        );
        let physical = host
            .renderer()
            .engine_id(approve, glory::html::Part::Main)
            .unwrap();
        let hit = host.renderer().inspect(|dom| {
            dom.hit(
                (rect.x + rect.width / 2.) as f32,
                (rect.y + rect.height / 2.) as f32,
            )
            .unwrap()
            .node_id
        });
        assert_eq!(hit, physical);
        app.scope().action(Action::Focus(approve)).unwrap();
        app.settle(16).unwrap();
        assert_eq!(query(app, Query::Focus(approve)), QueryValue::Focus(true));
        app.dispatch(approve, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(
            content(app).contains("Requests:1; Last:deployment-v1@3:Approve")
                && content(app).contains("Awaiting confirmation")
        );
        app.dispatch(
            check(app, "Application accepts requests"),
            Event::CheckedChanged(true),
        )
        .unwrap();
        app.settle(16).unwrap();
        app.dispatch(button(app, "Reject request"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert!(content(app).contains("Rejected by application"));
        assert_eq!(
            host.renderer()
                .engine_id(approve, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        let saved = class(app, "checkpoint-primary");
        let restore=nodes(app,vec![saved]).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(p)if p.label=="Restore checkpoint")).unwrap();
        app.dispatch(restore, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(content(app).contains("Restoration in progress"));
        app.dispatch(button(app, "Mark restore complete"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert!(content(app).contains("Restored by application"));
        let root = bounds(app, primary);
        let saved = bounds(app, saved);
        assert!(root.width > 300. && root.height > 60. && saved.y >= root.y + root.height - 1.);
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let color = theme.surface;
        assert!(
            raster
                .pixels()
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|pixel| **pixel == [color.0, color.1, color.2, color.3])
                .count()
                > 3000
        );
        app.dispatch(
            check(app, "Enable confirmation actions"),
            Event::CheckedChanged(false),
        )
        .unwrap();
        app.dispatch(button(app, "Reset confirmation data"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert!(
            matches!(app.tree().node(approve).unwrap().descriptor.view().props,Props::Button(p)if !p.enabled)
        );
        app.dispatch(button(app, "Toggle confirmation owner"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert!(app.tree().node(primary).is_err());
        assert!(
            host.renderer()
                .engine_id(approve, glory::html::Part::Main)
                .is_err()
        );
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(host.renderer().stats().nodes, 0);
    }
}
