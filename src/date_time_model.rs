//! Explicit civil date/time and application-owned resolution policy.
use crate::{
    CivilDate, DateBounds, DateError, DatesProvider, FixedOffset, LocalDateTime, LocalTime,
    TimeBounds, UnixInstant,
};
use std::{fmt, rc::Rc};
pub const MAX_DATE_TIME_TEXT_BYTES: usize = 29;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedDateTime {
    instant: UnixInstant,
    offset: FixedOffset,
}
impl ResolvedDateTime {
    pub fn from_fixed(local: LocalDateTime, offset: FixedOffset) -> Self {
        Self {
            instant: local.resolve_fixed(offset),
            offset,
        }
    }
    pub const fn instant(self) -> UnixInstant {
        self.instant
    }
    pub const fn offset(self) -> FixedOffset {
        self.offset
    }
    fn matches(self, local: LocalDateTime) -> bool {
        local.resolve_fixed(self.offset) == self.instant
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalResolution {
    Unique(ResolvedDateTime),
    Ambiguous {
        earlier: ResolvedDateTime,
        later: ResolvedDateTime,
    },
    Gap,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmbiguityPolicy {
    Reject,
    Earlier,
    Later,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateTimeResolution {
    Civil,
    Instant(ResolvedDateTime),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateTimeResolutionError {
    Ambiguous {
        earlier: ResolvedDateTime,
        later: ResolvedDateTime,
    },
    Gap,
    InvalidResolver,
}
impl fmt::Display for DateTimeResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Ambiguous { .. } => "ambiguous local date/time requires an explicit choice",
            Self::Gap => "local date/time does not exist in the supplied application rules",
            Self::InvalidResolver => {
                "resolver candidates do not match the local date/time or chronological order"
            }
        })
    }
}
impl std::error::Error for DateTimeResolutionError {}
/// No IANA/DST rules are installed. An application resolver owns its rules and zone.
#[derive(Clone)]
pub enum DateTimePolicy {
    Civil,
    FixedOffset(FixedOffset),
    Application {
        resolver: Rc<dyn Fn(LocalDateTime) -> LocalResolution>,
        ambiguity: AmbiguityPolicy,
    },
}
impl DateTimePolicy {
    pub fn application(
        ambiguity: AmbiguityPolicy,
        resolver: impl Fn(LocalDateTime) -> LocalResolution + 'static,
    ) -> Self {
        Self::Application {
            resolver: Rc::new(resolver),
            ambiguity,
        }
    }
    pub fn resolve(
        &self,
        local: LocalDateTime,
    ) -> Result<DateTimeResolution, DateTimeResolutionError> {
        match self {
            Self::Civil => Ok(DateTimeResolution::Civil),
            Self::FixedOffset(offset) => Ok(DateTimeResolution::Instant(
                ResolvedDateTime::from_fixed(local, *offset),
            )),
            Self::Application {
                resolver,
                ambiguity,
            } => match resolver(local) {
                LocalResolution::Gap => Err(DateTimeResolutionError::Gap),
                LocalResolution::Unique(value) if value.matches(local) => {
                    Ok(DateTimeResolution::Instant(value))
                }
                LocalResolution::Ambiguous { earlier, later }
                    if earlier.matches(local)
                        && later.matches(local)
                        && earlier.instant < later.instant =>
                {
                    match ambiguity {
                        AmbiguityPolicy::Reject => {
                            Err(DateTimeResolutionError::Ambiguous { earlier, later })
                        }
                        AmbiguityPolicy::Earlier => Ok(DateTimeResolution::Instant(earlier)),
                        AmbiguityPolicy::Later => Ok(DateTimeResolution::Instant(later)),
                    }
                }
                _ => Err(DateTimeResolutionError::InvalidResolver),
            },
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateTimeBounds {
    min: Option<LocalDateTime>,
    max: Option<LocalDateTime>,
}
impl DateTimeBounds {
    pub fn new(min: Option<LocalDateTime>, max: Option<LocalDateTime>) -> Result<Self, DateError> {
        if min.zip(max).is_some_and(|(min, max)| min > max) {
            Err(DateError::OutOfRange)
        } else {
            Ok(Self { min, max })
        }
    }
    pub const fn min(self) -> Option<LocalDateTime> {
        self.min
    }
    pub const fn max(self) -> Option<LocalDateTime> {
        self.max
    }
    pub fn contains(self, value: LocalDateTime) -> bool {
        self.min.is_none_or(|min| value >= min) && self.max.is_none_or(|max| value <= max)
    }
    pub fn date_bounds(self) -> DateBounds {
        DateBounds::new(self.min.map(|v| v.date()), self.max.map(|v| v.date())).unwrap()
    }
    pub fn time_bounds(self, date: Option<CivilDate>) -> TimeBounds {
        TimeBounds::new(
            self.min
                .filter(|min| Some(min.date()) == date)
                .map(|v| v.time()),
            self.max
                .filter(|max| Some(max.date()) == date)
                .map(|v| v.time()),
        )
        .unwrap()
    }
    pub fn provider(self, provider: DatesProvider) -> Result<DatesProvider, DateError> {
        let dates = provider.date_bounds();
        let own = self.date_bounds();
        let min = match (dates.min(), own.min()) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        };
        let max = match (dates.max(), own.max()) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        Ok(provider.bounds(DateBounds::new(min, max)?))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTimeOutcome {
    pub local: LocalDateTime,
    pub resolution: DateTimeResolution,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateTimeStatus {
    Empty,
    Incomplete,
    Valid(DateTimeOutcome),
    OutOfBounds(LocalDateTime),
    Ambiguous {
        earlier: ResolvedDateTime,
        later: ResolvedDateTime,
    },
    Gap,
    InvalidResolver,
    InvalidTime(crate::TimePickerStatus),
}
impl DateTimeStatus {
    pub fn proposal(self) -> Option<Option<LocalDateTime>> {
        match self {
            Self::Empty => Some(None),
            Self::Valid(value) => Some(Some(value.local)),
            _ => None,
        }
    }
    pub fn is_error(self) -> bool {
        matches!(
            self,
            Self::OutOfBounds(_)
                | Self::Ambiguous { .. }
                | Self::Gap
                | Self::InvalidResolver
                | Self::InvalidTime(_)
        )
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateTimeParts {
    pub date: Option<CivilDate>,
    pub time: Option<LocalTime>,
}
impl DateTimeParts {
    pub fn from_local(value: Option<LocalDateTime>) -> Self {
        Self {
            date: value.map(|v| v.date()),
            time: value.map(|v| v.time()),
        }
    }
    /// Call an application resolver outside reactive-value and registry borrows.
    pub fn classify(self, bounds: DateTimeBounds, policy: &DateTimePolicy) -> DateTimeStatus {
        let local = match (self.date, self.time) {
            (None, None) => return DateTimeStatus::Empty,
            (Some(date), Some(time)) => LocalDateTime::new(date, time),
            _ => return DateTimeStatus::Incomplete,
        };
        if !bounds.contains(local) {
            return DateTimeStatus::OutOfBounds(local);
        }
        match policy.resolve(local) {
            Ok(resolution) => DateTimeStatus::Valid(DateTimeOutcome { local, resolution }),
            Err(DateTimeResolutionError::Ambiguous { earlier, later }) => {
                DateTimeStatus::Ambiguous { earlier, later }
            }
            Err(DateTimeResolutionError::Gap) => DateTimeStatus::Gap,
            Err(DateTimeResolutionError::InvalidResolver) => DateTimeStatus::InvalidResolver,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTimeDraft {
    pub parts: DateTimeParts,
    pub status: DateTimeStatus,
}
pub fn format_local_date_time(value: LocalDateTime) -> String {
    format!("{}T{}", value.date(), value.time())
}
pub fn parse_local_date_time(text: &str) -> Result<LocalDateTime, DateError> {
    if text.len() > MAX_DATE_TIME_TEXT_BYTES {
        return Err(DateError::InvalidText);
    }
    let Some((date, time)) = text.split_once('T') else {
        return Err(DateError::InvalidText);
    };
    Ok(LocalDateTime::new(
        CivilDate::parse_iso(date)?,
        LocalTime::parse_iso(time)?,
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn local(s: &str) -> LocalDateTime {
        parse_local_date_time(s).unwrap()
    }
    #[test]
    fn exact_civil_text_roundtrips_without_offset_or_implicit_utc() {
        for s in [
            "0001-01-01T00:00:00",
            "9999-12-31T23:59:59.999999999",
            "2024-02-29T12:34:56.123",
        ] {
            let v = local(s);
            assert_eq!(
                parse_local_date_time(&format_local_date_time(v)).unwrap(),
                v
            );
        }
        for bad in [
            "2024-02-29 12:34",
            "2024-02-29T12:34Z",
            "2024-02-29T12:34+01:00",
            "2023-02-29T00:00",
            "😀T00:00",
        ] {
            assert!(parse_local_date_time(bad).is_err());
        }
        assert!(parse_local_date_time(&"1".repeat(1_000_000)).is_err());
    }
    #[test]
    fn missing_date_or_time_stays_a_draft_and_civil_policy_has_no_instant() {
        let value = local("2024-02-29T12:34");
        let p = DateTimeParts::from_local(Some(value));
        let bounds = DateTimeBounds::default();
        assert_eq!(
            p.classify(bounds, &DateTimePolicy::Civil),
            DateTimeStatus::Valid(DateTimeOutcome {
                local: value,
                resolution: DateTimeResolution::Civil
            })
        );
        assert_eq!(
            DateTimeParts {
                date: p.date,
                time: None
            }
            .classify(bounds, &DateTimePolicy::Civil),
            DateTimeStatus::Incomplete
        );
        assert_eq!(
            DateTimeParts {
                date: None,
                time: p.time
            }
            .classify(bounds, &DateTimePolicy::Civil),
            DateTimeStatus::Incomplete
        );
        assert_eq!(
            DateTimeParts::default()
                .classify(bounds, &DateTimePolicy::Civil)
                .proposal(),
            Some(None)
        );
    }
    #[test]
    fn explicit_fixed_offset_resolves_and_returns_the_exact_same_local_fields() {
        let v = local("2024-02-29T23:59:59");
        let offset = FixedOffset::new(345).unwrap();
        let DateTimeResolution::Instant(resolved) =
            DateTimePolicy::FixedOffset(offset).resolve(v).unwrap()
        else {
            panic!()
        };
        assert_eq!(resolved.offset(), offset);
        assert_eq!(resolved.instant().to_local(offset).unwrap(), v);
    }
    #[test]
    fn application_fold_requires_an_explicit_policy_and_preserves_chronological_candidates() {
        let v = local("2024-11-03T01:30");
        let earlier = ResolvedDateTime::from_fixed(v, FixedOffset::new(120).unwrap());
        let later = ResolvedDateTime::from_fixed(v, FixedOffset::new(60).unwrap());
        for (policy, expected) in [
            (AmbiguityPolicy::Earlier, earlier),
            (AmbiguityPolicy::Later, later),
        ] {
            assert_eq!(
                DateTimePolicy::application(policy, move |_| LocalResolution::Ambiguous {
                    earlier,
                    later
                })
                .resolve(v)
                .unwrap(),
                DateTimeResolution::Instant(expected)
            );
        }
        assert_eq!(
            DateTimePolicy::application(AmbiguityPolicy::Reject, move |_| {
                LocalResolution::Ambiguous { earlier, later }
            })
            .resolve(v),
            Err(DateTimeResolutionError::Ambiguous { earlier, later })
        );
    }
    #[test]
    fn gaps_wrong_local_candidates_and_reversed_fold_order_are_typed_rejections() {
        let v = local("2024-03-10T02:30");
        assert_eq!(
            DateTimePolicy::application(AmbiguityPolicy::Earlier, |_| LocalResolution::Gap)
                .resolve(v),
            Err(DateTimeResolutionError::Gap)
        );
        let wrong = ResolvedDateTime::from_fixed(local("2024-03-10T03:30"), FixedOffset::UTC);
        assert_eq!(
            DateTimePolicy::application(AmbiguityPolicy::Reject, move |_| {
                LocalResolution::Unique(wrong)
            })
            .resolve(v),
            Err(DateTimeResolutionError::InvalidResolver)
        );
        let one = ResolvedDateTime::from_fixed(v, FixedOffset::UTC);
        let two = ResolvedDateTime::from_fixed(v, FixedOffset::new(60).unwrap());
        assert_eq!(
            DateTimePolicy::application(AmbiguityPolicy::Later, move |_| {
                LocalResolution::Ambiguous {
                    earlier: one,
                    later: two,
                }
            })
            .resolve(v),
            Err(DateTimeResolutionError::InvalidResolver)
        );
    }
    #[test]
    fn bounds_project_edge_times_without_clamping_or_inventing_a_missing_day() {
        let min = local("2024-02-29T09:00");
        let max = local("2024-03-01T17:00");
        let b = DateTimeBounds::new(Some(min), Some(max)).unwrap();
        assert_eq!(b.time_bounds(Some(min.date())).min(), Some(min.time()));
        assert_eq!(b.time_bounds(Some(max.date())).max(), Some(max.time()));
        assert_eq!(b.time_bounds(None), TimeBounds::default());
        assert!(DateTimeBounds::new(Some(max), Some(min)).is_err());
        let p = DatesProvider::new(min.date())
            .bounds(DateBounds::new(Some(min.date()), Some(min.date())).unwrap());
        assert_eq!(b.provider(p).unwrap().date_bounds().max(), Some(min.date()));
        let bad = DatesProvider::new(CivilDate::MIN)
            .bounds(DateBounds::new(None, Some(CivilDate::MIN)).unwrap());
        assert!(b.provider(bad).is_err());
    }
}
