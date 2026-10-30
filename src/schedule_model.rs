//! Explicit half-open civil windows and bounded recurrence expansion.
use crate::{
    CivilDate, DateTimePolicy, DateTimeResolution, DateTimeResolutionError, FixedOffset,
    LocalDateTime, LocalTime, ResolvedDateTime, ScheduleRecurrence,
};
use std::{collections::HashSet, fmt};
pub const MAX_SCHEDULE_WINDOW_DAYS: u32 = 366;
pub const MAX_SCHEDULE_EVENTS: usize = 512;
pub const MAX_SCHEDULE_OCCURRENCES: usize = 2048;
pub const MAX_SCHEDULE_EXCEPTIONS: usize = 128;
pub const MAX_SCHEDULE_RESOURCES: usize = 64;
pub const MAX_SCHEDULE_CONFLICTS: usize = 2048;
const SECOND: i128 = 1_000_000_000;
const DAY: i128 = 86400 * SECOND;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleError {
    InvalidKey,
    InvalidText,
    InvalidInterval,
    WindowTooLarge,
    TooManyEvents,
    TooManyOccurrences,
    TooManyExceptions,
    DuplicateKey,
    InvalidRecurrence,
    InvalidResource,
    TooManyResources,
    TooManyConflicts,
    EditNotAllowed,
    StaleOccurrence,
    Resolution(DateTimeResolutionError),
    InstantOrder,
}
impl fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid finite schedule: {self:?}")
    }
}
impl std::error::Error for ScheduleError {}
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScheduleKey(String);
impl ScheduleKey {
    pub fn new(v: impl Into<String>) -> Result<Self, ScheduleError> {
        let v = v.into();
        if v.is_empty()
            || v.len() > 64
            || !v
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        {
            Err(ScheduleError::InvalidKey)
        } else {
            Ok(Self(v))
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
/// An application key names supplied rules; it does not install an IANA database.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScheduleZone {
    Civil,
    FixedOffset(FixedOffset),
    Application(ScheduleKey),
}
#[derive(Clone)]
pub struct ScheduleContext {
    zone: ScheduleZone,
    policy: DateTimePolicy,
}
impl ScheduleContext {
    pub fn civil() -> Self {
        Self {
            zone: ScheduleZone::Civil,
            policy: DateTimePolicy::Civil,
        }
    }
    pub fn fixed(offset: FixedOffset) -> Self {
        Self {
            zone: ScheduleZone::FixedOffset(offset),
            policy: DateTimePolicy::FixedOffset(offset),
        }
    }
    pub fn application(
        key: ScheduleKey,
        ambiguity: crate::AmbiguityPolicy,
        resolver: impl Fn(LocalDateTime) -> crate::LocalResolution + 'static,
    ) -> Self {
        Self {
            zone: ScheduleZone::Application(key),
            policy: DateTimePolicy::application(ambiguity, resolver),
        }
    }
    pub fn zone(&self) -> &ScheduleZone {
        &self.zone
    }
    /// Application code runs outside any caller-owned reactive borrow.
    pub fn resolve(&self, interval: ScheduleInterval) -> Result<ScheduleResolution, ScheduleError> {
        let start = self
            .policy
            .resolve(interval.start)
            .map_err(ScheduleError::Resolution)?;
        let end = self
            .policy
            .resolve(interval.end)
            .map_err(ScheduleError::Resolution)?;
        match (start, end) {
            (DateTimeResolution::Civil, DateTimeResolution::Civil) => Ok(ScheduleResolution::Civil),
            (DateTimeResolution::Instant(start), DateTimeResolution::Instant(end))
                if start.instant() < end.instant() =>
            {
                Ok(ScheduleResolution::Instants { start, end })
            }
            _ => Err(ScheduleError::InstantOrder),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleResolution {
    Civil,
    Instants {
        start: ResolvedDateTime,
        end: ResolvedDateTime,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduleInterval {
    start: LocalDateTime,
    end: LocalDateTime,
}
impl ScheduleInterval {
    pub fn new(start: LocalDateTime, end: LocalDateTime) -> Result<Self, ScheduleError> {
        if start >= end || ticks(end) - ticks(start) > i128::from(MAX_SCHEDULE_WINDOW_DAYS) * DAY {
            Err(ScheduleError::InvalidInterval)
        } else {
            Ok(Self { start, end })
        }
    }
    pub const fn start(self) -> LocalDateTime {
        self.start
    }
    pub const fn end(self) -> LocalDateTime {
        self.end
    }
    pub fn contains(self, value: LocalDateTime) -> bool {
        self.start <= value && value < self.end
    }
    pub fn overlaps(self, other: Self) -> bool {
        self.start < other.end && other.start < self.end
    }
    pub fn duration_nanoseconds(self) -> i128 {
        ticks(self.end) - ticks(self.start)
    }
    /// Exact civil translation. It preserves seconds/nanoseconds and duration.
    pub fn shifted_minutes(self, minutes: i64) -> Result<Self, ScheduleError> {
        let delta = i128::from(minutes) * 60 * SECOND;
        Self::new(
            from_ticks(ticks(self.start) + delta).ok_or(ScheduleError::InvalidInterval)?,
            from_ticks(ticks(self.end) + delta).ok_or(ScheduleError::InvalidInterval)?,
        )
    }
    pub fn resize_start_by_minutes(self, minutes: i64) -> Result<Self, ScheduleError> {
        Self::new(
            from_ticks(ticks(self.start) + i128::from(minutes) * 60 * SECOND)
                .ok_or(ScheduleError::InvalidInterval)?,
            self.end,
        )
    }
    pub fn resize_end_by_minutes(self, minutes: i64) -> Result<Self, ScheduleError> {
        Self::new(
            self.start,
            from_ticks(ticks(self.end) + i128::from(minutes) * 60 * SECOND)
                .ok_or(ScheduleError::InvalidInterval)?,
        )
    }
    pub fn all_day(start: CivilDate, end_exclusive: CivilDate) -> Result<Self, ScheduleError> {
        Self::new(
            LocalDateTime::new(start, midnight()),
            LocalDateTime::new(end_exclusive, midnight()),
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduleWindow(ScheduleInterval);
impl ScheduleWindow {
    pub fn new(start: LocalDateTime, end: LocalDateTime) -> Result<Self, ScheduleError> {
        if start >= end {
            return Err(ScheduleError::InvalidInterval);
        }
        if ticks(end) - ticks(start) > i128::from(MAX_SCHEDULE_WINDOW_DAYS) * DAY {
            return Err(ScheduleError::WindowTooLarge);
        }
        Ok(Self(ScheduleInterval::new(start, end)?))
    }
    pub fn days(start: CivilDate, count: u16) -> Result<Self, ScheduleError> {
        if count == 0 || u32::from(count) > MAX_SCHEDULE_WINDOW_DAYS {
            return Err(ScheduleError::WindowTooLarge);
        }
        let end = start
            .checked_add_days(i32::from(count))
            .map_err(|_| ScheduleError::InvalidInterval)?;
        Self::new(
            LocalDateTime::new(start, midnight()),
            LocalDateTime::new(end, midnight()),
        )
    }
    /// One exact calendar month. An unrepresentable exclusive end is rejected.
    pub fn month(month: crate::YearMonth) -> Result<Self, ScheduleError> {
        let end = month
            .checked_add(1)
            .map_err(|_| ScheduleError::InvalidInterval)?
            .first();
        Self::new(
            LocalDateTime::new(month.first(), midnight()),
            LocalDateTime::new(end, midnight()),
        )
    }
    /// One exact calendar year. Year 9999 has no representable exclusive end.
    pub fn year(year: crate::CivilYear) -> Result<Self, ScheduleError> {
        let start =
            CivilDate::new(year.value(), 1, 1).map_err(|_| ScheduleError::InvalidInterval)?;
        let end = CivilDate::new(
            year.checked_add(1)
                .map_err(|_| ScheduleError::InvalidInterval)?
                .value(),
            1,
            1,
        )
        .map_err(|_| ScheduleError::InvalidInterval)?;
        Self::new(
            LocalDateTime::new(start, midnight()),
            LocalDateTime::new(end, midnight()),
        )
    }
    pub const fn interval(self) -> ScheduleInterval {
        self.0
    }
    pub const fn start(self) -> LocalDateTime {
        self.0.start
    }
    pub const fn end(self) -> LocalDateTime {
        self.0.end
    }
    pub fn shifted(self, pages: i32) -> Result<Self, ScheduleError> {
        let delta = self.0.duration_nanoseconds() * i128::from(pages);
        let start =
            from_ticks(ticks(self.start()) + delta).ok_or(ScheduleError::InvalidInterval)?;
        let end = from_ticks(ticks(self.end()) + delta).ok_or(ScheduleError::InvalidInterval)?;
        Self::new(start, end)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleEvent {
    key: ScheduleKey,
    title: String,
    interval: ScheduleInterval,
    all_day: bool,
    resource: Option<ScheduleKey>,
    recurrence: Option<ScheduleRecurrence>,
    exceptions: Vec<CivilDate>,
    permissions: SchedulePermissions,
}
impl ScheduleEvent {
    pub fn new(
        key: ScheduleKey,
        title: impl Into<String>,
        interval: ScheduleInterval,
    ) -> Result<Self, ScheduleError> {
        let title = title.into();
        if title.len() > 256 || title.contains(['\0', '\r', '\n']) {
            return Err(ScheduleError::InvalidText);
        }
        Ok(Self {
            key,
            title,
            interval,
            all_day: false,
            resource: None,
            recurrence: None,
            exceptions: vec![],
            permissions: SchedulePermissions::default(),
        })
    }
    pub fn all_day(mut self) -> Result<Self, ScheduleError> {
        if self.interval.start.time() != midnight() || self.interval.end.time() != midnight() {
            return Err(ScheduleError::InvalidInterval);
        }
        self.all_day = true;
        Ok(self)
    }
    pub fn resource(mut self, key: ScheduleKey) -> Self {
        self.resource = Some(key);
        self
    }
    pub fn permissions(mut self, value: SchedulePermissions) -> Self {
        self.permissions = value;
        self
    }
    pub const fn edit_permissions(&self) -> SchedulePermissions {
        self.permissions
    }
    pub fn recurring(mut self, rule: ScheduleRecurrence) -> Result<Self, ScheduleError> {
        rule.validate_anchor(self.interval.start.date())?;
        self.recurrence = Some(rule);
        Ok(self)
    }
    pub fn except_dates(
        mut self,
        dates: impl IntoIterator<Item = CivilDate>,
    ) -> Result<Self, ScheduleError> {
        let mut dates: Vec<_> = dates
            .into_iter()
            .take(MAX_SCHEDULE_EXCEPTIONS + 1)
            .collect();
        if dates.len() > MAX_SCHEDULE_EXCEPTIONS {
            return Err(ScheduleError::TooManyExceptions);
        }
        dates.sort_unstable();
        dates.dedup();
        self.exceptions = dates;
        Ok(self)
    }
    pub fn key(&self) -> &ScheduleKey {
        &self.key
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub const fn interval(&self) -> ScheduleInterval {
        self.interval
    }
    pub const fn is_all_day(&self) -> bool {
        self.all_day
    }
    pub fn resource_key(&self) -> Option<&ScheduleKey> {
        self.resource.as_ref()
    }
    pub const fn recurrence(&self) -> Option<ScheduleRecurrence> {
        self.recurrence
    }
    pub fn exceptions(&self) -> &[CivilDate] {
        &self.exceptions
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduleOccurrence {
    event: usize,
    interval: ScheduleInterval,
    candidate: u32,
}
impl ScheduleOccurrence {
    pub const fn event_index(self) -> usize {
        self.event
    }
    pub const fn interval(self) -> ScheduleInterval {
        self.interval
    }
    pub const fn candidate_index(self) -> u32 {
        self.candidate
    }
}
/// One current authoritative window, never a cache of visited requests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleData {
    zone: ScheduleZone,
    window: ScheduleWindow,
    events: Vec<ScheduleEvent>,
}
impl ScheduleData {
    pub fn new(
        zone: ScheduleZone,
        window: ScheduleWindow,
        events: impl IntoIterator<Item = ScheduleEvent>,
    ) -> Result<Self, ScheduleError> {
        let events: Vec<_> = events.into_iter().take(MAX_SCHEDULE_EVENTS + 1).collect();
        if events.len() > MAX_SCHEDULE_EVENTS {
            return Err(ScheduleError::TooManyEvents);
        }
        let mut keys = HashSet::with_capacity(events.len());
        for event in &events {
            if !keys.insert(event.key.as_str()) {
                return Err(ScheduleError::DuplicateKey);
            }
        }
        Ok(Self {
            zone,
            window,
            events,
        })
    }
    pub fn zone(&self) -> &ScheduleZone {
        &self.zone
    }
    pub const fn window(&self) -> ScheduleWindow {
        self.window
    }
    pub fn events(&self) -> &[ScheduleEvent] {
        &self.events
    }
    pub fn event(&self, occurrence: ScheduleOccurrence) -> Option<&ScheduleEvent> {
        self.events.get(occurrence.event)
    }
    pub fn validate_resources(&self, resources: &ScheduleResources) -> Result<(), ScheduleError> {
        for event in &self.events {
            if event
                .resource_key()
                .is_some_and(|key| resources.get(key).is_none())
            {
                return Err(ScheduleError::InvalidResource);
            }
        }
        Ok(())
    }
    /// Conflicts use half-open intervals. The caller chooses whether separate
    /// resources may conflict, and supplies a complete-result budget.
    pub fn conflicts(
        &self,
        policy: ScheduleConflictPolicy,
        limit: usize,
    ) -> Result<Vec<ScheduleConflict>, ScheduleError> {
        if limit > MAX_SCHEDULE_CONFLICTS {
            return Err(ScheduleError::TooManyConflicts);
        }
        let occurrences = self.expand(MAX_SCHEDULE_OCCURRENCES)?;
        let mut result = Vec::new();
        for (first, a) in occurrences.iter().enumerate() {
            for (second, b) in occurrences.iter().enumerate().skip(first + 1) {
                if b.interval.start >= a.interval.end {
                    break;
                }
                if policy == ScheduleConflictPolicy::AllResources
                    || self.events[a.event].resource == self.events[b.event].resource
                {
                    if result.len() == limit {
                        return Err(ScheduleError::TooManyConflicts);
                    }
                    result.push(ScheduleConflict { first, second });
                }
            }
        }
        Ok(result)
    }
    /// This only validates an edit proposal against the current window and
    /// permissions. It never mutates events or chooses a recurrence edit policy.
    pub fn validate_edit(
        &self,
        resources: &ScheduleResources,
        edit: &ScheduleEdit,
    ) -> Result<(), ScheduleError> {
        self.validate_resources(resources)?;
        let event = self
            .events
            .iter()
            .find(|event| event.key == edit.event)
            .ok_or(ScheduleError::StaleOccurrence)?;
        let permitted = match edit.kind {
            ScheduleEditKind::Move => event.permissions.move_event,
            ScheduleEditKind::ResizeStart | ScheduleEditKind::ResizeEnd => {
                event.permissions.resize_event
            }
        };
        if !permitted {
            return Err(ScheduleError::EditNotAllowed);
        }
        if event.all_day
            && (edit.proposed.start.time() != midnight() || edit.proposed.end.time() != midnight())
        {
            return Err(ScheduleError::InvalidInterval);
        }
        if event.resource != edit.original_resource
            || edit
                .proposed_resource
                .as_ref()
                .is_some_and(|key| resources.get(key).is_none())
        {
            return Err(ScheduleError::InvalidResource);
        }
        let present = self
            .expand(MAX_SCHEDULE_OCCURRENCES)?
            .iter()
            .any(|occurrence| {
                self.events[occurrence.event].key == edit.event
                    && occurrence.candidate == edit.candidate
                    && occurrence.interval == edit.original
            });
        if !present {
            return Err(ScheduleError::StaleOccurrence);
        }
        let valid = match edit.kind {
            ScheduleEditKind::Move => {
                edit.original.duration_nanoseconds() == edit.proposed.duration_nanoseconds()
            }
            ScheduleEditKind::ResizeStart => {
                edit.original.end == edit.proposed.end
                    && edit.original_resource == edit.proposed_resource
            }
            ScheduleEditKind::ResizeEnd => {
                edit.original.start == edit.proposed.start
                    && edit.original_resource == edit.proposed_resource
            }
        };
        if !valid {
            return Err(ScheduleError::InvalidInterval);
        }
        Ok(())
    }
    /// Candidate starts are examined only in the requested window plus at most
    /// one bounded event-duration lookback. No walk begins at an ancient anchor.
    pub fn expand(&self, limit: usize) -> Result<Vec<ScheduleOccurrence>, ScheduleError> {
        if limit > MAX_SCHEDULE_OCCURRENCES {
            return Err(ScheduleError::TooManyOccurrences);
        }
        let mut output = Vec::new();
        for (event_index, event) in self.events.iter().enumerate() {
            let mut emit = |interval, candidate| -> Result<(), ScheduleError> {
                if self.window.0.overlaps(interval) {
                    if output.len() == limit {
                        return Err(ScheduleError::TooManyOccurrences);
                    }
                    output.push(ScheduleOccurrence {
                        event: event_index,
                        interval,
                        candidate,
                    });
                }
                Ok(())
            };
            if let Some(rule) = event.recurrence {
                let duration = event.interval.duration_nanoseconds();
                let first = (ticks(self.window.start()) - duration).max(0) / DAY;
                let last = (ticks(self.window.end()) - 1) / DAY;
                for ordinal in first..=last {
                    let date = CivilDate::from_ordinal(ordinal as u32)
                        .map_err(|_| ScheduleError::InvalidInterval)?;
                    let Some(index) = rule.index_on(event.interval.start.date(), date) else {
                        continue;
                    };
                    if event.exceptions.binary_search(&date).is_ok() {
                        continue;
                    }
                    let start = LocalDateTime::new(date, event.interval.start.time());
                    let Some(end) = from_ticks(ticks(start) + duration) else {
                        continue;
                    };
                    emit(ScheduleInterval { start, end }, index)?;
                }
            } else if event
                .exceptions
                .binary_search(&event.interval.start.date())
                .is_err()
            {
                emit(event.interval, 0)?;
            }
        }
        output.sort_unstable_by_key(|v| (v.interval.start, v.interval.end, v.event));
        Ok(output)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SchedulePermissions {
    pub select: bool,
    pub move_event: bool,
    pub resize_event: bool,
}
impl Default for SchedulePermissions {
    fn default() -> Self {
        Self {
            select: true,
            move_event: false,
            resize_event: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleResource {
    key: ScheduleKey,
    title: String,
}
impl ScheduleResource {
    pub fn new(key: ScheduleKey, title: impl Into<String>) -> Result<Self, ScheduleError> {
        let title = title.into();
        if title.len() > 256 || title.contains(['\0', '\r', '\n']) {
            return Err(ScheduleError::InvalidText);
        }
        Ok(Self { key, title })
    }
    pub fn key(&self) -> &ScheduleKey {
        &self.key
    }
    pub fn title(&self) -> &str {
        &self.title
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScheduleResources(Vec<ScheduleResource>);
impl ScheduleResources {
    pub fn new(values: impl IntoIterator<Item = ScheduleResource>) -> Result<Self, ScheduleError> {
        let values: Vec<_> = values
            .into_iter()
            .take(MAX_SCHEDULE_RESOURCES + 1)
            .collect();
        if values.len() > MAX_SCHEDULE_RESOURCES {
            return Err(ScheduleError::TooManyResources);
        }
        let mut keys = HashSet::with_capacity(values.len());
        for value in &values {
            if !keys.insert(value.key()) {
                return Err(ScheduleError::DuplicateKey);
            }
        }
        Ok(Self(values))
    }
    pub fn items(&self) -> &[ScheduleResource] {
        &self.0
    }
    pub fn get(&self, key: &ScheduleKey) -> Option<&ScheduleResource> {
        self.0.iter().find(|v| v.key() == key)
    }
    pub fn window(&self, first: usize, count: usize) -> Result<&[ScheduleResource], ScheduleError> {
        if count == 0 || count > 8 {
            return Err(ScheduleError::InvalidResource);
        }
        self.0
            .get(
                first
                    ..first
                        .checked_add(count)
                        .ok_or(ScheduleError::InvalidResource)?,
            )
            .ok_or(ScheduleError::InvalidResource)
    }
}
/// Explicit live resource window. It never records visited resources.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduleResourceWindow {
    first: u8,
    count: u8,
}
impl ScheduleResourceWindow {
    pub fn new(first: usize, count: usize) -> Result<Self, ScheduleError> {
        if count == 0
            || count > 8
            || first
                .checked_add(count)
                .is_none_or(|end| end > MAX_SCHEDULE_RESOURCES)
        {
            return Err(ScheduleError::InvalidResource);
        }
        Ok(Self {
            first: first as u8,
            count: count as u8,
        })
    }
    pub const fn first(self) -> usize {
        self.first as usize
    }
    pub const fn count(self) -> usize {
        self.count as usize
    }
    pub fn resolve(
        self,
        resources: &ScheduleResources,
    ) -> Result<&[ScheduleResource], ScheduleError> {
        if resources.items().is_empty() && self.first == 0 {
            return Ok(&[]);
        }
        resources.window(self.first(), self.count())
    }
}
impl Default for ScheduleResourceWindow {
    fn default() -> Self {
        Self { first: 0, count: 1 }
    }
}
/// Coherent resource-view input: the request data, registrations and live window
/// validate together before reactive publication. Unassigned events are rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceScheduleData {
    data: ScheduleData,
    resources: ScheduleResources,
    window: ScheduleResourceWindow,
}
impl ResourceScheduleData {
    pub fn new(
        data: ScheduleData,
        resources: ScheduleResources,
        window: ScheduleResourceWindow,
    ) -> Result<Self, ScheduleError> {
        data.validate_resources(&resources)?;
        if data
            .events()
            .iter()
            .any(|event| event.resource_key().is_none())
        {
            return Err(ScheduleError::InvalidResource);
        }
        window.resolve(&resources)?;
        Ok(Self {
            data,
            resources,
            window,
        })
    }
    pub fn data(&self) -> &ScheduleData {
        &self.data
    }
    pub fn resources(&self) -> &ScheduleResources {
        &self.resources
    }
    pub const fn resource_window(&self) -> ScheduleResourceWindow {
        self.window
    }
    pub fn visible_resources(&self) -> &[ScheduleResource] {
        self.window.resolve(&self.resources).unwrap()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleConflictPolicy {
    SameResource,
    AllResources,
}
/// Indices refer to this expansion's deterministic start/end/event ordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduleConflict {
    pub first: usize,
    pub second: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleEditKind {
    Move,
    ResizeStart,
    ResizeEnd,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleEdit {
    pub event: ScheduleKey,
    pub candidate: u32,
    pub kind: ScheduleEditKind,
    pub original: ScheduleInterval,
    pub proposed: ScheduleInterval,
    pub original_resource: Option<ScheduleKey>,
    pub proposed_resource: Option<ScheduleKey>,
}
pub(crate) fn midnight() -> LocalTime {
    LocalTime::new(0, 0, 0).unwrap()
}
pub(crate) fn ticks(v: LocalDateTime) -> i128 {
    i128::from(v.date().ordinal()) * DAY
        + i128::from(v.time().seconds_from_midnight()) * SECOND
        + i128::from(v.time().nanosecond())
}
fn from_ticks(v: i128) -> Option<LocalDateTime> {
    if v < 0 {
        return None;
    }
    let date = CivilDate::from_ordinal(u32::try_from(v / DAY).ok()?).ok()?;
    let rem = v % DAY;
    let seconds = (rem / SECOND) as u32;
    Some(LocalDateTime::new(
        date,
        LocalTime::with_nanosecond(
            (seconds / 3600) as u8,
            (seconds / 60 % 60) as u8,
            (seconds % 60) as u8,
            (rem % SECOND) as u32,
        )
        .ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AmbiguityPolicy, LocalResolution, MissingScheduleDay, RecurrenceEnd, RecurrenceFrequency,
        Weekday,
    };
    use std::{cell::Cell, rc::Rc};
    fn d(s: &str) -> CivilDate {
        CivilDate::parse_iso(s).unwrap()
    }
    fn dt(s: &str) -> LocalDateTime {
        crate::parse_local_date_time(s).unwrap()
    }
    fn interval(a: &str, b: &str) -> ScheduleInterval {
        ScheduleInterval::new(dt(a), dt(b)).unwrap()
    }
    fn event() -> ScheduleEvent {
        ScheduleEvent::new(
            ScheduleKey::new("event").unwrap(),
            "Planning",
            interval("2024-02-01T09:00", "2024-02-01T10:00"),
        )
        .unwrap()
    }
    #[test]
    fn intervals_are_half_open_and_windows_enforce_explicit_finite_domain() {
        let a = interval("2024-02-29T09:00", "2024-02-29T10:00");
        let b = interval("2024-02-29T10:00", "2024-02-29T11:00");
        assert!(!a.overlaps(b));
        assert!(a.contains(a.start()));
        assert!(!a.contains(a.end()));
        assert!(ScheduleInterval::new(a.end(), a.start()).is_err());
        assert!(ScheduleWindow::days(d("2024-01-01"), 366).is_ok());
        assert_eq!(
            ScheduleWindow::days(d("2024-01-01"), 367),
            Err(ScheduleError::WindowTooLarge)
        );
        assert!(ScheduleWindow::days(d("9999-12-31"), 1).is_err());
        assert_eq!(
            from_ticks(ticks(dt("9999-12-31T23:59:59.999999999"))),
            Some(dt("9999-12-31T23:59:59.999999999"))
        );
        assert_eq!(from_ticks(-1), None);
    }
    #[test]
    fn ancient_daily_anchor_jumps_to_window_and_exceptions_do_not_extend_count() {
        let ancient = ScheduleEvent::new(
            ScheduleKey::new("ancient").unwrap(),
            "Ancient daily",
            interval("0001-01-01T23:00", "0001-01-02T01:00"),
        )
        .unwrap()
        .recurring(
            ScheduleRecurrence::new(RecurrenceFrequency::Daily, 1, RecurrenceEnd::Never).unwrap(),
        )
        .unwrap();
        let data = ScheduleData::new(
            ScheduleZone::Civil,
            ScheduleWindow::days(d("9999-12-29"), 2).unwrap(),
            [ancient],
        )
        .unwrap();
        let rows = data.expand(16).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].interval.start().date(), d("9999-12-28"));
        assert!(rows[0].candidate_index() > 3_000_000);
        let e = event()
            .recurring(
                ScheduleRecurrence::new(RecurrenceFrequency::Daily, 1, RecurrenceEnd::Count(3))
                    .unwrap(),
            )
            .unwrap()
            .except_dates([d("2024-02-02")])
            .unwrap();
        let rows = ScheduleData::new(
            ScheduleZone::Civil,
            ScheduleWindow::days(d("2024-02-01"), 10).unwrap(),
            [e],
        )
        .unwrap()
        .expand(16)
        .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].candidate, 2);
        assert_eq!(rows[1].interval.start().date(), d("2024-02-03"));
    }
    #[test]
    fn weekly_first_partial_week_and_count_are_direct_and_week_start_explicit() {
        let anchor = d("2024-01-03");
        let r = ScheduleRecurrence::new(
            RecurrenceFrequency::Weekly {
                weekdays: (1 << Weekday::Monday.index()) | (1 << Weekday::Wednesday.index()),
                week_start: Weekday::Monday,
            },
            2,
            RecurrenceEnd::Count(4),
        )
        .unwrap();
        assert_eq!(r.index_on(anchor, anchor), Some(0));
        assert_eq!(r.index_on(anchor, d("2024-01-08")), None);
        assert_eq!(r.index_on(anchor, d("2024-01-15")), Some(1));
        assert_eq!(r.index_on(anchor, d("2024-01-17")), Some(2));
        assert_eq!(r.index_on(anchor, d("2024-01-29")), Some(3));
        assert_eq!(r.index_on(anchor, d("2024-01-31")), None);
        assert!(
            ScheduleRecurrence::new(
                RecurrenceFrequency::Weekly {
                    weekdays: 128,
                    week_start: Weekday::Monday
                },
                1,
                RecurrenceEnd::Never
            )
            .is_err()
        );
        assert!(
            ScheduleRecurrence::new(RecurrenceFrequency::Daily, 0, RecurrenceEnd::Never).is_err()
        );
    }
    #[test]
    fn month_end_and_leap_year_skip_clamp_and_until_have_distinct_count_semantics() {
        let a = d("2024-01-31");
        let skip = ScheduleRecurrence::new(
            RecurrenceFrequency::Monthly {
                missing: MissingScheduleDay::Skip,
            },
            1,
            RecurrenceEnd::Count(3),
        )
        .unwrap();
        let clamp = ScheduleRecurrence::new(
            RecurrenceFrequency::Monthly {
                missing: MissingScheduleDay::Clamp,
            },
            1,
            RecurrenceEnd::Until(d("2024-03-31")),
        )
        .unwrap();
        assert_eq!(skip.index_on(a, d("2024-02-29")), None);
        assert_eq!(skip.index_on(a, d("2024-03-31")), Some(1));
        assert_eq!(skip.index_on(a, d("2024-05-31")), Some(2));
        assert_eq!(skip.index_on(a, d("2024-07-31")), None);
        assert_eq!(clamp.index_on(a, d("2024-02-29")), Some(1));
        assert_eq!(clamp.index_on(a, d("2024-03-31")), Some(2));
        assert_eq!(clamp.index_on(a, d("2024-04-30")), None);
        let a = d("2000-02-29");
        let skip = ScheduleRecurrence::new(
            RecurrenceFrequency::Yearly {
                missing: MissingScheduleDay::Skip,
            },
            1,
            RecurrenceEnd::Never,
        )
        .unwrap();
        let clamp = ScheduleRecurrence::new(
            RecurrenceFrequency::Yearly {
                missing: MissingScheduleDay::Clamp,
            },
            1,
            RecurrenceEnd::Never,
        )
        .unwrap();
        assert_eq!(skip.index_on(a, d("2100-02-28")), None);
        assert_eq!(clamp.index_on(a, d("2100-02-28")), Some(100));
        assert_eq!(skip.index_on(a, d("2400-02-29")), Some(97));
    }
    #[test]
    fn count_arithmetic_matches_brute_force_across_full_civil_domain_without_history() {
        for step in [1, 2, 3, 7, 13, 4799, 4800, 65535] {
            let anchor = d("0001-01-31");
            let rule = ScheduleRecurrence::new(
                RecurrenceFrequency::Monthly {
                    missing: MissingScheduleDay::Skip,
                },
                step,
                RecurrenceEnd::Never,
            )
            .unwrap();
            let mut index = 0;
            for n in (0..119_988u32).step_by(usize::from(step)) {
                let month = crate::YearMonth::new((n / 12 + 1) as u16, (n % 12 + 1) as u8).unwrap();
                if month.days() == 31 {
                    assert_eq!(
                        rule.index_on(anchor, month.last()),
                        Some(index),
                        "stride {step}, {month}"
                    );
                    index += 1;
                }
            }
        }
        for step in [1, 2, 3, 4, 100, 400, 65535] {
            let anchor = d("0004-02-29");
            let rule = ScheduleRecurrence::new(
                RecurrenceFrequency::Yearly {
                    missing: MissingScheduleDay::Skip,
                },
                step,
                RecurrenceEnd::Never,
            )
            .unwrap();
            let mut index = 0;
            for year in (4..=9999u16).step_by(usize::from(step)) {
                if let Ok(day) = CivilDate::new(year, 2, 29) {
                    assert_eq!(rule.index_on(anchor, day), Some(index));
                    index += 1;
                }
            }
        }
    }
    #[test]
    fn iterator_admission_reads_only_limit_plus_one_and_rejects_duplicate_complete_keys() {
        let read = Rc::new(Cell::new(0));
        let r = read.clone();
        let source = (0..1_000_000).map(move |_| {
            r.set(r.get() + 1);
            event()
        });
        assert_eq!(
            ScheduleData::new(
                ScheduleZone::Civil,
                ScheduleWindow::days(d("2024-02-01"), 1).unwrap(),
                source
            )
            .unwrap_err(),
            ScheduleError::TooManyEvents
        );
        assert_eq!(read.get(), MAX_SCHEDULE_EVENTS + 1);
        let read = Cell::new(0);
        let result = event().except_dates((0..1_000_000).map(|_| {
            read.set(read.get() + 1);
            d("2024-02-01")
        }));
        assert_eq!(result.unwrap_err(), ScheduleError::TooManyExceptions);
        assert_eq!(read.get(), MAX_SCHEDULE_EXCEPTIONS + 1);
        assert_eq!(
            ScheduleData::new(
                ScheduleZone::Civil,
                ScheduleWindow::days(d("2024-02-01"), 1).unwrap(),
                [event(), event()]
            )
            .unwrap_err(),
            ScheduleError::DuplicateKey
        );
        assert!(ScheduleKey::new("1".repeat(1_000_000)).is_err());
    }
    #[test]
    fn occurrence_budget_rejects_whole_expansion_and_half_open_lookback_preserves_overlap() {
        let e = event()
            .recurring(
                ScheduleRecurrence::new(RecurrenceFrequency::Daily, 1, RecurrenceEnd::Never)
                    .unwrap(),
            )
            .unwrap();
        let data = ScheduleData::new(
            ScheduleZone::Civil,
            ScheduleWindow::days(d("2024-02-01"), 4).unwrap(),
            [e],
        )
        .unwrap();
        assert_eq!(
            data.expand(3).unwrap_err(),
            ScheduleError::TooManyOccurrences
        );
        assert_eq!(data.expand(4).unwrap().len(), 4);
        assert_eq!(
            data.expand(MAX_SCHEDULE_OCCURRENCES + 1).unwrap_err(),
            ScheduleError::TooManyOccurrences
        );
        let e = ScheduleEvent::new(
            ScheduleKey::new("all-day").unwrap(),
            "All day",
            ScheduleInterval::all_day(d("2024-02-01"), d("2024-02-03")).unwrap(),
        )
        .unwrap()
        .all_day()
        .unwrap();
        let data = ScheduleData::new(
            ScheduleZone::Civil,
            ScheduleWindow::days(d("2024-02-03"), 1).unwrap(),
            [e],
        )
        .unwrap();
        assert!(data.expand(4).unwrap().is_empty());
    }
    #[test]
    fn application_zone_is_only_a_key_and_gap_fold_and_instant_order_are_explicit() {
        let i = interval("2024-02-29T01:00", "2024-02-29T02:00");
        assert_eq!(
            ScheduleContext::civil().resolve(i).unwrap(),
            ScheduleResolution::Civil
        );
        let context = ScheduleContext::application(
            ScheduleKey::new("application-zone").unwrap(),
            AmbiguityPolicy::Reject,
            |_| LocalResolution::Gap,
        );
        assert_eq!(
            context.resolve(i),
            Err(ScheduleError::Resolution(DateTimeResolutionError::Gap))
        );
        let context = ScheduleContext::application(
            ScheduleKey::new("fold").unwrap(),
            AmbiguityPolicy::Earlier,
            |local| LocalResolution::Ambiguous {
                earlier: ResolvedDateTime::from_fixed(local, FixedOffset::new(120).unwrap()),
                later: ResolvedDateTime::from_fixed(local, FixedOffset::new(60).unwrap()),
            },
        );
        assert!(matches!(
            context.resolve(i),
            Ok(ScheduleResolution::Instants { .. })
        ));
        let context = ScheduleContext::application(
            ScheduleKey::new("reversed").unwrap(),
            AmbiguityPolicy::Reject,
            |local| {
                LocalResolution::Unique(ResolvedDateTime::from_fixed(
                    local,
                    FixedOffset::new(if local.time().hour() == 1 { 0 } else { 180 }).unwrap(),
                ))
            },
        );
        assert_eq!(context.resolve(i), Err(ScheduleError::InstantOrder));
    }
    #[test]
    fn resources_and_conflicts_are_bounded_and_touching_intervals_do_not_conflict() {
        let room = ScheduleKey::new("room").unwrap();
        let other = ScheduleKey::new("other").unwrap();
        let resources = ScheduleResources::new([
            ScheduleResource::new(room.clone(), "Room").unwrap(),
            ScheduleResource::new(other.clone(), "Other").unwrap(),
        ])
        .unwrap();
        assert_eq!(resources.window(0, 1).unwrap()[0].title(), "Room");
        assert!(resources.window(usize::MAX, 1).is_err());
        assert!(resources.window(0, 9).is_err());
        let count = Cell::new(0);
        let result = ScheduleResources::new((0..1_000_000).map(|_| {
            count.set(count.get() + 1);
            ScheduleResource::new(room.clone(), "Room").unwrap()
        }));
        assert_eq!(result.unwrap_err(), ScheduleError::TooManyResources);
        assert_eq!(count.get(), 65);
        let events = [
            event().resource(room.clone()),
            ScheduleEvent::new(
                ScheduleKey::new("b").unwrap(),
                "B",
                interval("2024-02-01T09:30", "2024-02-01T11:00"),
            )
            .unwrap()
            .resource(other),
            ScheduleEvent::new(
                ScheduleKey::new("c").unwrap(),
                "C",
                interval("2024-02-01T10:00", "2024-02-01T11:00"),
            )
            .unwrap()
            .resource(room),
        ];
        let data = ScheduleData::new(
            ScheduleZone::Civil,
            ScheduleWindow::days(d("2024-02-01"), 1).unwrap(),
            events,
        )
        .unwrap();
        data.validate_resources(&resources).unwrap();
        assert!(
            data.conflicts(ScheduleConflictPolicy::SameResource, 8)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            data.conflicts(ScheduleConflictPolicy::AllResources, 8)
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            data.conflicts(ScheduleConflictPolicy::AllResources, 1)
                .unwrap_err(),
            ScheduleError::TooManyConflicts
        );
    }
    #[test]
    fn editing_is_an_explicit_permission_checked_current_occurrence_proposal() {
        let room = ScheduleKey::new("room").unwrap();
        let resources =
            ScheduleResources::new([ScheduleResource::new(room.clone(), "Room").unwrap()]).unwrap();
        let window = ScheduleWindow::days(d("2024-02-01"), 1).unwrap();
        let editable = event().permissions(SchedulePermissions {
            select: true,
            move_event: true,
            resize_event: true,
        });
        let data = ScheduleData::new(ScheduleZone::Civil, window, [editable]).unwrap();
        let mut edit = ScheduleEdit {
            event: ScheduleKey::new("event").unwrap(),
            candidate: 0,
            kind: ScheduleEditKind::Move,
            original: interval("2024-02-01T09:00", "2024-02-01T10:00"),
            proposed: interval("2024-02-01T10:00", "2024-02-01T11:00"),
            original_resource: None,
            proposed_resource: Some(room),
        };
        data.validate_edit(&resources, &edit).unwrap();
        edit.candidate = 1;
        assert_eq!(
            data.validate_edit(&resources, &edit),
            Err(ScheduleError::StaleOccurrence)
        );
        edit.candidate = 0;
        edit.proposed = interval("2024-02-01T10:00", "2024-02-01T12:00");
        assert_eq!(
            data.validate_edit(&resources, &edit),
            Err(ScheduleError::InvalidInterval)
        );
        let locked = ScheduleData::new(ScheduleZone::Civil, window, [event()]).unwrap();
        assert_eq!(
            locked.validate_edit(&resources, &edit),
            Err(ScheduleError::EditNotAllowed)
        );
        edit.kind = ScheduleEditKind::ResizeEnd;
        edit.proposed_resource = None;
        edit.proposed = interval("2024-02-01T09:00", "2024-02-01T12:00");
        data.validate_edit(&resources, &edit).unwrap();
    }
}
