//! Finite application-supplied trace records. Payloads never execute anything.
use std::collections::HashMap;
pub const MAX_TRACE_RECORDS: usize = 256;
pub const MAX_TRACE_DEPTH: usize = 16;
pub const MAX_TRACE_BYTES: usize = 65_536;
pub const MAX_TRACE_PAYLOAD_BYTES: usize = 8_192;
pub const MAX_STACK_FRAMES: usize = 128;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceError {
    InvalidKey,
    InvalidText,
    TooManyRecords,
    TooManyBytes,
    DuplicateKey,
    MissingParent,
    Cycle,
    TooDeep,
    InvalidDuration,
    TooManyFrames,
}
impl std::fmt::Display for TraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for TraceError {}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraceKey(String);
impl TraceKey {
    pub fn new(value: &str) -> Result<Self, TraceError> {
        if value.is_empty()
            || value.len() > 64
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-.:".contains(&b))
        {
            return Err(TraceError::InvalidKey);
        }
        Ok(Self(value.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TracePayload {
    Public(TraceText),
    /// No original secret is stored. Applications must classify/redact before
    /// constructing the record; the widget cannot discover secrets in text.
    Redacted,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceText {
    text: String,
    truncated: bool,
}
impl TraceText {
    pub fn new(value: &str) -> Result<Self, TraceError> {
        if !valid_text(value, MAX_TRACE_PAYLOAD_BYTES, true) {
            return Err(TraceError::InvalidText);
        }
        Ok(Self {
            text: value.to_owned(),
            truncated: false,
        })
    }
    /// Explicit UTF-8 codepoint-safe prefix policy. It does not claim grapheme
    /// segmentation and never duplicates the discarded source suffix.
    pub fn prefix(value: &str, budget: usize) -> Result<Self, TraceError> {
        if budget > MAX_TRACE_PAYLOAD_BYTES {
            return Err(TraceError::TooManyBytes);
        }
        let mut end = budget.min(value.len());
        while !value.is_char_boundary(end) {
            end -= 1
        }
        let prefix = &value[..end];
        if !valid_text(prefix, MAX_TRACE_PAYLOAD_BYTES, true) {
            return Err(TraceError::InvalidText);
        }
        Ok(Self {
            text: prefix.to_owned(),
            truncated: end < value.len(),
        })
    }
    pub fn as_str(&self) -> &str {
        &self.text
    }
    pub const fn is_truncated(&self) -> bool {
        self.truncated
    }
}
fn valid_text(value: &str, max: usize, empty: bool) -> bool {
    value.len() <= max
        && !value
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
        && (empty || !value.trim().is_empty())
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TraceStatus {
    #[default]
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}
impl TraceStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Running => "Running",
            Self::Succeeded => "Succeeded",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TraceKind {
    #[default]
    Span,
    Tool,
    Subagent,
}
impl TraceKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Span => "Span",
            Self::Tool => "Tool",
            Self::Subagent => "Subagent",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceRecord {
    key: TraceKey,
    parent: Option<TraceKey>,
    kind: TraceKind,
    title: String,
    status: TraceStatus,
    elapsed_millis: Option<u64>,
    input: Option<TracePayload>,
    output: Option<TracePayload>,
}
impl TraceRecord {
    pub fn new(key: TraceKey, kind: TraceKind, title: &str) -> Result<Self, TraceError> {
        if !valid_text(title, 256, false) || title.contains('\n') {
            return Err(TraceError::InvalidText);
        }
        Ok(Self {
            key,
            parent: None,
            kind,
            title: title.to_owned(),
            status: TraceStatus::Pending,
            elapsed_millis: None,
            input: None,
            output: None,
        })
    }
    pub fn parent(mut self, key: TraceKey) -> Self {
        self.parent = Some(key);
        self
    }
    pub fn status(mut self, value: TraceStatus) -> Self {
        self.status = value;
        self
    }
    pub fn elapsed_millis(mut self, value: Option<u64>) -> Result<Self, TraceError> {
        if value.is_some_and(|v| v > 604_800_000) {
            return Err(TraceError::InvalidDuration);
        }
        self.elapsed_millis = value;
        Ok(self)
    }
    pub fn input(mut self, value: TracePayload) -> Self {
        self.input = Some(value);
        self
    }
    pub fn output(mut self, value: TracePayload) -> Self {
        self.output = Some(value);
        self
    }
    pub fn key(&self) -> &TraceKey {
        &self.key
    }
    pub fn parent_key(&self) -> Option<&TraceKey> {
        self.parent.as_ref()
    }
    pub const fn kind(&self) -> TraceKind {
        self.kind
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub const fn state(&self) -> TraceStatus {
        self.status
    }
    pub const fn elapsed(&self) -> Option<u64> {
        self.elapsed_millis
    }
    pub fn input_payload(&self) -> Option<&TracePayload> {
        self.input.as_ref()
    }
    pub fn output_payload(&self) -> Option<&TracePayload> {
        self.output.as_ref()
    }
    fn bytes(&self) -> usize {
        self.key.0.len()
            + self.parent.as_ref().map_or(0, |v| v.0.len())
            + self.title.len()
            + [&self.input, &self.output]
                .into_iter()
                .map(|p| match p {
                    Some(TracePayload::Public(v)) => v.text.len(),
                    _ => 0,
                })
                .sum::<usize>()
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TraceData {
    records: Vec<TraceRecord>,
    depths: Vec<u8>,
    lookup: HashMap<TraceKey, usize>,
    roots: Vec<usize>,
    children: Vec<Vec<usize>>,
}
impl TraceData {
    pub fn new(values: impl IntoIterator<Item = TraceRecord>) -> Result<Self, TraceError> {
        let records: Vec<_> = values.into_iter().take(MAX_TRACE_RECORDS + 1).collect();
        if records.len() > MAX_TRACE_RECORDS {
            return Err(TraceError::TooManyRecords);
        }
        if records.iter().map(|r| r.bytes()).sum::<usize>() > MAX_TRACE_BYTES {
            return Err(TraceError::TooManyBytes);
        }
        let mut indices = HashMap::with_capacity(records.len());
        for (index, record) in records.iter().enumerate() {
            if indices.insert(record.key.as_str(), index).is_some() {
                return Err(TraceError::DuplicateKey);
            }
        }
        let mut depths = Vec::with_capacity(records.len());
        for (index, record) in records.iter().enumerate() {
            let mut parent = record.parent.as_ref();
            let mut depth = 0;
            let mut chain = [usize::MAX; MAX_TRACE_DEPTH + 1];
            while let Some(key) = parent {
                let &at = indices.get(key.as_str()).ok_or(TraceError::MissingParent)?;
                if at == index || chain[..depth].contains(&at) {
                    return Err(TraceError::Cycle);
                }
                if depth == MAX_TRACE_DEPTH {
                    return Err(TraceError::TooDeep);
                }
                chain[depth] = at;
                depth += 1;
                parent = records[at].parent.as_ref();
            }
            depths.push(depth as u8);
        }
        let mut roots = vec![];
        let mut children = vec![vec![]; records.len()];
        for (index, record) in records.iter().enumerate() {
            if let Some(parent) = record.parent.as_ref() {
                children[indices[parent.as_str()]].push(index)
            } else {
                roots.push(index)
            }
        }
        let lookup = records
            .iter()
            .enumerate()
            .map(|(index, record)| (record.key.clone(), index))
            .collect();
        Ok(Self {
            records,
            depths,
            lookup,
            roots,
            children,
        })
    }
    pub fn records(&self) -> &[TraceRecord] {
        &self.records
    }
    pub fn depth(&self, index: usize) -> Option<usize> {
        self.depths.get(index).map(|&n| usize::from(n))
    }
    pub fn record(&self, key: &TraceKey) -> Option<&TraceRecord> {
        self.lookup.get(key).map(|&i| &self.records[i])
    }
    pub fn root_keys(&self) -> impl Iterator<Item = &TraceKey> {
        self.roots.iter().map(|&i| self.records[i].key())
    }
    pub fn child_keys(&self, key: &TraceKey) -> impl Iterator<Item = &TraceKey> {
        self.lookup
            .get(key)
            .into_iter()
            .flat_map(|&i| self.children[i].iter())
            .map(|&i| self.records[i].key())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackLocation {
    file: String,
    line: u32,
    column: Option<u32>,
}
impl StackLocation {
    /// A display location, never an automatically generated filesystem URL.
    pub fn new(file: &str, line: u32, column: Option<u32>) -> Result<Self, TraceError> {
        if !valid_text(file, 1024, false)
            || file.contains('\n')
            || line == 0
            || column == Some(0)
        {
            return Err(TraceError::InvalidText);
        }
        Ok(Self {
            file: file.to_owned(),
            line,
            column,
        })
    }
    pub fn file(&self) -> &str {
        &self.file
    }
    pub const fn line(&self) -> u32 {
        self.line
    }
    pub const fn column(&self) -> Option<u32> {
        self.column
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackFrame {
    key: TraceKey,
    function: String,
    location: Option<StackLocation>,
}
impl StackFrame {
    pub fn new(key: TraceKey, function: &str) -> Result<Self, TraceError> {
        if !valid_text(function, 256, false) || function.contains('\n') {
            return Err(TraceError::InvalidText);
        }
        Ok(Self {
            key,
            function: function.to_owned(),
            location: None,
        })
    }
    pub fn location(mut self, value: StackLocation) -> Self {
        self.location = Some(value);
        self
    }
    pub fn key(&self) -> &TraceKey {
        &self.key
    }
    pub fn function(&self) -> &str {
        &self.function
    }
    pub fn source(&self) -> Option<&StackLocation> {
        self.location.as_ref()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackTraceData {
    message: TracePayload,
    frames: Vec<StackFrame>,
    lookup: HashMap<TraceKey, usize>,
}
impl StackTraceData {
    pub fn new(
        message: TracePayload,
        values: impl IntoIterator<Item = StackFrame>,
    ) -> Result<Self, TraceError> {
        let frames: Vec<_> = values.into_iter().take(MAX_STACK_FRAMES + 1).collect();
        if frames.len() > MAX_STACK_FRAMES {
            return Err(TraceError::TooManyFrames);
        }
        let mut keys = HashMap::with_capacity(frames.len());
        let mut bytes = match &message {
            TracePayload::Public(v) => v.as_str().len(),
            TracePayload::Redacted => 0,
        };
        for frame in &frames {
            if keys.insert(frame.key.as_str(), ()).is_some() {
                return Err(TraceError::DuplicateKey);
            }
            bytes += frame.key.as_str().len()
                + frame.function.len()
                + frame.location.as_ref().map_or(0, |v| v.file.len());
            if bytes > MAX_TRACE_BYTES {
                return Err(TraceError::TooManyBytes);
            }
        }
        let lookup = frames
            .iter()
            .enumerate()
            .map(|(i, f)| (f.key.clone(), i))
            .collect();
        Ok(Self {
            message,
            frames,
            lookup,
        })
    }
    pub fn message(&self) -> &TracePayload {
        &self.message
    }
    pub fn frames(&self) -> &[StackFrame] {
        &self.frames
    }
    pub fn frame(&self, key: &TraceKey) -> Option<&StackFrame> {
        self.lookup.get(key).map(|&i| &self.frames[i])
    }
}
