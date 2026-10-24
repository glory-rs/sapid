#[path = "../../web/src/trace_form.rs"]
mod fixture;
use glory::{
    blitz::{Raster, shell::WindowApp},
    prelude::*,
    render::{Action, NodeId, Props, Query, QueryValue, Rect},
};
use std::{cell::RefCell, rc::Rc};
fn all(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = pending.pop() {
        pending.extend(&tree.node(id).unwrap().children);
        out.push(id)
    }
    out
}
fn class(app: &App, name: &str) -> Vec<NodeId> {
    all(app)
        .into_iter()
        .filter(|&id| {
            app.tree()
                .node(id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|c| c == name)
        })
        .collect()
}
fn button(app: &App, prefix: &str) -> NodeId {
    all(app).into_iter().find(|&id|matches!(app.tree().node(id).unwrap().descriptor.view().props,Props::Button(p)if p.label.starts_with(prefix))).unwrap()
}
fn text(app: &App) -> String {
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
    let response = Rc::new(RefCell::new(None));
    let out = response.clone();
    app.request(q, move |v| *out.borrow_mut() = Some(v.unwrap()))
        .unwrap();
    app.pump().unwrap();
    response.borrow_mut().take().unwrap()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let QueryValue::Bounds(r) = query(app, Query::Bounds(id)) else {
        panic!("bounds")
    };
    r
}
#[test]
fn trace_tool_and_subagent_have_real_four_theme_geometry_focus_pixels_and_owner_retirement() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(1000, 2200, 1.).unwrap();
        let store = Rc::new(RefCell::new(None));
        let out = store.clone();
        let palette = theme.clone();
        host.mount(widget(move |scope| {
            let (w, s) = fixture::form(scope, palette.clone(), false);
            *out.borrow_mut() = Some(s);
            w.show_in(scope)
        }))
        .unwrap();
        let app = host.app();
        app.settle(24).unwrap();
        let state = store.borrow().as_ref().unwrap().clone();
        assert_eq!(state.data.with(|d| d.records().len()), 4);
        assert_eq!(state.stack.with(|d| d.frames().len()), 3);
        let trigger = button(app, "Span · Application request");
        let id = host
            .renderer()
            .engine_id(trigger, glory::html::Part::Main)
            .unwrap();
        let r = bounds(app, trigger);
        assert!(r.width > 120. && r.height > 10., "{r:?}");
        let hit = host.renderer().inspect(|dom| {
            dom.hit((r.x + r.width / 2.) as f32, (r.y + r.height / 2.) as f32)
                .unwrap()
                .node_id
        });
        assert_eq!(id, hit);
        app.scope().action(Action::Focus(trigger)).unwrap();
        app.settle(16).unwrap();
        assert_eq!(query(app, Query::Focus(trigger)), QueryValue::Focus(true));
        let initial = class(app, "sapid-trace-payload").len();
        app.dispatch(trigger, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(class(app, "sapid-trace-payload").len() < initial);
        state.accept.revise(|v| *v = false);
        app.dispatch(trigger, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(
            !state
                .expanded
                .with(|e| e.contains(&sapid::chat::TraceKey::new("request").unwrap()))
        );
        state.accept.revise(|v| *v = true);
        app.dispatch(trigger, Event::Activate).unwrap();
        app.settle(16).unwrap();
        app.dispatch(button(app, "Update accepted record"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert_eq!(trigger, button(app, "Span · Updated request"));
        assert_eq!(
            host.renderer()
                .engine_id(trigger, glory::html::Part::Main)
                .unwrap(),
            id
        );
        assert!(text(app).contains("[redacted]"));
        assert!(text(app).contains("<script>window.traceExecuted"));
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
                > 400
        );
        state.enabled.revise(|v| *v = false);
        app.settle(16).unwrap();
        assert!(!app.dispatch(trigger, Event::Activate).unwrap());
        state.visible.revise(|v| *v = false);
        app.settle(16).unwrap();
        assert!(state.reference.get().is_none());
        assert!(class(app, "sapid-trace").is_empty());
        state.visible.revise(|v| *v = true);
        app.settle(16).unwrap();
        assert_ne!(trigger, button(app, "Span · Updated request"));
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(host.renderer().stats().nodes, 0);
    }
}
#[test]
fn stack_frames_use_literal_code_and_current_explicit_location_actions_with_no_native_url() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(1000, 1700, 1.).unwrap();
        let out = Rc::new(RefCell::new(None));
        let saved = out.clone();
        host.mount(widget(move |scope| {
            let (w, s) = fixture::form(scope, theme.clone(), true);
            *saved.borrow_mut() = Some(s);
            w.show_in(scope)
        }))
        .unwrap();
        let app = host.app();
        app.settle(24).unwrap();
        let location = button(app, "src/parser.rs:12:3");
        let r = bounds(app, location);
        assert!(r.width > 50. && r.height > 10.);
        app.scope().action(Action::Focus(location)).unwrap();
        app.settle(16).unwrap();
        assert_eq!(query(app, Query::Focus(location)), QueryValue::Focus(true));
        app.dispatch(location, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(text(app).contains("location:parse:12;generic;"));
        app.dispatch(button(app, "Update accepted record"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert_eq!(location, button(app, "src/parser.rs:44:3"));
        app.dispatch(location, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(text(app).contains("location:parse:44;generic;"));
        assert!(!all(app).iter().any(|&id| matches!(
            app.tree().node(id).unwrap().descriptor.view().props,
            Props::Link(_)
        )));
        assert!(text(app).contains("<img src=x onerror="));
        assert_eq!(class(app, "sapid-stack-frame").len(), 9);
        let state = out.borrow().as_ref().unwrap().clone();
        state.enabled.revise(|v| *v = false);
        app.settle(16).unwrap();
        assert!(!app.dispatch(location, Event::Activate).unwrap());
        state.visible.revise(|v| *v = false);
        app.settle(16).unwrap();
        assert!(state.reference.get().is_none());
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
    }
}
