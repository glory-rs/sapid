use glory::prelude::*;
use glory::render::{Event, EventKind, NodeId, Props};
use glory::style::{Color, Length};
use sapid::{Theme, color_swatch};
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
        pending.extend(&node.children)
    }
    panic!("missing {class}")
}
#[test]
fn passive_preview_keeps_alpha_in_its_path_and_has_no_fake_control_role() {
    let app = App::memory();
    app.mount(color_swatch(Color(255, 0, 0, 128)).name("Transparent red"))
        .unwrap();
    let root = find(&app, "sapid-color-swatch");
    let tree = app.tree();
    let node = tree.node(root).unwrap().descriptor.view();
    assert!(matches!(node.props, Props::View));
    assert_eq!(node.name.as_deref(), Some("Transparent red"));
    let drawing = find(&app, "sapid-color-swatch-color");
    assert!(
        matches!(tree.node(drawing).unwrap().descriptor.view().props,Props::Graphic(p)if p.fill==Some(Color(255,0,0,128))&&p.alternative.is_none())
    );
    let checker = find(&app, "sapid-color-swatch-checker");
    assert!(
        matches!(tree.node(checker).unwrap().descriptor.view().props,Props::Graphic(p)if p.commands.len()==40)
    );
    drop(tree);
    assert!(!app.dispatch(root, Event::Activate).unwrap());
}
#[test]
fn controlled_pressed_state_proposes_before_press_and_observer_and_keeps_identity() {
    let app = App::memory();
    let source = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    let color = app.scope().cage(Color(0, 0, 255, 255));
    let log = Rc::new(RefCell::new(Vec::new()));
    let change = log.clone();
    let press = log.clone();
    let observe = log.clone();
    app.mount(
        color_swatch(color)
            .pressed(source)
            .enabled(enabled)
            .on_pressed_change(move |value| change.borrow_mut().push(format!("change:{value}")))
            .on_press(move || press.borrow_mut().push("press".into()))
            .on(EventKind::Activate, move |_| {
                observe.borrow_mut().push("observe".into())
            }),
    )
    .unwrap();
    let root = find(&app, "sapid-color-swatch");
    let drawing = find(&app, "sapid-color-swatch-color");
    let mark = find(&app, "sapid-color-swatch-check");
    app.dispatch(root, Event::Activate).unwrap();
    assert!(!source.get());
    assert_eq!(*log.borrow(), ["change:true", "press", "observe"]);
    assert!(
        matches!(app.tree().node(mark).unwrap().descriptor.view().props,Props::Graphic(p)if p.commands.is_empty())
    );
    source.revise(|v| *v = true);
    color.revise(|v| *v = Color(0, 255, 0, 128));
    app.flush().unwrap();
    assert_eq!(find(&app, "sapid-color-swatch-color"), drawing);
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Color #00ff0080")
    );
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .pressed,
        Some(true)
    );
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(!app.dispatch(root, Event::Activate).unwrap());
    assert_eq!(log.borrow().len(), 3);
}
#[test]
fn local_toggle_is_real_button_and_ordinary_action_has_no_pressed_metadata() {
    let app = App::memory();
    app.mount(color_swatch(Color(0, 0, 0, 255)).default_pressed(false))
        .unwrap();
    let root = find(&app, "sapid-color-swatch");
    app.dispatch(root, Event::Activate).unwrap();
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .pressed,
        Some(true)
    );
    let ordinary =
        glory::ssr::render(color_swatch(Color(255, 255, 255, 255)).on_press(|| {})).unwrap();
    assert!(ordinary.contains("<button") && ordinary.contains("Color #ffffffff"));
    assert!(!ordinary.contains("aria-pressed"));
}
#[test]
fn invalid_size_and_names_reject_without_losing_accepted_reactive_name() {
    for side in [f64::NAN, f64::INFINITY, 0., 129.] {
        assert!(
            App::memory()
                .mount(color_swatch(Color(0, 0, 0, 0)).side(side))
                .is_err()
        )
    }
    let app = App::memory();
    let name = app.scope().cage("Accepted".to_string());
    app.mount(color_swatch(Color(0, 0, 0, 0)).name(name))
        .unwrap();
    let root = find(&app, "sapid-color-swatch");
    name.revise(|v| *v = "x".repeat(1_000_000));
    assert!(app.flush().is_err());
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Accepted")
    );
    name.revise(|v| *v = "Recovered".into());
    app.flush().unwrap();
    assert_eq!(find(&app, "sapid-color-swatch"), root);
}
#[test]
fn four_themes_unstyled_round_shape_and_owner_cleanup_keep_preview_geometry() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let app = App::memory();
        let mounted = app
            .scope()
            .mount_with(|| {
                color_swatch(Color(0, 0, 0, 0))
                    .theme(theme)
                    .side(40.)
                    .round(true)
                    .unstyled()
            })
            .unwrap();
        let root = find(&app, "sapid-color-swatch");
        let tree = app.tree();
        let descriptor = tree.node(root).unwrap().descriptor.view();
        assert_eq!(descriptor.style.width, Some(Length::Px(40.)));
        assert_eq!(descriptor.style.radius, Some(20.));
        assert_eq!(descriptor.style.background, Some(Color(255, 255, 255, 255)));
        assert!(descriptor.style.border_color.is_none());
        drop(tree);
        drop(mounted);
        app.flush().unwrap();
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(app.stats().handlers, 0)
    }
}
