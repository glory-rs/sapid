#[path = "../../web/src/output_form.rs"]
mod fixture;
use glory::{
    apace::{Raster, shell::WindowApp},
    prelude::*,
    render::{Action, NodeId, Props, Query, QueryValue, Rect},
};
use std::{cell::RefCell, rc::Rc};
fn all(app: &App) -> Vec<NodeId> {
    let t = app.tree();
    let mut pending = t.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = pending.pop() {
        pending.extend(&t.node(id).unwrap().children);
        out.push(id)
    }
    out
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
    all(app).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(p)if p.label==name)).unwrap_or_else(||panic!("missing {name}; buttons {:?}; content {}",all(app).into_iter().filter_map(|id|match app.tree().node(id).unwrap().descriptor.view().props{Props::Button(p)=>Some(p.label.clone()),_=>None}).collect::<Vec<_>>(),content(app)))
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
    let r = Rc::new(RefCell::new(None));
    let out = r.clone();
    app.request(Query::Bounds(id), move |v| {
        *out.borrow_mut() = Some(v.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(r) = r.borrow_mut().take().unwrap() else {
        panic!()
    };
    r
}
#[test]
fn output_has_real_native_four_theme_viewports_focus_disclosures_pixels_and_bounded_large_sources()
{
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(1000, 1600, 1.).unwrap();
        host.mount(fixture::output_form(theme.clone())).unwrap();
        let app = host.app();
        assert!(app.stats().nodes > 0, "fixture initially mounted");
        app.settle(24).unwrap();
        assert!(
            app.stats().nodes > 0,
            "fixture remains mounted after native observations"
        );
        let trigger = button(app, "Failed · renderer applies accepted prefix · 18 ms");
        let r = bounds(app, trigger);
        assert!(r.width > 100. && r.height > 10. && r.y < 1600., "{r:?}");
        let physical = host
            .renderer()
            .engine_id(trigger, glory::html::Part::Main)
            .unwrap();
        let hit = host.renderer().inspect(|dom| {
            dom.hit((r.x + r.width / 2.) as f32, (r.y + r.height / 2.) as f32)
                .unwrap()
                .node_id
        });
        assert_eq!(hit, physical);
        app.scope().action(Action::Focus(trigger)).unwrap();
        app.dispatch(trigger, Event::Activate).unwrap();
        app.settle(24).unwrap();
        assert!(content(app).contains("Application-supplied test output"));
        assert!(content(app).contains("\\u{1b}[31m"));
        assert_eq!(
            host.renderer()
                .engine_id(trigger, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        app.dispatch(button(app, "Use billion records"), Event::Activate)
            .unwrap();
        app.settle(24).unwrap();
        assert!(class(app, "sapid-log-record").len() <= 32);
        assert!(content(app).contains("Commands executed by this display: 0"));
        let stats = content(app);
        assert!(stats.contains("range 999999"), "{stats}");
        assert!(app.stats().nodes < 1500);
        app.dispatch(button(app, "Change output heights"), Event::Activate)
            .unwrap();
        app.settle(24).unwrap();
        assert!(content(app).contains("A taller accepted row"));
        assert_eq!(
            host.renderer()
                .engine_id(trigger, glory::html::Part::Main)
                .unwrap(),
            physical
        );
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
        app.dispatch(button(app, "Toggle output owner"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert!(class(app, "sapid-terminal").is_empty());
        assert!(class(app, "sapid-test-results").is_empty());
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(host.renderer().stats().nodes, 0);
    }
}
