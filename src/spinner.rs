//! A finite loading indicator; animation is owned by the rendering host.
use crate::appearance::{Common, configure, merge};
use glory::render::{GraphicAnimation, NodeId, Role, Semantics};
use glory::style::{Display, Length, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};

pub struct Spinner {
    common: Common,
    diameter: Option<f64>,
    thickness: f64,
    period: u32,
    visible: Vary<bool>,
    animated: Vary<bool>,
}
pub fn spinner() -> Spinner {
    Spinner {
        common: Common {
            name: Some("Loading".into()),
            ..Common::default()
        },
        diameter: None,
        thickness: 3.,
        period: 1000,
        visible: true.into(),
        animated: true.into(),
    }
}
/// The same circular indicator under the Loader name.
pub fn loader() -> Spinner {
    spinner()
}
impl Spinner {
    pub fn theme(mut self, theme: crate::Theme) -> Self {
        self.common.theme = theme;
        self
    }
    pub fn size(mut self, size: crate::Size) -> Self {
        self.common.size = size;
        self
    }
    pub fn diameter(mut self, diameter: f64) -> Self {
        self.diameter = Some(diameter);
        self
    }
    pub fn thickness(mut self, thickness: f64) -> Self {
        self.thickness = thickness;
        self
    }
    pub fn period_millis(mut self, period: u32) -> Self {
        self.period = period;
        self
    }
    pub fn name(mut self, name: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(name.into());
        self
    }
    /// Remove the status/name/busy announcement; the visible shape remains.
    pub fn decorative(mut self) -> Self {
        self.common.name = None;
        self
    }
    pub fn visible(mut self, visible: impl Into<Vary<bool>>) -> Self {
        self.visible = visible.into();
        self
    }
    pub fn animated(mut self, animated: impl Into<Vary<bool>>) -> Self {
        self.animated = animated.into();
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn class(mut self, class: impl Into<Vary<String>>) -> Self {
        self.common.class = class.into();
        self
    }
    pub fn style(mut self, style: impl Into<Vary<Style>>) -> Self {
        self.common.style = style.into();
        self
    }
    pub fn node_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(reference);
        self
    }
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn valid_name(value: &str) -> Result {
    if value.trim().is_empty() || value.len() > 256 {
        return Err(invalid(
            "spinner name must be nonempty and at most 256 UTF-8 bytes",
        ));
    }
    Ok(())
}
impl Widget for Spinner {
    fn build(&mut self, scope: &Scope) -> Result {
        let diameter = self.diameter.unwrap_or(match self.common.size {
            crate::Size::ExtraSmall => 12.,
            crate::Size::Small => 16.,
            crate::Size::Medium => 24.,
            crate::Size::Large => 32.,
            crate::Size::ExtraLarge => 40.,
        });
        if !diameter.is_finite()
            || !(8.0..=128.0).contains(&diameter)
            || !self.thickness.is_finite()
            || !(1.0..=64.0).contains(&self.thickness)
            || self.thickness > diameter / 2.
        {
            return Err(invalid(
                "spinner diameter must be 8..=128 and thickness 1..=64, at most half its diameter",
            ));
        }
        let animation = GraphicAnimation::Rotate {
            period_millis: self.period,
        };
        animation.validate()?;
        let named = self.common.name.is_some();
        if let Some(name) = &self.common.name {
            name.try_with(|name| valid_name(name))??;
        }
        if let Some(name) = self.common.name.take() {
            self.common.name = Some(
                crate::composite::validated(scope, &name, |name| valid_name(name))?.into(),
            );
        }
        let color = if self.common.unstyled {
            self.common.theme.foreground
        } else {
            self.common.theme.accent
        };
        let thickness = self.thickness;
        let animated = self.animated.clone();
        let drawing = move || {
            let mut drawing = crate::progress::arc(diameter, thickness, false, 0.75, color);
            drawing.animation = animated.get().then_some(animation);
            drawing
        };
        let drawing: Vary<_> = if self.animated.is_reactive() {
            scope.bond(drawing).into()
        } else {
            drawing().into()
        };
        let required = Style::new()
            .display(Display::InlineFlex)
            .width(Length::Px(diameter))
            .height(Length::Px(diameter));
        let custom = self.common.style.clone();
        self.common.style = if custom.is_reactive() {
            scope
                .bond(move || merge(required.clone(), custom.get()))
                .into()
        } else {
            merge(required, custom.get()).into()
        };
        let visible = self.visible.clone();
        let metadata = move || {
            Some(Box::new(Semantics {
                hidden: !visible.get(),
                busy: named.then_some(true),
                ..Default::default()
            }))
        };
        let metadata: Vary<_> = if self.visible.is_reactive() {
            scope.bond(metadata).into()
        } else {
            metadata().into()
        };
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-spinner",
            Style::new()
        )
        .semantics(metadata);
        if named {
            root = root.role(Role::Status);
        }
        root.fill(
            glory::ui::graphic()
                .class("sapid-spinner-graphic")
                .drawing(drawing)
                .style(
                    Style::new()
                        .display(Display::Block)
                        .width(Length::Percent(100.))
                        .height(Length::Percent(100.)),
                ),
        )
        .show_in(scope)
    }
}
