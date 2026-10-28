//! Editable hexadecimal color drafts with the shared Popover/ColorPicker.
use crate::{
    appearance::{Common, observe},
    color_model::{Hsva, format_hex_color, parse_hex_color},
    composite::{self, Callback},
    state::Value,
};
use glory::render::{Direction, Event, EventKind, NodeId, Semantics};
use glory::style::Style;
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};
const MAX_DRAFT_BYTES: usize = 256;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorValidity {
    Empty,
    Valid(glory::style::Color),
    Invalid(crate::ColorError),
}
fn validity(value: &str) -> ColorValidity {
    if value.is_empty() {
        ColorValidity::Empty
    } else {
        match parse_hex_color(value) {
            Ok(value) => ColorValidity::Valid(value),
            Err(error) => ColorValidity::Invalid(error),
        }
    }
}
fn draft(value: &str) -> Result {
    if value.len() > MAX_DRAFT_BYTES || value.contains(['\0', '\r', '\n']) {
        Err(glory::render::Error::InvalidProps(
            "color drafts require at most 256 canonical single-line bytes",
        )
        .into())
    } else {
        Ok(())
    }
}
pub struct ColorInput {
    key: String,
    common: Common,
    value: Value<String>,
    field: Option<crate::FieldControl>,
    readonly: Vary<bool>,
    required: Vary<bool>,
    invalid: Vary<bool>,
    open: Value<bool>,
    open_change: Option<Box<dyn FnMut(bool)>>,
    change: Option<Box<dyn FnMut(String)>>,
    commit: Option<Box<dyn FnMut(String)>>,
    validation: Option<Box<dyn FnMut(ColorValidity)>>,
    alpha: bool,
    picker: bool,
    direction: Direction,
}
pub fn color_input(key: impl Into<String>) -> ColorInput {
    ColorInput {
        key: key.into(),
        common: Common::default(),
        value: Value {
            source: None,
            default: "#ff0000ff".into(),
        },
        field: None,
        readonly: false.into(),
        required: false.into(),
        invalid: false.into(),
        open: Value::default(),
        open_change: None,
        change: None,
        commit: None,
        validation: None,
        alpha: true,
        picker: true,
        direction: Direction::Ltr,
    }
}
impl ColorInput {
    pub fn value(mut self, value: impl Into<Vary<String>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.value.default = value.into();
        self
    }
    pub fn on_value_change(mut self, value: impl FnMut(String) + 'static) -> Self {
        self.change = Some(Box::new(value));
        self
    }
    pub fn on_commit(mut self, value: impl FnMut(String) + 'static) -> Self {
        self.commit = Some(Box::new(value));
        self
    }
    pub fn on_validation(mut self, value: impl FnMut(ColorValidity) + 'static) -> Self {
        self.validation = Some(Box::new(value));
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
    pub fn on_open_change(mut self, value: impl FnMut(bool) + 'static) -> Self {
        self.open_change = Some(Box::new(value));
        self
    }
    pub fn field(mut self, value: crate::FieldControl) -> Self {
        self.common.theme = value.theme.clone();
        self.common.size = value.size;
        self.common.unstyled = value.unstyled;
        self.field = Some(value);
        self
    }
    pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
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
    pub fn class(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Vary<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn enabled(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.common.enabled = value.into();
        self
    }
    pub fn readonly(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn required(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.required = value.into();
        self
    }
    pub fn invalid(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.invalid = value.into();
        self
    }
    pub fn direction(mut self, value: Direction) -> Self {
        self.direction = value;
        self
    }
    /// Explicit hex-editor-only mode for hosts without top-layer support.
    pub fn show_picker(mut self, value: bool) -> Self {
        self.picker = value;
        self
    }
    pub fn show_alpha(mut self, value: bool) -> Self {
        self.alpha = value;
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
impl Widget for ColorInput {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.key.len() > crate::MAX_COLOR_PICKER_KEY_BYTES - 2 {
            return Err(glory::render::Error::InvalidProps(
                "color input keys are limited to 46 bytes",
            )
            .into());
        }
        if !self.picker
            && (self.open.source.is_some() || self.open.default || self.open_change.is_some())
        {
            return Err(glory::render::Error::InvalidProps(
                "hex-only color input cannot configure a popup",
            )
            .into());
        }
        let root_id = composite::id("color-input", &self.key, "root", "root")?;
        let error_id = composite::id("color-input", &self.key, "error", "root")?;
        let control = std::mem::take(&mut self.value).mount(scope);
        let accepted = composite::validated(scope, &control.source, |value| draft(value))?;
        let status = scope.bond(move || accepted.with(|value| validity(value)));
        let palette = scope.cage(Hsva::default());
        let validation = Callback::new(self.validation.take());
        let change = Callback::new(self.change.take());
        let commit = Callback::new(self.commit.take());
        let opened = Callback::new(self.open_change.take());
        let owner = scope.id();
        let weak = scope.downgrade();
        let observed = validation.clone();
        let mut previous = None;
        scope.effect(move || {
            let next = status.get();
            if let ColorValidity::Valid(color) = next {
                let mut hsv = Hsva::from_rgba(color);
                let old = palette.get();
                if hsv.saturation == 0. {
                    hsv.hue = old.hue;
                }
                if hsv != old {
                    palette.revise(|value| *value = hsv);
                }
            }
            if previous != Some(next) {
                previous = Some(next);
                if weak.upgrade().is_some() {
                    glory::core::untracked(|| observed.emit(owner, next));
                }
            }
        });
        for callback in [change.clone(), commit.clone()] {
            scope.on_cleanup(move || callback.clear())?;
        }
        scope.on_cleanup(move || validation.clear())?;
        let cleanup = opened.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let open = std::mem::take(&mut self.open).mount(scope);
        let enabled = self.field.as_ref().map_or_else(
            || self.common.enabled.clone(),
            |field| field.enabled(scope, self.common.enabled.clone()),
        );
        let readonly = self.field.as_ref().map_or_else(
            || self.readonly.clone(),
            |field| field.readonly_state(scope, self.readonly.clone()),
        );
        let composing = scope.cage(false);
        let read = readonly.clone();
        let active = enabled.clone();
        let picker_enabled =
            scope.bond(move || active.get() && !read.get() && !composing.get());
        let parse_message = scope.bond(move || match status.get() {
            ColorValidity::Invalid(error) => error.to_string(),
            _ => String::new(),
        });
        let invalid = self.invalid.clone();
        let error = scope
            .bond(move || invalid.get() || matches!(status.get(), ColorValidity::Invalid(_)));
        let mut input = crate::text_field()
            .theme(self.common.theme.clone())
            .size(self.common.size)
            .value(accepted)
            .enabled(enabled.clone())
            .readonly(self.readonly.clone())
            .required(self.required.clone())
            .invalid(error)
            .class("sapid-color-editor")
            .style(self.common.style.clone())
            .additional_error(error_id.clone(), parse_message.into());
        if self.common.unstyled {
            input = input.unstyled();
        }
        if let Some(field) = self.field.take() {
            input = input.field(field);
        }
        if let Some(name) = self.common.name.take() {
            input = input.name(name);
        } else if input.field.is_none() {
            input = input.name("Color");
        }
        if let Some(reference) = self.common.node_ref {
            input = input.node_ref(reference);
        }
        let editing = control.clone();
        let changed = change.clone();
        let weak = scope.downgrade();
        input = input.on_value_change(move |value| {
            let result = draft(&value).and_then(|()| editing.accept(&value));
            match result {
                Ok(()) => changed.emit(owner, value),
                Err(error) => {
                    if let Some(scope) = weak.upgrade() {
                        scope.report_error(error)
                    }
                }
            }
        });
        let committed = commit.clone();
        input = input.on_commit(move |value| committed.emit(owner, value));
        let mut observers = self
            .common
            .handlers
            .remove(&EventKind::Composition)
            .unwrap_or_default();
        input = input.on(EventKind::Composition, move |event| {
            if let Event::Composition { active } = event {
                composing.revise(|value| *value = active);
            }
            observe(&mut observers, owner, event);
        });
        for (kind, mut observers) in std::mem::take(&mut self.common.handlers) {
            input = input.on(kind, move |event| observe(&mut observers, owner, event));
        }
        let extra = self.common.class.clone();
        let prefix = format!(
            "sapid-color-input {}{}",
            self.common.theme.class(),
            if self.common.unstyled {
                " sapid-unstyled"
            } else {
                ""
            }
        );
        let theme = self.common.theme.clone();
        let size = self.common.size;
        let alpha = self.alpha;
        let direction = self.direction;
        let unstyled = self.common.unstyled;
        let edited = control.clone();
        let on_change = change;
        let on_commit = commit;
        let ready = picker_enabled;
        let palette_ready = picker_enabled;
        let read_palette = palette;
        let raw = control.source.clone();
        let picker_value = scope.bond(move || {
            raw.with(|value| match parse_hex_color(value) {
                Ok(color) => {
                    let mut hsv = Hsva::from_rgba(color);
                    if hsv.saturation == 0. {
                        hsv.hue = read_palette.get().hue;
                    }
                    hsv
                }
                Err(_) => read_palette.get(),
            })
        });
        let picker_key = format!("{}-c", self.key);
        let open_scope = scope.downgrade();
        let palette_scope = scope.downgrade();
        let popup = crate::popover(format!("{}-p", self.key))
            .label("Choose color")
            .title("Choose color")
            .theme(theme.clone())
            .size(size)
            .enabled(ready)
            .open(open.source.clone())
            .on_open_change(move |value| {
                if let Err(error) = open.accept(&value)
                    && let Some(scope) = open_scope.upgrade()
                {
                    scope.report_error(error)
                }
                opened.emit(owner, value);
            })
            .content(move || {
                let source = picker_value;
                let editing = edited.clone();
                let changed = on_change.clone();
                let callback = on_commit.clone();
                let weak = palette_scope.clone();
                let mut picker = crate::color_picker(picker_key.clone())
                    .value(source)
                    .enabled(palette_ready)
                    .theme(theme.clone())
                    .size(size)
                    .direction(direction)
                    .show_alpha(alpha)
                    .on_value_change(move |value| {
                        let text = format_hex_color(
                            value.to_rgba().expect("validated picker value"),
                            alpha,
                        );
                        match editing.accept(&text) {
                            Ok(()) => changed.emit(owner, text),
                            Err(error) => {
                                if let Some(scope) = weak.upgrade() {
                                    scope.report_error(error)
                                }
                            }
                        }
                    })
                    .on_value_commit(move |value| {
                        let text = format_hex_color(
                            value.to_rgba().expect("validated picker value"),
                            alpha,
                        );
                        callback.emit(owner, text);
                    });
                if unstyled {
                    picker = picker.unstyled();
                }
                picker
            });
        let mut root = glory::ui::view()
            .class(scope.bond(move || format!("{prefix} {}", extra.get())))
            .style(composite::column(6.))
            .semantics(composite::metadata(Semantics {
                id: Some(root_id),
                ..Default::default()
            }))
            .fill(input);
        if self.picker {
            root = root.fill(popup);
        }
        root.fill(
            glory::ui::view()
                .class("sapid-color-validation")
                .semantics(scope.bond(move || {
                    Some(Box::new(Semantics {
                        id: Some(error_id.clone()),
                        hidden: parse_message.with(|value| value.is_empty()),
                        ..Default::default()
                    }))
                }))
                .fill(glory::ui::text().content(parse_message)),
        )
        .show_in(scope)
    }
}
