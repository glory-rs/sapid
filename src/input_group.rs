use glory::style::{Align, Display, FlexDirection, Length, Style};
use glory::ui::{Children, IntoChildren, Result, Widget};
use glory::{Scope, Vary};

use crate::appearance::{Common, configure};

macro_rules! section_methods {
    () => {
        pub fn theme(mut self, value: crate::Theme) -> Self {
            self.common.theme = value;
            self
        }
        pub fn size(mut self, value: crate::Size) -> Self {
            self.common.size = value;
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
    };
}

/// Ordered, noninteractive layout around exactly one input widget. FieldControl
/// belongs to the input, so decorations never become its accessible name.
pub struct InputGroup {
    common: Common,
    input: Option<Box<dyn Widget>>,
    start: Children,
    end: Children,
}
pub fn input_group() -> InputGroup {
    InputGroup {
        common: Common::default(),
        input: None,
        start: ().into_children(),
        end: ().into_children(),
    }
}
impl InputGroup {
    section_methods!();
    pub fn input(mut self, input: impl Widget) -> Self {
        self.input = Some(Box::new(input));
        self
    }
    pub fn start(mut self, children: impl IntoChildren) -> Self {
        self.start = children.into_children();
        self
    }
    pub fn end(mut self, children: impl IntoChildren) -> Self {
        self.end = children.into_children();
        self
    }
}
fn row() -> Style {
    Style {
        align: Some(Align::Center),
        ..Style::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .gap(8.0)
            .min_width(Length::Px(0.0))
    }
}
impl Widget for InputGroup {
    fn build(&mut self, scope: &Scope) -> Result {
        let input = self.input.take().ok_or(glory::render::Error::InvalidProps(
            "input group requires one input widget",
        ))?;
        let start = std::mem::replace(&mut self.start, ().into_children());
        let end = std::mem::replace(&mut self.end, ().into_children());
        configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-input-group",
            row()
        )
        .fill(start)
        .fill(
            glory::ui::view()
                .class("sapid-input-group-control")
                .style(Style {
                    grow: Some(1.0),
                    shrink: Some(1.0),
                    ..Style::new().min_width(Length::Px(0.0))
                })
                .fill(input),
        )
        .fill(end)
        .show_in(scope)
    }
}

/// Text before/after an input. It is content, never an implicit label or tab stop.
pub struct InputAddon {
    common: Common,
    content: Vary<String>,
}
pub fn input_addon() -> InputAddon {
    InputAddon {
        common: Common::default(),
        content: "".into(),
    }
}
impl InputAddon {
    section_methods!();
    pub fn content(mut self, value: impl Into<Vary<String>>) -> Self {
        self.content = value.into();
        self
    }
}
impl Widget for InputAddon {
    fn build(&mut self, scope: &Scope) -> Result {
        configure!(
            self.common,
            glory::ui::text(),
            scope,
            "sapid-input-addon",
            self.common.theme.description(self.common.size)
        )
        .content(self.content.clone())
        .show_in(scope)
    }
}

/// A decoration or action slot. Interactive children retain their intrinsic
/// controls and natural keyboard order; the section adds no focus target.
pub struct InputSection {
    common: Common,
    children: Children,
}
pub fn input_section() -> InputSection {
    InputSection {
        common: Common::default(),
        children: ().into_children(),
    }
}
impl InputSection {
    section_methods!();
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.children = children.into_children();
        self
    }
}
impl Widget for InputSection {
    fn build(&mut self, scope: &Scope) -> Result {
        let children = std::mem::replace(&mut self.children, ().into_children());
        let root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-input-section",
            row()
        );
        root.fill(children).show_in(scope)
    }
}
