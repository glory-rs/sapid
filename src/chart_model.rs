//! Finite admitted Cartesian data and scales. This optional module has no host,
//! serde, parser or third-party chart dependency.
use std::collections::BTreeSet;

pub const MAX_CHART_POINTS: usize = 128;
pub const MAX_CHART_SERIES: usize = 8;
pub const MAX_CHART_TEXT_BYTES: usize = 65536;
pub const MAX_CHART_VALUE: f64 = 1_000_000_000_000.;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartSeriesKind {
    #[default]
    Line,
    Area,
    Bar,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChartCategory {
    pub key: String,
    pub label: String,
}
impl ChartCategory {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ChartSeries {
    pub key: String,
    pub label: String,
    pub kind: ChartSeriesKind,
    /// One value for each category. None is an explicit gap, never zero.
    pub values: Vec<Option<f64>>,
}
impl ChartSeries {
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        values: impl IntoIterator<Item = Option<f64>>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            kind: ChartSeriesKind::Line,
            values: values.into_iter().take(MAX_CHART_POINTS + 1).collect(),
        }
    }
    pub fn kind(mut self, kind: ChartSeriesKind) -> Self {
        self.kind = kind;
        self
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChartData {
    pub categories: Vec<ChartCategory>,
    pub series: Vec<ChartSeries>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChartError {
    Budget,
    Key,
    Label,
    Values,
    Domain,
    Brush,
    Geometry,
    SignedAreaStack,
    Schema,
    Negative,
    Hierarchy,
    Flow,
}
impl ChartError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Budget => "chart exceeds its finite points, series or text budget",
            Self::Key => "chart keys require distinct 1..64 ASCII semantic tokens",
            Self::Label => {
                "chart labels require nonempty canonical text within 256 UTF-8 bytes"
            }
            Self::Values => "chart values must match categories and be finite within +/-1e12",
            Self::Domain => "chart scale requires a finite strictly increasing domain",
            Self::Brush => "chart brush must select an ordered in-bounds category interval",
            Self::Geometry => "chart geometry must be finite and positive within 16384 pixels",
            Self::SignedAreaStack => "stacked area series require nonnegative values",
            Self::Schema => {
                "chart records do not match this plot kind or its cross-field invariants"
            }
            Self::Negative => "this chart requires nonnegative values",
            Self::Hierarchy => {
                "chart hierarchy requires complete acyclic parents, leaf weights and depth at most 32"
            }
            Self::Flow => {
                "chart flows require complete distinct endpoints and an acyclic graph of depth at most 32"
            }
        }
    }
}
pub(crate) fn key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}
pub(crate) fn label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && !value.trim().is_empty()
        && !value.contains(['\0', '\r'])
}
impl ChartData {
    pub fn validate(&self) -> Result<(), ChartError> {
        if self.categories.len() > MAX_CHART_POINTS || self.series.len() > MAX_CHART_SERIES {
            return Err(ChartError::Budget);
        }
        let mut categories = BTreeSet::new();
        let mut series = BTreeSet::new();
        let mut bytes = 0usize;
        for category in &self.categories {
            if !key(&category.key) || !categories.insert(&category.key) {
                return Err(ChartError::Key);
            }
            if !label(&category.label) {
                return Err(ChartError::Label);
            }
            bytes += category.key.len() + category.label.len();
        }
        for value in &self.series {
            if !key(&value.key) || !series.insert(&value.key) {
                return Err(ChartError::Key);
            }
            if !label(&value.label) {
                return Err(ChartError::Label);
            }
            if value.values.len() != self.categories.len()
                || value
                    .values
                    .iter()
                    .flatten()
                    .any(|value| !value.is_finite() || value.abs() > MAX_CHART_VALUE)
            {
                return Err(ChartError::Values);
            }
            bytes += value.key.len() + value.label.len();
        }
        if bytes > MAX_CHART_TEXT_BYTES {
            return Err(ChartError::Budget);
        }
        Ok(())
    }
    pub fn domain(
        &self,
        stacked: bool,
        hidden: &BTreeSet<String>,
    ) -> Result<ChartDomain, ChartError> {
        self.validate()?;
        if hidden.len() > MAX_CHART_SERIES
            || hidden
                .iter()
                .any(|key| !self.series.iter().any(|series| &series.key == key))
        {
            return Err(ChartError::Key);
        }
        let mut min = 0f64;
        let mut max = 0f64;
        if stacked {
            for point in 0..self.categories.len() {
                let mut positive = 0.;
                let mut negative = 0.;
                for series in self
                    .series
                    .iter()
                    .filter(|series| !hidden.contains(&series.key))
                {
                    if let Some(value) = series.values[point] {
                        if value >= 0. {
                            positive += value
                        } else {
                            negative += value
                        }
                    }
                }
                min = min.min(negative);
                max = max.max(positive);
            }
        } else {
            for value in self
                .series
                .iter()
                .filter(|series| !hidden.contains(&series.key))
                .flat_map(|series| series.values.iter())
                .flatten()
            {
                min = min.min(*value);
                max = max.max(*value)
            }
        }
        // Including zero gives signed bars/areas a truthful baseline. An empty
        // or all-zero current dataset uses a stable finite neutral scale.
        if min == max {
            max = 1.;
        }
        ChartDomain::new(min, max)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartDomain {
    min: f64,
    max: f64,
}
impl ChartDomain {
    pub fn new(min: f64, max: f64) -> Result<Self, ChartError> {
        if !min.is_finite()
            || !max.is_finite()
            || min >= max
            || min.abs() > MAX_CHART_VALUE * 8.
            || max.abs() > MAX_CHART_VALUE * 8.
        {
            return Err(ChartError::Domain);
        }
        Ok(Self { min, max })
    }
    pub fn min(self) -> f64 {
        self.min
    }
    pub fn max(self) -> f64 {
        self.max
    }
    pub fn fraction(self, value: f64) -> Result<f64, ChartError> {
        if !value.is_finite() || value.abs() > MAX_CHART_VALUE * 8. {
            return Err(ChartError::Values);
        }
        let fraction = (value - self.min) / (self.max - self.min);
        if !fraction.is_finite() {
            return Err(ChartError::Domain);
        }
        Ok(fraction)
    }
    /// Finite equal-interval labels; callers format values explicitly rather
    /// than inheriting OS locale or inventing an implicit numeric currency.
    pub fn ticks(self, count: usize) -> Result<Vec<f64>, ChartError> {
        if !(2..=9).contains(&count) {
            return Err(ChartError::Budget);
        }
        Ok((0..count)
            .map(|i| self.min + (self.max - self.min) * i as f64 / (count - 1) as f64)
            .collect())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChartBrush {
    pub first: usize,
    pub last: usize,
}
impl ChartBrush {
    pub fn validate(self, count: usize) -> Result<(), ChartError> {
        if self.first > self.last || self.last >= count || count > MAX_CHART_POINTS {
            return Err(ChartError::Brush);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> ChartData {
        ChartData {
            categories: vec![ChartCategory::new("a", "A"), ChartCategory::new("b", "B")],
            series: vec![
                ChartSeries::new("one", "One", [Some(-4.), Some(3.)]),
                ChartSeries::new("two", "Two", [Some(-2.), None]),
            ],
        }
    }
    #[test]
    fn signed_stacks_sum_positive_and_negative_independently_and_null_is_a_gap() {
        let d = data();
        assert_eq!(
            d.domain(true, &BTreeSet::new()).unwrap(),
            ChartDomain::new(-6., 3.).unwrap()
        );
        assert_eq!(
            d.domain(false, &BTreeSet::new()).unwrap(),
            ChartDomain::new(-4., 3.).unwrap()
        );
        assert_eq!(
            d.domain(true, &BTreeSet::from(["two".into()])).unwrap(),
            ChartDomain::new(-4., 3.).unwrap()
        );
        assert_eq!(d.series[1].values[1], None);
    }
    #[test]
    fn limits_are_checked_before_unbounded_payloads_and_invalid_floats() {
        let mut d = data();
        d.categories = (0..129)
            .map(|i| ChartCategory::new(format!("k{i}"), "Label"))
            .collect();
        assert_eq!(d.validate(), Err(ChartError::Budget));
        for value in [f64::NAN, f64::INFINITY, MAX_CHART_VALUE + 1.] {
            let mut d = data();
            d.series[0].values[0] = Some(value);
            assert_eq!(d.validate(), Err(ChartError::Values));
        }
        let mut d = data();
        d.series[0].values.pop();
        assert_eq!(d.validate(), Err(ChartError::Values));
        let mut d = data();
        d.categories[1].key = "a".into();
        assert_eq!(d.validate(), Err(ChartError::Key));
        let mut d = data();
        d.series[0].label = "x".repeat(257);
        assert_eq!(d.validate(), Err(ChartError::Label));
    }
    #[test]
    fn empty_zero_extreme_domains_and_brushes_are_explicit() {
        assert_eq!(
            ChartData::default()
                .domain(false, &BTreeSet::new())
                .unwrap(),
            ChartDomain::new(0., 1.).unwrap()
        );
        let d = ChartDomain::new(-MAX_CHART_VALUE * 8., MAX_CHART_VALUE * 8.).unwrap();
        assert_eq!(d.fraction(0.).unwrap(), 0.5);
        let ticks = d.ticks(5).unwrap();
        assert!(ticks.iter().all(|v| v.is_finite()));
        assert_eq!(ticks[0], d.min());
        assert_eq!(ticks[4], d.max());
        assert!(d.ticks(10).is_err());
        assert!(ChartDomain::new(1., 1.).is_err());
        assert!(ChartDomain::new(f64::NAN, 2.).is_err());
        assert!(
            ChartDomain::new(0., f64::MIN_POSITIVE)
                .unwrap()
                .fraction(MAX_CHART_VALUE)
                .is_err()
        );
        assert!(ChartBrush { first: 0, last: 1 }.validate(2).is_ok());
        assert!(ChartBrush { first: 1, last: 0 }.validate(2).is_err());
        assert!(ChartBrush { first: 0, last: 0 }.validate(0).is_err());
    }
    #[test]
    fn constructor_consumes_only_the_finite_sentinel() {
        use std::cell::Cell;
        let reads = Cell::new(0);
        let s = ChartSeries::new(
            "s",
            "Series",
            std::iter::from_fn(|| {
                reads.set(reads.get() + 1);
                Some(Some(1.))
            }),
        );
        assert_eq!(s.values.len(), 129);
        assert_eq!(reads.get(), 129);
    }
}
