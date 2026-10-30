//! Actual semantic images with host-owned request completion and bounded retry.
use crate::appearance::{Common, configure, observe};
use glory::render::{Event, EventKind, FocusMode, ImageStatus, NodeId};
use glory::style::{Display, Length, Limit, ObjectFit, Overflow, Position, Style};
use glory::ui::{IntoChildren, Result, Widget};
use glory::{Cage, Scope, Vary};

pub const MAX_IMAGE_SOURCE_BYTES: usize = 4096;
pub(crate) fn validate_source(source: &str) -> Result {
    if source.trim().is_empty() || source.len() > MAX_IMAGE_SOURCE_BYTES {
        return Err(glory::render::Error::InvalidProps(
            "image source must be nonempty and at most 4096 UTF-8 bytes",
        )
        .into());
    }
    Ok(())
}
#[derive(Clone, PartialEq)]
struct Selection {
    primary: String,
    current: String,
    fallback: bool,
}
type Completion = Option<Box<dyn FnMut(u64)>>;
pub struct Image {
    common: Common,
    source: Vary<String>,
    alternative: Option<Vary<String>>,
    decorative: bool,
    fallback: Option<String>,
    fit: Option<ObjectFit>,
    width: Option<Length>,
    height: Option<Length>,
    loaded: Completion,
    failed: Completion,
}
pub fn image(source: impl Into<Vary<String>>) -> Image {
    Image {
        common: Common::default(),
        source: source.into(),
        alternative: None,
        decorative: false,
        fallback: None,
        fit: None,
        width: None,
        height: None,
        loaded: None,
        failed: None,
    }
}
macro_rules! appearance {
    () => {
        pub fn theme(mut self, value: crate::Theme) -> Self {
            self.common.theme = value;
            self
        }
        pub fn unstyled(mut self) -> Self {
            self.common.unstyled = true;
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
        pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
            self.common.node_ref = Some(value);
            self
        }
        pub fn width(mut self, value: Length) -> Self {
            self.width = Some(value);
            self
        }
        pub fn height(mut self, value: Length) -> Self {
            self.height = Some(value);
            self
        }
    };
}
impl Image {
    appearance!();
    pub fn source(mut self, value: impl Into<Vary<String>>) -> Self {
        self.source = value.into();
        self
    }
    pub fn alt(mut self, value: impl Into<Vary<String>>) -> Self {
        self.alternative = Some(value.into());
        self.decorative = false;
        self
    }
    pub fn decorative(mut self) -> Self {
        self.alternative = None;
        self.decorative = true;
        self
    }
    /// One fallback resource attempt for each accepted primary source selection.
    pub fn fallback_src(mut self, value: impl Into<String>) -> Self {
        self.fallback = Some(value.into());
        self
    }
    pub fn fit(mut self, value: ObjectFit) -> Self {
        self.fit = Some(value);
        self
    }
    pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn focus(mut self, value: FocusMode) -> Self {
        self.common.focus = value;
        self
    }
    pub fn on_load(mut self, value: impl FnMut(u64) + 'static) -> Self {
        self.loaded = Some(Box::new(value));
        self
    }
    pub fn on_error(mut self, value: impl FnMut(u64) + 'static) -> Self {
        self.failed = Some(Box::new(value));
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
impl Widget for Image {
    fn build(&mut self, scope: &Scope) -> Result {
        let primary = self
            .source
            .try_with(|source| validate_source(source).map(|()| source.clone()))??;
        if let Some(fallback) = &self.fallback {
            validate_source(fallback)?;
        }
        if !self.decorative && self.alternative.is_none() {
            return Err(glory::render::Error::InvalidProps(
                "image requires alt or explicit decorative()",
            )
            .into());
        }
        if self.decorative && (self.common.name.is_some() || self.common.focus != FocusMode::Auto) {
            return Err(glory::render::Error::InvalidProps(
                "decorative image cannot have a name or focus",
            )
            .into());
        }
        let initial = Selection {
            current: primary.clone(),
            primary,
            fallback: false,
        };
        let selection = scope.cage(initial.clone());
        let node = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        self.common.node_ref = Some(node);
        let source = self.source.clone();
        if source.is_reactive() {
            let weak = scope.downgrade();
            scope.effect(move || {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    let next = source
                        .try_with(|source| validate_source(source).map(|()| source.clone()))??;
                    if selection.try_with_untracked(|old| old.primary != next)? {
                        selection.try_revise(|value| {
                            *value = Selection {
                                primary: next.clone(),
                                current: next,
                                fallback: false,
                            }
                        })?;
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                }
            });
        }
        let weak = scope.downgrade();
        let mut previous = initial.clone();
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let next = selection.try_get()?;
                let Some(node) = node.try_get()? else {
                    return Ok(());
                };
                if next != previous {
                    scope.set_image_source(node, next.current.clone())?;
                    previous = next;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        let base = Style {
            display: Some(Display::Block),
            max_width: Some(Limit::Percent(100.0)),
            background: Some(self.common.theme.surface),
            color: Some(self.common.theme.foreground),
            ..Style::new()
        };
        // Explicit sizing/fit survive unstyled; custom declarations remain last.
        let structural = Style {
            width: self.width,
            height: self.height,
            object_fit: self.fit,
            ..Style::new()
        };
        let custom = self.common.style.clone();
        let merged = move || crate::appearance::merge(structural.clone(), custom.get());
        self.common.style = if self.common.style.is_reactive() {
            scope.bond(merged).into()
        } else {
            merged().into()
        };
        let mut inner = configure!(self.common, glory::ui::image(), scope, "sapid-image", base)
            .source(initial.current);
        inner = if self.decorative {
            inner.decorative()
        } else {
            inner.alt(self.alternative.take().unwrap())
        };
        let fallback = self.fallback.take();
        let mut loaded = self.loaded.take();
        let mut failed = self.failed.take();
        let mut observers = self
            .common
            .handlers
            .remove(&EventKind::ImageStatus)
            .unwrap_or_default();
        let owner = scope.id();
        let weak = scope.downgrade();
        inner = inner.on(EventKind::ImageStatus, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            if let Event::ImageStatus { revision, status } = event {
                if status == ImageStatus::Failed
                    && let Some(fallback) = &fallback
                {
                    let result = (|| -> Result {
                        if selection.try_with_untracked(|value| !value.fallback)? {
                            selection.try_revise(|value| {
                                value.current = fallback.clone();
                                value.fallback = true;
                            })?;
                        }
                        Ok(())
                    })();
                    if let Err(error) = result {
                        scope.report_error(error);
                        return;
                    }
                }
                let callback = if status == ImageStatus::Loaded {
                    &mut loaded
                } else {
                    &mut failed
                };
                if let Some(callback) = callback {
                    callback(revision);
                }
            }
            observe(&mut observers, owner, event);
        });
        for (kind, mut observers) in std::mem::take(&mut self.common.handlers) {
            inner = inner.on(kind, move |event| observe(&mut observers, owner, event));
        }
        inner.show_in(scope)
    }
}

pub struct BackgroundImage {
    common: Common,
    image: Option<Image>,
    width: Option<Length>,
    height: Option<Length>,
    children: glory::ui::Children,
}
pub fn background_image(source: impl Into<Vary<String>>) -> BackgroundImage {
    BackgroundImage {
        common: Common::default(),
        image: Some(image(source).decorative().unstyled().fit(ObjectFit::Cover)),
        width: None,
        height: None,
        children: ().into_children(),
    }
}
impl BackgroundImage {
    appearance!();
    pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn fallback_src(mut self, value: impl Into<String>) -> Self {
        self.image = self.image.take().map(|image| image.fallback_src(value));
        self
    }
    pub fn fit(mut self, value: ObjectFit) -> Self {
        self.image = self.image.take().map(|image| image.fit(value));
        self
    }
    pub fn on_load(mut self, value: impl FnMut(u64) + 'static) -> Self {
        self.image = self.image.take().map(|image| image.on_load(value));
        self
    }
    pub fn on_error(mut self, value: impl FnMut(u64) + 'static) -> Self {
        self.image = self.image.take().map(|image| image.on_error(value));
        self
    }
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.children = children.into_children();
        self
    }
}
impl Widget for BackgroundImage {
    fn build(&mut self, scope: &Scope) -> Result {
        let image = self.image.take().ok_or(glory::ui::Error::Gone)?;
        image.source.try_with(|source| validate_source(source))??;
        if let Some(fallback) = &image.fallback {
            validate_source(fallback)?;
        }
        let base = Style {
            background: Some(self.common.theme.surface),
            color: Some(self.common.theme.foreground),
            ..Style::new()
        };
        let structural = Style {
            position: Some(Position::Relative),
            overflow_x: Some(Overflow::Hidden),
            overflow_y: Some(Overflow::Hidden),
            width: self.width,
            height: self.height,
            ..Style::new()
        };
        let custom = self.common.style.clone();
        let merged = move || crate::appearance::merge(structural.clone(), custom.get());
        self.common.style = if self.common.style.is_reactive() {
            scope.bond(merged).into()
        } else {
            merged().into()
        };
        configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-background-image",
            base
        )
        .fill((
            image.class("sapid-background-image-media").style(Style {
                position: Some(Position::Absolute),
                top: Some(Length::Px(0.0)),
                left: Some(Length::Px(0.0)),
                width: Some(Length::Percent(100.0)),
                height: Some(Length::Percent(100.0)),
                ..Style::new()
            }),
            glory::ui::view()
                .class("sapid-background-image-content")
                .style(Style::new().position(Position::Relative))
                .fill(std::mem::replace(&mut self.children, ().into_children())),
        ))
        .show_in(scope)
    }
}
