//! Compact single-value selection over the same genuine RadioGroup protocol.
use crate::radio::{Radio, RadioGroup, radio, radio_group};
use glory::render::{Event, EventKind, FocusMode, NodeId, Orientation};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};

pub struct Segment {
    inner: Radio,
    class: Lotus<String>,
}
pub fn segment(value: impl Into<String>) -> Segment {
    Segment {
        inner: radio(value),
        class: "".into(),
    }
}
impl Segment {
    pub fn label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.inner = self.inner.label(value);
        self
    }
    pub fn enabled(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.inner = self.inner.enabled(value);
        self
    }
    pub fn name(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.inner = self.inner.name(value);
        self
    }
    pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Lotus<glory::style::Style>>) -> Self {
        self.inner = self.inner.style(value);
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.inner = self.inner.node_ref(value);
        self
    }
    pub fn on(mut self, kind: EventKind, handler: impl FnMut(Event) + 'static) -> Self {
        self.inner = self.inner.on(kind, handler);
        self
    }
}

pub struct SegmentedControl {
    inner: Option<RadioGroup>,
    class: Lotus<String>,
    segments: Vec<Segment>,
}
pub fn segmented_control(key: impl Into<String>) -> SegmentedControl {
    SegmentedControl {
        inner: Some(radio_group(key).orientation(Orientation::Horizontal)),
        class: "".into(),
        segments: Vec::new(),
    }
}
impl SegmentedControl {
    fn map(mut self, update: impl FnOnce(RadioGroup) -> RadioGroup) -> Self {
        self.inner = self.inner.take().map(update);
        self
    }
    pub fn theme(self, value: crate::Theme) -> Self {
        self.map(|group| group.theme(value))
    }
    pub fn size(self, value: crate::Size) -> Self {
        self.map(|group| group.size(value))
    }
    pub fn unstyled(self) -> Self {
        self.map(RadioGroup::unstyled)
    }
    pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.class = value.into();
        self
    }
    pub fn style(self, value: impl Into<Lotus<glory::style::Style>>) -> Self {
        self.map(|group| group.style(value))
    }
    pub fn name(self, value: impl Into<Lotus<String>>) -> Self {
        self.map(|group| group.name(value))
    }
    pub fn label(self, value: impl Into<Lotus<String>>) -> Self {
        self.map(|group| group.label(value))
    }
    pub fn enabled(self, value: impl Into<Lotus<bool>>) -> Self {
        self.map(|group| group.enabled(value))
    }
    /// The group itself owns the member tab stops; Auto is the valid root mode.
    pub fn focus(self, value: FocusMode) -> Self {
        self.map(|group| group.focus(value))
    }
    pub fn node_ref(self, value: Cage<Option<NodeId>>) -> Self {
        self.map(|group| group.node_ref(value))
    }
    pub fn on(self, kind: EventKind, handler: impl FnMut(Event) + 'static) -> Self {
        self.map(|group| group.on(kind, handler))
    }
    pub fn field(self, value: crate::FieldControl) -> Self {
        self.map(|group| group.field(value))
    }
    pub fn form_name(self, value: impl Into<String>) -> Self {
        self.map(|group| group.form_name(value))
    }
    pub fn value(self, value: impl Into<Lotus<String>>) -> Self {
        self.map(|group| group.value(value))
    }
    pub fn default_value(self, value: impl Into<String>) -> Self {
        self.map(|group| group.default_value(value))
    }
    pub fn required(self, value: impl Into<Lotus<bool>>) -> Self {
        self.map(|group| group.required(value))
    }
    pub fn readonly(self, value: impl Into<Lotus<bool>>) -> Self {
        self.map(|group| group.readonly(value))
    }
    pub fn invalid(self, value: impl Into<Lotus<bool>>) -> Self {
        self.map(|group| group.invalid(value))
    }
    pub fn on_value_change(self, handler: impl FnMut(String) + 'static) -> Self {
        self.map(|group| group.on_value_change(handler))
    }
    pub fn orientation(self, value: Orientation) -> Self {
        self.map(|group| group.orientation(value))
    }
    pub fn loop_focus(self, value: bool) -> Self {
        self.map(|group| group.loop_focus(value))
    }
    pub fn direction(self, value: glory::render::Direction) -> Self {
        self.map(|group| group.direction(value))
    }
    pub fn typeahead(self, value: crate::TypeaheadConfig) -> Self {
        self.map(|group| group.typeahead(value))
    }
    pub fn segment(mut self, value: Segment) -> Self {
        if self.segments.len() <= crate::radio::MAX_RADIO_GROUP_ITEMS {
            self.segments.push(value);
        }
        self
    }
    pub fn segments(mut self, values: impl IntoIterator<Item = Segment>) -> Self {
        let remaining =
            (crate::radio::MAX_RADIO_GROUP_ITEMS + 1).saturating_sub(self.segments.len());
        self.segments.extend(values.into_iter().take(remaining));
        self
    }
}
impl Widget for SegmentedControl {
    fn build(&mut self, scope: &Scope) -> Result {
        let class = self.class.clone();
        let mut group = self
            .inner
            .take()
            .ok_or(glory::ui::Error::Gone)?
            .class(scope.bond(move || format!("sapid-segmented-control {}", class.get())));
        for segment in self.segments.drain(..) {
            let class = segment.class;
            group = group.radio(
                segment
                    .inner
                    .class(scope.bond(move || format!("sapid-segment {}", class.get()))),
            );
        }
        group.show_in(scope)
    }
}
