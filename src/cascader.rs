//! Column presentation of finite full-path choices, with real Tree navigation.
use crate::{TreeChoices, TreeNode, TreeSelect};
use glory::render::{Direction, Event, EventKind, FocusMode, NodeId};
use glory::style::Style;
use glory::ui::{Result, Widget};
use glory::{Cage, Vary};
use std::collections::BTreeSet;

/// Expansion is one complete ancestor chain. Each current sibling set is
/// limited to 64 entries; the whole source retains Tree's 2048-node budget.
pub struct Cascader {
    inner: TreeSelect,
}
pub fn cascader(key: impl Into<String>) -> Cascader {
    Cascader {
        inner: crate::tree_select(key)
            .cascading_columns()
            .class("sapid-cascader"),
    }
}
macro_rules! forward {
    ($($name:ident($argument:ident:$ty:ty)),* $(,)?) => {$ (
        pub fn $name(mut self, $argument:$ty)->Self {
            self.inner=self.inner.$name($argument);self
        }
    )*};
}
impl Cascader {
    forward!(theme(v:crate::Theme),size(v:crate::Size),focus(v:FocusMode),node_ref(v:Cage<Option<NodeId>>),
        default_value(v:TreeChoices),default_open(v:bool),default_expanded(v:BTreeSet<String>),
        direction(v:Direction),clearable(v:bool),field(v:crate::FieldControl));
    pub fn nodes(mut self, v: impl Into<Vary<Vec<TreeNode>>>) -> Self {
        self.inner = self.inner.nodes(v);
        self
    }
    pub fn value(mut self, v: impl Into<Vary<TreeChoices>>) -> Self {
        self.inner = self.inner.value(v);
        self
    }
    pub fn open(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.inner = self.inner.open(v);
        self
    }
    pub fn expanded(mut self, v: impl Into<Vary<BTreeSet<String>>>) -> Self {
        self.inner = self.inner.expanded(v);
        self
    }
    pub fn enabled(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.inner = self.inner.enabled(v);
        self
    }
    pub fn readonly(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.inner = self.inner.readonly(v);
        self
    }
    pub fn required(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.inner = self.inner.required(v);
        self
    }
    pub fn invalid(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.inner = self.inner.invalid(v);
        self
    }
    pub fn name(mut self, v: impl Into<Vary<String>>) -> Self {
        self.inner = self.inner.name(v);
        self
    }
    pub fn class(mut self, v: impl Into<Vary<String>>) -> Self {
        self.inner = self.inner.class(v);
        self
    }
    pub fn style(mut self, v: impl Into<Vary<Style>>) -> Self {
        self.inner = self.inner.style(v);
        self
    }
    pub fn form_name(mut self, v: impl Into<Vary<String>>) -> Self {
        self.inner = self.inner.form_name(v);
        self
    }
    pub fn placeholder(mut self, v: impl Into<String>) -> Self {
        self.inner = self.inner.placeholder(v);
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.inner = self.inner.unstyled();
        self
    }
    pub fn on_value_change(mut self, v: impl FnMut(TreeChoices) + 'static) -> Self {
        self.inner = self.inner.on_value_change(v);
        self
    }
    pub fn on_open_change(mut self, v: impl FnMut(bool) + 'static) -> Self {
        self.inner = self.inner.on_open_change(v);
        self
    }
    pub fn on_expanded_change(mut self, v: impl FnMut(BTreeSet<String>) + 'static) -> Self {
        self.inner = self.inner.on_expanded_change(v);
        self
    }
    pub fn on_load_children(mut self, v: impl FnMut(String) + 'static) -> Self {
        self.inner = self.inner.on_load_children(v);
        self
    }
    pub fn on(mut self, kind: EventKind, v: impl FnMut(Event) + 'static) -> Self {
        self.inner = self.inner.on(kind, v);
        self
    }
}
impl Widget for Cascader {
    fn build(&mut self, scope: &glory::Scope) -> Result {
        self.inner.build(scope)
    }
}
