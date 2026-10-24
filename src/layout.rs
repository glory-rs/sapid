//! Portable layout boxes. Sapid Group owns a box; Glory Group stays boxless.
use crate::appearance::{Common, merge};
use glory::render::NodeId;
use glory::style::{
    Align, Display, Edges, FlexDirection, FlexWrap, Justify, Length, Limit, Style,
};
use glory::ui::{Children, IntoChildren, Result, Widget};
use glory::{Cage, Lotus, Scope};

macro_rules! methods {
    () => {
        pub fn theme(mut self, theme: crate::Theme) -> Self {
            self.layout.common.theme = theme;
            self
        }
        pub fn size(mut self, size: crate::Size) -> Self {
            self.layout.common.size = size;
            self
        }
        /// Remove cosmetic padding while preserving the layout contract.
        pub fn unstyled(mut self) -> Self {
            self.layout.common.unstyled = true;
            self
        }
        pub fn class(mut self, class: impl Into<Lotus<String>>) -> Self {
            self.layout.common.class = class.into();
            self
        }
        pub fn style(mut self, style: impl Into<Lotus<Style>>) -> Self {
            self.layout.common.style = style.into();
            self
        }
        pub fn name(mut self, name: impl Into<Lotus<String>>) -> Self {
            self.layout.common.name = Some(name.into());
            self
        }
        pub fn node_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
            self.layout.common.node_ref = Some(reference);
            self
        }
        pub fn fill(mut self, children: impl IntoChildren) -> Self {
            self.layout.children = children.into_children();
            self
        }
        pub fn width(mut self, width: Length) -> Self {
            self.layout.base.width = Some(width);
            self
        }
        pub fn height(mut self, height: Length) -> Self {
            self.layout.base.height = Some(height);
            self
        }
        pub fn padding(mut self, padding: Edges) -> Self {
            self.layout.base.padding = Some(padding);
            self
        }
    };
}
struct Layout {
    common: Common,
    base: Style,
    children: Children,
    class: &'static str,
    cap: Option<Limit>,
    default_padding: bool,
}
impl Layout {
    fn new(class: &'static str, base: Style) -> Self {
        Self {
            common: Common::default(),
            base,
            children: ().into_children(),
            class,
            cap: None,
            default_padding: false,
        }
    }
    fn build(&mut self, scope: &Scope) -> Result {
        let mut base = self.base.clone();
        if self.default_padding && !self.common.unstyled && base.padding.is_none() {
            base.padding = Some(Edges {
                top: 0.0,
                bottom: 0.0,
                left: 16.0,
                right: 16.0,
            });
        }
        base.validate()
            .map_err(glory::render::Error::InvalidProps)?;
        if let Some(cap) = self.cap {
            Style::new()
                .max_width(cap)
                .validate()
                .map_err(glory::render::Error::InvalidProps)?;
        }
        // Keep structure even in unstyled mode. Explicit Style remains reactive.
        let (class, custom) = self.common.appearance(scope, self.class, Style::new());
        let style: Lotus<Style> = if custom.is_reactive() {
            scope.bond(move || merge(base.clone(), custom.get())).into()
        } else {
            merge(base, custom.get()).into()
        };
        let mut root = glory::ui::view().class(class).style(style);
        if let Some(name) = self.common.name.take() {
            root = root.name(name);
        }
        if let Some(reference) = self.common.node_ref {
            root = root.node_ref(reference);
        }
        let children = std::mem::replace(&mut self.children, ().into_children());
        if let Some(cap) = self.cap {
            root.fill(
                glory::ui::view()
                    .class("sapid-container-content")
                    .style(
                        Style::new()
                            .width(Length::Percent(100.0))
                            .max_width(cap)
                            .min_width(Length::Px(0.0)),
                    )
                    .fill(children),
            )
            .show_in(scope)
        } else {
            root.fill(children).show_in(scope)
        }
    }
}
macro_rules! widget {
    ($name:ident,$factory:ident,$class:literal,$style:expr) => {
        pub struct $name {
            layout: Layout,
        }
        pub fn $factory() -> $name {
            $name {
                layout: Layout::new($class, $style),
            }
        }
        impl $name {
            methods!();
        }
        impl Widget for $name {
            fn build(&mut self, scope: &Scope) -> Result {
                self.layout.build(scope)
            }
        }
    };
}
fn flex_style(direction: FlexDirection) -> Style {
    Style::new()
        .display(Display::Flex)
        .flex_direction(direction)
        .flex_wrap(FlexWrap::NoWrap)
        .gap(12.0)
        .min_width(Length::Px(0.0))
}
widget!(
    Box,
    box_,
    "sapid-box",
    Style::new().min_width(Length::Px(0.0))
);
widget!(Flex, flex, "sapid-flex", flex_style(FlexDirection::Row));
widget!(
    Stack,
    stack,
    "sapid-stack",
    flex_style(FlexDirection::Column)
);
widget!(
    Group,
    group,
    "sapid-group",
    Style {
        align: Some(Align::Center),
        ..flex_style(FlexDirection::Row).flex_wrap(FlexWrap::Wrap)
    }
);
widget!(
    Center,
    center,
    "sapid-center",
    Style {
        align: Some(Align::Center),
        justify: Some(Justify::Center),
        ..flex_style(FlexDirection::Row).gap(0.0)
    }
);
pub struct Container {
    layout: Layout,
}
pub fn container() -> Container {
    let mut layout = Layout::new(
        "sapid-container",
        Style {
            justify: Some(Justify::Center),
            ..flex_style(FlexDirection::Row)
                .gap(0.0)
                .width(Length::Percent(100.0))
        },
    );
    layout.cap = Some(Limit::Px(960.0));
    layout.default_padding = true;
    Container { layout }
}
impl Container {
    methods!();
}
impl Widget for Container {
    fn build(&mut self, scope: &Scope) -> Result {
        self.layout.build(scope)
    }
}
widget!(
    Space,
    space,
    "sapid-space",
    Style::new().width(Length::Px(0.0)).height(Length::Px(0.0))
);
macro_rules! flex_methods {($($name:ident),+)=>{$(impl $name {
    pub fn gap(mut self,gap:f64)->Self {self.layout.base.gap=Some(gap);self}
    pub fn align(mut self,align:Align)->Self {self.layout.base.align=Some(align);self}
    pub fn justify(mut self,justify:Justify)->Self {self.layout.base.justify=Some(justify);self}
    pub fn wrap(mut self,wrap:FlexWrap)->Self {self.layout.base.flex_wrap=Some(wrap);self}
})+}}
flex_methods!(Flex, Stack, Group);
impl Flex {
    pub fn direction(mut self, direction: FlexDirection) -> Self {
        self.layout.base.flex_direction = Some(direction);
        self
    }
}
impl Container {
    pub fn max_width(mut self, limit: Limit) -> Self {
        self.layout.cap = Some(limit);
        self
    }
    pub fn fluid(mut self) -> Self {
        self.layout.cap = Some(Limit::None);
        self
    }
}
