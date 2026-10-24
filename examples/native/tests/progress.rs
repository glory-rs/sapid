#[path = "../../web/src/progress_form.rs"]
mod progress_form;
use glory::blitz::Raster;
use glory::blitz::shell::WindowApp;
use glory::render::{Event, NodeId, Props, Query, QueryValue, Rect};
use glory::ui::App;
use std::{cell::RefCell, rc::Rc};
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            return id;
        }
        pending.extend(&node.children);
    }
    panic!("missing {class}")
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let result = Rc::new(RefCell::new(None));
    let reply = result.clone();
    app.request(Query::Bounds(id), move |value| {
        *reply.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = result.borrow_mut().take().unwrap() else {
        panic!()
    };
    rect
}
fn pixel(raster: &Raster, x: f64, y: f64) -> [u8; 4] {
    let width = raster.frame().unwrap().width as usize;
    let offset = (y.round() as usize * width + x.round() as usize) * 4;
    raster.pixels()[offset..offset + 4].try_into().unwrap()
}
fn rgba(color: glory::style::Color) -> [u8; 4] {
    [color.0, color.1, color.2, color.3]
}
#[test]
fn native_progress_has_real_fraction_geometry_circular_widgets_and_stable_updates_in_all_themes()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(progress_form::progress_form(theme.clone()))
            .unwrap();
        let app = host.app();
        app.settle(16).unwrap();
        let line = find(app, "progress-linear");
        let fill = app.tree().node(line).unwrap().children[0];
        let ring = find(app, "progress-ring");
        let arc = app.tree().node(ring).unwrap().children[1];
        let semi = find(app, "progress-semicircle");
        assert!((bounds(app, line).width - 240.).abs() < 0.1);
        assert!((bounds(app, fill).width - 60.).abs() < 0.1);
        assert!((bounds(app, ring).width - 100.).abs() < 0.1);
        assert!((bounds(app, semi).height - 66.).abs() < 0.1);
        let physical = host
            .renderer()
            .engine_id(arc, glory::html::Part::Main)
            .unwrap();
        host.renderer().inspect(|doc| {
            assert!(
                doc.get_node(physical)
                    .unwrap()
                    .element_data()
                    .unwrap()
                    .custom_widget_data()
                    .is_some()
            )
        });
        let ring_box = bounds(app, ring);
        let semi_box = bounds(app, semi);
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        assert_eq!(
            pixel(&raster, ring_box.x + 82., ring_box.y + 18.),
            rgba(theme.accent)
        );
        assert_eq!(
            pixel(&raster, ring_box.x + 50., ring_box.y + 95.),
            rgba(theme.border)
        );
        assert_eq!(
            pixel(&raster, semi_box.x + 10., semi_box.y + 39.),
            rgba(theme.accent)
        );
        assert_eq!(
            pixel(&raster, semi_box.x + 98., semi_box.y + 22.),
            rgba(theme.border)
        );
        let root = app.tree().roots()[0];
        let advance = app.tree().node(root).unwrap().children[3];
        app.dispatch(advance, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!((bounds(app, fill).width - 180.).abs() < 0.1);
        raster.draw(host.renderer()).unwrap();
        assert_eq!(
            pixel(&raster, ring_box.x + 50., ring_box.y + 95.),
            rgba(theme.accent)
        );
        assert_eq!(
            pixel(&raster, semi_box.x + 98., semi_box.y + 22.),
            rgba(theme.accent)
        );
        assert!(
            matches!(app.tree().node(arc).unwrap().descriptor.view().props,Props::Graphic(props) if props.commands.len()==4)
        );
        assert_eq!(
            host.renderer()
                .engine_id(arc, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        let toggle = app.tree().node(root).unwrap().children[4];
        app.dispatch(toggle, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!(
            app.tree()
                .node(line)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .progress
                .as_ref()
                .unwrap()
                .current
                .is_none()
        );
        assert!((bounds(app, fill).width - 72.).abs() < 0.1);
        app.dispatch(toggle, Event::Activate).unwrap();
        app.settle(16).unwrap();
        assert!((bounds(app, fill).width - 180.).abs() < 0.1);
        app.clear().unwrap();
        assert_eq!(host.renderer().stats().nodes, 0);
    }
}
