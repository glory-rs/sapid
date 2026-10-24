use glory::render::{DocumentPart, NodeId, Props};
use glory::style::Style;
use glory::ui::App;
use sapid::{MAX_TIMELINE_ITEMS, Theme, TimelineProgress, timeline, timeline_item};
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
fn states(app: &App) -> Vec<String> {
    let tree = app.tree();
    let root = tree.roots()[0];
    tree.node(root)
        .unwrap()
        .children
        .iter()
        .map(|id| {
            let children = &tree.node(*id).unwrap().children;
            let content = children[1];
            let state = tree.node(content).unwrap().children[1];
            let Props::Text(text) = tree.node(state).unwrap().descriptor.view().props else {
                panic!("state text")
            };
            text.content.clone()
        })
        .collect()
}
fn model() -> sapid::Timeline {
    timeline("release")
        .item(timeline_item("design").title("Design"))
        .item(timeline_item("build").title("Build"))
        .item(timeline_item("ship").title("Ship"))
}
#[test]
fn ordered_timeline_uses_real_list_items_in_source_order_and_readable_progress() {
    let app = App::memory();
    app.mount(model().active("build")).unwrap();
    let tree = app.tree();
    let root = tree.roots()[0];
    assert_eq!(
        tree.node(root).unwrap().descriptor.view().document_part(),
        Some(DocumentPart::OrderedList)
    );
    for child in &tree.node(root).unwrap().children {
        assert_eq!(
            tree.node(*child).unwrap().descriptor.view().document_part(),
            Some(DocumentPart::ListItem)
        );
        assert_eq!(tree.node(*child).unwrap().descriptor.view().role, None);
    }
    drop(tree);
    assert_eq!(states(&app), ["Completed", "Current", "Upcoming"]);
    assert_eq!(app.stats().handlers, 0);
}
#[test]
fn pending_complete_and_external_current_changes_are_silent_and_preserve_identity() {
    let app = App::memory();
    let progress = app.scope().cage(TimelineProgress::Pending);
    app.mount(model().progress(progress)).unwrap();
    let root = app.tree().roots()[0];
    let ids = app.tree().node(root).unwrap().children.clone();
    assert_eq!(states(&app), ["Upcoming", "Upcoming", "Upcoming"]);
    progress.revise(|value| *value = TimelineProgress::Active("ship".into()));
    app.flush().unwrap();
    assert_eq!(states(&app), ["Completed", "Completed", "Current"]);
    progress.revise(|value| *value = TimelineProgress::Complete);
    app.flush().unwrap();
    assert_eq!(states(&app), ["Completed", "Completed", "Completed"]);
    assert_eq!(app.tree().node(root).unwrap().children, ids);
    assert_eq!(app.stats().handlers, 0);
}

#[test]
fn current_status_has_a_real_accent_mark_and_readable_text_alternative() {
    let app = App::memory();
    app.mount(model().active("build").theme(Theme::green_dark()))
        .unwrap();
    let tree = app.tree();
    let list = tree.roots()[0];
    for (index, item) in tree.node(list).unwrap().children.iter().enumerate() {
        let content = tree.node(*item).unwrap().children[1];
        let state = tree.node(content).unwrap().children[1];
        let node = tree.node(state).unwrap();
        assert_eq!(
            node.descriptor.view().style.color,
            Some(if index == 1 {
                Theme::green_dark().accent
            } else {
                Theme::green_dark().muted
            })
        );
        assert_eq!(
            node.descriptor.view().style.font_weight,
            Some(if index == 1 { 600.0 } else { 400.0 })
        );
    }
}
#[test]
fn invalid_external_current_preserves_last_accepted_progress_and_can_recover() {
    let app = App::memory();
    let progress = app.scope().cage(TimelineProgress::Active("build".into()));
    app.mount(model().progress(progress)).unwrap();
    progress.revise(|value| *value = TimelineProgress::Active("not-mounted".into()));
    assert!(app.flush().is_err());
    assert_eq!(states(&app), ["Completed", "Current", "Upcoming"]);
    assert_eq!(
        progress.get(),
        TimelineProgress::Active("not-mounted".into())
    );
    progress.revise(|value| *value = TimelineProgress::Active("design".into()));
    app.flush().unwrap();
    assert_eq!(states(&app), ["Current", "Upcoming", "Upcoming"]);
}
#[test]
fn namespaces_keys_duplicates_limits_and_labels_validate_before_creation() {
    for widget in [
        timeline("bad key"),
        timeline("key").item(timeline_item("bad key")),
        timeline("key")
            .item(timeline_item("a"))
            .item(timeline_item("a")),
        model().active("missing"),
        model().heading_level(7),
        model().state_labels("", "Current", "Upcoming"),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert_eq!(app.tree().len(), 0);
    }
    let app = App::memory();
    assert!(
        app.mount(
            timeline("large")
                .items((0..=MAX_TIMELINE_ITEMS).map(|i| timeline_item(format!("i{i}"))))
        )
        .is_err()
    );
    assert_eq!(app.tree().len(), 0);
}
#[test]
fn timeline_slots_keep_date_text_marker_and_controls_independent() {
    let html = glory::ssr::render(
        timeline("release")
            .name("Release progress")
            .active("build")
            .item(
                timeline_item("build")
                    .title("Build <&>")
                    .timestamp("2026-10-24")
                    .description("Description")
                    .marker(glory::ui::text().content("Marker"))
                    .fill(sapid::button().label("View build")),
            ),
    )
    .unwrap();
    for text in [
        "Build &lt;&amp;&gt;",
        "2026-10-24",
        "Description",
        "Marker",
        "View build",
        "Current",
    ] {
        assert!(html.contains(text));
    }
    assert!(html.contains("<ol"));
    assert!(html.contains("<li"));
    assert!(html.contains("<h3"));
    assert!(html.contains("<button"));
    assert!(!html.contains("role=\"tab\""));
}
#[test]
fn all_themes_and_unstyled_cover_root_item_and_state_without_history() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let app = App::memory();
        app.mount(model().theme(theme.clone())).unwrap();
        assert_eq!(
            app.tree()
                .node(find(&app, "sapid-timeline"))
                .unwrap()
                .descriptor
                .view()
                .style
                .color,
            Some(theme.foreground)
        );
    }
    let app = App::memory();
    app.mount(model().unstyled()).unwrap();
    for class in [
        "sapid-timeline",
        "sapid-timeline-item",
        "sapid-timeline-title",
        "sapid-timeline-state",
        "sapid-timeline-content",
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
fn timeline_effects_sources_and_refs_retire_together() {
    let app = App::memory();
    let progress = app.scope().cage(TimelineProgress::Pending);
    let reference = app.scope().cage(None);
    let baseline = app.runtime_stats();
    let mounted = app
        .scope()
        .mount_with(|| model().progress(progress).node_ref(reference))
        .unwrap();
    app.flush().unwrap();
    assert!(reference.get().is_some());
    drop(mounted);
    app.flush().unwrap();
    assert_eq!(reference.get(), None);
    assert_eq!(app.runtime_stats().edges, baseline.edges);
    assert_eq!(app.stats().nodes, 0);
    progress.revise(|value| *value = TimelineProgress::Complete);
    app.flush().unwrap();
    assert_eq!(app.stats().nodes, 0);
}

#[test]
fn explicit_item_theme_and_size_override_parent_inheritance() {
    let app = App::memory();
    app.mount(
        timeline("override")
            .theme(Theme::green_dark())
            .size(sapid::Size::Large)
            .item(
                timeline_item("inherited")
                    .title("Inherited")
                    .class("inherited"),
            )
            .item(
                timeline_item("override")
                    .title("Override")
                    .class("override")
                    .theme(Theme::blue_light())
                    .size(sapid::Size::Small),
            ),
    )
    .unwrap();
    let tree = app.tree();
    for (item, color, size) in [
        (
            find(&app, "inherited"),
            Theme::green_dark().foreground,
            "sapid-size-large",
        ),
        (
            find(&app, "override"),
            Theme::blue_light().foreground,
            "sapid-size-small",
        ),
    ] {
        let node = tree.node(item).unwrap();
        assert!(
            node.descriptor
                .view()
                .classes
                .iter()
                .any(|class| class == size)
        );
        let content = node.children[1];
        let title = tree.node(content).unwrap().children[0];
        assert_eq!(
            tree.node(title).unwrap().descriptor.view().style.color,
            Some(color)
        );
    }
}
