//! Passive, anchored status markers around an independent content slot.
use crate::appearance::{Common, configure};
use glory::render::{NodeId, Semantics};
use glory::style::{Align, Display, Edges, Justify, Length, Position, Style, WhiteSpace};
use glory::ui::{Children, IntoChildren, Result, Widget};
use glory::{Cage, Scope, Vary};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IndicatorPosition {
    TopLeft,
    TopCenter,
    #[default]
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}
impl IndicatorPosition {
    fn coordinates(self) -> (f64, f64) {
        match self {
            Self::TopLeft => (0.0, 0.0),
            Self::TopCenter => (50.0, 0.0),
            Self::TopRight => (100.0, 0.0),
            Self::MiddleLeft => (0.0, 50.0),
            Self::MiddleCenter => (50.0, 50.0),
            Self::MiddleRight => (100.0, 50.0),
            Self::BottomLeft => (0.0, 100.0),
            Self::BottomCenter => (50.0, 100.0),
            Self::BottomRight => (100.0, 100.0),
        }
    }
}

pub struct Indicator {
    common: Common,
    position: IndicatorPosition,
    diameter: f64,
    visible: Vary<bool>,
    label: Option<Vary<String>>,
    children: Children,
    marker_ref: Option<Cage<Option<NodeId>>>,
}
pub fn indicator() -> Indicator {
    Indicator {
        common: Common::default(),
        position: IndicatorPosition::default(),
        diameter: 12.0,
        visible: true.into(),
        label: None,
        children: ().into_children(),
        marker_ref: None,
    }
}
impl Indicator {
    pub fn theme(mut self, theme: crate::Theme) -> Self {
        self.common.theme = theme;
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
    /// Styles the content wrapper. The anchored marker keeps its own geometry.
    pub fn style(mut self, style: impl Into<Vary<Style>>) -> Self {
        self.common.style = style.into();
        self
    }
    pub fn node_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(reference);
        self
    }
    pub fn marker_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
        self.marker_ref = Some(reference);
        self
    }
    /// An optional passive marker name; this does not make a live region.
    pub fn name(mut self, name: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(name.into());
        self
    }
    pub fn position(mut self, position: IndicatorPosition) -> Self {
        self.position = position;
        self
    }
    /// Fixed square marker box in pixels, in 4..=128. Labels do not resize it.
    pub fn diameter(mut self, diameter: f64) -> Self {
        self.diameter = diameter;
        self
    }
    pub fn visible(mut self, visible: impl Into<Vary<bool>>) -> Self {
        self.visible = visible.into();
        self
    }
    /// Optional nonempty label, at most 32 UTF-8 bytes. A dot has no text.
    pub fn label(mut self, label: impl Into<Vary<String>>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.children = children.into_children();
        self
    }
}
fn valid_text(value: &str, max: usize) -> Result {
    if value.trim().is_empty() || value.len() > max {
        return Err(glory::render::Error::InvalidProps(
            "indicator text must be nonempty and within its byte budget",
        )
        .into());
    }
    Ok(())
}
fn accepted_text(scope: &Scope, source: Vary<String>, max: usize) -> Result<Vary<String>> {
    let initial = source.try_with(|value| valid_text(value, max).map(|()| value.clone()))??;
    if !source.is_reactive() {
        return Ok(initial.into());
    }
    let accepted = scope.cage(initial);
    let weak = scope.downgrade();
    scope.effect(move || {
        let Some(scope) = weak.upgrade() else { return };
        let result = (|| -> Result {
            let next =
                source.try_with(|value| valid_text(value, max).map(|()| value.clone()))??;
            if accepted.try_with_untracked(|old| *old != next)? {
                accepted.try_revise(|old| *old = next)?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            scope.report_error(error);
        }
    });
    Ok(accepted.into())
}
impl Widget for Indicator {
    fn build(&mut self, scope: &Scope) -> Result {
        if !self.diameter.is_finite() || !(4.0..=128.0).contains(&self.diameter) {
            return Err(glory::render::Error::InvalidProps(
                "indicator diameter must be finite and in 4..=128 pixels",
            )
            .into());
        }
        // Validate all sources before any visible nodes are created.
        if let Some(label) = &self.label {
            label.try_with(|value| valid_text(value, 32))??;
        }
        if let Some(name) = &self.common.name {
            name.try_with(|value| valid_text(value, 256))??;
        }
        let label = self
            .label
            .take()
            .map(|source| accepted_text(scope, source, 32))
            .transpose()?;
        let name = self
            .common
            .name
            .take()
            .map(|source| accepted_text(scope, source, 256))
            .transpose()?;
        let (left, top) = self.position.coordinates();
        let mut marker_style = Style {
            position: Some(Position::Absolute),
            left: Some(Length::Percent(left)),
            top: Some(Length::Percent(top)),
            width: Some(Length::Px(self.diameter)),
            height: Some(Length::Px(self.diameter)),
            display: Some(Display::Flex),
            align: Some(Align::Center),
            justify: Some(Justify::Center),
            margin: Some(Edges {
                top: -self.diameter / 2.0,
                left: -self.diameter / 2.0,
                right: 0.0,
                bottom: 0.0,
            }),
            ..Style::new()
        };
        if !self.common.unstyled {
            marker_style.radius = Some(self.diameter / 2.0);
            marker_style.background = Some(self.common.theme.accent);
            marker_style.color = Some(self.common.theme.on_accent);
            marker_style.font_size = Some((self.diameter * 0.65).min(18.0));
            marker_style.font_weight = Some(600.0);
            marker_style.white_space = Some(WhiteSpace::NoWrap);
        }
        let visible = self.visible.clone();
        let semantics = move || {
            Some(Box::new(Semantics {
                hidden: !visible.get(),
                ..Default::default()
            }))
        };
        let mut marker = glory::ui::view()
            .class("sapid-indicator-marker")
            .style(marker_style);
        marker = if self.visible.is_reactive() {
            marker.semantics(scope.bond(semantics))
        } else {
            marker.semantics(semantics())
        };
        if let Some(name) = name {
            marker = marker.name(name);
        }
        if let Some(reference) = self.marker_ref {
            marker = marker.node_ref(reference);
        }
        if let Some(label) = label {
            marker = marker.fill(
                glory::ui::text()
                    .class("sapid-indicator-label")
                    .content(label),
            );
        }
        // Positioning is structural and stays meaningful in unstyled mode.
        let custom = self.common.style.clone();
        self.common.style = if custom.is_reactive() {
            scope
                .bond(move || {
                    crate::appearance::merge(
                        Style::new()
                            .position(Position::Relative)
                            .display(Display::InlineFlex),
                        custom.get(),
                    )
                })
                .into()
        } else {
            crate::appearance::merge(
                Style::new()
                    .position(Position::Relative)
                    .display(Display::InlineFlex),
                custom.get(),
            )
            .into()
        };
        configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-indicator",
            Style::new()
        )
        .fill(std::mem::replace(&mut self.children, ().into_children()))
        .fill(marker)
        .show_in(scope)
    }
}

impl Default for Indicator {
    fn default() -> Self {
        indicator()
    }
}
