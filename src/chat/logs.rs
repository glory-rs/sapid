//! Bounded, indexed, read-only output. No shell, test runner, ANSI interpreter,
//! automatic links or retained history of previously visited records.
use crate::appearance::{Common, common_methods, configure, observe};
use crate::chat::{MessageScrollerHandle, message_scroller};
use glory::render::{SemanticId, TextKind};
use glory::style::{
    Display, Edges, Family, FlexDirection, FlexWrap, Length, Limit, Overflow, Style, WhiteSpace,
};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};
use std::rc::Rc;

pub const MAX_OUTPUT_RECORDS: usize = 1_000_000_000;
pub const MAX_OUTPUT_TEXT_BYTES: usize = 8192;
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogPayload {
    Text(String),
    /// Contains no original secret. Redaction must happen before passing data
    /// to the widget; the widget cannot discover secrets in ordinary text.
    Redacted,
}
impl LogPayload {
    pub fn display_text(&self) -> Result<String> {
        let Self::Text(raw) = self else {
            return Ok("[redacted]".into());
        };
        if raw.len() > MAX_OUTPUT_TEXT_BYTES {
            return Err(invalid("output record exceeds 8192 bytes"));
        }
        let mut out = String::with_capacity(raw.len());
        let mut chars = raw.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    out.push('\n');
                }
                '\n' | '\t' => out.push(c),
                c if c.is_control() => {
                    use std::fmt::Write;
                    write!(&mut out, "\\u{{{:x}}}", c as u32).expect("String write");
                }
                c => out.push(c),
            }
        }
        Ok(out)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogStream {
    #[default]
    Stdout,
    Stderr,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogLevel {
    #[default]
    Info,
    Debug,
    Warning,
    Error,
}
impl LogLevel {
    fn label(self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Debug => "Debug",
            Self::Warning => "Warning",
            Self::Error => "Error",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogRecord {
    pub key: u64,
    pub stream: LogStream,
    pub level: LogLevel,
    pub payload: LogPayload,
}
impl LogRecord {
    pub fn new(key: u64, text: impl Into<String>) -> Self {
        Self {
            key,
            stream: LogStream::Stdout,
            level: LogLevel::Info,
            payload: LogPayload::Text(text.into()),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Running,
}
impl TestStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Passed => "Passed",
            Self::Failed => "Failed",
            Self::Skipped => "Skipped",
            Self::Running => "Running",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestCase {
    pub key: u64,
    pub name: String,
    pub status: TestStatus,
    pub elapsed_millis: Option<u64>,
    pub output: LogPayload,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TestCounts {
    pub passed: u64,
    pub failed: u64,
    pub skipped: u64,
    pub running: u64,
}
impl TestCounts {
    pub fn total(&self) -> Result<usize> {
        let total = self
            .passed
            .checked_add(self.failed)
            .and_then(|n| n.checked_add(self.skipped))
            .and_then(|n| n.checked_add(self.running))
            .ok_or_else(|| invalid("test count overflow"))?;
        if total > MAX_OUTPUT_RECORDS as u64 {
            return Err(invalid("output admits at most one billion indexed records"));
        }
        Ok(total as usize)
    }
    pub fn summary(&self) -> Result<String> {
        self.total()?;
        Ok(format!(
            "{} passed · {} failed · {} skipped · {} running",
            self.passed, self.failed, self.skipped, self.running
        ))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct ProjectedRow {
    title: String,
    body: String,
    failed: bool,
    test: bool,
}
enum Source {
    Logs(Rc<dyn Fn(usize) -> LogRecord>),
    Tests(Rc<dyn Fn(usize) -> TestCase>),
}
impl Clone for Source {
    fn clone(&self) -> Self {
        match self {
            Self::Logs(r) => Self::Logs(r.clone()),
            Self::Tests(r) => Self::Tests(r.clone()),
        }
    }
}
impl Source {
    fn read(&self, index: usize, key: u64) -> Result<ProjectedRow> {
        match self {
            Self::Logs(read) => {
                let row = read(index);
                if row.key != key {
                    return Err(invalid(
                        "indexed output record key does not match its mounted key",
                    ));
                }
                Ok(ProjectedRow {
                    title: format!(
                        "{} · {} · {}",
                        index + 1,
                        match row.stream {
                            LogStream::Stdout => "stdout",
                            LogStream::Stderr => "stderr",
                        },
                        row.level.label()
                    ),
                    body: row.payload.display_text()?,
                    failed: row.stream == LogStream::Stderr || row.level == LogLevel::Error,
                    test: false,
                })
            }
            Self::Tests(read) => {
                let row = read(index);
                if row.key != key {
                    return Err(invalid(
                        "indexed test record key does not match its mounted key",
                    ));
                }
                if row.name.len() > 256
                    || row.name.trim().is_empty()
                    || row.name.chars().any(char::is_control)
                    || row.elapsed_millis.is_some_and(|v| v > 604_800_000)
                {
                    return Err(invalid("invalid bounded test name or elapsed time"));
                }
                Ok(ProjectedRow {
                    title: format!(
                        "{} · {}{}",
                        row.status.label(),
                        row.name,
                        row.elapsed_millis
                            .map_or_else(String::new, |v| format!(" · {v} ms"))
                    ),
                    body: row.output.display_text()?,
                    failed: row.status == TestStatus::Failed,
                    test: true,
                })
            }
        }
    }
}
/// The controller retains no output source or application read callback.
#[derive(Clone)]
pub struct OutputHandle(MessageScrollerHandle);
impl OutputHandle {
    pub fn first(&self) -> Result {
        self.0.seek(0.)
    }
    pub fn seek(&self, logical: f64) -> Result {
        self.0.seek(logical)
    }
    pub fn newest(&self) -> Result {
        self.0.follow_tail()
    }
    pub fn pause(&self) -> Result {
        self.0.pause()
    }
    pub fn is_following(&self) -> Result<bool> {
        Ok(self.0.status()?.following)
    }
}
pub struct OutputWindow {
    common: Common,
    namespace: String,
    count: Vary<usize>,
    counts: Option<Vary<TestCounts>>,
    key: Rc<dyn Fn(usize) -> u64>,
    locate: Rc<dyn Fn(u64) -> Option<usize>>,
    source: Source,
    revision: Vary<u64>,
    height: f64,
    estimate: f64,
    row_limit: usize,
    measure_limit: usize,
    overscan: usize,
    following: bool,
    ready: Option<Box<dyn FnMut(OutputHandle)>>,
    inspect: Option<Box<dyn FnMut(glory::ui::list::ListStats)>>,
}
pub type Terminal = OutputWindow;
pub type TestResults = OutputWindow;
/// `key`, `locate` and `read` must use indexed application access. Only the live
/// window plus one retained focused record is sampled, never the whole source.
pub fn terminal(
    namespace: impl Into<String>,
    count: impl Into<Vary<usize>>,
    key: impl Fn(usize) -> u64 + 'static,
    locate: impl Fn(u64) -> Option<usize> + 'static,
    read: impl Fn(usize) -> LogRecord + 'static,
) -> Terminal {
    OutputWindow {
        common: Common::default(),
        namespace: namespace.into(),
        count: count.into(),
        counts: None,
        key: Rc::new(key),
        locate: Rc::new(locate),
        source: Source::Logs(Rc::new(read)),
        revision: 0.into(),
        height: 480.,
        estimate: 64.,
        row_limit: 64,
        measure_limit: 256,
        overscan: 2,
        following: true,
        ready: None,
        inspect: None,
    }
}
/// Counts are explicit application facts; no test execution or full-source
/// verification is performed. The indexed source must agree with their sum.
pub fn test_results(
    namespace: impl Into<String>,
    counts: impl Into<Vary<TestCounts>>,
    key: impl Fn(usize) -> u64 + 'static,
    locate: impl Fn(u64) -> Option<usize> + 'static,
    read: impl Fn(usize) -> TestCase + 'static,
) -> TestResults {
    let mut out = terminal(namespace, 0, key, locate, |_| {
        unreachable!("test source installed")
    });
    out.counts = Some(counts.into());
    out.source = Source::Tests(Rc::new(read));
    out.estimate = 96.;
    out
}
impl OutputWindow {
    common_methods!();
    pub fn revision(mut self, value: impl Into<Vary<u64>>) -> Self {
        self.revision = value.into();
        self
    }
    pub fn height(mut self, value: f64) -> Self {
        self.height = value;
        self
    }
    pub fn estimate(mut self, value: f64) -> Self {
        self.estimate = value;
        self
    }
    pub fn row_limit(mut self, value: usize) -> Self {
        self.row_limit = value;
        self
    }
    pub fn measure_limit(mut self, value: usize) -> Self {
        self.measure_limit = value;
        self
    }
    pub fn overscan(mut self, value: usize) -> Self {
        self.overscan = value;
        self
    }
    pub fn initially_following(mut self, value: bool) -> Self {
        self.following = value;
        self
    }
    pub fn on_ready(mut self, value: impl FnMut(OutputHandle) + 'static) -> Self {
        self.ready = Some(Box::new(value));
        self
    }
    pub fn inspect(mut self, value: impl FnMut(glory::ui::list::ListStats) + 'static) -> Self {
        self.inspect = Some(Box::new(value));
        self
    }
}
struct OutputRow {
    namespace: String,
    source: Source,
    locate: Rc<dyn Fn(u64) -> Option<usize>>,
    count: Vary<usize>,
    index: Cage<usize>,
    key: u64,
    revision: Vary<u64>,
    theme: crate::Theme,
    unstyled: bool,
}
impl Widget for OutputRow {
    fn build(&mut self, scope: &Scope) -> Result {
        let initial = glory::core::untracked(|| {
            let at = (self.locate)(self.key)
                .filter(|at| *at < self.count.get())
                .ok_or_else(|| invalid("output key is not in its indexed source"))?;
            self.source.read(at, self.key)
        })?;
        let test = initial.test;
        let accepted = scope.cage(initial);
        let source = self.source.clone();
        let index = self.index;
        let key = self.key;
        let revision = self.revision.clone();
        let locate = self.locate.clone();
        let count = self.count.clone();
        let weak = scope.downgrade();
        scope.structural(move || {
            let _ = revision.get();
            // List may not have updated its row-index Cage yet in this tick.
            // Resolve this mounted key with the current indexed lookup first.
            let _ = index.get();
            let Some(at) = locate(key).filter(|at| *at < count.get()) else {
                return;
            };
            let next = source.read(at, key);
            let Some(scope) = weak.upgrade() else { return };
            match next {
                Ok(next) => {
                    if glory::core::untracked(|| accepted.with(|old| old != &next)) {
                        accepted.revise(|old| *old = next);
                    }
                }
                Err(e) => scope.report_error(e),
            }
        });
        let mut body = if self.unstyled {
            Style::new()
        } else {
            self.theme.label(crate::Size::Medium)
        };
        body.font_family = Some(vec![Family::Mono]);
        body.white_space = Some(WhiteSpace::Pre);
        body.overflow_x = Some(Overflow::Auto);
        body.max_width = Some(Limit::Percent(100.));
        body.min_width = Some(Length::Px(0.));
        if test {
            let theme = self.theme.clone();
            let key = format!("{}-{}", self.namespace, self.key);
            let unstyled = self.unstyled;
            let mut disclosure = crate::collapsible(key)
                .class("sapid-test-case")
                .theme(theme)
                .label(scope.bond(move || accepted.with(|r| r.title.clone())))
                .content(move || {
                    glory::ui::text()
                        .kind(TextKind::Paragraph)
                        .class("sapid-output-text")
                        .content(accepted.map(|r| r.body.clone()))
                        .style(body.clone())
                });
            if unstyled {
                disclosure = disclosure.unstyled();
            }
            disclosure.show_in(scope)
        } else {
            let theme = self.theme.clone();
            let unstyled = self.unstyled;
            glory::ui::view()
                .class("sapid-log-record")
                .style(Style::new().min_width(Length::Px(0.)))
                .fill((
                    glory::ui::text()
                        .class("sapid-output-label")
                        .content(scope.bond(move || accepted.with(|r| r.title.clone())))
                        .style(scope.bond(move || {
                            let mut s = if unstyled {
                                Style::new()
                            } else {
                                theme.label(crate::Size::Small)
                            };
                            if !unstyled {
                                s.color = Some(if accepted.with(|r| r.failed) {
                                    theme.danger
                                } else {
                                    theme.muted
                                });
                            }
                            s
                        })),
                    glory::ui::text()
                        .kind(TextKind::Paragraph)
                        .class("sapid-output-text")
                        .content(scope.bond(move || accepted.with(|r| r.body.clone())))
                        .style(body),
                ))
                .show_in(scope)
        }
    }
}
impl Widget for OutputWindow {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.namespace.len() > 40 {
            return Err(invalid("output namespace exceeds 40 bytes"));
        }
        crate::composite::id("output", &self.namespace, "root", "root")?;
        let tests = self.counts.is_some();
        let summary = if let Some(source) = &self.counts {
            Some(crate::composite::validated(scope, source, |v| {
                v.total()?;
                Ok(())
            })?)
        } else {
            None
        };
        let count: Vary<usize> = if let Some(summary) = summary {
            scope
                .bond(move || summary.with(|v| v.total().expect("validated test counts")))
                .into()
        } else {
            crate::composite::validated(scope, &self.count, |v| {
                if *v <= MAX_OUTPUT_RECORDS {
                    Ok(())
                } else {
                    Err(invalid("output admits at most one billion indexed records"))
                }
            })?
            .into()
        };
        let handle = scope.cage(None::<OutputHandle>);
        let mut ready = self.ready.take();
        let key = self.key.clone();
        let locate = self.locate.clone();
        let source = self.source.clone();
        let revision = self.revision.clone();
        let theme = self.common.theme.clone();
        let unstyled = self.common.unstyled;
        let namespace = self.namespace.clone();
        let id_namespace = namespace.clone();
        let row_locate = locate.clone();
        let row_count = count.clone();
        let mut list = message_scroller(
            count,
            move |i| {
                SemanticId::new(format!(
                    "output-{}-{}",
                    id_namespace,
                    glory::core::untracked(|| key(i))
                ))
                .expect("numeric output identity")
            },
            move |id| {
                id.as_str()
                    .rsplit('-')
                    .next()
                    .and_then(|k| k.parse().ok())
                    .and_then(|key| glory::core::untracked(|| locate(key)))
            },
            move |index, key| OutputRow {
                namespace: namespace.clone(),
                source: source.clone(),
                locate: row_locate.clone(),
                count: row_count.clone(),
                index,
                key: key
                    .as_str()
                    .rsplit('-')
                    .next()
                    .and_then(|k| k.parse().ok())
                    .expect("numeric output identity"),
                revision: revision.clone(),
                theme: theme.clone(),
                unstyled,
            },
        )
        .name(if tests {
            "Test results"
        } else {
            "Terminal output"
        })
        .theme(self.common.theme.clone())
        .revision(self.revision.clone())
        .height(self.height)
        .estimate(self.estimate)
        .row_limit(self.row_limit)
        .measure_limit(self.measure_limit)
        .overscan(self.overscan)
        .initially_following(self.following)
        .on_ready(move |h| {
            let h = OutputHandle(h);
            handle.revise(|v| *v = Some(h.clone()));
            if let Some(callback) = &mut ready {
                callback(h);
            }
        });
        if let Some(inspect) = self.inspect.take() {
            list = list.inspect(inspect);
        }
        let theme = self.common.theme.clone();
        let mut base = theme.label(self.common.size);
        base.background = Some(theme.surface);
        base.padding = Some(Edges::all(8.));
        base.min_width = Some(Length::Px(0.));
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            if tests {
                "sapid-test-results"
            } else {
                "sapid-terminal"
            },
            base
        );
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        let toolbar_theme = theme.clone();
        let mut toolbar = glory::ui::view().class("sapid-output-toolbar").style(
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .gap(8.),
        );
        for (label, action) in [
            ("First record", 0),
            ("Newest records", 1),
            ("Pause following", 2),
        ] {
            let weak = scope.downgrade();
            toolbar = toolbar.fill(
                crate::button()
                    .theme(toolbar_theme.clone())
                    .enabled(self.common.enabled.clone())
                    .label(label)
                    .on_press(move || {
                        if let Some(scope) = weak.upgrade()
                            && let Some(h) = handle.get_untracked()
                        {
                            let result = match action {
                                0 => h.first(),
                                1 => h.newest(),
                                _ => h.pause(),
                            };
                            if let Err(e) = result {
                                scope.report_error(e);
                            }
                        }
                    }),
            );
        }
        root = root.fill(toolbar);
        if let Some(summary) = summary {
            root = root.fill(
                glory::ui::text()
                    .class("sapid-test-summary")
                    .content(scope.bond(move || {
                        summary.with(|v| {
                            if v.total().expect("validated counts") == 0 {
                                "No tests supplied".into()
                            } else {
                                v.summary().expect("validated counts")
                            }
                        })
                    })),
            );
        }
        root.fill(list).show_in(scope)
    }
}
