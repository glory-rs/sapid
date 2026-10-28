//! A finite placeholder. Applications own replacement content and loading state.
use crate::appearance::Common;
use crate::composite;
use glory::render::{GraphicProps, NodeId, Semantics};
use glory::style::{Display, Length, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};
pub struct Skeleton {
    common: Common,
    width: Vary<Length>,
    height: Vary<f64>,
    loading: Vary<bool>,
    radius: f64,
}
pub fn skeleton() -> Skeleton {
    Skeleton {
        common: Common::default(),
        width: Length::Percent(100.0).into(),
        height: 16.0.into(),
        loading: true.into(),
        radius: 4.0,
    }
}
impl Skeleton {
    pub fn width(mut self, value: impl Into<Vary<Length>>) -> Self {
        self.width = value.into();
        self
    }
    pub fn height(mut self, value: impl Into<Vary<f64>>) -> Self {
        self.height = value.into();
        self
    }
    pub fn loading(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.loading = value.into();
        self
    }
    pub fn radius(mut self, value: f64) -> Self {
        self.radius = value;
        self
    }
    pub fn circle(mut self, diameter: f64) -> Self {
        self.width = Length::Px(diameter).into();
        self.height = diameter.into();
        self.radius = diameter / 2.0;
        self
    }
    /// Names the busy region. Its graphic remains explicitly decorative.
    pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
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
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
}
fn dimension(value: f64) -> Result {
    if !value.is_finite() || !(0.000001..=1_000_000.0).contains(&value) {
        return Err(glory::render::Error::InvalidProps(
            "skeleton dimensions must be finite and in 0.000001..=1000000",
        )
        .into());
    }
    Ok(())
}
impl Widget for Skeleton {
    fn build(&mut self, scope: &Scope) -> Result {
        if !self.radius.is_finite() || !(0.0..=1_000_000.0).contains(&self.radius) {
            return Err(glory::render::Error::InvalidProps(
                "skeleton radius must be finite and in 0..=1000000",
            )
            .into());
        }
        let width = composite::validated(scope, &self.width, |value| match value {
            Length::Px(value) | Length::Percent(value) => dimension(*value),
            Length::Auto | Length::Ch(_) => Err(glory::render::Error::InvalidProps(
                "skeleton width must be explicit pixels or percent",
            )
            .into()),
        })?;
        let height = composite::validated(scope, &self.height, |value| dimension(*value))?;
        let own = self.common.style.clone();
        let unstyled = self.common.unstyled;
        let radius = self.radius;
        self.common.style = scope
            .bond(move || {
                crate::appearance::merge(
                    Style::new()
                        .display(Display::Block)
                        .width(width.get())
                        .height(Length::Px(height.get()))
                        .radius(radius),
                    own.get(),
                )
            })
            .into();
        let (classes, styles) = self
            .common
            .appearance(scope, "sapid-skeleton", Style::new());
        let loading = self.loading.clone();
        let mut root = glory::ui::view()
            .class(classes)
            .style(styles)
            .semantics(scope.bond(move || {
                let loading = loading.get();
                Some(Box::new(Semantics {
                    busy: Some(loading),
                    hidden: !loading,
                    ..Default::default()
                }))
            }));
        if let Some(name) = self.common.name.take() {
            root = root.name(name)
        }
        if let Some(reference) = self.common.node_ref {
            root = root.node_ref(reference)
        }
        let mut graphic_style = Style::new()
            .display(Display::Block)
            .width(Length::Percent(100.0))
            .height(Length::Percent(100.0))
            .radius(radius);
        if !unstyled {
            graphic_style.background = Some(self.common.theme.border);
        }
        root.fill(
            glory::ui::graphic()
                .class("sapid-skeleton-placeholder")
                .drawing(GraphicProps::default())
                .style(graphic_style),
        )
        .show_in(scope)
    }
}
