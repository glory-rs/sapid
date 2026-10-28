//! Passive display widgets and a separately named removable Pill action.
use glory::render::{EventKind, NodeId, Semantics, TextKind};
use glory::style::{
    Align, BorderStyle, BoxShadow, Display, Edges, FlexDirection, Length, Shadow, Style,
};
use glory::ui::{Children, IntoChildren, Result, Widget};
use glory::{Cage, Scope, Vary};

use crate::appearance::{Common, configure, observe};
use crate::state::Value;

macro_rules! display_methods {
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
    };
}

fn column(gap: f64) -> Style {
    Style::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(gap)
        .min_width(Length::Px(0.0))
}
fn take(children: &mut Children) -> Children {
    std::mem::replace(children, ().into_children())
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShadowSize {
    #[default]
    None,
    Small,
    Medium,
    Large,
}
impl ShadowSize {
    fn value(self, theme: &crate::Theme) -> BoxShadow {
        let (y, blur, spread, color) = match self {
            Self::None => return BoxShadow::None,
            Self::Small => (1., 3., 0., theme.shadow_small),
            Self::Medium => (4., 12., -2., theme.shadow_medium),
            Self::Large => (8., 24., -4., theme.shadow_large),
        };
        BoxShadow::Outset(Shadow {
            x: 0.0,
            y,
            blur,
            spread,
            color,
        })
    }
}
fn surface(common: &Common, bordered: bool, shadow: Option<ShadowSize>) -> Style {
    let mut style = Style {
        background: Some(common.theme.surface),
        color: Some(common.theme.foreground),
        ..column(12.0)
            .padding(Edges::all(16.0))
            .radius(common.theme.radius)
    };
    if bordered {
        style = style
            .border_width(Edges::all(1.0))
            .border_style(BorderStyle::Solid)
            .border_color(common.theme.border);
    }
    style.box_shadow = shadow.map(|shadow| shadow.value(&common.theme));
    style
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Soft,
    Solid,
    Outline,
}

/// A passive text label. It has no implicit status/live-region behavior.
pub struct Badge {
    common: Common,
    content: Vary<String>,
    variant: BadgeVariant,
}
pub fn badge() -> Badge {
    Badge {
        common: Common::default(),
        content: "".into(),
        variant: BadgeVariant::Soft,
    }
}
impl Badge {
    display_methods!();
    pub fn content(mut self, content: impl Into<Vary<String>>) -> Self {
        self.content = content.into();
        self
    }
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }
}
impl Widget for Badge {
    fn build(&mut self, scope: &Scope) -> Result {
        let theme = &self.common.theme;
        let mut base = theme
            .label(self.common.size)
            .padding(Edges {
                top: 2.0,
                bottom: 2.0,
                left: 8.0,
                right: 8.0,
            })
            .radius(100.0);
        let class = match self.variant {
            BadgeVariant::Soft => {
                base.background = Some(theme.accent_soft);
                base.color = Some(theme.accent_text);
                "sapid-badge sapid-badge-soft"
            }
            BadgeVariant::Solid => {
                base.background = Some(theme.accent);
                base.color = Some(theme.on_accent);
                "sapid-badge sapid-badge-solid"
            }
            BadgeVariant::Outline => {
                base.color = Some(theme.accent_text);
                base.border_width = Some(Edges::all(1.0));
                base.border_style = Some(BorderStyle::Solid);
                base.border_color = Some(theme.accent);
                "sapid-badge sapid-badge-outline"
            }
        };
        configure!(self.common, glory::ui::text(), scope, class, base)
            .content(self.content.clone())
            .show_in(scope)
    }
}

/// Removable text with its own native close button. A controlled visible source
/// is never written; on_remove proposes dismissal. Hidden content keeps identity.
pub struct Pill {
    common: Common,
    content: Vary<String>,
    visible: Value<bool>,
    remove_label: Option<String>,
    on_remove: Option<Box<dyn FnMut()>>,
    remove_ref: Option<Cage<Option<NodeId>>>,
}
pub fn pill() -> Pill {
    Pill {
        common: Common::default(),
        content: "".into(),
        visible: Value {
            source: None,
            default: true,
        },
        remove_label: None,
        on_remove: None,
        remove_ref: None,
    }
}
impl Pill {
    display_methods!();
    pub fn content(mut self, content: impl Into<Vary<String>>) -> Self {
        self.content = content.into();
        self
    }
    /// Add removal with a localized, nonempty accessible action name.
    pub fn removable(mut self, label: impl Into<String>) -> Self {
        self.remove_label = Some(label.into());
        self
    }
    pub fn visible(mut self, source: impl Into<Vary<bool>>) -> Self {
        self.visible.source = Some(source.into());
        self
    }
    pub fn default_visible(mut self, visible: bool) -> Self {
        self.visible.default = visible;
        self
    }
    pub fn enabled(mut self, enabled: impl Into<Vary<bool>>) -> Self {
        self.common.enabled = enabled.into();
        self
    }
    pub fn on_remove(mut self, callback: impl FnMut() + 'static) -> Self {
        self.on_remove = Some(Box::new(callback));
        self
    }
    pub fn remove_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
        self.remove_ref = Some(reference);
        self
    }
    /// Observe removal activation after internal behavior and on_remove.
    pub fn on_remove_press(
        mut self,
        observer: impl FnMut(glory::render::Event) + 'static,
    ) -> Self {
        self.common
            .handlers
            .entry(EventKind::Activate)
            .or_default()
            .push(Box::new(observer));
        self
    }
}
impl Widget for Pill {
    fn build(&mut self, scope: &Scope) -> Result {
        if let Some(label) = &self.remove_label
            && label.trim().is_empty()
        {
            return Err(
                glory::render::Error::InvalidProps("pill removal requires a name").into(),
            );
        }
        let visible = std::mem::replace(
            &mut self.visible,
            Value {
                source: None,
                default: true,
            },
        )
        .mount(scope);
        let source = visible.source.clone();
        let metadata = scope.bond(move || {
            Some(Box::new(Semantics {
                hidden: !source.get(),
                ..Default::default()
            }))
        });
        let base = crate::tag::style(&self.common.theme, self.common.size, true);
        let mut root = configure!(self.common, glory::ui::view(), scope, "sapid-pill", base)
            .semantics(metadata)
            .fill(
                glory::ui::text()
                    .class("sapid-pill-label")
                    .content(self.content.clone()),
            );
        if let Some(label) = self.remove_label.take() {
            let weak = scope.downgrade();
            let owner = scope.id();
            let mut callback = self.on_remove.take();
            let mut observers = self
                .common
                .handlers
                .remove(&EventKind::Activate)
                .unwrap_or_default();
            let enabled = self.common.enabled.clone();
            let source = visible.source.clone();
            let button_enabled = scope.bond(move || enabled.get() && source.get());
            let mut button = crate::close_button()
                .theme(self.common.theme.clone())
                .size(self.common.size)
                .style(crate::tag::remove_style(self.common.size))
                .class("sapid-pill-remove")
                .name(label)
                .enabled(button_enabled)
                .on_press(move || {
                    let Some(scope) = weak.upgrade() else { return };
                    let result = (|| -> Result {
                        if !visible.source.try_get()? {
                            return Ok(());
                        }
                        visible.accept(&false)?;
                        if let Some(callback) = &mut callback {
                            callback();
                        }
                        observe(&mut observers, owner, glory::render::Event::Activate);
                        Ok(())
                    })();
                    if let Err(error) = result {
                        scope.report_error(error);
                    }
                });
            if self.common.unstyled {
                button = button.unstyled();
            }
            if let Some(reference) = self.remove_ref {
                button = button.node_ref(reference);
            }
            root = root.fill(button);
        }
        root.show_in(scope)
    }
}

/// Passive surface. Interactive content retains its own intrinsic semantics.
pub struct Paper {
    common: Common,
    children: Children,
    bordered: bool,
    shadow: Option<ShadowSize>,
}
pub fn paper() -> Paper {
    Paper {
        common: Common::default(),
        children: ().into_children(),
        bordered: false,
        shadow: None,
    }
}
impl Paper {
    display_methods!();
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.children = children.into_children();
        self
    }
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }
    pub fn shadow(mut self, shadow: ShadowSize) -> Self {
        self.shadow = Some(shadow);
        self
    }
    pub fn name(mut self, name: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(name.into());
        self
    }
}
impl Widget for Paper {
    fn build(&mut self, scope: &Scope) -> Result {
        let base = surface(&self.common, self.bordered, self.shadow);
        configure!(self.common, glory::ui::view(), scope, "sapid-paper", base)
            .fill(take(&mut self.children))
            .show_in(scope)
    }
}

/// A passive card with ordered header, title/description, body and footer slots.
pub struct Card {
    common: Common,
    title: Option<Vary<String>>,
    description: Option<Vary<String>>,
    level: u8,
    header: Children,
    body: Children,
    footer: Children,
    bordered: bool,
    shadow: Option<ShadowSize>,
}
pub fn card() -> Card {
    Card {
        common: Common::default(),
        title: None,
        description: None,
        level: 3,
        header: ().into_children(),
        body: ().into_children(),
        footer: ().into_children(),
        bordered: true,
        shadow: None,
    }
}
impl Card {
    display_methods!();
    pub fn title(mut self, title: impl Into<Vary<String>>) -> Self {
        self.title = Some(title.into());
        self
    }
    pub fn description(mut self, description: impl Into<Vary<String>>) -> Self {
        self.description = Some(description.into());
        self
    }
    pub fn heading_level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }
    pub fn header(mut self, children: impl IntoChildren) -> Self {
        self.header = children.into_children();
        self
    }
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.body = children.into_children();
        self
    }
    pub fn footer(mut self, children: impl IntoChildren) -> Self {
        self.footer = children.into_children();
        self
    }
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }
    pub fn shadow(mut self, shadow: ShadowSize) -> Self {
        self.shadow = Some(shadow);
        self
    }
    pub fn name(mut self, name: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(name.into());
        self
    }
}
impl Widget for Card {
    fn build(&mut self, scope: &Scope) -> Result {
        validate_level(self.level)?;
        let base = surface(&self.common, self.bordered, self.shadow);
        let mut header = glory::ui::view()
            .class("sapid-card-header")
            .fill(take(&mut self.header));
        if let Some(title) = self.title.take() {
            let mut title = glory::ui::text()
                .class("sapid-card-title")
                .kind(TextKind::Heading(self.level))
                .content(title);
            if !self.common.unstyled {
                title =
                    title.style(self.common.theme.label(self.common.size).font_weight(600.0));
            }
            header = header.fill(title);
        }
        if let Some(description) = self.description.take() {
            let mut text = glory::ui::text()
                .class("sapid-card-description")
                .kind(TextKind::Paragraph)
                .content(description);
            if !self.common.unstyled {
                text = text.style(self.common.theme.description(self.common.size));
            }
            header = header.fill(text);
        }
        if !self.common.unstyled {
            header = header.style(column(4.0));
        }
        configure!(self.common, glory::ui::view(), scope, "sapid-card", base)
            .fill(header)
            .fill(
                glory::ui::view()
                    .class("sapid-card-body")
                    .fill(take(&mut self.body)),
            )
            .fill(
                glory::ui::view()
                    .class("sapid-card-footer")
                    .fill(take(&mut self.footer)),
            )
            .show_in(scope)
    }
}

fn validate_level(level: u8) -> Result {
    if !(1..=6).contains(&level) {
        return Err(glory::render::Error::InvalidProps(
            "heading level must be between one and six",
        )
        .into());
    }
    Ok(())
}

/// Ordinary empty content, without unsolicited alert/live announcements.
pub struct EmptyState {
    common: Common,
    title: Vary<String>,
    description: Option<Vary<String>>,
    level: u8,
    illustration: Children,
    actions: Children,
    children: Children,
}
pub fn empty_state() -> EmptyState {
    EmptyState {
        common: Common::default(),
        title: "No results".into(),
        description: None,
        level: 2,
        illustration: ().into_children(),
        actions: ().into_children(),
        children: ().into_children(),
    }
}
impl EmptyState {
    display_methods!();
    pub fn title(mut self, title: impl Into<Vary<String>>) -> Self {
        self.title = title.into();
        self
    }
    pub fn description(mut self, description: impl Into<Vary<String>>) -> Self {
        self.description = Some(description.into());
        self
    }
    pub fn heading_level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }
    pub fn illustration(mut self, children: impl IntoChildren) -> Self {
        self.illustration = children.into_children();
        self
    }
    pub fn actions(mut self, children: impl IntoChildren) -> Self {
        self.actions = children.into_children();
        self
    }
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.children = children.into_children();
        self
    }
}
impl Widget for EmptyState {
    fn build(&mut self, scope: &Scope) -> Result {
        validate_level(self.level)?;
        let base = Style {
            align: Some(Align::Center),
            ..column(12.0).padding(Edges::all(24.0))
        };
        let mut title = glory::ui::text()
            .class("sapid-empty-title")
            .kind(TextKind::Heading(self.level))
            .content(self.title.clone());
        if !self.common.unstyled {
            title = title.style(self.common.theme.label(self.common.size).font_weight(600.0));
        }
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-empty-state",
            base
        )
        .fill(
            glory::ui::view()
                .class("sapid-empty-illustration")
                .fill(take(&mut self.illustration)),
        )
        .fill(title);
        if let Some(description) = self.description.take() {
            let mut text = glory::ui::text()
                .class("sapid-empty-description")
                .kind(TextKind::Paragraph)
                .content(description);
            if !self.common.unstyled {
                text = text.style(self.common.theme.description(self.common.size));
            }
            root = root.fill(text);
        }
        root.fill(
            glory::ui::view()
                .class("sapid-empty-body")
                .fill(take(&mut self.children)),
        )
        .fill(
            glory::ui::view()
                .class("sapid-empty-actions")
                .fill(take(&mut self.actions)),
        )
        .show_in(scope)
    }
}
