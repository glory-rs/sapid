use crate::appearance::{Common, configure};
use glory::render::{Orientation, Role, Semantics};
use glory::style::{Length, Style};
use glory::ui::{Result, Widget};
use glory::{Scope, Vary};

/// A passive visual boundary; adjustable splitters are a separate interaction.
pub struct Separator {
    common: Common,
    axis: Orientation,
    decorative: bool,
    length: Option<Length>,
    thickness: f64,
}
pub fn separator() -> Separator {
    Separator {
        common: Common::default(),
        axis: Orientation::Horizontal,
        decorative: false,
        length: None,
        thickness: 1.0,
    }
}
impl Separator {
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
    pub fn style(mut self, style: impl Into<Vary<Style>>) -> Self {
        self.common.style = style.into();
        self
    }
    pub fn name(mut self, name: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(name.into());
        self
    }
    pub fn node_ref(mut self, reference: glory::Cage<Option<glory::render::NodeId>>) -> Self {
        self.common.node_ref = Some(reference);
        self
    }
    pub fn orientation(mut self, axis: Orientation) -> Self {
        self.axis = axis;
        self
    }
    pub fn decorative(mut self, decorative: bool) -> Self {
        self.decorative = decorative;
        self
    }
    pub fn length(mut self, length: Length) -> Self {
        self.length = Some(length);
        self
    }
    pub fn thickness(mut self, thickness: f64) -> Self {
        self.thickness = thickness;
        self
    }
}
impl Widget for Separator {
    fn build(&mut self, scope: &Scope) -> Result {
        if !self.thickness.is_finite() || self.thickness <= 0.0 {
            return Err(glory::render::Error::InvalidProps(
                "separator thickness must be finite and positive",
            )
            .into());
        }
        if self.decorative && self.common.name.is_some() {
            return Err(glory::render::Error::InvalidProps(
                "decorative separators cannot have names",
            )
            .into());
        }
        let length = self.length.unwrap_or(match self.axis {
            Orientation::Horizontal => Length::Percent(100.0),
            Orientation::Vertical => Length::Px(24.0),
        });
        let (width, height) = match self.axis {
            Orientation::Horizontal => (length, Length::Px(self.thickness)),
            Orientation::Vertical => (Length::Px(self.thickness), length),
        };
        let base = Style {
            background: Some(self.common.theme.border),
            shrink: Some(0.0),
            ..Style::new().width(width).height(height)
        };
        base.validate()
            .map_err(glory::render::Error::InvalidProps)?;
        let class = match self.axis {
            Orientation::Horizontal => "sapid-separator sapid-separator-horizontal",
            Orientation::Vertical => "sapid-separator sapid-separator-vertical",
        };
        let mut root = configure!(self.common, glory::ui::view(), scope, class, base);
        if !self.decorative {
            root = root
                .role(Role::Separator)
                .semantics(Some(Box::new(Semantics {
                    separator_axis: Some(self.axis),
                    ..Default::default()
                })));
        }
        root.show_in(scope)
    }
}

impl Default for Separator {
    fn default() -> Self {
        separator()
    }
}
