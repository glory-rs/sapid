//! Finite workflow steps. Applications own validation, submission and routing.
use crate::ContentPolicy;
use crate::appearance::{Common, common_methods, configure};
use crate::composite::{self, Callback, Factory, Item, Observers, item_methods};
use crate::state::Value;
use glory::render::{
    Current, Event, EventKind, FocusMode, NodeId, Orientation, Role, SemanticId, Semantics,
};
use glory::style::{Display, Edges, FlexDirection, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};
use std::rc::Rc;

pub const MAX_STEPPER_STEPS: usize = 128;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepStatus {
    #[default]
    Pending,
    Completed,
    Error,
}
pub struct Step {
    inner: Item,
    description: Vary<String>,
    status: Vary<Option<StepStatus>>,
    optional: bool,
}
pub fn step(key: impl Into<String>) -> Step {
    Step {
        inner: Item::new(key.into()),
        description: "".into(),
        status: None.into(),
        optional: false,
    }
}
impl Step {
    item_methods!();
    pub fn description(mut self, value: impl Into<Vary<String>>) -> Self {
        self.description = value.into();
        self
    }
    /// None derives completed/pending from the accepted active step.
    pub fn status(mut self, value: impl Into<Vary<Option<StepStatus>>>) -> Self {
        self.status = value.into();
        self
    }
    pub fn optional(mut self, value: bool) -> Self {
        self.optional = value;
        self
    }
}
pub struct Stepper {
    common: Common,
    key: String,
    steps: Vec<Step>,
    active: Value<usize>,
    selectable: Vary<bool>,
    skip: Vary<bool>,
    orientation: Orientation,
    policy: ContentPolicy,
    completed: Option<Factory>,
    completed_label: Vary<String>,
    change: Option<Box<dyn FnMut(usize)>>,
}
pub fn stepper(key: impl Into<String>) -> Stepper {
    Stepper {
        common: Common::default(),
        key: key.into(),
        steps: Vec::new(),
        active: Value::default(),
        selectable: false.into(),
        skip: false.into(),
        orientation: Orientation::Horizontal,
        policy: ContentPolicy::KeepMounted,
        completed: None,
        completed_label: "Completed".into(),
        change: None,
    }
}
impl Stepper {
    common_methods!();
    pub fn step(mut self, value: Step) -> Self {
        if self.steps.len() <= MAX_STEPPER_STEPS {
            self.steps.push(value);
        }
        self
    }
    pub fn steps(mut self, value: impl IntoIterator<Item = Step>) -> Self {
        self.steps = value.into_iter().take(MAX_STEPPER_STEPS + 1).collect();
        self
    }
    /// Zero-based; exactly the number of steps represents the completed stage.
    pub fn active(mut self, value: impl Into<Vary<usize>>) -> Self {
        self.active.source = Some(value.into());
        self
    }
    pub fn default_active(mut self, value: usize) -> Self {
        self.active.default = value;
        self
    }
    /// Explicit opt-in to selecting step headers. Body controls stay independent.
    pub fn selectable(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.selectable = value.into();
        self
    }
    /// Permit selecting beyond the immediately following step.
    pub fn allow_skip(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.skip = value.into();
        self
    }
    pub fn orientation(mut self, value: Orientation) -> Self {
        self.orientation = value;
        self
    }
    pub fn content_policy(mut self, value: ContentPolicy) -> Self {
        self.policy = value;
        self
    }
    pub fn completed<W: Widget>(mut self, mut factory: impl FnMut() -> W + 'static) -> Self {
        self.completed = Some(Box::new(move || Box::new(factory())));
        self
    }
    pub fn completed_label(mut self, value: impl Into<Vary<String>>) -> Self {
        self.completed_label = value.into();
        self
    }
    pub fn on_step_change(mut self, callback: impl FnMut(usize) + 'static) -> Self {
        self.change = Some(Box::new(callback));
        self
    }
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn metadata(id: SemanticId, label: SemanticId, hidden: bool) -> Option<Box<Semantics>> {
    composite::metadata(Semantics {
        id: Some(id),
        labelled_by: Some(label),
        hidden,
        ..Default::default()
    })
}

struct StepPanel {
    id: SemanticId,
    label: SemanticId,
    open: Vary<bool>,
    reference: Cage<Option<NodeId>>,
    restore: Vary<Option<NodeId>>,
    factory: Option<Factory>,
    policy: ContentPolicy,
    unstyled: bool,
}
impl Widget for StepPanel {
    fn build(&mut self, scope: &Scope) -> Result {
        let focused = scope.cage(false);
        let open = self.open.clone();
        let id = self.id.clone();
        let label = self.label.clone();
        let mut panel = glory::ui::view()
            .role(Role::Group)
            .focus(FocusMode::Programmatic)
            .node_ref(self.reference)
            .class("sapid-stepper-panel")
            .semantics(scope.bond(move || metadata(id.clone(), label.clone(), !open.get())))
            .on(EventKind::FocusWithin, move |event| {
                if let Event::FocusWithin { focused: value, .. } = event {
                    focused.revise(|old| *old = value)
                }
            });
        if !self.unstyled {
            panel = panel.style(Style::new().padding(Edges::all(12.0)));
        }
        if let Some(mut factory) = self.factory.take() {
            panel = match self.policy {
                ContentPolicy::KeepMounted => {
                    panel.fill(glory::ui::widget(move |scope| factory().show_in(scope)))
                }
                ContentPolicy::Unmount => {
                    panel.fill(glory::ui::when(self.open.clone(), factory))
                }
            };
        }
        panel.show_in(scope)?;
        let open = self.open.clone();
        let restore = self.restore.clone();
        let mut previous = open.try_get()?;
        let weak = scope.downgrade();
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let now = open.try_get()?;
                if previous && !now && focused.try_get()? {
                    focused.try_revise(|old| *old = false)?;
                    composite::focus_node(&scope, restore.try_get()?)?;
                }
                previous = now;
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        Ok(())
    }
}
impl Widget for Stepper {
    fn build(&mut self, scope: &Scope) -> Result {
        let count = self.steps.len();
        if count == 0 || count > MAX_STEPPER_STEPS {
            return Err(invalid("stepper requires 1..=128 steps"));
        }
        let root_id = composite::id("stepper", &self.key, "root", "root")?;
        let keys: Vec<_> = self
            .steps
            .iter()
            .map(|step| step.inner.key.clone())
            .collect();
        composite::validate_keys(&keys)?;
        let control = std::mem::take(&mut self.active).mount(scope);
        let accepted = composite::validated(scope, &control.source, move |value| {
            if *value <= count {
                Ok(())
            } else {
                Err(invalid("active step exceeds completion stage"))
            }
        })?;
        let references: Rc<Vec<_>> = Rc::new(
            self.steps
                .iter()
                .map(|step| step.inner.panel.unwrap_or_else(|| scope.cage(None)))
                .chain(std::iter::once(scope.cage(None)))
                .collect(),
        );
        let refs = references.clone();
        let restore: Vary<Option<NodeId>> =
            scope.bond(move || refs[accepted.get()].get()).into();
        if self.common.name.is_none() {
            self.common.name = Some("Progress".into());
        }
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-stepper",
            self.common.theme.label(self.common.size)
        )
        .role(Role::Group)
        .semantics(composite::metadata(Semantics {
            id: Some(root_id),
            ..Default::default()
        }));
        let mut headers = glory::ui::view()
            .role(Role::List)
            .name("Steps")
            .class("sapid-stepper-headers");
        if !self.common.unstyled {
            headers = headers.style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(if self.orientation == Orientation::Horizontal {
                        FlexDirection::Row
                    } else {
                        FlexDirection::Column
                    })
                    .gap(12.0),
            );
        }
        let callback = Callback::new(self.change.take());
        let observers = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Activate)
                .unwrap_or_default(),
        );
        let owner = scope.id();
        let mut panels = glory::ui::group();
        for (index, mut step) in std::mem::take(&mut self.steps).into_iter().enumerate() {
            let trigger_id = composite::id("stepper", &self.key, "trigger", &step.inner.key)?;
            let panel_id = composite::id("stepper", &self.key, "panel", &step.inner.key)?;
            let description_id =
                composite::id("stepper", &self.key, "description", &step.inner.key)?;
            let explicit = step.status.clone();
            let status = scope.bond(move || {
                explicit.get().unwrap_or(if index < accepted.get() {
                    StepStatus::Completed
                } else {
                    StepStatus::Pending
                })
            });
            let label = step.inner.label.clone();
            let content = scope.bond(move || format!("{}. {}", index + 1, label.get()));
            let description = step.description;
            let optional = step.optional;
            let caption = scope.bond(move || {
                let status = match status.get() {
                    StepStatus::Pending => "Pending",
                    StepStatus::Completed => "Completed",
                    StepStatus::Error => "Error",
                };
                let description = description.get();
                format!(
                    "Step {} of {count} · {status}{}{}{}",
                    index + 1,
                    if optional { " · Optional" } else { "" },
                    if description.is_empty() { "" } else { " · " },
                    description
                )
            });
            let own_enabled = step.inner.enabled.clone();
            let enabled = self.common.enabled.clone();
            let selectable = self.selectable.clone();
            let skip = self.skip.clone();
            let enabled = scope.bond(move || {
                enabled.get()
                    && selectable.get()
                    && own_enabled.get()
                    && (index <= accepted.get().saturating_add(1) || skip.get())
            });
            let selected = scope.bond(move || accepted.get() == index);
            let panel = panel_id.clone();
            let trigger = trigger_id.clone();
            let description = description_id.clone();
            let mut button = glory::ui::button()
                .label(content)
                .enabled(enabled)
                .class("sapid-button sapid-stepper-trigger")
                .semantics(scope.bond(move || {
                    composite::metadata(Semantics {
                        id: Some(trigger.clone()),
                        controls: Some(panel.clone()),
                        described_by: Some(description.clone()),
                        current: selected.get().then_some(Current::Step),
                        ..Default::default()
                    })
                }));
            if let Some(reference) = step.inner.trigger {
                button = button.node_ref(reference);
            }
            if !self.common.unstyled {
                let theme = self.common.theme.clone();
                let size = self.common.size;
                button = button.style(scope.bond(move || {
                    let mut style = theme.control(size);
                    if status.get() == StepStatus::Error {
                        style.background = Some(theme.danger_soft);
                        style.color = Some(theme.danger);
                        style.border_color = Some(theme.danger);
                    } else if selected.get() {
                        style.background = Some(theme.accent);
                        style.color = Some(theme.on_accent);
                    } else if status.get() == StepStatus::Completed {
                        style.background = Some(theme.accent_soft);
                        style.color = Some(theme.accent_text);
                    }
                    style
                }));
            }
            let control = control.clone();
            let callback = callback.clone();
            let observers = observers.clone();
            let weak = scope.downgrade();
            button = button.on(EventKind::Activate, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    if enabled.try_get()? && accepted.try_get()? != index {
                        control.accept(&index)?;
                        callback.emit(owner, index);
                    }
                    if owner.is_alive() {
                        observers.emit(owner, event);
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                }
            });
            let mut caption = glory::ui::text()
                .content(caption)
                .class("sapid-stepper-description")
                .semantics(composite::metadata(Semantics {
                    id: Some(description_id),
                    ..Default::default()
                }));
            if !self.common.unstyled {
                let theme = self.common.theme.clone();
                caption = caption.style(scope.bond(move || Style {
                    color: Some(if status.get() == StepStatus::Error {
                        theme.danger
                    } else {
                        theme.muted
                    }),
                    ..Style::new()
                }));
            }
            headers = headers.fill(
                glory::ui::view()
                    .role(Role::ListItem)
                    .class("sapid-stepper-step")
                    .fill((button, caption)),
            );
            panels = panels.fill(StepPanel {
                id: panel_id,
                label: trigger_id,
                open: selected.into(),
                reference: references[index],
                restore: restore.clone(),
                factory: step.inner.factory.take(),
                policy: self.policy,
                unstyled: self.common.unstyled,
            });
        }
        let completed_id = composite::id("stepper", &self.key, "completed", "completed")?;
        let completed_label_id =
            composite::id("stepper", &self.key, "completed-label", "completed")?;
        let id = completed_label_id.clone();
        let mut label = glory::ui::text()
            .content(self.completed_label.clone())
            .class("sapid-stepper-completed-label")
            .semantics(scope.bond(move || {
                composite::metadata(Semantics {
                    id: Some(id.clone()),
                    hidden: false,
                    ..Default::default()
                })
            }));
        // Text itself cannot hide. Keep it in a stable named View shell.
        label = label.style(if self.common.unstyled {
            Style::new()
        } else {
            Style {
                font_weight: Some(600.0),
                ..Style::new()
            }
        });
        root = root.fill((
            headers,
            panels,
            glory::ui::view()
                .semantics(scope.bond(move || {
                    composite::metadata(Semantics {
                        hidden: accepted.get() != count,
                        ..Default::default()
                    })
                }))
                .fill(label),
            StepPanel {
                id: completed_id,
                label: completed_label_id,
                open: scope.bond(move || accepted.get() == count).into(),
                reference: references[count],
                restore,
                factory: self.completed.take(),
                policy: self.policy,
                unstyled: self.common.unstyled,
            },
        ));
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            root = root.on(kind, move |event| {
                crate::appearance::observe(&mut handlers, owner, event)
            });
        }
        root.show_in(scope)
    }
}
