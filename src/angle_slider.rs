//! A clockwise degree slider with a true two-dimensional circular mapping.
use crate::{FieldControl, Size, Slider, Theme};
use glory::render::{Direction, Event, EventKind, NodeId};
use glory::style::Style;
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus};
/// Degrees are finite in 0..=360. Both endpoint values share the top position;
/// arithmetic only wraps after crossing the endpoint, preserving Home/End.
pub struct AngleSlider {
    inner: Slider,
}
pub fn angle_slider(key: impl Into<String>) -> AngleSlider {
    AngleSlider {
        inner: crate::slider(key).angular(),
    }
}
impl AngleSlider {
    pub fn value(mut self, value: impl Into<Lotus<f64>>) -> Self {
        self.inner = self.inner.value(value);
        self
    }
    pub fn default_value(mut self, value: f64) -> Self {
        self.inner = self.inner.default_value(value);
        self
    }
    pub fn step(mut self, value: impl Into<Lotus<f64>>) -> Self {
        self.inner = self.inner.step(value);
        self
    }
    pub fn page_step(mut self, value: impl Into<Lotus<f64>>) -> Self {
        self.inner = self.inner.page_step(value);
        self
    }
    pub fn value_text(mut self, value: impl Into<Lotus<Option<String>>>) -> Self {
        self.inner = self.inner.value_text(value);
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
    pub fn name(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.inner = self.inner.name(value);
        self
    }
    pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.inner = self.inner.class(value);
        self
    }
    pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
        self.inner = self.inner.style(value);
        self
    }
    pub fn enabled(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.inner = self.inner.enabled(value);
        self
    }
    pub fn readonly(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.inner = self.inner.readonly(value);
        self
    }
    pub fn required(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.inner = self.inner.required(value);
        self
    }
    pub fn invalid(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.inner = self.inner.invalid(value);
        self
    }
    pub fn direction(mut self, value: Direction) -> Self {
        self.inner = self.inner.direction(value);
        self
    }
    pub fn field(mut self, value: FieldControl) -> Self {
        self.inner = self.inner.field(value);
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.inner = self.inner.node_ref(value);
        self
    }
    pub fn on_value_change(mut self, value: impl FnMut(f64) + 'static) -> Self {
        self.inner = self.inner.on_value_change(value);
        self
    }
    pub fn on_value_commit(mut self, value: impl FnMut(f64) + 'static) -> Self {
        self.inner = self.inner.on_value_commit(value);
        self
    }
    pub fn on(mut self, kind: EventKind, value: impl FnMut(Event) + 'static) -> Self {
        self.inner = self.inner.on(kind, value);
        self
    }
}
impl Widget for AngleSlider {
    fn build(&mut self, scope: &glory::Scope) -> Result {
        self.inner.build(scope)
    }
}
