//! Explicit proleptic-Gregorian civil values and fixed-offset conversion.
//! No clock, locale lookup, timezone database or serde dependency is installed.
use std::{fmt, str::FromStr};

pub const MIN_DATE_YEAR: u16 = 1;
pub const MAX_DATE_YEAR: u16 = 9999;
const UNIX_EPOCH_ORDINAL: i64 = 719_162;
const MAX_ORDINAL: u32 = 3_652_058;
const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateError {
    InvalidDate,
    InvalidTime,
    InvalidOffset,
    InvalidText,
    InvalidBounds,
    OutOfRange,
}
impl fmt::Display for DateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidDate => "invalid finite Gregorian date",
            Self::InvalidTime => "invalid local time",
            Self::InvalidOffset => "invalid fixed timezone offset",
            Self::InvalidText => "invalid exact ISO civil text",
            Self::InvalidBounds => "date bounds are reversed",
            Self::OutOfRange => "date arithmetic exceeds the supported range",
        })
    }
}
impl std::error::Error for DateError {}
type DateResult<T> = std::result::Result<T, DateError>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Weekday {
    #[default]
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
impl Weekday {
    pub const ALL: [Self; 7] = [
        Self::Monday,
        Self::Tuesday,
        Self::Wednesday,
        Self::Thursday,
        Self::Friday,
        Self::Saturday,
        Self::Sunday,
    ];
    pub const fn index(self) -> u8 {
        match self {
            Self::Monday => 0,
            Self::Tuesday => 1,
            Self::Wednesday => 2,
            Self::Thursday => 3,
            Self::Friday => 4,
            Self::Saturday => 5,
            Self::Sunday => 6,
        }
    }
    pub const fn index_from(self, first: Self) -> u8 {
        (self.index() + 7 - first.index()) % 7
    }
}
pub const fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}
fn month_length(year: u16, month: u8) -> Option<u8> {
    Some(match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => return None,
    })
}
fn days_before_year(year: u16) -> u32 {
    let y = u32::from(year) - 1;
    365 * y + y / 4 - y / 100 + y / 400
}

/// A civil date, without a time, offset, timezone or implicit current clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CivilDate {
    year: u16,
    month: u8,
    day: u8,
}
impl CivilDate {
    pub const MIN: Self = Self {
        year: MIN_DATE_YEAR,
        month: 1,
        day: 1,
    };
    pub const MAX: Self = Self {
        year: MAX_DATE_YEAR,
        month: 12,
        day: 31,
    };
    pub fn new(year: u16, month: u8, day: u8) -> DateResult<Self> {
        if !(MIN_DATE_YEAR..=MAX_DATE_YEAR).contains(&year)
            || day == 0
            || month_length(year, month).is_none_or(|last| day > last)
        {
            return Err(DateError::InvalidDate);
        }
        Ok(Self { year, month, day })
    }
    pub const fn year(self) -> u16 {
        self.year
    }
    pub const fn month(self) -> u8 {
        self.month
    }
    pub const fn day(self) -> u8 {
        self.day
    }
    pub const fn year_month(self) -> YearMonth {
        YearMonth {
            year: self.year,
            month: self.month,
        }
    }
    /// Zero-based days since 0001-01-01, which is a Monday.
    pub fn ordinal(self) -> u32 {
        let mut days = days_before_year(self.year);
        for month in 1..self.month {
            days += u32::from(month_length(self.year, month).unwrap());
        }
        days + u32::from(self.day) - 1
    }
    pub fn from_ordinal(ordinal: u32) -> DateResult<Self> {
        if ordinal > MAX_ORDINAL {
            return Err(DateError::OutOfRange);
        }
        // At most fourteen year comparisons and twelve month comparisons.
        // This never enumerates all earlier dates or retains a date cache.
        let (mut low, mut high) = (1u16, MAX_DATE_YEAR);
        while low < high {
            let middle = low + (high - low).div_ceil(2);
            if days_before_year(middle) <= ordinal {
                low = middle
            } else {
                high = middle - 1
            }
        }
        let mut day = ordinal - days_before_year(low);
        let mut month = 1;
        while day >= u32::from(month_length(low, month).unwrap()) {
            day -= u32::from(month_length(low, month).unwrap());
            month += 1;
        }
        Ok(Self {
            year: low,
            month,
            day: day as u8 + 1,
        })
    }
    pub fn weekday(self) -> Weekday {
        Weekday::ALL[(self.ordinal() % 7) as usize]
    }
    pub fn checked_add_days(self, days: i32) -> DateResult<Self> {
        let ordinal = i64::from(self.ordinal()) + i64::from(days);
        if !(0..=i64::from(MAX_ORDINAL)).contains(&ordinal) {
            return Err(DateError::OutOfRange);
        }
        Self::from_ordinal(ordinal as u32)
    }
    pub fn days_until(self, other: Self) -> i32 {
        other.ordinal() as i32 - self.ordinal() as i32
    }
    pub fn add_months(self, months: i32, rollover: MonthRollover) -> DateResult<Self> {
        let month = self.year_month().checked_add(months)?;
        let last = month.days();
        let day = if self.day <= last {
            self.day
        } else if rollover == MonthRollover::Clamp {
            last
        } else {
            return Err(DateError::InvalidDate);
        };
        Ok(Self {
            year: month.year,
            month: month.month,
            day,
        })
    }
    pub fn parse_iso(text: &str) -> DateResult<Self> {
        let b = text.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return Err(DateError::InvalidText);
        }
        Self::new(
            digits(&b[..4])? as u16,
            digits(&b[5..7])? as u8,
            digits(&b[8..])? as u8,
        )
    }
}
impl fmt::Display for CivilDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}
impl FromStr for CivilDate {
    type Err = DateError;
    fn from_str(s: &str) -> DateResult<Self> {
        Self::parse_iso(s)
    }
}

/// Month shifts explicitly choose whether a missing day clamps or rejects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonthRollover {
    Clamp,
    Reject,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct YearMonth {
    year: u16,
    month: u8,
}
impl YearMonth {
    pub fn new(year: u16, month: u8) -> DateResult<Self> {
        CivilDate::new(year, month, 1).map(CivilDate::year_month)
    }
    pub const fn year(self) -> u16 {
        self.year
    }
    pub const fn month(self) -> u8 {
        self.month
    }
    pub fn days(self) -> u8 {
        month_length(self.year, self.month).unwrap()
    }
    pub fn first(self) -> CivilDate {
        CivilDate {
            year: self.year,
            month: self.month,
            day: 1,
        }
    }
    pub fn last(self) -> CivilDate {
        CivilDate {
            year: self.year,
            month: self.month,
            day: self.days(),
        }
    }
    pub fn checked_add(self, months: i32) -> DateResult<Self> {
        let index = (i64::from(self.year) - 1) * 12 + i64::from(self.month) - 1 + i64::from(months);
        if !(0..i64::from(MAX_DATE_YEAR) * 12).contains(&index) {
            return Err(DateError::OutOfRange);
        }
        Ok(Self {
            year: (index / 12 + 1) as u16,
            month: (index % 12 + 1) as u8,
        })
    }
}
impl fmt::Display for YearMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateBounds {
    min: Option<CivilDate>,
    max: Option<CivilDate>,
}
impl DateBounds {
    pub fn new(min: Option<CivilDate>, max: Option<CivilDate>) -> DateResult<Self> {
        if min.zip(max).is_some_and(|(a, b)| a > b) {
            return Err(DateError::InvalidBounds);
        }
        Ok(Self { min, max })
    }
    pub const fn min(self) -> Option<CivilDate> {
        self.min
    }
    pub const fn max(self) -> Option<CivilDate> {
        self.max
    }
    pub fn contains(self, date: CivilDate) -> bool {
        self.min.is_none_or(|d| date >= d) && self.max.is_none_or(|d| date <= d)
    }
    pub fn clamp(self, date: CivilDate) -> CivilDate {
        date.max(self.min.unwrap_or(CivilDate::MIN))
            .min(self.max.unwrap_or(CivilDate::MAX))
    }
    pub fn month_allowed(self, month: YearMonth) -> bool {
        self.min.is_none_or(|d| month.last() >= d) && self.max.is_none_or(|d| month.first() <= d)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CivilRange {
    start: CivilDate,
    end: CivilDate,
}
impl CivilRange {
    pub fn new(start: CivilDate, end: CivilDate) -> DateResult<Self> {
        if start > end {
            Err(DateError::InvalidBounds)
        } else {
            Ok(Self { start, end })
        }
    }
    pub const fn start(self) -> CivilDate {
        self.start
    }
    pub const fn end(self) -> CivilDate {
        self.end
    }
    pub fn contains(self, date: CivilDate) -> bool {
        self.start <= date && date <= self.end
    }
    pub fn len_days(self) -> u32 {
        self.start.days_until(self.end) as u32 + 1
    }
}

/// Exactly six current rows, with unrepresentable boundary cells left empty.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonthGrid {
    month: YearMonth,
    week_start: Weekday,
    cells: [Option<CivilDate>; 42],
}
impl MonthGrid {
    pub fn new(month: YearMonth, week_start: Weekday) -> Self {
        let first = month.first();
        let start = i64::from(first.ordinal()) - i64::from(first.weekday().index_from(week_start));
        let cells = std::array::from_fn(|i| {
            let ordinal = start + i as i64;
            if (0..=i64::from(MAX_ORDINAL)).contains(&ordinal) {
                CivilDate::from_ordinal(ordinal as u32).ok()
            } else {
                None
            }
        });
        Self {
            month,
            week_start,
            cells,
        }
    }
    pub const fn month(self) -> YearMonth {
        self.month
    }
    pub const fn week_start(self) -> Weekday {
        self.week_start
    }
    pub fn cells(&self) -> &[Option<CivilDate>; 42] {
        &self.cells
    }
    pub fn index_of(&self, date: CivilDate) -> Option<usize> {
        self.cells.iter().position(|d| *d == Some(date))
    }
}

/// Wall-clock time without a date, timezone, DST rule or leap second.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalTime {
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
}
impl LocalTime {
    pub fn new(hour: u8, minute: u8, second: u8) -> DateResult<Self> {
        Self::with_nanosecond(hour, minute, second, 0)
    }
    pub fn with_nanosecond(hour: u8, minute: u8, second: u8, nanosecond: u32) -> DateResult<Self> {
        if hour > 23 || minute > 59 || second > 59 || nanosecond >= 1_000_000_000 {
            return Err(DateError::InvalidTime);
        }
        Ok(Self {
            hour,
            minute,
            second,
            nanosecond,
        })
    }
    pub const fn hour(self) -> u8 {
        self.hour
    }
    pub const fn minute(self) -> u8 {
        self.minute
    }
    pub const fn second(self) -> u8 {
        self.second
    }
    pub const fn nanosecond(self) -> u32 {
        self.nanosecond
    }
    pub fn seconds_from_midnight(self) -> u32 {
        u32::from(self.hour) * 3600 + u32::from(self.minute) * 60 + u32::from(self.second)
    }
    pub fn parse_iso(text: &str) -> DateResult<Self> {
        let b = text.as_bytes();
        if !matches!(b.len(), 5 | 8 | 10..=18) || b[2] != b':' {
            return Err(DateError::InvalidText);
        }
        let hour = digits(&b[..2])? as u8;
        let minute = digits(&b[3..5])? as u8;
        let second = if b.len() > 5 {
            if b[5] != b':' {
                return Err(DateError::InvalidText);
            }
            digits(&b[6..8])? as u8
        } else {
            0
        };
        let nano = if b.len() > 8 {
            if b[8] != b'.' {
                return Err(DateError::InvalidText);
            }
            digits(&b[9..])? * 10u32.pow((18 - b.len()) as u32)
        } else {
            0
        };
        Self::with_nanosecond(hour, minute, second, nano)
    }
}
impl fmt::Display for LocalTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)?;
        if self.nanosecond != 0 {
            let mut value = self.nanosecond;
            let mut width = 9;
            while value.is_multiple_of(10) {
                value /= 10;
                width -= 1;
            }
            write!(f, ".{value:0width$}")?;
        }
        Ok(())
    }
}
impl FromStr for LocalTime {
    type Err = DateError;
    fn from_str(s: &str) -> DateResult<Self> {
        Self::parse_iso(s)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalDateTime {
    date: CivilDate,
    time: LocalTime,
}
impl LocalDateTime {
    pub const fn new(date: CivilDate, time: LocalTime) -> Self {
        Self { date, time }
    }
    pub const fn date(self) -> CivilDate {
        self.date
    }
    pub const fn time(self) -> LocalTime {
        self.time
    }
    pub fn resolve_fixed(self, offset: FixedOffset) -> UnixInstant {
        UnixInstant {
            seconds: (i64::from(self.date.ordinal()) - UNIX_EPOCH_ORDINAL) * SECONDS_PER_DAY
                + i64::from(self.time.seconds_from_midnight())
                - i64::from(offset.minutes) * 60,
            nanosecond: self.time.nanosecond,
        }
    }
}
/// A supplied constant offset, not an IANA timezone and not an unknown offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FixedOffset {
    minutes: i16,
}
impl FixedOffset {
    pub const UTC: Self = Self { minutes: 0 };
    pub fn new(minutes: i16) -> DateResult<Self> {
        if !(-1439..=1439).contains(&minutes) {
            Err(DateError::InvalidOffset)
        } else {
            Ok(Self { minutes })
        }
    }
    pub const fn minutes(self) -> i16 {
        self.minutes
    }
    pub fn parse_iso(text: &str) -> DateResult<Self> {
        if text == "Z" {
            return Ok(Self::UTC);
        }
        let b = text.as_bytes();
        if b.len() != 6 || !matches!(b[0], b'+' | b'-') || b[3] != b':' {
            return Err(DateError::InvalidText);
        }
        let hours = digits(&b[1..3])?;
        let minutes = digits(&b[4..])?;
        if hours > 23 || minutes > 59 {
            return Err(DateError::InvalidOffset);
        }
        let value = (hours * 60 + minutes) as i16;
        Self::new(if b[0] == b'-' { -value } else { value })
    }
}
impl fmt::Display for FixedOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.minutes.unsigned_abs();
        write!(
            f,
            "{}{:02}:{:02}",
            if self.minutes < 0 { "-" } else { "+" },
            value / 60,
            value % 60
        )
    }
}
/// Absolute Unix seconds plus a nonnegative fractional nanosecond. No timezone
/// is inferred, and conversion outside the finite civil range returns an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnixInstant {
    seconds: i64,
    nanosecond: u32,
}
impl UnixInstant {
    pub fn new(seconds: i64, nanosecond: u32) -> DateResult<Self> {
        if nanosecond >= 1_000_000_000 {
            Err(DateError::InvalidTime)
        } else {
            Ok(Self {
                seconds,
                nanosecond,
            })
        }
    }
    pub const fn seconds(self) -> i64 {
        self.seconds
    }
    pub const fn nanosecond(self) -> u32 {
        self.nanosecond
    }
    pub fn to_local(self, offset: FixedOffset) -> DateResult<LocalDateTime> {
        let seconds = self
            .seconds
            .checked_add(i64::from(offset.minutes) * 60)
            .ok_or(DateError::OutOfRange)?;
        let ordinal = seconds
            .div_euclid(SECONDS_PER_DAY)
            .checked_add(UNIX_EPOCH_ORDINAL)
            .ok_or(DateError::OutOfRange)?;
        if !(0..=i64::from(MAX_ORDINAL)).contains(&ordinal) {
            return Err(DateError::OutOfRange);
        }
        let s = seconds.rem_euclid(SECONDS_PER_DAY) as u32;
        Ok(LocalDateTime::new(
            CivilDate::from_ordinal(ordinal as u32)?,
            LocalTime::with_nanosecond(
                (s / 3600) as u8,
                (s / 60 % 60) as u8,
                (s % 60) as u8,
                self.nanosecond,
            )?,
        ))
    }
}
fn digits(bytes: &[u8]) -> DateResult<u32> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
        return Err(DateError::InvalidText);
    }
    Ok(bytes.iter().fold(0, |n, b| n * 10 + u32::from(b - b'0')))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DateLocale {
    #[default]
    En,
    ZhCn,
}
impl DateLocale {
    pub fn weekday_short(self, day: Weekday) -> &'static str {
        match self {
            Self::En => ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][day.index() as usize],
            Self::ZhCn => ["一", "二", "三", "四", "五", "六", "日"][day.index() as usize],
        }
    }
    pub fn weekday_long(self, day: Weekday) -> &'static str {
        match self {
            Self::En => [
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday",
                "Sunday",
            ][day.index() as usize],
            Self::ZhCn => [
                "星期一",
                "星期二",
                "星期三",
                "星期四",
                "星期五",
                "星期六",
                "星期日",
            ][day.index() as usize],
        }
    }
    pub fn month_name(self, month: YearMonth) -> &'static str {
        match self {
            Self::En => [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ][month.month as usize - 1],
            Self::ZhCn => [
                "一月",
                "二月",
                "三月",
                "四月",
                "五月",
                "六月",
                "七月",
                "八月",
                "九月",
                "十月",
                "十一月",
                "十二月",
            ][month.month as usize - 1],
        }
    }
    pub fn month_label(self, month: YearMonth) -> String {
        match self {
            Self::En => format!("{} {}", self.month_name(month), month.year),
            Self::ZhCn => format!("{}年{}月", month.year, month.month),
        }
    }
    pub fn date_label(self, date: CivilDate) -> String {
        match self {
            Self::En => format!(
                "{}, {} {}, {}",
                self.weekday_long(date.weekday()),
                self.month_name(date.year_month()),
                date.day,
                date.year
            ),
            Self::ZhCn => format!(
                "{}年{}月{}日 {}",
                date.year,
                date.month,
                date.day,
                self.weekday_long(date.weekday())
            ),
        }
    }
}
/// Explicit copyable settings. Supply a reference date for SSR/client parity;
/// today is a separate optional value and never sampled from the host clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DatesProvider {
    reference: CivilDate,
    today: Option<CivilDate>,
    locale: DateLocale,
    week_start: Weekday,
    bounds: DateBounds,
}
impl DatesProvider {
    pub const fn new(reference: CivilDate) -> Self {
        Self {
            reference,
            today: None,
            locale: DateLocale::En,
            week_start: Weekday::Monday,
            bounds: DateBounds {
                min: None,
                max: None,
            },
        }
    }
    pub const fn locale(mut self, locale: DateLocale) -> Self {
        self.locale = locale;
        self
    }
    pub const fn week_start(mut self, day: Weekday) -> Self {
        self.week_start = day;
        self
    }
    pub const fn today(mut self, today: Option<CivilDate>) -> Self {
        self.today = today;
        self
    }
    pub const fn bounds(mut self, bounds: DateBounds) -> Self {
        self.bounds = bounds;
        self
    }
    pub const fn reference_date(self) -> CivilDate {
        self.reference
    }
    pub const fn today_date(self) -> Option<CivilDate> {
        self.today
    }
    pub const fn locale_value(self) -> DateLocale {
        self.locale
    }
    pub const fn first_weekday(self) -> Weekday {
        self.week_start
    }
    pub const fn date_bounds(self) -> DateBounds {
        self.bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn d(s: &str) -> CivilDate {
        CivilDate::parse_iso(s).unwrap()
    }
    #[test]
    fn exact_iso_and_leap_century_rules_reject_ambiguous_or_oversized_text() {
        for s in [
            "0000-01-01",
            "2024-00-01",
            "2024-13-01",
            "2024-01-00",
            "2023-02-29",
            "1900-02-29",
            "2024-2-01",
            "2024-01-01Z",
            "２０２４-01-01",
        ] {
            assert!(CivilDate::parse_iso(s).is_err(), "{s}");
        }
        assert_eq!(d("2000-02-29").day(), 29);
        assert_eq!(d("2024-02-29").weekday(), Weekday::Thursday);
        assert!(CivilDate::parse_iso(&"1".repeat(1_000_000)).is_err());
    }
    #[test]
    fn full_gregorian_cycle_roundtrips_and_every_supported_month_boundary_is_ordered() {
        for ordinal in 0..146_097 {
            let date = CivilDate::from_ordinal(ordinal).unwrap();
            assert_eq!(date.ordinal(), ordinal);
            assert_eq!(date.weekday().index(), (ordinal % 7) as u8);
        }
        for year in 1..=9999 {
            for month in 1..=12 {
                let m = YearMonth::new(year, month).unwrap();
                for date in [m.first(), m.last()] {
                    assert_eq!(CivilDate::from_ordinal(date.ordinal()).unwrap(), date);
                    assert_eq!(date.to_string().parse::<CivilDate>().unwrap(), date);
                }
            }
        }
        assert_eq!(CivilDate::MAX.ordinal(), MAX_ORDINAL);
        assert_eq!(d("1970-01-01").ordinal(), UNIX_EPOCH_ORDINAL as u32);
    }
    #[test]
    fn arithmetic_has_checked_edges_and_explicit_month_rollover() {
        assert_eq!(
            d("2024-01-31").add_months(1, MonthRollover::Clamp).unwrap(),
            d("2024-02-29")
        );
        assert!(
            d("2024-01-31")
                .add_months(1, MonthRollover::Reject)
                .is_err()
        );
        assert_eq!(
            d("2024-03-31")
                .add_months(-1, MonthRollover::Clamp)
                .unwrap(),
            d("2024-02-29")
        );
        assert_eq!(
            d("2024-02-28").checked_add_days(2).unwrap(),
            d("2024-03-01")
        );
        for (date, delta) in [
            (CivilDate::MIN, -1),
            (CivilDate::MAX, 1),
            (CivilDate::MIN, i32::MIN),
            (CivilDate::MAX, i32::MAX),
        ] {
            assert!(date.checked_add_days(delta).is_err());
        }
        assert!(CivilDate::MAX.year_month().checked_add(1).is_err());
        assert!(CivilDate::MIN.year_month().checked_add(-1).is_err());
    }
    #[test]
    fn month_grids_are_constant_size_aligned_and_do_not_wrap_at_year_limits() {
        for first in Weekday::ALL {
            for month in 1..=12 {
                let m = YearMonth::new(2024, month).unwrap();
                let grid = MonthGrid::new(m, first);
                assert_eq!(grid.cells().len(), 42);
                assert_eq!(grid.cells()[0].unwrap().weekday(), first);
                assert_eq!(
                    grid.index_of(m.first()),
                    Some(m.first().weekday().index_from(first) as usize)
                );
                for pair in grid.cells().windows(2) {
                    assert_eq!(pair[0].unwrap().days_until(pair[1].unwrap()), 1);
                }
            }
        }
        assert!(
            MonthGrid::new(YearMonth::new(1, 1).unwrap(), Weekday::Sunday).cells()[0].is_none()
        );
        assert!(
            MonthGrid::new(YearMonth::new(9999, 12).unwrap(), Weekday::Monday)
                .cells()
                .last()
                .unwrap()
                .is_none()
        );
    }
    #[test]
    fn local_time_parsing_and_fractional_precision_have_no_leap_second_or_implicit_zone() {
        for s in [
            "24:00",
            "00:60",
            "12:00:60",
            "1:00",
            "01:00Z",
            "01:00:00.",
            "01:00:00.1234567890",
            "01:00:00.１２",
        ] {
            assert!(LocalTime::parse_iso(s).is_err(), "{s}");
        }
        assert_eq!(
            LocalTime::parse_iso("09:07").unwrap().to_string(),
            "09:07:00"
        );
        assert_eq!(
            LocalTime::parse_iso("09:07:01.000000001")
                .unwrap()
                .nanosecond(),
            1
        );
        assert_eq!(
            LocalTime::parse_iso("09:07:01.1200").unwrap().to_string(),
            "09:07:01.12"
        );
    }
    #[test]
    fn fixed_offsets_and_negative_unix_instants_convert_exactly_without_dst_inference() {
        let local = LocalDateTime::new(d("1970-01-01"), LocalTime::new(8, 0, 0).unwrap());
        let offset = FixedOffset::parse_iso("+08:00").unwrap();
        assert_eq!(local.resolve_fixed(offset), UnixInstant::new(0, 0).unwrap());
        let negative = UnixInstant::new(-1, 999_999_999)
            .unwrap()
            .to_local(FixedOffset::UTC)
            .unwrap();
        assert_eq!(negative.date(), d("1969-12-31"));
        assert_eq!(negative.time().to_string(), "23:59:59.999999999");
        for date in [
            CivilDate::MIN,
            d("1900-03-01"),
            d("2000-02-29"),
            CivilDate::MAX,
        ] {
            for minutes in [-1439, -480, 0, 330, 1439] {
                let offset = FixedOffset::new(minutes).unwrap();
                let local =
                    LocalDateTime::new(date, LocalTime::with_nanosecond(23, 59, 59, 123).unwrap());
                assert_eq!(local.resolve_fixed(offset).to_local(offset).unwrap(), local);
            }
        }
        assert!(
            UnixInstant::new(i64::MAX, 0)
                .unwrap()
                .to_local(FixedOffset::new(1).unwrap())
                .is_err()
        );
        assert!(
            UnixInstant::new(i64::MIN, 0)
                .unwrap()
                .to_local(FixedOffset::new(-1).unwrap())
                .is_err()
        );
        for s in ["+24:00", "+00:60", "UTC", "+8:00", "America/New_York"] {
            assert!(FixedOffset::parse_iso(s).is_err());
        }
    }
    #[test]
    fn bounds_ranges_and_explicit_provider_keep_locale_and_week_start_independent() {
        let min = d("2024-02-10");
        let max = d("2024-03-03");
        let b = DateBounds::new(Some(min), Some(max)).unwrap();
        assert!(!b.contains(d("2024-02-09")));
        assert!(b.contains(min));
        assert!(b.contains(max));
        assert_eq!(b.clamp(CivilDate::MIN), min);
        assert_eq!(b.clamp(CivilDate::MAX), max);
        assert!(b.month_allowed(YearMonth::new(2024, 2).unwrap()));
        assert!(!b.month_allowed(YearMonth::new(2024, 4).unwrap()));
        assert!(DateBounds::new(Some(max), Some(min)).is_err());
        assert_eq!(CivilRange::new(min, max).unwrap().len_days(), 23);
        assert_eq!(CivilRange::new(min, min).unwrap().len_days(), 1);
        assert!(CivilRange::new(max, min).is_err());
        let p = DatesProvider::new(min)
            .locale(DateLocale::ZhCn)
            .week_start(Weekday::Sunday)
            .today(Some(max))
            .bounds(b);
        assert_eq!(p.reference_date(), min);
        assert_eq!(p.today_date(), Some(max));
        assert_eq!(p.first_weekday(), Weekday::Sunday);
        assert_eq!(p.locale_value().month_label(min.year_month()), "2024年2月");
        assert_eq!(
            DateLocale::En.date_label(d("2024-02-29")),
            "Thursday, February 29, 2024"
        );
        assert_eq!(DatesProvider::new(min).today_date(), None);
    }
}
