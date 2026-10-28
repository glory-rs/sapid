//! A single current image inside the existing managed modal Layer.
use crate::{
    composite::{self, Callback},
    state::Value,
};
use glory::render::{Event, EventKind, FocusMode, Navigation, NodeId};
use glory::style::{
    Display, FlexDirection, FlexWrap, Length, Limit, ObjectFit, Overflow, Style,
};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};
use std::{collections::BTreeSet, rc::Rc};
pub const MAX_LIGHTBOX_IMAGES: usize = 128;
pub const MAX_LIGHTBOX_METADATA_BYTES: usize = 262_144;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LightboxImage {
    pub key: String,
    pub source: String,
    pub alternative: String,
    pub caption: String,
}
pub fn lightbox_image(
    key: impl Into<String>,
    source: impl Into<String>,
    alternative: impl Into<String>,
) -> LightboxImage {
    LightboxImage {
        key: key.into(),
        source: source.into(),
        alternative: alternative.into(),
        caption: String::new(),
    }
}
impl LightboxImage {
    pub fn caption(mut self, value: impl Into<String>) -> Self {
        self.caption = value.into();
        self
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LightboxZoom {
    #[default]
    Fit,
    Two,
    Three,
    Four,
}
impl LightboxZoom {
    pub fn factor(self) -> u8 {
        match self {
            Self::Fit => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
        }
    }
    fn change(self, up: bool) -> Self {
        match (self, up) {
            (Self::Fit, true) => Self::Two,
            (Self::Two, true) => Self::Three,
            (Self::Three, true) | (Self::Four, true) => Self::Four,
            (Self::Four, false) => Self::Three,
            (Self::Three, false) => Self::Two,
            _ => Self::Fit,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LightboxView {
    pub current: Option<String>,
    pub zoom: LightboxZoom,
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn label(value: &str, max: usize, empty: bool) -> bool {
    value.len() <= max && !value.contains(['\0', '\r']) && (empty || !value.trim().is_empty())
}
fn validate(images: &[LightboxImage], view: &LightboxView) -> Result {
    if images.len() > MAX_LIGHTBOX_IMAGES {
        return Err(invalid("lightbox admits at most 128 images"));
    }
    let mut keys = BTreeSet::new();
    let mut bytes = 0;
    for image in images {
        if image.key.is_empty()
            || image.key.len() > 64
            || !image
                .key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-.:".contains(&b))
            || !keys.insert(image.key.as_str())
        {
            return Err(invalid("lightbox requires distinct finite ASCII keys"));
        }
        crate::image::validate_source(&image.source)?;
        if image.source.contains(['\0', '\r', '\n']) {
            return Err(invalid(
                "lightbox source must be canonical single-line text",
            ));
        }
        if !label(&image.alternative, 1024, false) || !label(&image.caption, 2048, true) {
            return Err(invalid(
                "lightbox requires nonempty bounded alt text and canonical captions",
            ));
        }
        bytes += image.key.len()
            + image.source.len()
            + image.alternative.len()
            + image.caption.len();
        if bytes > MAX_LIGHTBOX_METADATA_BYTES {
            return Err(invalid("lightbox metadata exceeds its byte budget"));
        }
    }
    if view
        .current
        .as_ref()
        .is_some_and(|key| !keys.contains(key.as_str()))
    {
        return Err(invalid(
            "lightbox current key must belong to the supplied image snapshot",
        ));
    }
    Ok(())
}
#[derive(Clone, PartialEq)]
struct Model {
    images: Vec<LightboxImage>,
    view: LightboxView,
}
impl Model {
    fn index(&self) -> Option<usize> {
        self.view
            .current
            .as_ref()
            .and_then(|key| self.images.iter().position(|i| &i.key == key))
            .or_else(|| (!self.images.is_empty()).then_some(0))
    }
    fn image(&self) -> Option<&LightboxImage> {
        self.index().map(|i| &self.images[i])
    }
}
pub struct Lightbox {
    key: String,
    images: Vary<Vec<LightboxImage>>,
    view: Value<LightboxView>,
    open: Value<bool>,
    label: String,
    title: String,
    theme: crate::Theme,
    unstyled: bool,
    enabled: Vary<bool>,
    class: Vary<String>,
    style: Vary<Style>,
    panel: Option<Cage<Option<NodeId>>>,
    trigger: Option<Cage<Option<NodeId>>>,
    changed: Option<Box<dyn FnMut(LightboxView)>>,
    opened: Option<Box<dyn FnMut(bool)>>,
}
pub fn lightbox(key: impl Into<String>) -> Lightbox {
    Lightbox {
        key: key.into(),
        images: Vec::new().into(),
        view: Value::default(),
        open: Value::default(),
        label: "Open image gallery".into(),
        title: "Image gallery".into(),
        theme: crate::Theme::default(),
        unstyled: false,
        enabled: true.into(),
        class: String::new().into(),
        style: Style::new().into(),
        panel: None,
        trigger: None,
        changed: None,
        opened: None,
    }
}
impl Lightbox {
    pub fn images(mut self, value: impl Into<Vary<Vec<LightboxImage>>>) -> Self {
        self.images = value.into();
        self
    }
    pub fn view(mut self, value: impl Into<Vary<LightboxView>>) -> Self {
        self.view.source = Some(value.into());
        self
    }
    pub fn default_view(mut self, value: LightboxView) -> Self {
        self.view.default = value;
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
    pub fn label(mut self, value: impl Into<String>) -> Self {
        self.label = value.into();
        self
    }
    pub fn title(mut self, value: impl Into<String>) -> Self {
        self.title = value.into();
        self
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.theme = value;
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.unstyled = true;
        self
    }
    pub fn enabled(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.enabled = value.into();
        self
    }
    pub fn class(mut self, value: impl Into<Vary<String>>) -> Self {
        self.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Vary<Style>>) -> Self {
        self.style = value.into();
        self
    }
    pub fn panel_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.panel = Some(value);
        self
    }
    pub fn trigger_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.trigger = Some(value);
        self
    }
    pub fn on_view_change(mut self, value: impl FnMut(LightboxView) + 'static) -> Self {
        self.changed = Some(Box::new(value));
        self
    }
    pub fn on_open_change(mut self, value: impl FnMut(bool) + 'static) -> Self {
        self.opened = Some(Box::new(value));
        self
    }
}
#[derive(Clone)]
struct State {
    owner: glory::core::InstanceId,
    model: Cage<Rc<Model>>,
    images: Vary<Vec<LightboxImage>>,
    view: crate::state::Control<LightboxView>,
    open: Vary<bool>,
    enabled: Vary<bool>,
    changed: Callback<LightboxView>,
}
impl State {
    fn propose(&self, view: LightboxView) -> Result {
        if !self.owner.is_alive() || !self.enabled.get() || !self.open.get() {
            return Ok(());
        }
        self.images.with(|images| validate(images, &view))?;
        if self.view.source.get() == view {
            return Ok(());
        }
        self.view.accept(&view)?;
        self.changed.emit(self.owner, view);
        Ok(())
    }
    fn step(&self, command: Navigation) -> Result {
        let current = self.view.source.get();
        let view = self.images.with(|images| -> Result<Option<LightboxView>> {
            validate(images, &current)?;
            let Some(i) = current
                .current
                .as_ref()
                .and_then(|key| images.iter().position(|i| &i.key == key))
                .or_else(|| (!images.is_empty()).then_some(0))
            else {
                return Ok(None);
            };
            let next = match command {
                Navigation::Previous => i.saturating_sub(1),
                Navigation::Next => (i + 1).min(images.len() - 1),
                Navigation::First => 0,
                Navigation::Last => images.len() - 1,
                _ => return Ok(None),
            };
            Ok(Some(LightboxView {
                current: Some(images[next].key.clone()),
                zoom: current.zoom,
            }))
        })?;
        if let Some(view) = view {
            self.propose(view)?;
        }
        Ok(())
    }
    fn zoom(&self, up: Option<bool>) -> Result {
        let mut view = self.view.source.get();
        view.zoom = up.map_or(LightboxZoom::Fit, |up| view.zoom.change(up));
        self.propose(view)
    }
}
impl Widget for Lightbox {
    fn build(&mut self, scope: &Scope) -> Result {
        composite::id("lightbox", &self.key, "panel", "root")?;
        if !label(&self.label, 1024, false) || !label(&self.title, 1024, false) {
            return Err(invalid(
                "lightbox label/title must be nonempty bounded canonical text",
            ));
        }
        let view = std::mem::take(&mut self.view).mount(scope);
        let open = std::mem::take(&mut self.open).mount(scope);
        let source = self.images.clone();
        let selected = view.source.clone();
        let read = move || -> Result<Rc<Model>> {
            selected.try_with(|view| {
                source.try_with(|images| {
                    validate(images, view)?;
                    Ok(Rc::new(Model {
                        images: images.clone(),
                        view: view.clone(),
                    }))
                })?
            })?
        };
        let model = scope.cage(glory::core::untracked(&read)?);
        let weak = scope.downgrade();
        scope.structural(move || {
            let Some(scope) = weak.upgrade() else { return };
            match read() {
                Ok(next) => {
                    if glory::core::untracked(|| model.with(|m| **m != *next)) {
                        model.revise(|m| *m = next);
                    }
                }
                Err(error) => scope.report_error(error),
            }
        });
        let changed = Callback::new(self.changed.take());
        let opened = Callback::new(self.opened.take());
        let cleanup = changed.clone();
        let clean_open = opened.clone();
        scope.on_cleanup(move || {
            cleanup.clear();
            clean_open.clear();
        })?;
        let state = State {
            owner: scope.id(),
            model,
            images: self.images.clone(),
            view,
            open: open.source.clone(),
            enabled: self.enabled.clone(),
            changed,
        };
        let theme = self.theme.clone();
        let unstyled = self.unstyled;
        let enabled = self.enabled.clone();
        let state_content = state.clone();
        let owner = scope.id();
        let panel = self.panel.unwrap_or_else(|| scope.cage(None));
        let custom = self.style.clone();
        let panel_style = scope.bond(move || {
            crate::appearance::merge(
                Style::new()
                    .width(Length::Px(760.))
                    .max_width(Limit::Percent(95.))
                    .min_width(Length::Px(0.)),
                custom.get(),
            )
        });
        let mut dialog = crate::dialog(&self.key)
            .class(scope.bond({
                let class = self.class.clone();
                move || format!("sapid-lightbox {}", class.get())
            }))
            .label(self.label.clone())
            .title(self.title.clone())
            .open(open.source.clone())
            .enabled(enabled)
            .theme(theme.clone())
            .style(panel_style)
            .panel_ref(panel)
            .on_open_change(move |value| {
                if owner.is_alive() && open.accept(&value).is_ok() {
                    opened.emit(owner, value);
                }
            })
            .content(move || {
                let state = state_content.clone();
                let theme = theme.clone();
                glory::ui::widget(move |scope| build_content(scope, &state, &theme, unstyled))
            });
        if let Some(trigger) = self.trigger {
            dialog = dialog.trigger_ref(trigger);
        }
        if unstyled {
            dialog = dialog.unstyled();
        }
        dialog.show_in(scope)
    }
}
fn build_content(scope: &Scope, state: &State, theme: &crate::Theme, unstyled: bool) -> Result {
    let model = state.model;
    let root = scope.cage(None);
    glory::ui::view()
        .class("sapid-lightbox-body")
        .style(
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(8.)
                .min_width(Length::Px(0.)),
        )
        .node_ref(root)
        .show_in(scope)?;
    let area = scope.child_in(root.get().ok_or(glory::ui::Error::Gone)?)?;
    let viewport = area.cage(None);
    let event_state = state.clone();
    let weak = area.downgrade();
    glory::ui::view()
        .class("sapid-lightbox-viewport")
        .name("Image navigation")
        .focus(FocusMode::TabStop)
        .semantics({
            let enabled = state.enabled.clone();
            area.bond(move || {
                Some(Box::new(glory::render::Semantics {
                    navigation: Some(glory::render::Orientation::Horizontal),
                    disabled: !enabled.get(),
                    ..Default::default()
                }))
            })
        })
        .node_ref(viewport)
        .style(
            Style::new()
                .width(Length::Percent(100.))
                .height(Length::Px(320.))
                .min_width(Length::Px(0.))
                .overflow_x(Overflow::Auto)
                .overflow_y(Overflow::Auto),
        )
        .on(EventKind::Navigate, move |event| {
            if let Event::Navigate { from, command } = event
                && viewport.get() == Some(from)
                && let Err(error) = event_state.step(command)
                && let Some(scope) = weak.upgrade()
            {
                scope.report_error(error);
            }
        })
        .show_in(&area)?;
    let canvas = area.child_in(viewport.get().ok_or(glory::ui::Error::Gone)?)?;
    let exists = canvas.bond(move || model.with(|m| m.image().is_some()));
    let theme_image = theme.clone();
    glory::ui::when(exists, move || {
        let theme = theme_image.clone();
        glory::ui::widget(move |scope| {
            let status = scope.cage("Loading image".to_owned());
            let previous = std::cell::RefCell::new(None);
            scope.structural(move || {
                let source = model.with(|m| {
                    m.image()
                        .map(|i| (&i.key, &i.source))
                        .map(|(k, s)| (k.clone(), s.clone()))
                });
                if *previous.borrow() != source {
                    *previous.borrow_mut() = source;
                    status.revise(|v| *v = "Loading image".into());
                }
            });
            let mut image =
                crate::image(scope.bond(move || {
                    model.with(|m| m.image().expect("live image").source.clone())
                }))
                .alt(scope.bond(move || {
                    model.with(|m| m.image().expect("live image").alternative.clone())
                }))
                .class("sapid-lightbox-image")
                .theme(theme.clone())
                .fit(ObjectFit::Contain)
                .style(scope.bond(move || {
                    model.with(|m| {
                        let factor = f64::from(m.view.zoom.factor());
                        Style::new()
                            .width(Length::Percent(100. * factor))
                            .height(Length::Px(320. * factor))
                            .max_width(Limit::None)
                    })
                }))
                .on_load(move |_| status.revise(|v| v.clear()))
                .on_error(move |_| status.revise(|v| *v = "Image could not be loaded".into()));
            if unstyled {
                image = image.unstyled();
            }
            image.show_in(scope)?;
            glory::ui::text()
                .class("sapid-lightbox-status")
                .content(status)
                .show_in(scope)
        })
    })
    .show_in(&canvas)?;
    area.retain(canvas)?;
    glory::ui::text()
        .class("sapid-lightbox-caption")
        .content(area.bond(move || {
            model.with(|m| {
                m.image()
                    .map_or_else(|| "No images".into(), |i| i.caption.clone())
            })
        }))
        .show_in(&area)?;
    glory::ui::text()
        .class("sapid-lightbox-counter")
        .content(area.bond(move || {
            model.with(|m| {
                m.index().map_or_else(
                    || "0 of 0".into(),
                    |i| {
                        format!(
                            "{} of {} · {}%",
                            i + 1,
                            m.images.len(),
                            u16::from(m.view.zoom.factor()) * 100
                        )
                    },
                )
            })
        }))
        .show_in(&area)?;
    let controls = area.cage(None);
    glory::ui::view()
        .class("sapid-lightbox-controls")
        .style(
            Style::new()
                .display(Display::Flex)
                .flex_wrap(FlexWrap::Wrap)
                .gap(8.),
        )
        .node_ref(controls)
        .show_in(&area)?;
    let actions = area.child_in(controls.get().ok_or(glory::ui::Error::Gone)?)?;
    for (next, label) in [(false, "Previous image"), (true, "Next image")] {
        let enabled = state.enabled.clone();
        let action = state.clone();
        let available = actions.bond(move || {
            enabled.get()
                && model.with(|m| {
                    m.index()
                        .is_some_and(|i| if next { i + 1 < m.images.len() } else { i > 0 })
                })
        });
        let weak = actions.downgrade();
        let mut button = crate::button()
            .label(label)
            .theme(theme.clone())
            .enabled(available)
            .on_press(move || {
                if let Err(error) = action.step(if next {
                    Navigation::Next
                } else {
                    Navigation::Previous
                }) && let Some(scope) = weak.upgrade()
                {
                    scope.report_error(error);
                }
            });
        if unstyled {
            button = button.unstyled();
        }
        button.show_in(&actions)?;
    }
    for (zoom, label) in [
        (Some(false), "Zoom out"),
        (None, "Fit image"),
        (Some(true), "Zoom in"),
    ] {
        let action = state.clone();
        let enabled = state.enabled.clone();
        let available = actions.bond(move || {
            enabled.get()
                && model.with(|m| {
                    m.image().is_some()
                        && match zoom {
                            Some(true) => m.view.zoom != LightboxZoom::Four,
                            Some(false) | None => m.view.zoom != LightboxZoom::Fit,
                        }
                })
        });
        let weak = actions.downgrade();
        let mut button = crate::button()
            .label(label)
            .theme(theme.clone())
            .enabled(available)
            .on_press(move || {
                if let Err(error) = action.zoom(zoom)
                    && let Some(scope) = weak.upgrade()
                {
                    scope.report_error(error);
                }
            });
        if unstyled {
            button = button.unstyled();
        }
        button.show_in(&actions)?;
    }
    area.retain(actions)?;
    scope.retain(area)?;
    Ok(())
}
