use glory::render::NodeId;
use glory::style::{
    AspectRatio as Ratio, Display, GridPlacement, GridTrack, GridTracks, Length, Style,
};
use glory::ui::App;
use sapid::{Theme, aspect_ratio, grid, grid_item, simple_grid};
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
        pending.extend(node.children.iter().copied());
    }
    panic!("missing {class}")
}
fn slots(app: &App) -> Vec<(GridPlacement, GridPlacement)> {
    let tree = app.tree();
    tree.node(tree.roots()[0])
        .unwrap()
        .children
        .iter()
        .map(|id| {
            let style = &tree.node(*id).unwrap().descriptor.view().style;
            (style.grid_column.unwrap(), style.grid_row.unwrap())
        })
        .collect()
}
fn at(start: u16, span: u16) -> GridPlacement {
    GridPlacement::At { start, span }
}
#[test]
fn grid_uses_true_layout_declarations_and_finite_first_fit_spans_without_interactive_roles() {
    let app = App::memory();
    app.mount(
        grid("cards")
            .columns(3)
            .gap(8.0)
            .item(grid_item("a").span(2).row_span(2))
            .item(grid_item("b"))
            .item(grid_item("c"))
            .item(grid_item("d").span(3)),
    )
    .unwrap();
    assert_eq!(
        slots(&app),
        vec![
            (at(1, 2), at(1, 2)),
            (at(3, 1), at(1, 1)),
            (at(3, 1), at(2, 1)),
            (at(1, 3), at(3, 1))
        ]
    );
    let tree = app.tree();
    let root = tree.node(tree.roots()[0]).unwrap();
    assert_eq!(root.descriptor.view().style.display, Some(Display::Grid));
    assert_eq!(
        root.descriptor.view().style.grid_columns,
        Some(GridTracks::Tracks(vec![GridTrack::Fr(1.0); 3]))
    );
    assert_eq!(root.descriptor.view().role, None);
    assert_eq!(root.descriptor.view().style.gap, Some(8.0));
    for id in &root.children {
        let node = tree.node(*id).unwrap();
        assert_eq!(node.descriptor.view().role, None);
        assert_eq!(
            node.descriptor.view().style.min_width,
            Some(Length::Px(0.0))
        );
        assert!(
            node.descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .id
                .is_some()
        );
    }
}
#[test]
fn simple_grid_equal_tracks_preserve_source_order_and_advance_rows() {
    let app = App::memory();
    app.mount(simple_grid("simple").columns(2).items([
        grid_item("a"),
        grid_item("b"),
        grid_item("c"),
    ]))
    .unwrap();
    assert_eq!(
        slots(&app),
        [
            (at(1, 1), at(1, 1)),
            (at(2, 1), at(1, 1)),
            (at(1, 1), at(2, 1))
        ]
    );
    assert_eq!(app.stats().handlers, 0);
    assert!(glory::ssr::render(simple_grid("bad").item(grid_item("span").span(2))).is_err());
}
#[test]
fn invalid_keys_tracks_spans_collisions_and_item_budgets_fail_before_creation() {
    let invalid = vec![
        grid("bad key"),
        grid("x").columns(0),
        grid("x").columns(65),
        grid("x").gap(-1.0),
        grid("x").column_tracks([GridTrack::Fr(0.0)]),
        grid("x").item(grid_item("a")).item(grid_item("a")),
        grid("x").item(grid_item("bad key")),
        grid("x").columns(2).item(grid_item("a").span(3)),
        grid("x").item(grid_item("a").row(0)),
        grid("x").item(grid_item("a").row(64).row_span(2)),
        grid("x").item(grid_item("a").column(0)),
        grid("x").item(grid_item("a").column(64)),
        grid("x")
            .columns(2)
            .item(grid_item("a").column(1).row(1))
            .item(grid_item("b").column(1).row(1)),
        grid("x")
            .columns(1)
            .items((0..65).map(|i| grid_item(format!("item-{i}")))),
        grid("x").items((0..257).map(|i| grid_item(format!("item-{i}")))),
    ];
    for widget in invalid {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert_eq!(app.stats().nodes, 0);
    }
}
#[test]
fn maximum_column_and_row_end_are_valid_without_unbounded_implicit_placement() {
    let app = App::memory();
    app.mount(
        grid("limit")
            .columns(64)
            .item(grid_item("edge").column(64).row(64)),
    )
    .unwrap();
    assert_eq!(slots(&app), [(at(64, 1), at(64, 1))]);
    let app = App::memory();
    app.mount(
        grid("limit")
            .columns(64)
            .item(grid_item("full").span(64).row_span(64)),
    )
    .unwrap();
    assert_eq!(slots(&app), [(at(1, 64), at(1, 64))]);
}

#[test]
fn full_item_budget_fits_four_rows_and_ratio_effect_refs_retire_with_owner() {
    let app = App::memory();
    app.mount(
        grid("full")
            .columns(64)
            .items((0..256).map(|i| grid_item(format!("item-{i}")))),
    )
    .unwrap();
    let positions = slots(&app);
    assert_eq!(positions.len(), 256);
    assert_eq!(positions[255], (at(64, 1), at(4, 1)));
    let app = App::memory();
    let source = app.scope().cage(2.0);
    let reference = app.scope().cage(None);
    let baseline = app.runtime_stats();
    let mounted = app
        .scope()
        .mount_with(|| {
            aspect_ratio(source)
                .node_ref(reference)
                .style(Style::new().aspect_ratio(Ratio::Auto))
        })
        .unwrap();
    app.flush().unwrap();
    assert!(reference.get().is_some());
    source.revise(|ratio| *ratio = f64::NAN);
    assert!(app.flush().is_err());
    assert_eq!(
        app.tree()
            .node(reference.get().unwrap())
            .unwrap()
            .descriptor
            .view()
            .style
            .aspect_ratio,
        Some(Ratio::Auto)
    );
    source.revise(|ratio| *ratio = 1.0);
    app.flush().unwrap();
    drop(mounted);
    app.flush().unwrap();
    assert_eq!(reference.get(), None);
    assert_eq!(app.runtime_stats().edges, baseline.edges);
    source.revise(|ratio| *ratio = f64::NAN);
    app.flush().unwrap();
    assert!(app.tree().is_empty());
}
#[test]
fn all_themes_unstyled_and_explicit_item_theme_keep_structure_and_overrides() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let app = App::memory();
        app.mount(grid("theme").theme(theme.clone()).item(grid_item("a")))
            .unwrap();
        let tree = app.tree();
        assert_eq!(
            tree.node(tree.roots()[0])
                .unwrap()
                .descriptor
                .view()
                .style
                .color,
            Some(theme.foreground)
        );
    }
    let app = App::memory();
    app.mount(
        grid("theme").theme(Theme::green_dark()).item(
            grid_item("override")
                .theme(Theme::blue_light())
                .class("override"),
        ),
    )
    .unwrap();
    assert_eq!(
        app.tree()
            .node(find(&app, "override"))
            .unwrap()
            .descriptor
            .view()
            .style
            .color,
        Some(Theme::blue_light().foreground)
    );
    let app = App::memory();
    app.mount(
        grid("bare")
            .unstyled()
            .style(Style::new().gap(2.0))
            .item(grid_item("a")),
    )
    .unwrap();
    let tree = app.tree();
    let root = tree.node(tree.roots()[0]).unwrap();
    assert_eq!(root.descriptor.view().style.display, Some(Display::Grid));
    assert_eq!(root.descriptor.view().style.color, None);
    assert_eq!(root.descriptor.view().style.gap, Some(2.0));
    let child = tree.node(root.children[0]).unwrap();
    assert_eq!(child.descriptor.view().style.color, None);
    assert_eq!(child.descriptor.view().style.grid_column, Some(at(1, 1)));
}
#[test]
fn dynamic_explicit_styles_patch_only_root_and_retirement_clears_references() {
    let app = App::memory();
    let custom = app.scope().cage(Style::new().width(Length::Px(300.0)));
    let reference = app.scope().cage(None);
    let mounted = app
        .scope()
        .mount_with(|| {
            grid("live").style(custom).node_ref(reference).item(
                grid_item("a").fill(sapid::text_field().name("draft").default_value("Keep")),
            )
        })
        .unwrap();
    app.flush().unwrap();
    let id = reference.get().unwrap();
    let children = app.tree().node(id).unwrap().children.clone();
    custom.revise(|style| style.width = Some(Length::Px(200.0)));
    app.flush().unwrap();
    assert_eq!(app.tree().node(id).unwrap().children, children);
    assert_eq!(
        app.tree().node(id).unwrap().descriptor.view().style.width,
        Some(Length::Px(200.0))
    );
    drop(mounted);
    app.flush().unwrap();
    assert_eq!(reference.get(), None);
    assert!(app.tree().is_empty());
}
#[test]
fn ratio_finite_source_updates_are_silent_preserve_children_and_recover_from_invalid_input() {
    let app = App::memory();
    let source = app.scope().cage(2.0);
    app.mount(aspect_ratio(source).fill(glory::ui::text().content("content")))
        .unwrap();
    let root = app.tree().roots()[0];
    let children = app.tree().node(root).unwrap().children.clone();
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .style
            .aspect_ratio,
        Some(Ratio::Ratio(2.0))
    );
    source.revise(|value| *value = 1.0);
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .style
            .aspect_ratio,
        Some(Ratio::Ratio(1.0))
    );
    source.revise(|value| *value = f64::NAN);
    assert!(app.flush().is_err());
    assert!(source.get().is_nan());
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .style
            .aspect_ratio,
        Some(Ratio::Ratio(1.0))
    );
    source.revise(|value| *value = 4.0 / 3.0);
    app.flush().unwrap();
    assert_eq!(app.tree().node(root).unwrap().children, children);
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .style
            .aspect_ratio,
        Some(Ratio::Ratio(4.0 / 3.0))
    );
    assert_eq!(app.stats().handlers, 0);
}
#[test]
fn ratio_initial_rejection_unstyled_explicit_axes_and_css_precedence_are_clear() {
    for ratio in [0.0, -1.0, 1e-7, 1e7, f64::NAN, f64::INFINITY] {
        let app = App::memory();
        assert!(app.mount(aspect_ratio(ratio)).is_err());
        assert!(app.tree().is_empty());
    }
    let app = App::memory();
    app.mount(
        aspect_ratio(2.0)
            .width(Length::Px(100.0))
            .height(Length::Px(20.0))
            .unstyled()
            .style(Style::new().aspect_ratio(Ratio::Auto)),
    )
    .unwrap();
    let tree = app.tree();
    let node = tree.node(tree.roots()[0]).unwrap().descriptor.view();
    assert_eq!(node.style.width, Some(Length::Px(100.0)));
    assert_eq!(node.style.height, Some(Length::Px(20.0)));
    assert_eq!(node.style.aspect_ratio, Some(Ratio::Auto));
    assert_eq!(node.style.color, None);
}
#[test]
fn ssr_grid_slots_escape_content_and_use_layout_without_fake_table_roles() {
    let html = glory::ssr::render(
        grid("ssr")
            .column_tracks([GridTrack::Px(40.0), GridTrack::Fr(1.0)])
            .row_tracks([GridTrack::Auto])
            .item(grid_item("a").fill(glory::ui::text().content("<&>")))
            .item(grid_item("b").fill(aspect_ratio(16.0 / 9.0))),
    )
    .unwrap();
    assert!(
        html.contains("display:grid")
            && html.contains("grid-template-columns:40px 1fr")
            && html.contains("aspect-ratio:1.777")
    );
    assert!(html.contains("&lt;&amp;&gt;"));
    for role in ["grid", "row", "cell"] {
        assert!(!html.contains(&format!("role=\"{role}\"")));
    }
}
