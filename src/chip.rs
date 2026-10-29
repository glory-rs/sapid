//! Selectable pills backed by genuine checkbox/radio values and native events.
use crate::appearance::{Common, common_methods};
use crate::state::Value;
use glory::render::{Event, EventKind, FocusMode, NodeId, Orientation};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};

pub struct Chip {
    inner: Option<crate::Checkbox>,
    class: Vary<String>,
}
pub fn chip() -> Chip {
    Chip {
        inner: Some(crate::checkbox()),
        class: "".into(),
    }
}
impl Chip {
    fn map(mut self, update: impl FnOnce(crate::Checkbox) -> crate::Checkbox) -> Self {
        self.inner = self.inner.take().map(update);
        self
    }
    pub fn theme(self, value: crate::Theme) -> Self {
        self.map(|chip| chip.theme(value))
    }
    pub fn size(self, value: crate::Size) -> Self {
        self.map(|chip| chip.size(value))
    }
    pub fn unstyled(self) -> Self {
        self.map(crate::Checkbox::unstyled)
    }
    pub fn class(mut self, value: impl Into<Vary<String>>) -> Self {
        self.class = value.into();
        self
    }
    pub fn style(self, value: impl Into<Vary<glory::style::Style>>) -> Self {
        self.map(|chip| chip.style(value))
    }
    pub fn name(self, value: impl Into<Vary<String>>) -> Self {
        self.map(|chip| chip.name(value))
    }
    pub fn label(self, value: impl Into<Vary<String>>) -> Self {
        self.map(|chip| chip.label(value))
    }
    pub fn enabled(self, value: impl Into<Vary<bool>>) -> Self {
        self.map(|chip| chip.enabled(value))
    }
    pub fn focus(self, value: FocusMode) -> Self {
        self.map(|chip| chip.focus(value))
    }
    pub fn node_ref(self, value: Cage<Option<NodeId>>) -> Self {
        self.map(|chip| chip.node_ref(value))
    }
    pub fn on(self, kind: EventKind, handler: impl FnMut(Event) + 'static) -> Self {
        self.map(|chip| chip.on(kind, handler))
    }
    pub fn field(self, value: crate::FieldControl) -> Self {
        self.map(|chip| chip.field(value))
    }
    pub fn required(self, value: impl Into<Vary<bool>>) -> Self {
        self.map(|chip| chip.required(value))
    }
    pub fn readonly(self, value: impl Into<Vary<bool>>) -> Self {
        self.map(|chip| chip.readonly(value))
    }
    pub fn invalid(self, value: impl Into<Vary<bool>>) -> Self {
        self.map(|chip| chip.invalid(value))
    }
    pub fn checked(self, value: impl Into<Vary<bool>>) -> Self {
        self.map(|chip| chip.checked(value))
    }
    pub fn default_checked(self, value: bool) -> Self {
        self.map(|chip| chip.default_checked(value))
    }
    pub fn on_checked_change(self, handler: impl FnMut(bool) + 'static) -> Self {
        self.map(|chip| chip.on_checked_change(handler))
    }
}
impl Widget for Chip {
    fn build(&mut self, scope: &Scope) -> Result {
        let class = self.class.clone();
        self.inner
            .take()
            .ok_or(glory::ui::Error::Gone)?
            .class(scope.bond(move || format!("sapid-chip {}", class.get())))
            .show_in(scope)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChipMode {
    #[default]
    Single,
    Multiple,
}

pub struct ChipItem {
    key: String,
    label: Vary<String>,
    enabled: Vary<bool>,
}
pub fn chip_item(key: impl Into<String>, label: impl Into<Vary<String>>) -> ChipItem {
    ChipItem {
        key: key.into(),
        label: label.into(),
        enabled: true.into(),
    }
}
impl ChipItem {
    pub fn enabled(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.enabled = value.into();
        self
    }
}

pub struct ChipGroup {
    key: String,
    common: Common,
    mode: ChipMode,
    label: Vary<String>,
    items: Vec<ChipItem>,
    field: Option<crate::FieldControl>,
    required: Vary<bool>,
    readonly: Vary<bool>,
    invalid: Vary<bool>,
    single: Value<String>,
    multiple: Value<Vec<String>>,
    single_configured: bool,
    multiple_configured: bool,
    on_single: Option<Box<dyn FnMut(String)>>,
    on_multiple: Option<Box<dyn FnMut(Vec<String>)>>,
}
pub fn chip_group(key: impl Into<String>) -> ChipGroup {
    ChipGroup {
        key: key.into(),
        common: Common::default(),
        mode: ChipMode::Single,
        label: "".into(),
        items: Vec::new(),
        field: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
        single: Value::default(),
        multiple: Value::default(),
        single_configured: false,
        multiple_configured: false,
        on_single: None,
        on_multiple: None,
    }
}
impl ChipGroup {
    common_methods!();
    pub fn mode(mut self, value: ChipMode) -> Self {
        self.mode = value;
        self
    }
    pub fn label(mut self, value: impl Into<Vary<String>>) -> Self {
        self.label = value.into();
        self
    }
    pub fn item(mut self, value: ChipItem) -> Self {
        if self.items.len() <= crate::radio::MAX_RADIO_GROUP_ITEMS {
            self.items.push(value);
        }
        self
    }
    pub fn items(mut self, values: impl IntoIterator<Item = ChipItem>) -> Self {
        let remaining =
            (crate::radio::MAX_RADIO_GROUP_ITEMS + 1).saturating_sub(self.items.len());
        self.items.extend(values.into_iter().take(remaining));
        self
    }
    pub fn field(mut self, field: crate::FieldControl) -> Self {
        self.common.theme = field.theme.clone();
        self.common.size = field.size;
        self.common.unstyled = field.unstyled;
        self.field = Some(field);
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
    pub fn value(mut self, value: impl Into<Vary<String>>) -> Self {
        self.single.source = Some(value.into());
        self.single_configured = true;
        self
    }
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.single.default = value.into();
        self.single_configured = true;
        self
    }
    pub fn values(mut self, value: impl Into<Vary<Vec<String>>>) -> Self {
        self.multiple.source = Some(value.into());
        self.multiple_configured = true;
        self
    }
    pub fn default_values(mut self, values: impl IntoIterator<Item = String>) -> Self {
        self.multiple.default = values
            .into_iter()
            .take(crate::radio::MAX_RADIO_GROUP_ITEMS + 1)
            .collect();
        self.multiple_configured = true;
        self
    }
    pub fn on_value_change(mut self, handler: impl FnMut(String) + 'static) -> Self {
        self.on_single = Some(Box::new(handler));
        self.single_configured = true;
        self
    }
    pub fn on_values_change(mut self, handler: impl FnMut(Vec<String>) + 'static) -> Self {
        self.on_multiple = Some(Box::new(handler));
        self.multiple_configured = true;
        self
    }
}

impl Widget for ChipGroup {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.mode == ChipMode::Single && self.multiple_configured
            || self.mode == ChipMode::Multiple && self.single_configured
        {
            return Err(glory::render::Error::InvalidProps(
                "chip selection configuration does not match the group mode",
            )
            .into());
        }
        let class = self.common.class.clone();
        let class = scope.bond(move || format!("sapid-chip-group {}", class.get()));
        match self.mode {
            ChipMode::Single => {
                let value = std::mem::take(&mut self.single);
                let mut group = crate::radio_group(&self.key)
                    .label(self.label.clone())
                    .orientation(Orientation::Horizontal)
                    .theme(self.common.theme.clone())
                    .size(self.common.size)
                    .class(class)
                    .style(self.common.style.clone())
                    .enabled(self.common.enabled.clone())
                    .focus(self.common.focus)
                    .required(self.required.clone())
                    .readonly(self.readonly.clone())
                    .invalid(self.invalid.clone())
                    .default_value(value.default);
                if let Some(value) = value.source {
                    group = group.value(value);
                }
                if let Some(callback) = self.on_single.take() {
                    group = group.on_value_change(callback);
                }
                if let Some(field) = self.field.take() {
                    group = group.field(field);
                }
                if let Some(name) = self.common.name.take() {
                    group = group.name(name);
                }
                if let Some(reference) = self.common.node_ref {
                    group = group.node_ref(reference);
                }
                if self.common.unstyled {
                    group = group.unstyled();
                }
                for item in self.items.drain(..) {
                    group = group.radio(
                        crate::radio(item.key)
                            .label(item.label)
                            .enabled(item.enabled),
                    );
                }
                for (kind, handlers) in std::mem::take(&mut self.common.handlers) {
                    for handler in handlers {
                        group = group.on(kind, handler);
                    }
                }
                group.show_in(scope)
            }
            ChipMode::Multiple => {
                let value = std::mem::take(&mut self.multiple);
                let custom_style = self.common.style.clone();
                let style = scope.bond(move || {
                    crate::appearance::merge(
                        glory::style::Style::new()
                            .display(glory::style::Display::Flex)
                            .flex_direction(glory::style::FlexDirection::Row)
                            .gap(8.0),
                        custom_style.get(),
                    )
                });
                let mut group = crate::checkbox_group(&self.key)
                    .label(self.label.clone())
                    .theme(self.common.theme.clone())
                    .size(self.common.size)
                    .class(class)
                    .style(style)
                    .enabled(self.common.enabled.clone())
                    .focus(self.common.focus)
                    .required(self.required.clone())
                    .readonly(self.readonly.clone())
                    .invalid(self.invalid.clone())
                    .default_selected(value.default);
                if let Some(value) = value.source {
                    group = group.selected(value);
                }
                if let Some(callback) = self.on_multiple.take() {
                    group = group.on_selected_change(callback);
                }
                if let Some(field) = self.field.take() {
                    group = group.field(field);
                }
                if let Some(name) = self.common.name.take() {
                    group = group.name(name);
                }
                if let Some(reference) = self.common.node_ref {
                    group = group.node_ref(reference);
                }
                if self.common.unstyled {
                    group = group.unstyled();
                }
                for item in self.items.drain(..) {
                    group = group
                        .item(crate::checkbox_item(item.key, item.label).enabled(item.enabled));
                }
                for (kind, handlers) in std::mem::take(&mut self.common.handlers) {
                    for handler in handlers {
                        group = group.on(kind, handler);
                    }
                }
                group.show_in(scope)
            }
        }
    }
}

impl Default for Chip {
    fn default() -> Self {
        chip()
    }
}
