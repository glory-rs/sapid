//! Finite confirmation/checkpoint display and application-only request callbacks.
use super::confirmation_model::*;
use crate::appearance::{Common, common_methods, merge, observe};
use crate::composite::{self, Callback};
use glory::render::{Role, TextKind};
use glory::style::{
    Display, Edges, FlexDirection, FlexWrap, Length, Limit, Overflow, Style, WhiteSpace,
};
use glory::ui::{Result, Widget};
use glory::{Scope, Vary};

fn surface(common: &Common) -> Style {
    let mut style = common.theme.label(common.size);
    style.background = Some(common.theme.surface);
    style.padding = Some(Edges::all(12.));
    style.min_width = Some(Length::Px(0.));
    style.display = Some(Display::Flex);
    style.flex_direction = Some(FlexDirection::Column);
    style.gap = Some(8.);
    style
}
fn prose() -> Style {
    Style::new()
        .min_width(Length::Px(0.))
        .overflow_x(Overflow::Auto)
        .white_space(WhiteSpace::PreWrap)
}
fn actions() -> Style {
    Style::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .flex_wrap(FlexWrap::Wrap)
        .gap(8.)
        .min_width(Length::Px(0.))
}
fn action_label() -> Style {
    Style::new()
        .min_width(Length::Px(0.))
        .max_width(Limit::Percent(100.))
        .overflow_x(Overflow::Auto)
}
fn container(common: &mut Common, scope: &Scope, class: &str) -> glory::ui::View {
    let (classes, paint) = common.appearance(scope, class, surface(common));
    let mut root = glory::ui::view()
        .role(Role::Group)
        .class(classes)
        .style(scope.bond(move || {
            merge(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(8.)
                    .min_width(Length::Px(0.)),
                paint.get(),
            )
        }))
        .focus(common.focus);
    if let Some(name) = common.name.take() {
        root = root.name(name);
    }
    if let Some(reference) = common.node_ref {
        root = root.node_ref(reference);
    }
    root
}

pub struct Confirmation {
    common: Common,
    key: String,
    data: Vary<ConfirmationData>,
    labels: ConfirmationLabels,
    decision: Option<Box<dyn FnMut(ConfirmationRequest)>>,
}
pub fn confirmation(key: impl Into<String>) -> Confirmation {
    Confirmation {
        common: Common {
            name: Some("Confirmation request".into()),
            ..Default::default()
        },
        key: key.into(),
        data: ConfirmationData::default().into(),
        labels: Default::default(),
        decision: None,
    }
}
impl Confirmation {
    common_methods!();
    pub fn data(mut self, value: impl Into<Vary<ConfirmationData>>) -> Self {
        self.data = value.into();
        self
    }
    pub fn labels(mut self, value: ConfirmationLabels) -> Self {
        self.labels = value;
        self
    }
    pub fn on_decision(mut self, value: impl FnMut(ConfirmationRequest) + 'static) -> Self {
        self.decision = Some(Box::new(value));
        self
    }
}
impl Widget for Confirmation {
    fn build(&mut self, scope: &Scope) -> Result {
        composite::id("confirmation", &self.key, "root", "root")?;
        self.labels.validate()?;
        let accepted = composite::validated(scope, &self.data, ConfirmationData::validate)?;
        let available = self.decision.is_some();
        let callback = Callback::new(self.decision.take());
        let cleanup = callback.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let labels = self.labels.clone();
        let mut root = container(&mut self.common, scope, "sapid-confirmation").fill((
            glory::ui::text()
                .kind(TextKind::Heading(3))
                .class("sapid-confirmation-title")
                .style(prose())
                .content(scope.bond(move || accepted.with(|value| value.title.clone()))),
            glory::ui::text()
                .kind(TextKind::Paragraph)
                .class("sapid-confirmation-description")
                .style(prose())
                .content(scope.bond(move || accepted.with(|value| value.description.clone()))),
            glory::ui::text()
                .class("sapid-confirmation-status")
                .style(prose())
                .content(scope.bond(move || {
                    accepted.with(|value| labels.status(value.status).to_owned())
                })),
            glory::ui::text()
                .kind(TextKind::Paragraph)
                .class("sapid-confirmation-details")
                .style(prose())
                .content(scope.bond(move || accepted.with(|value| value.details.clone()))),
        ));
        let mut buttons = Vec::with_capacity(2);
        for (decision, label, class) in [
            (
                ConfirmationDecision::Approve,
                self.labels.approve.clone(),
                "sapid-confirmation-approve",
            ),
            (
                ConfirmationDecision::Reject,
                self.labels.reject.clone(),
                "sapid-confirmation-reject",
            ),
        ] {
            let current = callback.clone();
            let key = self.key.clone();
            let enabled = self.common.enabled.clone();
            let actual_enabled = enabled.clone();
            let owner = scope.id();
            let active = scope.bond(move || {
                available
                    && actual_enabled.get()
                    && accepted.with(|value| value.status == ConfirmationStatus::Pending)
            });
            let mut button = crate::button()
                .label(label)
                .class(class)
                .theme(self.common.theme.clone())
                .size(self.common.size)
                .style(action_label())
                .enabled(active)
                .on_press(move || {
                    if !enabled.get() {
                        return;
                    }
                    let request = accepted.with(|value| {
                        (value.status == ConfirmationStatus::Pending).then(|| {
                            ConfirmationRequest {
                                key: key.clone(),
                                revision: value.revision,
                                decision,
                            }
                        })
                    });
                    if let Some(request) = request {
                        current.emit(owner, request);
                    }
                });
            if self.common.unstyled {
                button = button.unstyled();
            }
            buttons.push(button);
        }
        root = root.fill(
            glory::ui::view()
                .class("sapid-confirmation-actions")
                .style(actions())
                .fill(buttons),
        );
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)
    }
}

pub struct Checkpoint {
    common: Common,
    key: String,
    data: Vary<CheckpointData>,
    labels: CheckpointLabels,
    restore: Option<Box<dyn FnMut(CheckpointRequest)>>,
}
pub fn checkpoint(key: impl Into<String>) -> Checkpoint {
    Checkpoint {
        common: Common {
            name: Some("Checkpoint".into()),
            ..Default::default()
        },
        key: key.into(),
        data: CheckpointData::default().into(),
        labels: Default::default(),
        restore: None,
    }
}
impl Checkpoint {
    common_methods!();
    pub fn data(mut self, value: impl Into<Vary<CheckpointData>>) -> Self {
        self.data = value.into();
        self
    }
    pub fn labels(mut self, value: CheckpointLabels) -> Self {
        self.labels = value;
        self
    }
    pub fn on_restore(mut self, value: impl FnMut(CheckpointRequest) + 'static) -> Self {
        self.restore = Some(Box::new(value));
        self
    }
}
impl Widget for Checkpoint {
    fn build(&mut self, scope: &Scope) -> Result {
        composite::id("checkpoint", &self.key, "root", "root")?;
        self.labels.validate()?;
        let accepted = composite::validated(scope, &self.data, CheckpointData::validate)?;
        let available = self.restore.is_some();
        let callback = Callback::new(self.restore.take());
        let cleanup = callback.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let labels = self.labels.clone();
        let enabled = self.common.enabled.clone();
        let actual_enabled = enabled.clone();
        let key = self.key.clone();
        let owner = scope.id();
        let mut button = crate::button()
            .label(self.labels.restore.clone())
            .class("sapid-checkpoint-restore")
            .theme(self.common.theme.clone())
            .size(self.common.size)
            .style(action_label())
            .enabled(scope.bond(move || {
                available
                    && actual_enabled.get()
                    && accepted.with(|value| value.status.can_request_restore())
            }))
            .on_press(move || {
                if !enabled.get() {
                    return;
                }
                let request = accepted.with(|value| {
                    value
                        .status
                        .can_request_restore()
                        .then(|| CheckpointRequest {
                            key: key.clone(),
                            revision: value.revision,
                        })
                });
                if let Some(request) = request {
                    callback.emit(owner, request);
                }
            });
        if self.common.unstyled {
            button = button.unstyled();
        }
        let mut root = container(&mut self.common, scope, "sapid-checkpoint").fill((
            glory::ui::text()
                .kind(TextKind::Heading(3))
                .class("sapid-checkpoint-title")
                .style(prose())
                .content(scope.bond(move || accepted.with(|value| value.title.clone()))),
            glory::ui::text()
                .kind(TextKind::Paragraph)
                .class("sapid-checkpoint-description")
                .style(prose())
                .content(scope.bond(move || accepted.with(|value| value.description.clone()))),
            glory::ui::text()
                .class("sapid-checkpoint-status")
                .style(prose())
                .content(scope.bond(move || {
                    accepted.with(|value| labels.status(value.status).to_owned())
                })),
            glory::ui::text()
                .kind(TextKind::Paragraph)
                .class("sapid-checkpoint-details")
                .style(prose())
                .content(scope.bond(move || accepted.with(|value| value.details.clone()))),
            button,
        ));
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)
    }
}
