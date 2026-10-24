//! Optional bounded JSON drafts; malformed syntax remains editable text.
use crate::appearance::{Common, observe};
use crate::composite::{self, Callback};
use crate::json_model::*;
use crate::state::Value;
use glory::render::{Event, EventKind, Role, Semantics};
use glory::style::{Family, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};
use std::{cell::Cell, rc::Rc};
pub struct JsonInput {
    key: String,
    common: Common,
    value: Value<String>,
    field: Option<crate::FieldControl>,
    required: Lotus<bool>,
    readonly: Lotus<bool>,
    invalid: Lotus<bool>,
    rows: u16,
    limits: JsonLimits,
    format: JsonFormat,
    formatter: bool,
    format_label: Lotus<String>,
    announce: bool,
    change: Option<Box<dyn FnMut(String)>>,
    commit: Option<Box<dyn FnMut(String)>>,
    validation: Option<Box<dyn FnMut(JsonValidity)>>,
    formatted: Option<Box<dyn FnMut(String)>>,
    format_error: Option<Box<dyn FnMut(JsonError)>>,
}
pub fn json_input(key: impl Into<String>) -> JsonInput {
    JsonInput {
        key: key.into(),
        common: Common::default(),
        value: Value::default(),
        field: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
        rows: 6,
        limits: JsonLimits::default(),
        format: JsonFormat::default(),
        formatter: true,
        format_label: "Format JSON".into(),
        announce: false,
        change: None,
        commit: None,
        validation: None,
        formatted: None,
        format_error: None,
    }
}
impl JsonInput {
    pub fn value(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.value.default = value.into();
        self
    }
    pub fn field(mut self, value: crate::FieldControl) -> Self {
        self.common.theme = value.theme.clone();
        self.common.size = value.size;
        self.common.unstyled = value.unstyled;
        self.field = Some(value);
        self
    }
    pub fn name(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn size(mut self, value: crate::Size) -> Self {
        self.common.size = value;
        self
    }
    pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn enabled(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.common.enabled = value.into();
        self
    }
    pub fn readonly(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn required(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.required = value.into();
        self
    }
    pub fn invalid(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.invalid = value.into();
        self
    }
    pub fn rows(mut self, value: u16) -> Self {
        self.rows = value;
        self
    }
    pub fn limits(mut self, value: JsonLimits) -> Self {
        self.limits = value;
        self
    }
    pub fn format(mut self, value: JsonFormat) -> Self {
        self.format = value;
        self
    }
    pub fn show_formatter(mut self, value: bool) -> Self {
        self.formatter = value;
        self
    }
    pub fn format_label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.format_label = value.into();
        self
    }
    pub fn announce_errors(mut self, value: bool) -> Self {
        self.announce = value;
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<glory::render::NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
    pub fn on_value_change(mut self, value: impl FnMut(String) + 'static) -> Self {
        self.change = Some(Box::new(value));
        self
    }
    pub fn on_commit(mut self, value: impl FnMut(String) + 'static) -> Self {
        self.commit = Some(Box::new(value));
        self
    }
    pub fn on_validation(mut self, value: impl FnMut(JsonValidity) + 'static) -> Self {
        self.validation = Some(Box::new(value));
        self
    }
    pub fn on_format(mut self, value: impl FnMut(String) + 'static) -> Self {
        self.formatted = Some(Box::new(value));
        self
    }
    pub fn on_format_error(mut self, value: impl FnMut(JsonError) + 'static) -> Self {
        self.format_error = Some(Box::new(value));
        self
    }
    pub fn on(mut self, kind: EventKind, value: impl FnMut(Event) + 'static) -> Self {
        self.common
            .handlers
            .entry(kind)
            .or_default()
            .push(Box::new(value));
        self
    }
}
fn draft(value: &str, limits: JsonLimits) -> Result {
    if value.len() > limits.bytes || value.contains(['\0', '\r']) {
        Err(glory::render::Error::InvalidProps(
            "JSON drafts require canonical text within their byte budget",
        )
        .into())
    } else {
        Ok(())
    }
}
fn label(value: &str) -> Result {
    if value.trim().is_empty() || value.len() > 256 || value.contains(['\0', '\r', '\n']) {
        Err(glory::render::Error::InvalidProps(
            "JSON names require nonempty canonical single-line text within 256 bytes",
        )
        .into())
    } else {
        Ok(())
    }
}
impl Widget for JsonInput {
    fn build(&mut self, scope: &Scope) -> Result {
        self.limits
            .validate()
            .map_err(|_| glory::render::Error::InvalidProps("invalid JSON limits"))?;
        if self.common.name.is_none() && self.field.is_none() {
            self.common.name = Some("JSON".into())
        }
        let root_id = composite::id("json-input", &self.key, "root", "root")?;
        let error_id = composite::id("json-input", &self.key, "error", "root")?;
        let control = std::mem::take(&mut self.value).mount(scope);
        let limits = self.limits;
        let accepted =
            composite::validated(scope, &control.source, move |value| draft(value, limits))?;
        let status = scope.cage(accepted.with(|value| validate_json(value, limits)));
        let format_failure = scope.cage(None::<JsonError>);
        let parse_message = scope.bond(move || match status.get() {
            JsonValidity::Invalid(error) => error.to_string(),
            _ => String::new(),
        });
        let validation = Callback::new(self.validation.take());
        let change = Callback::new(self.change.take());
        let formatted = Callback::new(self.formatted.take());
        let format_error = Callback::new(self.format_error.take());
        let owner = scope.id();
        let weak = scope.downgrade();
        let observed = validation.clone();
        let mut previous = None;
        let mut previous_version = None;
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let next = accepted.try_with(|value| validate_json(value, limits))?;
                let version = accepted.try_version()?;
                if previous_version != Some(version) {
                    format_failure.try_revise(|value| *value = None)?;
                    previous_version = Some(version);
                }
                if previous.as_ref() != Some(&next) {
                    status.try_revise(|value| *value = next.clone())?;
                    previous = Some(next.clone());
                    glory::core::untracked(|| observed.emit(owner, next));
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error)
            }
        });
        let cleanup = validation.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let cleanup = change.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let cleanup = formatted.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let cleanup = format_error.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let composing = scope.cage(false);
        let composing_now = Rc::new(Cell::new(false));
        let live_composing = composing_now.clone();
        let enabled = self
            .field
            .as_ref()
            .map(|field| field.enabled(scope, self.common.enabled.clone()))
            .unwrap_or_else(|| self.common.enabled.clone());
        let readonly = self
            .field
            .as_ref()
            .map(|field| field.readonly_state(scope, self.readonly.clone()))
            .unwrap_or_else(|| self.readonly.clone());
        let invalid = self.invalid.clone();
        let error_state = scope
            .bond(move || invalid.get() || matches!(status.get(), JsonValidity::Invalid(_)));
        let custom = self.common.style.clone();
        let unstyled = self.common.unstyled;
        let style = scope.bond(move || {
            let mut style = custom.get();
            if !unstyled && style.font_family.is_none() {
                style.font_family = Some(vec![Family::Mono])
            }
            style
        });
        let mut input = crate::textarea()
            .theme(self.common.theme.clone())
            .size(self.common.size)
            .rows(self.rows)
            .value(accepted)
            .enabled(enabled.clone())
            .readonly(self.readonly.clone())
            .required(self.required.clone())
            .invalid(error_state)
            .class("sapid-json-editor")
            .style(style)
            .additional_error(error_id.clone(), parse_message.into());
        if self.common.unstyled {
            input = input.unstyled()
        }
        if let Some(field) = self.field.take() {
            input = input.field(field)
        }
        // A surrounding Field retains its label relation unless a name was explicit.
        if let Some(name) = self.common.name.take() {
            input = input.name(composite::validated(scope, &name, |value| label(value))?)
        }
        if let Some(reference) = self.common.node_ref {
            input = input.node_ref(reference)
        }
        let editing = control.clone();
        let edited = change.clone();
        let weak = scope.downgrade();
        input = input.on_value_change(move |value| {
            let Some(scope) = weak.upgrade() else { return };
            let result = draft(&value, limits).and_then(|()| editing.accept(&value));
            match result {
                Ok(()) => edited.emit(owner, value),
                Err(error) => scope.report_error(error),
            }
        });
        if let Some(mut callback) = self.commit.take() {
            let weak = scope.downgrade();
            input = input.on_commit(move |value| {
                let Some(scope) = weak.upgrade() else { return };
                if let Err(error) = draft(&value, limits) {
                    scope.report_error(error)
                } else {
                    callback(value)
                }
            })
        }
        let mut composition_observers = self
            .common
            .handlers
            .remove(&EventKind::Composition)
            .unwrap_or_default();
        input = input.on(EventKind::Composition, move |event| {
            if let Event::Composition { active } = event {
                live_composing.set(active);
                composing.revise(|value| *value = active);
            }
            observe(&mut composition_observers, owner, event)
        });
        for (kind, mut callbacks) in std::mem::take(&mut self.common.handlers) {
            input = input.on(kind, move |event| observe(&mut callbacks, owner, event))
        }
        let extra = self.common.class.clone();
        let classes = format!(
            "sapid-json-input {}{}",
            self.common.theme.class(),
            if self.common.unstyled {
                " sapid-unstyled"
            } else {
                ""
            }
        );
        let mut root = glory::ui::view()
            .class(scope.bond(move || format!("{classes} {}", extra.get())))
            .style(composite::column(6.))
            .semantics(composite::metadata(Semantics {
                id: Some(root_id),
                ..Default::default()
            }))
            .fill(input);
        let mut message = glory::ui::view()
            .class("sapid-json-validation")
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    id: Some(error_id.clone()),
                    hidden: parse_message.with(|value| value.is_empty()),
                    ..Default::default()
                }))
            }))
            .fill(glory::ui::text().content(parse_message));
        if self.announce {
            message = message.role(Role::Status)
        }
        root = root.fill(message);
        root = root.fill(glory::ui::text().class("sapid-json-format-error").content(
            scope.bond(move || {
                format_failure
                    .with(|error| error.as_ref().map(ToString::to_string).unwrap_or_default())
            }),
        ));
        if self.formatter {
            let format = self.format;
            let ready = enabled.clone();
            let read = readonly.clone();
            let check = scope.bond(move || {
                ready.get()
                    && !read.get()
                    && !composing.get()
                    && matches!(status.get(), JsonValidity::Valid(_))
            });
            let label = composite::validated(scope, &self.format_label, |value| label(value))?;
            let ready = enabled;
            let read = readonly;
            let weak = scope.downgrade();
            let button = crate::button()
                .theme(self.common.theme.clone())
                .label(label)
                .enabled(check)
                .on_press(move || {
                    let Some(scope) = weak.upgrade() else { return };
                    if !ready.get() || read.get() || composing_now.get() {
                        return;
                    }
                    let result = accepted.with(|value| format_json(value, format, limits));
                    match result {
                        Ok(value) => {
                            format_failure.revise(|failure| *failure = None);
                            if let Err(error) = control.accept(&value) {
                                scope.report_error(error);
                                return;
                            }
                            change.emit(owner, value.clone());
                            formatted.emit(owner, value)
                        }
                        Err(error) => {
                            format_failure.revise(|failure| *failure = Some(error.clone()));
                            format_error.emit(owner, error)
                        }
                    }
                });
            root = root.fill(if self.common.unstyled {
                button.unstyled()
            } else {
                button
            });
        }
        root.show_in(scope)
    }
}
