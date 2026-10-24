use std::cell::{Cell, RefCell};
use std::rc::Rc;

use glory::render::{Event, NodeId, Props, TextKind};
use glory::style::{Edges, Style};
use glory::ui::App;
use sapid::{BadgeVariant, ShadowSize, Theme, badge, button, card, empty_state, paper, pill};

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
fn badge_is_passive_reactive_text_with_independent_theme_overrides() {
    let app = App::memory();
    let content = app.scope().cage("Ready".to_string());
    let style = app.scope().cage(Style::new().radius(0.0));
    let class = app.scope().cage("application-tag".to_string());
    app.mount(
        badge()
            .content(content)
            .class(class)
            .style(style)
            .theme(Theme::green_dark()),
    )
    .unwrap();
    let id = find(&app, "sapid-badge");
    let d = app.tree().node(id).unwrap().descriptor.view().to_owned();
    assert_eq!(d.role, None);
    assert_eq!(d.style.radius, Some(0.0));
    assert_eq!(d.style.background, Some(Theme::green_dark().accent_soft));
    assert_eq!(d.name, None);
    assert!(d.classes.iter().any(|c| c == "application-tag"));
    assert!(!app.dispatch(id, Event::Activate).unwrap());
    content.revise(|v| *v = "Complete".into());
    class.revise(|v| *v = "updated".into());
    style.revise(|v| *v = Style::new().padding(Edges::all(0.0)));
    app.flush().unwrap();
    let d = app.tree().node(id).unwrap().descriptor.view().to_owned();
    assert!(matches!(d.props, Props::Text(ref text) if text.content == "Complete"));
    assert_eq!(d.style.radius, Some(100.0));
    assert_eq!(d.style.padding, Some(Edges::all(0.0)));
    assert!(!d.classes.iter().any(|c| c == "application-tag"));
    assert_eq!(find(&app, "sapid-badge"), id);
}

#[test]
fn badge_variants_and_unstyled_have_real_independent_declarations() {
    for (variant, fill, border) in [
        (BadgeVariant::Soft, Some(Theme::light().accent_soft), None),
        (BadgeVariant::Solid, Some(Theme::light().accent), None),
        (BadgeVariant::Outline, None, Some(Theme::light().accent)),
    ] {
        let app = App::memory();
        app.mount(badge().variant(variant).content("A")).unwrap();
        let d = app
            .tree()
            .node(find(&app, "sapid-badge"))
            .unwrap()
            .descriptor
            .view()
            .to_owned();
        assert_eq!(d.style.background, fill);
        assert_eq!(d.style.border_color, border);
    }
    let app = App::memory();
    app.mount(badge().unstyled().style(Style::new().radius(7.0)))
        .unwrap();
    assert_eq!(
        app.tree()
            .node(find(&app, "sapid-badge"))
            .unwrap()
            .descriptor
            .view()
            .style
            .clone(),
        Style::new().radius(7.0)
    );
}

#[test]
fn pill_removal_is_separately_named_native_button_and_hides_after_local_acceptance() {
    let app = App::memory();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let typed = calls.clone();
    let observed = calls.clone();
    app.mount(
        pill()
            .content("Rust")
            .removable("Remove Rust")
            .on_remove(move || typed.borrow_mut().push("remove"))
            .on_remove_press(move |_| observed.borrow_mut().push("observer")),
    )
    .unwrap();
    let id = find(&app, "sapid-pill");
    let remove = find(&app, "sapid-pill-remove");
    let d = app
        .tree()
        .node(remove)
        .unwrap()
        .descriptor
        .view()
        .to_owned();
    assert!(matches!(d.props, Props::Button(_)));
    assert_eq!(d.name.as_deref(), Some("Remove Rust"));
    assert!(app.dispatch(remove, Event::Activate).unwrap());
    app.flush().unwrap();
    assert_eq!(*calls.borrow(), ["remove", "observer"]);
    assert!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
    assert!(!app.dispatch(remove, Event::Activate).unwrap());
    assert_eq!(find(&app, "sapid-pill"), id);
}

#[test]
fn controlled_pill_proposes_without_mutating_source_and_external_updates_are_silent() {
    let app = App::memory();
    let visible = app.scope().cage(true);
    let enabled = app.scope().cage(false);
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    app.mount(
        pill()
            .content("A")
            .removable("Remove A")
            .visible(visible)
            .default_visible(false)
            .enabled(enabled)
            .on_remove(move || seen.set(seen.get() + 1)),
    )
    .unwrap();
    let id = find(&app, "sapid-pill");
    let remove = find(&app, "sapid-pill-remove");
    assert!(!app.dispatch(remove, Event::Activate).unwrap());
    enabled.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(app.dispatch(remove, Event::Activate).unwrap());
    app.flush().unwrap();
    assert!(visible.get());
    assert!(
        !app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
    assert_eq!(calls.get(), 1);
    visible.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
    assert_eq!(calls.get(), 1);
    visible.revise(|v| *v = true);
    app.flush().unwrap();
    assert_eq!(find(&app, "sapid-pill-remove"), remove);
    assert_eq!(calls.get(), 1);
}

#[test]
fn nonremovable_pill_has_no_action_and_empty_action_names_are_rejected_before_creation() {
    let app = App::memory();
    app.mount(pill().content("Passive")).unwrap();
    assert_eq!(
        app.tree()
            .node(find(&app, "sapid-pill"))
            .unwrap()
            .descriptor
            .view()
            .style
            .display,
        Some(glory::style::Display::InlineFlex)
    );
    assert_eq!(app.tree().len(), 2);
    let app = App::memory();
    assert!(app.mount(pill().removable(" \t ")).is_err());
    assert_eq!(app.tree().len(), 0);
}

#[test]
fn inline_pill_display_can_be_explicitly_overridden_and_unstyled_removes_it() {
    let app = App::memory();
    app.mount(
        pill()
            .content("Block")
            .style(Style::new().display(glory::style::Display::Flex)),
    )
    .unwrap();
    assert_eq!(
        app.tree()
            .node(find(&app, "sapid-pill"))
            .unwrap()
            .descriptor
            .view()
            .style
            .display,
        Some(glory::style::Display::Flex)
    );
    let app = App::memory();
    app.mount(pill().content("Unstyled").unstyled()).unwrap();
    assert_eq!(
        app.tree()
            .node(find(&app, "sapid-pill"))
            .unwrap()
            .descriptor
            .view()
            .style
            .display,
        None
    );
}

#[test]
fn card_orders_real_slots_and_uses_heading_content_without_an_implicit_control() {
    let app = App::memory();
    app.mount(
        card()
            .header(glory::ui::text().content("Header"))
            .title("Settings")
            .description("Description")
            .heading_level(2)
            .fill(glory::ui::text().content("Body"))
            .footer(button().label("Save")),
    )
    .unwrap();
    let root = find(&app, "sapid-card");
    let title = find(&app, "sapid-card-title");
    let description = find(&app, "sapid-card-description");
    let d = app.tree().node(root).unwrap().descriptor.view().to_owned();
    assert_eq!(d.role, None);
    assert_eq!(d.name, None);
    assert!(matches!(d.props, Props::View));
    assert_eq!(d.style.border_color, Some(Theme::light().border));
    assert!(
        matches!(app.tree().node(title).unwrap().descriptor.view().props.clone(), Props::Text(ref t) if t.kind == TextKind::Heading(2) && t.content == "Settings")
    );
    assert!(
        matches!(app.tree().node(description).unwrap().descriptor.view().props.clone(), Props::Text(ref t) if t.kind == TextKind::Paragraph)
    );
    assert_eq!(
        app.tree().node(root).unwrap().children,
        [
            find(&app, "sapid-card-header"),
            find(&app, "sapid-card-body"),
            find(&app, "sapid-card-footer")
        ]
    );
    assert!(!app.dispatch(root, Event::Activate).unwrap());
}

#[test]
fn paper_border_is_explicit_and_unstyled_removes_descendant_theme_declarations() {
    let app = App::memory();
    app.mount(paper().fill(glory::ui::text().content("body")))
        .unwrap();
    assert_eq!(
        app.tree()
            .node(find(&app, "sapid-paper"))
            .unwrap()
            .descriptor
            .view()
            .style
            .border_color,
        None
    );
    let app = App::memory();
    app.mount(
        paper()
            .bordered(true)
            .theme(Theme::green_dark())
            .name("Panel"),
    )
    .unwrap();
    assert_eq!(
        app.tree()
            .node(find(&app, "sapid-paper"))
            .unwrap()
            .descriptor
            .view()
            .style
            .border_color,
        Some(Theme::green_dark().border)
    );
    let app = App::memory();
    app.mount(card().unstyled().title("Title").description("Description"))
        .unwrap();
    for class in [
        "sapid-card",
        "sapid-card-title",
        "sapid-card-description",
        "sapid-card-header",
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
fn empty_state_keeps_illustration_title_description_body_and_actions_ordered() {
    let app = App::memory();
    app.mount(
        empty_state()
            .title("No messages")
            .description("Start a conversation")
            .illustration(glory::ui::text().content("Inbox"))
            .fill(glory::ui::text().content("Additional information"))
            .actions(button().label("Compose")),
    )
    .unwrap();
    let id = find(&app, "sapid-empty-state");
    let children = app.tree().node(id).unwrap().children.clone();
    assert_eq!(
        children,
        [
            find(&app, "sapid-empty-illustration"),
            find(&app, "sapid-empty-title"),
            find(&app, "sapid-empty-description"),
            find(&app, "sapid-empty-body"),
            find(&app, "sapid-empty-actions")
        ]
    );
    assert_eq!(app.tree().node(id).unwrap().descriptor.view().role, None);
    assert!(
        matches!(app.tree().node(find(&app, "sapid-empty-title")).unwrap().descriptor.view().props.clone(), Props::Text(ref t) if t.kind == TextKind::Heading(2))
    );
}

#[test]
fn heading_validation_precedes_creation() {
    for level in [0, 7, 255] {
        let app = App::memory();
        assert!(app.mount(card().heading_level(level)).is_err());
        assert_eq!(app.tree().len(), 0);
        assert!(app.mount(empty_state().heading_level(level)).is_err());
        assert_eq!(app.tree().len(), 0);
    }
}

#[test]
fn removal_captures_and_references_retire_with_component_owner() {
    struct DropProbe(Rc<Cell<usize>>);
    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let app = App::memory();
    let baseline = app.runtime_stats();
    let root_ref = app.scope().cage(None);
    let remove_ref = app.scope().cage(None);
    let dropped = Rc::new(Cell::new(0));
    let probe = DropProbe(dropped.clone());
    let mounted = app
        .scope()
        .mount_with(|| {
            pill()
                .removable("Remove tag")
                .node_ref(root_ref)
                .remove_ref(remove_ref)
                .on_remove(move || {
                    let _ = &probe;
                })
        })
        .unwrap();
    app.flush().unwrap();
    let old = remove_ref.get().unwrap();
    assert!(root_ref.get().is_some());
    drop(mounted);
    app.flush().unwrap();
    assert_eq!(root_ref.get(), None);
    assert_eq!(remove_ref.get(), None);
    assert_eq!(dropped.get(), 1);
    assert_eq!(app.stats().handlers, 0);
    assert!(!app.dispatch(old, Event::Activate).unwrap());
    assert_eq!(app.runtime_stats().edges, baseline.edges);
}

#[test]
fn display_ssr_escapes_all_slots_and_keeps_native_button_and_heading_shapes() {
    let html = glory::ssr::render(
        card()
            .title("Title <&>")
            .description("Description <&>")
            .header(badge().content("Badge <&>"))
            .fill((
                pill().content("Pill <&>").removable("Remove <&>"),
                empty_state()
                    .title("Empty <&>")
                    .actions(button().label("Create <&>")),
            ))
            .footer(paper().fill(glory::ui::text().content("Footer <&>"))),
    )
    .unwrap();
    for content in [
        "Title",
        "Description",
        "Badge",
        "Pill",
        "Remove",
        "Empty",
        "Create",
        "Footer",
    ] {
        assert!(
            html.contains(&format!("{content} &lt;&amp;&gt;")),
            "missing {content}: {html}"
        );
    }
    assert!(html.contains("<h3"));
    assert!(html.contains("<h2"));
    assert!(html.contains("aria-label=\"Remove &lt;&amp;&gt;\""));
    assert!(html.contains("type=\"button\""));
    assert!(!html.contains("role=\"alert\""));
    assert!(!html.contains("<script"));
}

#[test]
fn surface_shadow_tokens_are_opt_in_theme_aware_and_overridable() {
    use glory::style::{BoxShadow, Color, Shadow};
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        for size in [
            ShadowSize::None,
            ShadowSize::Small,
            ShadowSize::Medium,
            ShadowSize::Large,
        ] {
            let app = App::memory();
            app.mount(card().theme(theme.clone()).shadow(size)).unwrap();
            let shadow = app
                .tree()
                .node(find(&app, "sapid-card"))
                .unwrap()
                .descriptor
                .view()
                .style
                .box_shadow
                .unwrap();
            if size == ShadowSize::None {
                assert_eq!(shadow, BoxShadow::None)
            } else {
                assert!(
                    matches!(shadow,BoxShadow::Outset(Shadow{color:Color(0,0,0,alpha),..})if alpha>0)
                );
            }
        }
    }
    let app = App::memory();
    app.mount(paper()).unwrap();
    assert_eq!(
        app.tree()
            .node(find(&app, "sapid-paper"))
            .unwrap()
            .descriptor
            .view()
            .style
            .box_shadow,
        None
    );
    let app = App::memory();
    app.mount(paper().shadow(ShadowSize::Medium).unstyled())
        .unwrap();
    assert_eq!(
        app.tree()
            .node(find(&app, "sapid-paper"))
            .unwrap()
            .descriptor
            .view()
            .style
            .box_shadow,
        None
    );
    let app = App::memory();
    let custom = app.scope().cage(Style::new().box_shadow(BoxShadow::None));
    app.mount(paper().shadow(ShadowSize::Medium).style(custom))
        .unwrap();
    let id = find(&app, "sapid-paper");
    assert_eq!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .style
            .box_shadow,
        Some(BoxShadow::None)
    );
    custom.revise(|value| *value = Style::new());
    app.flush().unwrap();
    assert!(matches!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .style
            .box_shadow,
        Some(BoxShadow::Outset(_))
    ));
}
