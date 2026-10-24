//! Native selection with bounded static options and owner-local controlled state.
use std::rc::Rc;

use glory::render::{
    Event, EventKind, FormControl, SelectEntry, SelectGroup, SelectOption, SelectProps,
    SemanticId,
};
use glory::ui::{Result, Widget};
use glory::{Lotus, Scope};

use crate::appearance::{Common, common_methods, configure, observe};
use crate::composite;
use crate::state::{Dirty, Value};

pub const MAX_NATIVE_SELECT_OPTIONS: usize = glory::render::MAX_SELECT_OPTIONS;

pub struct NativeOption {
    key: String,
    label: String,
    value: Option<String>,
    disabled: bool,
}
pub fn select_option(key: impl Into<String>, label: impl Into<String>) -> NativeOption {
    NativeOption {
        key: key.into(),
        label: label.into(),
        value: None,
        disabled: false,
    }
}
impl NativeOption {
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub(crate) fn lower(self) -> Result<SelectOption> {
        if self.key.len() > 64 {
            return Err(glory::render::Error::InvalidProps("native option key budget").into());
        }
        Ok(SelectOption {
            key: SemanticId::new(self.key.clone())?,
            label: self.label,
            value: self.value.unwrap_or(self.key),
            disabled: self.disabled,
        })
    }
}

pub struct NativeOptionGroup {
    pub(crate) label: String,
    pub(crate) disabled: bool,
    pub(crate) options: Vec<NativeOption>,
}
pub fn select_group(label: impl Into<String>) -> NativeOptionGroup {
    NativeOptionGroup {
        label: label.into(),
        disabled: false,
        options: Vec::new(),
    }
}
impl NativeOptionGroup {
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn option(mut self, value: NativeOption) -> Self {
        if self.options.len() <= MAX_NATIVE_SELECT_OPTIONS {
            self.options.push(value);
        }
        self
    }
    pub fn options(mut self, values: impl IntoIterator<Item = NativeOption>) -> Self {
        self.options.extend(
            values
                .into_iter()
                .take((MAX_NATIVE_SELECT_OPTIONS + 1).saturating_sub(self.options.len())),
        );
        self
    }
}
enum Entry {
    Option(NativeOption),
    Group(NativeOptionGroup),
}

pub struct NativeSelect {
    key: String,
    common: Common,
    entries: Vec<Entry>,
    count: usize,
    multiple: bool,
    single: Value<Option<String>>,
    many: Value<Vec<String>>,
    on_value: Option<Box<dyn FnMut(Option<String>)>>,
    on_values: Option<Box<dyn FnMut(Vec<String>)>>,
    rows: Option<u16>,
    placeholder: String,
    field: Option<crate::FieldControl>,
    form_name: Option<Lotus<String>>,
    required: Lotus<bool>,
    readonly: Lotus<bool>,
    invalid: Lotus<bool>,
}
pub fn native_select(key: impl Into<String>) -> NativeSelect {
    NativeSelect {
        key: key.into(),
        common: Common::default(),
        entries: Vec::new(),
        count: 0,
        multiple: false,
        single: Value::default(),
        many: Value::default(),
        on_value: None,
        on_values: None,
        rows: None,
        placeholder: String::new(),
        field: None,
        form_name: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
    }
}
impl NativeSelect {
    common_methods!();
    pub fn label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn field(mut self, field: crate::FieldControl) -> Self {
        self.common.theme = field.theme.clone();
        self.common.size = field.size;
        self.common.unstyled = field.unstyled;
        self.field = Some(field);
        self
    }
    pub fn form_name(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.form_name = Some(value.into());
        self
    }
    pub fn required(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.required = value.into();
        self
    }
    /// HTML select has no readonly mode; true is rejected before host mutation.
    pub fn readonly(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn invalid(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.invalid = value.into();
        self
    }
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = value.into();
        self
    }
    pub fn rows(mut self, value: u16) -> Self {
        self.rows = Some(value);
        self
    }
    pub fn multiple(mut self, value: bool) -> Self {
        self.multiple = value;
        self
    }
    pub fn value(mut self, value: impl Into<Lotus<Option<String>>>) -> Self {
        self.single.source = Some(value.into());
        self
    }
    pub fn values(mut self, value: impl Into<Lotus<Vec<String>>>) -> Self {
        self.multiple = true;
        self.many.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: Option<String>) -> Self {
        self.single.default = value;
        self
    }
    pub fn default_values(mut self, value: impl IntoIterator<Item = String>) -> Self {
        self.multiple = true;
        self.many.default = value
            .into_iter()
            .take(MAX_NATIVE_SELECT_OPTIONS + 1)
            .collect();
        self
    }
    pub fn on_value_change(mut self, callback: impl FnMut(Option<String>) + 'static) -> Self {
        self.on_value = Some(Box::new(callback));
        self
    }
    pub fn on_values_change(mut self, callback: impl FnMut(Vec<String>) + 'static) -> Self {
        self.on_values = Some(Box::new(callback));
        self
    }
    pub fn option(mut self, option: NativeOption) -> Self {
        if self.count <= MAX_NATIVE_SELECT_OPTIONS {
            self.entries.push(Entry::Option(option));
            self.count += 1;
        }
        self
    }
    pub fn options(mut self, options: impl IntoIterator<Item = NativeOption>) -> Self {
        for option in options
            .into_iter()
            .take((MAX_NATIVE_SELECT_OPTIONS + 1).saturating_sub(self.count))
        {
            self.entries.push(Entry::Option(option));
            self.count += 1;
        }
        self
    }
    pub fn group(mut self, group: NativeOptionGroup) -> Self {
        if self.count <= MAX_NATIVE_SELECT_OPTIONS
            && self.entries.len()
                <= MAX_NATIVE_SELECT_OPTIONS + glory::render::MAX_SELECT_GROUPS
        {
            self.count += group.options.len();
            self.entries.push(Entry::Group(group));
        }
        self
    }
}

fn canonical(config: &SelectProps, value: &[String]) -> Result<Vec<SemanticId>> {
    if value.len()
        > if config.multiple {
            MAX_NATIVE_SELECT_OPTIONS
        } else {
            1
        }
        || value.iter().any(|key| key.len() > 64)
    {
        return Err(glory::render::Error::InvalidProps(
            "native select selection exceeds budget",
        )
        .into());
    }
    let requested: std::collections::HashSet<_> = value.iter().map(String::as_str).collect();
    if requested.len() != value.len() {
        return Err(glory::render::Error::InvalidProps("duplicate native select key").into());
    }
    let result: Vec<_> = config
        .options()
        .filter(|(option, _)| requested.contains(option.key.as_str()))
        .map(|(option, _)| option.key.clone())
        .collect();
    if result.len() != value.len() {
        return Err(glory::render::Error::InvalidProps("unknown native select key").into());
    }
    Ok(result)
}

impl Widget for NativeSelect {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.count > MAX_NATIVE_SELECT_OPTIONS
            || self.entries.len() > MAX_NATIVE_SELECT_OPTIONS + glory::render::MAX_SELECT_GROUPS
            || self.multiple && (self.single.source.is_some() || self.single.default.is_some())
            || !self.multiple && (self.many.source.is_some() || !self.many.default.is_empty())
        {
            return Err(glory::render::Error::InvalidProps(
                "native select configuration/cardinality mismatch",
            )
            .into());
        }
        let id = composite::id("native-select", &self.key, "control", "root")?;
        let entries: Result<Vec<_>> = std::mem::take(&mut self.entries)
            .into_iter()
            .map(|entry| match entry {
                Entry::Option(value) => Ok(SelectEntry::Option(value.lower()?)),
                Entry::Group(group) => Ok(SelectEntry::Group(SelectGroup {
                    label: group.label,
                    disabled: group.disabled,
                    options: group
                        .options
                        .into_iter()
                        .map(NativeOption::lower)
                        .collect::<Result<Vec<_>>>()?,
                })),
            })
            .collect();
        let config = Rc::new(SelectProps {
            entries: Rc::new(entries?),
            multiple: self.multiple,
            rows: self.rows,
            placeholder: std::mem::take(&mut self.placeholder),
            ..Default::default()
        });
        config.validate()?;
        let form_name = composite::form_name(scope, &self.form_name)?;
        let single = (!self.multiple).then(|| std::mem::take(&mut self.single).mount(scope));
        let many = self
            .multiple
            .then(|| std::mem::take(&mut self.many).mount(scope));
        let selection: Lotus<Vec<String>> = if let Some(control) = &single {
            let validate = config.clone();
            let accepted = composite::validated(scope, &control.source, move |value| {
                if let Some(value) = value {
                    canonical(&validate, std::slice::from_ref(value))?;
                }
                Ok(())
            })?;
            scope
                .bond(move || accepted.get().into_iter().collect())
                .into()
        } else {
            let validate = config.clone();
            let accepted = composite::validated(
                scope,
                &many.as_ref().expect("multiple control").source,
                move |value| canonical(&validate, value).map(|_| ()),
            )?;
            accepted.into()
        };
        let initial = selection.try_with(|value| canonical(&config, value))??;
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let enabled = self
            .field
            .as_ref()
            .map(|field| field.enabled(scope, self.common.enabled.clone()))
            .unwrap_or_else(|| self.common.enabled.clone());
        let metadata = crate::field::control_metadata(
            scope,
            self.field.as_ref(),
            self.common.name.is_some(),
            self.required.clone(),
            self.readonly.clone(),
            self.invalid.clone(),
        );
        let field_id = self.field.as_ref().map(|field| field.id.clone());
        let metadata = scope.bond(move || {
            let mut value = metadata.get().unwrap_or_default();
            if value.id.is_none() {
                value.id = Some(field_id.clone().unwrap_or_else(|| id.clone()));
            }
            if let Some(name) = &form_name {
                value.form_control = Some(FormControl {
                    name: name.get(),
                    value: None,
                });
            }
            Some(value)
        });
        let dirty = Dirty::new(scope);
        let mut props = (*config).clone();
        props.selection.keys = initial.clone();
        let mut inner = configure!(
            self.common,
            glory::ui::select().configuration(props),
            scope,
            "sapid-native-select",
            self.common.theme.control(self.common.size)
        )
        .enabled(enabled)
        .semantics(metadata)
        .node_ref(reference);
        let mut on_value = self.on_value.take();
        let mut on_values = self.on_values.take();
        let mut handlers = self
            .common
            .handlers
            .remove(&EventKind::SelectChanged)
            .unwrap_or_default();
        let pulse = dirty.clone();
        let weak = scope.downgrade();
        let owner = scope.id();
        let validate = config.clone();
        inner = inner.on(EventKind::SelectChanged, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            if let Event::SelectChanged { keys, .. } = &event {
                let proposal: Vec<_> = keys.iter().map(|key| key.as_str().to_owned()).collect();
                let result = (|| -> Result {
                    if let Some(control) = &single {
                        control.source.try_with(|value| {
                            value.as_ref().map_or(Ok(()), |value| {
                                canonical(&validate, std::slice::from_ref(value)).map(|_| ())
                            })
                        })??;
                        control.accept(&proposal.first().cloned())?;
                    }
                    if let Some(control) = &many {
                        control
                            .source
                            .try_with(|value| canonical(&validate, value).map(|_| ()))??;
                        control.accept(&proposal)?;
                    }
                    pulse.touch()?;
                    if let Some(callback) = &mut on_value {
                        callback(proposal.first().cloned());
                    }
                    if owner.is_alive()
                        && let Some(callback) = &mut on_values
                    {
                        callback(proposal);
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                    return;
                }
            }
            observe(&mut handlers, owner, event);
        });
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            inner = inner.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        inner.show_in(scope)?;
        let node = reference.try_get()?.ok_or(glory::ui::Error::Gone)?;
        let weak = scope.downgrade();
        let mut previous = initial;
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let edited = dirty.take();
                let current = selection.try_with(|value| canonical(&config, value))??;
                if edited || current != previous {
                    scope.set_select_selection(node, current.clone())?;
                    previous = current;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        Ok(())
    }
}
