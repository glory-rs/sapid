use glory::prelude::*;
use glory::render::{NodeId, Props, TextKind};
use glory::style::{Color, Display, Length, Overflow, TextOverflow, WhiteSpace};
use sapid::{TextVariant, Theme};
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            return id;
        }
        pending.extend(node.children.iter().copied());
    }
    panic!("missing {class}")
}
#[test]
fn text_defaults_to_paragraph_and_inline_is_semantically_distinct() {
    let app = App::memory();
    app.mount(col().fill((
        sapid::text().class("paragraph").content("Body"),
        sapid::text().class("inline").inline(true).content("Inline"),
    )))
    .unwrap();
    assert!(
        matches!(&app.tree().node(find(&app,"paragraph")).unwrap().descriptor.view().props.clone(),Props::Text(p)if p.kind==TextKind::Paragraph)
    );
    assert!(
        matches!(&app.tree().node(find(&app,"inline")).unwrap().descriptor.view().props.clone(),Props::Text(p)if p.kind==TextKind::Inline)
    );
}
#[test]
fn title_levels_are_real_headings_and_invalid_levels_fail() {
    for level in 1..=6 {
        let html = glory::ssr::render(sapid::title(level).content("Heading")).unwrap();
        assert!(html.contains(&format!("<h{level}")));
    }
    for level in [0, 7, u8::MAX] {
        assert!(
            App::memory()
                .mount(sapid::title(level).content("Invalid"))
                .is_err()
        );
    }
}
#[test]
fn themed_text_variants_have_distinct_metrics_and_preserve_custom_declarations() {
    let app = App::memory();
    app.mount(
        col().fill((
            sapid::text()
                .class("body")
                .theme(Theme::green_dark())
                .content("Body"),
            sapid::text()
                .class("lead")
                .variant(TextVariant::Lead)
                .content("Lead"),
            sapid::text()
                .class("muted")
                .theme(Theme::green_dark())
                .variant(TextVariant::Muted)
                .content("Muted"),
            sapid::text()
                .class("caption")
                .variant(TextVariant::Caption)
                .font_size(13.0)
                .line_height(19.0)
                .weight(600.0)
                .color(Color(10, 20, 30, 255))
                .style(Style::new().width(Length::Px(80.0)))
                .content("Caption"),
        )),
    )
    .unwrap();
    let tree = app.tree();
    let body = &tree
        .node(find(&app, "body"))
        .unwrap()
        .descriptor
        .view()
        .style;
    let lead = &tree
        .node(find(&app, "lead"))
        .unwrap()
        .descriptor
        .view()
        .style;
    assert_eq!(body.color, Some(Theme::green_dark().foreground));
    assert!(lead.font_size > body.font_size);
    assert_eq!(
        tree.node(find(&app, "muted"))
            .unwrap()
            .descriptor
            .view()
            .style
            .color,
        Some(Theme::green_dark().muted)
    );
    let style = &tree
        .node(find(&app, "caption"))
        .unwrap()
        .descriptor
        .view()
        .style;
    assert_eq!(style.font_size, Some(13.0));
    assert_eq!(style.line_height, Some(19.0));
    assert_eq!(style.font_weight, Some(600.0));
    assert_eq!(style.color, Some(Color(10, 20, 30, 255)));
    assert_eq!(style.width, Some(Length::Px(80.0)));
}
#[test]
fn truncation_is_typed_keeps_full_text_and_false_omits_native_unsupported_property() {
    let app = App::memory();
    app.mount(
        col().fill((
            sapid::text()
                .class("truncated")
                .truncate(true)
                .content("Full accessible content"),
            sapid::text()
                .class("plain")
                .truncate(false)
                .content("Plain"),
        )),
    )
    .unwrap();
    let tree = app.tree();
    let node = tree
        .node(find(&app, "truncated"))
        .unwrap()
        .descriptor
        .view();
    assert_eq!(node.style.text_overflow, Some(TextOverflow::Ellipsis));
    assert_eq!(node.style.overflow_x, Some(Overflow::Hidden));
    assert_eq!(node.style.white_space, Some(WhiteSpace::NoWrap));
    assert_eq!(node.style.display, Some(Display::Block));
    assert!(matches!(&node.props,Props::Text(p)if p.content=="Full accessible content"));
    assert_eq!(
        tree.node(find(&app, "plain"))
            .unwrap()
            .descriptor
            .view()
            .style
            .text_overflow,
        None
    );
}
#[test]
fn unstyled_removes_theme_but_keeps_explicit_text_and_truncation_declarations() {
    let app = App::memory();
    app.mount(
        sapid::text()
            .class("text")
            .unstyled()
            .font_size(18.0)
            .truncate(true)
            .style(Style {
                line_height: Some(24.0),
                ..Style::new()
            })
            .content("Text"),
    )
    .unwrap();
    let tree = app.tree();
    let style = &tree
        .node(find(&app, "text"))
        .unwrap()
        .descriptor
        .view()
        .style;
    assert_eq!(style.color, None);
    assert_eq!(style.font_size, Some(18.0));
    assert_eq!(style.line_height, Some(24.0));
    assert_eq!(style.text_overflow, Some(TextOverflow::Ellipsis));
}
#[test]
fn reactive_content_updates_without_replacing_text_identity_or_heading_name() {
    let app = App::memory();
    let value = app.scope().cage("Before".to_owned());
    app.mount(
        sapid::title(2)
            .class("title")
            .content(value)
            .name("Stable section"),
    )
    .unwrap();
    let id = find(&app, "title");
    value.revise(|v| *v = "After".into());
    app.flush().unwrap();
    assert_eq!(find(&app, "title"), id);
    let tree = app.tree();
    let d = tree.node(id).unwrap().descriptor.view();
    assert_eq!(d.name.as_deref(), Some("Stable section"));
    assert!(matches!(&d.props,Props::Text(p)if p.content=="After"));
}
#[test]
fn typography_context_explicitly_inherits_theme_size_and_unstyled() {
    let app = App::memory();
    app.mount(
        sapid::typography()
            .theme(Theme::green_dark())
            .size(sapid::Size::Large)
            .content(|context| {
                (
                    context.title(2).class("title").content("Title"),
                    context.text().class("text").content("Body"),
                    context.anchor().class("anchor").href("/docs").label("Docs"),
                )
            }),
    )
    .unwrap();
    for class in ["title", "text", "anchor"] {
        let d = app
            .tree()
            .node(find(&app, class))
            .unwrap()
            .descriptor
            .view()
            .to_owned();
        assert!(d.classes.iter().any(|c| c == "sapid-theme-green-dark"));
    }
    let app = App::memory();
    app.mount(
        sapid::typography()
            .unstyled()
            .content(|context| context.text().class("plain").content("Plain")),
    )
    .unwrap();
    assert_eq!(
        app.tree()
            .node(find(&app, "plain"))
            .unwrap()
            .descriptor
            .view()
            .style
            .color,
        None
    );
}
#[test]
fn invalid_typographic_numbers_are_rejected_in_shared_style_validation() {
    for text in [
        sapid::text().font_size(f64::NAN),
        sapid::text().line_height(-1.0),
        sapid::text().weight(1001.0),
    ] {
        assert!(App::memory().mount(text).is_err());
    }
}
#[test]
fn typography_ssr_escapes_content_and_preserves_rich_semantics() {
    let html = glory::ssr::render(sapid::typography().content(|context| {
        (
            context.title(1).content("A <title>"),
            context.text().content("Body & detail"),
            context.anchor().label("Read <more>").href("/docs?a=1&b=2"),
        )
    }))
    .unwrap();
    assert!(html.contains("<h1"));
    assert!(html.contains("<p"));
    assert!(html.contains("<a"));
    assert!(html.contains("A &lt;title&gt;"));
    assert!(html.contains("Body &amp; detail"));
    assert!(html.contains("Read &lt;more&gt;"));
}
