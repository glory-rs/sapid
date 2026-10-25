//! Finite HSV picker with a pointer plane and genuine keyboard Slider alternatives.
use crate::{
    appearance::Common,
    color_model::Hsva,
    composite::{self, Callback, Observers},
    state::{Control, Value},
};
use glory::render::{Direction, DragCapture, Event, EventKind, NodeId, Role, Semantics};
use glory::style::{Color, Length, PointerEvents, Position, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};
use std::rc::Rc;
mod pad;
const CELLS: usize = 16;
pub const MAX_COLOR_PICKER_KEY_BYTES: usize = 48;
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn validate_name(value: &str) -> Result {
    if value.trim().is_empty() || value.len() > 256 || value.contains(['\0', '\r', '\n']) {
        Err(invalid(
            "color names require 1..=256 canonical single-line bytes",
        ))
    } else {
        Ok(())
    }
}
pub(super) struct PickerState {
    owner: glory::core::InstanceId,
    control: Control<Hsva>,
    accepted: Cage<Hsva>,
    enabled: Lotus<bool>,
    readonly: Lotus<bool>,
    direction: Direction,
    change: Callback<Hsva>,
    commit: Callback<Hsva>,
    drag_events: Observers,
    range_events: Observers,
}
impl PickerState {
    fn current(&self) -> Hsva {
        let value = self.control.source.get();
        if value.validate().is_ok() {
            value
        } else {
            self.accepted.get()
        }
    }
    fn ready(&self) -> bool {
        self.owner.is_alive() && self.enabled.get() && !self.readonly.get()
    }
    fn propose(&self, value: Hsva, commit: bool) -> Result {
        if !self.ready() {
            return Ok(());
        }
        value
            .validate()
            .map_err(|_| invalid("invalid color proposal"))?;
        if self.current() != value {
            self.control.accept(&value)?;
            self.change.emit(self.owner, value);
        }
        if commit && self.ready() {
            self.commit.emit(self.owner, value);
        }
        Ok(())
    }
    fn channel(&self, index: u8, value: f64, commit: bool) -> Result {
        let mut next = self.current();
        match index {
            0 => next.hue = value,
            1 => next.saturation = value,
            2 => next.value = value,
            _ => next.alpha = value,
        };
        if commit {
            if self.ready() {
                self.commit.emit(self.owner, next);
            }
            Ok(())
        } else {
            self.propose(next, false)
        }
    }
}
pub struct ColorPicker {
    key: String,
    common: Common,
    value: Value<Hsva>,
    readonly: Lotus<bool>,
    direction: Direction,
    alpha: bool,
    side: f64,
    change: Option<Box<dyn FnMut(Hsva)>>,
    commit: Option<Box<dyn FnMut(Hsva)>>,
}
pub fn color_picker(key: impl Into<String>) -> ColorPicker {
    ColorPicker {
        key: key.into(),
        common: Common::default(),
        value: Value {
            source: None,
            default: Hsva::default(),
        },
        readonly: false.into(),
        direction: Direction::Ltr,
        alpha: true,
        side: 240.,
        change: None,
        commit: None,
    }
}
impl ColorPicker {
    pub fn value(mut self, value: impl Into<Lotus<Hsva>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: Hsva) -> Self {
        self.value.default = value;
        self
    }
    pub fn on_value_change(mut self, value: impl FnMut(Hsva) + 'static) -> Self {
        self.change = Some(Box::new(value));
        self
    }
    pub fn on_value_commit(mut self, value: impl FnMut(Hsva) + 'static) -> Self {
        self.commit = Some(Box::new(value));
        self
    }
    pub fn name(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn size(mut self, value: crate::Size) -> Self {
        self.common.size = value;
        self
    }
    pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn enabled(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.common.enabled = value.into();
        self
    }
    pub fn readonly(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn direction(mut self, value: Direction) -> Self {
        self.direction = value;
        self
    }
    pub fn show_alpha(mut self, value: bool) -> Self {
        self.alpha = value;
        self
    }
    /// Intrinsic plane side; custom outer style does not turn the plane into a zero-height box.
    pub fn plane_size(mut self, value: f64) -> Self {
        self.side = value;
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
    pub fn on(mut self, kind: EventKind, value: impl FnMut(Event) + 'static) -> Self {
        self.common
            .handlers
            .entry(kind)
            .or_default()
            .push(Box::new(value));
        self
    }
}
fn surface(color: Color) -> Style {
    Style {
        background: Some(color),
        ..Style::new()
    }
}
fn channel(state: Rc<PickerState>, scope: &Scope, index: u8) -> Lotus<f64> {
    scope
        .bond(move || {
            let value = state.current();
            match index {
                0 => value.hue,
                1 => value.saturation,
                2 => value.value,
                _ => value.alpha,
            }
        })
        .into()
}
impl Widget for ColorPicker {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.key.len() > MAX_COLOR_PICKER_KEY_BYTES {
            return Err(invalid("color picker keys are limited to 48 bytes"));
        }
        if !self.side.is_finite() || !(128. ..=512.).contains(&self.side) {
            return Err(invalid("color plane side requires 128..=512 pixels"));
        }
        let id = composite::id("color-picker", &self.key, "root", "root")?;
        let control = std::mem::replace(
            &mut self.value,
            Value {
                source: None,
                default: Hsva::default(),
            },
        )
        .mount(scope);
        let accepted = composite::validated(scope, &control.source, |value| {
            value.validate().map_err(|_| invalid("invalid HSV source"))
        })?;
        let name = self.common.name.take().unwrap_or_else(|| "Color".into());
        let name = composite::validated(scope, &name, |value| validate_name(value))?;
        let state = Rc::new(PickerState {
            owner: scope.id(),
            control,
            accepted,
            enabled: self.common.enabled.clone(),
            readonly: self.readonly.clone(),
            direction: self.direction,
            change: Callback::new(self.change.take()),
            commit: Callback::new(self.commit.take()),
            drag_events: Observers::new(
                self.common
                    .handlers
                    .remove(&EventKind::Drag)
                    .unwrap_or_default(),
            ),
            range_events: Observers::new(
                self.common
                    .handlers
                    .remove(&EventKind::Range)
                    .unwrap_or_default(),
            ),
        });
        let cleanup = state.clone();
        scope.on_cleanup(move || {
            cleanup.change.clear();
            cleanup.commit.clear();
            cleanup.drag_events.clear();
            cleanup.range_events.clear();
        })?;
        let (class, style) = self
            .common
            .appearance(scope, "sapid-color-picker", Style::new());
        let side = self.side;
        let style = scope.bond(move || {
            let mut value = style.get();
            if value.width.is_none() {
                value.width = Some(Length::Px(side));
            }
            crate::appearance::merge(composite::column(8.), value)
        });
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let mut root = glory::ui::view()
            .role(Role::Group)
            .name(name)
            .class(class)
            .style(style)
            .node_ref(reference)
            .semantics(composite::metadata(Semantics {
                id: Some(id),
                ..Default::default()
            }));
        for (kind, mut callbacks) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |event| {
                crate::appearance::observe(&mut callbacks, owner, event)
            });
        }
        root.show_in(scope)?;
        let parent = scope.child_in(reference.get().ok_or(glory::ui::Error::Gone)?)?;
        let pad_ref = scope.cage(None);
        let driver = pad::Pad::new(scope, state.clone(), pad_ref)?;
        let enabled = self.common.enabled.clone();
        let readonly = self.readonly.clone();
        let weak = scope.downgrade();
        let drag = driver.clone();
        glory::ui::view()
            .class("sapid-color-plane")
            .node_ref(pad_ref)
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    drag_capture: Some(DragCapture::Press),
                    disabled: !enabled.get() || readonly.get(),
                    ..Default::default()
                }))
            }))
            .style(
                Style::new()
                    .position(Position::Relative)
                    .width(Length::Percent(100.))
                    .height(Length::Px(side))
                    .min_height(Length::Px(side)),
            )
            .on(EventKind::Drag, move |event| {
                if let Err(error) = drag.event(event)
                    && let Some(scope) = weak.upgrade()
                {
                    scope.report_error(error)
                }
            })
            .show_in(&parent)?;
        let plane = scope.child_in(pad_ref.get().ok_or(glory::ui::Error::Gone)?)?;
        for row in 0..CELLS {
            for column in 0..CELLS {
                let read = state.clone();
                let direction = self.direction;
                glory::ui::view()
                    .class("sapid-color-plane-cell")
                    .style(scope.bond(move || {
                        let hue = read.current().hue;
                        let value = Hsva {
                            hue,
                            saturation: column as f64 / (CELLS - 1) as f64,
                            value: 1. - row as f64 / (CELLS - 1) as f64,
                            alpha: 1.,
                        }
                        .to_rgba()
                        .expect("validated color cell");
                        surface(value)
                            .position(Position::Absolute)
                            .left(Length::Percent(
                                (if direction == Direction::Rtl {
                                    CELLS - column - 1
                                } else {
                                    column
                                }) as f64
                                    * 100.
                                    / CELLS as f64,
                            ))
                            .top(Length::Percent(row as f64 * 100. / CELLS as f64))
                            .width(Length::Percent(100. / CELLS as f64))
                            .height(Length::Percent(100. / CELLS as f64))
                            .pointer_events(PointerEvents::None)
                    }))
                    .show_in(&plane)?;
            }
        }
        let read = state.clone();
        let marker_border = self.common.theme.control_thumb;
        glory::ui::view()
            .class("sapid-color-plane-indicator")
            .style(scope.bond(move || {
                let value = read.current();
                let x = if read.direction == Direction::Rtl {
                    1. - value.saturation
                } else {
                    value.saturation
                };
                surface(value.to_rgba().expect("validated color indicator"))
                    .position(Position::Absolute)
                    .left(Length::Percent(x * 100.))
                    .top(Length::Percent((1. - value.value) * 100.))
                    .width(Length::Px(10.))
                    .height(Length::Px(10.))
                    .margin(glory::style::Edges {
                        top: -5.,
                        left: -5.,
                        right: 0.,
                        bottom: 0.,
                    })
                    .radius(5.)
                    .border_width(glory::style::Edges::all(2.))
                    .border_style(glory::style::BorderStyle::Solid)
                    .border_color(marker_border)
                    .pointer_events(PointerEvents::None)
            }))
            .show_in(&plane)?;
        for index in [1, 2] {
            let value = channel(state.clone(), scope, index);
            let changed = state.clone();
            let committed = state.clone();
            let weak = scope.downgrade();
            let weak_commit = scope.downgrade();
            let observe_range = state.clone();
            let observe_drag = state.clone();
            let label = if index == 1 {
                "Saturation"
            } else {
                "Brightness"
            };
            let mut slider = crate::slider(format!("{}-{index}", self.key))
                .value(value)
                .min(0.)
                .max(1.)
                .step(0.01)
                .name(label)
                .theme(self.common.theme.clone())
                .size(self.common.size)
                .style(Style::new().width(Length::Percent(100.)))
                .enabled(self.common.enabled.clone())
                .readonly(self.readonly.clone())
                .direction(self.direction)
                .on(EventKind::Range, move |event| {
                    observe_range.range_events.emit(observe_range.owner, event)
                })
                .on(EventKind::Drag, move |event| {
                    observe_drag.drag_events.emit(observe_drag.owner, event)
                })
                .on_value_change(move |value| {
                    if let Err(error) = changed.channel(index, value, false)
                        && let Some(scope) = weak.upgrade()
                    {
                        scope.report_error(error)
                    }
                })
                .on_value_commit(move |value| {
                    if let Err(error) = committed.channel(index, value, true)
                        && let Some(scope) = weak_commit.upgrade()
                    {
                        scope.report_error(error)
                    }
                });
            if self.common.unstyled {
                slider = slider.unstyled();
            }
            slider.show_in(&parent)?;
        }
        let changed = state.clone();
        let committed = state.clone();
        let weak = scope.downgrade();
        let weak_commit = scope.downgrade();
        let observe_range = state.clone();
        let observe_drag = state.clone();
        let mut hue = crate::hue_slider(format!("{}-h", self.key))
            .value(channel(state.clone(), scope, 0))
            .theme(self.common.theme.clone())
            .size(self.common.size)
            .style(Style::new().width(Length::Percent(100.)))
            .enabled(self.common.enabled.clone())
            .readonly(self.readonly.clone())
            .direction(self.direction)
            .on(EventKind::Range, move |event| {
                observe_range.range_events.emit(observe_range.owner, event)
            })
            .on(EventKind::Drag, move |event| {
                observe_drag.drag_events.emit(observe_drag.owner, event)
            })
            .on_value_change(move |value| {
                if let Err(error) = changed.channel(0, value, false)
                    && let Some(scope) = weak.upgrade()
                {
                    scope.report_error(error)
                }
            })
            .on_value_commit(move |value| {
                if let Err(error) = committed.channel(0, value, true)
                    && let Some(scope) = weak_commit.upgrade()
                {
                    scope.report_error(error)
                }
            });
        if self.common.unstyled {
            hue = hue.unstyled();
        }
        hue.show_in(&parent)?;
        if self.alpha {
            let preview = state.clone();
            let color = scope.bond(move || {
                preview
                    .current()
                    .to_rgba()
                    .expect("validated color preview")
            });
            let changed = state.clone();
            let committed = state.clone();
            let weak = scope.downgrade();
            let weak_commit = scope.downgrade();
            let observe_range = state.clone();
            let observe_drag = state.clone();
            let mut alpha = crate::alpha_slider(format!("{}-a", self.key))
                .value(channel(state.clone(), scope, 3))
                .color(color)
                .theme(self.common.theme.clone())
                .size(self.common.size)
                .style(Style::new().width(Length::Percent(100.)))
                .enabled(self.common.enabled.clone())
                .readonly(self.readonly.clone())
                .direction(self.direction)
                .on(EventKind::Range, move |event| {
                    observe_range.range_events.emit(observe_range.owner, event)
                })
                .on(EventKind::Drag, move |event| {
                    observe_drag.drag_events.emit(observe_drag.owner, event)
                })
                .on_value_change(move |value| {
                    if let Err(error) = changed.channel(3, value, false)
                        && let Some(scope) = weak.upgrade()
                    {
                        scope.report_error(error)
                    }
                })
                .on_value_commit(move |value| {
                    if let Err(error) = committed.channel(3, value, true)
                        && let Some(scope) = weak_commit.upgrade()
                    {
                        scope.report_error(error)
                    }
                });
            if self.common.unstyled {
                alpha = alpha.unstyled();
            }
            alpha.show_in(&parent)?;
        }
        scope.retain(plane)?;
        scope.retain(parent)?;
        Ok(())
    }
}
