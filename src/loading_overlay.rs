//! Local decorative shading and an explicit busy, inert loading region.
use crate::appearance::{Common, configure};
use glory::render::{NodeId, Semantics};
use glory::style::{Align, Color, Display, Justify, Length, PointerEvents, Position, Style};
use glory::ui::{Children, IntoChildren, Result, Widget};
use glory::{Cage, Scope, Vary};

struct Shade {
    common: Common,
    visible: Vary<bool>,
    loading: bool,
    opacity: f64,
    color: Option<Color>,
    label: Vary<String>,
    children: Children,
    mask_ref: Option<Cage<Option<NodeId>>>,
    content_ref: Option<Cage<Option<NodeId>>>,
}
impl Shade {
    fn new(loading: bool, visible: Vary<bool>) -> Self {
        Self {
            common: Common::default(),
            visible,
            loading,
            opacity: 0.65,
            color: None,
            label: "Loading".into(),
            children: ().into_children(),
            mask_ref: None,
            content_ref: None,
        }
    }
}
/// A retained content wrapper with a purely decorative, pointer-transparent shade.
pub struct Overlay(Shade);
/// A retained content region whose loading source controls busy and inert together.
pub struct LoadingOverlay(Shade);
pub fn overlay() -> Overlay {
    Overlay(Shade::new(false, true.into()))
}
pub fn loading_overlay(loading: impl Into<Vary<bool>>) -> LoadingOverlay {
    LoadingOverlay(Shade::new(true, loading.into()))
}
macro_rules! common {
    ($ty:ty) => {
        impl $ty {
            pub fn theme(mut self, theme: crate::Theme) -> Self {
                self.0.common.theme = theme;
                self
            }
            pub fn unstyled(mut self) -> Self {
                self.0.common.unstyled = true;
                self
            }
            pub fn class(mut self, class: impl Into<Vary<String>>) -> Self {
                self.0.common.class = class.into();
                self
            }
            pub fn style(mut self, style: impl Into<Vary<Style>>) -> Self {
                self.0.common.style = style.into();
                self
            }
            /// Optional name for the containing content region, not for the mask.
            pub fn name(mut self, name: impl Into<Vary<String>>) -> Self {
                self.0.common.name = Some(name.into());
                self
            }
            pub fn color(mut self, color: Color) -> Self {
                self.0.color = Some(color);
                self
            }
            /// Opacity of the shade alone; the content and spinner keep their opacity.
            pub fn opacity(mut self, opacity: f64) -> Self {
                self.0.opacity = opacity;
                self
            }
            pub fn node_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
                self.0.common.node_ref = Some(reference);
                self
            }
            pub fn content_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
                self.0.content_ref = Some(reference);
                self
            }
            pub fn mask_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
                self.0.mask_ref = Some(reference);
                self
            }
            pub fn fill(mut self, children: impl IntoChildren) -> Self {
                self.0.children = children.into_children();
                self
            }
        }
        impl Widget for $ty {
            fn build(&mut self, scope: &Scope) -> Result {
                self.0.build(scope)
            }
        }
    };
}
common!(Overlay);
common!(LoadingOverlay);
impl Overlay {
    pub fn visible(mut self, visible: impl Into<Vary<bool>>) -> Self {
        self.0.visible = visible.into();
        self
    }
}
impl LoadingOverlay {
    pub fn loading(mut self, loading: impl Into<Vary<bool>>) -> Self {
        self.0.visible = loading.into();
        self
    }
    /// Localized status text announced by the independent retained spinner.
    pub fn label(mut self, label: impl Into<Vary<String>>) -> Self {
        self.0.label = label.into();
        self
    }
}
fn name(value: &str) -> Result {
    if value.trim().is_empty() || value.len() > 256 {
        return Err(glory::render::Error::InvalidProps(
            "overlay names must be nonempty and at most 256 UTF-8 bytes",
        )
        .into());
    }
    Ok(())
}
fn positioned(mut style: Style) -> Result<Style> {
    if style.position == Some(Position::Static) {
        return Err(glory::render::Error::InvalidProps(
            "overlay wrapper requires a positioned containing block",
        )
        .into());
    }
    if style.position.is_none() {
        style.position = Some(Position::Relative);
    }
    Ok(style)
}
impl Widget for Shade {
    fn build(&mut self, scope: &Scope) -> Result {
        if !self.opacity.is_finite() || !(0.0..=1.0).contains(&self.opacity) {
            return Err(
                glory::render::Error::InvalidProps("overlay opacity must be in 0..=1").into(),
            );
        }
        if self.loading {
            self.label.try_with(|value| name(value))??;
        }
        if let Some(value) = &self.common.name {
            value.try_with(|value| name(value))??;
        }
        positioned(self.common.style.try_get()?)?;
        if let Some(value) = self.common.name.take() {
            self.common.name =
                Some(crate::composite::validated(scope, &value, |value| name(value))?.into());
        }
        let custom = self.common.style.clone();
        self.common.style = crate::composite::validated(scope, &custom, |style| {
            positioned(style.clone()).map(|_| ())
        })?
        .map(|style| positioned(style.clone()).expect("validated positioned style"))
        .into();
        let visible = self.visible.clone();
        let loading = self.loading;
        let busy = scope.bond(move || {
            Some(Box::new(Semantics {
                busy: loading.then_some(visible.get()),
                ..Default::default()
            }))
        });
        let visible = self.visible.clone();
        let inert = scope.bond(move || {
            Some(Box::new(Semantics {
                inert: loading && visible.get(),
                ..Default::default()
            }))
        });
        let visible = self.visible.clone();
        let hidden = scope.bond(move || {
            Some(Box::new(Semantics {
                hidden: !visible.get(),
                ..Default::default()
            }))
        });
        let background = self
            .color
            .or_else(|| (!self.common.unstyled).then_some(self.common.theme.surface))
            .map(|color| {
                Color(
                    color.0,
                    color.1,
                    color.2,
                    (f64::from(color.3) * self.opacity).round() as u8,
                )
            });
        let mut mask = glory::ui::view()
            .class("sapid-overlay-mask")
            .semantics(hidden)
            .style(
                Style {
                    background,
                    align: Some(Align::Center),
                    justify: Some(Justify::Center),
                    ..Style::new()
                }
                .position(Position::Absolute)
                .top(Length::Px(0.))
                .right(Length::Px(0.))
                .bottom(Length::Px(0.))
                .left(Length::Px(0.))
                .display(Display::Flex)
                .pointer_events(if loading {
                    PointerEvents::Auto
                } else {
                    PointerEvents::None
                }),
            );
        if let Some(reference) = self.mask_ref {
            mask = mask.node_ref(reference);
        }
        if loading {
            let mut spinner = crate::spinner()
                .theme(self.common.theme.clone())
                .name(self.label.clone())
                .class("sapid-loading-overlay-spinner");
            if self.common.unstyled {
                spinner = spinner.unstyled();
            }
            mask = mask.fill(spinner);
        }
        let mut content = glory::ui::view()
            .class("sapid-overlay-content")
            .semantics(inert)
            .fill(std::mem::replace(&mut self.children, ().into_children()));
        if let Some(reference) = self.content_ref {
            content = content.node_ref(reference);
        }
        configure!(
            self.common,
            glory::ui::view(),
            scope,
            if loading {
                "sapid-loading-overlay"
            } else {
                "sapid-overlay"
            },
            Style::new()
        )
        .semantics(busy)
        .fill((content, mask))
        .show_in(scope)
    }
}
