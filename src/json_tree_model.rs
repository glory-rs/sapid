//! Bounded current JSON structure. Raw numeric spelling and duplicate-key checks
//! precede any UI work; no floating-point business-value conversion occurs.
use crate::{JsonKind, JsonLimits, JsonValidity, validate_json};
use std::{collections::HashSet, ops::Range};
pub const MAX_JSON_VIEW_NODES: usize = 2048;
pub const MAX_JSON_VIEW_DEPTH: usize = 64;
pub const MAX_JSON_VIEW_KEY_BYTES: usize = 256;
pub const MAX_JSON_VIEW_PATH_BYTES: usize = 2048;
pub const MAX_JSON_SEARCH_BYTES: usize = 256;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JsonViewLimits {
    pub bytes: usize,
    pub nodes: usize,
    pub depth: usize,
}
impl Default for JsonViewLimits {
    fn default() -> Self {
        Self {
            bytes: crate::MAX_JSON_BYTES,
            nodes: MAX_JSON_VIEW_NODES,
            depth: 32,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonViewError {
    Configuration,
    Syntax(crate::JsonError),
    Empty,
    Nodes,
    Depth,
    Key,
    Path,
    DuplicateKey,
    Search,
}
impl std::fmt::Display for JsonViewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax(error) => error.fmt(f),
            other => f.write_str(match other {
                Self::Configuration => "JSON viewer limits exceed their finite domain",
                Self::Empty => "JSON viewer requires a JSON value",
                Self::Nodes => "JSON viewer exceeds its current node budget",
                Self::Depth => "JSON viewer exceeds its nesting budget",
                Self::Key => "JSON object key exceeds 256 UTF-8 bytes",
                Self::Path => "JSON pointer exceeds 2048 UTF-8 bytes",
                Self::DuplicateKey => "JSON viewer rejects duplicate decoded object keys",
                Self::Search => "JSON search exceeds 256 UTF-8 bytes",
                Self::Syntax(_) => unreachable!(),
            }),
        }
    }
}
impl std::error::Error for JsonViewError {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonViewNode {
    pub path: String,
    pub label: String,
    pub kind: JsonKind,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub depth: usize,
    pub range: Range<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonTree {
    source: String,
    nodes: Vec<JsonViewNode>,
}
impl JsonTree {
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn nodes(&self) -> &[JsonViewNode] {
        &self.nodes
    }
    /// Exact case-sensitive key/raw-value substring search. Return current
    /// preorder indices, never a visited source or an implicit recursive scan.
    pub fn search(&self, query: &str) -> Result<Vec<usize>, JsonViewError> {
        if query.len() > MAX_JSON_SEARCH_BYTES {
            return Err(JsonViewError::Search);
        }
        if query.is_empty() {
            return Ok(Vec::new());
        }
        Ok(self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                (node.label.contains(query)
                    || node.children.is_empty() && self.source[node.range.clone()].contains(query))
                .then_some(index)
            })
            .collect())
    }
    pub fn value(&self, index: usize) -> Option<&str> {
        self.nodes
            .get(index)
            .map(|node| &self.source[node.range.clone()])
    }
}
impl JsonViewLimits {
    pub fn validate(self) -> Result<(), JsonViewError> {
        if !(1..=crate::MAX_JSON_BYTES).contains(&self.bytes)
            || !(1..=MAX_JSON_VIEW_NODES).contains(&self.nodes)
            || !(1..=MAX_JSON_VIEW_DEPTH).contains(&self.depth)
        {
            Err(JsonViewError::Configuration)
        } else {
            Ok(())
        }
    }
}
struct Parser<'a> {
    source: &'a str,
    cursor: usize,
    nodes: Vec<JsonViewNode>,
    limits: JsonViewLimits,
}
impl Parser<'_> {
    fn spaces(&mut self) {
        while self
            .source
            .as_bytes()
            .get(self.cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.cursor += 1;
        }
    }
    fn string_end(&mut self) -> usize {
        self.cursor += 1;
        while self.cursor < self.source.len() {
            match self.source.as_bytes()[self.cursor] {
                b'\\' => self.cursor += 2,
                b'"' => {
                    self.cursor += 1;
                    break;
                }
                _ => self.cursor += 1,
            }
        }
        self.cursor
    }
    fn value(
        &mut self,
        parent: Option<usize>,
        path: String,
        label: String,
        depth: usize,
    ) -> Result<usize, JsonViewError> {
        if self.nodes.len() == self.limits.nodes {
            return Err(JsonViewError::Nodes);
        }
        if depth > self.limits.depth {
            return Err(JsonViewError::Depth);
        }
        if path.len() > MAX_JSON_VIEW_PATH_BYTES {
            return Err(JsonViewError::Path);
        }
        self.spaces();
        let start = self.cursor;
        let b = self.source.as_bytes()[start];
        let kind = match b {
            b'{' => JsonKind::Object,
            b'[' => JsonKind::Array,
            b'"' => JsonKind::String,
            b't' | b'f' => JsonKind::Boolean,
            b'n' => JsonKind::Null,
            _ => JsonKind::Number,
        };
        let index = self.nodes.len();
        self.nodes.push(JsonViewNode {
            path,
            label,
            kind,
            parent,
            children: Vec::new(),
            depth,
            range: start..start,
        });
        if matches!(kind, JsonKind::Object | JsonKind::Array) {
            self.cursor += 1;
            self.spaces();
            let close = if b == b'{' { b'}' } else { b']' };
            let mut keys = HashSet::new();
            while self.source.as_bytes()[self.cursor] != close {
                let (segment, label) = if kind == JsonKind::Object {
                    let beginning = self.cursor;
                    let end = self.string_end();
                    let key: String = serde_json::from_str(&self.source[beginning..end])
                        .expect("syntax was validated");
                    if key.len() > MAX_JSON_VIEW_KEY_BYTES {
                        return Err(JsonViewError::Key);
                    }
                    if !keys.insert(key.clone()) {
                        return Err(JsonViewError::DuplicateKey);
                    }
                    self.spaces();
                    self.cursor += 1; // validated colon
                    (key.replace('~', "~0").replace('/', "~1"), key)
                } else {
                    let n = self.nodes[index].children.len().to_string();
                    (n.clone(), n)
                };
                let parent_path = &self.nodes[index].path;
                if parent_path.len() + 1 + segment.len() > MAX_JSON_VIEW_PATH_BYTES {
                    return Err(JsonViewError::Path);
                }
                let next_path = format!("{parent_path}/{segment}");
                let child = self.value(Some(index), next_path, label, depth + 1)?;
                self.nodes[index].children.push(child);
                self.spaces();
                if self.source.as_bytes()[self.cursor] == b',' {
                    self.cursor += 1;
                    self.spaces();
                } else {
                    break;
                }
            }
            self.cursor += 1;
        } else if b == b'"' {
            self.string_end();
        } else {
            while self
                .source
                .as_bytes()
                .get(self.cursor)
                .is_some_and(|b| !b.is_ascii_whitespace() && !matches!(b, b',' | b'}' | b']'))
            {
                self.cursor += 1;
            }
        }
        self.nodes[index].range.end = self.cursor;
        Ok(index)
    }
}
pub fn parse_json_tree(source: &str, limits: JsonViewLimits) -> Result<JsonTree, JsonViewError> {
    limits.validate()?;
    match validate_json(
        source,
        JsonLimits {
            bytes: limits.bytes,
            depth: limits.depth,
        },
    ) {
        JsonValidity::Empty => return Err(JsonViewError::Empty),
        JsonValidity::Invalid(error) => return Err(JsonViewError::Syntax(error)),
        JsonValidity::Valid(_) => {}
    }
    let mut parser = Parser {
        source,
        cursor: 0,
        nodes: Vec::new(),
        limits,
    };
    parser.value(None, String::new(), "$".into(), 0)?;
    Ok(JsonTree {
        source: source.into(),
        nodes: parser.nodes,
    })
}
