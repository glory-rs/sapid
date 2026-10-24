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
            if !id.is_empty() {
                assert!(html.contains("site-code"));
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
