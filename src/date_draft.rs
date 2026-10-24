//! Bounded text drafts are distinct from committed civil values.
use crate::{CivilDate, DateBounds};
pub const MAX_DATE_DRAFT_BYTES: usize = 64;
/// A parser outcome contains no arbitrary diagnostic payload or timezone guess.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateParse {
    Empty,
    Incomplete,
    Date(CivilDate),
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateDraftStatus {
    Empty,
    Incomplete,
    Valid(CivilDate),
    Invalid,
    OutOfBounds(CivilDate),
    TooLong,
}
impl DateDraftStatus {
    pub fn proposal(self) -> Option<Option<CivilDate>> {
        match self {
            Self::Empty => Some(None),
            Self::Valid(d) => Some(Some(d)),
            _ => None,
        }
    }
    pub fn is_error(self) -> bool {
        matches!(self, Self::Invalid | Self::OutOfBounds(_) | Self::TooLong)
    }
}
/// Exact YYYY-MM-DD with finite prefix states. No trimming or implicit local clock.
pub fn parse_iso_date_draft(text: &str) -> DateParse {
    if text.is_empty() {
        return DateParse::Empty;
    }
    if text.len() > 10 || !text.is_ascii() {
        return DateParse::Invalid;
    }
    for (i, b) in text.bytes().enumerate() {
        if if i == 4 || i == 7 {
            b != b'-'
        } else {
            !b.is_ascii_digit()
        } {
            return DateParse::Invalid;
        }
    }
    if text.len() < 10 {
        return DateParse::Incomplete;
    }
    match CivilDate::parse_iso(text) {
        Ok(d) => DateParse::Date(d),
        Err(_) => DateParse::Invalid,
    }
}
/// Budget is checked before invoking the optional application parser.
pub fn classify_date_draft(
    text: &str,
    bounds: DateBounds,
    parser: impl FnOnce(&str) -> DateParse,
) -> DateDraftStatus {
    if text.len() > MAX_DATE_DRAFT_BYTES {
        return DateDraftStatus::TooLong;
    }
    match parser(text) {
        DateParse::Empty => DateDraftStatus::Empty,
        DateParse::Incomplete => DateDraftStatus::Incomplete,
        DateParse::Invalid => DateDraftStatus::Invalid,
        DateParse::Date(d) if bounds.contains(d) => DateDraftStatus::Valid(d),
        DateParse::Date(d) => DateDraftStatus::OutOfBounds(d),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prefixes_are_drafts_and_complete_invalid_dates_never_become_committed_values() {
        for s in ["2", "2024", "2024-", "2024-02", "2024-02-"] {
            assert_eq!(parse_iso_date_draft(s), DateParse::Incomplete);
        }
        for s in [
            "2023-02-29",
            "2024-13-01",
            "2024-01-00",
            "2024/02/29",
            "2024-02-29Z",
            " 2024-02-29",
            "２０２４",
        ] {
            assert_eq!(parse_iso_date_draft(s), DateParse::Invalid);
        }
        assert_eq!(parse_iso_date_draft(""), DateParse::Empty);
        assert!(matches!(
            parse_iso_date_draft("2024-02-29"),
            DateParse::Date(_)
        ));
    }
    #[test]
    fn outside_bounds_and_incomplete_preserve_the_business_value_and_empty_is_explicit() {
        let date = CivilDate::new(2024, 2, 29).unwrap();
        let bounds = DateBounds::new(Some(date), Some(date)).unwrap();
        assert_eq!(
            classify_date_draft("2024-02-28", bounds, parse_iso_date_draft),
            DateDraftStatus::OutOfBounds(CivilDate::new(2024, 2, 28).unwrap())
        );
        assert_eq!(
            classify_date_draft("2024-", bounds, parse_iso_date_draft).proposal(),
            None
        );
        assert_eq!(
            classify_date_draft("", bounds, parse_iso_date_draft).proposal(),
            Some(None)
        );
        assert_eq!(
            classify_date_draft("2024-02-29", bounds, parse_iso_date_draft).proposal(),
            Some(Some(date))
        );
    }
    #[test]
    fn rejected_huge_input_does_not_call_application_parser_and_custom_parser_gets_bounded_borrow()
     {
        assert_eq!(
            classify_date_draft(&"x".repeat(1_000_000), DateBounds::default(), |_| panic!(
                "must not call"
            )),
            DateDraftStatus::TooLong
        );
        let date = CivilDate::new(2024, 2, 29).unwrap();
        assert_eq!(
            classify_date_draft("29/02/2024", DateBounds::default(), |v| {
                assert_eq!(v, "29/02/2024");
                DateParse::Date(date)
            }),
            DateDraftStatus::Valid(date)
        );
    }
}
