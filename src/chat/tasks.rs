//! Finite application plans, tasks and queues. Actions are proposals delivered
//! to the application; this module has no scheduler or execution service.
use crate::appearance::{Common, common_methods, configure, observe};
use crate::composite::{self, Callback};
use glory::render::{DocumentPart, Semantics, TextKind};
use glory::style::{Edges, Length, Limit, Overflow, Style, WhiteSpace};
use glory::ui::{Result, Widget};
use glory::{Lotus, Scope};
use std::collections::BTreeSet;
pub const MAX_PLAN_TASKS: usize = 128;
pub const MAX_PLAN_BYTES: usize = 131072;
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))
}
fn label(value: &str) -> bool {
    value.len() <= 256 && !value.trim().is_empty() && !value.chars().any(char::is_control)
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TaskStatus {
    #[default]
    Pending,
    Running,
    Complete,
    Failed,
    Skipped,
    Cancelled,
}
impl TaskStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Running => "Running",
            Self::Complete => "Complete",
            Self::Failed => "Failed",
            Self::Skipped => "Skipped",
            Self::Cancelled => "Cancelled",
        }
    }
    pub fn is_terminal(self) -> bool {
        !matches!(self, Self::Pending | Self::Running)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskAction {
    pub key: String,
    pub label: String,
    pub enabled: bool,
}
impl TaskAction {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            enabled: true,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskRecord {
    pub key: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub progress: Option<u8>,
    pub action: Option<TaskAction>,
}
impl TaskRecord {
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            description: String::new(),
            status: TaskStatus::Pending,
            progress: None,
            action: None,
        }
    }
    pub fn validate(&self) -> Result {
        if !key(&self.key)
            || !label(&self.title)
            || self.description.len() > 8192
            || self.description.contains(['\0', '\r'])
            || self.progress.is_some_and(|v| v > 100)
        {
            return Err(invalid("invalid bounded task key, text or percentage"));
        }
        if self
            .action
            .as_ref()
            .is_some_and(|a| !key(&a.key) || !label(&a.label))
        {
            return Err(invalid("invalid bounded application task action"));
        }
        Ok(())
    }
    fn bytes(&self) -> usize {
        self.key.len()
            + self.title.len()
            + self.description.len()
            + self
                .action
                .as_ref()
                .map_or(0, |a| a.key.len() + a.label.len())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanData {
    pub title: String,
    pub description: String,
    pub tasks: Vec<TaskRecord>,
}
impl Default for PlanData {
    fn default() -> Self {
        Self {
            title: "Application plan".into(),
            description: String::new(),
            tasks: vec![],
        }
    }
}
impl PlanData {
    pub fn validate(&self) -> Result {
        if self.tasks.len() > MAX_PLAN_TASKS
            || !label(&self.title)
            || self.description.len() > 8192
            || self.description.contains(['\0', '\r'])
        {
            return Err(invalid("invalid bounded plan header or task count"));
        }
        let mut keys = BTreeSet::new();
        let mut bytes = self.title.len() + self.description.len();
        for task in &self.tasks {
            task.validate()?;
            if !keys.insert(task.key.as_str()) {
                return Err(invalid("task keys must be distinct within a plan or queue"));
            }
            bytes += task.bytes();
            if bytes > MAX_PLAN_BYTES {
                return Err(invalid("plan exceeds 128 KiB of public text"));
            }
        }
        Ok(())
    }
    pub fn processed(&self) -> usize {
        self.tasks.iter().filter(|t| t.status.is_terminal()).count()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskRequest {
    pub task_key: String,
    pub action_key: String,
}
pub struct Task {
    common: Common,
    namespace: String,
    data: Lotus<TaskRecord>,
    action: Option<Box<dyn FnMut(TaskRequest)>>,
}
pub fn task(namespace: impl Into<String>, data: impl Into<Lotus<TaskRecord>>) -> Task {
    Task {
        common: Common::default(),
        namespace: namespace.into(),
        data: data.into(),
        action: None,
    }
}
impl Task {
    common_methods!();
    pub fn on_action(mut self, value: impl FnMut(TaskRequest) + 'static) -> Self {
        self.action = Some(Box::new(value));
        self
    }
}
impl Widget for Task {
    fn build(&mut self, scope: &Scope) -> Result {
        composite::id("task", &self.namespace, "root", "root")?;
        let identity = glory::core::untracked(|| {
            self.data.try_with(|v| {
                v.validate()?;
                Ok::<_, glory::ui::Error>(v.key.clone())
            })
        })??;
        let semantic = composite::id("task", &self.namespace, "entry", &identity)?;
        let accepted = composite::validated(scope, &self.data, move |v| {
            v.validate()?;
            if v.key != identity {
                return Err(invalid("a live Task keeps its key; remount to replace it"));
            }
            Ok(())
        })?;
        let available = self.action.is_some();
        let callback = Callback::new(self.action.take());
        let cleanup = callback.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let owner = scope.id();
        let theme = self.common.theme.clone();
        let size = self.common.size;
        let unstyled = self.common.unstyled;
        let enabled = self.common.enabled.clone();
        let mut base = theme.label(size);
        base.background = Some(theme.surface);
        base.padding = Some(Edges::all(8.));
        base.min_width = Some(Length::Px(0.));
        base.max_width = Some(Limit::Percent(100.));
        let mut root = configure!(self.common, glory::ui::view(), scope, "sapid-task", base)
            .semantics(composite::metadata(Semantics {
                id: Some(semantic),
                ..Default::default()
            }))
            .fill((
                glory::ui::text()
                    .class("sapid-task-title")
                    .content(scope.bond(move || {
                        accepted.with(|t| format!("{} · {}", t.status.label(), t.title))
                    })),
                glory::ui::text()
                    .kind(TextKind::Paragraph)
                    .class("sapid-task-description")
                    .content(scope.bond(move || accepted.with(|t| t.description.clone())))
                    .style(
                        Style::new()
                            .white_space(WhiteSpace::PreWrap)
                            .overflow_x(Overflow::Auto)
                            .min_width(Length::Px(0.)),
                    ),
            ));
        let palette = theme.clone();
        root = root.fill(glory::ui::when(
            scope.bond(move || accepted.with(|t| t.progress.is_some())),
            move || {
                let theme = palette.clone();
                glory::ui::component(move |scope| {
                    let mut progress =
                        crate::progress(scope.bond(move || {
                            accepted.with(|t| f64::from(t.progress.unwrap_or(0)))
                        }))
                        .theme(theme.clone())
                        .name(scope.bond(move || {
                            accepted.with(|t| format!("Task progress: {}", t.title))
                        }));
                    if unstyled {
                        progress = progress.unstyled();
                    }
                    progress.show_in(scope)
                })
            },
        ));
        let palette = theme.clone();
        root = root.fill(glory::ui::when(
            scope.bond(move || accepted.with(|t| t.action.is_some())),
            move || {
                let theme = palette.clone();
                let callback = callback.clone();
                let enabled = enabled.clone();
                glory::ui::component(move |scope| {
                    let enabled = enabled.clone();
                    let callback = callback.clone();
                    let mut button = crate::button()
                        .class("sapid-task-action")
                        .theme(theme.clone())
                        .size(size)
                        .label(scope.bond(move || {
                            accepted.with(|t| {
                                t.action
                                    .as_ref()
                                    .map_or_else(String::new, |a| a.label.clone())
                            })
                        }))
                        .enabled(scope.bond(move || {
                            enabled.get()
                                && available
                                && accepted
                                    .with(|t| t.action.as_ref().is_some_and(|a| a.enabled))
                        }))
                        .on_press(move || {
                            let request = accepted.with(|t| {
                                t.action
                                    .as_ref()
                                    .filter(|a| a.enabled)
                                    .map(|a| TaskRequest {
                                        task_key: t.key.clone(),
                                        action_key: a.key.clone(),
                                    })
                            });
                            if let Some(request) = request {
                                callback.emit(owner, request);
                            }
                        });
                    if unstyled {
                        button = button.unstyled();
                    }
                    button.show_in(scope)
                })
            },
        ));
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |e| observe(&mut handlers, owner, e));
        }
        root.show_in(scope)
    }
}
pub struct Plan {
    common: Common,
    namespace: String,
    data: Lotus<PlanData>,
    queue: bool,
    action: Option<Box<dyn FnMut(TaskRequest)>>,
}
pub type Queue = Plan;
pub fn plan(namespace: impl Into<String>) -> Plan {
    Plan {
        common: Common::default(),
        namespace: namespace.into(),
        data: PlanData::default().into(),
        queue: false,
        action: None,
    }
}
pub fn queue(namespace: impl Into<String>) -> Queue {
    Plan {
        queue: true,
        ..plan(namespace)
    }
}
impl Plan {
    common_methods!();
    pub fn data(mut self, value: impl Into<Lotus<PlanData>>) -> Self {
        self.data = value.into();
        self
    }
    pub fn on_action(mut self, value: impl FnMut(TaskRequest) + 'static) -> Self {
        self.action = Some(Box::new(value));
        self
    }
}
impl Widget for Plan {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.namespace.len() > 40 {
            return Err(invalid("plan namespace exceeds 40 bytes"));
        }
        composite::id("plan", &self.namespace, "root", "root")?;
        let accepted = composite::validated(scope, &self.data, PlanData::validate)?;
        let available = self.action.is_some();
        let callback = Callback::new(self.action.take());
        let cleanup = callback.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let owner = scope.id();
        let theme = self.common.theme.clone();
        let size = self.common.size;
        let unstyled = self.common.unstyled;
        let enabled = self.common.enabled.clone();
        let namespace = self.namespace.clone();
        let mut base = theme.label(size);
        base.background = Some(theme.surface);
        base.padding = Some(Edges::all(8.));
        base.min_width = Some(Length::Px(0.));
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            if self.queue {
                "sapid-queue"
            } else {
                "sapid-plan"
            },
            base
        )
        .fill((
            glory::ui::text()
                .kind(TextKind::Heading(3))
                .class("sapid-plan-title")
                .content(scope.bond(move || accepted.with(|p| p.title.clone()))),
            glory::ui::text()
                .kind(TextKind::Paragraph)
                .class("sapid-plan-description")
                .content(scope.bond(move || accepted.with(|p| p.description.clone())))
                .style(Style::new().white_space(WhiteSpace::PreWrap)),
            glory::ui::text()
                .class("sapid-plan-summary")
                .content(scope.bond(move || {
                    accepted.with(|p| {
                        if p.tasks.is_empty() {
                            "No tasks supplied".into()
                        } else {
                            format!("Processed {} of {} tasks", p.processed(), p.tasks.len())
                        }
                    })
                })),
        ));
        let palette = theme.clone();
        root = root.fill(glory::ui::when(
            scope.bond(move || accepted.with(|p| !p.tasks.is_empty())),
            move || {
                let theme = palette.clone();
                glory::ui::component(move |scope| {
                    let mut progress = crate::progress(scope.bond(move || {
                        accepted
                            .with(|p| p.processed() as f64 / p.tasks.len().max(1) as f64 * 100.)
                    }))
                    .theme(theme.clone())
                    .class("sapid-plan-progress")
                    .name("Processed tasks");
                    if unstyled {
                        progress = progress.unstyled();
                    }
                    progress.show_in(scope)
                })
            },
        ));
        let palette = theme.clone();
        root = root.fill(
            glory::ui::view()
                .class("sapid-task-list")
                .name(if self.queue {
                    "Task queue"
                } else {
                    "Plan tasks"
                })
                .document_part(DocumentPart::OrderedList)
                .style(Style::new().min_width(Length::Px(0.)))
                .fill(glory::ui::each(
                    scope.bond(move || {
                        accepted
                            .with(|p| p.tasks.iter().map(|t| t.key.clone()).collect::<Vec<_>>())
                    }),
                    |k| k.clone(),
                    move |key, _| {
                        let key = key.get_untracked();
                        let theme = palette.clone();
                        let callback = callback.clone();
                        let enabled = enabled.clone();
                        let namespace = namespace.clone();
                        glory::ui::component(move |scope| {
                            let lookup = key.clone();
                            let initial = glory::core::untracked(|| {
                                accepted
                                    .with(|p| p.tasks.iter().find(|t| t.key == lookup).cloned())
                            })
                            .ok_or(glory::ui::Error::Gone)?;
                            let data = scope.cage(initial);
                            scope.structural(move || {
                                // A row invalidation may precede Each's
                                // retirement of a removed key. Keep the last
                                // complete row until that Owner is retired.
                                if let Some(next) = accepted
                                    .with(|p| p.tasks.iter().find(|t| t.key == lookup).cloned())
                                    && glory::core::untracked(|| data.with(|old| old != &next))
                                {
                                    data.revise(|old| *old = next);
                                }
                            });
                            let mut row = task(namespace.clone(), data)
                                .theme(theme.clone())
                                .size(size)
                                .enabled(enabled.clone());
                            if available {
                                let callback = callback.clone();
                                row = row.on_action(move |r| callback.emit(owner, r));
                            }
                            if unstyled {
                                row = row.unstyled();
                            }
                            glory::ui::view()
                                .class("sapid-task-list-item")
                                .document_part(DocumentPart::ListItem)
                                .fill(row)
                                .show_in(scope)
                        })
                    },
                )),
        );
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |e| observe(&mut handlers, owner, e));
        }
        root.show_in(scope)
    }
}
