//! Optional JSON validation/formatting without decoding business numbers or keys.
pub const MAX_JSON_BYTES: usize = 65_536;
pub const MAX_JSON_DEPTH: usize = 128;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JsonFormat {
    #[default]
    PrettyTwo,
    PrettyFour,
    Compact,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonKind {
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonErrorKind {
    Syntax,
    InputBudget,
    OutputBudget,
    DepthBudget,
    Configuration,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonError {
    pub kind: JsonErrorKind,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
}
impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for JsonError {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonValidity {
    Empty,
    Valid(JsonKind),
    Invalid(JsonError),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JsonLimits {
    pub bytes: usize,
    pub depth: usize,
}
impl Default for JsonLimits {
    fn default() -> Self {
        Self {
            bytes: MAX_JSON_BYTES,
            depth: 64,
        }
    }
}
fn error(kind: JsonErrorKind, message: &str) -> JsonError {
    JsonError {
        kind,
        line: None,
        column: None,
        message: message.into(),
    }
}
impl JsonLimits {
    pub fn validate(self) -> Result<(), JsonError> {
        if !(1..=MAX_JSON_BYTES).contains(&self.bytes)
            || !(1..=MAX_JSON_DEPTH).contains(&self.depth)
        {
            Err(error(
                JsonErrorKind::Configuration,
                "JSON limits require 1..=65536 bytes and 1..=128 nesting depth",
            ))
        } else {
            Ok(())
        }
    }
}
pub fn validate_json(value: &str, limits: JsonLimits) -> JsonValidity {
    let validate = (|| -> Result<Option<JsonKind>, JsonError> {
        limits.validate()?;
        if value.len() > limits.bytes {
            return Err(error(
                JsonErrorKind::InputBudget,
                "JSON draft exceeds its byte budget",
            ));
        }
        let trimmed = value.trim_matches([' ', '\t', '\r', '\n']);
        if trimmed.is_empty() {
            return Ok(None);
        }
        let mut quoted = false;
        let mut escaped = false;
        let mut depth = 0usize;
        for byte in trimmed.bytes() {
            if quoted {
                if escaped {
                    escaped = false
                } else if byte == b'\\' {
                    escaped = true
                } else if byte == b'"' {
                    quoted = false
                }
            } else {
                match byte {
                    b'"' => quoted = true,
                    b'{' | b'[' => {
                        depth += 1;
                        if depth > limits.depth {
                            return Err(error(
                                JsonErrorKind::DepthBudget,
                                "JSON draft exceeds its nesting budget",
                            ));
                        }
                    }
                    b'}' | b']' => depth = depth.saturating_sub(1),
                    _ => {}
                }
            }
        }
        // Budget the nesting before invoking the parser. No business tree is decoded.
        let _: &serde_json::value::RawValue =
            serde_json::from_str(trimmed).map_err(|failure| {
                let failure: serde_json::Error = failure;
                JsonError {
                    kind: JsonErrorKind::Syntax,
                    line: Some(failure.line()),
                    column: Some(failure.column()),
                    message: failure.to_string().chars().take(256).collect(),
                }
            })?;
        Ok(Some(match trimmed.as_bytes()[0] {
            b'n' => JsonKind::Null,
            b't' | b'f' => JsonKind::Boolean,
            b'"' => JsonKind::String,
            b'[' => JsonKind::Array,
            b'{' => JsonKind::Object,
            _ => JsonKind::Number,
        }))
    })();
    match validate {
        Ok(None) => JsonValidity::Empty,
        Ok(Some(kind)) => JsonValidity::Valid(kind),
        Err(error) => JsonValidity::Invalid(error),
    }
}
pub fn format_json(
    value: &str,
    format: JsonFormat,
    limits: JsonLimits,
) -> Result<String, JsonError> {
    match validate_json(value, limits) {
        JsonValidity::Invalid(error) => return Err(error),
        JsonValidity::Empty => return Ok(String::new()),
        JsonValidity::Valid(_) => {}
    }
    let bytes = value.as_bytes();
    let indent = match format {
        JsonFormat::PrettyTwo => 2,
        JsonFormat::PrettyFour => 4,
        JsonFormat::Compact => 0,
    };
    let mut output = Vec::with_capacity(value.len().min(limits.bytes));
    let mut quoted = false;
    let mut escaped = false;
    let mut level = 0usize;
    let mut previous = None;
    fn push(output: &mut Vec<u8>, bytes: &[u8], limit: usize) -> Result<(), JsonError> {
        if bytes.len() > limit.saturating_sub(output.len()) {
            return Err(error(
                JsonErrorKind::OutputBudget,
                "formatted JSON exceeds its byte budget",
            ));
        }
        output.extend_from_slice(bytes);
        Ok(())
    }
    fn newline(
        output: &mut Vec<u8>,
        level: usize,
        indent: usize,
        limit: usize,
    ) -> Result<(), JsonError> {
        push(output, b"\n", limit)?;
        for _ in 0..level * indent {
            push(output, b" ", limit)?
        }
        Ok(())
    }
    for (index, byte) in bytes.iter().copied().enumerate() {
        if quoted {
            push(&mut output, &[byte], limits.bytes)?;
            if escaped {
                escaped = false
            } else if byte == b'\\' {
                escaped = true
            } else if byte == b'"' {
                quoted = false
            }
            continue;
        }
        if byte.is_ascii_whitespace() {
            continue;
        }
        match byte {
            b'"' => {
                quoted = true;
                push(&mut output, &[byte], limits.bytes)?
            }
            b'{' | b'[' => {
                push(&mut output, &[byte], limits.bytes)?;
                level += 1;
                let next = bytes[index + 1..]
                    .iter()
                    .copied()
                    .find(|value| !value.is_ascii_whitespace());
                if indent > 0 && !matches!(next, Some(b'}' | b']')) {
                    newline(&mut output, level, indent, limits.bytes)?
                }
            }
            b'}' | b']' => {
                level -= 1;
                if indent > 0 && !matches!(previous, Some(b'{' | b'[')) {
                    newline(&mut output, level, indent, limits.bytes)?
                }
                push(&mut output, &[byte], limits.bytes)?
            }
            b',' => {
                push(&mut output, b",", limits.bytes)?;
                if indent > 0 {
                    newline(&mut output, level, indent, limits.bytes)?
                }
            }
            b':' => push(
                &mut output,
                if indent > 0 { b": " } else { b":" },
                limits.bytes,
            )?,
            _ => push(&mut output, &[byte], limits.bytes)?,
        }
        previous = Some(byte);
    }
    // Input was UTF-8; whitespace edits never split or modify quoted/non-ASCII bytes.
    Ok(String::from_utf8(output).expect("JSON whitespace formatting retains UTF-8"))
}
