#[path = "../../web/src/spinner_form.rs"]
mod spinner_form;
use glory::apace::shell::WindowApp;
use glory::apace::{Config, Raster};
use glory::render::{Event, MotionPreference, NodeId};
use glory::ui::App;
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|s| s == class) {
            return id;
        }
        pending.extend(&node.children);
    }
    panic!("missing {class}");
}
fn press(app: &App, label: &str) {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let id = loop {
        let id = pending.pop().unwrap();
        let n = tree.node(id).unwrap();
        if matches!(n.descriptor.view().props,glory::render::Props::Button(props) if props.label == label)
        {
            break id;
        }
        pending.extend(&n.children);
    };
    drop(tree);
    app.dispatch(id, Event::Activate).unwrap();
    app.flush().unwrap();
}
#[test]
fn real_spinner_pixels_rotate_only_when_host_allows_motion_and_controlled_visibility_retires_cleanly()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::with_config(Config {
            motion: MotionPreference::NoPreference,
            ..Default::default()
        });
        host.mount(spinner_form::spinner_form(theme)).unwrap();
        host.app().settle(16).unwrap();
        let root = find(host.app(), "spinner-controlled");
        let graphic = host.app().tree().node(root).unwrap().children[0];
        let physical = host
            .renderer()
            .engine_id(graphic, glory::html::Part::Main)
            .unwrap();
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        assert!(host.renderer().inspect(|doc| doc.is_animating()));
        let before = raster.pixels().to_vec();
        let writes = host.app().runtime_stats().source_writes;
        host.renderer().advance_time(0.25).unwrap();
        raster.draw(host.renderer()).unwrap();
        assert!(
            before
                .iter()
                .zip(raster.pixels())
                .filter(|(a, b)| a != b)
                .count()
                > 100
        );
        assert_eq!(host.app().runtime_stats().source_writes, writes);
        assert_eq!(
            host.renderer()
                .engine_id(graphic, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        host.renderer()
            .set_motion_preference(MotionPreference::Reduce);
        raster.draw(host.renderer()).unwrap();
        assert!(!host.renderer().inspect(|doc| doc.is_animating()));
        let revision = host.renderer().stats().visual_revision;
        host.renderer().advance_time(0.5).unwrap();
        assert_eq!(host.renderer().stats().visual_revision, revision);
        assert!(!raster.draw(host.renderer()).unwrap().rendered);
        press(host.app(), "Toggle spinner animation");
        press(host.app(), "Toggle spinner visibility");
        assert!(
            host.app()
                .tree()
                .node(root)
                .unwrap()
                .descriptor
                .view()
                .hidden()
        );
        press(host.app(), "Toggle spinner visibility");
        assert_eq!(
            host.renderer()
                .engine_id(graphic, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        host.app().clear().unwrap();
        assert_eq!(host.renderer().stats().nodes, 0);
        assert!(!host.renderer().inspect(|doc| doc.is_animating()));
    }
}
#[test]
fn native_unknown_preference_defaults_to_a_static_visible_spinner() {
    let host = WindowApp::new();
    host.mount(sapid::spinner()).unwrap();
    let mut raster = Raster::new();
    raster.draw(host.renderer()).unwrap();
    assert!(
        raster
            .pixels()
            .as_chunks::<4>()
            .0
            .iter()
            .any(|rgba| rgba[3] > 0)
    );
    assert!(!host.renderer().inspect(|doc| doc.is_animating()));
    let revision = host.renderer().stats().visual_revision;
    host.renderer().advance_time(1.).unwrap();
    assert_eq!(host.renderer().stats().visual_revision, revision);
}
