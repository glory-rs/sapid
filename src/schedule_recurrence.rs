//! Finite civil recurrence arithmetic; no instance history or timezone rules.
use super::schedule_model::ScheduleError;
use crate::{CivilDate, Weekday, YearMonth};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissingScheduleDay {
    Skip,
    Clamp,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecurrenceEnd {
    Never,
    Count(u32),
    Until(CivilDate),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecurrenceFrequency {
    Daily,
    Weekly { weekdays: u8, week_start: Weekday },
    Monthly { missing: MissingScheduleDay },
    Yearly { missing: MissingScheduleDay },
}
/// Count includes rule candidates before exception dates are removed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduleRecurrence {
    frequency: RecurrenceFrequency,
    interval: u16,
    end: RecurrenceEnd,
}
impl ScheduleRecurrence {
    pub fn new(
        frequency: RecurrenceFrequency,
        interval: u16,
        end: RecurrenceEnd,
    ) -> Result<Self, ScheduleError> {
        if interval == 0
            || matches!(end, RecurrenceEnd::Count(0 | 1_000_001..))
            || matches!(
                frequency,
                RecurrenceFrequency::Weekly {
                    weekdays: 0 | 128..,
                    ..
                }
            )
        {
            return Err(ScheduleError::InvalidRecurrence);
        }
        Ok(Self {
            frequency,
            interval,
            end,
        })
    }
    pub const fn frequency(self) -> RecurrenceFrequency {
        self.frequency
    }
    pub const fn interval(self) -> u16 {
        self.interval
    }
    pub const fn end(self) -> RecurrenceEnd {
        self.end
    }
    pub(crate) fn validate_anchor(self, anchor: CivilDate) -> Result<(), ScheduleError> {
        if matches!(self.end,RecurrenceEnd::Until(end) if end<anchor) {
            Err(ScheduleError::InvalidRecurrence)
        } else {
            Ok(())
        }
    }
    /// The candidate index is calculated directly, including COUNT for skipped
    /// month-end/leap dates. Gregorian counting examines at most one 400-year
    /// pattern (4800 month slots), independent of distance from the anchor.
    pub fn index_on(self, anchor: CivilDate, date: CivilDate) -> Option<u32> {
        if date < anchor || matches!(self.end,RecurrenceEnd::Until(end)if date>end) {
            return None;
        }
        let interval = u32::from(self.interval);
        let index = match self.frequency {
            RecurrenceFrequency::Daily => {
                let delta = anchor.days_until(date) as u32;
                if !delta.is_multiple_of(interval) {
                    return None;
                }
                delta / interval
            }
            RecurrenceFrequency::Weekly {
                weekdays,
                week_start,
            } => {
                if weekdays & (1 << date.weekday().index()) == 0 {
                    return None;
                }
                let a = i64::from(anchor.ordinal())
                    - i64::from(anchor.weekday().index_from(week_start));
                let b = i64::from(date.ordinal())
                    - i64::from(date.weekday().index_from(week_start));
                let weeks = ((b - a) / 7) as u32;
                if !weeks.is_multiple_of(interval) {
                    return None;
                }
                let before = |limit: u8| -> u32 {
                    (0..limit)
                        .filter(|pos| {
                            let weekday = (week_start.index() + pos) % 7;
                            weekdays & (1 << weekday) != 0
                        })
                        .count() as u32
                };
                let first_before = before(anchor.weekday().index_from(week_start));
                let in_week = before(date.weekday().index_from(week_start));
                let cycle = weeks / interval;
                if cycle == 0 {
                    in_week.checked_sub(first_before)?
                } else {
                    u32::from(weekdays.count_ones() as u8) * cycle - first_before + in_week
                }
            }
            RecurrenceFrequency::Monthly { missing } => {
                let a = month_index(anchor.year_month());
                let b = month_index(date.year_month());
                let delta = b - a;
                if !delta.is_multiple_of(interval) {
                    return None;
                }
                let day = match missing {
                    MissingScheduleDay::Skip => anchor.day(),
                    MissingScheduleDay::Clamp => anchor.day().min(date.year_month().days()),
                };
                if date.day() != day {
                    return None;
                }
                let n = delta / interval;
                match missing {
                    MissingScheduleDay::Clamp => n,
                    MissingScheduleDay::Skip => {
                        periodic_count(n, 4800 / gcd(interval, 4800), |at| {
                            let month = (a + at * interval) % 4800;
                            YearMonth::new((month / 12 + 1) as u16, (month % 12 + 1) as u8)
                                .unwrap()
                                .days()
                                >= anchor.day()
                        })
                    }
                }
            }
            RecurrenceFrequency::Yearly { missing } => {
                let delta = u32::from(date.year() - anchor.year());
                if !delta.is_multiple_of(interval) || date.month() != anchor.month() {
                    return None;
                }
                let day = match missing {
                    MissingScheduleDay::Skip => anchor.day(),
                    MissingScheduleDay::Clamp => anchor.day().min(date.year_month().days()),
                };
                if date.day() != day {
                    return None;
                }
                let n = delta / interval;
                match missing {
                    MissingScheduleDay::Clamp => n,
                    MissingScheduleDay::Skip => {
                        periodic_count(n, 400 / gcd(interval, 400), |at| {
                            let year = (u32::from(anchor.year()) - 1 + at * interval) % 400 + 1;
                            YearMonth::new(year as u16, anchor.month()).unwrap().days()
                                >= anchor.day()
                        })
                    }
                }
            }
        };
        if matches!(self.end,RecurrenceEnd::Count(count)if index>=count) {
            None
        } else {
            Some(index)
        }
    }
}
fn month_index(v: YearMonth) -> u32 {
    (u32::from(v.year()) - 1) * 12 + u32::from(v.month()) - 1
}
fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}
fn periodic_count(n: u32, period: u32, valid: impl Fn(u32) -> bool) -> u32 {
    let full = n / period;
    let rest = n % period;
    let mut count = 0;
    let mut remainder = 0;
    for at in 0..n.min(period) {
        if valid(at) {
            count += 1;
            if at < rest {
                remainder += 1;
            }
        }
    }
    if full == 0 {
        count
    } else {
        count * full + remainder
    }
}
