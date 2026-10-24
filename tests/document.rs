use glory::render::{DocumentPart, NodeId, Props, TextKind};
use glory::style::{Family, Style};
use glory::ui::App;
use sapid::{
    KbdKey, KbdPlatform, Theme, blockquote, code, data_list, data_list_item, highlight, item,
    kbd, mark, text_list, text_list_item,
};
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let n = tree.node(id).unwrap();
        if n.descriptor.view().classes.iter().any(|c| c == class) {
            return id;
        }
        pending.extend(n.children.iter().copied());
    }
    panic!("missing {class}")
}
fn content(app: &App, id: NodeId) -> String {
    let Props::Text(text) = app.tree().node(id).unwrap().descriptor.view().props.clone() else {
        panic!("expected text")
    };
    text.content
}

#[test]
fn inline_widgets_have_real_text_kinds_and_reactive_content_preserves_identity() {
    let app = App::memory();
    let value = app.scope().cage("a < b".to_string());
    app.mount(glory::ui::row().fill((
        code().content(value),
        kbd().content("Ctrl"),
        mark().content("marked"),
    )))
    .unwrap();
    for (class, kind) in [
        ("sapid-code", TextKind::Code),
        ("sapid-kbd", TextKind::Kbd),
        ("sapid-mark", TextKind::Mark),
    ] {
        let id = find(&app, class);
        let d = app.tree().node(id).unwrap().descriptor.view().to_owned();
        assert!(matches!(d.props,Props::Text(ref t)if t.kind==kind));
        assert_eq!(d.name, None);
        assert_eq!(d.role, None);
    }
    let id = find(&app, "sapid-code");
    value.revise(|v| *v = "updated 😀".into());
    app.flush().unwrap();
    assert_eq!(content(&app, id), "updated 😀");
    assert_eq!(find(&app, "sapid-code"), id);
}
#[test]
fn platform_key_names_are_explicit_and_register_no_shortcut() {
    for (platform, key, expected) in [
        (KbdPlatform::Mac, KbdKey::Meta, "Command"),
        (KbdPlatform::Windows, KbdKey::Meta, "Windows"),
        (KbdPlatform::Linux, KbdKey::Meta, "Super"),
        (KbdPlatform::Mac, KbdKey::Alt, "Option"),
        (KbdPlatform::Windows, KbdKey::Control, "Ctrl"),
        (KbdPlatform::Mac, KbdKey::Enter, "Return"),
    ] {
        let app = App::memory();
        app.mount(kbd().platform(platform).key(key)).unwrap();
        assert_eq!(content(&app, find(&app, "sapid-kbd")), expected);
        assert_eq!(app.stats().handlers, 0);
    }
    let app = App::memory();
    app.mount(kbd().key(KbdKey::Meta).content("Custom"))
        .unwrap();
    assert_eq!(content(&app, find(&app, "sapid-kbd")), "Custom");
}
#[test]
fn code_kbd_and_mark_theme_tokens_unstyled_and_overrides_remain_independent() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let app = App::memory();
        app.mount(glory::ui::row().fill((
            code().theme(theme.clone()).content("code"),
            kbd().theme(theme.clone()).content("key"),
            mark().theme(theme.clone()).content("mark"),
        )))
        .unwrap();
        for class in ["sapid-code", "sapid-kbd"] {
            assert_eq!(
                app.tree()
                    .node(find(&app, class))
                    .unwrap()
                    .descriptor
                    .view()
                    .style
                    .font_family,
                Some(vec![Family::Mono])
            );
        }
        assert_eq!(
            app.tree()
                .node(find(&app, "sapid-mark"))
                .unwrap()
                .descriptor
                .view()
                .style
                .background,
            Some(theme.accent_soft)
        );
    }
    let app = App::memory();
    app.mount(code().unstyled().style(Style::new().radius(7.0)))
        .unwrap();
    assert_eq!(
        app.tree()
            .node(find(&app, "sapid-code"))
            .unwrap()
            .descriptor
            .view()
            .style
            .clone(),
        Style::new().radius(7.0)
    );
}
#[test]
fn text_lists_keep_real_ordered_and_unordered_structure_and_nesting() {
    let html = glory::ssr::render(
        text_list()
            .ordered(true)
            .item(text_list_item().fill((
                glory::ui::text().content("First"),
                text_list().item(text_list_item().fill(glory::ui::text().content("Nested"))),
            )))
            .item(text_list_item().fill(code().content("Second"))),
    )
    .unwrap();
    assert!(html.contains("<ol"));
    assert!(html.contains("<ul"));
    assert_eq!(html.matches("<li ").count(), 3);
    assert!(html.contains("<code"));
    assert!(!html.contains("role=\"list\""));
    let app = App::memory();
    assert!(
        app.mount(text_list_item().fill(glory::ui::text().content("orphan")))
            .is_err()
    );
    assert_eq!(app.tree().len(), 0);
}
#[test]
fn description_list_pairs_use_dt_dd_and_share_parent_theme_and_unstyled() {
    let app = App::memory();
    app.mount(
        data_list().theme(Theme::green_dark()).item(
            data_list_item()
                .term("Name")
                .fill(glory::ui::text().content("Sapid")),
        ),
    )
    .unwrap();
    for (class, part) in [
        ("sapid-data-list", DocumentPart::DescriptionList),
        ("sapid-data-list-term", DocumentPart::Term),
        ("sapid-data-list-definition", DocumentPart::Definition),
    ] {
        let d = app
            .tree()
            .node(find(&app, class))
            .unwrap()
            .descriptor
            .view()
            .to_owned();
        assert_eq!(d.document_part(), Some(part));
        assert_eq!(d.effective_role(), None);
    }
    assert_eq!(
        app.tree()
            .node(find(&app, "sapid-data-list-term"))
            .unwrap()
            .descriptor
            .view()
            .style
            .color,
        Some(Theme::green_dark().foreground)
    );
    let app = App::memory();
    app.mount(
        data_list().unstyled().item(
            data_list_item()
                .term("Name")
                .fill(glory::ui::text().content("Value")),
        ),
    )
    .unwrap();
    for class in [
        "sapid-data-list",
        "sapid-data-list-term",
        "sapid-data-list-definition",
    ] {
        assert_eq!(
            app.tree()
                .node(find(&app, class))
                .unwrap()
                .descriptor
                .view()
                .style
                .clone(),
            Style::new()
        );
    }
}
#[test]
fn description_values_keep_interactive_controls_separate_from_terms() {
    let html = glory::ssr::render(
        data_list().name("Account metadata").item(
            data_list_item().term("Website").fill(
                glory::ui::link()
                    .target("https://example.com")
                    .label("Open website"),
            ),
        ),
    )
    .unwrap();
    assert!(html.contains("<dl"));
    assert!(html.contains("<dt"));
    assert!(html.contains("<dd"));
    assert!(html.contains("<a "));
    assert!(!html.contains("role=\"group\""));
}
#[test]
fn blockquote_is_intrinsic_with_separate_source_and_citation_slots() {
    let html = glory::ssr::render(
        blockquote()
            .fill(glory::ui::text().content("A quote <&>"))
            .citation(
                glory::ui::link()
                    .label("Source")
                    .target("https://example.com"),
            ),
    )
    .unwrap();
    assert!(html.starts_with("<blockquote"));
    assert!(html.contains("A quote &lt;&amp;&gt;"));
    assert!(html.contains("sapid-blockquote-citation"));
    assert!(html.contains("<a "));
}
#[test]
fn inline_document_text_is_valid_in_native_button_content() {
    let html = glory::ssr::render(sapid::button().label("Run").fill((
        code().content("cmd"),
        kbd().content("Enter"),
        mark().content("Now"),
    )))
    .unwrap();
    assert!(html.contains("<button"));
    assert!(html.contains("<code"));
    assert!(html.contains("<kbd"));
    assert!(html.contains("<mark"));
}
#[test]
fn document_references_and_bindings_retire_without_owner_or_edge_leaks() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    let term = app.scope().cage("Term".to_string());
    let before = app.runtime_stats();
    let mounted = app
        .scope()
        .mount_with(|| {
            data_list().item(
                data_list_item()
                    .term(term)
                    .node_ref(reference)
                    .fill(code().content("Value")),
            )
        })
        .unwrap();
    app.flush().unwrap();
    assert!(reference.get().is_some());
    drop(mounted);
    app.flush().unwrap();
    assert_eq!(reference.get(), None);
    assert_eq!(app.runtime_stats().edges, before.edges);
    assert_eq!(app.stats().nodes, 0);
}
#[test]
fn all_document_content_escapes_without_changing_business_text() {
    let app = App::memory();
    let source = app.scope().cage("<&>😀".to_string());
    app.mount(mark().content(source)).unwrap();
    assert_eq!(source.get(), "<&>😀");
    let html = glory::ssr::render(
        glory::ui::col().fill((
            code().content("<&>"),
            kbd().content("<&>"),
            mark().content("<&>"),
            data_list().item(
                data_list_item()
                    .term("<&>")
                    .fill(glory::ui::text().content("<&>")),
            ),
        )),
    )
    .unwrap();
    assert_eq!(html.matches("&lt;&amp;&gt;").count(), 5);
    assert!(!html.contains("<script"));
}

#[test]
fn highlight_exact_matches_keep_canonical_text_order_and_longest_overlap() {
    let app = App::memory();
    app.mount(highlight().content("a😀<&>ababa\r\nb\0").queries(vec![
        "a".into(),
        "aba".into(),
        "😀".into(),
    ]))
    .unwrap();
    let tree = app.tree();
    let root = tree.roots()[0];
    let mut combined = String::new();
    let mut marked = String::new();
    for id in &tree.node(root).unwrap().children {
        let Props::Text(text) = &tree.node(*id).unwrap().descriptor.view().props else {
            panic!("text expected")
        };
        combined.push_str(&text.content);
        if text.kind == TextKind::Mark {
            marked.push_str(&text.content);
        }
    }
    assert_eq!(combined, "a😀<&>ababa\nb�");
    assert_eq!(marked, "a😀abaa");
    let html = glory::ssr::render(
        highlight()
            .content("abab")
            .queries(vec!["a".into(), "ab".into()]),
    )
    .unwrap();
    assert_eq!(html.matches("<mark ").count(), 1);
    assert!(html.contains(">abab</mark>"));
    assert!(!html.contains("<div"));
}

#[test]
fn highlight_budgets_reject_before_creating_any_nodes() {
    for value in [
        highlight().content("x".repeat(65537)),
        highlight().query(""),
        highlight().query("x".repeat(257)),
        highlight().queries(vec!["x".into(); 33]),
        highlight().content("ab".repeat(1025)).query("a"),
    ] {
        let app = App::memory();
        assert!(app.mount(value).is_err());
        assert_eq!(app.tree().len(), 0);
    }
    let app = App::memory();
    app.mount(highlight().content("a".repeat(65536)).query("a"))
        .unwrap();
    assert_eq!(app.tree().len(), 2);
}

#[test]
fn highlight_is_inline_button_content_and_never_writes_or_registers_handlers() {
    let html = glory::ssr::render(
        sapid::button()
            .label("Find")
            .fill(highlight().content("Find <&> text").query("<&>")),
    )
    .unwrap();
    assert!(html.contains("<mark"));
    assert!(html.contains("&lt;&amp;&gt;"));
    let app = App::memory();
    app.mount(highlight().content("text").query("missing"))
        .unwrap();
    assert_eq!(app.stats().handlers, 0);
    assert_eq!(app.tree().len(), 2);
}

#[test]
fn item_regions_are_passive_and_actions_keep_intrinsic_names_and_roles() {
    let app = App::memory();
    app.mount(
        item()
            .title("Workspace")
            .description("Settings")
            .media(glory::ui::text().content("Icon"))
            .fill(code().content("glory"))
            .actions(sapid::button().label("Open")),
    )
    .unwrap();
    let id = find(&app, "sapid-item");
    assert_eq!(app.tree().node(id).unwrap().descriptor.view().role, None);
    assert_eq!(
        app.tree().node(id).unwrap().descriptor.view().name.clone(),
        None
    );
    assert_eq!(
        app.tree().node(id).unwrap().children,
        [
            find(&app, "sapid-item-media"),
            find(&app, "sapid-item-content"),
            find(&app, "sapid-item-actions")
        ]
    );
    let button = find(&app, "sapid-button");
    assert!(
        matches!(app.tree().node(button).unwrap().descriptor.view().props.clone(),Props::Button(ref p)if p.label=="Open")
    );
}

#[test]
fn item_and_highlight_unstyled_remove_their_own_theme_declarations() {
    let app = App::memory();
    app.mount(
        item()
            .unstyled()
            .title("Title")
            .description("Description")
            .fill(highlight().unstyled().content("word").query("word")),
    )
    .unwrap();
    for class in [
        "sapid-item",
        "sapid-item-content",
        "sapid-item-title",
        "sapid-item-description",
        "sapid-highlight-match",
    ] {
        assert_eq!(
            app.tree()
                .node(find(&app, class))
                .unwrap()
                .descriptor
                .view()
                .style
                .clone(),
            Style::new()
        );
    }
}
