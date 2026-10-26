//! The free tag editor with pill-shaped accepted values.
use crate::tags_input::{TagDuplicatePolicy, TagRejection, TagsInput, TagsInputState};
use crate::{FieldControl, Size, Theme};
use glory::render::{Event, EventKind, FocusMode, NodeId, TagSeparators};
use glory::style::Style;
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};

pub struct PillsInput {
    inner: TagsInput,
}
pub fn pills_input(key: impl Into<String>) -> PillsInput {
    PillsInput {
        inner: crate::tags_input::tags_input(key).pills(),
    }
}
macro_rules! forward {
    ($name:ident,$ty:ty) => {
        pub fn $name(mut self, value: $ty) -> Self {
            self.inner = self.inner.$name(value);
            self
        }
    };
}
impl PillsInput {
    forward!(theme, Theme);
    forward!(size, Size);
    forward!(class, impl Into<Lotus<String>>);
    forward!(style, impl Into<Lotus<Style>>);
    forward!(name, impl Into<Lotus<String>>);
    forward!(placeholder, impl Into<Lotus<String>>);
    forward!(label, impl Into<Lotus<String>>);
    forward!(enabled, impl Into<Lotus<bool>>);
    forward!(focus, FocusMode);
    forward!(node_ref, Cage<Option<NodeId>>);
    forward!(field, FieldControl);
    forward!(form_name, impl Into<Lotus<String>>);
    forward!(required, impl Into<Lotus<bool>>);
    forward!(readonly, impl Into<Lotus<bool>>);
    forward!(invalid, impl Into<Lotus<bool>>);
    forward!(state, impl Into<Lotus<TagsInputState>>);
    forward!(default_value, Vec<String>);
    forward!(default_search, impl Into<String>);
    forward!(max_tags, usize);
    forward!(duplicate_policy, TagDuplicatePolicy);
    forward!(separators, TagSeparators);
    pub fn unstyled(mut self) -> Self {
        self.inner = self.inner.unstyled();
        self
    }
    pub fn on(mut self, kind: EventKind, callback: impl FnMut(Event) + 'static) -> Self {
        self.inner = self.inner.on(kind, callback);
        self
    }
    pub fn on_change(mut self, callback: impl FnMut(TagsInputState) + 'static) -> Self {
        self.inner = self.inner.on_change(callback);
        self
    }
    pub fn on_commit(mut self, callback: impl FnMut(Vec<String>) + 'static) -> Self {
        self.inner = self.inner.on_commit(callback);
        self
    }
    pub fn on_remove(mut self, callback: impl FnMut(String) + 'static) -> Self {
        self.inner = self.inner.on_remove(callback);
        self
    }
    pub fn on_reject(mut self, callback: impl FnMut(TagRejection) + 'static) -> Self {
        self.inner = self.inner.on_reject(callback);
        self
    }
}
impl Widget for PillsInput {
    fn build(&mut self, scope: &Scope) -> Result {
        self.inner.build(scope)
    }
}
