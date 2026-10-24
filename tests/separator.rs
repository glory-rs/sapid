use glory::render::{Action, FocusMode, Orientation, Role};
use glory::style::{Length, Style};
use glory::ui::App;
use sapid::{Theme, separator};
#[test]
fn horizontal_vertical_and_decorative_separators_have_distinct_semantics_and_geometry() {
    for axis in [Orientation::Horizontal, Orientation::Vertical] {
        let app = App::memory();
        app.mount(
            separator()
                .orientation(axis)
                .length(Length::Px(100.0))
                .thickness(2.0),
        )
        .unwrap();
        let id = app.tree().roots()[0];
        let d = app.tree().node(id).unwrap().descriptor.view().to_owned();
        assert_eq!(d.role, Some(Role::Separator));
        assert_eq!(d.focus, FocusMode::Auto);
        assert_eq!(d.semantics.as_ref().unwrap().separator_axis, Some(axis));
        assert!(Action::Focus(id).validate_target(&d).is_err());
        let (w, h) = if axis == Orientation::Horizontal {
            (100.0, 2.0)
        } else {
            (2.0, 100.0)
        };
        assert_eq!(
            (d.style.width, d.style.height),
            (Some(Length::Px(w)), Some(Length::Px(h)))
        );
    }
    let app = App::memory();
    app.mount(separator().decorative(true)).unwrap();
    let d = app
        .tree()
        .node(app.tree().roots()[0])
        .unwrap()
        .descriptor
        .view()
        .to_owned();
    assert_eq!(d.role, None);
    assert_eq!(d.semantics, None);
    assert_eq!(d.name, None);
}
#[test]
fn invalid_separator_configuration_fails_before_creating_nodes() {
    for widget in [
        separator().thickness(0.0),
        separator().thickness(f64::NAN),
        separator().thickness(-1.0),
        separator().length(Length::Px(-1.0)),
        separator().decorative(true).name("Line"),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert_eq!(app.tree().len(), 0);
    }
}
#[test]
fn theme_unstyled_and_custom_declarations_preserve_independent_line_tokens() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let app = App::memory();
        app.mount(
            separator()
                .theme(theme.clone())
                .style(Style::new().radius(3.0)),
        )
        .unwrap();
        let d = app
            .tree()
            .node(app.tree().roots()[0])
            .unwrap()
            .descriptor
            .view()
            .to_owned();
        assert_eq!(d.style.background, Some(theme.border));
        assert_eq!(d.style.radius, Some(3.0));
    }
    let app = App::memory();
    app.mount(separator().unstyled().style(Style::new().radius(3.0)))
        .unwrap();
    assert_eq!(
        app.tree()
            .node(app.tree().roots()[0])
            .unwrap()
            .descriptor
            .view()
            .style
            .clone(),
        Style::new().radius(3.0)
    );
}
#[test]
fn ssr_separator_keeps_orientation_and_escaped_optional_name_without_tabstop() {
    let html = glory::ssr::render(
        separator()
            .name("Boundary <&>")
            .orientation(Orientation::Vertical),
    )
    .unwrap();
    assert!(html.contains("role=\"separator\""));
    assert!(html.contains("aria-orientation=\"vertical\""));
    assert!(html.contains("Boundary &lt;&amp;&gt;"));
    assert!(!html.contains("tabindex"));
}
