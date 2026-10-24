#![cfg(feature = "charts")]
use glory::{
    prelude::*,
    render::{Event, NodeId, Props, Role},
};
use sapid::charts::*;
use std::{cell::RefCell, rc::Rc};
fn data() -> ChartData {
    ChartData {
        categories: vec![
            ChartCategory::new("head", "January"),
            ChartCategory::new("feb", "February"),
        ],
        series: vec![
            ChartSeries::new("category", "Revenue", [Some(2.), Some(-1.)]),
            ChartSeries::new("cost", "Cost", [None, Some(3.)]).kind(ChartSeriesKind::Bar),
        ],
    }
}
fn all(app: &App) -> Vec<NodeId> {
    let t = app.tree();
    let mut pending = t.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = pending.pop() {
        pending.extend(&t.node(id).unwrap().children);
        out.push(id);
    }
    out
}
fn named(app: &App, name: &str) -> NodeId {
    all(app)
        .into_iter()
        .find(|id| {
            let t = app.tree();
            let d = t.node(*id).unwrap().descriptor.view();
            d.name.as_deref() == Some(name)
                || matches!(d.props,Props::Button(p)if p.label==name)
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
fn count(app: &App, class: &str) -> usize {
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
                .any(|c| c == class)
        })
        .count()
}
#[test]
fn full_table_uses_intrinsic_headers_and_point_controls_do_not_invent_grid_roles() {
    let app = App::memory();
    app.mount(composite_chart("sales").data(data())).unwrap();
    assert_eq!(count(&app, "sapid-chart-point"), 3);
    let t = app.tree();
    let nodes = all(&app);
    assert!(nodes.iter().any(|id| {
        t.node(*id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .is_some_and(|s| s.document_part == Some(glory::render::DocumentPart::Table))
    }));
    assert!(
        nodes
            .iter()
            .all(|id| t.node(*id).unwrap().descriptor.view().role != Some(Role::Grid))
    );
    drop(t);
    assert!(text(&app).contains("No value"));
    app.dispatch(named(&app, "Next data point"), Event::Activate)
        .unwrap();
    assert!(text(&app).contains("January · Revenue: 2"));
    app.dispatch(named(&app, "Next data point"), Event::Activate)
        .unwrap();
    assert!(text(&app).contains("February · Revenue: -1"));
    app.dispatch(named(&app, "Clear data point"), Event::Activate)
        .unwrap();
    assert!(text(&app).contains("No data point selected"));
}
#[test]
fn legend_retains_its_identity_and_controlled_refusal_keeps_geometry() {
    let app = App::memory();
    let out = Rc::new(RefCell::new(vec![]));
    let result = out.clone();
    app.mount(
        chart("controlled")
            .data(data())
            .options(CartesianOptions::default())
            .on_options_change(move |v| result.borrow_mut().push(v)),
    )
    .unwrap();
    let id = named(&app, "Hide Revenue");
    app.dispatch(id, Event::Activate).unwrap();
    assert!(out.borrow()[0].hidden.contains("category"));
    assert_eq!(count(&app, "sapid-chart-point"), 3);
    assert_eq!(id, named(&app, "Hide Revenue"));
    let app = App::memory();
    app.mount(chart("uncontrolled").data(data())).unwrap();
    let id = named(&app, "Hide Revenue");
    app.dispatch(id, Event::Activate).unwrap();
    assert_eq!(id, named(&app, "Show Revenue"));
    assert_eq!(count(&app, "sapid-chart-point"), 1);
    assert!(text(&app).contains("January"));
    app.dispatch(id, Event::Activate).unwrap();
    assert_eq!(count(&app, "sapid-chart-point"), 3);
}
#[test]
fn controlled_selection_emits_proposals_without_changing_details() {
    let app = App::memory();
    let out = Rc::new(RefCell::new(vec![]));
    let result = out.clone();
    app.mount(
        chart("selected")
            .data(data())
            .selection(None)
            .on_selection_change(move |v| result.borrow_mut().push(v)),
    )
    .unwrap();
    app.dispatch(named(&app, "Next data point"), Event::Activate)
        .unwrap();
    assert_eq!(
        out.borrow()[0],
        Some(ChartSelection {
            series: "category".into(),
            category: "head".into()
        })
    );
    assert!(text(&app).contains("No data point selected"));
}
#[test]
fn paired_data_and_options_admission_preserves_previous_table_on_invalid_update() {
    let app = App::memory();
    let data = app.scope().cage(data());
    let options = app.scope().cage(CartesianOptions::default());
    app.mount(chart("admission").data(data).options(options))
        .unwrap();
    let before = count(&app, "sapid-chart-point");
    options.revise(|o| o.hidden.insert("missing".into()));
    assert!(app.flush().is_err());
    assert_eq!(before, count(&app, "sapid-chart-point"));
    assert!(text(&app).contains("Revenue"));
    app.scope()
        .batch(|| {
            data.revise(|d| d.series[0].key = "missing".into());
            options.revise(|o| {
                o.hidden.clear();
            });
        })
        .unwrap();
    app.flush().unwrap();
    assert_eq!(count(&app, "sapid-chart-point"), 3);
}
#[test]
fn invalid_initial_data_is_rejected_before_mount_and_zero_data_has_truthful_empty_state() {
    let app = App::memory();
    let mut d = data();
    d.series[0].values[0] = Some(f64::NAN);
    assert!(app.mount(chart("bad").data(d)).is_err());
    let app = App::memory();
    app.mount(chart("empty")).unwrap();
    assert_eq!(count(&app, "sapid-chart-point"), 0);
    assert!(text(&app).contains("No visible chart data"));
}
#[test]
fn disabled_controls_emit_nothing_and_repeated_data_updates_release_old_rows() {
    let app = App::memory();
    let data = app.scope().cage(data());
    app.mount(chart("lifetime").enabled(false).data(data))
        .unwrap();
    app.dispatch(named(&app, "Next data point"), Event::Activate)
        .unwrap();
    assert!(text(&app).contains("No data point selected"));
    let initial = app.stats();
    for i in 0..40 {
        data.revise(|d| d.categories[0].label = format!("Month {i}"));
        app.flush().unwrap();
    }
    let final_stats = app.stats();
    assert_eq!(initial.nodes, final_stats.nodes);
}
#[test]
fn standalone_legend_has_real_buttons_color_swatches_and_controlled_proposals() {
    let app = App::memory();
    let proposals = Rc::new(RefCell::new(vec![]));
    let output = proposals.clone();
    app.mount(
        legend()
            .items(vec![LegendItem {
                key: "revenue".into(),
                label: "Revenue".into(),
                color: glory::style::Color(0, 128, 255, 255),
            }])
            .hidden(std::collections::BTreeSet::new())
            .on_hidden_change(move |v| output.borrow_mut().push(v)),
    )
    .unwrap();
    assert_eq!(count(&app, "sapid-chart-legend-swatch"), 1);
    app.dispatch(named(&app, "Hide Revenue"), Event::Activate)
        .unwrap();
    assert!(proposals.borrow()[0].contains("revenue"));
    assert!(all(&app).contains(&named(&app, "Hide Revenue")));
    let app = App::memory();
    assert!(
        app.mount(legend().items(vec![LegendItem {
            key: "x".into(),
            label: "Bad\rlabel".into(),
            color: glory::style::Color(0, 0, 0, 255)
        }]))
        .is_err()
    );
}
#[test]
fn reactive_button_section_minimal_reproduction() {
    let app = App::memory();
    app.mount(glory::ui::component(|scope| {
        let label = scope.cage("A".to_owned());
        sapid::button()
            .label(scope.bond(move || label.get()))
            .left_section(glory::ui::text().content("x"))
            .show_in(scope)
    }))
    .unwrap();
}
#[test]
fn chart_tooltip_uses_real_description_relations_and_no_point_disables_the_relation() {
    let app = App::memory();
    let point = app.scope().cage(Some(ChartPointDetails {
        selection: ChartSelection {
            series: "s".into(),
            category: "c".into(),
        },
        series_label: "Revenue".into(),
        category_label: "January".into(),
        value: 3.,
    }));
    app.mount(
        tooltip("tooltip")
            .point(point)
            .target(|target| target.button(glory::ui::button().label("Inspect data"))),
    )
    .unwrap();
    let id = named(&app, "Inspect data");
    assert!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .described_by
            .is_some()
    );
    app.dispatch(
        id,
        Event::FocusWithin {
            node: id,
            focused: true,
        },
    )
    .unwrap();
    assert!(text(&app).contains("January · Revenue: 3"));
    point.revise(|p| *p = None);
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
            .described_by
            .is_none()
    );
}
