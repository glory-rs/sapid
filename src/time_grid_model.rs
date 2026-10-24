//! Arithmetic time slot window. Admission precedes any slot allocation.
use crate::{DateError, LocalTime};
pub const MAX_TIME_GRID_ITEMS: usize = 288;
pub const MAX_DISABLED_TIMES: usize = 128;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeGridSpec {
    start: LocalTime,
    end: LocalTime,
    step: u32,
    columns: u8,
    count: u16,
}
impl Default for TimeGridSpec {
    fn default() -> Self {
        Self::new(
            LocalTime::new(0, 0, 0).unwrap(),
            LocalTime::new(23, 30, 0).unwrap(),
            1800,
            4,
        )
        .unwrap()
    }
}
impl TimeGridSpec {
    pub fn new(
        start: LocalTime,
        end: LocalTime,
        step_seconds: u32,
        columns: u8,
    ) -> Result<Self, DateError> {
        if start.nanosecond() != 0
            || end.nanosecond() != 0
            || start > end
            || step_seconds == 0
            || step_seconds > 86400
            || !(1..=12).contains(&columns)
        {
            return Err(DateError::InvalidTime);
        }
        let count =
            (end.seconds_from_midnight() - start.seconds_from_midnight()) / step_seconds + 1;
        if count > MAX_TIME_GRID_ITEMS as u32 {
            return Err(DateError::OutOfRange);
        }
        Ok(Self {
            start,
            end,
            step: step_seconds,
            columns,
            count: count as u16,
        })
    }
    pub const fn start(self) -> LocalTime {
        self.start
    }
    pub const fn end(self) -> LocalTime {
        self.end
    }
    pub const fn step_seconds(self) -> u32 {
        self.step
    }
    pub const fn columns(self) -> usize {
        self.columns as usize
    }
    pub const fn len(self) -> usize {
        self.count as usize
    }
    pub const fn is_empty(self) -> bool {
        false
    }
    pub fn rows(self) -> usize {
        self.len().div_ceil(self.columns())
    }
    pub fn at(self, index: usize) -> Option<LocalTime> {
        if index >= self.len() {
            return None;
        }
        let seconds = self.start.seconds_from_midnight() + index as u32 * self.step;
        LocalTime::new(
            (seconds / 3600) as u8,
            ((seconds % 3600) / 60) as u8,
            (seconds % 60) as u8,
        )
        .ok()
    }
    pub fn index_of(self, value: LocalTime) -> Option<usize> {
        if value.nanosecond() != 0 || value < self.start || value > self.end {
            return None;
        }
        let delta = value.seconds_from_midnight() - self.start.seconds_from_midnight();
        if delta.is_multiple_of(self.step) {
            let at = (delta / self.step) as usize;
            if at < self.len() {
                return Some(at);
            }
        }
        None
    }
    /// A bounded arithmetic preference; this does not make an unavailable slot valid.
    pub fn nearest_index(self, value: LocalTime) -> usize {
        let delta = value
            .seconds_from_midnight()
            .saturating_sub(self.start.seconds_from_midnight());
        ((delta + self.step / 2) / self.step).min(u32::from(self.count) - 1) as usize
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn t(v: &str) -> LocalTime {
        LocalTime::parse_iso(v).unwrap()
    }
    #[test]
    fn grid_admission_is_arithmetic_and_never_materializes_a_day_of_seconds() {
        assert!(TimeGridSpec::new(t("00:00"), t("23:59:59"), 1, 4).is_err());
        for step in [0, 86401] {
            assert!(TimeGridSpec::new(t("00:00"), t("23:59"), step, 4).is_err());
        }
        for cols in [0, 13] {
            assert!(TimeGridSpec::new(t("00:00"), t("23:59"), 300, cols).is_err());
        }
        let s = TimeGridSpec::new(t("00:00"), t("23:59:59"), 300, 12).unwrap();
        assert_eq!(s.len(), 288);
        assert_eq!(s.rows(), 24);
        assert_eq!(s.at(287), Some(t("23:55")));
        assert_eq!(s.at(288), None);
    }
    #[test]
    fn slots_keep_real_seconds_and_do_not_wrap_or_invent_a_final_endpoint() {
        let s = TimeGridSpec::new(t("09:00:01"), t("09:01:00"), 17, 3).unwrap();
        assert_eq!(s.len(), 4);
        assert_eq!(s.at(3), Some(t("09:00:52")));
        assert_eq!(s.index_of(t("09:01:00")), None);
        assert_eq!(s.index_of(t("09:00:35")), Some(2));
        assert_eq!(s.nearest_index(t("23:59")), 3);
        assert!(TimeGridSpec::new(t("23:00"), t("01:00"), 1800, 4).is_err());
        assert!(
            TimeGridSpec::new(
                LocalTime::with_nanosecond(0, 0, 0, 1).unwrap(),
                t("01:00"),
                1800,
                4
            )
            .is_err()
        );
    }
}
