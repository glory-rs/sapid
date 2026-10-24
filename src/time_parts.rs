//! Finite segment draft, distinct from a complete wall-clock business value.
use crate::{HourCycle, LocalTime, TimeBounds, TimeFormat, TimePrecision};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimeParts {
    pub hour: Option<u8>,
    pub minute: Option<u8>,
    pub second: Option<u8>,
    pub afternoon: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimePickerStatus {
    Empty,
    Incomplete,
    Valid(LocalTime),
    Invalid,
    OutOfBounds(LocalTime),
    Unavailable(LocalTime),
    TooLong,
}
impl TimePickerStatus {
    pub fn proposal(self) -> Option<Option<LocalTime>> {
        match self {
            Self::Empty => Some(None),
            Self::Valid(v) => Some(Some(v)),
            _ => None,
        }
    }
    pub fn is_error(self) -> bool {
        matches!(
            self,
            Self::Invalid | Self::OutOfBounds(_) | Self::Unavailable(_) | Self::TooLong
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimePickerDraft {
    pub parts: TimeParts,
    pub status: TimePickerStatus,
}
impl TimeParts {
    pub fn from_time(
        value: Option<LocalTime>,
        format: TimeFormat,
    ) -> Result<Self, crate::DateError> {
        let Some(value) = value else {
            return Ok(Self::default());
        };
        format.validate(value)?;
        Ok(Self {
            hour: Some(if format.cycle == HourCycle::H12 {
                (value.hour() + 11) % 12 + 1
            } else {
                value.hour()
            }),
            minute: Some(value.minute()),
            second: if format.precision == TimePrecision::Second {
                Some(value.second())
            } else {
                None
            },
            afternoon: value.hour() >= 12,
        })
    }
    pub fn classify(self, format: TimeFormat, bounds: TimeBounds) -> TimePickerStatus {
        if format.precision == TimePrecision::Minute && self.second.is_some_and(|v| v != 0) {
            return TimePickerStatus::Invalid;
        }
        if self.hour.is_none()
            && self.minute.is_none()
            && (format.precision == TimePrecision::Minute || self.second.is_none())
        {
            return TimePickerStatus::Empty;
        }
        if self.hour.is_some_and(|h| {
            if format.cycle == HourCycle::H12 {
                h == 0 || h > 12
            } else {
                h > 23
            }
        }) || self.minute.is_some_and(|m| m > 59)
            || format.precision == TimePrecision::Second && self.second.is_some_and(|s| s > 59)
        {
            return TimePickerStatus::Invalid;
        }
        let (Some(mut hour), Some(minute)) = (self.hour, self.minute) else {
            return TimePickerStatus::Incomplete;
        };
        let second = if format.precision == TimePrecision::Second {
            let Some(second) = self.second else {
                return TimePickerStatus::Incomplete;
            };
            second
        } else {
            0
        };
        if format.cycle == HourCycle::H12 {
            hour = hour % 12 + if self.afternoon { 12 } else { 0 };
        }
        let value = LocalTime::new(hour, minute, second).unwrap();
        if bounds.contains(value) {
            TimePickerStatus::Valid(value)
        } else {
            TimePickerStatus::OutOfBounds(value)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_wall_clock_seconds_split_and_reassemble_without_precision_loss() {
        for cycle in [HourCycle::H12, HourCycle::H24] {
            let format = TimeFormat::new(cycle, TimePrecision::Second, crate::DateLocale::En);
            for n in 0..86400 {
                let v =
                    LocalTime::new((n / 3600) as u8, ((n % 3600) / 60) as u8, (n % 60) as u8)
                        .unwrap();
                assert_eq!(
                    TimeParts::from_time(Some(v), format)
                        .unwrap()
                        .classify(format, TimeBounds::default()),
                    TimePickerStatus::Valid(v)
                );
            }
        }
    }
    #[test]
    fn empty_incomplete_invalid_and_bounds_do_not_fabricate_missing_segments() {
        let f = TimeFormat::default();
        let b = TimeBounds::default();
        let mut p = TimeParts::default();
        assert_eq!(p.classify(f, b), TimePickerStatus::Empty);
        p.hour = Some(12);
        assert_eq!(p.classify(f, b), TimePickerStatus::Incomplete);
        p.minute = Some(60);
        assert_eq!(p.classify(f, b), TimePickerStatus::Invalid);
        p.minute = Some(30);
        let v = LocalTime::new(12, 30, 0).unwrap();
        assert_eq!(p.classify(f, b), TimePickerStatus::Valid(v));
        let b = TimeBounds::new(None, Some(LocalTime::new(12, 0, 0).unwrap())).unwrap();
        assert_eq!(p.classify(f, b), TimePickerStatus::OutOfBounds(v));
    }
}
