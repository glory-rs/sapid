//! Bounded exact decimal arithmetic and explicit localized editing states.
use crate::NumberLocale;
use std::fmt;
pub const MAX_NUMBER_INPUT_BYTES: usize = 128;
const MAX_SCALE: u8 = 18;
const MAX_INTEGER: i128 = 1_000_000_000_000_000_000;
const ATOM_SCALE: i128 = 1_000_000_000_000_000_000;
const MAX_ATOMS: i128 = MAX_INTEGER * ATOM_SCALE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberError {
    InputTooLong,
    InvalidCharacter,
    InvalidGrouping,
    TooManyFractionDigits,
    OutsideDecimalDomain,
    InvalidRange,
    InvalidStep,
    Incomplete,
}
impl fmt::Display for NumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InputTooLong => "number input exceeds 128 UTF-8 bytes",
            Self::InvalidCharacter => "invalid number character",
            Self::InvalidGrouping => "invalid localized number grouping",
            Self::TooManyFractionDigits => "decimal scale exceeds 18 digits",
            Self::OutsideDecimalDomain => "number exceeds the exact decimal domain",
            Self::InvalidRange => "number minimum exceeds maximum",
            Self::InvalidStep => "number steps must be positive",
            Self::Incomplete => "number input is incomplete",
        })
    }
}
impl std::error::Error for NumberError {}

/// Exact normalized decimal, with at most 18 fractional digits and absolute
/// value at most 10^18. Construction and arithmetic never use binary floats.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Decimal {
    coefficient: i128,
    scale: u8,
}
impl Decimal {
    pub fn new(mut coefficient: i128, mut scale: u8) -> Result<Self, NumberError> {
        if scale > MAX_SCALE {
            return Err(NumberError::TooManyFractionDigits);
        }
        let limit = MAX_INTEGER * 10_i128.pow(u32::from(scale));
        if coefficient.unsigned_abs() > limit as u128 {
            return Err(NumberError::OutsideDecimalDomain);
        }
        while scale > 0 && coefficient % 10 == 0 {
            coefficient /= 10;
            scale -= 1
        }
        Ok(Self { coefficient, scale })
    }
    pub fn coefficient(self) -> i128 {
        self.coefficient
    }
    pub fn scale(self) -> u8 {
        self.scale
    }
    pub fn parse(value: &str) -> Result<Self, NumberError> {
        match parse_number(value, NumberLocale::EnUs) {
            NumberParse::Complete(value) => Ok(value),
            NumberParse::Invalid(error) => Err(error),
            _ => Err(NumberError::Incomplete),
        }
    }
    fn atoms(self) -> i128 {
        self.coefficient * 10_i128.pow(u32::from(MAX_SCALE - self.scale))
    }
    fn from_atoms(atoms: i128) -> Result<Self, NumberError> {
        Self::new(atoms, MAX_SCALE)
    }
    /// A lossy accessibility coordinate; exact values and stepping stay decimal.
    pub fn to_f64(self) -> f64 {
        self.coefficient as f64 / 10_f64.powi(i32::from(self.scale))
    }
    pub fn format(self, locale: NumberLocale, grouping: bool) -> String {
        let canonical = self.to_string();
        let (sign, unsigned) = canonical
            .strip_prefix('-')
            .map_or(("", canonical.as_str()), |v| ("-", v));
        let (integer, fraction) = unsigned
            .split_once('.')
            .map_or((unsigned, None), |(a, b)| (a, Some(b)));
        let (decimal, group) = separators(locale);
        let mut result = String::with_capacity(48);
        result.push_str(sign);
        for (index, digit) in integer.bytes().enumerate() {
            if grouping && index > 0 && (integer.len() - index) % 3 == 0 {
                result.push(char::from(group))
            }
            result.push(char::from(digit))
        }
        if let Some(fraction) = fraction {
            result.push(char::from(decimal));
            result.push_str(fraction)
        }
        result
    }
}
impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let digits = self.coefficient.unsigned_abs().to_string();
        let scale = usize::from(self.scale);
        if self.coefficient < 0 {
            f.write_str("-")?
        }
        if scale == 0 {
            f.write_str(&digits)
        } else if digits.len() > scale {
            let split = digits.len() - scale;
            write!(f, "{}.{}", &digits[..split], &digits[split..])
        } else {
            write!(f, "0.{}{}", "0".repeat(scale - digits.len()), digits)
        }
    }
}
impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Decimal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.atoms().cmp(&other.atoms())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum NumberValue {
    #[default]
    Empty,
    Decimal(Decimal),
    /// All user lexemes, including complete numbers, preserve their raw text.
    Text(String),
}
impl From<Decimal> for NumberValue {
    fn from(value: Decimal) -> Self {
        Self::Decimal(value)
    }
}
impl NumberValue {
    pub fn validate(&self) -> Result<(), NumberError> {
        if let Self::Text(text) = self {
            if text.len() > MAX_NUMBER_INPUT_BYTES {
                return Err(NumberError::InputTooLong);
            }
            if text.contains(['\0', '\r', '\n']) {
                return Err(NumberError::InvalidCharacter);
            }
        }
        Ok(())
    }
    pub fn parse(&self, locale: NumberLocale) -> NumberParse {
        match self {
            Self::Empty => NumberParse::Empty,
            Self::Decimal(value) => NumberParse::Complete(*value),
            Self::Text(text) => parse_number(text, locale),
        }
    }
    pub fn text(&self, locale: NumberLocale, grouping: bool) -> Result<String, NumberError> {
        self.validate()?;
        Ok(match self {
            Self::Empty => String::new(),
            Self::Decimal(value) => value.format(locale, grouping),
            Self::Text(value) => value.clone(),
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberParse {
    Empty,
    Complete(Decimal),
    Incomplete { candidate: Option<Decimal> },
    Invalid(NumberError),
}
fn separators(locale: NumberLocale) -> (u8, u8) {
    match locale {
        NumberLocale::DeDe => (b',', b'.'),
        NumberLocale::EnUs | NumberLocale::ZhCn => (b'.', b','),
    }
}
pub fn parse_number(text: &str, locale: NumberLocale) -> NumberParse {
    match parse_inner(text, locale) {
        Ok(value) => value,
        Err(error) => NumberParse::Invalid(error),
    }
}
fn parse_inner(text: &str, locale: NumberLocale) -> Result<NumberParse, NumberError> {
    if text.len() > MAX_NUMBER_INPUT_BYTES {
        return Err(NumberError::InputTooLong);
    }
    if text.is_empty() {
        return Ok(NumberParse::Empty);
    }
    let bytes = text.as_bytes();
    let (negative, unsigned) = match bytes[0] {
        b'-' => (true, &bytes[1..]),
        b'+' => (false, &bytes[1..]),
        _ => (false, bytes),
    };
    let (decimal, group) = separators(locale);
    if unsigned
        .iter()
        .any(|b| !b.is_ascii_digit() && *b != decimal && *b != group)
    {
        return Err(NumberError::InvalidCharacter);
    }
    let mut split = unsigned.split(|b| *b == decimal);
    let integer = split.next().unwrap_or_default();
    let fraction = split.next();
    if split.next().is_some() {
        return Err(NumberError::InvalidCharacter);
    }
    let parts: Vec<_> = integer.split(|b| *b == group).collect();
    if parts.len() > 1 {
        if parts[0].is_empty() || parts[0].len() > 3 {
            return Err(NumberError::InvalidGrouping);
        }
        for (index, part) in parts.iter().enumerate().skip(1) {
            if part.len() != 3 {
                if index == parts.len() - 1 && part.len() < 3 && fraction.is_none() {
                    return Ok(NumberParse::Incomplete { candidate: None });
                }
                return Err(NumberError::InvalidGrouping);
            }
        }
    }
    let fraction = fraction.unwrap_or_default();
    if fraction.iter().any(|b| !b.is_ascii_digit()) {
        return Err(NumberError::InvalidGrouping);
    }
    if fraction.len() > usize::from(MAX_SCALE) {
        return Err(NumberError::TooManyFractionDigits);
    }
    let digit_count = parts.iter().map(|v| v.len()).sum::<usize>() + fraction.len();
    if digit_count == 0 {
        return Ok(NumberParse::Incomplete { candidate: None });
    }
    let mut coefficient = 0_i128;
    for digit in parts.iter().flat_map(|part| part.iter()).chain(fraction) {
        coefficient = coefficient
            .checked_mul(10)
            .and_then(|v| v.checked_add(i128::from(*digit - b'0')))
            .ok_or(NumberError::OutsideDecimalDomain)?;
    }
    if negative {
        coefficient = -coefficient
    }
    let value = Decimal::new(coefficient, fraction.len() as u8)?;
    if unsigned.last() == Some(&decimal) {
        Ok(NumberParse::Incomplete {
            candidate: Some(value),
        })
    } else {
        Ok(NumberParse::Complete(value))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NumberDomain {
    pub min: Option<Decimal>,
    pub max: Option<Decimal>,
    pub step: Decimal,
    pub page: Decimal,
}
impl NumberDomain {
    pub fn validate(self) -> Result<(), NumberError> {
        if self.min.zip(self.max).is_some_and(|(min, max)| min > max) {
            return Err(NumberError::InvalidRange);
        }
        if self.step.coefficient <= 0 || self.page.coefficient <= 0 {
            return Err(NumberError::InvalidStep);
        }
        Ok(())
    }
    pub fn outside(self, value: Decimal) -> bool {
        self.min.is_some_and(|v| value < v) || self.max.is_some_and(|v| value > v)
    }
    pub fn clamp(self, value: Decimal) -> Decimal {
        self.clamp_atoms(value.atoms())
    }
    fn clamp_atoms(self, atoms: i128) -> Decimal {
        Decimal::from_atoms(atoms.clamp(
            self.min.map_or(-MAX_ATOMS, Decimal::atoms),
            self.max.map_or(MAX_ATOMS, Decimal::atoms),
        ))
        .expect("validated decimal bounds")
    }
    pub fn increment(self, state: NumberParse, increase: bool, page: bool) -> Option<Decimal> {
        let current = match state {
            NumberParse::Complete(v) | NumberParse::Incomplete { candidate: Some(v) } => {
                Some(v)
            }
            _ => None,
        };
        let amount = if page { self.page } else { self.step }.atoms();
        let next = self.clamp_atoms(
            current.map_or(0, Decimal::atoms) + if increase { amount } else { -amount },
        );
        if current.is_some_and(|v| if increase { next <= v } else { next >= v }) {
            None
        } else {
            Some(next)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn d(value: &str) -> Decimal {
        Decimal::parse(value).unwrap()
    }
    fn domain() -> NumberDomain {
        NumberDomain {
            min: Some(d("0")),
            max: Some(d("10")),
            step: d("0.1"),
            page: d("1"),
        }
    }
    #[test]
    fn exact_decimal_normalizes_and_never_uses_binary_float_for_ordering() {
        assert_eq!(Decimal::new(100, 2).unwrap(), d("1"));
        assert_eq!(d("-0.00"), Decimal::default());
        assert_eq!(d("0.000000000000000001").scale(), 18);
        assert_eq!(d("1000000000000000000").coefficient(), MAX_INTEGER);
        assert!(d("999999999999999999.999999999999999999") < d("1000000000000000000"));
        assert!(Decimal::new(i128::MIN, 0).is_err());
        assert!(Decimal::new(1, 19).is_err());
    }
    #[test]
    fn localized_grouping_is_strict_and_drafts_remain_distinct_from_numeric_values() {
        assert_eq!(
            parse_number("1,234.50", NumberLocale::EnUs),
            NumberParse::Complete(d("1234.5"))
        );
        assert_eq!(
            parse_number("1.234,50", NumberLocale::DeDe),
            NumberParse::Complete(d("1234.5"))
        );
        assert_eq!(
            parse_number("1,234.50", NumberLocale::ZhCn),
            NumberParse::Complete(d("1234.5"))
        );
        assert_eq!(
            parse_number("1.", NumberLocale::EnUs),
            NumberParse::Incomplete {
                candidate: Some(d("1"))
            }
        );
        for text in ["-", "+", ".", "-.", "1,", "1,2"] {
            assert!(matches!(
                parse_number(text, NumberLocale::EnUs),
                NumberParse::Incomplete { candidate: None }
            ));
        }
        assert_eq!(
            parse_number("12,34,567", NumberLocale::EnUs),
            NumberParse::Invalid(NumberError::InvalidGrouping)
        );
        assert!(matches!(
            parse_number("1,23.0", NumberLocale::EnUs),
            NumberParse::Invalid(_)
        ));
        assert!(matches!(
            parse_number("1e3", NumberLocale::EnUs),
            NumberParse::Invalid(_)
        ));
    }
    #[test]
    fn source_text_is_bounded_before_cloning_and_preserves_complete_user_lexemes() {
        let value = NumberValue::Text("1,234.00".into());
        assert_eq!(value.text(NumberLocale::EnUs, false).unwrap(), "1,234.00");
        assert_eq!(
            value.parse(NumberLocale::EnUs),
            NumberParse::Complete(d("1234"))
        );
        let huge = NumberValue::Text("9".repeat(1_000_000));
        assert_eq!(
            huge.text(NumberLocale::EnUs, false),
            Err(NumberError::InputTooLong)
        );
        assert!(NumberValue::Text("1\n2".into()).validate().is_err());
        assert_eq!(
            NumberValue::Empty.parse(NumberLocale::EnUs),
            NumberParse::Empty
        );
        assert_eq!(
            NumberValue::from(d("2.5"))
                .text(NumberLocale::DeDe, false)
                .unwrap(),
            "2,5"
        );
    }
    #[test]
    fn fractional_steps_are_exact_and_range_recovery_never_reverses_a_direction() {
        let domain = domain();
        domain.validate().unwrap();
        assert_eq!(
            domain.increment(NumberParse::Complete(d("0.1")), true, false),
            Some(d("0.2"))
        );
        assert_eq!(
            domain.increment(NumberParse::Complete(d("0.2")), true, false),
            Some(d("0.3"))
        );
        assert_eq!(
            domain.increment(NumberParse::Complete(d("20")), true, false),
            None
        );
        assert_eq!(
            domain.increment(NumberParse::Complete(d("20")), false, false),
            Some(d("10"))
        );
        assert_eq!(
            domain.increment(NumberParse::Complete(d("-2")), false, false),
            None
        );
        assert_eq!(
            domain.increment(NumberParse::Complete(d("-2")), true, false),
            Some(d("0"))
        );
        assert_eq!(
            domain.increment(NumberParse::Empty, true, false),
            Some(d("0.1"))
        );
        assert_eq!(
            domain.increment(
                NumberParse::Invalid(NumberError::InvalidCharacter),
                true,
                true
            ),
            Some(d("1"))
        );
        assert_eq!(domain.clamp(d("20")), d("10"));
        assert!(domain.outside(d("20")));
    }
    #[test]
    fn scale_and_absolute_domain_overflow_are_explicit_not_rounded() {
        for text in [
            "0.0000000000000000001",
            "1000000000000000000.1",
            "99999999999999999999999999999999999999999999999",
        ] {
            assert!(
                matches!(
                    parse_number(text, NumberLocale::EnUs),
                    NumberParse::Invalid(_)
                ),
                "{text}"
            );
        }
        let mut invalid = domain();
        invalid.min = Some(d("11"));
        assert_eq!(invalid.validate(), Err(NumberError::InvalidRange));
        invalid = domain();
        invalid.step = Decimal::default();
        assert_eq!(invalid.validate(), Err(NumberError::InvalidStep));
    }
    #[test]
    fn finite_locale_formatting_round_trips_without_an_ambient_locale_or_exponent() {
        let value = d("-1234567.000000000000000001");
        assert_eq!(
            value.format(NumberLocale::EnUs, true),
            "-1,234,567.000000000000000001"
        );
        assert_eq!(
            value.format(NumberLocale::DeDe, true),
            "-1.234.567,000000000000000001"
        );
        for locale in [NumberLocale::EnUs, NumberLocale::DeDe, NumberLocale::ZhCn] {
            assert_eq!(
                parse_number(&value.format(locale, true), locale),
                NumberParse::Complete(value)
            );
        }
        assert!(value.to_f64().is_finite());
        assert_eq!(
            NumberError::Incomplete.to_string(),
            "number input is incomplete"
        );
    }
}
