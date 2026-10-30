#![cfg(feature = "charts")]
use glory::{
    prelude::*,
    render::{Event, NodeId, Props, Role},
};
use sapid::charts::*;
use std::{cell::RefCell, rc::Rc};
#[path = "../examples/fixtures/plot_samples.rs"]
mod samples;
fn all(app: &App) -> Vec<NodeId> {
    let t = app.tree();
    let mut p = t.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = p.pop() {
        p.extend(&t.node(id).unwrap().children);
        out.push(id);
    }
    out
}
fn class(app: &App, c: &str) -> Vec<NodeId> {
    all(app)
        .into_iter()
        .filter(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|v| v == c)
        })
        .collect()
}
fn named(app: &App, name: &str) -> NodeId {
    all(app)
        .into_iter()
        .find(|id| {
            let t = app.tree();
            let d = t.node(*id).unwrap().descriptor.view();
            d.name.as_deref() == Some(name) || matches!(d.props,Props::Button(p)if p.label==name)
        })
        .unwrap_or_else(|| panic!("missing {name}"))
}
fn text(app: &App) -> String {
    all(app)
        .into_iter()
        .filter_map(
            |id| match app.tree().node(id).unwrap().descriptor.view().props {
                Props::Text(p) => Some(p.content.clone()),
                _ => None,
            },
        )
        .collect::<Vec<_>>()
        .join("|")
}
#[test]
fn eighteen_families_mount_real_vectors_exact_tables_keyboard_details_and_empty_states() {
    for kind in samples::KINDS {
        let app = App::memory();
        let data = app.scope().cage(samples::sample(kind));
        app.mount(plot("test", kind).name(samples::title(kind)).data(data))
            .unwrap_or_else(|e| panic!("{kind:?}: {e:?}"));
        assert!(!class(&app, "sapid-plot-layer").is_empty(), "{kind:?}");
        assert!(!class(&app, "sapid-plot-point").is_empty(), "{kind:?}");
        app.dispatch(named(&app, "Next data point"), Event::Activate)
            .unwrap();
        assert!(!text(&app).contains("No data point selected"), "{kind:?}");
        data.revise(|v| *v = samples::empty(kind));
        app.flush().unwrap();
        assert!(class(&app, "sapid-plot-point").is_empty());
        assert!(text(&app).contains("No visible plot data"));
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
    }
}
#[test]
fn paired_admission_keeps_the_accepted_scene_and_table_on_rejection() {
    let app = App::memory();
    let data = app.scope().cage(samples::sample(PlotKind::Pie));
    let opts = app.scope().cage(PlotOptions::default());
    app.mount(pie_chart("admit").data(data).options(opts))
        .unwrap();
    let before = class(&app, "sapid-plot-point");
    opts.revise(|o| o.hidden.insert("absent".into()));
    assert!(app.flush().is_err());
    assert_eq!(before, class(&app, "sapid-plot-point"));
    assert!(text(&app).contains("North"));
    opts.revise(|v| *v = PlotOptions::default());
    data.revise(|v| *v = PlotData::Points(vec![]));
    assert!(app.flush().is_err());
    assert_eq!(before, class(&app, "sapid-plot-point"));
}
#[test]
fn controlled_refusal_and_stable_keys_preserve_controls_and_exact_source_values() {
    let app = App::memory();
    let data = app.scope().cage(samples::sample(PlotKind::Pie));
    let proposed = Rc::new(RefCell::new(vec![]));
    let out = proposed.clone();
    app.mount(
        pie_chart("controlled")
            .data(data)
            .options(PlotOptions::default())
            .selection(None)
            .on_selection_change(move |v| out.borrow_mut().push(v)),
    )
    .unwrap();
    let mark = class(&app, "sapid-plot-point")[0];
    app.dispatch(named(&app, "Next data point"), Event::Activate)
        .unwrap();
    assert_eq!(proposed.borrow()[0], Some("one".into()));
    assert!(text(&app).contains("No data point selected"));
    let hide = named(&app, "Hide North");
    app.dispatch(hide, Event::Activate).unwrap();
    assert_eq!(hide, named(&app, "Hide North"));
    assert_eq!(class(&app, "sapid-plot-point").len(), 3);
    data.revise(samples::update);
    app.flush().unwrap();
    assert!(class(&app, "sapid-plot-point").contains(&mark));
    assert!(text(&app).contains("Updated North"));
}
#[test]
fn hierarchy_navigation_visits_current_parent_and_child_without_data_source_lookup() {
    let app = App::memory();
    app.mount(treemap("tree").data(samples::sample(PlotKind::Treemap)))
        .unwrap();
    app.dispatch(named(&app, "Next data point"), Event::Activate)
        .unwrap();
    assert!(text(&app).contains("Regions:"));
    app.dispatch(named(&app, "First child"), Event::Activate)
        .unwrap();
    assert!(text(&app).contains("North:"));
    app.dispatch(named(&app, "Parent node"), Event::Activate)
        .unwrap();
    assert!(text(&app).contains("Regions:"));
}
#[test]
fn ranges_expose_read_only_progress_and_full_matrix_tables_have_unique_bounded_ids() {
    for kind in [PlotKind::BarsList, PlotKind::Bullet, PlotKind::Gauge] {
        let app = App::memory();
        app.mount(plot("range", kind).data(samples::sample(kind)))
            .unwrap();
        assert!(all(&app).iter().any(|id| {
            let t = app.tree();
            let d = t.node(*id).unwrap().descriptor.view();
            d.role == Some(Role::ProgressBar)
                && d.semantics.as_ref().is_some_and(|s| s.progress.is_some())
        }));
    }
    let app = App::memory();
    let axis = |prefix: &str| {
        (0..32)
            .map(|i| ChartCategory::new(format!("{prefix}{i}"), format!("{prefix}{i}")))
            .collect::<Vec<_>>()
    };
    let data = PlotData::Matrix(MatrixData {
        rows: axis("r"),
        columns: axis("c"),
        cells: (0..1024)
            .map(|i| MatrixCell {
                key: format!("cell{i}"),
                row: format!("r{}", i / 32),
                column: format!("c{}", i % 32),
                value: i as f64,
            })
            .collect(),
    });
    app.mount(heatmap("m".repeat(MAX_PLOT_KEY_BYTES)).data(data))
        .unwrap();
    assert_eq!(class(&app, "sapid-plot-point").len(), 1024);
    let html =
        glory::ssr::render(heatmap("matrix").data(samples::sample(PlotKind::Heatmap))).unwrap();
    assert!(html.contains("<table"));
    assert!(!html.contains("role=\"grid\""));
    assert!(text(&app).contains("1023"));
}
#[test]
fn sampled_points_disclose_the_source_count_and_do_not_claim_a_complete_source_table() {
    let app = App::memory();
    let data = PlotData::sample_points(1_000_000, 16, |i| PointDatum {
        key: format!("p{i}"),
        label: format!("Row {i}"),
        x: (i % 100) as f64,
        y: 1.,
        size: 1.,
    })
    .unwrap();
    app.mount(scatter_chart("sample").data(data)).unwrap();
    assert_eq!(class(&app, "sapid-plot-point").len(), 16);
    assert!(text(&app).contains("16 of 1000000 sampled"));
}
