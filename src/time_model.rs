//! Finite wall-clock formatting and drafts, independent of a date or timezone.
use crate::{DateError, DateLocale, LocalTime};
pub const MAX_TIME_DRAFT_BYTES: usize = 32;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HourCycle {
    H12,
    #[default]
    H24,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimePrecision {
    #[default]
    Minute,
    Second,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeFormat {
    pub cycle: HourCycle,
    pub precision: TimePrecision,
    pub locale: DateLocale,
}
impl Default for TimeFormat {
    fn default() -> Self {
        Self::new(HourCycle::H24, TimePrecision::Minute, DateLocale::En)
    }
}
impl TimeFormat {
    pub const fn new(cycle: HourCycle, precision: TimePrecision, locale: DateLocale) -> Self {
        Self {
            cycle,
            precision,
            locale,
        }
    }
    /// Reject a value whose precision this display would lose.
    pub fn validate(self, value: LocalTime) -> Result<(), DateError> {
        if value.nanosecond() != 0
            || (self.precision == TimePrecision::Minute && value.second() != 0)
        {
            Err(DateError::InvalidTime)
        } else {
            Ok(())
        }
    }
    pub fn format(self, value: LocalTime) -> Result<String, DateError> {
        self.validate(value)?;
        let hour = if self.cycle == HourCycle::H12 {
            (value.hour() + 11) % 12 + 1
        } else {
            value.hour()
        };
        let mut result = format!("{hour:02}:{:02}", value.minute());
        if self.precision == TimePrecision::Second {
            result.push_str(&format!(":{:02}", value.second()));
        }
        if self.cycle == HourCycle::H12 {
            result.push(' ');
            result.push_str(self.marker(value.hour() >= 12));
        }
        Ok(result)
    }
    fn marker(self, afternoon: bool) -> &'static str {
        match (self.locale, afternoon) {
            (DateLocale::En, false) => "AM",
            (DateLocale::En, true) => "PM",
            (DateLocale::ZhCn, false) => "上午",
            (DateLocale::ZhCn, true) => "下午",
        }
    }
    /// Parse the exact configured display. Prefixes are incomplete, never a time.
    pub fn parse(self, text: &str) -> TimeParse {
        if text.is_empty() {
            return TimeParse::Empty;
        }
        if text.len() > MAX_TIME_DRAFT_BYTES {
            return TimeParse::Invalid;
        }
        let body = if self.precision == TimePrecision::Second {
            8
        } else {
            5
        };
        let bytes = text.as_bytes();
        for (at, &byte) in bytes.iter().take(body).enumerate() {
            if at == 2 || at == 5 {
                if byte != b':' {
                    return TimeParse::Invalid;
                }
            } else if !byte.is_ascii_digit() {
                return TimeParse::Invalid;
            }
        }
        if let Some(&first) = bytes.first()
            && first
                > if self.cycle == HourCycle::H12 {
                    b'1'
                } else {
                    b'2'
                }
        {
            return TimeParse::Invalid;
        }
        if bytes.len() >= 2 {
            let hour = (bytes[0] - b'0') * 10 + bytes[1] - b'0';
            if (self.cycle == HourCycle::H12 && !(1..=12).contains(&hour))
                || (self.cycle == HourCycle::H24 && hour > 23)
            {
                return TimeParse::Invalid;
            }
        }
        for at in [3, 6] {
            if at < body && bytes.get(at).is_some_and(|b| *b > b'5') {
                return TimeParse::Invalid;
            }
        }
        if bytes.len() < body {
            return TimeParse::Incomplete;
        }
        let hour = (bytes[0] - b'0') * 10 + bytes[1] - b'0';
        let minute = (bytes[3] - b'0') * 10 + bytes[4] - b'0';
        let second = if body == 8 {
            (bytes[6] - b'0') * 10 + bytes[7] - b'0'
        } else {
            0
        };
        let mut hour = hour;
        if self.cycle == HourCycle::H12 {
            let tail = &text[body..];
            if tail.is_empty() {
                return TimeParse::Incomplete;
            }
            if !tail.starts_with(' ') {
                return TimeParse::Invalid;
            }
            let marker = &tail[1..];
            if marker == self.marker(false) {
                hour %= 12;
            } else if marker == self.marker(true) {
                hour = hour % 12 + 12;
            } else if self.marker(false).starts_with(marker)
                || self.marker(true).starts_with(marker)
            {
                return TimeParse::Incomplete;
            } else {
                return TimeParse::Invalid;
            }
        } else if bytes.len() != body {
            return TimeParse::Invalid;
        }
        match LocalTime::new(hour, minute, second) {
            Ok(v) => TimeParse::Time(v),
            Err(_) => TimeParse::Invalid,
        }
    }
}
/// Inclusive same-day bounds. An overnight interval must be split explicitly.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimeBounds {
    min: Option<LocalTime>,
    max: Option<LocalTime>,
}
impl TimeBounds {
    pub fn new(min: Option<LocalTime>, max: Option<LocalTime>) -> Result<Self, DateError> {
        if min.zip(max).is_some_and(|(min, max)| min > max) {
            Err(DateError::OutOfRange)
        } else {
            Ok(Self { min, max })
        }
    }
    pub const fn min(self) -> Option<LocalTime> {
        self.min
    }
    pub const fn max(self) -> Option<LocalTime> {
        self.max
    }
    pub fn contains(self, value: LocalTime) -> bool {
        self.min.is_none_or(|min| value >= min) && self.max.is_none_or(|max| value <= max)
    }
    pub fn clamp(self, mut value: LocalTime) -> LocalTime {
        if let Some(min) = self.min {
            value = value.max(min);
        }
        if let Some(max) = self.max {
            value = value.min(max);
        }
        value
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeParse {
    Empty,
    Incomplete,
    Time(LocalTime),
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeDraftStatus {
    Empty,
    Incomplete,
    Valid(LocalTime),
    Invalid,
    OutOfBounds(LocalTime),
    TooLong,
}
impl TimeDraftStatus {
    pub fn proposal(self) -> Option<Option<LocalTime>> {
        match self {
            Self::Empty => Some(None),
            Self::Valid(v) => Some(Some(v)),
            _ => None,
        }
    }
    pub fn is_error(self) -> bool {
        matches!(self, Self::Invalid | Self::OutOfBounds(_) | Self::TooLong)
    }
}
pub fn classify_time_draft(text: &str, format: TimeFormat, bounds: TimeBounds) -> TimeDraftStatus {
    if text.len() > MAX_TIME_DRAFT_BYTES {
        return TimeDraftStatus::TooLong;
    }
    match format.parse(text) {
        TimeParse::Empty => TimeDraftStatus::Empty,
        TimeParse::Incomplete => TimeDraftStatus::Incomplete,
        TimeParse::Time(v) if bounds.contains(v) => TimeDraftStatus::Valid(v),
        TimeParse::Time(v) => TimeDraftStatus::OutOfBounds(v),
        TimeParse::Invalid => TimeDraftStatus::Invalid,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn time(h: u8, m: u8, s: u8) -> LocalTime {
        LocalTime::new(h, m, s).unwrap()
    }
    #[test]
    fn every_second_roundtrips_all_finite_locale_cycle_formats() {
        for locale in [DateLocale::En, DateLocale::ZhCn] {
            for cycle in [HourCycle::H12, HourCycle::H24] {
                let f = TimeFormat::new(cycle, TimePrecision::Second, locale);
                for seconds in 0..86400 {
                    let v = time(
                        (seconds / 3600) as u8,
                        ((seconds % 3600) / 60) as u8,
                        (seconds % 60) as u8,
                    );
                    let text = f.format(v).unwrap();
                    assert_eq!(f.parse(&text), TimeParse::Time(v));
                    for (at, _) in text.char_indices().skip(1) {
                        assert_eq!(
                            f.parse(&text[..at]),
                            TimeParse::Incomplete,
                            "{text} at {at}"
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn midnight_noon_seconds_and_precision_loss_are_explicit() {
        let f = TimeFormat::new(HourCycle::H12, TimePrecision::Minute, DateLocale::En);
        assert_eq!(f.format(time(0, 0, 0)).unwrap(), "12:00 AM");
        assert_eq!(f.format(time(12, 0, 0)).unwrap(), "12:00 PM");
        assert_eq!(f.format(time(23, 59, 0)).unwrap(), "11:59 PM");
        assert!(f.format(time(1, 2, 3)).is_err());
        assert!(
            f.format(LocalTime::with_nanosecond(1, 2, 0, 1).unwrap())
                .is_err()
        );
        for bad in [
            "00:00 AM",
            "13:00 PM",
            "12:60 AM",
            "12:00 am",
            "12:00AM",
            "12:00 AMx",
            "😀:00",
            "2",
            "24:00",
            "12:00\0",
        ] {
            assert_eq!(f.parse(bad), TimeParse::Invalid, "{bad:?}");
        }
        assert_eq!(f.parse("12:00"), TimeParse::Incomplete);
        assert_eq!(f.parse("12:00 P"), TimeParse::Incomplete);
    }
    #[test]
    fn minute_domain_all_1440_values_and_empty_drafts_are_exact() {
        let f = TimeFormat::default();
        for h in 0..24 {
            for m in 0..60 {
                let v = time(h, m, 0);
                assert_eq!(f.parse(&f.format(v).unwrap()), TimeParse::Time(v));
            }
        }
        assert_eq!(f.parse(""), TimeParse::Empty);
        for text in ["2", "23", "23:", "23:5"] {
            assert_eq!(f.parse(text), TimeParse::Incomplete);
        }
        for text in ["3", "24", "23:6", "23:59:", "-1:00", "1:00", "00:00:00"] {
            assert_eq!(f.parse(text), TimeParse::Invalid);
        }
    }
    #[test]
    fn same_day_bounds_do_not_invent_an_overnight_interval_or_clamp_a_draft() {
        let min = time(9, 0, 0);
        let max = time(17, 0, 0);
        let b = TimeBounds::new(Some(min), Some(max)).unwrap();
        assert!(TimeBounds::new(Some(max), Some(min)).is_err());
        assert_eq!(b.clamp(time(0, 0, 0)), min);
        assert_eq!(b.clamp(time(23, 59, 0)), max);
        let f = TimeFormat::default();
        assert_eq!(
            classify_time_draft("08:59", f, b),
            TimeDraftStatus::OutOfBounds(time(8, 59, 0))
        );
        assert_eq!(
            classify_time_draft("09:00", f, b),
            TimeDraftStatus::Valid(min)
        );
        assert_eq!(classify_time_draft("", f, b).proposal(), Some(None));
        assert_eq!(classify_time_draft("1", f, b).proposal(), None);
        assert_eq!(
            classify_time_draft(&"0".repeat(1_000_000), f, b),
            TimeDraftStatus::TooLong
        );
    }
}
