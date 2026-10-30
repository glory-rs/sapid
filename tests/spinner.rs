use glory::render::{GraphicAnimation, Props, Role};
use glory::style::{Color, Length, Style};
use glory::ui::App;
use sapid::{Size, Theme, loader, spinner};

#[test]
fn named_status_has_a_real_decorative_finite_graphic_and_no_input_handlers() {
    let app = App::memory();
    app.mount(
        spinner()
            .name("Loading files")
            .diameter(40.)
            .period_millis(800),
    )
    .unwrap();
    let tree = app.tree();
    let root = tree.node(tree.roots()[0]).unwrap();
    let descriptor = root.descriptor.view();
    assert_eq!(descriptor.role, Some(Role::Status));
    assert_eq!(descriptor.name.as_deref(), Some("Loading files"));
    assert_eq!(descriptor.semantics.as_ref().unwrap().busy, Some(true));
    let drawing = tree.node(root.children[0]).unwrap().descriptor.view();
    let Props::Graphic(props) = drawing.props else {
        panic!()
    };
    props.validate().unwrap();
    assert!(props.alternative.is_none());
    assert!(props.commands.len() <= 6);
    assert_eq!(
        props.animation,
        Some(GraphicAnimation::Rotate { period_millis: 800 })
    );
    assert_eq!(props.width, 40.);
    assert_eq!(app.stats().handlers, 0);
    let html = glory::ssr::render(loader().name("Loading files")).unwrap();
    assert!(html.contains("role=\"status\"") && html.contains("aria-busy=\"true\""));
    assert!(html.contains("data-gly-rotate=\"\"") && html.contains("animation-duration:1000ms"));
    assert!(html.contains("<svg") && !html.contains("tabindex"));
}

#[test]
fn every_theme_size_and_unstyled_keep_visible_content_and_explicit_overrides() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        for (size, expected) in [
            (Size::ExtraSmall, 12.),
            (Size::Small, 16.),
            (Size::Medium, 24.),
            (Size::Large, 32.),
            (Size::ExtraLarge, 40.),
        ] {
            for unstyled in [false, true] {
                let app = App::memory();
                let widget = spinner().theme(theme.clone()).size(size).decorative();
                app.mount(if unstyled { widget.unstyled() } else { widget })
                    .unwrap();
                let tree = app.tree();
                let root = tree.node(tree.roots()[0]).unwrap();
                assert_eq!(root.descriptor.view().role, None);
                assert_eq!(
                    root.descriptor.view().semantics.as_ref().unwrap().busy,
                    None
                );
                assert_eq!(
                    root.descriptor.view().style.width,
                    Some(Length::Px(expected))
                );
                let Props::Graphic(props) =
                    tree.node(root.children[0]).unwrap().descriptor.view().props
                else {
                    panic!()
                };
                assert_eq!(
                    props.stroke.unwrap().color,
                    if unstyled {
                        theme.foreground
                    } else {
                        theme.accent
                    }
                );
            }
        }
    }
    let app = App::memory();
    app.mount(spinner().unstyled().class("custom").style(Style {
        background: Some(Color(1, 2, 3, 255)),
        ..Style::new()
    }))
    .unwrap();
    let tree = app.tree();
    let d = tree.node(tree.roots()[0]).unwrap().descriptor.view();
    assert!(
        d.classes.iter().any(|v| v == "sapid-spinner") && d.classes.iter().any(|v| v == "custom")
    );
    assert_eq!(d.style.background, Some(Color(1, 2, 3, 255)));
}

#[test]
fn controlled_inputs_are_silent_preserve_identity_reject_invalid_names_and_retire() {
    let app = App::memory();
    let name = app.scope().cage("Fetching".to_string());
    let visible = app.scope().cage(true);
    let animated = app.scope().cage(true);
    let reference = app.scope().cage(None);
    app.mount(
        spinner()
            .name(name)
            .visible(visible)
            .animated(animated)
            .node_ref(reference),
    )
    .unwrap();
    let root = reference.get().unwrap();
    let graphic = app.tree().node(root).unwrap().children[0];
    name.revise(|s| *s = " ".into());
    assert!(app.flush().is_err());
    assert_eq!(name.get(), " ");
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Fetching")
    );
    name.revise(|s| *s = "Saving".into());
    visible.revise(|v| *v = false);
    animated.revise(|v| *v = false);
    app.flush().unwrap();
    assert_eq!(reference.get(), Some(root));
    let tree = app.tree();
    assert!(tree.node(root).unwrap().descriptor.view().hidden());
    let Props::Graphic(props) = tree.node(graphic).unwrap().descriptor.view().props else {
        panic!()
    };
    assert!(props.animation.is_none());
    drop(tree);
    visible.revise(|v| *v = true);
    animated.revise(|v| *v = true);
    app.flush().unwrap();
    assert_eq!(app.tree().node(root).unwrap().children[0], graphic);
    assert_eq!(app.stats().handlers, 0);
    app.clear().unwrap();
    assert_eq!(reference.get(), None);
    assert_eq!(app.stats().nodes, 0);
}

#[test]
fn invalid_initial_geometry_period_and_name_reject_before_creating_nodes() {
    for widget in [
        spinner().diameter(f64::NAN),
        spinner().diameter(7.),
        spinner().diameter(129.),
        spinner().thickness(0.),
        spinner().thickness(f64::INFINITY),
        spinner().diameter(8.).thickness(5.),
        spinner().period_millis(15),
        spinner().period_millis(60001),
        spinner().name(" "),
        spinner().name("x".repeat(257)),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert!(app.tree().roots().is_empty());
    }
}
