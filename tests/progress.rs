use glory::render::{NodeId, Props, Role};
use glory::style::{Color, Length, Style};
use glory::ui::App;
use sapid::{Theme, progress, ring_progress, semi_circle_progress};
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
            .any(|value| value == class)
        {
            return id;
        }
        pending.extend(&node.children);
    }
    panic!("missing {class}")
}
#[test]
fn linear_progress_has_finite_semantics_actual_fraction_and_no_range_actions() {
    let app = App::memory();
    app.mount(
        progress(30.)
            .name("Upload")
            .range(10., 50.)
            .value_text("Half uploaded"),
    )
    .unwrap();
    let tree = app.tree();
    let root = tree.node(tree.roots()[0]).unwrap().descriptor.view();
    assert_eq!(root.role, Some(Role::ProgressBar));
    let value = root.semantics.as_ref().unwrap().progress.as_ref().unwrap();
    assert_eq!(value.fraction().unwrap(), Some(0.5));
    assert_eq!(value.value_text.as_deref(), Some("Half uploaded"));
    drop(tree);
    let fill = find(&app, "sapid-progress-fill");
    assert_eq!(
        app.tree().node(fill).unwrap().descriptor.view().style.width,
        Some(Length::Percent(50.))
    );
    assert_eq!(app.stats().handlers, 0);
    let html =
        glory::ssr::render(progress(30.).range(10., 50.).value_text("Half uploaded")).unwrap();
    assert!(
        html.contains("role=\"progressbar\"")
            && html.contains("aria-valuemin=\"10\"")
            && html.contains("aria-valuemax=\"50\"")
            && html.contains("aria-valuenow=\"30\"")
            && html.contains("aria-valuetext=\"Half uploaded\"")
    );
    assert!(!html.contains("tabindex"));
}
#[test]
fn circular_progress_uses_bounded_real_paths_for_all_themes_and_endpoints() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        for semi in [false, true] {
            for value in [0., 25., 50., 100.] {
                let app = App::memory();
                let widget = if semi {
                    semi_circle_progress(value)
                } else {
                    ring_progress(value)
                };
                app.mount(widget.theme(theme.clone()).size(120.).thickness(12.))
                    .unwrap();
                let arc = find(&app, "sapid-progress-arc");
                let tree = app.tree();
                let Props::Graphic(props) = tree.node(arc).unwrap().descriptor.view().props else {
                    panic!()
                };
                props.validate().unwrap();
                assert!(props.commands.len() <= 6);
                assert_eq!(props.commands.is_empty(), value == 0.);
                assert!(props.alternative.is_none());
                assert_eq!(props.stroke.unwrap().color, theme.accent);
                assert_eq!(props.height, if semi { 66. } else { 120. });
            }
        }
    }
}
#[test]
fn controlled_value_indeterminate_and_text_reject_invalid_input_and_recover() {
    let app = App::memory();
    let value = app.scope().cage(25.);
    let unknown = app.scope().cage(false);
    let text = app.scope().cage("Quarter".to_string());
    app.mount(progress(value).indeterminate(unknown).value_text(text))
        .unwrap();
    let root = app.tree().roots()[0];
    let fill = find(&app, "sapid-progress-fill");
    value.revise(|value| *value = 101.);
    text.revise(|value| *value = " ".into());
    assert!(app.flush().is_err());
    assert_eq!(value.get(), 101.);
    assert_eq!(
        app.tree().node(fill).unwrap().descriptor.view().style.width,
        Some(Length::Percent(25.))
    );
    value.revise(|value| *value = 50.);
    text.revise(|value| *value = "Waiting".into());
    unknown.revise(|value| *value = true);
    app.flush().unwrap();
    assert!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .progress
            .as_ref()
            .unwrap()
            .current
            .is_none()
    );
    assert_eq!(
        app.tree().node(fill).unwrap().descriptor.view().style.width,
        Some(Length::Percent(30.))
    );
    assert_eq!(find(&app, "sapid-progress-fill"), fill);
    unknown.revise(|value| *value = false);
    app.flush().unwrap();
    assert_eq!(
        app.tree().node(fill).unwrap().descriptor.view().style.width,
        Some(Length::Percent(50.))
    );
    assert_eq!(app.stats().handlers, 0);
}
#[test]
fn invalid_initial_ranges_geometry_and_strings_create_no_visible_nodes() {
    for widget in [
        progress(f64::NAN),
        progress(f64::INFINITY),
        progress(-1.),
        progress(101.),
        progress(20.).range(50., 10.),
        progress(20.).range(0., f64::MAX).size(f64::NAN),
        ring_progress(20.).size(15.),
        ring_progress(20.).thickness(51.),
        semi_circle_progress(20.).thickness(0.),
        progress(20.).name(" "),
        progress(20.).value_text("x".repeat(1025)),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert!(app.tree().roots().is_empty());
    }
}
#[test]
fn unstyled_keeps_progress_content_and_explicit_overrides() {
    let app = App::memory();
    app.mount(
        progress(50.)
            .theme(Theme::green_dark())
            .unstyled()
            .class("custom-progress")
            .style(Style::new().height(Length::Px(10.))),
    )
    .unwrap();
    let root = app.tree().roots()[0];
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .style
            .height,
        Some(Length::Px(10.))
    );
    assert!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .style
            .background
            .is_none()
    );
    let fill = find(&app, "sapid-progress-fill");
    assert_eq!(
        app.tree()
            .node(fill)
            .unwrap()
            .descriptor
            .view()
            .style
            .background,
        Some(Theme::green_dark().foreground)
    );
    let app = App::memory();
    app.mount(ring_progress(50.).unstyled()).unwrap();
    let arc = find(&app, "sapid-progress-arc");
    let tree = app.tree();
    let Props::Graphic(value) = tree.node(arc).unwrap().descriptor.view().props else {
        panic!()
    };
    assert_ne!(value.stroke.unwrap().color, Color(0, 0, 0, 0));
    assert!(tree.node(tree.roots()[0]).unwrap().children.len() == 1);
}
