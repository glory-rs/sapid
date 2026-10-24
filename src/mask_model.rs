//! Bounded ASCII mask model. No host, input listener or reactive dependency.
use std::fmt;
pub const MAX_MASK_PARTS: usize = 128;
pub const MAX_MASK_SLOTS: usize = 64;
pub const MAX_MASK_INPUT_BYTES: usize = 256;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaskRejection {
    InvalidPattern,
    TooManySlots,
    InputTooLong,
    InvalidCharacter,
    InvalidSelection,
}
impl fmt::Display for MaskRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidPattern => "invalid finite ASCII mask",
            Self::TooManySlots => "mask exceeds its slot budget",
            Self::InputTooLong => "masked input exceeds its budget",
            Self::InvalidCharacter => "masked input contains an invalid character",
            Self::InvalidSelection => "masked input selection is invalid",
        })
    }
}
impl std::error::Error for MaskRejection {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Slot {
    Digit,
    Letter,
    Alphanumeric,
}
impl Slot {
    fn accepts(self, value: u8) -> bool {
        match self {
            Self::Digit => value.is_ascii_digit(),
            Self::Letter => value.is_ascii_alphabetic(),
            Self::Alphanumeric => value.is_ascii_alphanumeric(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    Slot(Slot),
    Literal(u8),
}
/// `#` is an ASCII digit, `A` an ASCII letter, `*` ASCII alphanumeric. Backslash
/// escapes a literal. All literals are printable ASCII; no regex VM is installed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaskPattern {
    parts: Vec<Part>,
    slots: Vec<Slot>,
}
impl MaskPattern {
    pub fn parse(pattern: &str) -> Result<Self, MaskRejection> {
        if pattern.len() > MAX_MASK_INPUT_BYTES || !pattern.is_ascii() {
            return Err(MaskRejection::InvalidPattern);
        }
        let mut parts = Vec::new();
        let mut slots = Vec::new();
        let mut escaped = false;
        for byte in pattern.bytes() {
            if !(b' '..=b'~').contains(&byte) {
                return Err(MaskRejection::InvalidPattern);
            }
            if !escaped && byte == b'\\' {
                escaped = true;
                continue;
            }
            let part = if escaped {
                escaped = false;
                Part::Literal(byte)
            } else {
                match byte {
                    b'#' => Part::Slot(Slot::Digit),
                    b'A' => Part::Slot(Slot::Letter),
                    b'*' => Part::Slot(Slot::Alphanumeric),
                    _ => Part::Literal(byte),
                }
            };
            if parts.len() == MAX_MASK_PARTS {
                return Err(MaskRejection::InvalidPattern);
            }
            if let Part::Slot(slot) = part {
                if slots.len() == MAX_MASK_SLOTS {
                    return Err(MaskRejection::TooManySlots);
                }
                slots.push(slot)
            }
            parts.push(part);
        }
        if escaped || slots.is_empty() {
            return Err(MaskRejection::InvalidPattern);
        }
        Ok(Self { parts, slots })
    }
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
    pub fn validate_raw(&self, value: &str) -> Result<(), MaskRejection> {
        self.validate_fragment(0, value)
    }
    fn validate_fragment(&self, start: usize, value: &str) -> Result<(), MaskRejection> {
        if start > self.slots.len() || value.len() > self.slots.len() - start {
            return Err(MaskRejection::InputTooLong);
        }
        if !value
            .bytes()
            .zip(&self.slots[start..])
            .all(|(v, s)| s.accepts(v))
        {
            return Err(MaskRejection::InvalidCharacter);
        }
        Ok(())
    }
    pub fn format(&self, raw: &str) -> Result<MaskedText, MaskRejection> {
        self.validate_raw(raw)?;
        let mut display = String::new();
        let mut to_raw = vec![0];
        let mut to_display = vec![0; raw.len() + 1];
        let mut count = 0;
        if !raw.is_empty() {
            for part in &self.parts {
                match *part {
                    Part::Literal(byte) => {
                        display.push(char::from(byte));
                        to_raw.push(count);
                    }
                    Part::Slot(_) => {
                        to_display[count] = display.len();
                        if count == raw.len() {
                            break;
                        }
                        display.push(char::from(raw.as_bytes()[count]));
                        count += 1;
                        to_raw.push(count);
                        to_display[count] = display.len();
                    }
                }
            }
            to_display[count] = display.len();
        }
        Ok(MaskedText {
            display,
            to_raw,
            to_display,
        })
    }
    /// A whole unseparated raw value takes precedence over a formatted paste
    /// when both interpretations are possible. No characters are silently lost.
    pub fn parse_display(&self, value: &str) -> Result<String, MaskRejection> {
        self.fragment(0, value).map(|v| v.raw)
    }
    fn fragment(&self, start: usize, value: &str) -> Result<Fragment, MaskRejection> {
        if value.len() > MAX_MASK_INPUT_BYTES {
            return Err(MaskRejection::InputTooLong);
        }
        if self.validate_fragment(start, value).is_ok() {
            return Ok(Fragment {
                raw: value.into(),
                map: (0..=value.len()).collect(),
            });
        }
        if !value.is_ascii() {
            return Err(MaskRejection::InvalidCharacter);
        }
        let start_part = if start == 0 {
            0
        } else {
            self.parts
                .iter()
                .enumerate()
                .filter(|(_, p)| matches!(p, Part::Slot(_)))
                .nth(start)
                .map(|(i, _)| i)
                .unwrap_or(self.parts.len())
        };
        let mut raw = String::new();
        let mut map = vec![0];
        let mut position = 0;
        for part in &self.parts[start_part..] {
            if position == value.len() {
                break;
            }
            match *part {
                Part::Literal(byte) => {
                    if value.as_bytes()[position] == byte {
                        position += 1;
                        map.push(raw.len());
                    }
                }
                Part::Slot(slot) => {
                    let byte = value.as_bytes()[position];
                    if !slot.accepts(byte) {
                        return Err(MaskRejection::InvalidCharacter);
                    }
                    raw.push(char::from(byte));
                    position += 1;
                    map.push(raw.len());
                }
            }
        }
        if position != value.len() {
            return Err(MaskRejection::InputTooLong);
        }
        self.validate_fragment(start, &raw)?;
        Ok(Fragment { raw, map })
    }
}
struct Fragment {
    raw: String,
    map: Vec<usize>,
}
/// Index maps are ASCII byte indices and therefore exact UTF-16 indices too.
/// Debug deliberately omits the displayed business value.
#[derive(Clone, PartialEq, Eq)]
pub struct MaskedText {
    display: String,
    to_raw: Vec<usize>,
    to_display: Vec<usize>,
}
impl fmt::Debug for MaskedText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MaskedText")
            .field("display_length", &self.display.len())
            .field("raw_length", &(self.to_display.len() - 1))
            .finish_non_exhaustive()
    }
}
impl MaskedText {
    pub fn display(&self) -> &str {
        &self.display
    }
    pub fn raw_index(&self, display_index: usize) -> Option<usize> {
        self.to_raw.get(display_index).copied()
    }
    pub fn display_index(&self, raw_index: usize) -> Option<usize> {
        self.to_display.get(raw_index).copied()
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Direction {
    #[default]
    None,
    Forward,
    Backward,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Selection {
    pub start: usize,
    pub end: usize,
    pub direction: Direction,
}
impl Selection {
    fn validate(self, len: usize) -> Result<(), MaskRejection> {
        if self.start <= self.end && self.end <= len {
            Ok(())
        } else {
            Err(MaskRejection::InvalidSelection)
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Intent {
    Insert,
    Backward,
    Forward,
    Replace,
}
pub(crate) struct Proposal {
    pub raw: String,
    pub formatted: MaskedText,
    pub selection: Selection,
}
/// Current physical editor only. It is replaced at reconciliation, never a
/// history of visited values. A transient removed literal may map to no raw slot.
pub(crate) struct Draft {
    raw: String,
    text: String,
    map: Vec<usize>,
}
impl Draft {
    pub fn new(pattern: &MaskPattern, raw: &str) -> Result<Self, MaskRejection> {
        let text = pattern.format(raw)?;
        Ok(Self {
            raw: raw.into(),
            text: text.display,
            map: text.to_raw,
        })
    }
    pub fn raw(&self) -> &str {
        &self.raw
    }
    pub fn apply(
        &mut self,
        pattern: &MaskPattern,
        value: &str,
        after: Selection,
        before: Option<Selection>,
        intent: Intent,
    ) -> Result<Proposal, MaskRejection> {
        if value.len() > MAX_MASK_INPUT_BYTES {
            return Err(MaskRejection::InputTooLong);
        }
        if !value.is_ascii() {
            return Err(MaskRejection::InvalidCharacter);
        }
        after.validate(value.len())?;
        let Some(before) = before else {
            return self.whole(pattern, value, after);
        };
        before.validate(self.text.len())?;
        let (mut start, mut end) = (before.start, before.end);
        if start == end {
            match intent {
                Intent::Backward if start > 0 => start -= 1,
                Intent::Forward if end < self.text.len() => end += 1,
                _ => {}
            }
        }
        let prefix = &self.text[..start];
        let suffix = &self.text[end..];
        if value.len() < prefix.len() + suffix.len()
            || !value.starts_with(prefix)
            || !value.ends_with(suffix)
        {
            return self.whole(pattern, value, after);
        }
        let inserted = &value[start..value.len() - suffix.len()];
        let (mut rs, mut re) = (self.map[start], self.map[end]);
        if rs == re && inserted.is_empty() {
            match intent {
                Intent::Backward if rs > 0 => rs -= 1,
                Intent::Forward if re < self.raw.len() => re += 1,
                _ => {}
            }
        }
        let fragment = pattern.fragment(rs, inserted)?;
        let new_len = self.raw.len() - re + rs + fragment.raw.len();
        if new_len > pattern.capacity() {
            return Err(MaskRejection::InputTooLong);
        }
        let mut raw = self.raw[..rs].to_owned();
        raw.push_str(&fragment.raw);
        raw.push_str(&self.raw[re..]);
        pattern.validate_raw(&raw)?;
        let transform = |index: usize| {
            if index <= rs {
                index
            } else if index < re {
                rs
            } else {
                index - re + rs + fragment.raw.len()
            }
        };
        let mut map = self.map[..=start]
            .iter()
            .copied()
            .map(transform)
            .collect::<Vec<_>>();
        map.extend(fragment.map.iter().skip(1).map(|i| rs + i));
        map.extend(self.map[end + 1..].iter().copied().map(transform));
        if map.len() != value.len() + 1 || map.iter().any(|i| *i > raw.len()) {
            return Err(MaskRejection::InvalidSelection);
        }
        let formatted = pattern.format(&raw)?;
        let selection = Selection {
            start: formatted.display_index(map[after.start]).unwrap(),
            end: formatted.display_index(map[after.end]).unwrap(),
            direction: after.direction,
        };
        self.raw = raw.clone();
        self.text = value.into();
        self.map = map;
        Ok(Proposal {
            raw,
            formatted,
            selection,
        })
    }
    fn whole(
        &mut self,
        pattern: &MaskPattern,
        value: &str,
        selection: Selection,
    ) -> Result<Proposal, MaskRejection> {
        let fragment = pattern.fragment(0, value)?;
        let formatted = pattern.format(&fragment.raw)?;
        let selection = Selection {
            start: formatted
                .display_index(fragment.map[selection.start])
                .unwrap(),
            end: formatted
                .display_index(fragment.map[selection.end])
                .unwrap(),
            direction: selection.direction,
        };
        self.raw = fragment.raw.clone();
        self.text = value.into();
        self.map = fragment.map;
        Ok(Proposal {
            raw: fragment.raw,
            formatted,
            selection,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn s(start: usize, end: usize) -> Selection {
        Selection {
            start,
            end,
            ..Default::default()
        }
    }
    #[test]
    fn finite_pattern_validation_precedes_allocation_and_invalid_values_are_static() {
        for p in ["", "no slots", "#\n#", "#\\", "中#"] {
            assert!(MaskPattern::parse(p).is_err())
        }
        assert!(MaskPattern::parse(&"#".repeat(65)).is_err());
        assert!(MaskPattern::parse(&"#".repeat(1_000_000)).is_err());
        let p = MaskPattern::parse("###").unwrap();
        assert_eq!(
            p.validate_raw(&"8".repeat(1_000_000)),
            Err(MaskRejection::InputTooLong)
        );
        assert!(!MaskRejection::InvalidCharacter.to_string().contains("123"));
    }
    #[test]
    fn formatting_and_exact_bidirectional_boundaries_preserve_partial_raw() {
        let p = MaskPattern::parse("(###) ###").unwrap();
        assert_eq!(p.format("").unwrap().display(), "");
        let text = p.format("123").unwrap();
        assert_eq!(text.display(), "(123) ");
        assert_eq!(text.raw_index(0), Some(0));
        assert_eq!(text.raw_index(1), Some(0));
        assert_eq!(text.raw_index(4), Some(3));
        assert_eq!(text.display_index(0), Some(1));
        assert_eq!(text.display_index(3), Some(6));
        assert_eq!(p.format("123456").unwrap().display(), "(123) 456");
    }
    #[test]
    fn literal_digit_prefix_and_escaped_slot_syntax_have_an_explicit_raw_first_paste_policy() {
        let p = MaskPattern::parse("+1 (###) ###").unwrap();
        assert_eq!(p.parse_display("123456").unwrap(), "123456");
        assert_eq!(p.parse_display("+1 (123) 456").unwrap(), "123456");
        assert_eq!(
            MaskPattern::parse("\\A##")
                .unwrap()
                .format("12")
                .unwrap()
                .display(),
            "A12"
        );
    }
    #[test]
    fn ascii_slots_preserve_case_and_never_discard_a_bad_paste_suffix() {
        let p = MaskPattern::parse("AA-##-**").unwrap();
        assert_eq!(p.parse_display("aZ-09-x1").unwrap(), "aZ09x1");
        for value in ["a1-09-x1", "aZ09x!", "aZ09x12", "aZ０9x1"] {
            assert!(p.parse_display(value).is_err())
        }
    }
    #[test]
    fn ordinary_insert_and_boundary_insert_only_reformat_when_a_literal_is_due() {
        let p = MaskPattern::parse("##-##").unwrap();
        let mut d = Draft::new(&p, "").unwrap();
        let a = d
            .apply(&p, "1", s(1, 1), Some(s(0, 0)), Intent::Insert)
            .unwrap();
        assert_eq!(a.formatted.display(), "1");
        let b = d
            .apply(&p, "12", s(2, 2), Some(s(1, 1)), Intent::Insert)
            .unwrap();
        assert_eq!(b.formatted.display(), "12-");
        assert_eq!(b.selection, s(3, 3));
        let c = d
            .apply(&p, "123", s(3, 3), Some(s(2, 2)), Intent::Insert)
            .unwrap();
        assert_eq!(c.raw, "123");
        assert_eq!(c.formatted.display(), "12-3");
        assert_eq!(c.selection, s(4, 4));
    }
    #[test]
    fn backspace_at_a_literal_removes_the_previous_raw_slot_and_same_tick_edit_uses_a_bounded_current_map()
     {
        let p = MaskPattern::parse("##-##").unwrap();
        let mut d = Draft::new(&p, "12").unwrap();
        let a = d
            .apply(&p, "12", s(2, 2), Some(s(3, 3)), Intent::Backward)
            .unwrap();
        assert_eq!(a.raw, "1");
        assert_eq!(a.selection, s(1, 1));
        let b = d
            .apply(&p, "1", s(1, 1), Some(s(2, 2)), Intent::Backward)
            .unwrap();
        assert_eq!(b.raw, "");
        assert_eq!(d.raw(), "");
    }
    #[test]
    fn selected_formatted_range_replaces_raw_once_and_retains_backward_direction() {
        let p = MaskPattern::parse("##-##").unwrap();
        let mut d = Draft::new(&p, "1234").unwrap();
        let a = d
            .apply(
                &p,
                "1954",
                Selection {
                    direction: Direction::Backward,
                    ..s(1, 3)
                },
                Some(s(1, 4)),
                Intent::Insert,
            )
            .unwrap();
        assert_eq!(a.raw, "1954");
        assert_eq!(a.formatted.display(), "19-54");
        assert_eq!(a.selection.direction, Direction::Backward);
        assert_eq!(a.selection.start, 1);
        assert_eq!(a.selection.end, 4);
    }
    #[test]
    fn forward_delete_at_a_literal_removes_the_following_raw_slot() {
        let p = MaskPattern::parse("##-##").unwrap();
        let mut d = Draft::new(&p, "1234").unwrap();
        let a = d
            .apply(&p, "1234", s(2, 2), Some(s(2, 2)), Intent::Forward)
            .unwrap();
        assert_eq!(a.raw, "124");
        assert_eq!(a.formatted.display(), "12-4");
    }
    #[test]
    fn whole_paste_and_rejection_are_atomic_and_do_not_mutate_the_current_draft() {
        let p = MaskPattern::parse("(###) ###").unwrap();
        let mut d = Draft::new(&p, "").unwrap();
        let a = d
            .apply(&p, "(123) 456", s(9, 9), None, Intent::Replace)
            .unwrap();
        assert_eq!(a.raw, "123456");
        let before = d.raw().to_owned();
        assert!(
            d.apply(&p, "1234567", s(7, 7), None, Intent::Replace)
                .is_err()
        );
        assert_eq!(d.raw(), before);
        assert!(d.apply(&p, "123", s(4, 4), None, Intent::Replace).is_err());
    }
}
