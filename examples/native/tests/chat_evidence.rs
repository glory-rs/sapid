#[path = "../../web/src/chat_evidence_form.rs"]
mod fixture;
use glory::{
    apace::{Raster, shell::WindowApp},
    prelude::*,
    render::{Action, NodeId, Props, Query, QueryValue, Rect},
};
use std::{cell::RefCell, rc::Rc};
fn descendants(app: &App, roots: Vec<NodeId>) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = roots;
    let mut out = vec![];
    while let Some(id) = pending.pop() {
        pending.extend(&tree.node(id).unwrap().children);
        out.push(id);
    }
    out
}
fn all(app: &App) -> Vec<NodeId> {
    descendants(app, app.tree().roots().to_vec())
}
fn class_in(app: &App, roots: Vec<NodeId>, name: &str) -> Vec<NodeId> {
    descendants(app, roots)
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
fn class(app: &App, name: &str) -> Vec<NodeId> {
    class_in(app, app.tree().roots().to_vec(), name)
}
fn button(app: &App, name: &str) -> NodeId {
    all(app).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(p)if p.label==name)).unwrap()
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
fn bounds(app: &App, id: NodeId) -> Rect {
    let value = Rc::new(RefCell::new(None));
    let out = value.clone();
    app.request(Query::Bounds(id), move |v| {
        *out.borrow_mut() = Some(v.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(r) = value.borrow_mut().take().unwrap() else {
        panic!()
    };
    r
}
#[test]
fn public_disclosures_and_citations_have_real_native_four_theme_text_controls_focus_ranges_and_cleanup()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        for citations in [false, true] {
            let host = WindowApp::new();
            host.renderer().set_viewport(1000, 1400, 1.).unwrap();
            host.mount(fixture::chat_evidence_form(theme.clone(), citations))
                .unwrap();
            let app = host.app();
            app.settle(16).unwrap();
            let target = if citations {
                class(app, "sapid-inline-citation")[0]
            } else {
                let primary = class(app, "reasoning-primary");
                class_in(app, primary, "sapid-collapsible-trigger")[0]
            };
            let rect = bounds(app, target);
            assert!(
                rect.width > 5. && rect.height > 5. && rect.y < 1400.,
                "{rect:?}"
            );
            let physical = host
                .renderer()
                .engine_id(target, glory::html::Part::Main)
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
            app.scope().action(Action::Focus(target)).unwrap();
            app.settle(16).unwrap();
            if citations {
                let link = class(app, "sapid-source-link")[0];
                app.dispatch(link, Event::Activate).unwrap();
                app.dispatch(button(app, "Overflow context capacity"), Event::Activate)
                    .unwrap();
                app.settle(16).unwrap();
                assert!(content(app).contains("Source requests: 1"));
                assert!(content(app).contains("120 of 100 tokens; 20 over capacity"));
                app.dispatch(button(app, "Update supplied source"), Event::Activate)
                    .unwrap();
                app.settle(16).unwrap();
                assert_eq!(
                    host.renderer().engine_id(target, glory::html::Part::Main),
                    Ok(physical)
                );
                assert!(
                    matches!(app.tree().node(link).unwrap().descriptor.view().props,Props::Link(p)if p.label=="Updated design notes")
                );
                assert!(content(app).contains("Application trust: verified"));
            } else {
                app.dispatch(target, Event::Activate).unwrap();
                app.settle(16).unwrap();
                let primary = class(app, "reasoning-primary");
                assert_eq!(class_in(app, primary, "sapid-explanation-step").len(), 2);
                assert!(content(app).contains("Complete · 32 ms"));
                app.dispatch(button(app, "Update public explanation"), Event::Activate)
                    .unwrap();
                app.settle(16).unwrap();
                assert!(
                    content(app).contains("Updated public summary supplied by the application")
                );
                assert_eq!(
                    host.renderer().engine_id(target, glory::html::Part::Main),
                    Ok(physical)
                );
            }
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
            app.clear().unwrap();
            assert_eq!(app.stats().nodes, 0);
            assert_eq!(host.renderer().stats().nodes, 0);
        }
    }
}
