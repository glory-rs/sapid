//! Finite typed records for the non-Cartesian chart families. All admission
//! reads the supplied current snapshot; it never visits a business data source.
use crate::chart_model::{self, ChartCategory, ChartError, MAX_CHART_VALUE};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_PLOT_RECORDS: usize = 128;
pub const MAX_MATRIX_AXIS: usize = 32;
pub const MAX_MATRIX_CELLS: usize = 1024;
pub const MAX_FLOW_LINKS: usize = 256;
pub const MAX_PLOT_DEPTH: usize = 32;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlotKind {
    #[default]
    Pie,
    Donut,
    RadialBar,
    Scatter,
    Bubble,
    Candlestick,
    Radar,
    Funnel,
    Sparkline,
    Heatmap,
    Matrix,
    Waffle,
    BarsList,
    Bullet,
    Gauge,
    Treemap,
    Sunburst,
    Sankey,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ValueDatum {
    pub key: String,
    pub label: String,
    pub value: Option<f64>,
}
impl ValueDatum {
    pub fn new(key: impl Into<String>, label: impl Into<String>, value: f64) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value: Some(value),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct PointDatum {
    pub key: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub size: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CandleDatum {
    pub key: String,
    pub label: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MatrixCell {
    pub key: String,
    pub row: String,
    pub column: String,
    pub value: f64,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MatrixData {
    pub rows: Vec<ChartCategory>,
    pub columns: Vec<ChartCategory>,
    pub cells: Vec<MatrixCell>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TargetDatum {
    pub key: String,
    pub label: String,
    pub min: f64,
    pub max: f64,
    pub value: f64,
    pub target: Option<f64>,
    pub bands: Vec<f64>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct HierarchyDatum {
    pub key: String,
    pub label: String,
    pub parent: Option<String>,
    pub weight: Option<f64>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct FlowLink {
    pub key: String,
    pub source: String,
    pub target: String,
    pub value: f64,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlowData {
    pub nodes: Vec<ChartCategory>,
    pub links: Vec<FlowLink>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum PlotData {
    Values(Vec<ValueDatum>),
    Points(Vec<PointDatum>),
    /// Explicit bounded uniform sampling. Unsampled source records are never read.
    SampledPoints {
        points: Vec<PointDatum>,
        source_count: usize,
        indices: Vec<usize>,
    },
    Candles(Vec<CandleDatum>),
    Matrix(MatrixData),
    Targets(Vec<TargetDatum>),
    Hierarchy(Vec<HierarchyDatum>),
    Flows(FlowData),
}
impl Default for PlotData {
    fn default() -> Self {
        Self::Values(Vec::new())
    }
}
fn value(value: f64) -> Result<(), ChartError> {
    if value.is_finite() && value.abs() <= MAX_CHART_VALUE {
        Ok(())
    } else {
        Err(ChartError::Values)
    }
}
fn nonnegative(v: f64) -> Result<(), ChartError> {
    value(v)?;
    if v < 0. {
        Err(ChartError::Negative)
    } else {
        Ok(())
    }
}
fn record<'a>(
    keys: &mut BTreeSet<&'a str>,
    key: &'a str,
    label: &str,
) -> Result<(), ChartError> {
    if !chart_model::key(key) || !keys.insert(key) {
        return Err(ChartError::Key);
    }
    if !chart_model::label(label) {
        return Err(ChartError::Label);
    }
    Ok(())
}
fn count(count: usize, max: usize) -> Result<(), ChartError> {
    if count > max {
        Err(ChartError::Budget)
    } else {
        Ok(())
    }
}
impl PlotData {
    /// Samples at most `budget` indexed records, including both endpoints when
    /// at least two records are requested. The callback runs only for that sample.
    pub fn sample_points(
        source_count: usize,
        budget: usize,
        mut point_at: impl FnMut(usize) -> PointDatum,
    ) -> Result<Self, ChartError> {
        if budget == 0 || budget > MAX_PLOT_RECORDS {
            return Err(ChartError::Budget);
        }
        let count = source_count.min(budget);
        let indices = (0..count)
            .map(|i| {
                if count <= 1 {
                    0
                } else {
                    ((i as u128 * (source_count - 1) as u128) / (count - 1) as u128) as usize
                }
            })
            .collect::<Vec<_>>();
        let points = indices.iter().map(|i| point_at(*i)).collect();
        let data = Self::SampledPoints {
            points,
            source_count,
            indices,
        };
        data.validate(PlotKind::Scatter)?;
        Ok(data)
    }
    pub fn values(values: impl IntoIterator<Item = ValueDatum>) -> Self {
        Self::Values(values.into_iter().take(MAX_PLOT_RECORDS + 1).collect())
    }
    pub fn validate(&self, kind: PlotKind) -> Result<(), ChartError> {
        let mut keys = BTreeSet::new();
        match self {
            Self::Values(values) => {
                if !matches!(
                    kind,
                    PlotKind::Pie
                        | PlotKind::Donut
                        | PlotKind::RadialBar
                        | PlotKind::Radar
                        | PlotKind::Funnel
                        | PlotKind::Sparkline
                        | PlotKind::Waffle
                        | PlotKind::BarsList
                ) {
                    return Err(ChartError::Schema);
                }
                let maximum = match kind {
                    PlotKind::RadialBar => 16,
                    PlotKind::Radar | PlotKind::BarsList => 32,
                    _ => MAX_PLOT_RECORDS,
                };
                count(values.len(), maximum)?;
                if kind == PlotKind::Radar && !values.is_empty() && values.len() < 3 {
                    return Err(ChartError::Schema);
                }
                for r in values {
                    record(&mut keys, &r.key, &r.label)?;
                    if let Some(v) = r.value {
                        value(v)?;
                        if !matches!(kind, PlotKind::Sparkline | PlotKind::BarsList) {
                            nonnegative(v)?;
                        }
                    }
                }
            }
            Self::Points(values) | Self::SampledPoints { points: values, .. } => {
                if !matches!(kind, PlotKind::Scatter | PlotKind::Bubble) {
                    return Err(ChartError::Schema);
                }
                count(values.len(), MAX_PLOT_RECORDS)?;
                if let Self::SampledPoints {
                    source_count,
                    indices,
                    ..
                } = self
                    && (values.len() > *source_count
                        || indices.len() != values.len()
                        || (*source_count > 0 && values.is_empty())
                        || indices.iter().any(|i| *i >= *source_count)
                        || indices.windows(2).any(|p| p[0] >= p[1]))
                {
                    return Err(ChartError::Schema);
                }
                for r in values {
                    record(&mut keys, &r.key, &r.label)?;
                    value(r.x)?;
                    value(r.y)?;
                    nonnegative(r.size)?;
                }
            }
            Self::Candles(values) => {
                if kind != PlotKind::Candlestick {
                    return Err(ChartError::Schema);
                }
                count(values.len(), MAX_PLOT_RECORDS)?;
                for r in values {
                    record(&mut keys, &r.key, &r.label)?;
                    for v in [r.open, r.high, r.low, r.close] {
                        value(v)?;
                    }
                    if r.low > r.open.min(r.close)
                        || r.high < r.open.max(r.close)
                        || r.low > r.high
                    {
                        return Err(ChartError::Schema);
                    }
                }
            }
            Self::Matrix(data) => {
                if !matches!(kind, PlotKind::Heatmap | PlotKind::Matrix) {
                    return Err(ChartError::Schema);
                }
                count(data.rows.len(), MAX_MATRIX_AXIS)?;
                count(data.columns.len(), MAX_MATRIX_AXIS)?;
                count(data.cells.len(), MAX_MATRIX_CELLS)?;
                let mut rows = BTreeSet::new();
                for row in &data.rows {
                    record(&mut rows, &row.key, &row.label)?;
                }
                let mut columns = BTreeSet::new();
                for c in &data.columns {
                    record(&mut columns, &c.key, &c.label)?;
                }
                let mut positions = BTreeSet::new();
                for c in &data.cells {
                    if !chart_model::key(&c.key)
                        || !keys.insert(c.key.as_str())
                        || !rows.contains(c.row.as_str())
                        || !columns.contains(c.column.as_str())
                        || !positions.insert((&c.row, &c.column))
                    {
                        return Err(ChartError::Key);
                    }
                    value(c.value)?;
                }
            }
            Self::Targets(values) => {
                if !matches!(kind, PlotKind::Bullet | PlotKind::Gauge) {
                    return Err(ChartError::Schema);
                }
                count(values.len(), if kind == PlotKind::Gauge { 1 } else { 16 })?;
                for r in values {
                    record(&mut keys, &r.key, &r.label)?;
                    count(r.bands.len(), 8)?;
                    for v in [r.min, r.max, r.value] {
                        value(v)?;
                    }
                    if r.min >= r.max || r.value < r.min || r.value > r.max {
                        return Err(ChartError::Domain);
                    }
                    if let Some(t) = r.target {
                        value(t)?;
                        if t < r.min || t > r.max {
                            return Err(ChartError::Domain);
                        }
                    }
                    let mut previous = r.min;
                    for end in &r.bands {
                        value(*end)?;
                        if *end <= previous || *end > r.max {
                            return Err(ChartError::Domain);
                        }
                        previous = *end;
                    }
                }
            }
            Self::Hierarchy(values) => {
                if !matches!(kind, PlotKind::Treemap | PlotKind::Sunburst) {
                    return Err(ChartError::Schema);
                }
                count(values.len(), MAX_PLOT_RECORDS)?;
                for r in values {
                    record(&mut keys, &r.key, &r.label)?;
                    if let Some(v) = r.weight {
                        nonnegative(v)?;
                    }
                }
                let indices = values
                    .iter()
                    .enumerate()
                    .map(|(i, r)| (r.key.as_str(), i))
                    .collect::<BTreeMap<_, _>>();
                let mut children = vec![0; values.len()];
                for r in values {
                    if let Some(parent) = &r.parent {
                        let Some(i) = indices.get(parent.as_str()) else {
                            return Err(ChartError::Hierarchy);
                        };
                        children[*i] += 1;
                    }
                }
                for (i, r) in values.iter().enumerate() {
                    if (children[i] > 0) == r.weight.is_some() {
                        return Err(ChartError::Hierarchy);
                    }
                    let mut cursor = Some(i);
                    let mut seen = BTreeSet::new();
                    while let Some(index) = cursor {
                        if !seen.insert(index) || seen.len() > MAX_PLOT_DEPTH {
                            return Err(ChartError::Hierarchy);
                        }
                        cursor = values[index]
                            .parent
                            .as_ref()
                            .and_then(|key| indices.get(key.as_str()).copied());
                    }
                }
            }
            Self::Flows(data) => {
                if kind != PlotKind::Sankey {
                    return Err(ChartError::Schema);
                }
                count(data.nodes.len(), MAX_PLOT_RECORDS)?;
                count(data.links.len(), MAX_FLOW_LINKS)?;
                let mut nodes = BTreeSet::new();
                for r in &data.nodes {
                    record(&mut nodes, &r.key, &r.label)?;
                }
                let indices = data
                    .nodes
                    .iter()
                    .enumerate()
                    .map(|(i, r)| (r.key.as_str(), i))
                    .collect::<BTreeMap<_, _>>();
                let mut indegree = vec![0; data.nodes.len()];
                let mut children = vec![Vec::new(); data.nodes.len()];
                for edge in &data.links {
                    if !chart_model::key(&edge.key) || !keys.insert(edge.key.as_str()) {
                        return Err(ChartError::Key);
                    }
                    nonnegative(edge.value)?;
                    let (Some(from), Some(to)) = (
                        indices.get(edge.source.as_str()),
                        indices.get(edge.target.as_str()),
                    ) else {
                        return Err(ChartError::Flow);
                    };
                    if from == to {
                        return Err(ChartError::Flow);
                    }
                    indegree[*to] += 1;
                    children[*from].push(*to);
                }
                let mut ready = (0..indegree.len())
                    .filter(|i| indegree[*i] == 0)
                    .collect::<Vec<_>>();
                let mut visited = 0;
                let mut depth = vec![1; data.nodes.len()];
                while let Some(i) = ready.pop() {
                    visited += 1;
                    for child in &children[i] {
                        depth[*child] = depth[*child].max(depth[i] + 1);
                        if depth[*child] > MAX_PLOT_DEPTH {
                            return Err(ChartError::Flow);
                        }
                        indegree[*child] -= 1;
                        if indegree[*child] == 0 {
                            ready.push(*child);
                        }
                    }
                }
                if visited != data.nodes.len() {
                    return Err(ChartError::Flow);
                }
            }
        }
        Ok(())
    }
    pub fn keys(&self) -> Vec<&str> {
        match self {
            Self::Values(v) => v.iter().map(|r| r.key.as_str()).collect(),
            Self::Points(v) | Self::SampledPoints { points: v, .. } => {
                v.iter().map(|r| r.key.as_str()).collect()
            }
            Self::Candles(v) => v.iter().map(|r| r.key.as_str()).collect(),
            Self::Matrix(v) => v.cells.iter().map(|r| r.key.as_str()).collect(),
            Self::Targets(v) => v.iter().map(|r| r.key.as_str()).collect(),
            Self::Hierarchy(v) => v.iter().map(|r| r.key.as_str()).collect(),
            Self::Flows(v) => v.nodes.iter().map(|r| r.key.as_str()).collect(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sampling_reads_only_the_bounded_indices_and_checks_untrusted_metadata() {
        let mut calls = Vec::new();
        let data = PlotData::sample_points(usize::MAX, 128, |i| {
            calls.push(i);
            PointDatum {
                key: format!("p{i}"),
                label: i.to_string(),
                x: 1.,
                y: 2.,
                size: 3.,
            }
        })
        .unwrap();
        assert_eq!(calls.len(), 128);
        assert_eq!(calls[0], 0);
        assert_eq!(calls[127], usize::MAX - 1);
        assert!(calls.windows(2).all(|p| p[0] < p[1]));
        assert!(data.validate(PlotKind::Bubble).is_ok());
        let mut empty_calls = 0;
        PlotData::sample_points(0, 8, |_| {
            empty_calls += 1;
            unreachable!()
        })
        .unwrap();
        assert_eq!(empty_calls, 0);
        assert_eq!(
            PlotData::sample_points(1, 129, |_| unreachable!()),
            Err(ChartError::Budget)
        );
        let PlotData::SampledPoints {
            points,
            source_count,
            mut indices,
        } = data
        else {
            unreachable!()
        };
        indices[1] = indices[0];
        assert_eq!(
            PlotData::SampledPoints {
                points,
                source_count,
                indices
            }
            .validate(PlotKind::Scatter),
            Err(ChartError::Schema)
        );
    }
    #[test]
    fn typed_datasets_reject_cross_family_aliases_and_nonfinite_values() {
        let d = PlotData::values([ValueDatum::new("a", "A", 1.)]);
        assert_eq!(d.validate(PlotKind::Scatter), Err(ChartError::Schema));
        for v in [f64::NAN, f64::INFINITY, MAX_CHART_VALUE + 1.] {
            assert_eq!(
                PlotData::values([ValueDatum::new("a", "A", v)]).validate(PlotKind::Pie),
                Err(ChartError::Values)
            );
        }
        assert_eq!(
            PlotData::values([ValueDatum::new("a", "A", -1.)]).validate(PlotKind::Pie),
            Err(ChartError::Negative)
        );
        assert!(
            PlotData::values([ValueDatum::new("a", "A", -1.)])
                .validate(PlotKind::Sparkline)
                .is_ok()
        );
    }
    #[test]
    fn candle_ohlc_and_target_domains_are_cross_field_invariants() {
        let mut c = CandleDatum {
            key: "a".into(),
            label: "A".into(),
            open: 2.,
            close: 3.,
            low: 1.,
            high: 4.,
        };
        assert!(
            PlotData::Candles(vec![c.clone()])
                .validate(PlotKind::Candlestick)
                .is_ok()
        );
        c.low = 2.5;
        assert_eq!(
            PlotData::Candles(vec![c]).validate(PlotKind::Candlestick),
            Err(ChartError::Schema)
        );
        let mut t = TargetDatum {
            key: "a".into(),
            label: "A".into(),
            min: -10.,
            max: 10.,
            value: 0.,
            target: Some(5.),
            bands: vec![-5., 0., 10.],
        };
        assert!(
            PlotData::Targets(vec![t.clone()])
                .validate(PlotKind::Gauge)
                .is_ok()
        );
        t.bands = vec![0., -5.];
        assert_eq!(
            PlotData::Targets(vec![t]).validate(PlotKind::Bullet),
            Err(ChartError::Domain)
        );
    }
    #[test]
    fn hierarchy_requires_leaf_weights_complete_parents_no_cycles_and_bounded_depth() {
        let node = |key: &str, parent: Option<&str>, weight| HierarchyDatum {
            key: key.into(),
            label: key.into(),
            parent: parent.map(str::to_owned),
            weight,
        };
        assert!(
            PlotData::Hierarchy(vec![
                node("root", None, None),
                node("leaf", Some("root"), Some(2.))
            ])
            .validate(PlotKind::Treemap)
            .is_ok()
        );
        for records in [
            vec![node("a", Some("missing"), Some(2.))],
            vec![node("a", Some("b"), None), node("b", Some("a"), None)],
            vec![
                node("root", None, Some(2.)),
                node("leaf", Some("root"), Some(2.)),
            ],
        ] {
            assert_eq!(
                PlotData::Hierarchy(records).validate(PlotKind::Treemap),
                Err(ChartError::Hierarchy)
            );
        }
        let chain = |count| {
            PlotData::Hierarchy(
                (0..count)
                    .map(|i| HierarchyDatum {
                        key: format!("n{i}"),
                        label: format!("N{i}"),
                        parent: (i > 0).then(|| format!("n{}", i - 1)),
                        weight: (i + 1 == count).then_some(1.),
                    })
                    .collect(),
            )
        };
        assert!(chain(32).validate(PlotKind::Sunburst).is_ok());
        assert_eq!(
            chain(33).validate(PlotKind::Sunburst),
            Err(ChartError::Hierarchy)
        );
    }
    #[test]
    fn flows_require_a_dag_and_complete_current_endpoints() {
        let nodes = vec![ChartCategory::new("a", "A"), ChartCategory::new("b", "B")];
        let edge = |key: &str, from: &str, to: &str| FlowLink {
            key: key.into(),
            source: from.into(),
            target: to.into(),
            value: 2.,
        };
        assert!(
            PlotData::Flows(FlowData {
                nodes: nodes.clone(),
                links: vec![edge("ab", "a", "b")]
            })
            .validate(PlotKind::Sankey)
            .is_ok()
        );
        for links in [
            vec![edge("bad", "a", "c")],
            vec![edge("bad", "a", "a")],
            vec![edge("ab", "a", "b"), edge("ba", "b", "a")],
        ] {
            assert_eq!(
                PlotData::Flows(FlowData {
                    nodes: nodes.clone(),
                    links
                })
                .validate(PlotKind::Sankey),
                Err(ChartError::Flow)
            );
        }
    }
    #[test]
    fn matrix_rejects_unknown_axes_duplicate_cells_and_excess_before_scanning() {
        let mut d = MatrixData {
            rows: vec![ChartCategory::new("r", "Row")],
            columns: vec![ChartCategory::new("c", "Column")],
            cells: vec![MatrixCell {
                key: "one".into(),
                row: "r".into(),
                column: "c".into(),
                value: 1.,
            }],
        };
        assert!(
            PlotData::Matrix(d.clone())
                .validate(PlotKind::Heatmap)
                .is_ok()
        );
        let mut second = d.cells[0].clone();
        second.key = "two".into();
        d.cells.push(second);
        assert_eq!(
            PlotData::Matrix(d).validate(PlotKind::Matrix),
            Err(ChartError::Key)
        );
        let records = (0..129).map(|i| ValueDatum::new(format!("k{i}"), "A", 1.));
        assert_eq!(
            PlotData::values(records).validate(PlotKind::Pie),
            Err(ChartError::Budget)
        );
    }
    #[test]
    fn constructor_stops_at_the_finite_sentinel_and_nulls_remain_gaps() {
        use std::cell::Cell;
        let calls = Cell::new(0);
        let data = PlotData::values(std::iter::from_fn(|| {
            calls.set(calls.get() + 1);
            Some(ValueDatum::new("a", "A", 0.))
        }));
        assert_eq!(calls.get(), 129);
        assert_eq!(data.validate(PlotKind::Pie), Err(ChartError::Budget));
        let data = PlotData::Values(vec![ValueDatum {
            key: "gap".into(),
            label: "Gap".into(),
            value: None,
        }]);
        assert!(data.validate(PlotKind::Sparkline).is_ok());
    }
}
