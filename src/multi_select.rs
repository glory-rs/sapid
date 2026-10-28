//! Bounded multiple selection with an editable search and live keyed tags.
pub use crate::combobox::MultiSelectHandle;
use crate::{
    Combobox, ComboboxFilter, FieldControl, NativeOption, NativeOptionGroup, Size, Theme,
};
use glory::render::{Event, EventKind, FocusMode, NodeId};
use glory::style::Style;
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};

pub struct MultiSelect {
    inner: Combobox,
}
pub fn multi_select(key: impl Into<String>) -> MultiSelect {
    MultiSelect {
        inner: crate::combobox(key).multiple(),
    }
}
impl MultiSelect {
    pub fn placeholder(mut self, value: impl Into<Vary<String>>) -> Self {
        self.inner = self.inner.placeholder(value);
        self
    }
    /// Application selection limit, from zero through the 256-key safety budget.
    pub fn max_values(mut self, maximum: usize) -> Self {
        self.inner = self.inner.max_values(maximum);
        self
    }
    /// Hide accepted selected options, including now-empty groups.
    pub fn hide_picked_options(mut self, hide: bool) -> Self {
        self.inner = self.inner.hide_picked_options(hide);
        self
    }
    pub fn clearable(mut self, clearable: bool) -> Self {
        self.inner = self.inner.clearable(clearable);
        self
    }
    pub fn clear_label(mut self, label: impl Into<String>) -> Self {
        self.inner = self.inner.clear_label(label.into());
        self
    }
    pub fn on_ready(mut self, callback: impl FnMut(MultiSelectHandle) + 'static) -> Self {
        self.inner = self.inner.on_multiple_ready(callback);
        self
    }
    pub fn theme(mut self, value: Theme) -> Self {
        self.inner = self.inner.theme(value);
        self
    }
    pub fn size(mut self, value: Size) -> Self {
        self.inner = self.inner.size(value);
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.inner = self.inner.unstyled();
        self
    }
    pub fn class(mut self, value: impl Into<Vary<String>>) -> Self {
        self.inner = self.inner.class(value);
        self
    }
    pub fn style(mut self, value: impl Into<Vary<Style>>) -> Self {
        self.inner = self.inner.style(value);
        self
    }
    pub fn label(mut self, value: impl Into<Vary<String>>) -> Self {
        self.inner = self.inner.label(value);
        self
    }
    pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.inner = self.inner.name(value);
        self
    }
    pub fn enabled(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.inner = self.inner.enabled(value);
        self
    }
    pub fn focus(mut self, value: FocusMode) -> Self {
        self.inner = self.inner.focus(value);
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.inner = self.inner.node_ref(value);
        self
    }
    pub fn on(mut self, kind: EventKind, callback: impl FnMut(Event) + 'static) -> Self {
        self.inner = self.inner.on(kind, callback);
        self
    }
    pub fn field(mut self, value: FieldControl) -> Self {
        self.inner = self.inner.field(value);
        self
    }
    pub fn form_name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.inner = self.inner.form_name(value);
        self
    }
    pub fn required(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.inner = self.inner.required(value);
        self
    }
    pub fn readonly(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.inner = self.inner.readonly(value);
        self
    }
    pub fn invalid(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.inner = self.inner.invalid(value);
        self
    }
    pub fn value(mut self, value: impl Into<Vary<Vec<String>>>) -> Self {
        self.inner = self.inner.values(value);
        self
    }
    pub fn default_value(mut self, value: Vec<String>) -> Self {
        self.inner = self.inner.default_values(value);
        self
    }
    pub fn search(mut self, value: impl Into<Vary<String>>) -> Self {
        self.inner = self.inner.search(value);
        self
    }
    pub fn default_search(mut self, value: impl Into<String>) -> Self {
        self.inner = self.inner.default_search(value);
        self
    }
    pub fn open(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.inner = self.inner.open(value);
        self
    }
    pub fn default_open(mut self, value: bool) -> Self {
        self.inner = self.inner.default_open(value);
        self
    }
    pub fn on_value_change(mut self, callback: impl FnMut(Vec<String>) + 'static) -> Self {
        self.inner = self.inner.on_values_change(callback);
        self
    }
    pub fn on_search_change(mut self, callback: impl FnMut(String) + 'static) -> Self {
        self.inner = self.inner.on_search_change(callback);
        self
    }
    pub fn on_search_commit(mut self, callback: impl FnMut(String) + 'static) -> Self {
        self.inner = self.inner.on_search_commit(callback);
        self
    }
    pub fn on_open_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.inner = self.inner.on_open_change(callback);
        self
    }
    pub fn filter(mut self, value: ComboboxFilter) -> Self {
        self.inner = self.inner.filter(value);
        self
    }
    pub fn empty_message(mut self, value: impl Into<String>) -> Self {
        self.inner = self.inner.empty_message(value);
        self
    }
    pub fn option(mut self, value: NativeOption) -> Self {
        self.inner = self.inner.option(value);
        self
    }
    pub fn options(mut self, values: impl IntoIterator<Item = NativeOption>) -> Self {
        self.inner = self.inner.options(values);
        self
    }
    pub fn group(mut self, value: NativeOptionGroup) -> Self {
        self.inner = self.inner.group(value);
        self
    }
}
impl Widget for MultiSelect {
    fn build(&mut self, scope: &Scope) -> Result {
        self.inner.build(scope)
    }
}
