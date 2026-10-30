//! Pure timeline geometry over one admitted request window.
use crate::schedule_model::{midnight, ticks};
use crate::{
    CivilDate, LocalDateTime, LocalTime, MAX_SCHEDULE_OCCURRENCES, ScheduleData, ScheduleError,
    ScheduleKey, ScheduleOccurrence,
};
mod resource_month;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduleTimeWindow {
    start: u16,
    end: u16,
    step: u16,
    count: u16,
}
impl ScheduleTimeWindow {
    /// Minute coordinates may use 1440 only for the exclusive end.
    pub fn new(start: u16, end: u16, step: u16) -> Result<Self, ScheduleError> {
        if start >= end
            || end > 1440
            || step == 0
            || step > 240
            || !(end - start).is_multiple_of(step)
            || (end - start) / step > 96
        {
            return Err(ScheduleError::InvalidInterval);
        }
        Ok(Self {
            start,
            end,
            step,
            count: (end - start) / step,
        })
    }
    pub const fn start_minutes(self) -> u16 {
        self.start
    }
    pub const fn end_minutes(self) -> u16 {
        self.end
    }
    pub const fn step_minutes(self) -> u16 {
        self.step
    }
    pub const fn len(self) -> usize {
        self.count as usize
    }
    pub const fn is_empty(self) -> bool {
        false
    }
    pub fn at(
        self,
        date: CivilDate,
        index: usize,
    ) -> Result<crate::ScheduleInterval, ScheduleError> {
        if index >= self.len() {
            return Err(ScheduleError::InvalidInterval);
        }
        let start = self.start + index as u16 * self.step;
        let end = start + self.step;
        crate::ScheduleInterval::new(at_minute(date, start)?, at_minute(date, end)?)
    }
}
impl Default for ScheduleTimeWindow {
    fn default() -> Self {
        Self::new(480, 1080, 30).unwrap()
    }
}
/// One live resource/date column; indices refer to the current ResourceScheduleData.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceScheduleColumn {
    resource: usize,
    day: u8,
    date: CivilDate,
}
impl ResourceScheduleColumn {
    pub const fn resource_index(self) -> usize {
        self.resource
    }
    pub const fn day_index(self) -> usize {
        self.day as usize
    }
    pub const fn date(self) -> CivilDate {
        self.date
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResourceSchedulePlacement {
    column: usize,
    position: ScheduleDayPlacement,
}
impl ResourceSchedulePlacement {
    pub const fn column(self) -> usize {
        self.column
    }
    pub const fn position(self) -> ScheduleDayPlacement {
        self.position
    }
}
/// Finite geometry for the admitted resource window, with no business record copies.
#[derive(Clone, Debug, PartialEq)]
pub struct ResourceScheduleTimeline {
    columns: Vec<ResourceScheduleColumn>,
    placements: Vec<ResourceSchedulePlacement>,
    slots: usize,
}
impl ResourceScheduleTimeline {
    pub fn columns(&self) -> &[ResourceScheduleColumn] {
        &self.columns
    }
    pub fn placements(&self) -> &[ResourceSchedulePlacement] {
        &self.placements
    }
    pub const fn slot_count(&self) -> usize {
        self.slots
    }
}
impl crate::ResourceScheduleData {
    /// Resource-major date columns. Only one-day/seven-day civil windows are
    /// admitted, with at most 2048 time slots, 16 overlap lanes and 4096 segments.
    pub fn layout_timeline(
        &self,
        time: ScheduleTimeWindow,
        days: usize,
    ) -> Result<ResourceScheduleTimeline, ScheduleError> {
        self.validate_timeline_window(days)?;
        let occurrences = self.data().expand(MAX_SCHEDULE_OCCURRENCES)?;
        self.layout_timeline_expanded(time, days, &occurrences)
    }

    fn validate_timeline_window(&self, days: usize) -> Result<(), ScheduleError> {
        let data = self.data();
        let window = data.window();
        if !matches!(days, 1 | 7)
            || window.start().time() != midnight()
            || window.end().time() != midnight()
            || window.start().date().days_until(window.end().date()) != days as i32
        {
            return Err(ScheduleError::InvalidInterval);
        }
        Ok(())
    }

    /// Reuse the occurrence order of the current admitted model. Callers must
    /// pass the complete expansion of this data, never a filtered projection.
    pub(crate) fn layout_timeline_expanded(
        &self,
        time: ScheduleTimeWindow,
        days: usize,
        occurrences: &[ScheduleOccurrence],
    ) -> Result<ResourceScheduleTimeline, ScheduleError> {
        self.validate_timeline_window(days)?;
        let data = self.data();
        let window = data.window();
        let visible = self.visible_resources();
        let count = visible.len() * days;
        let slots = count * time.len();
        if slots > 2048 {
            return Err(ScheduleError::InvalidResource);
        }
        let mut columns = Vec::with_capacity(count);
        let mut placements = vec![];
        for (resource, key) in visible.iter().enumerate() {
            for day in 0..days {
                let date = window
                    .start()
                    .date()
                    .checked_add_days(day as i32)
                    .map_err(|_| ScheduleError::InvalidInterval)?;
                let column = columns.len();
                columns.push(ResourceScheduleColumn {
                    resource: self.resource_window().first() + resource,
                    day: day as u8,
                    date,
                });
                for position in
                    data.layout_day_expanded(date, time, Some(key.key()), occurrences)?
                {
                    if position.lanes() > 16 || placements.len() == 4096 {
                        return Err(ScheduleError::InvalidResource);
                    }
                    placements.push(ResourceSchedulePlacement { column, position });
                }
            }
        }
        Ok(ResourceScheduleTimeline {
            columns,
            placements,
            slots,
        })
    }
}
fn at_minute(date: CivilDate, minute: u16) -> Result<LocalDateTime, ScheduleError> {
    if minute == 1440 {
        return Ok(LocalDateTime::new(
            date.checked_add_days(1)
                .map_err(|_| ScheduleError::InvalidInterval)?,
            midnight(),
        ));
    }
    Ok(LocalDateTime::new(
        date,
        LocalTime::new((minute / 60) as u8, (minute % 60) as u8, 0)
            .map_err(|_| ScheduleError::InvalidInterval)?,
    ))
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScheduleDayPlacement {
    occurrence: usize,
    lane: u16,
    lanes: u16,
    start: f64,
    end: f64,
}
impl ScheduleDayPlacement {
    pub const fn occurrence_index(self) -> usize {
        self.occurrence
    }
    pub const fn lane(self) -> usize {
        self.lane as usize
    }
    pub const fn lanes(self) -> usize {
        self.lanes as usize
    }
    pub const fn start_fraction(self) -> f64 {
        self.start
    }
    pub const fn end_fraction(self) -> f64 {
        self.end
    }
}
impl ScheduleData {
    /// `None` combines resources in one column; a key filters exactly that
    /// resource. All-day rows are separate from timed overlap lanes.
    pub fn layout_day(
        &self,
        date: CivilDate,
        time: ScheduleTimeWindow,
        resource: Option<&ScheduleKey>,
    ) -> Result<Vec<ScheduleDayPlacement>, ScheduleError> {
        let occurrences = self.expand(MAX_SCHEDULE_OCCURRENCES)?;
        self.layout_day_expanded(date, time, resource, &occurrences)
    }

    /// Internal geometry over the complete expansion of this exact model.
    pub(crate) fn layout_day_expanded(
        &self,
        date: CivilDate,
        time: ScheduleTimeWindow,
        resource: Option<&ScheduleKey>,
        occurrences: &[ScheduleOccurrence],
    ) -> Result<Vec<ScheduleDayPlacement>, ScheduleError> {
        let start = ticks(at_minute(date, time.start)?);
        let end = ticks(at_minute(date, time.end)?);
        let extent = (end - start) as f64;
        let mut rows = Vec::new();
        for (index, occurrence) in occurrences.iter().enumerate() {
            let event = self
                .event(*occurrence)
                .ok_or(ScheduleError::StaleOccurrence)?;
            if event.is_all_day() || resource.is_some_and(|key| event.resource_key() != Some(key)) {
                continue;
            }
            let a = ticks(occurrence.interval().start()).max(start);
            let b = ticks(occurrence.interval().end()).min(end);
            if a < b {
                rows.push((index, a, b));
            }
        }
        let mut result = Vec::with_capacity(rows.len());
        let mut group = 0;
        let mut group_end = start;
        let mut lane_ends = Vec::new();
        for (index, a, b) in rows {
            if a >= group_end {
                let lanes = lane_ends.len() as u16;
                for item in &mut result[group..] {
                    let item: &mut ScheduleDayPlacement = item;
                    item.lanes = lanes;
                }
                group = result.len();
                lane_ends.clear();
            }
            let lane = if let Some(lane) = lane_ends.iter().position(|end| *end <= a) {
                lane_ends[lane] = b;
                lane
            } else {
                lane_ends.push(b);
                lane_ends.len() - 1
            };
            group_end = group_end.max(b);
            result.push(ScheduleDayPlacement {
                occurrence: index,
                lane: lane as u16,
                lanes: 0,
                start: (a - start) as f64 / extent,
                end: (b - start) as f64 / extent,
            });
        }
        let lanes = lane_ends.len() as u16;
        for item in &mut result[group..] {
            item.lanes = lanes;
        }
        Ok(result)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScheduleEvent, ScheduleInterval, ScheduleWindow, ScheduleZone};
    fn d(s: &str) -> CivilDate {
        CivilDate::parse_iso(s).unwrap()
    }
    fn t(s: &str) -> LocalDateTime {
        crate::parse_local_date_time(s).unwrap()
    }
    fn event(key: &str, start: &str, end: &str) -> ScheduleEvent {
        ScheduleEvent::new(
            ScheduleKey::new(key).unwrap(),
            key,
            ScheduleInterval::new(t(start), t(end)).unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn clipped_overlap_clusters_reuse_lanes_on_half_open_boundaries() {
        let data = ScheduleData::new(
            ScheduleZone::Civil,
            ScheduleWindow::days(d("2024-02-29"), 1).unwrap(),
            [
                event("a", "2024-02-29T07:00", "2024-02-29T12:00"),
                event("b", "2024-02-29T10:00", "2024-02-29T11:00"),
                event("c", "2024-02-29T11:00", "2024-02-29T13:00"),
                event("d", "2024-02-29T13:00", "2024-02-29T14:00"),
            ],
        )
        .unwrap();
        let rows = data
            .layout_day(d("2024-02-29"), ScheduleTimeWindow::default(), None)
            .unwrap();
        assert_eq!(rows.len(), 4);
        assert_eq!((rows[0].lane(), rows[0].lanes()), (0, 2));
        assert_eq!((rows[1].lane(), rows[1].lanes()), (1, 2));
        assert_eq!((rows[2].lane(), rows[2].lanes()), (1, 2));
        assert_eq!((rows[3].lane(), rows[3].lanes()), (0, 1));
        assert_eq!(rows[0].start_fraction(), 0.);
        assert_eq!(rows[3].start_fraction(), 0.5);
        assert_eq!(rows[3].end_fraction(), 0.6);
    }
    #[test]
    fn slot_budget_midnight_and_resource_filter_are_real_coordinates() {
        assert!(ScheduleTimeWindow::new(0, 1440, 1).is_err());
        assert!(ScheduleTimeWindow::new(0, 1440, 15).is_ok());
        assert!(ScheduleTimeWindow::new(0, 61, 30).is_err());
        let window = ScheduleTimeWindow::new(1380, 1440, 30).unwrap();
        let span = window.at(d("2024-02-29"), 1).unwrap();
        assert_eq!(span.end(), t("2024-03-01T00:00"));
        assert!(window.at(d("9999-12-31"), 1).is_err());
        assert!(window.at(d("2024-02-29"), 2).is_err());
        let resource = ScheduleKey::new("room").unwrap();
        let data = ScheduleData::new(
            ScheduleZone::Civil,
            ScheduleWindow::days(d("2024-02-29"), 1).unwrap(),
            [
                event("a", "2024-02-29T09:00", "2024-02-29T10:00").resource(resource.clone()),
                event("b", "2024-02-29T09:00", "2024-02-29T10:00"),
            ],
        )
        .unwrap();
        assert_eq!(
            data.layout_day(d("2024-02-29"), ScheduleTimeWindow::default(), None)
                .unwrap()
                .len(),
            2
        );
        let rows = data
            .layout_day(
                d("2024-02-29"),
                ScheduleTimeWindow::default(),
                Some(&resource),
            )
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].lanes(), 1);
    }
}
