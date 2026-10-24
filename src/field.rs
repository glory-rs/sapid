use glory::render::{FieldPart, FormControl, SemanticId, Semantics};
use glory::style::{BorderStyle, Edges};
use glory::ui::{IntoChildren, Result, Widget};
use glory::{Cage, Lotus, Scope};

use crate::appearance::{Common, common_methods, configure};
use crate::composite;

/// Explicit field context. Pass it to one TextField/Checkbox with `.field(...)`;
/// clones contain only owner-local reactive handles and stable semantic names.
#[derive(Clone)]
pub struct FieldControl {
    pub id: SemanticId,
    form_control: Option<Cage<FormControl>>,
    label: Option<SemanticId>,
    description: Option<SemanticId>,
    error_id: Option<SemanticId>,
    error: Lotus<String>,
    required: Lotus<bool>,
    readonly: Lotus<bool>,
    invalid: Lotus<bool>,
    disabled: Lotus<bool>,
    pub(crate) theme: crate::Theme,
    pub(crate) size: crate::Size,
    pub(crate) unstyled: bool,
}
impl FieldControl {
    pub(crate) fn readonly_state(&self, scope: &Scope, readonly: Lotus<bool>) -> Lotus<bool> {
        let inherited = self.readonly.clone();
        scope.bond(move || readonly.get() || inherited.get()).into()
    }
    pub(crate) fn enabled(&self, scope: &Scope, enabled: Lotus<bool>) -> Lotus<bool> {
        let disabled = self.disabled.clone();
        scope.bond(move || enabled.get() && !disabled.get()).into()
    }
    pub(crate) fn metadata(
        &self,
        scope: &Scope,
        named: bool,
        required: Lotus<bool>,
        readonly: Lotus<bool>,
        invalid: Lotus<bool>,
    ) -> Lotus<Option<Box<Semantics>>> {
        let field = self.clone();
        scope
            .bond(move || {
                let error = field.error.with(|error| !error.is_empty());
                composite::metadata(Semantics {
                    id: Some(field.id.clone()),
                    form_control: field.form_control.as_ref().map(|control| control.get()),
                    labelled_by: if named { None } else { field.label.clone() },
                    described_by: field.description.clone(),
                    error_message: if error { field.error_id.clone() } else { None },
                    required: required.get() || field.required.get(),
                    readonly: readonly.get() || field.readonly.get(),
                    invalid: invalid.get() || field.invalid.get() || error,
                    ..Default::default()
                })
            })
            .into()
    }
}

type ControlFactory = Box<dyn FnMut(FieldControl) -> Box<dyn Widget>>;
fn accepted_form_control(
    scope: &Scope,
    name: Option<Lotus<String>>,
    value: Option<Lotus<String>>,
) -> Result<Option<Cage<FormControl>>> {
    let Some(name) = name else { return Ok(None) };
    fn read(name: &Lotus<String>, value: &Option<Lotus<String>>) -> Result<FormControl> {
        name.try_with(|name| {
            if name.len() > 256 || name.contains(['\0', '\r', '\n']) {
                return Err(glory::render::Error::InvalidProps(
                    "invalid successful-control name",
                )
                .into());
            }
            if let Some(value) = value {
                value.try_with(|value| {
                    if value.contains(['\0', '\r']) {
                        return Err(glory::render::Error::InvalidProps(
                            "invalid successful-control value",
                        )
                        .into());
                    }
                    Ok(FormControl {
                        name: name.clone(),
                        value: Some(value.clone()),
                    })
                })?
            } else {
                Ok(FormControl {
                    name: name.clone(),
                    value: None,
                })
            }
        })?
    }
    // Raw Lotus values move into the observer; invalid strings are checked by
    // borrow before either name/value is cloned into the accepted pair.
    let accepted = scope.cage(read(&name, &value)?);
    if !name.is_reactive() && !value.as_ref().is_some_and(Lotus::is_reactive) {
        return Ok(Some(accepted));
    }
    let weak = scope.downgrade();
    scope.structural(move || {
        let Some(scope) = weak.upgrade() else { return };
        let result = (|| -> Result {
            let next = read(&name, &value)?;
            if accepted.try_with(|old| old != &next)? {
                accepted.try_revise(|old| *old = next)?
            }
            Ok(())
        })();
        if let Err(error) = result {
            scope.report_error(error)
        }
    });
    Ok(Some(accepted))
}
pub struct Field {
    key: String,
    common: Common,
    form_name: Option<Lotus<String>>,
    form_value: Option<Lotus<String>>,
    label: Option<Lotus<String>>,
    description: Option<Lotus<String>>,
    error: Option<Lotus<String>>,
    required: Lotus<bool>,
    readonly: Lotus<bool>,
    invalid: Lotus<bool>,
    control: Option<ControlFactory>,
}
pub fn field(key: impl Into<String>) -> Field {
    Field {
        key: key.into(),
        common: Common::default(),
        form_name: None,
        form_value: None,
        label: None,
        description: None,
        error: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
        control: None,
    }
}
impl Field {
    common_methods!();
    /// Successful-control name, independent of accessible labels and names.
    pub fn form_name(mut self, name: impl Into<Lotus<String>>) -> Self {
        self.form_name = Some(name.into());
        self
    }
    /// Explicit Check value. Input values always come from their InputProps.
    pub fn form_value(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.form_value = Some(value.into());
        self
    }
    pub fn label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.label = Some(value.into());
        self
    }
    pub fn description(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.description = Some(value.into());
        self
    }
    pub fn error(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.error = Some(value.into());
        self
    }
    pub fn required(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.required = value.into();
        self
    }
    pub fn readonly(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn invalid(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.invalid = value.into();
        self
    }
    pub fn control<W: Widget>(
        mut self,
        mut factory: impl FnMut(FieldControl) -> W + 'static,
    ) -> Self {
        self.control = Some(Box::new(move |context| Box::new(factory(context))));
        self
    }
}
impl Widget for Field {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.form_value.is_some() && self.form_name.is_none() {
            return Err(
                glory::render::Error::InvalidProps("form value requires a form name").into(),
            );
        }
        let mut factory = self
            .control
            .take()
            .ok_or(glory::render::Error::InvalidProps(
                "field requires one control factory",
            ))?;
        let id = composite::id("field", &self.key, "control", "root")?;
        let label_id = composite::id("field", &self.key, "label", "root")?;
        let description_id = composite::id("field", &self.key, "description", "root")?;
        let error_id = composite::id("field", &self.key, "error", "root")?;
        let enabled = self.common.enabled.clone();
        let form_control =
            accepted_form_control(scope, self.form_name.take(), self.form_value.take())?;
        let context = FieldControl {
            id: id.clone(),
            form_control,
            label: self.label.as_ref().map(|_| label_id.clone()),
            description: self.description.as_ref().map(|_| description_id.clone()),
            error_id: self.error.as_ref().map(|_| error_id.clone()),
            error: self.error.clone().unwrap_or_else(|| "".into()),
            required: self.required.clone(),
            readonly: self.readonly.clone(),
            invalid: self.invalid.clone(),
            disabled: scope.bond(move || !enabled.get()).into(),
            theme: self.common.theme.clone(),
            size: self.common.size,
            unstyled: self.common.unstyled,
        };
        let label = self.label.take();
        let description = self.description.take();
        let error = self.error.take();
        let theme = self.common.theme.clone();
        let size = self.common.size;
        let unstyled = self.common.unstyled;
        let required = self.required.clone();
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-field",
            composite::column(6.0)
        );
        root = root.fill(glory::ui::widget(move |scope| {
            if let Some(content) = label.clone() {
                let mut part = label_part(label_id.clone(), id.clone())
                    .content(content)
                    .theme(theme.clone())
                    .size(size)
                    .required(required.clone());
                if unstyled {
                    part = part.unstyled();
                }
                part.show_in(scope)?;
            }
            if let Some(content) = description.clone() {
                let mut part = field_description(description_id.clone())
                    .content(content)
                    .theme(theme.clone())
                    .size(size);
                if unstyled {
                    part = part.unstyled();
                }
                part.show_in(scope)?;
            }
            factory(context.clone()).show_in(scope)?;
            if let Some(content) = error.clone() {
                let mut part = field_error(error_id.clone())
                    .content(content)
                    .theme(theme.clone())
                    .size(size);
                if unstyled {
                    part = part.unstyled();
                }
                part.show_in(scope)?;
            }
            Ok(())
        }));
        finish(&mut self.common, root, scope)
    }
}

pub struct Label {
    common: Common,
    id: SemanticId,
    target: SemanticId,
    content: Lotus<String>,
    required: Lotus<bool>,
}
pub fn label(id: SemanticId, target: SemanticId) -> Label {
    label_part(id, target)
}
fn label_part(id: SemanticId, target: SemanticId) -> Label {
    Label {
        common: Common::default(),
        id,
        target,
        content: "".into(),
        required: false.into(),
    }
}
impl Label {
    common_methods!();
    pub fn content(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.content = value.into();
        self
    }
    pub fn required(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.required = value.into();
        self
    }
}
impl Widget for Label {
    fn build(&mut self, scope: &Scope) -> Result {
        let required = self.required.clone();
        let classes = self.common.class.clone();
        self.common.class = scope
            .bond(move || {
                format!(
                    "{} {}",
                    classes.get(),
                    if required.get() { "is-required" } else { "" }
                )
            })
            .into();
        let root = configure!(
            self.common,
            glory::ui::view().field_part(FieldPart::Label),
            scope,
            "sapid-field-label",
            self.common.theme.label(self.common.size)
        )
        .semantics(composite::metadata(Semantics {
            field_part: Some(FieldPart::Label),
            id: Some(self.id.clone()),
            label_for: Some(self.target.clone()),
            ..Default::default()
        }))
        .fill(glory::ui::text().content(self.content.clone()));
        finish(&mut self.common, root, scope)
    }
}

pub struct Description {
    common: Common,
    id: SemanticId,
    content: Lotus<String>,
    error: bool,
}
pub struct FieldError(Description);
pub fn field_description(id: SemanticId) -> Description {
    Description {
        common: Common::default(),
        id,
        content: "".into(),
        error: false,
    }
}
pub fn description(id: SemanticId) -> Description {
    field_description(id)
}
pub fn field_error(id: SemanticId) -> FieldError {
    FieldError(Description {
        error: true,
        ..field_description(id)
    })
}
impl Description {
    common_methods!();
    pub fn content(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.content = value.into();
        self
    }
}
impl FieldError {
    pub fn content(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.0.content = value.into();
        self
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.0.common.theme = value;
        self
    }
    pub fn size(mut self, value: crate::Size) -> Self {
        self.0.common.size = value;
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.0.common.unstyled = true;
        self
    }
}
impl Widget for FieldError {
    fn build(&mut self, scope: &Scope) -> Result {
        self.0.build(scope)
    }
}
impl Widget for Description {
    fn build(&mut self, scope: &Scope) -> Result {
        let content = self.content.clone();
        let id = self.id.clone();
        let mut base = self.common.theme.description(self.common.size);
        if self.error {
            base.color = Some(self.common.theme.danger);
        }
        let root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            if self.error {
                "sapid-field-error"
            } else {
                "sapid-field-description"
            },
            base
        )
        .semantics(scope.bond(move || {
            composite::metadata(Semantics {
                id: Some(id.clone()),
                hidden: content.get().is_empty(),
                ..Default::default()
            })
        }))
        .fill(glory::ui::text().content(self.content.clone()));
        finish(&mut self.common, root, scope)
    }
}

pub struct Fieldset {
    common: Common,
    key: String,
    legend: Lotus<String>,
    disabled: Lotus<bool>,
    children: Option<glory::ui::Children>,
}
pub fn fieldset(key: impl Into<String>) -> Fieldset {
    Fieldset {
        common: Common::default(),
        key: key.into(),
        legend: "".into(),
        disabled: false.into(),
        children: None,
    }
}
impl Fieldset {
    common_methods!();
    pub fn legend(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.legend = value.into();
        self
    }
    pub fn disabled(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.disabled = value.into();
        self
    }
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.children = Some(children.into_children());
        self
    }
}
impl Widget for Fieldset {
    fn build(&mut self, scope: &Scope) -> Result {
        let id = composite::id("fieldset", &self.key, "root", "root")?;
        let legend_id = composite::id("fieldset", &self.key, "legend", "root")?;
        let disabled = self.disabled.clone();
        let enabled = self.common.enabled.clone();
        let mut base = composite::column(18.0)
            .padding(Edges::all(20.0))
            .border_width(Edges::all(1.0))
            .border_style(BorderStyle::Solid)
            .border_color(self.common.theme.border)
            .radius(self.common.theme.radius);
        base.background = Some(self.common.theme.surface);
        let legend = glory::ui::view()
            .field_part(FieldPart::Legend)
            .class(if self.common.unstyled {
                "sapid-fieldset-legend sapid-unstyled"
            } else {
                "sapid-fieldset-legend"
            })
            .style(if self.common.unstyled {
                glory::style::Style::new()
            } else {
                self.common.theme.label(self.common.size)
            })
            .semantics(composite::metadata(Semantics {
                field_part: Some(FieldPart::Legend),
                id: Some(legend_id.clone()),
                ..Default::default()
            }))
            .fill(glory::ui::text().content(self.legend.clone()));
        let mut root = configure!(
            self.common,
            glory::ui::view().field_part(FieldPart::Fieldset),
            scope,
            "sapid-fieldset",
            base
        )
        .semantics(scope.bond(move || {
            composite::metadata(Semantics {
                field_part: Some(FieldPart::Fieldset),
                id: Some(id.clone()),
                labelled_by: Some(legend_id.clone()),
                disabled: disabled.get() || !enabled.get(),
                ..Default::default()
            })
        }))
        .fill(legend);
        if let Some(children) = self.children.take() {
            root = root.fill(children);
        }
        finish(&mut self.common, root, scope)
    }
}
fn finish(common: &mut Common, mut root: glory::ui::View, scope: &Scope) -> Result {
    let owner = scope.id();
    for (kind, handlers) in std::mem::take(&mut common.handlers) {
        let observers = composite::Observers::new(handlers);
        root = root.on(kind, move |event| observers.emit(owner, event));
    }
    root.show_in(scope)
}

pub(crate) fn control_metadata(
    scope: &Scope,
    field: Option<&FieldControl>,
    named: bool,
    required: Lotus<bool>,
    readonly: Lotus<bool>,
    invalid: Lotus<bool>,
) -> Lotus<Option<Box<Semantics>>> {
    if let Some(field) = field {
        return field.metadata(scope, named, required, readonly, invalid);
    }
    if !required.is_reactive() && !readonly.is_reactive() && !invalid.is_reactive() {
        let (required, readonly, invalid) = (required.get(), readonly.get(), invalid.get());
        return if required || readonly || invalid {
            composite::metadata(Semantics {
                required,
                readonly,
                invalid,
                ..Default::default()
            })
            .into()
        } else {
            None.into()
        };
    }
    scope
        .bond(move || {
            composite::metadata(Semantics {
                required: required.get(),
                readonly: readonly.get(),
                invalid: invalid.get(),
                ..Default::default()
            })
        })
        .into()
}
