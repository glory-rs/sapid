//! Portable geometry for the finite typed plot records.
use crate::charts::{ChartDomain, ChartError, ChartFrame, ChartPath, ChartTheme, LegendItem};
use crate::plot_model::*;
use glory::render::{
    GraphicPoint as Point, GraphicProps, GraphicStroke, PathCommand as Cmd, ProgressValue,
    StrokeCap,
};
use glory::style::Color;
use std::{
    collections::BTreeSet,
    f64::consts::{FRAC_PI_2, PI, TAU},
};
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlotOptions {
    pub hidden: BTreeSet<String>,
    pub domain: Option<ChartDomain>,
    pub x_domain: Option<ChartDomain>,
}
pub(crate) fn path_key(key: &str, part: &str) -> String {
    format!("record-{}_{key}-{}_{part}", key.len(), part.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_layers_do_not_collide_with_valid_business_keys() {
        let data = PlotData::values(
            [
                "track",
                "axes",
                "radar",
                "a-track",
                "a-band-0",
                "record-1_a-5_shape",
            ]
            .into_iter()
            .map(|k| ValueDatum::new(k, k, 1.)),
        );
        for kind in [
            PlotKind::Pie,
            PlotKind::Donut,
            PlotKind::RadialBar,
            PlotKind::Radar,
            PlotKind::Waffle,
        ] {
            let scene = build(&data, kind);
            let keys = scene.paths.iter().map(|p| &p.key).collect::<BTreeSet<_>>();
            assert_eq!(keys.len(), scene.paths.len(), "{kind:?}");
        }
    }
    fn theme() -> ChartTheme {
        ChartTheme::from_theme(&crate::Theme::green_dark())
    }
    fn values() -> PlotData {
        PlotData::values([
            ValueDatum::new("one", "One", 1.),
            ValueDatum::new("two", "Two", 3.),
            ValueDatum::new("three", "Three", 2.),
        ])
    }
    fn build(data: &PlotData, kind: PlotKind) -> PlotScene {
        PlotScene::build(
            data,
            kind,
            ChartFrame::default(),
            &PlotOptions::default(),
            &theme(),
        )
        .unwrap()
    }
    fn sample(kind: PlotKind) -> PlotData {
        match kind {
            PlotKind::Scatter | PlotKind::Bubble => PlotData::Points(vec![
                PointDatum {
                    key: "a".into(),
                    label: "A".into(),
                    x: 1.,
                    y: 2.,
                    size: 1.,
                },
                PointDatum {
                    key: "b".into(),
                    label: "B".into(),
                    x: 3.,
                    y: 4.,
                    size: 4.,
                },
            ]),
            PlotKind::Candlestick => PlotData::Candles(vec![CandleDatum {
                key: "a".into(),
                label: "A".into(),
                open: 2.,
                high: 4.,
                low: 1.,
                close: 3.,
            }]),
            PlotKind::Heatmap | PlotKind::Matrix => PlotData::Matrix(MatrixData {
                rows: vec![crate::charts::ChartCategory::new("r", "Row")],
                columns: vec![
                    crate::charts::ChartCategory::new("a", "A"),
                    crate::charts::ChartCategory::new("b", "B"),
                ],
                cells: vec![
                    MatrixCell {
                        key: "one".into(),
                        row: "r".into(),
                        column: "a".into(),
                        value: 1.,
                    },
                    MatrixCell {
                        key: "two".into(),
                        row: "r".into(),
                        column: "b".into(),
                        value: 4.,
                    },
                ],
            }),
            PlotKind::Bullet | PlotKind::Gauge => PlotData::Targets(vec![TargetDatum {
                key: "a".into(),
                label: "A".into(),
                min: 0.,
                max: 100.,
                value: 30.,
                target: Some(70.),
                bands: vec![40., 80., 100.],
            }]),
            PlotKind::Treemap | PlotKind::Sunburst => PlotData::Hierarchy(vec![
                HierarchyDatum {
                    key: "root".into(),
                    label: "Root".into(),
                    parent: None,
                    weight: None,
                },
                HierarchyDatum {
                    key: "one".into(),
                    label: "One".into(),
                    parent: Some("root".into()),
                    weight: Some(1.),
                },
                HierarchyDatum {
                    key: "two".into(),
                    label: "Two".into(),
                    parent: Some("root".into()),
                    weight: Some(3.),
                },
            ]),
            PlotKind::Sankey => PlotData::Flows(FlowData {
                nodes: vec![
                    crate::charts::ChartCategory::new("a", "A"),
                    crate::charts::ChartCategory::new("b", "B"),
                    crate::charts::ChartCategory::new("c", "C"),
                ],
                links: vec![
                    FlowLink {
                        key: "ab".into(),
                        source: "a".into(),
                        target: "b".into(),
                        value: 2.,
                    },
                    FlowLink {
                        key: "bc".into(),
                        source: "b".into(),
                        target: "c".into(),
                        value: 2.,
                    },
                ],
            }),
            _ => values(),
        }
    }
    const KINDS: [PlotKind; 18] = [
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
    #[test]
    fn all_eighteen_kinds_generate_distinct_valid_geometry_and_finite_current_marks() {
        for kind in KINDS {
            let scene = build(&sample(kind), kind);
            assert!(!scene.paths.is_empty(), "{kind:?}");
            assert!(!scene.marks.is_empty(), "{kind:?}");
            for path in &scene.paths {
                path.drawing.validate().unwrap();
                assert!(path.drawing.alternative.is_none());
            }
            for mark in &scene.marks {
                assert!(mark.x.is_finite() && mark.y.is_finite());
                assert!(
                    (0. ..=640.).contains(&mark.x) && (0. ..=320.).contains(&mark.y),
                    "{kind:?} {mark:?}"
                );
            }
            for range in &scene.ranges {
                range.value.validate().unwrap();
            }
        }
        assert_ne!(
            build(&values(), PlotKind::Pie).paths,
            build(&values(), PlotKind::Donut).paths
        );
        assert_ne!(
            build(&sample(PlotKind::Scatter), PlotKind::Scatter).paths,
            build(&sample(PlotKind::Bubble), PlotKind::Bubble).paths
        );
        assert_ne!(
            build(&sample(PlotKind::Matrix), PlotKind::Matrix).paths,
            build(&sample(PlotKind::Heatmap), PlotKind::Heatmap).paths
        );
    }
    #[test]
    fn pie_angles_and_zero_totals_do_not_invent_percentages_or_slices() {
        let data = PlotData::values([
            ValueDatum::new("a", "A", 1.),
            ValueDatum::new("b", "B", 3.),
            ValueDatum::new("zero", "Zero", 0.),
        ]);
        let scene = build(&data, PlotKind::Pie);
        assert_eq!(scene.paths.len(), 2);
        assert!(scene.marks[0].value_text.contains("25%"));
        assert!(scene.marks[1].value_text.contains("75%"));
        for kind in [PlotKind::Pie, PlotKind::Donut, PlotKind::Waffle] {
            let scene = build(
                &PlotData::values([ValueDatum::new("zero", "Zero", 0.)]),
                kind,
            );
            assert!(scene.paths.is_empty() && scene.marks.is_empty());
        }
    }
    #[test]
    fn waffle_uses_exactly_one_hundred_cells_and_deterministic_remainders() {
        let data =
            PlotData::values((0..3).map(|i| ValueDatum::new(format!("k{i}"), "Equal", 1.)));
        let scene = build(&data, PlotKind::Waffle);
        assert_eq!(
            scene
                .paths
                .iter()
                .flat_map(|p| &p.drawing.commands)
                .filter(|c| matches!(c, Cmd::Close))
                .count(),
            100
        );
        assert!(scene.marks[0].value_text.contains("34 of 100"));
        assert!(scene.marks[1].value_text.contains("33 of 100"));
    }
    #[test]
    fn bubble_area_tracks_size_instead_of_radius_and_zero_size_has_no_fake_circle() {
        let data = sample(PlotKind::Bubble);
        let scene = build(&data, PlotKind::Bubble);
        let radius = |i: usize| {
            let Cmd::Move(p) = scene.paths[i].drawing.commands[0] else {
                panic!()
            };
            p.x - scene.marks[i].x
        };
        assert!((radius(0) - 12.).abs() < 1e-9);
        assert!((radius(1) - 24.).abs() < 1e-9);
        let PlotData::Points(mut points) = data else {
            panic!()
        };
        points[0].size = 0.;
        assert_eq!(
            build(&PlotData::Points(points), PlotKind::Bubble)
                .marks
                .len(),
            1
        );
    }
    #[test]
    fn sparkline_nulls_create_true_disconnected_paths_and_hidden_keys_are_checked() {
        let data = PlotData::Values(vec![
            ValueDatum::new("a", "A", 1.),
            ValueDatum {
                key: "gap".into(),
                label: "Gap".into(),
                value: None,
            },
            ValueDatum::new("b", "B", 2.),
        ]);
        let scene = build(&data, PlotKind::Sparkline);
        assert_eq!(
            scene.paths[0]
                .drawing
                .commands
                .iter()
                .filter(|c| matches!(c, Cmd::Move(_)))
                .count(),
            2
        );
        assert_eq!(scene.marks.len(), 2);
        assert_eq!(
            PlotScene::build(
                &data,
                PlotKind::Sparkline,
                ChartFrame::default(),
                &PlotOptions {
                    hidden: BTreeSet::from(["unknown".into()]),
                    ..Default::default()
                },
                &theme()
            ),
            Err(ChartError::Key)
        );
    }
    #[test]
    fn explicit_point_domains_clip_and_disclose_values_while_tiny_domains_reject_overflow() {
        let data = sample(PlotKind::Scatter);
        let options = PlotOptions {
            x_domain: Some(ChartDomain::new(0., 2.).unwrap()),
            domain: Some(ChartDomain::new(0., 3.).unwrap()),
            ..Default::default()
        };
        let scene = PlotScene::build(
            &data,
            PlotKind::Scatter,
            ChartFrame::default(),
            &options,
            &theme(),
        )
        .unwrap();
        assert_eq!(scene.clipped_values, 1);
        assert_eq!(scene.marks[1].x, 608.);
        assert_eq!(scene.marks[1].y, 28.);
        let options = PlotOptions {
            x_domain: Some(ChartDomain::new(0., f64::from_bits(1)).unwrap()),
            ..Default::default()
        };
        assert!(
            PlotScene::build(
                &data,
                PlotKind::Scatter,
                ChartFrame::default(),
                &options,
                &theme()
            )
            .is_err()
        );
    }
    #[test]
    fn treemap_parent_weights_and_sunburst_bands_follow_only_visible_descendants() {
        let data = sample(PlotKind::Treemap);
        let scene = build(&data, PlotKind::Treemap);
        assert!(scene.marks[0].value_text.contains('4'));
        assert_eq!(scene.marks.len(), 3);
        for kind in [PlotKind::Treemap, PlotKind::Sunburst] {
            let scene = PlotScene::build(
                &data,
                kind,
                ChartFrame::default(),
                &PlotOptions {
                    hidden: BTreeSet::from(["root".into()]),
                    ..Default::default()
                },
                &theme(),
            )
            .unwrap();
            assert!(scene.paths.is_empty() && scene.marks.is_empty());
        }
        let scene = PlotScene::build(
            &data,
            PlotKind::Treemap,
            ChartFrame::default(),
            &PlotOptions {
                hidden: BTreeSet::from(["two".into()]),
                ..Default::default()
            },
            &theme(),
        )
        .unwrap();
        assert!(scene.marks[0].value_text.contains('1'));
        assert_eq!(scene.marks.len(), 2);
    }
    #[test]
    fn sankey_preserves_port_thickness_across_stages_and_omits_zero_flow_ribbons() {
        let data = sample(PlotKind::Sankey);
        let scene = build(&data, PlotKind::Sankey);
        let edges = scene
            .paths
            .iter()
            .filter(|p| p.key.starts_with("edge-"))
            .collect::<Vec<_>>();
        assert_eq!(edges.len(), 2);
        let thickness = |p: &ChartPath| {
            let Cmd::Move(a) = p.drawing.commands[0] else {
                panic!()
            };
            let Cmd::Line(b) = p.drawing.commands[2] else {
                panic!()
            };
            b.y - a.y
        };
        assert!((thickness(edges[0]) - thickness(edges[1])).abs() < 1e-9);
        let PlotData::Flows(mut data) = data else {
            panic!()
        };
        for e in &mut data.links {
            e.value = 0.;
        }
        let scene = build(&PlotData::Flows(data), PlotKind::Sankey);
        assert!(scene.paths.iter().all(|p| p.key.starts_with("node-")));
        assert_eq!(scene.marks.len(), 3);
    }
    #[test]
    fn maximum_matrix_and_dense_zero_graph_remain_within_protocol_and_frame_budgets() {
        let axes = (0..32)
            .map(|i| crate::charts::ChartCategory::new(format!("k{i}"), format!("Axis {i}")))
            .collect::<Vec<_>>();
        let data = PlotData::Matrix(MatrixData {
            rows: axes.clone(),
            columns: axes,
            cells: (0..1024)
                .map(|i| MatrixCell {
                    key: format!("c{i}"),
                    row: format!("k{}", i / 32),
                    column: format!("k{}", i % 32),
                    value: i as f64,
                })
                .collect(),
        });
        for kind in [PlotKind::Matrix, PlotKind::Heatmap] {
            let scene = build(&data, kind);
            assert_eq!(scene.marks.len(), 1024);
            assert!(scene.paths.len() <= 1024);
            for path in scene.paths {
                path.drawing.validate().unwrap();
            }
        }
        let data = PlotData::Flows(FlowData {
            nodes: (0..128)
                .map(|i| crate::charts::ChartCategory::new(format!("n{i}"), "Node"))
                .collect(),
            links: Vec::new(),
        });
        let scene = build(&data, PlotKind::Sankey);
        assert_eq!(scene.marks.len(), 128);
        assert!(scene.marks.iter().all(|m| m.y <= 320.));
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct PlotMark {
    pub key: String,
    pub label: String,
    pub value_text: String,
    pub x: f64,
    pub y: f64,
    pub color: Color,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PlotLabel {
    pub text: String,
    pub x: f64,
    pub y: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PlotScaleStop {
    pub value: f64,
    pub color: Color,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PlotRange {
    pub key: String,
    pub label: String,
    pub value: ProgressValue,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PlotScene {
    pub frame: ChartFrame,
    pub paths: Vec<ChartPath>,
    pub marks: Vec<PlotMark>,
    pub labels: Vec<PlotLabel>,
    pub scale_legend: Vec<PlotScaleStop>,
    pub ranges: Vec<PlotRange>,
    pub legend: Vec<LegendItem>,
    pub clipped_values: usize,
}
pub(crate) fn rect(x: f64, y: f64, w: f64, h: f64) -> Vec<Cmd> {
    vec![
        Cmd::Move(Point::new(x, y)),
        Cmd::Line(Point::new(x + w, y)),
        Cmd::Line(Point::new(x + w, y + h)),
        Cmd::Line(Point::new(x, y + h)),
        Cmd::Close,
    ]
}
fn arc(out: &mut Vec<Cmd>, cx: f64, cy: f64, r: f64, start: f64, sweep: f64) {
    let segments = (sweep.abs() / FRAC_PI_2).ceil().max(1.) as usize;
    let step = sweep / segments as f64;
    for i in 0..segments {
        let a = start + i as f64 * step;
        let b = a + step;
        let k = 4. / 3. * libm::tan(step / 4.);
        out.push(Cmd::Cubic {
            first: Point::new(
                cx + r * (libm::cos(a) - k * libm::sin(a)),
                cy + r * (libm::sin(a) + k * libm::cos(a)),
            ),
            second: Point::new(
                cx + r * (libm::cos(b) + k * libm::sin(b)),
                cy + r * (libm::sin(b) - k * libm::cos(b)),
            ),
            end: Point::new(cx + r * libm::cos(b), cy + r * libm::sin(b)),
        });
    }
}
pub(crate) fn ring(
    cx: f64,
    cy: f64,
    inner: f64,
    outer: f64,
    start: f64,
    sweep: f64,
) -> Vec<Cmd> {
    if sweep <= 0. || outer <= 0. {
        return Vec::new();
    }
    let mut out = vec![Cmd::Move(Point::new(
        cx + outer * libm::cos(start),
        cy + outer * libm::sin(start),
    ))];
    arc(&mut out, cx, cy, outer, start, sweep);
    let end = start + sweep;
    out.push(Cmd::Line(Point::new(
        cx + inner * libm::cos(end),
        cy + inner * libm::sin(end),
    )));
    if inner > 0. {
        arc(&mut out, cx, cy, inner, end, -sweep);
    }
    out.push(Cmd::Close);
    out
}
pub(crate) fn stroke(color: Color, width: f64) -> GraphicStroke {
    GraphicStroke {
        color,
        width,
        cap: StrokeCap::Round,
    }
}
fn scale(values: impl Iterator<Item = f64>) -> Result<ChartDomain, ChartError> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for v in values {
        min = min.min(v);
        max = max.max(v);
    }
    if !min.is_finite() {
        return ChartDomain::new(0., 1.);
    }
    if min == max {
        let pad = (min.abs() * 0.05).max(1.);
        min -= pad;
        max += pad;
    }
    ChartDomain::new(min, max)
}
fn color_mix(a: Color, b: Color, fraction: f64) -> Color {
    let f = fraction.clamp(0., 1.);
    let c = |a: u8, b: u8| ((f64::from(a) * (1. - f) + f64::from(b) * f).round()) as u8;
    Color(c(a.0, b.0), c(a.1, b.1), c(a.2, b.2), 255)
}
impl PlotScene {
    pub(crate) fn layer(
        &mut self,
        key: impl Into<String>,
        commands: Vec<Cmd>,
        fill: Option<Color>,
        stroke: Option<GraphicStroke>,
    ) -> Result<(), ChartError> {
        if self.paths.len() >= 1024 {
            return Err(ChartError::Budget);
        }
        let drawing = GraphicProps {
            width: self.frame.width,
            height: self.frame.height,
            commands,
            fill,
            stroke,
            ..Default::default()
        };
        drawing.validate().map_err(|_| ChartError::Geometry)?;
        self.paths.push(ChartPath {
            key: key.into(),
            series: None,
            drawing,
        });
        Ok(())
    }
    pub(crate) fn mark(
        &mut self,
        key: &str,
        label: &str,
        value_text: String,
        x: f64,
        y: f64,
        color: Color,
    ) -> Result<(), ChartError> {
        if self.marks.len() >= MAX_MATRIX_CELLS
            || ![x, y].into_iter().all(|v| v.is_finite() && v.abs() <= 1e6)
            || value_text.len() > 768
        {
            return Err(ChartError::Geometry);
        }
        self.marks.push(PlotMark {
            key: key.into(),
            label: label.into(),
            value_text,
            x,
            y,
            color,
        });
        Ok(())
    }
    pub fn build(
        data: &PlotData,
        kind: PlotKind,
        frame: ChartFrame,
        options: &PlotOptions,
        theme: &ChartTheme,
    ) -> Result<Self, ChartError> {
        frame.validate()?;
        data.validate(kind)?;
        if options.hidden.len() > MAX_MATRIX_CELLS {
            return Err(ChartError::Budget);
        }
        let keys = data.keys().into_iter().collect::<BTreeSet<_>>();
        if options
            .hidden
            .iter()
            .any(|key| !keys.contains(key.as_str()))
        {
            return Err(ChartError::Key);
        }
        if options.x_domain.is_some() && !matches!(kind, PlotKind::Scatter | PlotKind::Bubble) {
            return Err(ChartError::Domain);
        }
        if options.domain.is_some()
            && !matches!(
                kind,
                PlotKind::Scatter
                    | PlotKind::Bubble
                    | PlotKind::Candlestick
                    | PlotKind::Sparkline
                    | PlotKind::Radar
                    | PlotKind::RadialBar
                    | PlotKind::Heatmap
                    | PlotKind::Matrix
                    | PlotKind::BarsList
            )
        {
            return Err(ChartError::Domain);
        }
        let mut scene = Self {
            frame,
            paths: Vec::new(),
            marks: Vec::new(),
            labels: Vec::new(),
            scale_legend: Vec::new(),
            ranges: Vec::new(),
            legend: Vec::new(),
            clipped_values: 0,
        };
        match data {
            PlotData::Values(values) => scene.values(values, kind, options, theme)?,
            PlotData::Points(values) | PlotData::SampledPoints { points: values, .. } => {
                scene.points(values, kind, options, theme)?
            }
            PlotData::Candles(values) => scene.candles(values, options, theme)?,
            PlotData::Matrix(values) => scene.matrix(values, kind, options, theme)?,
            PlotData::Targets(values) => scene.targets(values, kind, options, theme)?,
            PlotData::Hierarchy(values) => {
                crate::plot_hierarchy::hierarchy(&mut scene, values, kind, options, theme)?
            }
            PlotData::Flows(values) => {
                crate::plot_hierarchy::flows(&mut scene, values, options, theme)?
            }
        }
        let mut paths = BTreeSet::new();
        let mut marks = BTreeSet::new();
        if scene.paths.iter().any(|p| !paths.insert(&p.key))
            || scene.marks.iter().any(|m| !marks.insert(&m.key))
        {
            return Err(ChartError::Key);
        }
        Ok(scene)
    }
    fn values(
        &mut self,
        values: &[ValueDatum],
        kind: PlotKind,
        options: &PlotOptions,
        theme: &ChartTheme,
    ) -> Result<(), ChartError> {
        let current = values
            .iter()
            .enumerate()
            .filter(|(_, r)| !options.hidden.contains(&r.key))
            .collect::<Vec<_>>();
        for (i, r) in values.iter().enumerate() {
            self.legend.push(LegendItem {
                key: r.key.clone(),
                label: r.label.clone(),
                color: theme.series[i % 8],
            });
        }
        let (w, h) = (self.frame.width, self.frame.height);
        let (cx, cy, radius) = (w / 2., h / 2., (w.min(h) - 40.) / 2.);
        match kind {
            PlotKind::Pie | PlotKind::Donut => {
                let total = current.iter().filter_map(|(_, r)| r.value).sum::<f64>();
                let mut start = -FRAC_PI_2;
                if total > 0. {
                    for (i, r) in &current {
                        let Some(value) = r.value else {
                            continue;
                        };
                        let sweep = value / total * TAU;
                        if sweep <= 0. {
                            continue;
                        }
                        let inner = if kind == PlotKind::Donut {
                            radius * 0.62
                        } else {
                            0.
                        };
                        let color = theme.series[i % 8];
                        self.layer(
                            path_key(&r.key, "shape"),
                            ring(cx, cy, inner, radius, start, sweep),
                            Some(color),
                            Some(stroke(theme.grid, 1.)),
                        )?;
                        let angle = start + sweep / 2.;
                        let middle = (inner + radius) / 2.;
                        self.mark(
                            &r.key,
                            &r.label,
                            format!("{value} ({}%)", (value / total * 10000.).round() / 100.),
                            cx + middle * libm::cos(angle),
                            cy + middle * libm::sin(angle),
                            color,
                        )?;
                        start += sweep;
                    }
                }
            }
            PlotKind::RadialBar => {
                let max = current
                    .iter()
                    .filter_map(|(_, r)| r.value)
                    .fold(0f64, f64::max)
                    .max(1.);
                let domain = options.domain.unwrap_or(ChartDomain::new(0., max)?);
                if domain.min() != 0. {
                    return Err(ChartError::Domain);
                }
                let thickness = radius / current.len().max(1) as f64;
                for (row, (i, r)) in current.iter().enumerate() {
                    let Some(value) = r.value else {
                        continue;
                    };
                    let outer = radius - row as f64 * thickness;
                    let inner = (outer - thickness * 0.75).max(0.);
                    let fraction = domain.fraction(value)?.clamp(0., 1.);
                    if value > domain.max() {
                        self.clipped_values += 1;
                    }
                    self.layer(
                        path_key(&r.key, "track"),
                        ring(cx, cy, inner, outer, -FRAC_PI_2, TAU),
                        Some(theme.grid),
                        None,
                    )?;
                    self.layer(
                        path_key(&r.key, "shape"),
                        ring(cx, cy, inner, outer, -FRAC_PI_2, TAU * fraction),
                        Some(theme.series[i % 8]),
                        None,
                    )?;
                    let angle = -FRAC_PI_2 + TAU * fraction;
                    self.mark(
                        &r.key,
                        &r.label,
                        value.to_string(),
                        cx + (inner + outer) / 2. * libm::cos(angle),
                        cy + (inner + outer) / 2. * libm::sin(angle),
                        theme.series[i % 8],
                    )?;
                }
            }
            PlotKind::Radar => {
                let max = current
                    .iter()
                    .filter_map(|(_, r)| r.value)
                    .fold(0f64, f64::max)
                    .max(1.);
                let domain = options.domain.unwrap_or(ChartDomain::new(0., max)?);
                if domain.min() != 0. {
                    return Err(ChartError::Domain);
                }
                let mut axes = Vec::new();
                let mut polygon = Vec::new();
                let mut open = false;
                let mut gaps = false;
                for (index, (i, r)) in current.iter().enumerate() {
                    let angle = -FRAC_PI_2 + TAU * index as f64 / current.len().max(1) as f64;
                    axes.extend([
                        Cmd::Move(Point::new(cx, cy)),
                        Cmd::Line(Point::new(
                            cx + radius * libm::cos(angle),
                            cy + radius * libm::sin(angle),
                        )),
                    ]);
                    self.labels.push(PlotLabel {
                        text: r.label.clone(),
                        x: cx + (radius + 8.) * libm::cos(angle),
                        y: cy + (radius + 8.) * libm::sin(angle),
                    });
                    if let Some(value) = r.value {
                        let fraction = domain.fraction(value)?.clamp(0., 1.);
                        if value > domain.max() {
                            self.clipped_values += 1;
                        }
                        let (x, y) = (
                            cx + radius * fraction * libm::cos(angle),
                            cy + radius * fraction * libm::sin(angle),
                        );
                        polygon.push(if open {
                            Cmd::Line(Point::new(x, y))
                        } else {
                            Cmd::Move(Point::new(x, y))
                        });
                        open = true;
                        self.mark(
                            &r.key,
                            &r.label,
                            value.to_string(),
                            x,
                            y,
                            theme.series[i % 8],
                        )?;
                    } else {
                        open = false;
                        gaps = true;
                    }
                }
                self.layer("axes", axes, None, Some(stroke(theme.grid, 1.)))?;
                if !polygon.is_empty() {
                    let mut color = theme.series[0];
                    color.3 = 80;
                    if !gaps {
                        polygon.push(Cmd::Close);
                    }
                    self.layer(
                        "radar",
                        polygon,
                        (!gaps).then_some(color),
                        Some(stroke(theme.series[0], 2.)),
                    )?;
                }
            }
            PlotKind::Funnel => {
                let max = current
                    .iter()
                    .filter_map(|(_, r)| r.value)
                    .fold(0f64, f64::max)
                    .max(1.);
                let height = (h - 32.) / current.len().max(1) as f64;
                for (row, (i, r)) in current.iter().enumerate() {
                    let Some(value) = r.value else {
                        continue;
                    };
                    let top = (w - 48.) * value / max;
                    let next = current
                        .get(row + 1)
                        .and_then(|(_, r)| r.value)
                        .unwrap_or(value);
                    let bottom = (w - 48.) * next / max;
                    let y = 16. + row as f64 * height;
                    let color = theme.series[i % 8];
                    self.layer(
                        path_key(&r.key, "shape"),
                        vec![
                            Cmd::Move(Point::new(cx - top / 2., y)),
                            Cmd::Line(Point::new(cx + top / 2., y)),
                            Cmd::Line(Point::new(cx + bottom / 2., y + height * 0.85)),
                            Cmd::Line(Point::new(cx - bottom / 2., y + height * 0.85)),
                            Cmd::Close,
                        ],
                        Some(color),
                        None,
                    )?;
                    self.mark(
                        &r.key,
                        &r.label,
                        value.to_string(),
                        cx,
                        y + height / 2.,
                        color,
                    )?;
                }
            }
            PlotKind::Sparkline => {
                let domain = options
                    .domain
                    .unwrap_or(scale(current.iter().filter_map(|(_, r)| r.value))?);
                let mut drawing = Vec::new();
                let mut open = false;
                for (index, (i, r)) in current.iter().enumerate() {
                    if let Some(value) = r.value {
                        let fraction = domain.fraction(value)?.clamp(0., 1.);
                        if value < domain.min() || value > domain.max() {
                            self.clipped_values += 1;
                        }
                        let x = 16.
                            + (w - 32.) * index as f64
                                / (current.len().saturating_sub(1)).max(1) as f64;
                        let y = h - 16. - (h - 32.) * fraction;
                        drawing.push(if open {
                            Cmd::Line(Point::new(x, y))
                        } else {
                            Cmd::Move(Point::new(x, y))
                        });
                        open = true;
                        self.mark(
                            &r.key,
                            &r.label,
                            value.to_string(),
                            x,
                            y,
                            theme.series[i % 8],
                        )?;
                    } else {
                        open = false;
                    }
                }
                if !drawing.is_empty() {
                    self.layer(
                        "sparkline",
                        drawing,
                        None,
                        Some(stroke(theme.series[0], 2.)),
                    )?;
                }
            }
            PlotKind::Waffle => {
                let total = current.iter().filter_map(|(_, r)| r.value).sum::<f64>();
                if total <= 0. {
                    return Ok(());
                }
                let mut allocations = current
                    .iter()
                    .map(|(i, r)| {
                        let units = r.value.unwrap_or(0.) / total * 100.;
                        (*i, units.floor() as usize, units - units.floor())
                    })
                    .collect::<Vec<_>>();
                let assigned: usize = allocations.iter().map(|(_, n, _)| n).sum();
                let mut order = (0..allocations.len()).collect::<Vec<_>>();
                order.sort_by(|a, b| {
                    allocations[*b]
                        .2
                        .total_cmp(&allocations[*a].2)
                        .then(a.cmp(b))
                });
                for i in order.into_iter().take(100usize.saturating_sub(assigned)) {
                    allocations[i].1 += 1;
                }
                let cell = (w.min(h) - 32.) / 10.;
                let left = cx - cell * 5.;
                let top = cy - cell * 5.;
                let mut next = 0;
                for (i, count, _) in allocations {
                    let r = &values[i];
                    let mut commands = Vec::new();
                    let first = next;
                    for _ in 0..count {
                        let col = next % 10;
                        let row = next / 10;
                        if commands.len() + 5 > 256 {
                            self.layer(
                                path_key(&r.key, &format!("cells-{next}")),
                                std::mem::take(&mut commands),
                                Some(theme.series[i % 8]),
                                None,
                            )?;
                        }
                        commands.extend(rect(
                            left + col as f64 * cell,
                            top + row as f64 * cell,
                            cell - 2.,
                            cell - 2.,
                        ));
                        next += 1;
                    }
                    if !commands.is_empty() {
                        self.layer(
                            path_key(&r.key, "shape"),
                            commands,
                            Some(theme.series[i % 8]),
                            None,
                        )?;
                        self.mark(
                            &r.key,
                            &r.label,
                            format!("{}; {count} of 100 cells", r.value.unwrap_or(0.)),
                            left + (first % 10) as f64 * cell + cell / 2.,
                            top + (first / 10) as f64 * cell + cell / 2.,
                            theme.series[i % 8],
                        )?;
                    }
                }
            }
            PlotKind::BarsList => {
                let natural = scale(current.iter().filter_map(|(_, r)| r.value).chain([0.]))?;
                let domain = options.domain.unwrap_or(natural);
                let x = |v: f64| -> Result<f64, ChartError> {
                    Ok(24. + domain.fraction(v)?.clamp(0., 1.) * (w - 48.))
                };
                let height = (h - 24.) / current.len().max(1) as f64;
                for (row, (i, r)) in current.iter().enumerate() {
                    let Some(value) = r.value else {
                        continue;
                    };
                    let start = x(0.)?;
                    let end = x(value)?;
                    let y = 12. + row as f64 * height;
                    if value < domain.min() || value > domain.max() {
                        self.clipped_values += 1;
                    }
                    self.layer(
                        path_key(&r.key, "shape"),
                        rect(start.min(end), y, (end - start).abs(), height * 0.72),
                        Some(theme.series[i % 8]),
                        None,
                    )?;
                    self.mark(
                        &r.key,
                        &r.label,
                        value.to_string(),
                        end,
                        y + height * 0.36,
                        theme.series[i % 8],
                    )?;
                    self.ranges.push(PlotRange {
                        key: r.key.clone(),
                        label: r.label.clone(),
                        value: ProgressValue {
                            min: domain.min(),
                            max: domain.max(),
                            current: Some(value.clamp(domain.min(), domain.max())),
                            value_text: Some(value.to_string()),
                        },
                        x: 24.,
                        y,
                        width: w - 48.,
                        height: height * 0.72,
                    });
                }
            }
            _ => return Err(ChartError::Schema),
        }
        Ok(())
    }
    fn points(
        &mut self,
        values: &[PointDatum],
        kind: PlotKind,
        options: &PlotOptions,
        theme: &ChartTheme,
    ) -> Result<(), ChartError> {
        let current = values
            .iter()
            .enumerate()
            .filter(|(_, r)| !options.hidden.contains(&r.key))
            .collect::<Vec<_>>();
        let xd = options
            .x_domain
            .unwrap_or(scale(current.iter().map(|(_, r)| r.x))?);
        let yd = options
            .domain
            .unwrap_or(scale(current.iter().map(|(_, r)| r.y))?);
        let maximum = current
            .iter()
            .map(|(_, r)| r.size)
            .fold(0f64, f64::max)
            .max(f64::MIN_POSITIVE);
        let (w, h) = (self.frame.width, self.frame.height);
        for (i, r) in current {
            let x = 32. + xd.fraction(r.x)?.clamp(0., 1.) * (w - 64.);
            let y = h - 28. - yd.fraction(r.y)?.clamp(0., 1.) * (h - 56.);
            let radius = if kind == PlotKind::Bubble {
                (r.size / maximum).sqrt() * 24.
            } else {
                4.
            };
            if r.x < xd.min() || r.x > xd.max() || r.y < yd.min() || r.y > yd.max() {
                self.clipped_values += 1;
            }
            if radius > 0. {
                self.layer(
                    path_key(&r.key, "shape"),
                    ring(x, y, 0., radius, 0., TAU),
                    Some(theme.series[i % 8]),
                    None,
                )?;
                self.mark(
                    &r.key,
                    &r.label,
                    format!("x={}, y={}, size={}", r.x, r.y, r.size),
                    x,
                    y,
                    theme.series[i % 8],
                )?;
            }
        }
        self.labels.extend([
            PlotLabel {
                text: format!("x: {} .. {}", xd.min(), xd.max()),
                x: 32.,
                y: h - 20.,
            },
            PlotLabel {
                text: format!("y: {} .. {}", yd.min(), yd.max()),
                x: 32.,
                y: 8.,
            },
        ]);
        Ok(())
    }
    fn candles(
        &mut self,
        values: &[CandleDatum],
        options: &PlotOptions,
        theme: &ChartTheme,
    ) -> Result<(), ChartError> {
        let current = values
            .iter()
            .filter(|r| !options.hidden.contains(&r.key))
            .collect::<Vec<_>>();
        let domain = options
            .domain
            .unwrap_or(scale(current.iter().flat_map(|r| [r.low, r.high]))?);
        let (w, h) = (self.frame.width, self.frame.height);
        let band = (w - 48.) / current.len().max(1) as f64;
        let y = |v: f64| -> Result<f64, ChartError> {
            Ok(h - 24. - domain.fraction(v)?.clamp(0., 1.) * (h - 48.))
        };
        for (index, r) in current.into_iter().enumerate() {
            let x = 24. + band * (index as f64 + 0.5);
            let color = theme.series[if r.close >= r.open { 2 } else { 1 }];
            let width = band * 0.6;
            let mut commands = vec![
                Cmd::Move(Point::new(x, y(r.high)?)),
                Cmd::Line(Point::new(x, y(r.low)?)),
            ];
            commands.extend(rect(
                x - width / 2.,
                y(r.open)?.min(y(r.close)?),
                width,
                (y(r.close)? - y(r.open)?).abs().max(1.),
            ));
            self.layer(
                path_key(&r.key, "shape"),
                commands,
                Some(color),
                Some(stroke(color, 1.)),
            )?;
            self.mark(
                &r.key,
                &r.label,
                format!(
                    "open {}, high {}, low {}, close {}",
                    r.open, r.high, r.low, r.close
                ),
                x,
                y(r.close)?,
                color,
            )?;
            if r.low < domain.min() || r.high > domain.max() {
                self.clipped_values += 1;
            }
        }
        Ok(())
    }
    fn matrix(
        &mut self,
        data: &MatrixData,
        kind: PlotKind,
        options: &PlotOptions,
        theme: &ChartTheme,
    ) -> Result<(), ChartError> {
        let current = data
            .cells
            .iter()
            .filter(|c| !options.hidden.contains(&c.key))
            .collect::<Vec<_>>();
        let domain = options
            .domain
            .unwrap_or(scale(current.iter().map(|r| r.value))?);
        for i in 0..5 {
            let fraction = f64::from(i) / 4.;
            self.scale_legend.push(PlotScaleStop {
                value: domain.min() + (domain.max() - domain.min()) * fraction,
                color: color_mix(theme.series[0], theme.series[1], fraction),
            });
        }
        let width = (self.frame.width - 48.) / data.columns.len().max(1) as f64;
        let height = (self.frame.height - 48.) / data.rows.len().max(1) as f64;
        let maximum = current
            .iter()
            .map(|r| r.value.abs())
            .fold(0f64, f64::max)
            .max(f64::MIN_POSITIVE);
        for c in current {
            let row = data
                .rows
                .iter()
                .position(|r| r.key == c.row)
                .expect("admitted row");
            let col = data
                .columns
                .iter()
                .position(|r| r.key == c.column)
                .expect("admitted column");
            let x = 24. + (col as f64 + 0.5) * width;
            let y = 24. + (row as f64 + 0.5) * height;
            let fraction = domain.fraction(c.value)?.clamp(0., 1.);
            let color = color_mix(theme.series[0], theme.series[1], fraction);
            let scale = if kind == PlotKind::Matrix {
                (c.value.abs() / maximum).sqrt()
            } else {
                1.
            };
            if scale > 0. {
                self.layer(
                    path_key(&c.key, "shape"),
                    rect(
                        x - width * scale / 2.,
                        y - height * scale / 2.,
                        (width * scale - 2.).max(0.),
                        (height * scale - 2.).max(0.),
                    ),
                    Some(color),
                    None,
                )?;
            }
            self.mark(
                &c.key,
                &format!("{} · {}", data.rows[row].label, data.columns[col].label),
                c.value.to_string(),
                x,
                y,
                color,
            )?;
            self.labels.push(PlotLabel {
                text: c.value.to_string(),
                x: x - width / 3.,
                y: y - 6.,
            });
            if c.value < domain.min() || c.value > domain.max() {
                self.clipped_values += 1;
            }
        }
        Ok(())
    }
    fn targets(
        &mut self,
        values: &[TargetDatum],
        kind: PlotKind,
        options: &PlotOptions,
        theme: &ChartTheme,
    ) -> Result<(), ChartError> {
        let current = values
            .iter()
            .filter(|r| !options.hidden.contains(&r.key))
            .collect::<Vec<_>>();
        let (w, h) = (self.frame.width, self.frame.height);
        let row_height = (h - 32.) / current.len().max(1) as f64;
        for (index, r) in current.iter().enumerate() {
            let domain = ChartDomain::new(r.min, r.max)?;
            let fraction = domain.fraction(r.value)?;
            let top = 16. + index as f64 * row_height;
            self.labels.extend([
                PlotLabel {
                    text: r.min.to_string(),
                    x: 24.,
                    y: top + row_height * 0.78,
                },
                PlotLabel {
                    text: r.max.to_string(),
                    x: w - 80.,
                    y: top + row_height * 0.78,
                },
            ]);
            if kind == PlotKind::Gauge {
                let radius = (w.min(h) - 48.) / 2.;
                let (cx, cy) = (w / 2., h / 2.);
                let inner = radius * 0.78;
                self.layer(
                    "track",
                    ring(cx, cy, inner, radius, PI, PI),
                    Some(theme.grid),
                    None,
                )?;
                self.layer(
                    path_key(&r.key, "shape"),
                    ring(cx, cy, inner, radius, PI, PI * fraction),
                    Some(theme.series[0]),
                    None,
                )?;
                if let Some(target) = r.target {
                    let angle = PI + PI * domain.fraction(target)?;
                    self.layer(
                        "target",
                        vec![
                            Cmd::Move(Point::new(
                                cx + inner * libm::cos(angle),
                                cy + inner * libm::sin(angle),
                            )),
                            Cmd::Line(Point::new(
                                cx + (radius + 5.) * libm::cos(angle),
                                cy + (radius + 5.) * libm::sin(angle),
                            )),
                        ],
                        None,
                        Some(stroke(theme.series[1], 3.)),
                    )?;
                }
                self.mark(
                    &r.key,
                    &r.label,
                    format!(
                        "{} in {} .. {}; target {}",
                        r.value,
                        r.min,
                        r.max,
                        r.target.map_or_else(|| "none".into(), |v| v.to_string())
                    ),
                    cx,
                    cy,
                    theme.series[0],
                )?;
                self.labels.push(PlotLabel {
                    text: format!("{} / {}", r.value, r.max),
                    x: cx - 32.,
                    y: cy + 16.,
                });
            } else {
                let mut previous = r.min;
                for (band, end) in r.bands.iter().enumerate() {
                    let start = domain.fraction(previous)?;
                    let end_fraction = domain.fraction(*end)?;
                    let mut color = theme.series[band % 8];
                    color.3 = 80;
                    self.layer(
                        path_key(&r.key, &format!("band-{band}")),
                        rect(
                            24. + start * (w - 48.),
                            top,
                            (end_fraction - start) * (w - 48.),
                            row_height * 0.75,
                        ),
                        Some(color),
                        None,
                    )?;
                    previous = *end;
                }
                self.layer(
                    path_key(&r.key, "shape"),
                    rect(
                        24.,
                        top + row_height * 0.2,
                        fraction * (w - 48.),
                        row_height * 0.35,
                    ),
                    Some(theme.series[0]),
                    None,
                )?;
                if let Some(target) = r.target {
                    let x = 24. + domain.fraction(target)? * (w - 48.);
                    self.layer(
                        path_key(&r.key, "target"),
                        vec![
                            Cmd::Move(Point::new(x, top)),
                            Cmd::Line(Point::new(x, top + row_height * 0.75)),
                        ],
                        None,
                        Some(stroke(theme.series[1], 3.)),
                    )?;
                }
                self.mark(
                    &r.key,
                    &r.label,
                    format!(
                        "{} in {} .. {}; target {}",
                        r.value,
                        r.min,
                        r.max,
                        r.target.map_or_else(|| "none".into(), |v| v.to_string())
                    ),
                    24. + fraction * (w - 48.),
                    top + row_height * 0.375,
                    theme.series[0],
                )?;
            }
            self.ranges.push(PlotRange {
                key: r.key.clone(),
                label: r.label.clone(),
                value: ProgressValue {
                    min: r.min,
                    max: r.max,
                    current: Some(r.value),
                    value_text: Some(format!(
                        "{}; target {}",
                        r.value,
                        r.target.map_or_else(|| "none".into(), |v| v.to_string())
                    )),
                },
                x: 24.,
                y: if kind == PlotKind::Gauge { 16. } else { top },
                width: w - 48.,
                height: if kind == PlotKind::Gauge {
                    h - 32.
                } else {
                    row_height * 0.75
                },
            });
        }
        Ok(())
    }
}
