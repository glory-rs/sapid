//! Sealed finite month/year domains, with no implicit representative day.
use crate::{
    CivilDate, DateBounds, DateError, DatePickerMode, DateSelectionError, MAX_SELECTED_DATES,
    YearMonth,
};
use std::fmt;
mod sealed {
    pub trait Sealed {}
}
/// A finite year, independent of month/day/timezone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CivilYear(u16);
impl CivilYear {
    pub const MIN: Self = Self(1);
    pub const MAX: Self = Self(9999);
    pub fn new(year: u16) -> Result<Self, DateError> {
        if (1..=9999).contains(&year) {
            Ok(Self(year))
        } else {
            Err(DateError::OutOfRange)
        }
    }
    pub const fn value(self) -> u16 {
        self.0
    }
    pub fn checked_add(self, years: i32) -> Result<Self, DateError> {
        let n = i64::from(self.0) + i64::from(years);
        if !(1..=9999).contains(&n) {
            Err(DateError::OutOfRange)
        } else {
            Ok(Self(n as u16))
        }
    }
    pub fn parse_iso(value: &str) -> Result<Self, DateError> {
        if value.len() != 4 || !value.bytes().all(|b| b.is_ascii_digit()) {
            return Err(DateError::InvalidText);
        }
        Self::new(value.parse().map_err(|_| DateError::InvalidText)?)
    }
}
impl fmt::Display for CivilYear {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}", self.0)
    }
}
impl sealed::Sealed for CivilYear {}
impl sealed::Sealed for YearMonth {}
/// Sealed to the two small civil period values; user business types cannot widen storage.
pub trait DatePeriod: sealed::Sealed + Copy + Ord + fmt::Debug + fmt::Display + 'static {
    const TEXT_BYTES: usize;
    const MONTH: bool;
    fn ordinal(self) -> u32;
    fn from_ordinal(value: u32) -> Result<Self, DateError>;
    fn from_date(value: CivilDate) -> Self;
    fn parse_iso(value: &str) -> Result<Self, DateError>;
    fn checked_shift(self, steps: i32) -> Result<Self, DateError> {
        let next = i64::from(self.ordinal()) + i64::from(steps);
        let next = u32::try_from(next).map_err(|_| DateError::OutOfRange)?;
        Self::from_ordinal(next)
    }
}
impl DatePeriod for CivilYear {
    const TEXT_BYTES: usize = 4;
    const MONTH: bool = false;
    fn ordinal(self) -> u32 {
        u32::from(self.0) - 1
    }
    fn from_ordinal(value: u32) -> Result<Self, DateError> {
        if value < 9999 {
            Ok(Self(value as u16 + 1))
        } else {
            Err(DateError::OutOfRange)
        }
    }
    fn from_date(value: CivilDate) -> Self {
        Self(value.year())
    }
    fn parse_iso(value: &str) -> Result<Self, DateError> {
        Self::parse_iso(value)
    }
}
impl DatePeriod for YearMonth {
    const TEXT_BYTES: usize = 7;
    const MONTH: bool = true;
    fn ordinal(self) -> u32 {
        (u32::from(self.year()) - 1) * 12 + u32::from(self.month()) - 1
    }
    fn from_ordinal(value: u32) -> Result<Self, DateError> {
        if value < 9999 * 12 {
            Self::new((value / 12 + 1) as u16, (value % 12 + 1) as u8)
        } else {
            Err(DateError::OutOfRange)
        }
    }
    fn from_date(value: CivilDate) -> Self {
        value.year_month()
    }
    fn parse_iso(value: &str) -> Result<Self, DateError> {
        if value.len() != 7
            || !value.bytes().enumerate().all(|(i, b)| {
                if i == 4 {
                    b == b'-'
                } else {
                    b.is_ascii_digit()
                }
            })
        {
            return Err(DateError::InvalidText);
        }
        CivilDate::parse_iso(&format!("{value}-01")).map(CivilDate::year_month)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PeriodBounds<T: DatePeriod> {
    min: Option<T>,
    max: Option<T>,
}
impl<T: DatePeriod> Default for PeriodBounds<T> {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
        }
    }
}
impl<T: DatePeriod> PeriodBounds<T> {
    pub fn new(min: Option<T>, max: Option<T>) -> Result<Self, DateError> {
        if min.zip(max).is_some_and(|(a, b)| a > b) {
            Err(DateError::InvalidBounds)
        } else {
            Ok(Self { min, max })
        }
    }
    /// Date endpoints project to their containing month/year, explicitly allowing partial periods.
    pub fn from_dates(bounds: DateBounds) -> Self {
        Self {
            min: bounds.min().map(T::from_date),
            max: bounds.max().map(T::from_date),
        }
    }
    pub fn min(self) -> Option<T> {
        self.min
    }
    pub fn max(self) -> Option<T> {
        self.max
    }
    pub fn contains(self, value: T) -> bool {
        self.min.is_none_or(|v| value >= v) && self.max.is_none_or(|v| value <= v)
    }
    pub fn clamp(self, value: T) -> T {
        let value = self.min.map_or(value, |v| value.max(v));
        self.max.map_or(value, |v| value.min(v))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PeriodRange<T: DatePeriod> {
    start: T,
    end: T,
}
impl<T: DatePeriod> PeriodRange<T> {
    pub fn new(start: T, end: T) -> Result<Self, DateError> {
        if start > end {
            Err(DateError::InvalidBounds)
        } else {
            Ok(Self { start, end })
        }
    }
    pub fn start(self) -> T {
        self.start
    }
    pub fn end(self) -> T {
        self.end
    }
    pub fn contains(self, value: T) -> bool {
        self.start <= value && value <= self.end
    }
    pub fn len_periods(self) -> u32 {
        self.end.ordinal() - self.start.ordinal() + 1
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectedPeriods<T: DatePeriod> {
    values: [Option<T>; MAX_SELECTED_DATES],
    len: u8,
}
impl<T: DatePeriod> Default for SelectedPeriods<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: DatePeriod> SelectedPeriods<T> {
    pub const fn new() -> Self {
        Self {
            values: [None; MAX_SELECTED_DATES],
            len: 0,
        }
    }
    pub fn try_from_iter(values: impl IntoIterator<Item = T>) -> Result<Self, DateSelectionError> {
        let mut out = Self::new();
        for (index, value) in values.into_iter().enumerate() {
            if index == MAX_SELECTED_DATES {
                return Err(DateSelectionError::TooManyDates);
            }
            out.insert(value)?;
        }
        Ok(out)
    }
    pub const fn len(self) -> usize {
        self.len as usize
    }
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = T> + '_ {
        self.values[..self.len as usize].iter().map(|v| v.unwrap())
    }
    pub fn contains(&self, value: T) -> bool {
        self.values[..self.len as usize]
            .binary_search(&Some(value))
            .is_ok()
    }
    pub fn insert(&mut self, value: T) -> Result<bool, DateSelectionError> {
        let index = match self.values[..self.len as usize].binary_search(&Some(value)) {
            Ok(_) => return Ok(false),
            Err(i) => i,
        };
        let len = self.len as usize;
        if len == MAX_SELECTED_DATES {
            return Err(DateSelectionError::TooManyDates);
        }
        self.values.copy_within(index..len, index + 1);
        self.values[index] = Some(value);
        self.len += 1;
        Ok(true)
    }
    pub fn remove(&mut self, value: T) -> bool {
        let Ok(index) = self.values[..self.len as usize].binary_search(&Some(value)) else {
            return false;
        };
        let len = self.len as usize;
        self.values.copy_within(index + 1..len, index);
        self.values[len - 1] = None;
        self.len -= 1;
        true
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeriodRangeSelection<T: DatePeriod> {
    Empty,
    Start(T),
    Complete(PeriodRange<T>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeriodSelection<T: DatePeriod> {
    Single(Option<T>),
    Multiple(SelectedPeriods<T>),
    Range(PeriodRangeSelection<T>),
}
impl<T: DatePeriod> Default for PeriodSelection<T> {
    fn default() -> Self {
        Self::Single(None)
    }
}
impl<T: DatePeriod> PeriodSelection<T> {
    pub const fn empty(mode: DatePickerMode) -> Self {
        match mode {
            DatePickerMode::Single => Self::Single(None),
            DatePickerMode::Multiple => Self::Multiple(SelectedPeriods::new()),
            DatePickerMode::Range => Self::Range(PeriodRangeSelection::Empty),
        }
    }
    pub const fn mode(self) -> DatePickerMode {
        match self {
            Self::Single(_) => DatePickerMode::Single,
            Self::Multiple(_) => DatePickerMode::Multiple,
            Self::Range(_) => DatePickerMode::Range,
        }
    }
    pub fn anchor(self) -> Option<T> {
        match self {
            Self::Single(v) => v,
            Self::Multiple(v) => v.iter().last(),
            Self::Range(PeriodRangeSelection::Empty) => None,
            Self::Range(PeriodRangeSelection::Start(v)) => Some(v),
            Self::Range(PeriodRangeSelection::Complete(v)) => Some(v.end),
        }
    }
    pub fn contains(self, value: T) -> bool {
        match self {
            Self::Single(v) => v == Some(value),
            Self::Multiple(v) => v.contains(value),
            Self::Range(PeriodRangeSelection::Empty) => false,
            Self::Range(PeriodRangeSelection::Start(v)) => v == value,
            Self::Range(PeriodRangeSelection::Complete(v)) => v.contains(value),
        }
    }
    pub fn is_complete(self) -> bool {
        match self {
            Self::Single(v) => v.is_some(),
            Self::Multiple(v) => !v.is_empty(),
            Self::Range(PeriodRangeSelection::Complete(_)) => true,
            Self::Range(_) => false,
        }
    }
    pub fn validate(
        self,
        mode: DatePickerMode,
        bounds: PeriodBounds<T>,
    ) -> Result<(), DateSelectionError> {
        if self.mode() != mode {
            return Err(DateSelectionError::InvalidMode);
        }
        let allowed = match self {
            Self::Single(v) => v.is_none_or(|v| bounds.contains(v)),
            Self::Multiple(v) => v.iter().all(|v| bounds.contains(v)),
            Self::Range(PeriodRangeSelection::Empty) => true,
            Self::Range(PeriodRangeSelection::Start(v)) => bounds.contains(v),
            Self::Range(PeriodRangeSelection::Complete(v)) => {
                bounds.contains(v.start) && bounds.contains(v.end)
            }
        };
        if allowed {
            Ok(())
        } else {
            Err(DateSelectionError::OutOfBounds)
        }
    }
    pub fn activate(self, value: T, deselect: bool) -> Result<Self, DateSelectionError> {
        Ok(match self {
            Self::Single(old) => Self::Single(if deselect && old == Some(value) {
                None
            } else {
                Some(value)
            }),
            Self::Multiple(mut values) => {
                if !values.remove(value) {
                    values.insert(value)?;
                }
                Self::Multiple(values)
            }
            Self::Range(PeriodRangeSelection::Start(start)) => {
                Self::Range(PeriodRangeSelection::Complete(PeriodRange::new(
                    start.min(value),
                    start.max(value),
                )?))
            }
            Self::Range(_) => Self::Range(PeriodRangeSelection::Start(value)),
        })
    }
    pub fn format_iso(self) -> String {
        match self {
            Self::Single(v) => v.map(|v| v.to_string()).unwrap_or_default(),
            Self::Multiple(v) => {
                let mut text = String::new();
                for p in v.iter() {
                    if !text.is_empty() {
                        text.push(',');
                    }
                    text.push_str(&p.to_string());
                }
                text
            }
            Self::Range(PeriodRangeSelection::Empty) => String::new(),
            Self::Range(PeriodRangeSelection::Start(v)) => format!("{v}/"),
            Self::Range(PeriodRangeSelection::Complete(v)) => format!("{}/{}", v.start, v.end),
        }
    }
    pub fn parse_iso(mode: DatePickerMode, value: &str) -> Result<Self, DateSelectionError> {
        if value.len() > MAX_SELECTED_DATES * (T::TEXT_BYTES + 1) - 1 {
            return Err(DateSelectionError::InvalidText);
        }
        if value.is_empty() {
            return Ok(Self::empty(mode));
        }
        Ok(match mode {
            DatePickerMode::Single => Self::Single(Some(T::parse_iso(value)?)),
            DatePickerMode::Multiple => {
                let mut selected = SelectedPeriods::new();
                for (count, part) in value.split(',').enumerate() {
                    if count == MAX_SELECTED_DATES {
                        return Err(DateSelectionError::TooManyDates);
                    }
                    selected.insert(T::parse_iso(part)?)?;
                }
                Self::Multiple(selected)
            }
            DatePickerMode::Range => {
                let Some((start, end)) = value.split_once('/') else {
                    return Err(DateSelectionError::InvalidText);
                };
                let start = T::parse_iso(start)?;
                Self::Range(if end.is_empty() {
                    PeriodRangeSelection::Start(start)
                } else {
                    PeriodRangeSelection::Complete(PeriodRange::new(start, T::parse_iso(end)?)?)
                })
            }
        })
    }
}
pub type MonthSelection = PeriodSelection<YearMonth>;
pub type YearSelection = PeriodSelection<CivilYear>;
pub type SelectedMonths = SelectedPeriods<YearMonth>;
pub type SelectedYears = SelectedPeriods<CivilYear>;
pub type MonthRange = PeriodRange<YearMonth>;
pub type YearRange = PeriodRange<CivilYear>;
pub type MonthRangeSelection = PeriodRangeSelection<YearMonth>;
pub type YearRangeSelection = PeriodRangeSelection<CivilYear>;
pub type MonthBounds = PeriodBounds<YearMonth>;
pub type YearBounds = PeriodBounds<CivilYear>;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_supported_year_and_month_roundtrips_without_inventing_a_day() {
        for y in 1..=9999 {
            let year = CivilYear::new(y).unwrap();
            assert_eq!(CivilYear::parse_iso(&year.to_string()).unwrap(), year);
            assert_eq!(CivilYear::from_ordinal(year.ordinal()).unwrap(), year);
            for m in 1..=12 {
                let month = YearMonth::new(y, m).unwrap();
                assert_eq!(
                    <YearMonth as DatePeriod>::parse_iso(&month.to_string()).unwrap(),
                    month
                );
                assert_eq!(YearMonth::from_ordinal(month.ordinal()).unwrap(), month);
            }
        }
    }
    #[test]
    fn arithmetic_and_exact_spelling_reject_domain_edges_and_unicode() {
        assert!(CivilYear::MIN.checked_add(-1).is_err());
        assert!(CivilYear::MAX.checked_add(1).is_err());
        assert!(CivilYear::new(0).is_err());
        for v in ["24", "0000", "10000", "２０２４", "2024 "] {
            assert!(CivilYear::parse_iso(v).is_err());
        }
        for v in ["2024-2", "2024-00", "2024-13", "2024/02", "2024-02-01"] {
            assert!(<YearMonth as DatePeriod>::parse_iso(v).is_err());
        }
        assert!(YearMonth::new(1, 1).unwrap().checked_shift(-1).is_err());
        assert!(YearMonth::new(9999, 12).unwrap().checked_shift(1).is_err());
    }
    #[test]
    fn multiple_admission_is_a_fixed_unique_set_with_a_limit_plus_one_sentinel() {
        let seen = std::cell::Cell::new(0);
        assert!(
            SelectedYears::try_from_iter((0..1_000_000).map(|_| {
                seen.set(seen.get() + 1);
                CivilYear::MIN
            }))
            .is_err()
        );
        assert_eq!(seen.get(), 33);
        let years = SelectedYears::try_from_iter([
            CivilYear::new(2024).unwrap(),
            CivilYear::MIN,
            CivilYear::MIN,
        ])
        .unwrap();
        assert_eq!(years.len(), 2);
        assert_eq!(years.iter().next(), Some(CivilYear::MIN));
    }
    #[test]
    fn range_membership_is_inclusive_constant_storage_and_restarts_after_completion() {
        let start = YearSelection::empty(DatePickerMode::Range)
            .activate(CivilYear::MAX, false)
            .unwrap();
        assert!(!start.is_complete());
        let end = start.activate(CivilYear::MIN, false).unwrap();
        assert!(end.is_complete());
        assert!(end.contains(CivilYear::new(5000).unwrap()));
        assert_eq!(end.format_iso(), "0001/9999");
        assert_eq!(
            end.activate(CivilYear::new(2024).unwrap(), false)
                .unwrap()
                .format_iso(),
            "2024/"
        );
        assert_eq!(
            YearRange::new(CivilYear::MIN, CivilYear::MAX)
                .unwrap()
                .len_periods(),
            9999
        );
    }
    #[test]
    fn all_modes_roundtrip_and_reversed_ranges_or_wrong_modes_reject() {
        for (mode, v) in [
            (DatePickerMode::Single, "2024-02"),
            (DatePickerMode::Multiple, "2024-01,2024-12"),
            (DatePickerMode::Range, "2024-02/"),
            (DatePickerMode::Range, "2024-02/2025-01"),
        ] {
            assert_eq!(MonthSelection::parse_iso(mode, v).unwrap().format_iso(), v);
        }
        assert!(YearSelection::parse_iso(DatePickerMode::Range, "2024/2023").is_err());
        assert!(MonthSelection::parse_iso(DatePickerMode::Single, &"8".repeat(1_000_000)).is_err());
        assert!(
            YearSelection::empty(DatePickerMode::Single)
                .validate(DatePickerMode::Range, YearBounds::default())
                .is_err()
        );
    }
    #[test]
    fn date_bounds_project_to_containing_periods_with_no_full_span_scan() {
        let b = DateBounds::new(
            Some(CivilDate::new(2024, 3, 15).unwrap()),
            Some(CivilDate::new(2025, 2, 4).unwrap()),
        )
        .unwrap();
        let months = MonthBounds::from_dates(b);
        assert!(months.contains(YearMonth::new(2024, 3).unwrap()));
        assert!(!months.contains(YearMonth::new(2024, 2).unwrap()));
        let years = YearBounds::from_dates(b);
        assert!(years.contains(CivilYear::new(2024).unwrap()));
        assert!(years.contains(CivilYear::new(2025).unwrap()));
        assert_eq!(years.clamp(CivilYear::MAX), CivilYear::new(2025).unwrap());
    }
}
