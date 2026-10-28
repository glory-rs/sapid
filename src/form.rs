//! Host-owned submit/reset requests, with application-owned validation and I/O.
use glory::render::{Event, EventKind, FieldPart, FormAction, FormControl, NodeId, Semantics};
use glory::ui::{IntoChildren, Result, Widget};
use glory::{Scope, Vary};

use crate::appearance::{Common, common_methods, configure, observe};
use crate::composite::{self, Callback, Observers};

pub struct Form {
    key: String,
    common: Common,
    children: Option<glory::ui::Children>,
    submit: Option<Box<dyn FnMut(Option<NodeId>)>>,
    reset: Option<Box<dyn FnMut()>>,
}
pub fn form(key: impl Into<String>) -> Form {
    Form {
        key: key.into(),
        common: Common::default(),
        children: None,
        submit: None,
        reset: None,
    }
}
impl Form {
    common_methods!();
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.children = Some(children.into_children());
        self
    }
    /// A host request, not successful asynchronous validation or HTTP completion.
    pub fn on_submit(mut self, callback: impl FnMut(Option<NodeId>) + 'static) -> Self {
        self.submit = Some(Box::new(callback));
        self
    }
    /// Application reset policy runs before generic Reset observers.
    pub fn on_reset(mut self, callback: impl FnMut() + 'static) -> Self {
        self.reset = Some(Box::new(callback));
        self
    }
}
impl Widget for Form {
    fn build(&mut self, scope: &Scope) -> Result {
        let id = composite::id("form", &self.key, "root", "root")?;
        let enabled = self.common.enabled.clone();
        let mut root = configure!(
            self.common,
            glory::ui::view().field_part(FieldPart::Form),
            scope,
            "sapid-form",
            composite::column(18.0)
        );
        let metadata_enabled = enabled.clone();
        root = root.semantics(scope.bond(move || {
            composite::metadata(Semantics {
                field_part: Some(FieldPart::Form),
                id: Some(id.clone()),
                disabled: !metadata_enabled.get(),
                ..Default::default()
            })
        }));
        let submit = Callback::new(self.submit.take());
        let reset = Callback::new(
            self.reset
                .take()
                .map(|mut handler| Box::new(move |()| handler()) as Box<dyn FnMut(())>),
        );
        let mut handlers = std::mem::take(&mut self.common.handlers);
        let submit_observers =
            Observers::new(handlers.remove(&EventKind::Submit).unwrap_or_default());
        let reset_observers =
            Observers::new(handlers.remove(&EventKind::Reset).unwrap_or_default());
        let owner = scope.id();
        let submit_enabled = enabled.clone();
        root = root
            .on(EventKind::Submit, move |event| {
                if !submit_enabled.get() {
                    return;
                }
                if let Event::Submit { submitter } = event {
                    submit.emit(owner, submitter);
                }
                submit_observers.emit(owner, event);
            })
            .on(EventKind::Reset, move |event| {
                if !enabled.get() {
                    return;
                }
                reset.emit(owner, ());
                reset_observers.emit(owner, event);
            });
        for (kind, mut observers) in handlers {
            root = root.on(kind, move |event| observe(&mut observers, owner, event));
        }
        let controls_enabled = self.common.enabled.clone();
        let reset = glory::style::Edges::all(0.0);
        let base = if self.common.unstyled {
            glory::style::Style::new()
        } else {
            composite::column(18.0)
        };
        let mut controls = glory::ui::view()
            .field_part(FieldPart::Fieldset)
            .class("sapid-form-controls")
            .style(base.border_width(reset).padding(reset).margin(reset))
            .semantics(scope.bond(move || {
                composite::metadata(Semantics {
                    field_part: Some(FieldPart::Fieldset),
                    disabled: !controls_enabled.get(),
                    ..Default::default()
                })
            }));
        if let Some(children) = self.children.take() {
            controls = controls.fill(children);
        }
        root = root.fill(controls);
        root.show_in(scope)
    }
}

pub struct FormButton {
    common: Common,
    action: FormAction,
    label: Vary<String>,
    entry: Option<FormControl>,
}
pub fn submit_button() -> FormButton {
    form_button(FormAction::Submit, "Submit")
}
pub fn reset_button() -> FormButton {
    form_button(FormAction::Reset, "Reset")
}
fn form_button(action: FormAction, label: &str) -> FormButton {
    FormButton {
        common: Common::default(),
        action,
        label: label.into(),
        entry: None,
    }
}
impl FormButton {
    common_methods!();
    pub fn label(mut self, value: impl Into<Vary<String>>) -> Self {
        self.label = value.into();
        self
    }
    /// Successful submitter metadata, separate from the accessible button name.
    pub fn entry(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.entry = Some(FormControl {
            name: name.into(),
            value: Some(value.into()),
        });
        self
    }
}
impl Widget for FormButton {
    fn build(&mut self, scope: &Scope) -> Result {
        let class = match self.action {
            FormAction::Submit => "sapid-button sapid-form-button sapid-submit-button",
            FormAction::Reset => "sapid-button sapid-form-button sapid-reset-button",
        };
        let base = self.common.theme.control(self.common.size);
        let mut button = configure!(self.common, glory::ui::button(), scope, class, base)
            .label(self.label.clone())
            .enabled(self.common.enabled.clone())
            .semantics(composite::metadata(Semantics {
                form_action: Some(self.action),
                form_control: self.entry.clone(),
                ..Default::default()
            }));
        let owner = scope.id();
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            button = button.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        button.show_in(scope)
    }
}
