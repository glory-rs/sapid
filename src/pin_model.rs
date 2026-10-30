//! Finite public policy for one actual input; error payloads never contain codes.
use std::fmt;
pub const MAX_PIN_LENGTH: usize = 16;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PinAlphabet {
    #[default]
    Digits,
    AsciiAlphanumeric,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinRejection {
    InvalidLength,
    TooLong,
    InvalidCharacter,
}
impl fmt::Display for PinRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidLength => "PIN length must be between 1 and 16",
            Self::TooLong => "PIN input exceeds its configured length",
            Self::InvalidCharacter => {
                "PIN input contains a character outside its configured alphabet"
            }
        })
    }
}
impl std::error::Error for PinRejection {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PinDomain {
    pub length: usize,
    pub alphabet: PinAlphabet,
}
impl Default for PinDomain {
    fn default() -> Self {
        Self {
            length: 6,
            alphabet: PinAlphabet::Digits,
        }
    }
}
impl PinDomain {
    pub fn validate(self) -> Result<(), PinRejection> {
        if (1..=MAX_PIN_LENGTH).contains(&self.length) {
            Ok(())
        } else {
            Err(PinRejection::InvalidLength)
        }
    }
    pub fn validate_value(self, value: &str) -> Result<(), PinRejection> {
        self.validate()?;
        if value.len() > self.length {
            return Err(PinRejection::TooLong);
        }
        if !value.bytes().all(|c| match self.alphabet {
            PinAlphabet::Digits => c.is_ascii_digit(),
            PinAlphabet::AsciiAlphanumeric => c.is_ascii_alphanumeric(),
        }) {
            return Err(PinRejection::InvalidCharacter);
        }
        Ok(())
    }
    pub fn complete(self, value: &str) -> bool {
        self.validate_value(value).is_ok() && value.len() == self.length
    }
    pub fn completed_user_transition(self, before: &str, proposal: &str, accepted: &str) -> bool {
        before != proposal && proposal == accepted && self.complete(proposal)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn domain_is_finite_and_rejects_invalid_counts_before_allocating_cells() {
        for length in [0, 17, usize::MAX] {
            assert_eq!(
                PinDomain {
                    length,
                    ..Default::default()
                }
                .validate(),
                Err(PinRejection::InvalidLength)
            );
        }
        for length in 1..=16 {
            assert!(
                PinDomain {
                    length,
                    ..Default::default()
                }
                .validate()
                .is_ok()
            );
        }
    }
    #[test]
    fn one_model_accepts_empty_partial_and_whole_codes_without_numeric_conversion() {
        let domain = PinDomain::default();
        for value in ["", "0", "000", "000000"] {
            assert!(domain.validate_value(value).is_ok());
        }
        assert!(!domain.complete("000"));
        assert!(domain.complete("000000"));
    }
    #[test]
    fn strict_ascii_policy_does_not_discard_characters_from_pastes() {
        let domain = PinDomain::default();
        for value in ["123 45", "123-45", "１２３", "一二三", "12\n34", "12\0"] {
            assert!(domain.validate_value(value).is_err());
        }
        assert_eq!(domain.validate_value("0000000"), Err(PinRejection::TooLong));
    }
    #[test]
    fn alphanumeric_policy_preserves_case_and_excludes_punctuation() {
        let domain = PinDomain {
            alphabet: PinAlphabet::AsciiAlphanumeric,
            ..Default::default()
        };
        assert!(domain.validate_value("aZ09bC").is_ok());
        assert!(domain.complete("aZ09bC"));
        assert!(domain.validate_value("aZ_09b").is_err());
    }
    #[test]
    fn completion_requires_an_accepted_different_user_value_and_has_no_source_history() {
        let domain = PinDomain::default();
        assert!(domain.completed_user_transition("00000", "000000", "000000"));
        assert!(!domain.completed_user_transition("00000", "000000", "00000"));
        assert!(!domain.completed_user_transition("000000", "000000", "000000"));
        assert!(!domain.completed_user_transition("0000", "00000", "00000"));
        assert!(domain.completed_user_transition("000000", "000001", "000001"));
    }
    #[test]
    fn oversized_values_reject_by_borrow_and_error_messages_never_include_the_input() {
        let value = "9".repeat(1_000_000);
        let error = PinDomain::default().validate_value(&value).unwrap_err();
        assert_eq!(error, PinRejection::TooLong);
        assert!(!error.to_string().contains("999"));
        assert!(!format!("{error:?}").contains("999"));
    }
}
