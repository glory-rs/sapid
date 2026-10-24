use glory::render::{NodeId, Props, Role};
use glory::style::{Display, Edges, Length, Position, Style};
use glory::ui::App;
use sapid::{IndicatorPosition, Theme, indicator};

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
            .any(|candidate| candidate == class)
        {
            return id;
        }
        pending.extend(&node.children);
    }
    panic!("missing {class}")
}
#[test]
fn unnamed_dot_is_passive_and_preserves_independent_child_control() {
    let app = App::memory();
    app.mount(indicator().fill(glory::ui::button().label("Messages")))
        .unwrap();
    let marker = find(&app, "sapid-indicator-marker");
    let tree = app.tree();
    let descriptor = tree.node(marker).unwrap().descriptor.view();
    assert_eq!(descriptor.effective_role(), None);
    assert!(descriptor.name.is_none());
    assert!(tree.node(marker).unwrap().children.is_empty());
    assert_eq!(descriptor.style.position, Some(Position::Absolute));
    assert_eq!(descriptor.style.width, Some(Length::Px(12.0)));
    assert_eq!(descriptor.style.left, Some(Length::Percent(100.0)));
    let root = tree.node(tree.roots()[0]).unwrap();
    assert!(matches!(
        tree.node(root.children[0]).unwrap().descriptor.view().props,
        Props::Button(_)
    ));
    assert_eq!(app.stats().handlers, 0);
}
#[test]
fn all_nine_anchors_and_themes_have_finite_marker_geometry() {
    use IndicatorPosition::*;
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        for (position, left, top) in [
            (TopLeft, 0., 0.),
            (TopCenter, 50., 0.),
            (TopRight, 100., 0.),
            (MiddleLeft, 0., 50.),
            (MiddleCenter, 50., 50.),
            (MiddleRight, 100., 50.),
            (BottomLeft, 0., 100.),
            (BottomCenter, 50., 100.),
            (BottomRight, 100., 100.),
        ] {
            let app = App::memory();
            app.mount(
                indicator()
                    .theme(theme.clone())
                    .position(position)
                    .diameter(24.0)
                    .label("3")
                    .name("Unread messages"),
            )
            .unwrap();
            let marker = find(&app, "sapid-indicator-marker");
            let tree = app.tree();
            let descriptor = tree.node(marker).unwrap().descriptor.view();
            assert_eq!(descriptor.effective_role(), Some(Role::Group));
            assert_eq!(descriptor.name.as_deref(), Some("Unread messages"));
            assert_eq!(descriptor.style.left, Some(Length::Percent(left)));
            assert_eq!(descriptor.style.top, Some(Length::Percent(top)));
            assert_eq!(descriptor.style.background, Some(theme.accent));
            assert_eq!(
                descriptor.style.margin,
                Some(Edges {
                    top: -12.0,
                    left: -12.0,
                    right: 0.0,
                    bottom: 0.0
                })
            );
        }
    }
}
#[test]
fn invalid_initial_inputs_create_no_nodes() {
    for diameter in [f64::NAN, f64::INFINITY, -1.0, 0.0, 3.9, 129.0] {
        let app = App::memory();
        assert!(app.mount(indicator().diameter(diameter)).is_err());
        assert!(app.tree().roots().is_empty());
    }
    for label in ["".to_string(), " ".to_string(), "x".repeat(33)] {
        let app = App::memory();
        assert!(app.mount(indicator().label(label)).is_err());
        assert!(app.tree().roots().is_empty());
    }
    let app = App::memory();
    assert!(app.mount(indicator().name(" ")).is_err());
    assert!(app.tree().roots().is_empty());
}
#[test]
fn controlled_label_name_visibility_preserve_identity_and_recover_without_callbacks() {
    let app = App::memory();
    let label = app.scope().cage("3".to_string());
    let name = app.scope().cage("3 unread".to_string());
    let visible = app.scope().cage(true);
    app.mount(
        indicator()
            .label(label)
            .name(name)
            .visible(visible)
            .fill(glory::ui::input().value("retained")),
    )
    .unwrap();
    let root = app.tree().roots()[0];
    let marker = find(&app, "sapid-indicator-marker");
    let text = find(&app, "sapid-indicator-label");
    label.revise(|value| *value = " ".into());
    name.revise(|value| *value = "x".repeat(257));
    assert!(app.flush().is_err());
    assert_eq!(label.get(), " ");
    {
        let tree = app.tree();
        let Props::Text(value) = tree.node(text).unwrap().descriptor.view().props else {
            panic!()
        };
        assert_eq!(value.content, "3");
        assert_eq!(
            tree.node(marker).unwrap().descriptor.view().name.as_deref(),
            Some("3 unread")
        );
    }
    label.revise(|value| *value = "4".into());
    name.revise(|value| *value = "4 unread".into());
    visible.revise(|value| *value = false);
    app.flush().unwrap();
    assert!(app.tree().node(marker).unwrap().descriptor.view().hidden());
    visible.revise(|value| *value = true);
    app.flush().unwrap();
    assert_eq!(find(&app, "sapid-indicator-label"), text);
    assert_eq!(app.tree().roots()[0], root);
    assert_eq!(app.stats().handlers, 0);
    app.clear().unwrap();
    assert_eq!(app.stats().handlers, 0);
}
#[test]
fn unstyled_keeps_structural_anchors_and_custom_root_overrides() {
    let app = App::memory();
    app.mount(
        indicator()
            .unstyled()
            .style(Style::new().width(Length::Px(120.0)))
            .label("5"),
    )
    .unwrap();
    let marker = find(&app, "sapid-indicator-marker");
    let tree = app.tree();
    let root = tree.node(tree.roots()[0]).unwrap().descriptor.view();
    assert_eq!(root.style.position, Some(Position::Relative));
    assert_eq!(root.style.display, Some(Display::InlineFlex));
    assert_eq!(root.style.width, Some(Length::Px(120.0)));
    let marker = tree.node(marker).unwrap().descriptor.view();
    assert!(marker.style.background.is_none());
    assert!(marker.style.radius.is_none());
    assert_eq!(marker.style.position, Some(Position::Absolute));
    let html = glory::ssr::render(indicator().name("Unread").label("<&>")).unwrap();
    assert!(html.contains("&lt;&amp;&gt;"));
    assert!(!html.contains("role=\"status\""));
    assert!(!html.contains("aria-live"));
}
