use glory::render::{Event, EventKind, Props};
use glory::style::{Color, Length, PointerEvents, Position, Style};
use glory::ui::{App, button};
use sapid::{Theme, loading_overlay, overlay};
#[test]
fn controlled_loading_publishes_busy_inert_and_mask_visibility_in_one_flush_and_preserves_sources()
{
    let app = App::memory();
    let loading = app.scope().cage(false);
    let clicks = app.scope().cage(0usize);
    let content = app.scope().cage(None);
    let mask = app.scope().cage(None);
    let root_ref = app.scope().cage(None);
    app.mount(
        loading_overlay(loading)
            .name("Upload area")
            .label("Loading files")
            .content_ref(content)
            .mask_ref(mask)
            .node_ref(root_ref)
            .fill(
                button()
                    .label("Edit upload")
                    .on(EventKind::Activate, move |_| clicks.revise(|n| *n += 1)),
            ),
    )
    .unwrap();
    let root = root_ref.get().unwrap();
    let child = content.get().unwrap();
    let cover = mask.get().unwrap();
    let button = app.tree().node(child).unwrap().children[0];
    for next in [true, false, true, false] {
        let writes = app.runtime_stats().source_writes;
        loading.revise(|v| *v = next);
        app.flush().unwrap();
        assert_eq!(app.runtime_stats().source_writes, writes + 1);
        let tree = app.tree();
        assert_eq!(
            tree.node(root)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .busy,
            Some(next)
        );
        assert_eq!(
            tree.node(child)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .inert,
            next
        );
        assert_eq!(tree.node(cover).unwrap().descriptor.view().hidden(), !next);
        assert_eq!(tree.node(root).unwrap().children.to_vec(), [child, cover]);
        drop(tree);
        assert_eq!(app.dispatch(button, Event::Activate).unwrap(), !next);
        app.flush().unwrap();
        assert_eq!(loading.get(), next);
    }
    assert_eq!(clicks.get(), 2);
    app.clear().unwrap();
    assert_eq!(
        (root_ref.get(), content.get(), mask.get()),
        (None, None, None)
    );
    assert_eq!(app.stats().handlers, 0);
}
#[test]
fn decorative_overlay_never_sets_busy_inert_or_input_roles_and_pointer_targeting_is_transparent() {
    let app = App::memory();
    let visible = app.scope().cage(true);
    let clicks = app.scope().cage(0usize);
    app.mount(
        overlay()
            .visible(visible)
            .fill(button().on(EventKind::Activate, move |_| clicks.revise(|n| *n += 1))),
    )
    .unwrap();
    let tree = app.tree();
    let root = tree.node(tree.roots()[0]).unwrap();
    let content = root.children[0];
    let mask = root.children[1];
    let button = tree.node(content).unwrap().children[0];
    assert_eq!(
        root.descriptor.view().semantics.as_ref().unwrap().busy,
        None
    );
    assert!(
        !tree
            .node(content)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .inert
    );
    assert_eq!(
        tree.node(mask)
            .unwrap()
            .descriptor
            .view()
            .style
            .pointer_events,
        Some(PointerEvents::None)
    );
    assert!(tree.node(mask).unwrap().children.is_empty());
    assert_eq!(tree.node(mask).unwrap().descriptor.view().role, None);
    drop(tree);
    assert!(app.dispatch(button, Event::Activate).unwrap());
    visible.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(app.tree().node(mask).unwrap().descriptor.view().hidden());
    assert!(app.dispatch(button, Event::Activate).unwrap());
    assert_eq!(clicks.get(), 2);
}
#[test]
fn theme_unstyled_and_explicit_alpha_keep_geometry_and_spinner_foreground_independent() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        for unstyled in [false, true] {
            let app = App::memory();
            let widget = loading_overlay(true)
                .theme(theme.clone())
                .style(Style::new().width(Length::Px(100.)).height(Length::Px(80.)));
            app.mount(if unstyled { widget.unstyled() } else { widget })
                .unwrap();
            let tree = app.tree();
            let root = tree.node(tree.roots()[0]).unwrap();
            let mask = tree.node(root.children[1]).unwrap();
            assert_eq!(
                root.descriptor.view().style.position,
                Some(Position::Relative)
            );
            assert_eq!(
                mask.descriptor.view().style.position,
                Some(Position::Absolute)
            );
            assert_eq!(mask.descriptor.view().style.opacity, None);
            assert_eq!(
                mask.descriptor.view().style.background,
                if unstyled {
                    None
                } else {
                    Some(Color(
                        theme.surface.0,
                        theme.surface.1,
                        theme.surface.2,
                        166,
                    ))
                }
            );
            let spinner = tree.node(mask.children[0]).unwrap();
            let drawing = tree.node(spinner.children[0]).unwrap();
            let Props::Graphic(drawing) = drawing.descriptor.view().props else {
                panic!()
            };
            assert_eq!(
                drawing.stroke.unwrap().color,
                if unstyled {
                    theme.foreground
                } else {
                    theme.accent
                }
            );
        }
    }
    let html = glory::ssr::render(
        loading_overlay(true)
            .name("Busy region")
            .fill(button().label("Retained action")),
    )
    .unwrap();
    assert!(
        html.contains("aria-busy=\"true\"")
            && html.contains("data-gly-inert=\"\"")
            && html.contains("inert=\"\"")
    );
    assert!(!html.contains("aria-modal") && !html.contains("popover="));
    let app = App::memory();
    app.mount(overlay().unstyled().color(Color(1, 2, 3, 128)).opacity(0.5))
        .unwrap();
    let root = app.tree().roots()[0];
    let mask = app.tree().node(root).unwrap().children[1];
    assert_eq!(
        app.tree()
            .node(mask)
            .unwrap()
            .descriptor
            .view()
            .style
            .background,
        Some(Color(1, 2, 3, 64))
    );
}
#[test]
fn invalid_initial_budget_and_runtime_names_or_position_retain_the_last_valid_widget() {
    for widget in [
        loading_overlay(true).opacity(f64::NAN),
        loading_overlay(true).opacity(-0.1),
        loading_overlay(true).opacity(1.1),
        loading_overlay(true).label(" "),
        loading_overlay(true).name("x".repeat(257)),
        loading_overlay(true).style(Style::new().position(Position::Static)),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert!(app.tree().roots().is_empty());
    }
    let app = App::memory();
    let label = app.scope().cage("Working".to_string());
    let style = app.scope().cage(Style::new());
    app.mount(loading_overlay(true).label(label).style(style))
        .unwrap();
    let root = app.tree().roots()[0];
    let mask = app.tree().node(root).unwrap().children[1];
    let spinner = app.tree().node(mask).unwrap().children[0];
    label.revise(|v| *v = " ".into());
    style.revise(|v| v.position = Some(Position::Static));
    assert!(app.flush().is_err());
    assert_eq!(label.get(), " ");
    assert_eq!(style.get().position, Some(Position::Static));
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .style
            .position,
        Some(Position::Relative)
    );
    assert_eq!(
        app.tree()
            .node(spinner)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Working")
    );
    label.revise(|v| *v = "Saved".into());
    style.revise(|v| v.position = Some(Position::Absolute));
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .style
            .position,
        Some(Position::Absolute)
    );
    assert_eq!(
        app.tree()
            .node(spinner)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Saved")
    );
}
