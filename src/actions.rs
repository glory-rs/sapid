//! Named native button actions. Expansion is independent from actual focus.
use glory::render::{Direction, Event, EventKind, Orientation, Role, SemanticId, Semantics};
use glory::style::{
    Align, BorderStyle, Color, Display, Edges, FlexDirection, Justify, Length, Overflow, Style,
};
use glory::ui::{Children, IntoChildren, Result, Widget};
use glory::{Lotus, Scope};

use crate::appearance::{Common, configure, observe};
use crate::state::Value;

macro_rules! action_methods {
    () => {
        pub fn theme(mut self, theme: crate::Theme) -> Self {
            self.common.theme = theme;
            self
        }
        pub fn size(mut self, size: crate::Size) -> Self {
            self.common.size = size;
            self
        }
        pub fn unstyled(mut self) -> Self {
            self.common.unstyled = true;
            self
        }
        pub fn class(mut self, class: impl Into<Lotus<String>>) -> Self {
            self.common.class = class.into();
            self
        }
        pub fn style(mut self, style: impl Into<Lotus<Style>>) -> Self {
            self.common.style = style.into();
            self
        }
        pub fn enabled(mut self, enabled: impl Into<Lotus<bool>>) -> Self {
            self.common.enabled = enabled.into();
            self
        }
        pub fn focus(mut self, focus: glory::render::FocusMode) -> Self {
            self.common.focus = focus;
            self
        }
        pub fn node_ref(
            mut self,
            reference: glory::Cage<Option<glory::render::NodeId>>,
        ) -> Self {
            self.common.node_ref = Some(reference);
            self
        }
        pub fn on(mut self, kind: EventKind, handler: impl FnMut(Event) + 'static) -> Self {
            self.common
                .handlers
                .entry(kind)
                .or_default()
                .push(Box::new(handler));
            self
        }
        pub fn on_press(mut self, handler: impl FnMut() + 'static) -> Self {
            self.presses.push(Box::new(handler));
            self
        }
    };
}

fn square(common: &Common) -> Style {
    let side = match common.size {
        crate::Size::ExtraSmall => 24.0,
        crate::Size::Small => 30.0,
        crate::Size::Medium => 36.0,
        crate::Size::Large => 42.0,
        crate::Size::ExtraLarge => 50.0,
    };
    let mut style = common.theme.control(common.size);
    style.width = Some(Length::Px(side));
    style.height = Some(Length::Px(side));
    style.min_width = Some(Length::Px(side));
    style.padding = Some(Edges::all(0.0));
    style.display = Some(Display::InlineFlex);
    style.align = Some(Align::Center);
    style.justify = Some(Justify::Center);
    style
}

/// Portable icon-action appearances. Explicit style declarations take precedence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActionIconVariant {
    #[default]
    Default,
    Filled,
    Light,
    Outline,
    Subtle,
}
impl ActionIconVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "sapid-action-default",
            Self::Filled => "sapid-action-filled",
            Self::Light => "sapid-action-light",
            Self::Outline => "sapid-action-outline",
            Self::Subtle => "sapid-action-subtle",
        }
    }
}
fn validate_name(name: &str) -> Result {
    if name.trim().is_empty() {
        return Err(glory::render::Error::InvalidProps(
            "icon actions require a nonempty accessible name",
        )
        .into());
    }
    Ok(())
}

/// A square native Button with an explicit accessible name and caller icon slot.
/// The icon is display content; nested interactive children are rejected by Glory.
pub struct ActionIcon {
    common: Common,
    action_name: String,
    icon: Option<Children>,
    class: &'static str,
    variant: ActionIconVariant,
    presses: Vec<Box<dyn FnMut()>>,
}
pub fn action_icon(name: impl Into<String>) -> ActionIcon {
    ActionIcon {
        common: Common::default(),
        action_name: name.into(),
        icon: None,
        class: "sapid-action-icon",
        variant: ActionIconVariant::Default,
        presses: Vec::new(),
    }
}
impl ActionIcon {
    action_methods!();
    pub fn variant(mut self, variant: ActionIconVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.action_name = name.into();
        self
    }
    pub fn icon(mut self, icon: impl IntoChildren) -> Self {
        self.icon = Some(icon.into_children());
        self
    }
}
impl Widget for ActionIcon {
    fn build(&mut self, scope: &Scope) -> Result {
        validate_name(&self.action_name)?;
        let icon = self.icon.take().ok_or(glory::render::Error::InvalidProps(
            "action icon requires icon content",
        ))?;
        let mut base = square(&self.common);
        let theme = self.common.theme.clone();
        match self.variant {
            ActionIconVariant::Default => {}
            ActionIconVariant::Filled => {
                base.background = Some(theme.accent);
                base.color = Some(theme.on_accent);
                base.border_color = Some(theme.accent);
            }
            ActionIconVariant::Light | ActionIconVariant::Subtle => {
                base.background = Some(if self.variant == ActionIconVariant::Light {
                    theme.accent_soft
                } else {
                    Color(0, 0, 0, 0)
                });
                base.color = Some(if self.variant == ActionIconVariant::Subtle {
                    theme.muted
                } else {
                    theme.accent_text
                });
                base.border_color = Some(Color(0, 0, 0, 0));
            }
            ActionIconVariant::Outline => {
                base.background = Some(Color(0, 0, 0, 0));
                base.color = Some(theme.accent_text);
                base.border_color = Some(theme.accent);
            }
        }
        let own_style = self.common.style.clone();
        let enabled = self.common.enabled.clone();
        let plain = self.common.unstyled;
        let state_style = move || {
            let mut base = if plain { Style::new() } else { base.clone() };
            if !plain && !enabled.get() {
                base.background = Some(theme.disabled);
                base.color = Some(theme.disabled_text);
                base.border_color = Some(theme.border);
            }
            crate::appearance::merge(base, own_style.get())
        };
        self.common.style =
            if self.common.enabled.is_reactive() || self.common.style.is_reactive() {
                scope.bond(state_style).into()
            } else {
                state_style().into()
            };
        let class = format!("{} {}", self.class, self.variant.class());
        let mut button = configure!(
            self.common,
            glory::ui::button(),
            scope,
            &class,
            Style::new()
        )
        .name(self.action_name.clone())
        .enabled(self.common.enabled.clone())
        .fill(icon);
        let owner = scope.id();
        let mut presses = std::mem::take(&mut self.presses);
        let mut observers = self
            .common
            .handlers
            .remove(&EventKind::Activate)
            .unwrap_or_default();
        if !presses.is_empty() || !observers.is_empty() {
            button = button.on(EventKind::Activate, move |event| {
                for press in &mut presses {
                    if !owner.is_alive() {
                        return;
                    }
                    press();
                }
                observe(&mut observers, owner, event);
            });
        }
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            button = button.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        button.show_in(scope)
    }
}

/// A named close action, reusing the same native activation/disabled behavior.
pub struct CloseButton {
    inner: ActionIcon,
}
pub fn close_button() -> CloseButton {
    let mut inner = action_icon("Close")
        .variant(ActionIconVariant::Subtle)
        .icon(glory::ui::text().content("×").selectable(false));
    inner.class = "sapid-action-icon sapid-close-button";
    CloseButton { inner }
}
impl CloseButton {
    /// Replace the decorative close glyph while retaining the named action.
    pub fn icon(mut self, icon: impl IntoChildren) -> Self {
        self.inner = self.inner.icon(icon);
        self
    }
    pub fn variant(mut self, variant: ActionIconVariant) -> Self {
        self.inner = self.inner.variant(variant);
        self
    }
    pub fn theme(mut self, theme: crate::Theme) -> Self {
        self.inner = self.inner.theme(theme);
        self
    }
    pub fn size(mut self, size: crate::Size) -> Self {
        self.inner = self.inner.size(size);
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.inner = self.inner.unstyled();
        self
    }
    pub fn class(mut self, class: impl Into<Lotus<String>>) -> Self {
        self.inner = self.inner.class(class);
        self
    }
    pub fn style(mut self, style: impl Into<Lotus<Style>>) -> Self {
        self.inner = self.inner.style(style);
        self
    }
    pub fn enabled(mut self, enabled: impl Into<Lotus<bool>>) -> Self {
        self.inner = self.inner.enabled(enabled);
        self
    }
    pub fn focus(mut self, focus: glory::render::FocusMode) -> Self {
        self.inner = self.inner.focus(focus);
        self
    }
    pub fn node_ref(mut self, reference: glory::Cage<Option<glory::render::NodeId>>) -> Self {
        self.inner = self.inner.node_ref(reference);
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.inner = self.inner.name(name);
        self
    }
    pub fn on(mut self, kind: EventKind, handler: impl FnMut(Event) + 'static) -> Self {
        self.inner = self.inner.on(kind, handler);
        self
    }
    pub fn on_press(mut self, handler: impl FnMut() + 'static) -> Self {
        self.inner = self.inner.on_press(handler);
        self
    }
}
impl Widget for CloseButton {
    fn build(&mut self, scope: &Scope) -> Result {
        self.inner.build(scope)
    }
}

/// A named disclosure button. It controls only expanded state: panel ownership,
/// navigation and actual focus remain with the application.
pub struct Burger {
    common: Common,
    open: Value<bool>,
    on_change: Option<Box<dyn FnMut(bool)>>,
    open_name: String,
    close_name: String,
    animated: Lotus<bool>,
    duration_millis: u16,
    controls: Option<SemanticId>,
    presses: Vec<Box<dyn FnMut()>>,
}
pub fn burger() -> Burger {
    Burger {
        common: Common::default(),
        open: Value::default(),
        on_change: None,
        open_name: "Open navigation".into(),
        close_name: "Close navigation".into(),
        animated: true.into(),
        duration_millis: 180,
        controls: None,
        presses: Vec::new(),
    }
}
impl Burger {
    pub fn animated(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.animated = value.into();
        self
    }
    pub fn duration_millis(mut self, value: u16) -> Self {
        self.duration_millis = value;
        self
    }
    action_methods!();
    pub fn open(mut self, source: impl Into<Lotus<bool>>) -> Self {
        self.open.source = Some(source.into());
        self
    }
    pub fn default_open(mut self, open: bool) -> Self {
        self.open.default = open;
        self
    }
    pub fn on_open_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }
    pub fn labels(mut self, open: impl Into<String>, close: impl Into<String>) -> Self {
        self.open_name = open.into();
        self.close_name = close.into();
        self
    }
    pub fn controls(mut self, target: SemanticId) -> Self {
        self.controls = Some(target);
        self
    }
}
impl Widget for Burger {
    fn build(&mut self, scope: &Scope) -> Result {
        validate_name(&self.open_name)?;
        validate_name(&self.close_name)?;
        glory::render::GraphicTransition {
            duration_millis: self.duration_millis,
        }
        .validate()?;
        let control = std::mem::take(&mut self.open).mount(scope);
        let open = control.source.clone();
        let target = self.controls.clone();
        let semantics = scope.bond(move || {
            Some(Box::new(Semantics {
                expanded: Some(open.get()),
                pressed: Some(open.get()),
                controls: target.clone(),
                ..Default::default()
            }))
        });
        let open = control.source.clone();
        let open_name = self.open_name.clone();
        let close_name = self.close_name.clone();
        let name = scope.bond(move || {
            if open.get() {
                close_name.clone()
            } else {
                open_name.clone()
            }
        });
        let open = control.source.clone();
        let animated = self.animated.clone();
        let duration_millis = self.duration_millis;
        let theme = self.common.theme.clone();
        let unstyled = self.common.unstyled;
        let enabled = self.common.enabled.clone();
        let own_style = self.common.style.clone();
        let mut base = square(&self.common)
            .display(Display::InlineFlex)
            .padding(Edges::all(4.0));
        base.align = Some(glory::style::Align::Center);
        base.justify = Some(glory::style::Justify::Center);
        self.common.style = scope
            .bond(move || {
                let mut style = if unstyled { Style::new() } else { base.clone() };
                if !unstyled && !enabled.get() {
                    style.background = Some(theme.disabled);
                    style.color = Some(theme.disabled_text);
                    style.border_color = Some(theme.border);
                }
                crate::appearance::merge(style, own_style.get())
            })
            .into();
        let style = self.common.style.clone();
        let fallback = self.common.theme.foreground;
        let drawing = scope.bond(move || {
            use glory::render::{GraphicPoint as P, PathCommand as C};
            let coordinates = if open.get() {
                [(5., 5., 19., 19.), (12., 12., 12., 12.), (5., 19., 19., 5.)]
            } else {
                [(4., 6., 20., 6.), (4., 12., 20., 12.), (4., 18., 20., 18.)]
            };
            glory::render::GraphicProps {
                commands: coordinates
                    .into_iter()
                    .flat_map(|(x, y, a, b)| [C::Move(P::new(x, y)), C::Line(P::new(a, b))])
                    .collect(),
                stroke: Some(glory::render::GraphicStroke {
                    color: style.get().color.unwrap_or(fallback),
                    width: 2.,
                    cap: glory::render::StrokeCap::Round,
                }),
                transition: animated
                    .get()
                    .then_some(glory::render::GraphicTransition { duration_millis }),
                ..Default::default()
            }
        });
        let icon_side = match self.common.size {
            crate::Size::ExtraSmall => 12.,
            crate::Size::Small => 15.,
            crate::Size::Medium => 18.,
            crate::Size::Large => 21.,
            crate::Size::ExtraLarge => 25.,
        };
        let mut button = configure!(
            self.common,
            glory::ui::button(),
            scope,
            "sapid-burger",
            Style::new()
        )
        .enabled(self.common.enabled.clone())
        .semantics(semantics)
        .name(name)
        .label("")
        .fill(
            glory::ui::graphic()
                .class("sapid-burger-graphic")
                .drawing(drawing)
                .style(
                    Style::new()
                        .width(Length::Px(icon_side))
                        .height(Length::Px(icon_side)),
                ),
        );
        let weak = scope.downgrade();
        let owner = scope.id();
        let mut callback = self.on_change.take();
        let mut presses = std::mem::take(&mut self.presses);
        let mut observers = self
            .common
            .handlers
            .remove(&EventKind::Activate)
            .unwrap_or_default();
        button = button.on(EventKind::Activate, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let next = !control.source.try_get()?;
                control.accept(&next)?;
                if let Some(callback) = &mut callback {
                    callback(next);
                }
                for press in &mut presses {
                    if !owner.is_alive() {
                        return Ok(());
                    }
                    press();
                }
                observe(&mut observers, owner, event);
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            button = button.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        button.show_in(scope)
    }
}

/// Connected native buttons. Toolbar mode opts into a shared roving Tab entry.
pub struct ButtonGroup {
    common: Common,
    buttons: Vec<crate::Button>,
    orientation: Orientation,
    joined: bool,
    toolbar: bool,
    looping: bool,
    direction: Direction,
}
pub fn button_group() -> ButtonGroup {
    ButtonGroup {
        common: Common::default(),
        buttons: Vec::new(),
        orientation: Orientation::Horizontal,
        joined: true,
        toolbar: false,
        looping: true,
        direction: Direction::Ltr,
    }
}
impl ButtonGroup {
    pub fn toolbar(mut self, enabled: bool) -> Self {
        self.toolbar = enabled;
        self
    }
    pub fn loop_focus(mut self, enabled: bool) -> Self {
        self.looping = enabled;
        self
    }
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
    pub fn theme(mut self, theme: crate::Theme) -> Self {
        self.common.theme = theme;
        self
    }
    pub fn size(mut self, size: crate::Size) -> Self {
        self.common.size = size;
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn class(mut self, class: impl Into<Lotus<String>>) -> Self {
        self.common.class = class.into();
        self
    }
    pub fn style(mut self, style: impl Into<Lotus<Style>>) -> Self {
        self.common.style = style.into();
        self
    }
    pub fn enabled(mut self, enabled: impl Into<Lotus<bool>>) -> Self {
        self.common.enabled = enabled.into();
        self
    }
    pub fn node_ref(mut self, reference: glory::Cage<Option<glory::render::NodeId>>) -> Self {
        self.common.node_ref = Some(reference);
        self
    }
    pub fn name(mut self, name: impl Into<Lotus<String>>) -> Self {
        self.common.name = Some(name.into());
        self
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn joined(mut self, joined: bool) -> Self {
        self.joined = joined;
        self
    }
    pub fn button(mut self, button: crate::Button) -> Self {
        crate::document::extend_bounded(&mut self.buttons, std::iter::once(button), 128);
        self
    }
    pub fn buttons(mut self, buttons: impl IntoIterator<Item = crate::Button>) -> Self {
        crate::document::extend_bounded(&mut self.buttons, buttons, 128);
        self
    }
}
impl Widget for ButtonGroup {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.buttons.is_empty() || self.buttons.len() > 128 {
            return Err(glory::render::Error::InvalidProps(
                "button group requires 1..=128 buttons",
            )
            .into());
        }
        let mut base = Style::new()
            .display(Display::Flex)
            .flex_direction(match self.orientation {
                Orientation::Horizontal => FlexDirection::Row,
                Orientation::Vertical => FlexDirection::Column,
            })
            .gap(if self.joined { 0.0 } else { 8.0 });
        if self.joined {
            base.border_width = Some(Edges::all(1.0));
            base.border_style = Some(BorderStyle::Solid);
            base.border_color = Some(self.common.theme.border);
            base.radius = Some(self.common.theme.radius);
            base.overflow_x = Some(Overflow::Hidden);
            base.overflow_y = Some(Overflow::Hidden);
        }
        let structural = Style::new()
            .display(Display::Flex)
            .flex_direction(match self.orientation {
                Orientation::Horizontal => FlexDirection::Row,
                Orientation::Vertical => FlexDirection::Column,
            })
            .gap(if self.joined { 0.0 } else { 8.0 });
        let own_style = self.common.style.clone();
        self.common.style = if own_style.is_reactive() {
            scope
                .bond(move || crate::appearance::merge(structural.clone(), own_style.get()))
                .into()
        } else {
            crate::appearance::merge(structural, own_style.get()).into()
        };
        let count = self.buttons.len();
        let navigation = if self.toolbar {
            let collection = crate::Collection::new(
                scope,
                crate::CollectionLimits {
                    items: 128,
                    label_bytes: 1,
                },
            )?;
            let roving = crate::RovingFocus::new(scope, collection.clone(), self.looping)?;
            Some((collection, roving))
        } else {
            None
        };
        let mut buttons = Vec::with_capacity(count);
        for (index, button) in self.buttons.drain(..).enumerate() {
            let layout = if self.joined {
                let separator = if index + 1 == count { 0.0 } else { 1.0 };
                Style::new()
                    .radius(0.0)
                    .border_width(match self.orientation {
                        Orientation::Horizontal => Edges {
                            top: 0.0,
                            bottom: 0.0,
                            left: 0.0,
                            right: separator,
                        },
                        Orientation::Vertical => Edges {
                            top: 0.0,
                            bottom: separator,
                            left: 0.0,
                            right: 0.0,
                        },
                    })
            } else {
                Style::new()
            };
            let button = button.in_group(
                scope,
                self.common.theme.clone(),
                self.common.size,
                self.common.enabled.clone(),
                self.common.unstyled,
                layout,
            );
            buttons.push(if let Some((collection, roving)) = &navigation {
                button.in_toolbar(
                    scope,
                    collection.clone(),
                    roving.clone(),
                    format!("action-{index}"),
                )?
            } else {
                button
            });
        }
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-button-group",
            base
        )
        .role(if self.toolbar {
            Role::Toolbar
        } else {
            Role::Group
        })
        .fill(buttons);
        if let Some((_, roving)) = navigation {
            root = root.semantics(Some(Box::new(Semantics {
                navigation: Some(self.orientation),
                navigation_direction: self.direction,
                ..Default::default()
            })));
            let live = roving.clone();
            let weak = scope.downgrade();
            root = root.on(EventKind::Navigate, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    if let Event::Navigate { from, command } = event
                        && let Some(next) = live.destination(from, command)?
                    {
                        crate::composite::focus_node(&scope, Some(next.node))?;
                        live.accept(&next)?;
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                }
            });
            root.show_in(scope)?;
            roving.repair(None)?;
            let weak = scope.downgrade();
            scope.effect(move || {
                let Some(scope) = weak.upgrade() else { return };
                if let Err(error) = roving.repair(None) {
                    scope.report_error(error);
                }
            });
            Ok(())
        } else {
            root.show_in(scope)
        }
    }
}
