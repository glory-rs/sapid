//! Portable finite geometry for Cartesian charts. Every layer is one existing
//! validated Graphic, shared by HTML/SVG and native rendering.
use crate::chart_model::*;
use glory::render::{
    GraphicPoint as Point, GraphicProps, GraphicStroke, PathCommand as Cmd, StrokeCap,
};
use glory::style::Color;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CartesianKind {
    #[default]
    Line,
    Area,
    Bar,
    Composite,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartFrame {
    pub width: f64,
    pub height: f64,
}
impl Default for ChartFrame {
    fn default() -> Self {
        Self {
            width: 640.,
            height: 320.,
        }
    }
}
impl ChartFrame {
    pub fn validate(self) -> Result<(), ChartError> {
        if !self.width.is_finite()
            || !self.height.is_finite()
            || !(160.0..=16384.0).contains(&self.width)
            || !(120.0..=16384.0).contains(&self.height)
        {
            return Err(ChartError::Geometry);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ChartTheme {
    pub series: [Color; MAX_CHART_SERIES],
    pub grid: Color,
}
impl ChartTheme {
    pub fn from_theme(theme: &crate::Theme) -> Self {
        Self {
            series: theme.series,
            grid: theme.border,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CartesianOptions {
    pub kind: CartesianKind,
    pub stacked: bool,
    pub hidden: BTreeSet<String>,
    pub brush: Option<ChartBrush>,
    pub domain: Option<ChartDomain>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ChartPath {
    pub key: String,
    pub series: Option<usize>,
    pub drawing: GraphicProps,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartLocation {
    pub series: usize,
    pub category: usize,
    pub value: f64,
    pub x: f64,
    pub y: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartTick {
    pub value: f64,
    pub x: f64,
    pub y: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ChartLabel {
    pub category: usize,
    pub x: f64,
    pub y: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CartesianScene {
    pub frame: ChartFrame,
    pub domain: ChartDomain,
    pub paths: Vec<ChartPath>,
    pub locations: Vec<ChartLocation>,
    pub ticks: Vec<ChartTick>,
    pub labels: Vec<ChartLabel>,
    pub clipped_values: usize,
}
fn path(
    frame: ChartFrame,
    key: String,
    series: Option<usize>,
    commands: Vec<Cmd>,
    fill: Option<Color>,
    stroke: Option<GraphicStroke>,
) -> Result<ChartPath, ChartError> {
    let drawing = GraphicProps {
        width: frame.width,
        height: frame.height,
        commands,
        fill,
        stroke,
        ..Default::default()
    };
    drawing.validate().map_err(|_| ChartError::Geometry)?;
    Ok(ChartPath {
        key,
        series,
        drawing,
    })
}
fn stroke(color: Color, width: f64) -> GraphicStroke {
    GraphicStroke {
        color,
        width,
        cap: StrokeCap::Round,
    }
}
fn rectangle(x: f64, y: f64, width: f64, height: f64) -> [Cmd; 5] {
    [
        Cmd::Move(Point::new(x, y)),
        Cmd::Line(Point::new(x + width, y)),
        Cmd::Line(Point::new(x + width, y + height)),
        Cmd::Line(Point::new(x, y + height)),
        Cmd::Close,
    ]
}
impl CartesianScene {
    /// Export the finite drawing layers. Labels and exact values are exported
    /// separately in `ticks`, `labels` and the widget's intrinsic data table.
    /// This geometry export does not claim a captured browser/native frame.
    pub fn to_svg(&self) -> Result<String, ChartError> {
        use std::fmt::Write;
        self.frame.validate()?;
        if self.paths.len() > MAX_CHART_SERIES * 3 + 1 {
            return Err(ChartError::Budget);
        }
        for path in &self.paths {
            path.drawing.validate().map_err(|_| ChartError::Geometry)?;
            if path.drawing.width != self.frame.width
                || path.drawing.height != self.frame.height
                || path.drawing.animation.is_some()
                || path.drawing.transition.is_some()
            {
                return Err(ChartError::Geometry);
            }
        }
        let mut svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\">",
            self.frame.width, self.frame.height
        );
        let color =
            |c: Color| format!("rgba({},{},{},{})", c.0, c.1, c.2, f64::from(c.3) / 255.);
        for path in &self.paths {
            svg.push_str("<path d=\"");
            for command in &path.drawing.commands {
                match command {
                    Cmd::Move(p) => write!(svg, "M{} {}", p.x, p.y),
                    Cmd::Line(p) => write!(svg, "L{} {}", p.x, p.y),
                    Cmd::Cubic { first, second, end } => write!(
                        svg,
                        "C{} {} {} {} {} {}",
                        first.x, first.y, second.x, second.y, end.x, end.y
                    ),
                    Cmd::Close => write!(svg, "Z"),
                }
                .expect("String writing");
            }
            write!(
                svg,
                "\" fill=\"{}\"",
                path.drawing.fill.map_or_else(|| "none".into(), color)
            )
            .expect("String writing");
            if let Some(stroke) = path.drawing.stroke {
                let cap = match stroke.cap {
                    StrokeCap::Butt => "butt",
                    StrokeCap::Round => "round",
                    StrokeCap::Square => "square",
                };
                write!(
                    svg,
                    " stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"{}\"",
                    color(stroke.color),
                    stroke.width,
                    cap
                )
                .expect("String writing");
            }
            svg.push_str("/>");
        }
        svg.push_str("</svg>");
        Ok(svg)
    }
    pub fn build(
        data: &ChartData,
        frame: ChartFrame,
        options: &CartesianOptions,
        theme: &ChartTheme,
    ) -> Result<Self, ChartError> {
        frame.validate()?;
        data.validate()?;
        // Validate hidden keys even when the caller supplies an explicit domain.
        let natural = data.domain(options.stacked, &options.hidden)?;
        let domain = options.domain.unwrap_or(natural);
        if let Some(brush) = options.brush {
            brush.validate(data.categories.len())?;
        }
        let first = options.brush.map_or(0, |brush| brush.first);
        let end = options
            .brush
            .map_or(data.categories.len(), |brush| brush.last + 1);
        let count = end - first;
        let left = 52.;
        let top = 12.;
        let right = frame.width - 16.;
        let bottom = frame.height - 36.;
        let y = |value: f64| -> Result<f64, ChartError> {
            Ok(bottom - domain.fraction(value)?.clamp(0., 1.) * (bottom - top))
        };
        let band = (right - left) / count.max(1) as f64;
        let x = |index: usize| left + (index - first) as f64 * band + band / 2.;
        let ticks = domain
            .ticks(5)?
            .into_iter()
            .map(|value| {
                Ok(ChartTick {
                    value,
                    x: left - 6.,
                    y: y(value)?,
                })
            })
            .collect::<Result<Vec<_>, ChartError>>()?;
        let mut grid = Vec::new();
        for tick in &ticks {
            grid.extend([
                Cmd::Move(Point::new(left, tick.y)),
                Cmd::Line(Point::new(right, tick.y)),
            ]);
        }
        let mut paths = vec![path(
            frame,
            "grid".into(),
            None,
            grid,
            None,
            Some(stroke(theme.grid, 1.)),
        )?];
        let labels = (first..end)
            .filter(|index| {
                (*index - first).is_multiple_of(count.div_ceil(8).max(1)) || *index + 1 == end
            })
            .map(|category| ChartLabel {
                category,
                x: x(category),
                y: bottom + 8.,
            })
            .collect();
        let mut locations = Vec::new();
        let mut clipped_values = 0;
        let mut positive = vec![0.; data.categories.len()];
        let mut negative = positive.clone();
        let visible = data
            .series
            .iter()
            .enumerate()
            .filter(|(_, series)| !options.hidden.contains(&series.key))
            .collect::<Vec<_>>();
        let bar_count = visible
            .iter()
            .filter(|(_, series)| match options.kind {
                CartesianKind::Bar => true,
                CartesianKind::Composite => series.kind == ChartSeriesKind::Bar,
                _ => false,
            })
            .count()
            .max(1);
        let mut bar_index = 0;
        for (series_index, series) in visible {
            let kind = match options.kind {
                CartesianKind::Line => ChartSeriesKind::Line,
                CartesianKind::Area => ChartSeriesKind::Area,
                CartesianKind::Bar => ChartSeriesKind::Bar,
                CartesianKind::Composite => series.kind,
            };
            // Signed stacks are well-defined for bars. Filled continuous areas
            // can cross zero only on an unstacked baseline in this first model.
            if options.stacked
                && kind == ChartSeriesKind::Area
                && series.values.iter().flatten().any(|v| *v < 0.)
            {
                return Err(ChartError::SignedAreaStack);
            }
            let mut commands = Vec::new();
            let mut batches = Vec::new();
            let mut run = Vec::new();
            let mut spans = Vec::new();
            for category in first..end {
                let Some(value) = series.values[category] else {
                    if !run.is_empty() {
                        spans.push(std::mem::take(&mut run));
                    }
                    continue;
                };
                let baseline = if options.stacked {
                    let slot = if value >= 0. {
                        &mut positive[category]
                    } else {
                        &mut negative[category]
                    };
                    let old = *slot;
                    *slot += value;
                    old
                } else {
                    0.
                };
                let total = baseline + value;
                let py = y(total)?;
                let px = x(category);
                if total < domain.min() || total > domain.max() {
                    clipped_values += 1;
                }
                let location = ChartLocation {
                    series: series_index,
                    category,
                    value,
                    x: px,
                    y: py,
                };
                if kind == ChartSeriesKind::Bar {
                    let width = band * 0.72
                        / if options.stacked {
                            1.
                        } else {
                            bar_count as f64
                        };
                    let bx = if options.stacked {
                        px - width / 2.
                    } else {
                        px - band * 0.36 + bar_index as f64 * width
                    };
                    locations.push(ChartLocation {
                        x: bx + width / 2.,
                        ..location
                    });
                    if commands.len() + 5 > 256 {
                        batches.push(std::mem::take(&mut commands));
                    }
                    commands.extend(rectangle(
                        bx,
                        py.min(y(baseline)?),
                        width,
                        (py - y(baseline)?).abs(),
                    ));
                } else {
                    locations.push(location);
                    run.push((px, py, y(baseline)?));
                }
            }
            if !run.is_empty() {
                spans.push(run);
            }
            for span in spans {
                let mut start = 0;
                loop {
                    let end = if kind == ChartSeriesKind::Area {
                        (start + 127).min(span.len())
                    } else {
                        span.len()
                    };
                    let slice = &span[start..end];
                    let size = if kind == ChartSeriesKind::Area {
                        slice.len() * 2 + 1
                    } else {
                        slice.len()
                    };
                    if commands.len() + size > 256 {
                        batches.push(std::mem::take(&mut commands));
                    }
                    commands.push(Cmd::Move(Point::new(slice[0].0, slice[0].1)));
                    commands.extend(
                        slice
                            .iter()
                            .skip(1)
                            .map(|(x, y, _)| Cmd::Line(Point::new(*x, *y))),
                    );
                    if kind == ChartSeriesKind::Area {
                        commands.extend(
                            slice
                                .iter()
                                .rev()
                                .map(|(x, _, baseline)| Cmd::Line(Point::new(*x, *baseline))),
                        );
                        commands.push(Cmd::Close);
                    }
                    if end == span.len() {
                        break;
                    }
                    start = end - 1;
                }
            }
            if !commands.is_empty() {
                batches.push(commands);
            }
            let mut fill = theme.series[series_index];
            if kind == ChartSeriesKind::Area {
                fill.3 = 96;
            }
            for (batch, commands) in batches.into_iter().enumerate() {
                paths.push(path(
                    frame,
                    format!("{}-{batch}", series.key),
                    Some(series_index),
                    commands,
                    (kind != ChartSeriesKind::Line).then_some(fill),
                    (kind != ChartSeriesKind::Bar)
                        .then_some(stroke(theme.series[series_index], 2.)),
                )?);
            }
            if kind == ChartSeriesKind::Bar {
                bar_index += 1;
            }
        }
        Ok(Self {
            frame,
            domain,
            paths,
            locations,
            ticks,
            labels,
            clipped_values,
        })
    }
    pub fn nearest(&self, x: f64, y: f64) -> Option<ChartLocation> {
        if !x.is_finite() || !y.is_finite() || x.abs() > 1_000_000. || y.abs() > 1_000_000. {
            return None;
        }
        self.locations
            .iter()
            .min_by(|a, b| {
                let da = (a.x - x).powi(2) + (a.y - y).powi(2);
                let db = (b.x - x).powi(2) + (b.y - y).powi(2);
                da.total_cmp(&db)
            })
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data(count: usize, series: usize) -> ChartData {
        ChartData {
            categories: (0..count)
                .map(|i| ChartCategory::new(format!("c{i}"), format!("Category {i}")))
                .collect(),
            series: (0..series)
                .map(|s| {
                    ChartSeries::new(
                        format!("s{s}"),
                        format!("Series {s}"),
                        (0..count)
                            .map(|i| Some((i as f64 - count as f64 / 2.) * (s + 1) as f64)),
                    )
                })
                .collect(),
        }
    }
    fn build(
        data: &ChartData,
        options: CartesianOptions,
    ) -> Result<CartesianScene, ChartError> {
        CartesianScene::build(
            data,
            ChartFrame::default(),
            &options,
            &ChartTheme::from_theme(&crate::Theme::default()),
        )
    }
    #[test]
    fn every_cartesian_mode_retains_finite_protocol_budgets_at_maximum_data() {
        let mut d = data(128, 8);
        for kind in [
            CartesianKind::Line,
            CartesianKind::Area,
            CartesianKind::Bar,
            CartesianKind::Composite,
        ] {
            for stacked in [false, true] {
                if stacked && kind == CartesianKind::Area {
                    for s in &mut d.series {
                        for v in s.values.iter_mut().flatten() {
                            *v = v.abs();
                        }
                    }
                }
                let scene = build(
                    &d,
                    CartesianOptions {
                        kind,
                        stacked,
                        ..Default::default()
                    },
                )
                .unwrap();
                assert_eq!(scene.locations.len(), 1024);
                assert!(scene.paths.len() <= 33);
                assert!(scene.labels.len() <= 9);
                for layer in scene.paths {
                    layer.drawing.validate().unwrap();
                    assert!(layer.drawing.commands.len() <= 256);
                }
            }
        }
    }
    #[test]
    fn gaps_split_line_and_filled_area_without_joining_missing_categories() {
        let mut d = data(5, 1);
        d.series[0].values = vec![Some(1.), Some(2.), None, Some(3.), Some(1.)];
        for kind in [CartesianKind::Line, CartesianKind::Area] {
            let scene = build(
                &d,
                CartesianOptions {
                    kind,
                    ..Default::default()
                },
            )
            .unwrap();
            let drawing = &scene.paths[1].drawing;
            assert_eq!(
                drawing
                    .commands
                    .iter()
                    .filter(|c| matches!(c, Cmd::Move(_)))
                    .count(),
                2
            );
            assert_eq!(
                drawing
                    .commands
                    .iter()
                    .filter(|c| matches!(c, Cmd::Close))
                    .count(),
                if kind == CartesianKind::Area { 2 } else { 0 }
            );
            assert_eq!(
                scene
                    .locations
                    .iter()
                    .map(|p| p.category)
                    .collect::<Vec<_>>(),
                [0, 1, 3, 4]
            );
        }
    }
    #[test]
    fn grouped_bar_hit_locations_match_their_actual_distinct_centers() {
        let d = data(1, 2);
        let scene = build(
            &d,
            CartesianOptions {
                kind: CartesianKind::Bar,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(scene.locations[0].x < scene.locations[1].x);
        for p in &scene.locations {
            assert_eq!(scene.nearest(p.x, p.y), Some(*p));
        }
        assert!(scene.nearest(f64::NAN, 0.).is_none());
        assert!(scene.nearest(1e7, 0.).is_none());
    }
    #[test]
    fn signed_bars_stack_separately_and_negative_area_stack_is_explicitly_rejected() {
        let mut d = data(1, 4);
        for (s, v) in d.series.iter_mut().zip([-2., -3., 4., 1.]) {
            s.values = vec![Some(v)];
        }
        let scene = build(
            &d,
            CartesianOptions {
                kind: CartesianKind::Bar,
                stacked: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(scene.domain, ChartDomain::new(-5., 5.).unwrap());
        assert!(scene.locations[1].y > scene.locations[0].y);
        assert!(scene.locations[3].y < scene.locations[2].y);
        assert_eq!(
            build(
                &d,
                CartesianOptions {
                    kind: CartesianKind::Area,
                    stacked: true,
                    ..Default::default()
                }
            ),
            Err(ChartError::SignedAreaStack)
        );
    }
    #[test]
    fn brush_changes_only_current_geometry_hidden_series_and_explicit_domain_are_validated() {
        let d = data(6, 2);
        let scene = build(
            &d,
            CartesianOptions {
                brush: Some(ChartBrush { first: 2, last: 3 }),
                hidden: BTreeSet::from(["s1".into()]),
                domain: Some(ChartDomain::new(0., 0.5).unwrap()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(scene.locations.len(), 2);
        assert_eq!(
            scene
                .locations
                .iter()
                .map(|p| p.category)
                .collect::<Vec<_>>(),
            [2, 3]
        );
        assert_eq!(scene.clipped_values, 1);
        assert!(scene.locations.iter().all(|p| (12. ..=284.).contains(&p.y)));
        assert_eq!(
            build(
                &d,
                CartesianOptions {
                    hidden: BTreeSet::from(["unknown".into()]),
                    domain: Some(ChartDomain::new(0., 1.).unwrap()),
                    ..Default::default()
                }
            ),
            Err(ChartError::Key)
        );
        assert_eq!(
            build(
                &d,
                CartesianOptions {
                    brush: Some(ChartBrush { first: 4, last: 6 }),
                    ..Default::default()
                }
            ),
            Err(ChartError::Brush)
        );
    }
    #[test]
    fn empty_data_and_finite_frame_rejection_never_generate_nan() {
        let empty = build(&ChartData::default(), CartesianOptions::default()).unwrap();
        assert!(empty.locations.is_empty() && empty.labels.is_empty());
        assert_eq!(empty.paths.len(), 1);
        for frame in [
            ChartFrame {
                width: f64::INFINITY,
                height: 320.,
            },
            ChartFrame {
                width: 159.,
                height: 320.,
            },
            ChartFrame {
                width: 640.,
                height: 119.,
            },
        ] {
            assert_eq!(
                CartesianScene::build(
                    &data(1, 1),
                    frame,
                    &CartesianOptions::default(),
                    &ChartTheme::from_theme(&crate::Theme::default())
                ),
                Err(ChartError::Geometry)
            );
        }
    }
    #[test]
    fn svg_export_validates_mutated_public_geometry_and_contains_no_business_markup() {
        let mut d = data(3, 1);
        d.series[0].label = "<script>untrusted</script>".into();
        let mut scene = build(&d, CartesianOptions::default()).unwrap();
        let svg = scene.to_svg().unwrap();
        assert!(svg.starts_with("<svg xmlns="));
        assert!(!svg.contains("script"));
        assert!(svg.contains("<path d=\"M"));
        scene.paths[0]
            .drawing
            .commands
            .push(Cmd::Line(Point::new(f64::NAN, 0.)));
        assert_eq!(scene.to_svg(), Err(ChartError::Geometry));
    }
}
