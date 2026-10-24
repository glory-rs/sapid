use glory::render::{GraphicProps, Props};
use glory::style::{Color, Length, Style};
use glory::ui::App;
use sapid::{Icon, Theme, ThemeIconVariant, theme_icon};
fn drawing(app: &App) -> GraphicProps {
    let tree = app.tree();
    let root = tree.node(tree.roots()[0]).unwrap();
    let Props::Graphic(graphic) = tree.node(root.children[0]).unwrap().descriptor.view().props
    else {
        panic!()
    };
    graphic.clone()
}
#[test]
fn themed_icons_use_real_finite_paths_and_optional_image_alternatives() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        for icon in [Icon::Check, Icon::Close, Icon::Info, Icon::ArrowRight] {
            for variant in [
                ThemeIconVariant::Soft,
                ThemeIconVariant::Solid,
                ThemeIconVariant::Outline,
            ] {
                let app = App::memory();
                app.mount(
                    theme_icon(icon)
                        .theme(theme.clone())
                        .variant(variant)
                        .name("Icon meaning"),
                )
                .unwrap();
                let value = drawing(&app);
                value.validate().unwrap();
                assert!(!value.commands.is_empty());
                assert_eq!(value.alternative.as_deref(), Some("Icon meaning"));
                let tree = app.tree();
                let root = tree.node(tree.roots()[0]).unwrap().descriptor.view();
                assert!(root.name.is_none());
                assert!(root.effective_role().is_none());
                assert_eq!(root.style.width, Some(Length::Px(36.0)));
                assert_eq!(
                    value.stroke.unwrap().color,
                    if variant == ThemeIconVariant::Solid {
                        theme.on_accent
                    } else {
                        theme.accent_text
                    }
                );
            }
        }
    }
    let html = glory::ssr::render(theme_icon(Icon::Check)).unwrap();
    assert!(html.contains("<svg"));
    assert!(html.contains("<path"));
    assert!(html.contains("aria-hidden=\"true\""));
    assert!(!html.contains("role=\"img\""));
}
#[test]
fn names_icons_and_custom_color_patch_without_recreation_or_source_writes() {
    let app = App::memory();
    let name = app.scope().cage("Confirmed".to_string());
    let icon = app.scope().cage(Icon::Check);
    app.mount(theme_icon(icon).name(name).color(Color(1, 2, 3, 255)))
        .unwrap();
    let root = app.tree().roots()[0];
    let child = app.tree().node(root).unwrap().children[0];
    let original = drawing(&app).commands;
    name.revise(|value| *value = " ".into());
    assert!(app.flush().is_err());
    assert_eq!(name.get(), " ");
    assert_eq!(drawing(&app).alternative.as_deref(), Some("Confirmed"));
    name.revise(|value| *value = "Closed".into());
    icon.revise(|value| *value = Icon::Close);
    app.flush().unwrap();
    assert_eq!(drawing(&app).alternative.as_deref(), Some("Closed"));
    assert_ne!(drawing(&app).commands, original);
    assert_eq!(drawing(&app).stroke.unwrap().color, Color(1, 2, 3, 255));
    assert_eq!(app.tree().node(root).unwrap().children[0], child);
    assert_eq!(app.stats().handlers, 0);
}
#[test]
fn invalid_names_reject_before_creation_and_unstyled_keeps_content_paths() {
    for name in ["".to_string(), " ".to_string(), "x".repeat(257)] {
        let app = App::memory();
        assert!(app.mount(theme_icon(Icon::Check).name(name)).is_err());
        assert!(app.tree().roots().is_empty());
    }
    let app = App::memory();
    app.mount(
        theme_icon(Icon::Info)
            .unstyled()
            .style(Style::new().width(Length::Px(64.0))),
    )
    .unwrap();
    assert!(!drawing(&app).commands.is_empty());
    let tree = app.tree();
    let style = tree.node(tree.roots()[0]).unwrap().descriptor.view().style;
    assert!(style.background.is_none());
    assert!(style.border_width.is_none());
    assert_eq!(style.width, Some(Length::Px(64.0)));
}
