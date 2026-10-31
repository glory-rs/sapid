use glory::render::{Event, NodeId, Props};
use glory::ui::App;
use sapid::Theme;
use sapid_web_example::{showcase, showcase_catalog::ENTRIES};

fn nodes(app: &App) -> Vec<NodeId> {
    let mut pending = app.tree().roots().to_vec();
    let mut result = Vec::new();
    while let Some(id) = pending.pop() {
        pending.extend(app.tree().node(id).unwrap().children.iter().copied());
        result.push(id);
    }
    result
}

#[test]
fn catalogue_has_exactly_149_unique_routes_and_real_example_sources() {
    let mut ids = std::collections::BTreeSet::new();
    assert_eq!(ENTRIES.len(), 149);
    for entry in ENTRIES {
        assert!(ids.insert(entry.id), "duplicate {}", entry.id);
        assert!(!entry.name.is_empty() && !entry.description.is_empty());
        assert!(!entry.code.trim().is_empty(), "missing source {}", entry.id);
        assert!(showcase::route(entry.id, &Theme::blue_light()).contains(entry.id));
    }
}

#[test]
fn docs_navigation_is_live_semantic_links_and_mobile_menu_is_rust_state() {
    let app = App::memory();
    app.mount(showcase::site(Theme::blue_light(), "P0-01".into()))
        .unwrap();
    let all = nodes(&app);
    let navigation: Vec<_> = all
        .iter()
        .copied()
        .filter(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|c| c == "site-nav-link")
        })
        .collect();
    assert_eq!(navigation.len(), 149);
    assert!(navigation.iter().all(|id| matches!(
        app.tree().node(*id).unwrap().descriptor.view().props,
        Props::Link(_)
    )));
    let toggle = all
        .into_iter()
        .find(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .name
                .as_deref()
                == Some("Open navigation")
        })
        .unwrap();
    app.dispatch(toggle, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(toggle)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Close navigation")
    );
    app.dispatch(toggle, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(toggle)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Open navigation")
    );
}

#[test]
fn documentation_tab_switches_directly_to_host_support() {
    let app = App::memory();
    app.mount(showcase::site(Theme::blue_light(), "P0-01".into()))
        .unwrap();
    let capabilities = nodes(&app)
        .into_iter()
        .find(|id| {
            matches!(
                app.tree().node(*id).unwrap().descriptor.view().props,
                Props::Button(button) if button.label == "Capabilities"
            )
        })
        .unwrap();
    app.dispatch(capabilities, Event::Activate).unwrap();
    app.flush().unwrap();
    assert!(nodes(&app).into_iter().any(|id| {
        matches!(
            app.tree().node(id).unwrap().descriptor.view().props,
            Props::Text(text) if text.content == "Host support"
        )
    }));
}

#[test]
fn representative_ssr_documents_render_in_all_themes_without_frames() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        for id in ["", "P0-01", "P0-02", "P9-01", "P13-01"] {
            let html = glory::ssr::snapshot(showcase::site(theme.clone(), id.into()))
                .unwrap()
                .html;
            assert!(html.contains("sapid-site"));
            assert!(!html.contains("<iframe"));
            assert!(html.contains("Search widgets"));
            assert!(html.contains("v0.6.1"));
            assert!(!html.contains("v0.6.0"));
            if !id.is_empty() {
                assert!(html.contains("site-code"));
                assert!(html.contains(">Tested<") || html.contains(">Model only<"));
                assert!(!html.contains(">tested<"));
            }
        }
    }
}

#[test]
fn unknown_route_is_a_friendly_rust_rendered_page() {
    let html = glory::ssr::snapshot(showcase::site(Theme::blue_light(), "missing".into()))
        .unwrap()
        .html;
    assert!(html.contains("Widget not found"));
    assert!(html.contains("showcase.html?widget="));
}

#[test]
fn every_displayed_source_builds_its_preview_and_fits_the_highlighter() {
    for entry in ENTRIES {
        assert!(entry.code.contains("pub fn demo("), "{}", entry.id);
        assert!(
            !entry.code.contains("crate::"),
            "{} must be self-contained",
            entry.id
        );
        sapid::tokenize_code(entry.code, sapid::CodeLanguage::Rust)
            .unwrap_or_else(|error| panic!("{} source: {error}", entry.id));
        let app = App::memory();
        app.mount((entry.build)(Theme::blue_light()))
            .unwrap_or_else(|error| panic!("{} preview: {error}", entry.id));
        assert!(!app.tree().is_empty(), "{} preview", entry.id);
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0, "{} cleanup", entry.id);
        assert_eq!(app.runtime_stats().tasks, 0, "{} tasks", entry.id);
    }
}

#[test]
fn studio_builds_real_role_inputs_mode_controls_and_retires_all_state() {
    for theme in [Theme::blue_light(), Theme::green_dark()] {
        let app = App::memory();
        app.mount(showcase::site(theme, "theme-studio".into()))
            .unwrap();
        let all = nodes(&app);
        let input = all
            .iter()
            .find(|id| {
                app.tree()
                    .node(**id)
                    .unwrap()
                    .descriptor
                    .view()
                    .name
                    .as_deref()
                    == Some("Base canvas")
            })
            .unwrap();
        assert!(matches!(
            app.tree().node(*input).unwrap().descriptor.view().props,
            Props::Input(_)
        ));
        let tab = all
            .iter()
            .find(|id| {
                app.tree()
                    .node(**id)
                    .unwrap()
                    .descriptor
                    .view()
                    .name
                    .as_deref()
                    == Some("Edit Dark colors")
            })
            .unwrap();
        app.dispatch(*tab, Event::Activate).unwrap();
        app.flush().unwrap();
        assert!(nodes(&app).iter().any(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .name
                .as_deref()
                == Some("Dark canvas")
        }));
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(app.stats().handlers, 0);
        assert_eq!(app.runtime_stats().tasks, 0);
    }
}

#[test]
fn route_changes_preserve_one_editor_workspace_without_retaining_old_views() {
    use sapid_web_example::showcase_navigation::{Page, State, app as route_app};
    use sapid_web_example::theme_studio::model::{Layer, Target};
    let app = App::memory();
    let state = State::new(
        app.scope(),
        Page::builtin(Theme::blue_dark(), "theme-studio".into()),
        Vec::new(),
        false,
    );
    app.mount(route_app(state)).unwrap();
    state.editor.draft.revise(|d| {
        d.record.title = "Kept draft".into();
        d.set_text(Layer::Base, Target::Role(0), "#1".into());
    });
    state.editor.layer.revise(|v| *v = Layer::Dark);
    let mut next = Page::builtin(Theme::green_light(), "P0-01".into());
    next.revision = 1;
    state.page.revise(|v| *v = next);
    app.flush().unwrap();
    let mut next = Page::builtin(Theme::green_dark(), "theme-studio".into());
    next.revision = 2;
    state.page.revise(|v| *v = next);
    app.flush().unwrap();
    assert_eq!(
        state.editor.draft.with(|d| d.record.title.clone()),
        "Kept draft"
    );
    assert_eq!(
        state
            .editor
            .draft
            .with(|d| d.text(Layer::Base, Target::Role(0))),
        "#1"
    );
    assert_eq!(state.editor.layer.get(), Layer::Dark);
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(app.stats().handlers, 0);
    drop(app);
    assert!(state.editor.draft.try_get().is_err());
}
