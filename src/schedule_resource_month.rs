use super::*;
impl crate::ResourceScheduleData {
    /// Month bars project every intersected civil day to a complete cell. Exact
    /// intervals stay in occurrence records and edit/selection callbacks.
    pub fn layout_month(&self) -> Result<ResourceScheduleTimeline, ScheduleError> {
        let data = self.data();
        let window = data.window();
        if crate::ScheduleWindow::month(window.start().date().year_month())? != window {
            return Err(ScheduleError::InvalidInterval);
        }
        let occurrences = data.expand(MAX_SCHEDULE_OCCURRENCES)?;
        let first = ticks(window.start());
        let last = ticks(window.end());
        let extent = (last - first) as f64;
        let mut columns = vec![];
        let mut placements = vec![];
        for (resource, registration) in self.visible_resources().iter().enumerate() {
            let mut rows = vec![];
            for (index, occ) in occurrences.iter().enumerate() {
                if data.event(*occ).unwrap().resource_key() != Some(registration.key()) {
                    continue;
                }
                let a = occ.interval().start().max(window.start());
                let b = occ.interval().end().min(window.end());
                if a >= b {
                    continue;
                }
                let start = ticks(LocalDateTime::new(a.date(), midnight()));
                let end = if b.time() == midnight() {
                    ticks(b)
                } else {
                    ticks(LocalDateTime::new(
                        b.date()
                            .checked_add_days(1)
                            .map_err(|_| ScheduleError::InvalidInterval)?,
                        midnight(),
                    ))
                };
                rows.push((index, start, end));
            }
            // Day projection may change ordering among starts on the same day.
            rows.sort_by_key(|&(index, start, end)| (start, end, index));
            let column = columns.len();
            columns.push(ResourceScheduleColumn {
                resource: self.resource_window().first() + resource,
                day: 0,
                date: window.start().date(),
            });
            let mut local: Vec<ScheduleDayPlacement> = vec![];
            let mut ends: Vec<i128> = vec![];
            let mut group = 0;
            let mut group_end = first;
            for (occurrence, start, end) in rows {
                if start >= group_end {
                    for row in &mut local[group..] {
                        row.lanes = ends.len() as u16
                    }
                    group = local.len();
                    ends.clear();
                }
                let lane = if let Some(lane) = ends.iter().position(|&old| old <= start) {
                    ends[lane] = end;
                    lane
                } else {
                    if ends.len() == 16 {
                        return Err(ScheduleError::InvalidResource);
                    }
                    ends.push(end);
                    ends.len() - 1
                };
                group_end = group_end.max(end);
                local.push(ScheduleDayPlacement {
                    occurrence,
                    lane: lane as u16,
                    lanes: 0,
                    start: (start - first) as f64 / extent,
                    end: (end - first) as f64 / extent,
                });
            }
            for row in &mut local[group..] {
                row.lanes = ends.len() as u16
            }
            for position in local {
                placements.push(ResourceSchedulePlacement { column, position })
            }
        }
        Ok(ResourceScheduleTimeline {
            slots: columns.len()
                * window.start().date().days_until(window.end().date()) as usize,
            columns,
            placements,
        })
    }
}
