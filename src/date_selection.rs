//! Finite civil selection values. Multiple selections never contain an unbounded Vec.
use crate::dates::{CivilDate, CivilRange, DateBounds, DateError};
use std::fmt;
pub const MAX_SELECTED_DATES: usize = 32;
pub const MAX_DATE_SELECTION_TEXT_BYTES: usize = MAX_SELECTED_DATES * 11 - 1;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DatePickerMode {
    #[default]
    Single,
    Multiple,
    Range,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateSelectionError {
    TooManyDates,
    InvalidMode,
    InvalidText,
    OutOfBounds,
    Date(DateError),
}
impl fmt::Display for DateSelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooManyDates => "date selection exceeds its finite capacity",
            Self::InvalidMode => "date selection does not match the configured mode",
            Self::InvalidText => "invalid exact ISO date selection",
            Self::OutOfBounds => "date selection is outside its explicit bounds",
            Self::Date(_) => "invalid date in selection",
        })
    }
}
impl std::error::Error for DateSelectionError {}
impl From<DateError> for DateSelectionError {
    fn from(value: DateError) -> Self {
        Self::Date(value)
    }
}
/// Sorted unique civil dates, with fixed capacity and bounded iterator admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectedDates {
    values: [Option<CivilDate>; MAX_SELECTED_DATES],
    len: u8,
}
impl Default for SelectedDates {
    fn default() -> Self {
        Self::new()
    }
}
impl SelectedDates {
    pub const fn new() -> Self {
        Self {
            values: [None; MAX_SELECTED_DATES],
            len: 0,
        }
    }
    pub fn try_from_iter(
        values: impl IntoIterator<Item = CivilDate>,
    ) -> Result<Self, DateSelectionError> {
        let mut selected = Self::new();
        for (count, date) in values.into_iter().enumerate() {
            if count == MAX_SELECTED_DATES {
                return Err(DateSelectionError::TooManyDates);
            }
            selected.insert(date)?;
        }
        Ok(selected)
    }
    pub const fn len(self) -> usize {
        self.len as usize
    }
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = CivilDate> + '_ {
        self.values[..self.len as usize].iter().map(|v| v.unwrap())
    }
    pub fn contains(&self, date: CivilDate) -> bool {
        self.values[..self.len as usize]
            .binary_search(&Some(date))
            .is_ok()
    }
    pub fn insert(&mut self, date: CivilDate) -> Result<bool, DateSelectionError> {
        let index = match self.values[..self.len as usize].binary_search(&Some(date)) {
            Ok(_) => return Ok(false),
            Err(index) => index,
        };
        let len = self.len as usize;
        if len == MAX_SELECTED_DATES {
            return Err(DateSelectionError::TooManyDates);
        }
        self.values.copy_within(index..len, index + 1);
        self.values[index] = Some(date);
        self.len += 1;
        Ok(true)
    }
    pub fn remove(&mut self, date: CivilDate) -> bool {
        let Ok(index) = self.values[..self.len as usize].binary_search(&Some(date)) else {
            return false;
        };
        let len = self.len as usize;
        self.values.copy_within(index + 1..len, index);
        self.values[len - 1] = None;
        self.len -= 1;
        true
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DateRangeSelection {
    #[default]
    Empty,
    Start(CivilDate),
    Complete(CivilRange),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateSelection {
    Single(Option<CivilDate>),
    Multiple(SelectedDates),
    Range(DateRangeSelection),
}
impl Default for DateSelection {
    fn default() -> Self {
        Self::Single(None)
    }
}
impl DateSelection {
    pub const fn empty(mode: DatePickerMode) -> Self {
        match mode {
            DatePickerMode::Single => Self::Single(None),
            DatePickerMode::Multiple => Self::Multiple(SelectedDates::new()),
            DatePickerMode::Range => Self::Range(DateRangeSelection::Empty),
        }
    }
    pub const fn mode(self) -> DatePickerMode {
        match self {
            Self::Single(_) => DatePickerMode::Single,
            Self::Multiple(_) => DatePickerMode::Multiple,
            Self::Range(_) => DatePickerMode::Range,
        }
    }
    pub fn contains(self, date: CivilDate) -> bool {
        match self {
            Self::Single(value) => value == Some(date),
            Self::Multiple(values) => values.contains(date),
            Self::Range(DateRangeSelection::Empty) => false,
            Self::Range(DateRangeSelection::Start(start)) => start == date,
            Self::Range(DateRangeSelection::Complete(range)) => range.contains(date),
        }
    }
    pub fn anchor(self) -> Option<CivilDate> {
        match self {
            Self::Single(value) => value,
            Self::Multiple(values) => values.iter().last(),
            Self::Range(DateRangeSelection::Empty) => None,
            Self::Range(DateRangeSelection::Start(start)) => Some(start),
            Self::Range(DateRangeSelection::Complete(range)) => Some(range.end()),
        }
    }
    pub fn is_complete(self) -> bool {
        match self {
            Self::Single(value) => value.is_some(),
            Self::Multiple(values) => !values.is_empty(),
            Self::Range(DateRangeSelection::Complete(_)) => true,
            Self::Range(_) => false,
        }
    }
    pub fn validate(
        self,
        mode: DatePickerMode,
        bounds: DateBounds,
    ) -> Result<(), DateSelectionError> {
        if self.mode() != mode {
            return Err(DateSelectionError::InvalidMode);
        }
        let valid = match self {
            Self::Single(value) => value.is_none_or(|d| bounds.contains(d)),
            Self::Multiple(values) => values.iter().all(|d| bounds.contains(d)),
            Self::Range(DateRangeSelection::Empty) => true,
            Self::Range(DateRangeSelection::Start(start)) => bounds.contains(start),
            Self::Range(DateRangeSelection::Complete(range)) => {
                bounds.contains(range.start()) && bounds.contains(range.end())
            }
        };
        if valid {
            Ok(())
        } else {
            Err(DateSelectionError::OutOfBounds)
        }
    }
    pub fn activate(
        self,
        date: CivilDate,
        allow_deselect: bool,
    ) -> Result<Self, DateSelectionError> {
        Ok(match self {
            Self::Single(value) => Self::Single(if allow_deselect && value == Some(date) {
                None
            } else {
                Some(date)
            }),
            Self::Multiple(mut values) => {
                if !values.remove(date) {
                    values.insert(date)?;
                }
                Self::Multiple(values)
            }
            Self::Range(DateRangeSelection::Start(start)) => {
                Self::Range(DateRangeSelection::Complete(CivilRange::new(
                    start.min(date),
                    start.max(date),
                )?))
            }
            Self::Range(_) => Self::Range(DateRangeSelection::Start(date)),
        })
    }
    /// Explicit successful-value spelling; no locale or timezone is inferred.
    pub fn format_iso(self) -> String {
        match self {
            Self::Single(value) => value.map(|d| d.to_string()).unwrap_or_default(),
            Self::Multiple(values) => {
                let mut text = String::new();
                for date in values.iter() {
                    if !text.is_empty() {
                        text.push(',');
                    }
                    text.push_str(&date.to_string());
                }
                text
            }
            Self::Range(DateRangeSelection::Empty) => String::new(),
            Self::Range(DateRangeSelection::Start(start)) => format!("{start}/"),
            Self::Range(DateRangeSelection::Complete(range)) => {
                format!("{}/{}", range.start(), range.end())
            }
        }
    }
    pub fn parse_iso(mode: DatePickerMode, text: &str) -> Result<Self, DateSelectionError> {
        if text.len() > MAX_DATE_SELECTION_TEXT_BYTES {
            return Err(DateSelectionError::InvalidText);
        }
        if text.is_empty() {
            return Ok(Self::empty(mode));
        }
        Ok(match mode {
            DatePickerMode::Single => Self::Single(Some(CivilDate::parse_iso(text)?)),
            DatePickerMode::Multiple => {
                let mut selected = SelectedDates::new();
                for (count, part) in text.split(',').enumerate() {
                    if count == MAX_SELECTED_DATES {
                        return Err(DateSelectionError::TooManyDates);
                    }
                    selected.insert(CivilDate::parse_iso(part)?)?;
                }
                Self::Multiple(selected)
            }
            DatePickerMode::Range => {
                let Some((start, end)) = text.split_once('/') else {
                    return Err(DateSelectionError::InvalidText);
                };
                let start = CivilDate::parse_iso(start)?;
                Self::Range(if end.is_empty() {
                    DateRangeSelection::Start(start)
                } else {
                    DateRangeSelection::Complete(CivilRange::new(
                        start,
                        CivilDate::parse_iso(end)?,
                    )?)
                })
            }
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, rc::Rc};
    fn d(value: &str) -> CivilDate {
        CivilDate::parse_iso(value).unwrap()
    }
    #[test]
    fn multiple_admission_is_fixed_capacity_sorted_unique_and_never_consumes_a_large_iterator()
    {
        let mut selected =
            SelectedDates::try_from_iter([d("2024-02-29"), d("2024-01-01"), d("2024-02-29")])
                .unwrap();
        assert_eq!(
            selected.iter().collect::<Vec<_>>(),
            [d("2024-01-01"), d("2024-02-29")]
        );
        assert!(!selected.insert(d("2024-02-29")).unwrap());
        assert!(selected.remove(d("2024-01-01")));
        assert!(!selected.remove(d("2024-01-01")));
        let seen = Rc::new(Cell::new(0));
        let out = seen.clone();
        assert!(
            SelectedDates::try_from_iter((0..1_000_000).map(move |_| {
                out.set(out.get() + 1);
                CivilDate::MIN
            }))
            .is_err()
        );
        assert_eq!(seen.get(), MAX_SELECTED_DATES + 1);
    }
    #[test]
    fn toggling_multiple_dates_respects_capacity_without_changing_the_old_copy() {
        let mut values = SelectedDates::new();
        for i in 0..32 {
            values
                .insert(CivilDate::MIN.checked_add_days(i).unwrap())
                .unwrap();
        }
        let old = DateSelection::Multiple(values);
        assert!(
            old.activate(CivilDate::MIN.checked_add_days(32).unwrap(), false)
                .is_err()
        );
        assert_eq!(values.len(), 32);
        let next = old.activate(CivilDate::MIN, false).unwrap();
        assert!(!next.contains(CivilDate::MIN));
        assert!(old.contains(CivilDate::MIN));
    }
    #[test]
    fn range_partial_complete_reverse_and_restart_are_explicit_and_do_not_scan_the_span() {
        let empty = DateSelection::empty(DatePickerMode::Range);
        let start = empty.activate(d("2024-03-05"), false).unwrap();
        assert!(!start.is_complete());
        assert!(start.contains(d("2024-03-05")));
        let complete = start.activate(d("2024-02-29"), false).unwrap();
        assert!(complete.is_complete());
        assert!(complete.contains(d("2024-03-01")));
        assert_eq!(complete.format_iso(), "2024-02-29/2024-03-05");
        assert_eq!(
            complete.activate(CivilDate::MIN, false).unwrap(),
            DateSelection::Range(DateRangeSelection::Start(CivilDate::MIN))
        );
        let all = DateSelection::Range(DateRangeSelection::Complete(
            CivilRange::new(CivilDate::MIN, CivilDate::MAX).unwrap(),
        ));
        assert!(all.contains(d("5000-01-01")));
    }
    #[test]
    fn exact_iso_roundtrips_empty_partial_and_all_modes_without_timezone_or_locale() {
        for (mode, text) in [
            (DatePickerMode::Single, ""),
            (DatePickerMode::Single, "2024-02-29"),
            (DatePickerMode::Multiple, "2024-01-01,2024-02-29"),
            (DatePickerMode::Range, "2024-02-29/"),
            (DatePickerMode::Range, "2024-02-29/2025-01-01"),
        ] {
            assert_eq!(
                DateSelection::parse_iso(mode, text).unwrap().format_iso(),
                text
            );
        }
        for text in ["2024-2-29", "2024-02-29Z", " 2024-02-29"] {
            assert!(DateSelection::parse_iso(DatePickerMode::Single, text).is_err());
        }
        assert!(
            DateSelection::parse_iso(DatePickerMode::Range, "2025-01-01/2024-01-01").is_err()
        );
        assert!(
            DateSelection::parse_iso(DatePickerMode::Multiple, &"8".repeat(1_000_000)).is_err()
        );
    }
    #[test]
    fn selection_mode_bounds_and_single_deselection_are_independent() {
        let date = d("2024-02-29");
        let single = DateSelection::Single(Some(date));
        assert_eq!(
            single.activate(date, true).unwrap(),
            DateSelection::Single(None)
        );
        assert_eq!(single.activate(date, false).unwrap(), single);
        let b = DateBounds::new(Some(date), Some(date)).unwrap();
        assert!(single.validate(DatePickerMode::Single, b).is_ok());
        assert_eq!(
            single.validate(DatePickerMode::Range, b),
            Err(DateSelectionError::InvalidMode)
        );
        assert_eq!(
            DateSelection::Range(DateRangeSelection::Start(CivilDate::MIN))
                .validate(DatePickerMode::Range, b),
            Err(DateSelectionError::OutOfBounds)
        );
    }
}
