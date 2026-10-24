use glory::render::{FocusScopeConfig, SemanticId};
use glory::ui::App;
use sapid::focus_scope;
#[path = "../examples/web/focus_scope_form.rs"]
mod fixture;

#[test]
fn reactive_focus_policy_is_descriptor_only_and_keeps_content_identity() {
    let app = App::memory();
    let active = app.scope().cage(false);
    let reference = app.scope().cage(None);
    app.mount(
        focus_scope()
            .active(active)
            .node_ref(reference)
            .fill(glory::ui::input().value("Draft")),
    )
    .unwrap();
    let root = reference.get().unwrap();
    let before = app.tree().node(root).unwrap().children.to_vec();
    active.revise(|value| *value = true);
    app.flush().unwrap();
    let tree = app.tree();
    let node = tree.node(root).unwrap();
    assert_eq!(node.children.to_vec(), before);
    assert!(
        node.descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .focus_scope
            .as_ref()
            .unwrap()
            .active
    );
    assert_eq!(node.descriptor.view().role, None);
    assert!(!tree.inert(before[0]));
    assert_eq!(tree.focus_scope_nodes(root).unwrap(), Some(2));
    drop(tree);
    app.clear().unwrap();
    assert_eq!(app.tree().focus_scopes(), 0);
}

#[test]
fn explicit_policy_overrides_remain_reactive_without_writing_the_controlled_source() {
    let app = App::memory();
    let source = app.scope().cage(FocusScopeConfig::default());
    let active = app.scope().cage(false);
    let reference = app.scope().cage(None);
    app.mount(
        focus_scope()
            .configuration(source)
            .active(active)
            .loop_tab(false)
            .contain_focus(false)
            .auto_focus(false)
            .restore_focus(false)
            .initial_focus(SemanticId::new("late-initial").unwrap())
            .return_focus(SemanticId::new("late-return").unwrap())
            .node_ref(reference)
            .fill(glory::ui::text().content("Visible while inactive")),
    )
    .unwrap();
    assert_eq!(source.get(), FocusScopeConfig::default());
    active.revise(|value| *value = true);
    app.flush().unwrap();
    let tree = app.tree();
    let semantics = tree
        .node(reference.get().unwrap())
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap();
    let config = semantics.focus_scope.as_ref().unwrap();
    assert!(config.active);
    assert!(
        !config.loop_tab
            && !config.contain_focus
            && !config.auto_focus
            && !config.restore_focus
    );
    assert_eq!(
        config.initial_focus.as_ref().unwrap().as_str(),
        "late-initial"
    );
    assert_eq!(source.get(), FocusScopeConfig::default());
}

#[test]
fn scope_fixture_four_themes_has_stable_ssr_shape_and_no_input_capability_claim() {
    for hue in [sapid::Hue::Blue, sapid::Hue::Green] {
        for scheme in [sapid::ColorScheme::Light, sapid::ColorScheme::Dark] {
            let theme = sapid::Theme::new(hue, scheme);
            let first = glory::ssr::snapshot(fixture::focus_scope_form(theme.clone()))
                .unwrap()
                .html;
            let second = glory::ssr::snapshot(fixture::focus_scope_form(theme))
                .unwrap()
                .html;
            assert_eq!(first, second);
            assert!(first.contains("data-gly-focus-scope=\"false\""));
            assert!(first.contains("data-gly-focus-loop=\"true\""));
            assert!(!first.contains("aria-modal") && !first.contains("popover="));
        }
    }
}
