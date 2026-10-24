//! Finite checkbox groups with a genuine mixed select-all control.
//! This is an explicit, bounded mounted group, not a virtual data-source registry.
use std::collections::HashSet;
use std::rc::Rc;

use glory::render::{FieldPart, Semantics};
use glory::ui::{Result, Widget};
use glory::{Lotus, Scope};

use crate::appearance::{Common, common_methods, configure, observe};
use crate::composite::{self, Callback};
use crate::mixed_checkbox::{CheckState, mixed_checkbox};
use crate::state::{Control, Value};

/// Maximum explicit mounted items; use application-level pagination for larger sets.
pub const MAX_CHECKBOX_GROUP_ITEMS: usize = 1024;

pub struct CheckboxItem {
    key: String,
    label: Lotus<String>,
    enabled: Lotus<bool>,
}
pub fn checkbox_item(key: impl Into<String>, label: impl Into<Lotus<String>>) -> CheckboxItem {
    CheckboxItem {
        key: key.into(),
        label: label.into(),
        enabled: true.into(),
    }
}
impl CheckboxItem {
    pub fn enabled(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.enabled = value.into();
        self
    }
}

pub struct CheckboxGroup {
    key: String,
    common: Common,
    label: Lotus<String>,
    all_label: Option<Lotus<String>>,
    value: Value<Vec<String>>,
    items: Vec<CheckboxItem>,
    on_change: Option<Box<dyn FnMut(Vec<String>)>>,
    field: Option<crate::FieldControl>,
    required: Lotus<bool>,
    readonly: Lotus<bool>,
    invalid: Lotus<bool>,
}
pub fn checkbox_group(key: impl Into<String>) -> CheckboxGroup {
    CheckboxGroup {
        key: key.into(),
        common: Common::default(),
        label: "".into(),
        all_label: None,
        value: Value::default(),
        items: Vec::new(),
        on_change: None,
        field: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
    }
}
impl CheckboxGroup {
    common_methods!();
    pub fn field(mut self, field: crate::FieldControl) -> Self {
        self.common.theme = field.theme.clone();
        self.common.size = field.size;
        self.common.unstyled = field.unstyled;
        self.field = Some(field);
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
    pub fn label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.label = value.into();
        self
    }
    pub fn select_all_label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.all_label = Some(value.into());
        self
    }
    pub fn selected(mut self, value: impl Into<Lotus<Vec<String>>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_selected(mut self, value: impl IntoIterator<Item = String>) -> Self {
        self.value.default = value
            .into_iter()
            .take(MAX_CHECKBOX_GROUP_ITEMS + 1)
            .collect();
        self
    }
    pub fn item(mut self, item: CheckboxItem) -> Self {
        if self.items.len() <= MAX_CHECKBOX_GROUP_ITEMS {
            self.items.push(item);
        }
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = CheckboxItem>) -> Self {
        let remaining = (MAX_CHECKBOX_GROUP_ITEMS + 1).saturating_sub(self.items.len());
        self.items.extend(items.into_iter().take(remaining));
        self
    }
    pub fn on_selected_change(mut self, handler: impl FnMut(Vec<String>) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

fn validate(keys: &HashSet<String>, selected: &[String]) -> Result {
    let mut unique = HashSet::new();
    if selected.len() > keys.len()
        || selected
            .iter()
            .any(|key| key.len() > 64 || !keys.contains(key) || !unique.insert(key))
    {
        return Err(glory::render::Error::InvalidProps(
            "checkbox group selection requires unique registered keys",
        )
        .into());
    }
    Ok(())
}
fn propose(
    scope: &glory::ui::WeakScope,
    control: &Control<Vec<String>>,
    keys: &HashSet<String>,
    callback: &Callback<Vec<String>>,
    value: Vec<String>,
) {
    let Some(scope) = scope.upgrade() else { return };
    if let Err(error) = validate(keys, &value).and_then(|()| control.accept(&value)) {
        scope.report_error(error);
        return;
    }
    callback.emit(scope.id(), value);
}

impl Widget for CheckboxGroup {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.items.len() > MAX_CHECKBOX_GROUP_ITEMS {
            return Err(glory::render::Error::InvalidProps(
                "checkbox group exceeds mounted item budget",
            )
            .into());
        }
        let id = composite::id("checkbox-group", &self.key, "root", "root")?;
        let legend_id = composite::id("checkbox-group", &self.key, "legend", "root")?;
        let mut keys = HashSet::new();
        for item in &self.items {
            composite::id("checkbox-group", &self.key, "item", &item.key)?;
            if !keys.insert(item.key.clone()) {
                return Err(
                    glory::render::Error::InvalidProps("duplicate checkbox group key").into(),
                );
            }
        }
        let keys = Rc::new(keys);
        let items = Rc::new(std::mem::take(&mut self.items));
        let control = std::mem::take(&mut self.value).mount(scope);
        let callback = Callback::new(self.on_change.take());
        let valid_keys = keys.clone();
        let selected = composite::validated(scope, &control.source, move |value| {
            validate(&valid_keys, value)
        })?;
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
        let states = metadata.clone();
        let requirement_items = items.clone();
        let requirement_enabled = enabled.clone();
        let required_index = scope.bond(move || {
            if !requirement_enabled.get() || !states.get().is_some_and(|states| states.required)
            {
                return None;
            }
            selected.with(|selected| {
                let mut first = None;
                for (index, item) in requirement_items
                    .iter()
                    .enumerate()
                    .filter(|(_, item)| item.enabled.get())
                {
                    if selected.contains(&item.key) {
                        return None;
                    }
                    if first.is_none() {
                        first = Some(index);
                    }
                }
                first
            })
        });
        let root_enabled = enabled.clone();
        let root_states = metadata.clone();
        let legend = glory::ui::view()
            .field_part(FieldPart::Legend)
            .semantics(composite::metadata(Semantics {
                field_part: Some(FieldPart::Legend),
                id: Some(legend_id.clone()),
                ..Default::default()
            }))
            .fill(glory::ui::text().content(self.label.clone()));
        let mut root = configure!(
            self.common,
            glory::ui::view().field_part(FieldPart::Fieldset),
            scope,
            "sapid-checkbox-group",
            composite::column(8.0)
        )
        .semantics(scope.bond(move || {
            let mut states = root_states.get().unwrap_or_default();
            states.field_part = Some(FieldPart::Fieldset);
            if states.id.is_none() {
                states.id = Some(id.clone());
            }
            if states.labelled_by.is_none() {
                states.labelled_by = Some(legend_id.clone());
            }
            states.disabled = !root_enabled.get();
            states.required = false;
            states.readonly = false;
            Some(states)
        }))
        .fill(legend);
        if let Some(label) = self.all_label.take() {
            let source: Lotus<Vec<String>> = selected.into();
            let all_items = items.clone();
            let state = scope.bond(move || {
                let selected = source.get();
                let mut count = 0;
                let mut selected_count = 0;
                for item in all_items.iter().filter(|item| item.enabled.get()) {
                    count += 1;
                    selected_count += usize::from(selected.contains(&item.key));
                }
                if selected_count == 0 {
                    CheckState::Unchecked
                } else if selected_count == count {
                    CheckState::Checked
                } else {
                    CheckState::Mixed
                }
            });
            let all_items = items.clone();
            let enabled = enabled.clone();
            let active = scope
                .bond(move || enabled.get() && all_items.iter().any(|item| item.enabled.get()));
            let all_items = items.clone();
            let control = control.clone();
            let callback = callback.clone();
            let keys = keys.clone();
            let weak = scope.downgrade();
            let readonly = metadata.clone();
            let invalid = metadata.clone();
            let mut all = mixed_checkbox()
                .class("sapid-checkbox-group-all")
                .theme(self.common.theme.clone())
                .size(self.common.size)
                .state(state)
                .readonly(
                    scope.bond(move || readonly.get().is_some_and(|states| states.readonly)),
                )
                .invalid(scope.bond(move || invalid.get().is_some_and(|states| states.invalid)))
                .label(label)
                .enabled(active)
                .on_state_change(move |state| {
                    let Some(scope) = weak.upgrade() else { return };
                    let current = match control.source.try_with(|value| {
                        validate(&keys, value)?;
                        Ok::<_, glory::ui::Error>(value.clone())
                    }) {
                        Ok(Ok(value)) => value,
                        Ok(Err(error)) => {
                            scope.report_error(error);
                            return;
                        }
                        Err(error) => {
                            scope.report_error(error);
                            return;
                        }
                    };
                    let selected = all_items
                        .iter()
                        .filter(|item| {
                            if item.enabled.get() {
                                state == CheckState::Checked
                            } else {
                                current.contains(&item.key)
                            }
                        })
                        .map(|item| item.key.clone())
                        .collect();
                    propose(&scope.downgrade(), &control, &keys, &callback, selected);
                });
            if self.common.unstyled {
                all = all.unstyled();
            }
            root = root.fill(all);
        }
        for (index, item) in items.iter().enumerate() {
            let source: Lotus<Vec<String>> = selected.into();
            let key = item.key.clone();
            let checked = scope.bond(move || source.with(|selected| selected.contains(&key)));
            let enabled = enabled.clone();
            let item_enabled = item.enabled.clone();
            let active = scope.bond(move || enabled.get() && item_enabled.get());
            let key = item.key.clone();
            let all_items = items.clone();
            let control = control.clone();
            let callback = callback.clone();
            let keys = keys.clone();
            let weak = scope.downgrade();
            let readonly = metadata.clone();
            let invalid = metadata.clone();
            let mut checkbox = crate::checkbox()
                .class("sapid-checkbox-group-item")
                .theme(self.common.theme.clone())
                .size(self.common.size)
                .label(item.label.clone())
                .checked(checked)
                .required(scope.bond(move || required_index.get() == Some(index)))
                .readonly(
                    scope.bond(move || readonly.get().is_some_and(|states| states.readonly)),
                )
                .invalid(scope.bond(move || invalid.get().is_some_and(|states| states.invalid)))
                .enabled(active)
                .on_checked_change(move |checked| {
                    let Some(scope) = weak.upgrade() else { return };
                    let current = match control.source.try_with(|value| {
                        validate(&keys, value)?;
                        Ok::<_, glory::ui::Error>(value.clone())
                    }) {
                        Ok(Ok(value)) => value,
                        Ok(Err(error)) => {
                            scope.report_error(error);
                            return;
                        }
                        Err(error) => {
                            scope.report_error(error);
                            return;
                        }
                    };
                    let selected = all_items
                        .iter()
                        .filter(|item| {
                            if item.key == key {
                                checked
                            } else {
                                current.contains(&item.key)
                            }
                        })
                        .map(|item| item.key.clone())
                        .collect();
                    propose(&scope.downgrade(), &control, &keys, &callback, selected);
                });
            if self.common.unstyled {
                checkbox = checkbox.unstyled();
            }
            root = root.fill(checkbox);
        }
        let owner = scope.id();
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)
    }
}
