//! An editable combobox retaining the host Input editing protocol.
use std::rc::{Rc, Weak};

use glory::render::{
    ChoiceCommand, ComboboxKind, Event, EventKind, FocusMode, FormControl, Layer, LayerFocus,
    LayerPlacement, Navigation, OptionSemantics, Orientation, PopupKind, Props, Role,
    SelectEntry, SelectGroup, SelectProps, SemanticId, Semantics,
};
use glory::style::{
    Align, BorderStyle, Color, Display, Edges, FlexDirection, FlexWrap, Length, Limit,
    Overflow, Style,
};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};

use crate::appearance::{Common, common_methods, configure};
use crate::collection::{Collection, CollectionLimits, MountedItem, RovingFocus};
use crate::composite::{self, Callback, Observers};
use crate::native_select::{NativeOption, NativeOptionGroup};
use crate::state::{Control, Dirty, Value};

pub const MAX_COMBOBOX_SEARCH_BYTES: usize = 4096;
pub const MAX_MULTI_SELECT_VALUES: usize = 256;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComboboxFilter {
    #[default]
    Contains,
    Prefix,
    None,
}
impl ComboboxFilter {
    fn matches(self, label: &str, query: &str) -> bool {
        match self {
            Self::None => true,
            Self::Contains => label.to_lowercase().contains(query),
            Self::Prefix => label.to_lowercase().starts_with(query),
        }
    }
}
enum Entry {
    Option(NativeOption),
    Group(NativeOptionGroup),
}

pub struct Combobox {
    kind: ComboboxKind,
    common: Common,
    key: String,
    entries: Vec<Entry>,
    count: usize,
    value: Value<Option<String>>,
    values: Value<Vec<String>>,
    search: Value<String>,
    explicit_search: bool,
    open: Value<bool>,
    changed: Option<Box<dyn FnMut(Option<String>)>>,
    values_changed: Option<Box<dyn FnMut(Vec<String>)>>,
    ready: Option<Box<dyn FnMut(MultiSelectHandle)>>,
    searched: Option<Box<dyn FnMut(String)>>,
    committed: Option<Box<dyn FnMut(String)>>,
    opened: Option<Box<dyn FnMut(bool)>>,
    filter: ComboboxFilter,
    empty: String,
    field: Option<crate::FieldControl>,
    form_name: Option<Vary<String>>,
    required: Vary<bool>,
    readonly: Vary<bool>,
    invalid: Vary<bool>,
    max_values: usize,
    hide_picked_options: bool,
    clearable: bool,
    clear_label: String,
    placeholder: Option<Vary<String>>,
}
pub fn combobox(key: impl Into<String>) -> Combobox {
    Combobox {
        kind: ComboboxKind::Editable,
        common: Common::default(),
        key: key.into(),
        entries: Vec::new(),
        count: 0,
        value: Value::default(),
        values: Value::default(),
        search: Value::default(),
        explicit_search: false,
        open: Value::default(),
        changed: None,
        values_changed: None,
        ready: None,
        searched: None,
        committed: None,
        opened: None,
        filter: ComboboxFilter::default(),
        empty: "No matching options".into(),
        field: None,
        form_name: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
        max_values: MAX_MULTI_SELECT_VALUES,
        hide_picked_options: false,
        clearable: false,
        clear_label: "Clear selection".into(),
        placeholder: None,
    }
}
impl Combobox {
    pub fn placeholder(mut self, value: impl Into<Vary<String>>) -> Self {
        self.placeholder = Some(value.into());
        self
    }
    pub(crate) fn max_values(mut self, maximum: usize) -> Self {
        self.max_values = maximum;
        self
    }
    pub(crate) fn hide_picked_options(mut self, hide: bool) -> Self {
        self.hide_picked_options = hide;
        self
    }
    pub(crate) fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }
    pub(crate) fn clear_label(mut self, label: String) -> Self {
        self.clear_label = label;
        self
    }
    pub(crate) fn autocomplete(mut self) -> Self {
        self.kind = ComboboxKind::Autocomplete;
        self
    }
    pub(crate) fn multiple(mut self) -> Self {
        self.kind = ComboboxKind::Multiple;
        self
    }
    pub(crate) fn values(mut self, values: impl Into<Vary<Vec<String>>>) -> Self {
        self.values.source = Some(values.into());
        self
    }
    pub(crate) fn default_values(mut self, values: Vec<String>) -> Self {
        self.values.default = values;
        self
    }
    pub(crate) fn on_values_change(
        mut self,
        callback: impl FnMut(Vec<String>) + 'static,
    ) -> Self {
        self.values_changed = Some(Box::new(callback));
        self
    }
    pub(crate) fn on_multiple_ready(
        mut self,
        callback: impl FnMut(MultiSelectHandle) + 'static,
    ) -> Self {
        self.ready = Some(Box::new(callback));
        self
    }
    common_methods!();
    pub fn label(mut self, value: impl Into<Vary<String>>) -> Self {
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
    pub fn form_name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.form_name = Some(value.into());
        self
    }
    pub fn value(mut self, value: impl Into<Vary<Option<String>>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: Option<String>) -> Self {
        self.value.default = value;
        self
    }
    pub fn search(mut self, value: impl Into<Vary<String>>) -> Self {
        self.search.source = Some(value.into());
        self.explicit_search = true;
        self
    }
    pub fn default_search(mut self, value: impl Into<String>) -> Self {
        self.search.default = value.into();
        self.explicit_search = true;
        self
    }
    pub fn open(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.open.source = Some(value.into());
        self
    }
    pub fn default_open(mut self, value: bool) -> Self {
        self.open.default = value;
        self
    }
    pub fn on_value_change(mut self, callback: impl FnMut(Option<String>) + 'static) -> Self {
        self.changed = Some(Box::new(callback));
        self
    }
    pub fn on_search_change(mut self, callback: impl FnMut(String) + 'static) -> Self {
        self.searched = Some(Box::new(callback));
        self
    }
    pub fn on_search_commit(mut self, callback: impl FnMut(String) + 'static) -> Self {
        self.committed = Some(Box::new(callback));
        self
    }
    pub fn on_open_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.opened = Some(Box::new(callback));
        self
    }
    pub fn filter(mut self, value: ComboboxFilter) -> Self {
        self.filter = value;
        self
    }
    pub fn empty_message(mut self, value: impl Into<String>) -> Self {
        self.empty = value.into();
        self
    }
    pub fn required(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.required = value.into();
        self
    }
    pub fn readonly(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn invalid(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.invalid = value.into();
        self
    }
    pub fn option(mut self, value: NativeOption) -> Self {
        if self.count <= 1024 {
            self.entries.push(Entry::Option(value));
            self.count += 1;
        }
        self
    }
    pub fn options(mut self, values: impl IntoIterator<Item = NativeOption>) -> Self {
        for value in values
            .into_iter()
            .take(1025usize.saturating_sub(self.count))
        {
            self.entries.push(Entry::Option(value));
            self.count += 1;
        }
        self
    }
    pub fn group(mut self, value: NativeOptionGroup) -> Self {
        if self.count <= 1024 && self.entries.len() <= 1056 {
            self.count += value.options.len();
            self.entries.push(Entry::Group(value));
        }
        self
    }
}

fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn validate_search(value: &str) -> Result {
    if value.len() > MAX_COMBOBOX_SEARCH_BYTES || value.contains(['\0', '\r', '\n']) {
        return Err(invalid(
            "combobox search must be bounded canonical single-line text",
        ));
    }
    Ok(())
}
fn validate_value(config: &SelectProps, value: &Option<String>) -> Result {
    if let Some(value) = value
        && (value.len() > 64
            || !config
                .options()
                .any(|(option, _)| option.key.as_str() == value))
    {
        return Err(invalid("combobox value must be a known bounded option key"));
    }
    Ok(())
}
fn validate_values(config: &SelectProps, values: &[String]) -> Result {
    if values.len() > MAX_MULTI_SELECT_VALUES {
        return Err(invalid("multiple selection exceeds 256 selected keys"));
    }
    for (index, key) in values.iter().enumerate() {
        if key.len() > 64
            || values[..index].contains(key)
            || !config
                .options()
                .any(|(option, _)| option.key.as_str() == key)
        {
            return Err(invalid(
                "multiple selection requires unique known bounded keys",
            ));
        }
    }
    Ok(())
}
fn query_for(
    config: &SelectProps,
    search: &str,
    selected: Option<&str>,
    kind: ComboboxKind,
) -> String {
    if selected.is_some_and(|key| {
        config.options().any(|(option, _)| {
            option.key.as_str() == key
                && if kind == ComboboxKind::Autocomplete {
                    option.value == search
                } else {
                    option.label == search
                }
        })
    }) {
        String::new()
    } else {
        search.to_lowercase()
    }
}

struct State {
    kind: ComboboxKind,
    owner: glory::core::InstanceId,
    value: Control<Option<String>>,
    selected: Cage<Option<String>>,
    multiple: Option<Multiple>,
    search: Control<String>,
    accepted_search: Cage<String>,
    query: Cage<String>,
    composing: Cage<bool>,
    dirty: Dirty,
    open: Control<bool>,
    visible: Vary<bool>,
    editable: Vary<bool>,
    collection: Collection,
    cursor: RovingFocus,
    config: Rc<SelectProps>,
    filter: ComboboxFilter,
    changed: Callback<Option<String>>,
    searched: Callback<String>,
    committed: Callback<String>,
    opened: Callback<bool>,
    activated: Observers,
}
struct Multiple {
    values: Control<Vec<String>>,
    selected: Cage<Vec<String>>,
    changed: Callback<Vec<String>>,
    maximum: usize,
    hide_picked: bool,
}
/// A command handle retains no Scope or controller Owner.
#[derive(Clone)]
pub struct MultiSelectHandle(Weak<State>);
impl MultiSelectHandle {
    /// Append enabled options matching the current filter. Existing selections
    /// outside the filter stay selected; an over-budget proposal rejects whole.
    pub fn select_all(&self) -> Result {
        self.0
            .upgrade()
            .filter(|state| state.owner.is_alive())
            .ok_or(glory::ui::Error::Gone)?
            .select_all(false)
    }
    /// Propose an empty selection without editing the search text.
    pub fn clear_all(&self) -> Result {
        self.0
            .upgrade()
            .filter(|state| state.owner.is_alive())
            .ok_or(glory::ui::Error::Gone)?
            .select_all(true)
    }
}
impl State {
    fn option_visible(&self, key: &str, label: &str) -> bool {
        self.query.with(|query| self.filter.matches(label, query))
            && !self.multiple.as_ref().is_some_and(|multiple| {
                multiple.hide_picked
                    && multiple
                        .selected
                        .with(|values| values.iter().any(|value| value == key))
            })
    }
    fn option_admitted(&self, key: &str) -> bool {
        self.multiple.as_ref().is_none_or(|multiple| {
            multiple.selected.with(|values| {
                values.len() < multiple.maximum || values.iter().any(|value| value == key)
            })
        })
    }
    fn select_all(&self, clear: bool) -> Result {
        if !self.editable.try_get()? || self.composing.try_get()? {
            return Err(invalid(
                "multiple commands require an enabled editable control outside composition",
            ));
        }
        let multiple = self
            .multiple
            .as_ref()
            .ok_or_else(|| invalid("multiple selection required"))?;
        let mut values = self.values()?;
        if clear {
            if values.is_empty() {
                return Ok(());
            }
            values.clear();
        } else {
            let query = self.search.source.try_with(|search| {
                validate_search(search)?;
                Ok::<_, glory::ui::Error>(search.to_lowercase())
            })??;
            for (option, group_disabled) in self.config.options() {
                if !option.disabled
                    && !group_disabled
                    && self.filter.matches(&option.label, &query)
                    && !values.iter().any(|key| key == option.key.as_str())
                {
                    if values.len() == multiple.maximum {
                        return Err(invalid(
                            "multiple select-all exceeds configured selection limit",
                        ));
                    }
                    values.push(option.key.as_str().to_owned());
                }
            }
            if multiple
                .values
                .source
                .try_with(|current| *current == values)?
            {
                return Ok(());
            }
        }
        multiple.values.accept(&values)?;
        multiple.changed.emit(self.owner, values);
        Ok(())
    }
    fn values(&self) -> Result<Vec<String>> {
        let multiple = self
            .multiple
            .as_ref()
            .ok_or_else(|| invalid("multiple selection required"))?;
        multiple.values.source.try_with(|values| {
            if values.len() > multiple.maximum {
                return Err(invalid(
                    "multiple selection exceeds configured selection limit",
                ));
            }
            validate_values(&self.config, values)?;
            Ok::<_, glory::ui::Error>(values.clone())
        })?
    }
    fn remove(&self, key: &str) -> Result {
        if !self.owner.is_alive() || !self.editable.try_get()? || self.composing.try_get()? {
            return Ok(());
        }
        let mut values = self.values()?;
        let Some(index) = values.iter().position(|value| value == key) else {
            return Ok(());
        };
        values.remove(index);
        let multiple = self.multiple.as_ref().expect("validated multiple mode");
        multiple.values.accept(&values)?;
        multiple.changed.emit(self.owner, values);
        Ok(())
    }
    fn opened(&self, value: bool) -> Result {
        if self.owner.is_alive() && self.open.source.try_get()? != value {
            self.open.accept(&value)?;
            self.opened.emit(self.owner, value);
        }
        Ok(())
    }
    fn current(&self) -> Result<Option<MountedItem>> {
        let current = self.cursor.current()?;
        Ok(self
            .collection
            .mounted()?
            .into_iter()
            .find(|item| item.enabled && Some(&item.member) == current.as_ref()))
    }
    fn initial_cursor(&self) -> Result {
        self.cursor.repair(self.selected.try_get()?.as_deref())
    }
    fn navigate(&self, command: Navigation) -> Result {
        let items = self.collection.mounted()?;
        let next = match command {
            Navigation::First => items.iter().find(|item| item.enabled).cloned(),
            Navigation::Last => items.iter().rev().find(|item| item.enabled).cloned(),
            _ => match self.current()? {
                Some(item) => self.cursor.destination(item.node, command)?,
                None => items.into_iter().find(|item| item.enabled),
            },
        };
        if let Some(item) = next {
            self.cursor.accept(&item)?;
        }
        Ok(())
    }
    fn finalize_search(&self, edited: bool) -> Result {
        if !self.owner.is_alive() || self.composing.try_get()? {
            return Ok(());
        }
        let search = self.search.source.try_with(|value| {
            validate_search(value)?;
            Ok::<_, glory::ui::Error>(value.clone())
        })??;
        let selected = self.value.source.try_with(|value| {
            validate_value(&self.config, value)?;
            Ok::<_, glory::ui::Error>(value.clone())
        })??;
        let query = query_for(&self.config, &search, selected.as_deref(), self.kind);
        if self.query.try_with(|value| *value != query)? {
            self.query.try_revise(|value| *value = query)?;
        }
        // A selected option may be refused independently from its accepted search
        // text. Only an actual editor change proposes clearing a mismatched key.
        if edited
            && self.editable.try_get()?
            && selected.as_ref().is_some_and(|key| {
                self.config.options().any(|(option, _)| {
                    option.key.as_str() == key
                        && if self.kind == ComboboxKind::Autocomplete {
                            option.value != search
                        } else {
                            option.label != search
                        }
                })
            })
        {
            self.value.accept(&None)?;
            self.changed.emit(self.owner, None);
        }
        Ok(())
    }
    fn edit(&self, value: String, composing: bool, commit: bool) -> Result {
        if !self.editable.try_get()? {
            return Ok(());
        }
        self.dirty.touch()?;
        validate_search(&value)?;
        let changed = self.search.source.try_with(|current| {
            validate_search(current)?;
            Ok::<_, glory::ui::Error>(current != &value)
        })??;
        self.composing.try_revise(|current| *current = composing)?;
        self.search.accept(&value)?;
        if commit {
            self.committed.emit(self.owner, value);
        } else {
            self.searched.emit(self.owner, value);
        }
        if self.owner.is_alive() {
            self.finalize_search(true)?;
            if !commit && changed {
                self.opened(true)?;
            }
        }
        Ok(())
    }
    fn choose(&self, key: &str) -> Result {
        if !self.editable.try_get()? || self.composing.try_get()? || !self.visible.try_get()? {
            return Ok(());
        }
        if !self
            .collection
            .mounted()?
            .iter()
            .any(|item| item.key == key && item.enabled)
        {
            return Ok(());
        }
        self.value
            .source
            .try_with(|value| validate_value(&self.config, value))??;
        let Some((option, group_disabled)) = self
            .config
            .options()
            .find(|(option, _)| option.key.as_str() == key)
        else {
            return Err(invalid("unknown combobox option"));
        };
        if option.disabled || group_disabled {
            return Ok(());
        }
        if let Some(multiple) = &self.multiple {
            let mut values = self.values()?;
            if let Some(index) = values.iter().position(|value| value == key) {
                values.remove(index);
            } else if values.len() < multiple.maximum {
                values.push(key.to_owned());
            } else {
                return Ok(());
            }
            multiple.values.accept(&values)?;
            multiple.changed.emit(self.owner, values);
            if self.owner.is_alive() {
                self.search.accept(&String::new())?;
                self.dirty.touch()?;
                self.searched.emit(self.owner, String::new());
            }
            if self.owner.is_alive() {
                self.finalize_search(false)?;
            }
            return Ok(());
        }
        let value = Some(key.to_owned());
        let search = if self.kind == ComboboxKind::Autocomplete {
            option.value.clone()
        } else {
            option.label.clone()
        };
        self.value.accept(&value)?;
        self.search.accept(&search)?;
        self.dirty.touch()?;
        self.changed.emit(self.owner, value);
        if self.owner.is_alive() {
            self.searched.emit(self.owner, search);
        }
        if self.owner.is_alive() {
            self.finalize_search(false)?;
            self.opened(false)?;
        }
        Ok(())
    }
    fn command(&self, command: ChoiceCommand) -> Result {
        if command == ChoiceCommand::Close {
            return self.opened(false);
        }
        if !self.editable.try_get()? || self.composing.try_get()? {
            return Ok(());
        }
        match command {
            ChoiceCommand::OpenFirst | ChoiceCommand::Next => {
                if self.visible.try_get()? {
                    self.navigate(Navigation::Next)?;
                } else {
                    self.opened(true)?;
                    self.initial_cursor()?;
                }
            }
            ChoiceCommand::OpenLast | ChoiceCommand::Previous => {
                if self.visible.try_get()? {
                    self.navigate(Navigation::Previous)?;
                } else {
                    self.opened(true)?;
                    self.navigate(Navigation::Last)?;
                }
            }
            ChoiceCommand::First => {
                self.opened(true)?;
                self.navigate(Navigation::First)?;
            }
            ChoiceCommand::Last => {
                self.opened(true)?;
                self.navigate(Navigation::Last)?;
            }
            ChoiceCommand::Commit => {
                if self.visible.try_get()?
                    && let Some(item) = self.current()?
                {
                    self.choose(&item.key)?;
                } else if self.kind == ComboboxKind::Autocomplete {
                    let text = self.search.source.try_with(|value| {
                        validate_search(value)?;
                        Ok::<_, glory::ui::Error>(value.clone())
                    })??;
                    self.committed.emit(self.owner, text);
                    self.opened(false)?;
                }
            }
            ChoiceCommand::Close => {}
            ChoiceCommand::RemoveLast => {
                if self.multiple.is_none() {
                    return Err(invalid("remove-last requires multiple selection"));
                }
                // The host's empty editor observation can predate another
                // controlled edit in this tick. Recheck the accepted source.
                if self.search.source.try_with(|value| {
                    validate_search(value)?;
                    Ok::<_, glory::ui::Error>(value.is_empty())
                })?? && let Some(last) = self.values()?.last()
                {
                    self.remove(last)?;
                }
            }
        }
        Ok(())
    }
}

impl Widget for Combobox {
    fn build(&mut self, scope: &Scope) -> Result {
        let placeholder =
            crate::text_input::placeholder(scope, self.placeholder.take(), false)?;
        if self.count > 1024
            || self.max_values > MAX_MULTI_SELECT_VALUES
            || (self.clearable
                && (self.clear_label.trim().is_empty()
                    || self.clear_label.len() > 256
                    || self.clear_label.contains(['\0', '\r', '\n'])))
            || self.entries.len() > 1056
            || self.empty.len() > 256
            || self.empty.contains(['\0', '\r'])
        {
            return Err(invalid("combobox configuration exceeds its finite budget"));
        }
        let trigger_id = self
            .field
            .as_ref()
            .map(|field| Ok(field.id.clone()))
            .unwrap_or_else(|| composite::id("combobox", &self.key, "input", "root"))?;
        let panel_id = composite::id("combobox", &self.key, "listbox", "root")?;
        let proxy_id = composite::id("combobox", &self.key, "proxy", "root")?;
        let entries = std::mem::take(&mut self.entries)
            .into_iter()
            .map(|entry| match entry {
                Entry::Option(value) => Ok(SelectEntry::Option(value.lower()?)),
                Entry::Group(value) => Ok(SelectEntry::Group(SelectGroup {
                    label: value.label,
                    disabled: value.disabled,
                    options: value
                        .options
                        .into_iter()
                        .map(NativeOption::lower)
                        .collect::<Result<Vec<_>>>()?,
                })),
            })
            .collect::<Result<Vec<_>>>()?;
        let config = Rc::new(SelectProps {
            entries: Rc::new(entries),
            multiple: self.kind == ComboboxKind::Multiple,
            ..Default::default()
        });
        config.validate()?;
        let form_name = composite::form_name(scope, &self.form_name)?;
        if self.kind == ComboboxKind::Autocomplete {
            for (option, _) in config.options() {
                validate_search(&option.value)?;
            }
        }
        let value = std::mem::take(&mut self.value).mount(scope);
        let check_config = config.clone();
        let selected = composite::validated(scope, &value.source, move |value| {
            validate_value(&check_config, value)
        })?;
        let multiple = if self.kind == ComboboxKind::Multiple {
            let values = std::mem::take(&mut self.values).mount(scope);
            let check_config = config.clone();
            let maximum = self.max_values;
            let selected = composite::validated(scope, &values.source, move |values| {
                if values.len() > maximum {
                    return Err(invalid(
                        "multiple selection exceeds configured selection limit",
                    ));
                }
                validate_values(&check_config, values)
            })?;
            Some(Multiple {
                values,
                selected,
                changed: Callback::new(self.values_changed.take()),
                maximum,
                hide_picked: self.hide_picked_options,
            })
        } else {
            None
        };
        if !self.explicit_search {
            self.search.default = selected
                .get()
                .as_ref()
                .and_then(|key| {
                    config
                        .options()
                        .find(|(option, _)| option.key.as_str() == key)
                        .map(|(option, _)| {
                            if self.kind == ComboboxKind::Autocomplete {
                                option.value.clone()
                            } else {
                                option.label.clone()
                            }
                        })
                })
                .unwrap_or_default();
        }
        let search = std::mem::take(&mut self.search).mount(scope);
        let accepted_search =
            composite::validated(scope, &search.source, |value| validate_search(value))?;
        let open = std::mem::take(&mut self.open).mount(scope);
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
        let visible_open = open.source.clone();
        let visible_enabled = enabled.clone();
        let visible: Vary<bool> = scope
            .bond(move || visible_open.get() && visible_enabled.get())
            .into();
        let editable_enabled = enabled.clone();
        let editable_meta = metadata.clone();
        let editable: Vary<bool> = scope
            .bond(move || {
                editable_enabled.get()
                    && !editable_meta.get().is_some_and(|value| value.readonly)
            })
            .into();
        let collection = Collection::new(scope, CollectionLimits::default())?;
        let cursor = RovingFocus::new(scope, collection.clone(), true)?;
        let query = scope.cage(query_for(
            &config,
            &accepted_search.get(),
            selected.get().as_deref(),
            self.kind,
        ));
        let state = Rc::new(State {
            kind: self.kind,
            owner: scope.id(),
            value,
            selected,
            multiple,
            search,
            accepted_search,
            query,
            composing: scope.cage(false),
            dirty: Dirty::new(scope),
            open,
            visible: visible.clone(),
            editable,
            collection,
            cursor,
            config: config.clone(),
            filter: self.filter,
            changed: Callback::new(self.changed.take()),
            searched: Callback::new(self.searched.take()),
            committed: Callback::new(self.committed.take()),
            opened: Callback::new(self.opened.take()),
            activated: Observers::new(
                self.common
                    .handlers
                    .remove(&EventKind::Activate)
                    .unwrap_or_default(),
            ),
        });
        let cleanup = state.clone();
        scope.on_cleanup(move || {
            cleanup.changed.clear();
            if let Some(multiple) = &cleanup.multiple {
                multiple.changed.clear();
            }
            cleanup.searched.clear();
            cleanup.committed.clear();
            cleanup.opened.clear();
            cleanup.activated.clear();
        })?;
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let input_id = trigger_id.clone();
        let input_panel = panel_id.clone();
        let input_state = state.clone();
        let panel_metadata = metadata.clone();
        let panel_accessible_name = self.common.name.clone();
        let panel_named = panel_accessible_name.is_some();
        let key = self.key.clone();
        let input_meta = scope.bond(move || {
            let mut value = metadata.get().map(|value| *value).unwrap_or_default();
            value.placeholder = placeholder.map(|placeholder| placeholder.get());
            value.id = Some(input_id.clone());
            if input_state.kind == ComboboxKind::Autocomplete {
                if let Some(name) = form_name {
                    value.form_control = Some(FormControl {
                        name: name.get(),
                        value: None,
                    });
                }
            } else {
                value.form_control = None;
            }
            value.combobox = Some(input_state.kind);
            value.controls = Some(input_panel.clone());
            value.popup = Some(PopupKind::Listbox);
            value.expanded = Some(input_state.visible.get());
            value.active_descendant =
                if input_state.visible.get() {
                    input_state.current().ok().flatten().and_then(|item| {
                        composite::id("combobox", &key, "option", &item.key).ok()
                    })
                } else {
                    None
                };
            Some(Box::new(value))
        });
        let base = self.common.theme.control(self.common.size);
        let mut input = configure!(
            self.common,
            glory::ui::input(),
            scope,
            "sapid-combobox",
            base
        )
        .value(accepted_search.get())
        .enabled(enabled.clone())
        .semantics(input_meta)
        .node_ref(reference);
        for kind in [EventKind::ValueChanged, EventKind::Commit] {
            let controller = state.clone();
            let weak = scope.downgrade();
            let observers =
                Observers::new(self.common.handlers.remove(&kind).unwrap_or_default());
            input = input.on(kind, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                let result = match &event {
                    Event::ValueChanged {
                        value, composing, ..
                    } => controller.edit(value.clone(), *composing, false),
                    Event::Commit { value, .. } => controller.edit(value.clone(), false, true),
                    _ => Ok(()),
                };
                if let Err(error) = result {
                    scope.report_error(error);
                    return;
                }
                observers.emit(scope.id(), event);
            });
        }
        let controller = state.clone();
        let weak = scope.downgrade();
        let composition_observers = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Composition)
                .unwrap_or_default(),
        );
        input = input.on(EventKind::Composition, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            if let Event::Composition { active } = &event {
                let result = (|| -> Result {
                    controller.composing.try_revise(|value| *value = *active)?;
                    if !active {
                        controller.dirty.touch()?;
                        controller.finalize_search(true)?;
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                    return;
                }
            }
            composition_observers.emit(scope.id(), event);
        });
        let controller = state.clone();
        let weak = scope.downgrade();
        let choice_observers = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Choice)
                .unwrap_or_default(),
        );
        input = input.on(EventKind::Choice, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            if let Event::Choice { command, .. } = &event
                && let Err(error) = controller.command(*command)
            {
                scope.report_error(error);
                return;
            }
            choice_observers.emit(scope.id(), event);
        });
        let controller = state.clone();
        let weak = scope.downgrade();
        let focus_observers = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::FocusChanged)
                .unwrap_or_default(),
        );
        input = input.on(EventKind::FocusChanged, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            let result = match event {
                Event::FocusChanged(false) => controller.opened(false),
                Event::FocusChanged(true) if controller.editable.try_get().unwrap_or(false) => {
                    controller
                        .opened(true)
                        .and_then(|()| controller.initial_cursor())
                }
                _ => Ok(()),
            };
            if let Err(error) = result {
                scope.report_error(error);
                return;
            }
            focus_observers.emit(scope.id(), event);
        });
        for (kind, handlers) in std::mem::take(&mut self.common.handlers) {
            let observers = Observers::new(handlers);
            let owner = scope.id();
            input = input.on(kind, move |event| observers.emit(owner, event));
        }
        let root = scope.create(glory::render::Descriptor::new(Props::Group))?;
        let content = scope.child_in(root)?;
        if let Some(multiple) = &state.multiple {
            let controller = state.clone();
            let theme = self.common.theme.clone();
            let plain = self.common.unstyled;
            let size = self.common.size;
            let values_ref = scope.cage(None);
            glory::ui::view()
                .class(if plain {
                    "sapid-multi-select-values sapid-unstyled"
                } else {
                    "sapid-multi-select-values"
                })
                .style(Style {
                    align: Some(Align::Center),
                    ..Style::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Row)
                        .flex_wrap(FlexWrap::Wrap)
                        .gap(4.0)
                        .min_width(Length::Px(0.0))
                })
                .node_ref(values_ref)
                .show_in(&content)?;
            let values_scope =
                scope.child_in(values_ref.get().ok_or(glory::ui::Error::Gone)?)?;
            glory::ui::each(
                multiple.selected,
                |key| key.clone(),
                move |key, _| {
                    let controller = controller.clone();
                    let theme = theme.clone();
                    glory::ui::widget(move |scope| {
                        let key = key.get();
                        let label = controller
                            .config
                            .options()
                            .find(|(option, _)| option.key.as_str() == key)
                            .map(|(option, _)| option.label.clone())
                            .ok_or_else(|| invalid("retired multiple option"))?;
                        let mut style = Style::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Row);
                        if !plain {
                            style = crate::tag::style(&theme, size, false);
                        }
                        let command = controller.clone();
                        let weak = scope.downgrade();
                        let mut remove = crate::close_button()
                            .theme(theme.clone())
                            .size(size)
                            .style(if plain {
                                Style::new()
                            } else {
                                crate::tag::remove_style(size)
                            })
                            .class("sapid-tag-remove")
                            .name(format!("Remove {label}"))
                            .enabled(controller.editable.clone())
                            .on(EventKind::Activate, move |_| {
                                if let Some(scope) = weak.upgrade() {
                                    let result = command.remove(&key).and_then(|()| {
                                        if scope.capabilities()?.focus
                                            && let Some(node) = reference.try_get()?
                                        {
                                            scope.action(glory::render::Action::Focus(node))?;
                                        }
                                        Ok(())
                                    });
                                    if let Err(error) = result {
                                        scope.report_error(error);
                                    }
                                }
                            });
                        if plain {
                            remove = remove.unstyled();
                        }
                        glory::ui::view()
                            .class(format!(
                                "sapid-multi-select-tag {} {}{}",
                                theme.class(),
                                size.class(),
                                if plain { " sapid-unstyled" } else { "" }
                            ))
                            .style(style)
                            .fill(glory::ui::text().class("sapid-tag-label").content(label))
                            .fill(remove)
                            .show_in(scope)
                    })
                },
            )
            .show_in(&values_scope)?;
            content.retain(values_scope)?;
        }
        if self.clearable && self.kind == ComboboxKind::Multiple {
            let row_ref = scope.cage(None);
            glory::ui::view()
                .class("sapid-combobox-editor")
                .style(Style {
                    align: Some(Align::Center),
                    ..Style::new()
                        .display(Display::Flex)
                        .gap(6.0)
                        .min_width(Length::Px(0.0))
                })
                .node_ref(row_ref)
                .show_in(&content)?;
            let row = scope.child_in(row_ref.get().ok_or(glory::ui::Error::Gone)?)?;
            input.show_in(&row)?;
            let controller = state.clone();
            let shown = scope.bond(move || {
                controller.editable.get()
                    && !controller.composing.get()
                    && controller
                        .multiple
                        .as_ref()
                        .is_some_and(|multiple| !multiple.selected.with(Vec::is_empty))
            });
            let controller = state.clone();
            let theme = self.common.theme.clone();
            let size = self.common.size;
            let plain = self.common.unstyled;
            let label = self.clear_label.clone();
            glory::ui::when(shown, move || {
                let controller = controller.clone();
                let theme = theme.clone();
                let label = label.clone();
                glory::ui::widget(move |scope| {
                    let command = controller.clone();
                    let weak = scope.downgrade();
                    let mut button = crate::close_button()
                        .theme(theme.clone())
                        .size(size)
                        .class("sapid-multi-select-clear")
                        .name(label.clone())
                        .on_press(move || {
                            let Some(scope) = weak.upgrade() else { return };
                            let result = (|| -> Result {
                                // Another event in this host tick may have changed
                                // policy before the clear button's DOM catches up.
                                if !command.owner.is_alive()
                                    || !command.editable.try_get()?
                                    || command.composing.try_get()?
                                {
                                    return Ok(());
                                }
                                command.select_all(true)?;
                                if command.owner.is_alive()
                                    && scope.capabilities()?.focus
                                    && let Some(node) = reference.try_get()?
                                {
                                    scope.action(glory::render::Action::Focus(node))?;
                                }
                                Ok(())
                            })();
                            if let Err(error) = result {
                                scope.report_error(error);
                            }
                        });
                    if plain {
                        button = button.unstyled();
                    }
                    button.show_in(scope)
                })
            })
            .show_in(&row)?;
            content.retain(row)?;
        } else {
            input.show_in(&content)?;
        }
        let input_node = reference.get().ok_or(glory::ui::Error::Gone)?;
        let panel_label = trigger_id.clone();
        let panel_name = panel_id.clone();
        let panel_visible = visible.clone();
        let panel_multiple = self.kind == ComboboxKind::Multiple;
        let panel_meta = scope.bond(move || {
            Some(Box::new(Semantics {
                id: Some(panel_name.clone()),
                labelled_by: if panel_named {
                    None
                } else {
                    panel_metadata
                        .get()
                        .and_then(|metadata| metadata.labelled_by)
                        .or_else(|| Some(panel_label.clone()))
                },
                navigation: Some(Orientation::Vertical),
                listbox_multiple: panel_multiple,
                hidden: !panel_visible.get(),
                layer: Some(Layer {
                    modal: false,
                    focus: LayerFocus::Preserve,
                    placement: LayerPlacement::Anchor(glory::render::AnchorPlacement::new(
                        glory::render::AnchorSide::Bottom,
                        glory::render::AnchorAlign::Start,
                    )),
                    anchor: Some(trigger_id.clone()),
                    initial_focus: None,
                    return_focus: None,
                    ..Default::default()
                }),
                ..Default::default()
            }))
        });
        let mut panel_style = Style::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .min_width(Length::Px(180.0))
            .max_height(Limit::Px(288.0))
            .overflow_y(Overflow::Auto);
        if !self.common.unstyled {
            panel_style.background = Some(self.common.theme.surface);
            panel_style = panel_style
                .padding(Edges::all(4.0))
                .border_width(Edges::all(1.0))
                .border_style(BorderStyle::Solid)
                .border_color(self.common.theme.border)
                .radius(6.0);
        }
        let panel_ref = scope.cage(None);
        let controller = state.clone();
        let weak = scope.downgrade();
        let mut panel = glory::ui::view()
            .role(Role::Listbox)
            .semantics(panel_meta)
            .node_ref(panel_ref)
            .class(format!(
                "sapid-combobox-listbox {}",
                self.common.theme.class()
            ))
            .style(panel_style)
            .on(EventKind::Dismiss, move |_| {
                if let Some(scope) = weak.upgrade()
                    && let Err(error) = controller.opened(false)
                {
                    scope.report_error(error);
                }
            });
        if let Some(name) = panel_accessible_name {
            panel = panel.name(name);
        }
        panel.show_in(&content)?;
        let items_scope = scope.child_in(panel_ref.get().ok_or(glory::ui::Error::Gone)?)?;
        for (entry_index, entry) in config.entries.iter().enumerate() {
            match entry {
                SelectEntry::Option(option) => mount_option(
                    &items_scope,
                    &state,
                    option,
                    false,
                    &self.key,
                    &panel_id,
                    &self.common,
                )?,
                SelectEntry::Group(group) => {
                    let reference = scope.cage(None);
                    let group_state = state.clone();
                    let group_config = config.clone();
                    glory::ui::view()
                        .role(Role::Group)
                        .name(group.label.clone())
                        .style(
                            Style::new()
                                .display(Display::Flex)
                                .flex_direction(FlexDirection::Column),
                        )
                        .semantics(scope.bond(move || {
                            Some(Box::new(Semantics {
                                hidden: !{
                                    match &group_config.entries[entry_index] {
                                        SelectEntry::Group(group) => {
                                            group.options.iter().any(|option| {
                                                group_state.option_visible(
                                                    option.key.as_str(),
                                                    &option.label,
                                                )
                                            })
                                        }
                                        SelectEntry::Option(_) => {
                                            unreachable!("static group declaration")
                                        }
                                    }
                                },
                                ..Default::default()
                            }))
                        }))
                        .node_ref(reference)
                        .fill(glory::ui::text().content(group.label.clone()))
                        .show_in(&items_scope)?;
                    let group_scope =
                        scope.child_in(reference.get().ok_or(glory::ui::Error::Gone)?)?;
                    for option in &group.options {
                        mount_option(
                            &group_scope,
                            &state,
                            option,
                            group.disabled,
                            &self.key,
                            &panel_id,
                            &self.common,
                        )?;
                    }
                    items_scope.retain(group_scope)?;
                }
            }
        }
        let controller = state.clone();
        glory::ui::view()
            .fill(glory::ui::text().content(self.empty.clone()))
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    hidden: {
                        controller.config.options().any(|(option, _)| {
                            controller.option_visible(option.key.as_str(), &option.label)
                        })
                    },
                    ..Default::default()
                }))
            }))
            .show_in(&items_scope)?;
        content.retain(items_scope)?;
        if self.kind != ComboboxKind::Autocomplete {
            let hidden = scope.cage(None);
            glory::ui::view()
                .semantics(Some(Box::new(Semantics {
                    hidden: true,
                    ..Default::default()
                })))
                .node_ref(hidden)
                .show_in(&content)?;
            let proxy_scope = scope.child_in(hidden.get().ok_or(glory::ui::Error::Gone)?)?;
            let form_meta = crate::field::control_metadata(
                scope,
                self.field.as_ref(),
                false,
                false.into(),
                false.into(),
                false.into(),
            );
            let proxy_meta = scope.bond(move || {
                let field_name = form_meta
                    .get()
                    .and_then(|value| value.form_control.map(|value| value.name));
                Some(Box::new(Semantics {
                    id: Some(proxy_id.clone()),
                    form_control: form_name
                        .as_ref()
                        .map(|name| name.get())
                        .or(field_name)
                        .map(|name| FormControl { name, value: None }),
                    ..Default::default()
                }))
            });
            let proxy_multiple = state.multiple.as_ref().map(|multiple| multiple.selected);
            let selection = scope.bond(move || {
                let keys = proxy_multiple
                    .map(|selected| selected.get())
                    .unwrap_or_else(|| selected.get().into_iter().collect());
                keys.into_iter()
                    .map(|key| SemanticId::new(key).expect("validated option key"))
                    .collect::<Vec<_>>()
            });
            glory::ui::select()
                .configuration((*config).clone())
                .selected(selection)
                .enabled(enabled)
                .semantics(proxy_meta)
                .show_in(&proxy_scope)?;
            content.retain(proxy_scope)?;
        }

        let controller = state.clone();
        let weak = scope.downgrade();
        let mut previous = accepted_search.get();
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let dirty = controller.dirty.take();
                let value = controller.accepted_search.try_get()?;
                if !controller.composing.try_get()? && (dirty || value != previous) {
                    scope.set_input_value(input_node, value.clone())?;
                    previous = value;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        let controller = state.clone();
        let weak = scope.downgrade();
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let search = controller.accepted_search.try_get()?;
                let selected = controller.selected.try_get()?;
                if !controller.composing.try_get()? {
                    let query = query_for(
                        &controller.config,
                        &search,
                        selected.as_deref(),
                        controller.kind,
                    );
                    if controller.query.try_with(|old| *old != query)? {
                        controller.query.try_revise(|old| *old = query)?;
                    }
                }
                if controller.visible.try_get()? {
                    controller.cursor.repair(None)?;
                } else {
                    controller.initial_cursor()?;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        let controller = state.clone();
        let weak = scope.downgrade();
        let mut revealed = None;
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let current = if controller.visible.try_get()? {
                    controller.current()?.map(|item| item.node)
                } else {
                    None
                };
                if current != revealed {
                    revealed = current;
                    if let Some(node) = current
                        && scope.capabilities()?.scroll
                    {
                        scope.action(glory::render::Action::RevealOption { node })?;
                    }
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        scope.retain(content)?;
        if self.kind == ComboboxKind::Multiple {
            Callback::new(self.ready.take())
                .emit(scope.id(), MultiSelectHandle(Rc::downgrade(&state)));
        }
        scope.retain(state)?;
        Ok(())
    }
}

fn mount_option(
    scope: &Scope,
    state: &Rc<State>,
    option: &glory::render::SelectOption,
    group_disabled: bool,
    key: &str,
    owner: &SemanticId,
    common: &Common,
) -> Result {
    let parent = scope;
    let item_scope = parent.child()?;
    let scope = &item_scope;
    let id = composite::id("combobox", key, "option", option.key.as_str())?;
    let reference = scope.cage(None);
    let label = option.label.clone();
    let visible_state = state.clone();
    let match_key = option.key.as_str().to_owned();
    let matching = scope.bond(move || visible_state.option_visible(&match_key, &label));
    let allowed = !option.disabled && !group_disabled;
    let admitted_state = state.clone();
    let admit_key = option.key.as_str().to_owned();
    let enabled = scope
        .bond(move || allowed && matching.get() && admitted_state.option_admitted(&admit_key));
    let member = state.collection.register(
        scope,
        option.key.as_str(),
        reference,
        enabled,
        option.label.clone(),
    )?;
    let selected = state.selected;
    let multiple = state.multiple.as_ref().map(|multiple| multiple.selected);
    let option_key = option.key.as_str().to_owned();
    let semantic_owner = owner.clone();
    let metadata = scope.bond(move || {
        Some(Box::new(Semantics {
            id: Some(id.clone()),
            option: Some(OptionSemantics {
                owner: semantic_owner.clone(),
            }),
            selected: Some(
                multiple
                    .map(|values| values.with(|values| values.contains(&option_key)))
                    .unwrap_or_else(|| selected.get().as_deref() == Some(&option_key)),
            ),
            hidden: !matching.get(),
            ..Default::default()
        }))
    });
    let mut base = if common.unstyled {
        Style::new()
    } else {
        common.theme.control(common.size)
    };
    if !common.unstyled {
        base.border_width = Some(Edges::all(0.0));
        base.background = Some(Color(0, 0, 0, 0));
    }
    let styled = state.clone();
    let styled_member = member.clone();
    let accent = common.theme.accent_soft;
    let foreground = common.theme.accent_text;
    let plain = common.unstyled;
    let style = scope.bond(move || {
        let mut value = base.clone();
        if !plain && styled.cursor.current().ok().flatten().as_ref() == Some(&styled_member) {
            value.background = Some(accent);
            value.color = Some(foreground);
        }
        value
    });
    let controller = state.clone();
    let weak = scope.downgrade();
    let choose_key = option.key.as_str().to_owned();
    let pointer = state.clone();
    let pointer_weak = scope.downgrade();
    let pointer_member = member.clone();
    glory::ui::button()
        .label(option.label.clone())
        .focus(FocusMode::Programmatic)
        .class(format!(
            "sapid-combobox-option {}{}",
            common.theme.class(),
            if common.unstyled {
                " sapid-unstyled"
            } else {
                ""
            }
        ))
        .style(style)
        .semantics(metadata)
        .node_ref(reference)
        .enabled(enabled)
        .on(EventKind::Activate, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            if let Err(error) = controller.choose(&choose_key) {
                scope.report_error(error);
                return;
            }
            controller.activated.emit(scope.id(), event);
        })
        .on(EventKind::Pointer, move |event| {
            let Some(scope) = pointer_weak.upgrade() else {
                return;
            };
            if matches!(
                event,
                Event::Pointer {
                    phase: glory::render::PointerPhase::Move,
                    ..
                }
            ) {
                let result = (|| -> Result {
                    if !pointer.composing.try_get()? {
                        pointer.cursor.observe(&pointer_member)?;
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                }
            }
        })
        .show_in(scope)?;
    parent.retain(item_scope)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use glory::render::{InputProps, NodeId};
    use glory::ui::App;
    struct QueuedHost {
        memory: glory::render::memory::Memory,
        input: Rc<std::cell::RefCell<Vec<glory::render::Input>>>,
    }
    impl glory::render::Render for QueuedHost {
        fn style_profile(&self) -> glory::style::Profile {
            glory::style::Profile::RECORD
        }
        fn capabilities(&self) -> glory::render::Capabilities {
            glory::render::Capabilities::default()
        }
        fn apply(
            &mut self,
            mutations: &[glory::render::Mutation],
        ) -> std::result::Result<glory::render::Receipt, glory::render::Error> {
            self.memory.apply(mutations)
        }
        fn request(
            &mut self,
            query: glory::render::Query,
            token: glory::render::QueryToken,
        ) -> std::result::Result<(), glory::render::Error> {
            self.memory.request(query, token)
        }
        fn drain(&mut self, inputs: &mut Vec<glory::render::Input>) {
            inputs.append(&mut self.input.borrow_mut());
            self.memory.drain(inputs);
        }
    }
    fn queued() -> (App, Rc<std::cell::RefCell<Vec<glory::render::Input>>>) {
        let input = Rc::new(std::cell::RefCell::new(Vec::new()));
        let queue = input.clone();
        (
            App::new(move |runtime| QueuedHost {
                memory: glory::render::memory::Memory::new(runtime),
                input: queue,
            }),
            input,
        )
    }
    fn nodes(app: &App) -> Vec<NodeId> {
        let tree = app.tree();
        let mut pending = tree.roots().to_vec();
        let mut nodes = Vec::new();
        while let Some(id) = pending.pop() {
            nodes.push(id);
            pending.extend(tree.node(id).unwrap().children.iter().copied());
        }
        nodes
    }
    fn editor(app: &App) -> NodeId {
        nodes(app)
            .into_iter()
            .find(|id| {
                matches!(
                    app.tree().node(*id).unwrap().descriptor.view().props,
                    Props::Input(_)
                )
            })
            .unwrap()
    }
    fn input(app: &App) -> InputProps {
        let tree = app.tree();
        let node = editor(app);
        let Props::Input(value) = tree.node(node).unwrap().descriptor.view().props else {
            panic!()
        };
        value.clone()
    }
    fn option(app: &App, label: &str) -> NodeId {
        nodes(app).into_iter().find(|id| matches!(app.tree().node(*id).unwrap().descriptor.view().props, Props::Button(value) if value.label == label)).unwrap()
    }
    fn selected(app: &App, label: &str) -> bool {
        app.tree()
            .node(option(app, label))
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .selected
            == Some(true)
    }
    fn edit(app: &App, value: &str, composing: bool) {
        let editor = editor(app);
        app.dispatch(
            editor,
            Event::ValueChanged {
                value: value.into(),
                composing,
                revision: input(app).revision,
            },
        )
        .unwrap();
    }
    fn choices() -> Combobox {
        combobox("edit-choice").label("Editable choice").options([
            crate::select_option("a", "Alpha"),
            crate::select_option("b", "Bravo"),
            crate::select_option("c", "Branch"),
            crate::select_option("d", "Blocked").disabled(true),
        ])
    }
    #[test]
    fn editing_filters_real_options_and_keeps_input_activity_and_selection_separate() {
        let app = App::memory();
        app.mount(choices().default_value(Some("a".into())))
            .unwrap();
        assert_eq!(input(&app).value, "Alpha");
        edit(&app, "br", false);
        let editor = editor(&app);
        assert!(!selected(&app, "Alpha"));
        assert!(!app.tree().active(option(&app, "Alpha")));
        assert!(app.tree().active(option(&app, "Bravo")));
        assert_eq!(
            app.tree().resolve_active_descendant(editor).unwrap(),
            Some(option(&app, "Bravo"))
        );
        app.dispatch(
            editor,
            Event::Choice {
                from: editor,
                command: ChoiceCommand::Next,
            },
        )
        .unwrap();
        assert_eq!(
            app.tree().resolve_active_descendant(editor).unwrap(),
            Some(option(&app, "Branch"))
        );
        app.dispatch(
            editor,
            Event::Choice {
                from: editor,
                command: ChoiceCommand::Commit,
            },
        )
        .unwrap();
        assert_eq!(input(&app).value, "Branch");
        assert!(selected(&app, "Branch"));
        assert_eq!(app.tree().resolve_active_descendant(editor).unwrap(), None);
    }
    #[test]
    fn composition_freezes_filter_navigation_selection_and_controlled_reconciliation_until_end()
    {
        let app = App::memory();
        let search = app.scope().cage("Alpha".to_owned());
        app.mount(
            choices()
                .search(search)
                .default_value(Some("a".into()))
                .default_open(true)
                .on_search_change(move |value| search.revise(|current| *current = value)),
        )
        .unwrap();
        let editor = editor(&app);
        let before = input(&app);
        app.dispatch(editor, Event::Composition { active: true })
            .unwrap();
        edit(&app, "ni", true);
        assert_eq!(search.get(), "ni");
        assert_eq!(input(&app), before);
        assert!(selected(&app, "Alpha"));
        assert!(app.tree().active(option(&app, "Bravo")));
        app.dispatch(
            editor,
            Event::Choice {
                from: editor,
                command: ChoiceCommand::Next,
            },
        )
        .unwrap();
        app.dispatch(option(&app, "Bravo"), Event::Activate)
            .unwrap();
        assert!(selected(&app, "Alpha"));
        assert_eq!(input(&app), before);
        app.dispatch(editor, Event::Composition { active: false })
            .unwrap();
        assert_eq!(input(&app).value, "ni");
        assert!(!selected(&app, "Alpha"));
        assert!(!app.tree().active(option(&app, "Bravo")));
        assert_eq!(app.tree().resolve_active_descendant(editor).unwrap(), None);
    }
    #[test]
    fn controlled_rejected_search_value_and_open_are_not_overwritten_by_native_proposals() {
        let app = App::memory();
        let search = app.scope().cage("Alpha".to_owned());
        let value = app.scope().cage(Some("a".to_owned()));
        let open = app.scope().cage(false);
        app.mount(choices().search(search).value(value).open(open))
            .unwrap();
        let revision = input(&app).revision;
        edit(&app, "Br", false);
        assert_eq!(input(&app).value, "Alpha");
        assert!(input(&app).revision > revision);
        assert_eq!(search.get(), "Alpha");
        assert_eq!(value.get(), Some("a".into()));
        assert!(!open.get());
        assert_eq!(
            app.tree().resolve_active_descendant(editor(&app)).unwrap(),
            None
        );
        open.revise(|value| *value = true);
        app.flush().unwrap();
        app.dispatch(option(&app, "Bravo"), Event::Activate)
            .unwrap();
        assert!(selected(&app, "Alpha"));
        assert_eq!(input(&app).value, "Alpha");
        assert_eq!(value.get(), Some("a".into()));
        assert!(open.get());
    }
    #[test]
    fn field_proxy_submits_selected_business_value_and_real_input_retains_editing_semantics() {
        let snapshot = glory::ssr::snapshot(
            crate::field("editable-plan")
                .label("Plan")
                .description("Search plans")
                .required(true)
                .form_name("plan")
                .control(|field| choices().field(field).default_value(Some("b".into()))),
        )
        .unwrap();
        assert!(
            snapshot.html.contains("<input")
                && snapshot.html.contains("role=\"combobox\"")
                && snapshot.html.contains("aria-autocomplete=\"list\"")
                && snapshot.html.contains("value=\"Bravo\"")
        );
        assert!(
            snapshot.html.contains("name=\"plan\"")
                && snapshot.html.contains("value=\"b\" selected")
        );
        assert_eq!(snapshot.html.matches(" required=\"\"").count(), 1);
    }
    #[test]
    fn bounded_receivers_and_huge_constant_or_live_search_reject_without_source_clones() {
        let app = App::memory();
        let consumed = Rc::new(std::cell::Cell::new(0));
        let count = consumed.clone();
        let widget = combobox("bound").options((0..1_000_000).map(move |index| {
            count.set(count.get() + 1);
            crate::select_option(format!("k{index}"), "Label")
        }));
        assert_eq!(consumed.get(), 1025);
        assert!(app.mount(widget).is_err());
        assert_eq!(app.stats().nodes, 0);
        assert!(app.mount(choices().search("x".repeat(1_000_000))).is_err());
        assert_eq!(app.stats().nodes, 0);
        let search = app.scope().cage("Alpha".to_owned());
        app.mount(choices().search(search)).unwrap();
        search.revise(|value| *value = "x".repeat(1_000_000));
        assert!(app.flush().is_err());
        assert_eq!(input(&app).value, "Alpha");
        search.revise(|value| *value = "Br".into());
        app.flush().unwrap();
        assert_eq!(input(&app).value, "Br");
    }
    #[test]
    fn filters_are_finite_case_insensitive_and_support_prefix_or_no_filter() {
        assert!(ComboboxFilter::Contains.matches("Canadá", "adá"));
        assert!(!ComboboxFilter::Prefix.matches("Canadá", "adá"));
        assert!(ComboboxFilter::Prefix.matches("Canadá", "can"));
        assert!(ComboboxFilter::None.matches("Canada", "anything"));
    }
    #[test]
    fn callback_captures_drop_outside_graph_borrows_when_widget_owner_retires() {
        struct Capture {
            source: Cage<usize>,
            released: Rc<std::cell::Cell<bool>>,
        }
        impl Drop for Capture {
            fn drop(&mut self) {
                self.released.set(self.source.try_get().is_ok());
            }
        }
        let app = App::memory();
        let released = Rc::new(std::cell::Cell::new(false));
        let capture = Capture {
            source: app.scope().cage(1),
            released: released.clone(),
        };
        let mounted = choices()
            .on_search_change(move |_| {
                let _ = &capture;
            })
            .mount(app.scope())
            .unwrap();
        drop(mounted);
        app.flush().unwrap();
        assert!(released.get());
        assert_eq!(app.stats().nodes, 0);
    }
    #[test]
    fn filtering_before_binding_flush_rejects_queued_pointer_activation_and_stale_keyboard_commit()
     {
        use glory::render::Input;
        let (app, queue) = queued();
        let proposals = Rc::new(std::cell::RefCell::new(Vec::new()));
        let out = proposals.clone();
        app.mount(
            choices()
                .default_value(Some("a".into()))
                .default_open(true)
                .on_value_change(move |value| out.borrow_mut().push(value)),
        )
        .unwrap();
        let editor = editor(&app);
        let alpha = option(&app, "Alpha");
        let revision = input(&app).revision;
        queue.borrow_mut().extend([
            Input::Event {
                node: editor,
                event: Event::ValueChanged {
                    value: "Br".into(),
                    composing: false,
                    revision,
                },
            },
            Input::Event {
                node: alpha,
                event: Event::Activate,
            },
            Input::Event {
                node: editor,
                event: Event::Choice {
                    from: editor,
                    command: ChoiceCommand::Commit,
                },
            },
        ]);
        app.pump().unwrap();
        assert_eq!(*proposals.borrow(), [None]);
        assert_eq!(input(&app).value, "Br");
        assert!(!selected(&app, "Alpha"));
        assert!(!app.tree().active(alpha));
        assert!(!app.dispatch(alpha, Event::Activate).unwrap());
        app.dispatch(
            editor,
            Event::Choice {
                from: editor,
                command: ChoiceCommand::Commit,
            },
        )
        .unwrap();
        assert!(selected(&app, "Bravo"));
    }
    #[test]
    fn all_same_revision_edits_in_a_host_tick_run_before_controlled_reconciliation() {
        use glory::render::Input;
        let (app, queue) = queued();
        let search = app.scope().cage(String::new());
        let observed = Rc::new(std::cell::RefCell::new(Vec::new()));
        let out = observed.clone();
        app.mount(choices().search(search).on_search_change(move |value| {
            out.borrow_mut().push(value.clone());
            search.revise(|old| *old = value);
        }))
        .unwrap();
        let editor = editor(&app);
        let revision = input(&app).revision;
        queue
            .borrow_mut()
            .extend(["B", "Br"].map(|value| Input::Event {
                node: editor,
                event: Event::ValueChanged {
                    value: value.into(),
                    composing: false,
                    revision,
                },
            }));
        app.pump().unwrap();
        assert_eq!(*observed.borrow(), ["B", "Br"]);
        assert_eq!(search.get(), "Br");
        assert_eq!(input(&app).value, "Br");
    }
    #[test]
    fn closed_native_commit_stays_closed_and_is_distinct_from_option_selection() {
        let app = App::memory();
        let commits = Rc::new(std::cell::RefCell::new(Vec::new()));
        let out = commits.clone();
        app.mount(
            choices()
                .default_value(Some("a".into()))
                .on_search_commit(move |value| out.borrow_mut().push(value)),
        )
        .unwrap();
        let editor = editor(&app);
        app.dispatch(
            editor,
            Event::Commit {
                value: "Alpha".into(),
                revision: input(&app).revision,
            },
        )
        .unwrap();
        assert_eq!(*commits.borrow(), ["Alpha"]);
        assert!(selected(&app, "Alpha"));
        assert_eq!(
            app.tree()
                .node(editor)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .expanded,
            Some(false)
        );
        edit(&app, "Alpha", false);
        assert_eq!(
            app.tree()
                .node(editor)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .expanded,
            Some(false)
        );
    }
    #[test]
    fn readonly_during_composition_preserves_program_source_until_terminal_cleanup_without_value_proposals()
     {
        let app = App::memory();
        let search = app.scope().cage("Alpha".to_owned());
        let readonly = app.scope().cage(false);
        let proposed = Rc::new(std::cell::RefCell::new(Vec::new()));
        let out = proposed.clone();
        app.mount(
            choices()
                .search(search)
                .readonly(readonly)
                .default_open(true)
                .default_value(Some("a".into()))
                .on_value_change(move |value| out.borrow_mut().push(value)),
        )
        .unwrap();
        let editor = editor(&app);
        let before = input(&app);
        app.dispatch(editor, Event::Composition { active: true })
            .unwrap();
        edit(&app, "preedit", true);
        readonly.revise(|value| *value = true);
        search.revise(|value| *value = "Program".into());
        app.flush().unwrap();
        assert_eq!(input(&app).value, before.value);
        app.dispatch(editor, Event::Composition { active: false })
            .unwrap();
        assert_eq!(input(&app).value, "Program");
        assert!(selected(&app, "Alpha"));
        assert!(proposed.borrow().is_empty());
    }
    #[test]
    fn refusing_a_chosen_key_does_not_emit_an_extra_clear_when_local_search_accepts_its_label()
    {
        let app = App::memory();
        let source = app.scope().cage(Some("a".to_owned()));
        let proposed = Rc::new(std::cell::RefCell::new(Vec::new()));
        let output = proposed.clone();
        app.mount(
            choices()
                .value(source)
                .default_open(true)
                .on_value_change(move |value| output.borrow_mut().push(value)),
        )
        .unwrap();
        app.dispatch(option(&app, "Bravo"), Event::Activate)
            .unwrap();
        assert_eq!(*proposed.borrow(), [Some("b".into())]);
        assert_eq!(source.get(), Some("a".into()));
        assert!(selected(&app, "Alpha"));
        assert_eq!(input(&app).value, "Bravo");
        edit(&app, "User edit", false);
        assert_eq!(
            proposed.borrow().last(),
            Some(&None),
            "actual editing still proposes clearing a mismatched key"
        );
    }
    include!("combobox_advanced_tests.rs");
}
