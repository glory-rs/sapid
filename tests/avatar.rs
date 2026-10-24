use glory::render::{NodeId, Props, Role};
use glory::style::{Display, Edges, Length, Style};
use glory::ui::App;
use sapid::{AvatarShape, Size, Theme, avatar, avatar_group};
use std::cell::Cell;
use std::rc::Rc;

fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut nodes = tree.roots().to_vec();
    while let Some(id) = nodes.pop() {
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
        nodes.extend(&node.children);
    }
    panic!("missing {class}");
}
fn fallback(app: &App, root: NodeId) -> String {
    let tree = app.tree();
    let child = tree.node(root).unwrap().children[0];
    let Props::Text(text) = &tree.node(child).unwrap().descriptor.view().props else {
        panic!("fallback is text")
    };
    text.content.clone()
}
#[test]
fn explicit_text_avatar_has_name_content_and_no_forged_image_role() {
    let app = App::memory();
    app.mount(avatar("Ada Lovelace").fallback("AL")).unwrap();
    let root = app.tree().roots()[0];
    let tree = app.tree();
    let descriptor = tree.node(root).unwrap().descriptor.view();
    assert_eq!(descriptor.name.as_deref(), Some("Ada Lovelace"));
    assert_eq!(descriptor.effective_role(), Some(Role::Group));
    assert_eq!(descriptor.style.display, Some(Display::InlineFlex));
    assert_eq!(descriptor.style.width, Some(Length::Px(36.0)));
    assert_eq!(descriptor.style.radius, Some(18.0));
    drop(tree);
    assert_eq!(fallback(&app, root), "AL");
    assert_eq!(app.stats().handlers, 0);
    let html = glory::ssr::render(avatar("Ada").fallback("<&>")).unwrap();
    assert!(html.contains("&lt;&amp;&gt;"));
    assert!(!html.contains("<img"));
}
#[test]
fn reactive_sources_reject_invalid_values_preserve_identity_and_recover_without_writes() {
    let app = App::memory();
    let name = app.scope().cage("Ada".to_string());
    let text = app.scope().cage("AL".to_string());
    app.mount(avatar(name).fallback(text)).unwrap();
    let root = app.tree().roots()[0];
    let children = app.tree().node(root).unwrap().children.clone();
    name.revise(|value| *value = " ".into());
    text.revise(|value| *value = "x".repeat(65));
    assert!(app.flush().is_err());
    assert_eq!(name.get(), " ");
    assert_eq!(text.get().len(), 65);
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Ada")
    );
    assert_eq!(fallback(&app, root), "AL");
    name.revise(|value| *value = "Grace".into());
    text.revise(|value| *value = "GH".into());
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Grace")
    );
    assert_eq!(fallback(&app, root), "GH");
    assert_eq!(app.tree().node(root).unwrap().children, children);
}
#[test]
fn all_themes_shapes_sizes_and_custom_overrides_are_effective() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let app = App::memory();
        app.mount(
            avatar("Ada")
                .theme(theme.clone())
                .shape(AvatarShape::Square)
                .size(Size::Large),
        )
        .unwrap();
        let tree = app.tree();
        let style = &tree.node(tree.roots()[0]).unwrap().descriptor.view().style;
        assert_eq!(style.background, Some(theme.accent_soft));
        assert_eq!(style.color, Some(theme.accent_text));
        assert_eq!(style.radius, Some(theme.radius));
        assert_eq!(style.width, Some(Length::Px(42.0)));
    }
    let app = App::memory();
    app.mount(
        avatar("Ada")
            .unstyled()
            .style(Style::new().width(Length::Px(50.0))),
    )
    .unwrap();
    assert_eq!(
        app.tree()
            .node(app.tree().roots()[0])
            .unwrap()
            .descriptor
            .view()
            .style
            .clone(),
        Style::new().width(Length::Px(50.0))
    );
}
#[test]
fn groups_bound_visible_members_and_name_the_overflow_in_source_order() {
    let app = App::memory();
    app.mount(
        avatar_group("team")
            .name("Team")
            .max_visible(2)
            .overlap(10.0)
            .avatars((0..5).map(|i| {
                (
                    format!("i{i}"),
                    avatar(format!("Person {i}")).fallback(format!("{i}")),
                )
            })),
    )
    .unwrap();
    let tree = app.tree();
    let root = tree.roots()[0];
    let children = tree.node(root).unwrap().children.clone();
    assert_eq!(children.len(), 3);
    for (index, name) in ["Person 0", "Person 1", "3 more people"].iter().enumerate() {
        let child = tree.node(children[index]).unwrap();
        assert_eq!(child.descriptor.view().name.as_deref(), Some(*name));
        assert_eq!(
            child.descriptor.view().style.margin,
            Some(Edges {
                left: if index == 0 { 0.0 } else { -10.0 },
                ..Edges::all(0.0)
            })
        );
        assert!(
            child
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .id
                .is_some()
        );
    }
    drop(tree);
    assert_eq!(fallback(&app, children[2]), "+3");
}
#[test]
fn invalid_configuration_is_rejected_before_visible_creation() {
    for widget in [
        avatar_group("bad key"),
        avatar_group("team").max_visible(0),
        avatar_group("team").overlap(f64::NAN),
        avatar_group("team").overlap(25.0),
        avatar_group("team")
            .avatar("a", avatar("Ada"))
            .avatar("a", avatar("Grace")),
        avatar_group("team").avatar("a", avatar(" ")),
        avatar_group("team")
            .max_visible(1)
            .overflow_label(|_| "".into())
            .avatar("a", avatar("Ada"))
            .avatar("b", avatar("Grace")),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert_eq!(app.tree().len(), 0);
    }
    for widget in [
        avatar(" "),
        avatar("Ada").fallback(""),
        avatar("x".repeat(257)),
        avatar("Ada").fallback("x".repeat(65)),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert_eq!(app.tree().len(), 0);
    }
}
#[test]
fn group_budget_consumes_only_limit_plus_sentinel_and_shared_setters_do_not_grow() {
    let consumed = Rc::new(Cell::new(0));
    let counter = consumed.clone();
    let mut widget = avatar_group("bounded").avatars((0..1_000_000).map(move |i| {
        counter.set(counter.get() + 1);
        (format!("a{i}"), avatar("Person"))
    }));
    for i in 0..4096 {
        widget = widget.avatar(format!("extra{i}"), avatar("Extra"));
    }
    let app = App::memory();
    assert!(app.mount(widget).is_err());
    assert_eq!(consumed.get(), 65);
    assert_eq!(app.tree().len(), 0);
}
#[test]
fn group_inheritance_preserves_explicit_child_tokens_and_unstyled() {
    let app = App::memory();
    app.mount(
        avatar_group("theme")
            .theme(Theme::green_dark())
            .size(Size::Large)
            .avatar("a", avatar("Ada").class("inherited"))
            .avatar(
                "b",
                avatar("Grace")
                    .class("explicit")
                    .theme(Theme::blue_light())
                    .size(Size::Small),
            ),
    )
    .unwrap();
    for (class, width, color) in [
        ("inherited", 42.0, Theme::green_dark().accent_soft),
        ("explicit", 30.0, Theme::blue_light().accent_soft),
    ] {
        let tree = app.tree();
        let style = &tree
            .node(find(&app, class))
            .unwrap()
            .descriptor
            .view()
            .style;
        assert_eq!(style.width, Some(Length::Px(width)));
        assert_eq!(style.background, Some(color));
    }
    let app = App::memory();
    app.mount(avatar_group("plain").unstyled().avatar("a", avatar("Ada")))
        .unwrap();
    assert_eq!(
        app.tree()
            .node(find(&app, "sapid-avatar"))
            .unwrap()
            .descriptor
            .view()
            .style
            .clone(),
        Style::new()
    );
}
#[test]
fn avatar_effects_and_refs_retire_with_the_owner() {
    let app = App::memory();
    let source = app.scope().cage("AL".to_string());
    let reference = app.scope().cage(None);
    let baseline = app.runtime_stats();
    let mounted = app
        .scope()
        .mount_with(|| avatar("Ada").fallback(source).node_ref(reference))
        .unwrap();
    app.flush().unwrap();
    assert!(reference.get().is_some());
    drop(mounted);
    app.flush().unwrap();
    assert_eq!(reference.get(), None);
    assert_eq!(app.runtime_stats().edges, baseline.edges);
    source.revise(|value| *value = "GH".into());
    app.flush().unwrap();
    assert_eq!(app.stats().nodes, 0);
}

#[test]
fn real_avatar_image_load_failure_source_reset_and_invalid_recovery_keep_named_wrapper() {
    use glory::render::{Event, ImageStatus};
    let app = App::memory();
    let source = app.scope().cage("a.png".to_string());
    app.mount(avatar("Ada").fallback("AL").source(source))
        .unwrap();
    let root = app.tree().roots()[0];
    let media = find(&app, "sapid-avatar-image");
    let text = find(&app, "sapid-avatar-fallback");
    let Props::Image(image) = app
        .tree()
        .node(media)
        .unwrap()
        .descriptor
        .view()
        .props
        .clone()
    else {
        panic!()
    };
    assert_eq!(image.alternative, None);
    app.dispatch(
        media,
        Event::ImageStatus {
            revision: 1,
            status: ImageStatus::Loaded,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(text)
            .unwrap()
            .descriptor
            .view()
            .style
            .display,
        Some(Display::None)
    );
    assert_eq!(
        app.tree()
            .node(media)
            .unwrap()
            .descriptor
            .view()
            .style
            .display,
        Some(Display::Block)
    );
    source.revise(|value| *value = " ".into());
    assert!(app.flush().is_err());
    assert_eq!(
        app.tree()
            .node(media)
            .unwrap()
            .descriptor
            .view()
            .style
            .display,
        Some(Display::Block)
    );
    source.revise(|value| *value = "b.png".into());
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(text)
            .unwrap()
            .descriptor
            .view()
            .style
            .display,
        Some(Display::Block)
    );
    app.dispatch(
        media,
        Event::ImageStatus {
            revision: 2,
            status: ImageStatus::Failed,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(text)
            .unwrap()
            .descriptor
            .view()
            .style
            .display,
        Some(Display::Block)
    );
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Ada")
    );
    assert_eq!(find(&app, "sapid-avatar-image"), media);
}
