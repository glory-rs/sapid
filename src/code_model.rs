//! Bounded lexical highlighting. Tokens are UTF-8 source ranges, never markup.
use std::ops::Range;

pub const MAX_CODE_BYTES: usize = 65_536;
pub const MAX_CODE_TOKENS: usize = 8192;
pub const MAX_CODE_LINES: usize = 2048;
pub const MAX_CODE_COMMENT_DEPTH: usize = 32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CodeLanguage {
    #[default]
    Plain,
    Rust,
    Json,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeTokenKind {
    Plain,
    Whitespace,
    Keyword,
    Identifier,
    Number,
    String,
    Comment,
    Punctuation,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeToken {
    pub kind: CodeTokenKind,
    pub range: Range<usize>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeError {
    Bytes,
    Tokens,
    Lines,
    CommentDepth,
    RawStringHashes,
}
impl std::fmt::Display for CodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Bytes => "code exceeds 65536 UTF-8 bytes",
            Self::Tokens => "code exceeds 8192 lexical tokens",
            Self::Lines => "code exceeds 2048 lines",
            Self::CommentDepth => "Rust block comments exceed 32 nesting levels",
            Self::RawStringHashes => "Rust raw strings exceed 16 hash delimiters",
        })
    }
}
impl std::error::Error for CodeError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeDocument {
    source: String,
    tokens: Vec<CodeToken>,
    line_starts: Vec<usize>,
}
impl CodeDocument {
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn tokens(&self) -> &[CodeToken] {
        &self.tokens
    }
    pub fn line_starts(&self) -> &[usize] {
        &self.line_starts
    }
}
fn keyword(text: &str, language: CodeLanguage) -> bool {
    match language {
        CodeLanguage::Plain => false,
        CodeLanguage::Json => matches!(text, "true" | "false" | "null"),
        CodeLanguage::Rust => matches!(
            text,
            "as" | "async"
                | "await"
                | "break"
                | "const"
                | "continue"
                | "crate"
                | "dyn"
                | "else"
                | "enum"
                | "extern"
                | "false"
                | "fn"
                | "for"
                | "if"
                | "impl"
                | "in"
                | "let"
                | "loop"
                | "match"
                | "mod"
                | "move"
                | "mut"
                | "pub"
                | "ref"
                | "return"
                | "self"
                | "Self"
                | "static"
                | "struct"
                | "super"
                | "trait"
                | "true"
                | "type"
                | "unsafe"
                | "use"
                | "where"
                | "while"
                | "yield"
                | "try"
                | "union"
        ),
    }
}
fn quoted(source: &str, start: usize, quote: u8) -> usize {
    let b = source.as_bytes();
    let mut i = start + 1;
    while i < b.len() {
        if b[i] == b'\\' {
            i += 1;
            if i < b.len() {
                i += source[i..].chars().next().unwrap().len_utf8();
            }
        } else if b[i] == quote {
            return i + 1;
        } else {
            i += source[i..].chars().next().unwrap().len_utf8();
        }
    }
    i
}
// A lifetime starts with an apostrophe too. Only a single scalar or escape with
// a closing apostrophe is a character literal; malformed input stays ordinary.
fn character_end(source: &str, start: usize) -> Option<usize> {
    let rest = source.get(start + 1..)?;
    let first = rest.chars().next()?;
    let end = if first == '\\' {
        let escaped = rest.get(1..)?.chars().next()?;
        if escaped == 'u' {
            let tail = rest.get(2..)?;
            if !tail.starts_with('{') {
                return None;
            }
            start + 3 + tail.find('}')? + 1
        } else if escaped == 'x' {
            start + 5
        } else {
            start + 2 + escaped.len_utf8()
        }
    } else if first == '\n' || first == '\r' || first == '\'' {
        return None;
    } else {
        start + 1 + first.len_utf8()
    };
    (source.as_bytes().get(end) == Some(&b'\'')).then_some(end + 1)
}
/// Lex a finite source without parsing/evaluating it. Incomplete strings and
/// comments extend to EOF so an editable draft remains visible and lossless.
pub fn tokenize_code(source: &str, language: CodeLanguage) -> Result<CodeDocument, CodeError> {
    if source.len() > MAX_CODE_BYTES {
        return Err(CodeError::Bytes);
    }
    let mut line_starts = vec![0];
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' {
            if line_starts.len() == MAX_CODE_LINES {
                return Err(CodeError::Lines);
            }
            line_starts.push(i + 1);
        }
    }
    let mut tokens = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < source.len() {
        if tokens.len() == MAX_CODE_TOKENS {
            return Err(CodeError::Tokens);
        }
        let start = i;
        let c = source[i..].chars().next().unwrap();
        let kind;
        if language == CodeLanguage::Plain {
            i = source.len();
            kind = CodeTokenKind::Plain;
        } else if c.is_whitespace() {
            i += c.len_utf8();
            while i < source.len() && source[i..].chars().next().unwrap().is_whitespace() {
                i += source[i..].chars().next().unwrap().len_utf8();
            }
            kind = CodeTokenKind::Whitespace;
        } else if language == CodeLanguage::Rust && source[i..].starts_with("//") {
            i = source[i..].find('\n').map_or(source.len(), |n| i + n);
            kind = CodeTokenKind::Comment;
        } else if language == CodeLanguage::Rust && source[i..].starts_with("/*") {
            i += 2;
            let mut depth = 1;
            while i < source.len() && depth != 0 {
                if source[i..].starts_with("/*") {
                    depth += 1;
                    if depth > MAX_CODE_COMMENT_DEPTH {
                        return Err(CodeError::CommentDepth);
                    }
                    i += 2;
                } else if source[i..].starts_with("*/") {
                    depth -= 1;
                    i += 2;
                } else {
                    i += source[i..].chars().next().unwrap().len_utf8();
                }
            }
            kind = CodeTokenKind::Comment;
        } else {
            let raw_start = if language == CodeLanguage::Rust && c == 'r' {
                Some(i + 1)
            } else if language == CodeLanguage::Rust && source[i..].starts_with("br") {
                Some(i + 2)
            } else {
                None
            };
            let raw = raw_start.and_then(|mut cursor| {
                let beginning = cursor;
                while bytes.get(cursor) == Some(&b'#') {
                    cursor += 1;
                }
                (bytes.get(cursor) == Some(&b'"')).then_some((cursor, cursor - beginning))
            });
            if let Some((quote, hashes)) = raw {
                if hashes > 16 {
                    return Err(CodeError::RawStringHashes);
                }
                i = quote + 1;
                while i < source.len() {
                    if bytes[i] == b'"'
                        && bytes
                            .get(i + 1..i + 1 + hashes)
                            .is_some_and(|tail| tail.iter().all(|b| *b == b'#'))
                    {
                        i += 1 + hashes;
                        break;
                    }
                    i += source[i..].chars().next().unwrap().len_utf8();
                }
                kind = CodeTokenKind::String;
            } else if language == CodeLanguage::Rust
                && (source[i..].starts_with("b\"") || source[i..].starts_with("c\""))
            {
                i = quoted(source, i + 1, b'"');
                kind = CodeTokenKind::String;
            } else if c == '"' {
                i = quoted(source, i, b'"');
                kind = CodeTokenKind::String;
            } else if language == CodeLanguage::Rust
                && c == '\''
                && character_end(source, i).is_some()
            {
                i = character_end(source, i).unwrap();
                kind = CodeTokenKind::String;
            } else if c.is_ascii_digit()
                || (language == CodeLanguage::Json
                    && c == '-'
                    && bytes.get(i + 1).is_some_and(u8::is_ascii_digit))
            {
                i += 1;
                while i < bytes.len() {
                    let b = bytes[i];
                    if b.is_ascii_alphanumeric()
                        || b == b'_'
                        || b == b'.' && bytes.get(i + 1).is_some_and(u8::is_ascii_digit)
                        || matches!(b, b'+' | b'-') && matches!(bytes[i - 1], b'e' | b'E')
                    {
                        i += 1;
                    } else {
                        break;
                    }
                }
                kind = CodeTokenKind::Number;
            } else if language == CodeLanguage::Rust
                && source[i..].starts_with("r#")
                && source
                    .get(i + 2..)
                    .and_then(|s| s.chars().next())
                    .is_some_and(|c| c.is_alphabetic() || c == '_')
            {
                i += 2;
                while i < source.len() {
                    let next = source[i..].chars().next().unwrap();
                    if !next.is_alphanumeric() && next != '_' {
                        break;
                    }
                    i += next.len_utf8();
                }
                kind = CodeTokenKind::Identifier;
            } else if c.is_alphabetic() || c == '_' {
                i += c.len_utf8();
                while i < source.len() {
                    let next = source[i..].chars().next().unwrap();
                    if !next.is_alphanumeric() && next != '_' {
                        break;
                    }
                    i += next.len_utf8();
                }
                kind = if keyword(&source[start..i], language) {
                    CodeTokenKind::Keyword
                } else {
                    CodeTokenKind::Identifier
                };
            } else {
                i += c.len_utf8();
                kind = CodeTokenKind::Punctuation;
            }
        }
        tokens.push(CodeToken {
            kind,
            range: start..i,
        });
    }
    Ok(CodeDocument {
        source: source.into(),
        tokens,
        line_starts,
    })
}
