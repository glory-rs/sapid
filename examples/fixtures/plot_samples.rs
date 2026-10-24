//! Shared finite fixture data; the host tests exercise the same public builders.
use sapid::charts::*;
pub const KINDS: [PlotKind; 18] = [
    PlotKind::Pie,
    PlotKind::Donut,
    PlotKind::RadialBar,
    PlotKind::Scatter,
    PlotKind::Bubble,
    PlotKind::Candlestick,
    PlotKind::Radar,
    PlotKind::Funnel,
    PlotKind::Sparkline,
    PlotKind::Heatmap,
    PlotKind::Matrix,
    PlotKind::Waffle,
    PlotKind::BarsList,
    PlotKind::Bullet,
    PlotKind::Gauge,
    PlotKind::Treemap,
    PlotKind::Sunburst,
    PlotKind::Sankey,
];
pub fn title(kind: PlotKind) -> &'static str {
    match kind {
        PlotKind::Pie => "Pie",
        PlotKind::Donut => "Donut",
        PlotKind::RadialBar => "Radial bar",
        PlotKind::Scatter => "Scatter",
        PlotKind::Bubble => "Bubble",
        PlotKind::Candlestick => "Candlestick",
        PlotKind::Radar => "Radar",
        PlotKind::Funnel => "Funnel",
        PlotKind::Sparkline => "Sparkline",
        PlotKind::Heatmap => "Heatmap",
        PlotKind::Matrix => "Matrix",
        PlotKind::Waffle => "Waffle",
        PlotKind::BarsList => "Bars list",
        PlotKind::Bullet => "Bullet",
        PlotKind::Gauge => "Gauge",
        PlotKind::Treemap => "Treemap",
        PlotKind::Sunburst => "Sunburst",
        PlotKind::Sankey => "Sankey",
    }
}
pub fn sample(kind: PlotKind) -> PlotData {
    match kind {
        PlotKind::Scatter | PlotKind::Bubble => PlotData::Points(vec![
            PointDatum {
                key: "one".into(),
                label: "North".into(),
                x: 1.,
                y: 3.,
                size: 1.,
            },
            PointDatum {
                key: "two".into(),
                label: "South".into(),
                x: 4.,
                y: 2.,
                size: 4.,
            },
            PointDatum {
                key: "three".into(),
                label: "West".into(),
                x: 2.,
                y: 5.,
                size: 9.,
            },
        ]),
        PlotKind::Candlestick => PlotData::Candles(vec![
            CandleDatum {
                key: "one".into(),
                label: "Monday".into(),
                open: 3.,
                high: 8.,
                low: 1.,
                close: 6.,
            },
            CandleDatum {
                key: "two".into(),
                label: "Tuesday".into(),
                open: 6.,
                high: 9.,
                low: 2.,
                close: 4.,
            },
            CandleDatum {
                key: "three".into(),
                label: "Wednesday".into(),
                open: 4.,
                high: 7.,
                low: 3.,
                close: 4.,
            },
        ]),
        PlotKind::Heatmap | PlotKind::Matrix => PlotData::Matrix(MatrixData {
            rows: vec![
                ChartCategory::new("north", "North"),
                ChartCategory::new("south", "South"),
            ],
            columns: vec![
                ChartCategory::new("mon", "Monday"),
                ChartCategory::new("tue", "Tuesday"),
            ],
            cells: [
                ("one", "north", "mon", 2.),
                ("two", "north", "tue", 8.),
                ("three", "south", "mon", -3.),
                ("four", "south", "tue", 0.),
            ]
            .into_iter()
            .map(|(key, row, column, value)| MatrixCell {
                key: key.into(),
                row: row.into(),
                column: column.into(),
                value,
            })
            .collect(),
        }),
        PlotKind::Bullet | PlotKind::Gauge => PlotData::Targets(vec![TargetDatum {
            key: "one".into(),
            label: "Capacity".into(),
            min: 0.,
            max: 100.,
            value: 42.,
            target: Some(75.),
            bands: vec![40., 70., 100.],
        }]),
        PlotKind::Treemap | PlotKind::Sunburst => PlotData::Hierarchy(vec![
            HierarchyDatum {
                key: "one".into(),
                label: "Regions".into(),
                parent: None,
                weight: None,
            },
            HierarchyDatum {
                key: "two".into(),
                label: "North".into(),
                parent: Some("one".into()),
                weight: Some(3.),
            },
            HierarchyDatum {
                key: "three".into(),
                label: "South".into(),
                parent: Some("one".into()),
                weight: Some(7.),
            },
        ]),
        PlotKind::Sankey => PlotData::Flows(FlowData {
            nodes: vec![
                ChartCategory::new("one", "Source"),
                ChartCategory::new("two", "Review"),
                ChartCategory::new("three", "Published"),
            ],
            links: vec![
                FlowLink {
                    key: "first".into(),
                    source: "one".into(),
                    target: "two".into(),
                    value: 8.,
                },
                FlowLink {
                    key: "second".into(),
                    source: "two".into(),
                    target: "three".into(),
                    value: 5.,
                },
            ],
        }),
        _ => PlotData::values([
            ValueDatum::new("one", "North", 3.),
            ValueDatum::new("two", "South", 7.),
            ValueDatum::new("three", "West", 5.),
        ]),
    }
}
pub fn empty(kind: PlotKind) -> PlotData {
    match kind {
        PlotKind::Scatter | PlotKind::Bubble => PlotData::Points(vec![]),
        PlotKind::Candlestick => PlotData::Candles(vec![]),
        PlotKind::Heatmap | PlotKind::Matrix => PlotData::Matrix(MatrixData::default()),
        PlotKind::Bullet | PlotKind::Gauge => PlotData::Targets(vec![]),
        PlotKind::Treemap | PlotKind::Sunburst => PlotData::Hierarchy(vec![]),
        PlotKind::Sankey => PlotData::Flows(FlowData::default()),
        _ => PlotData::default(),
    }
}
pub fn update(data: &mut PlotData) {
    match data {
        PlotData::Values(v) => {
            if let Some(r) = v.first_mut() {
                r.label = "Updated North".into();
                r.value = Some(4.);
            }
        }
        PlotData::Points(v) | PlotData::SampledPoints { points: v, .. } => {
            if let Some(r) = v.first_mut() {
                r.label = "Updated North".into();
                r.y = 4.;
            }
        }
        PlotData::Candles(v) => {
            if let Some(r) = v.first_mut() {
                r.label = "Updated Monday".into();
                r.close = 5.;
            }
        }
        PlotData::Matrix(v) => {
            if let Some(r) = v.cells.first_mut() {
                r.value = 4.;
            }
        }
        PlotData::Targets(v) => {
            if let Some(r) = v.first_mut() {
                r.value = 56.;
            }
        }
        PlotData::Hierarchy(v) => {
            if let Some(r) = v.first_mut() {
                r.label = "Updated Regions".into();
            }
            if let Some(r) = v.get_mut(1) {
                r.weight = Some(4.);
            }
        }
        PlotData::Flows(v) => {
            if let Some(r) = v.nodes.first_mut() {
                r.label = "Updated Source".into();
            }
            if let Some(r) = v.links.first_mut() {
                r.value = 9.;
            }
        }
    }
}
