//! Named avatars with explicit text fallback and host-owned image completion.
use crate::appearance::{Common, configure};
use glory::render::{NodeId, SemanticId, Semantics};
use glory::style::{
    Align, BorderStyle, Display, Edges, Justify, Length, ObjectFit, Overflow, Style, WhiteSpace,
};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};
use std::collections::HashSet;
pub const MAX_AVATAR_GROUP_ITEMS: usize = 64;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarShape {
    #[default]
    Circle,
    Square,
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn valid_text(value: &str, max: usize) -> Result {
    if value.trim().is_empty() || value.len() > max {
        Err(invalid(
            "avatar names/fallbacks must be nonempty and within their byte budget",
        ))
    } else {
        Ok(())
    }
}
fn text_source(scope: &Scope, source: Vary<String>, max: usize) -> Result<Vary<String>> {
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
            if accepted.try_with_untracked(|previous| *previous != next)? {
                accepted.try_revise(|previous| *previous = next)?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            scope.report_error(error);
        }
    });
    Ok(accepted.into())
}
fn diameter(size: crate::Size) -> f64 {
    match size {
        crate::Size::ExtraSmall => 24.0,
        crate::Size::Small => 30.0,
        crate::Size::Medium => 36.0,
        crate::Size::Large => 42.0,
        crate::Size::ExtraLarge => 50.0,
    }
}
macro_rules! appearance {
    () => {
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
    };
}
pub struct Avatar {
    common: Common,
    theme: Option<crate::Theme>,
    size: Option<crate::Size>,
    name: Vary<String>,
    fallback: Vary<String>,
    shape: AvatarShape,
    id: Option<SemanticId>,
    overlap: Option<f64>,
    source: Option<Vary<String>>,
    loaded: Option<Box<dyn FnMut(u64)>>,
    failed: Option<Box<dyn FnMut(u64)>>,
}
pub fn avatar(name: impl Into<Vary<String>>) -> Avatar {
    Avatar {
        common: Common::default(),
        theme: None,
        size: None,
        name: name.into(),
        fallback: "?".into(),
        shape: AvatarShape::Circle,
        id: None,
        overlap: None,
        source: None,
        loaded: None,
        failed: None,
    }
}
impl Avatar {
    appearance!();
    fn validate(&self) -> Result {
        self.name.try_with(|value| valid_text(value, 256))??;
        self.fallback.try_with(|value| valid_text(value, 64))??;
        if let Some(source) = &self.source {
            source.try_with(|value| crate::image::validate_source(value))??;
        }
        Ok(())
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value.clone();
        self.theme = Some(value);
        self
    }
    pub fn size(mut self, value: crate::Size) -> Self {
        self.common.size = value;
        self.size = Some(value);
        self
    }
    pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.name = value.into();
        self
    }
    /// Explicit fallback text; no guessed locale-dependent initials.
    pub fn fallback(mut self, value: impl Into<Vary<String>>) -> Self {
        self.fallback = value.into();
        self
    }
    pub fn source(mut self, value: impl Into<Vary<String>>) -> Self {
        self.source = Some(value.into());
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
    pub fn shape(mut self, value: AvatarShape) -> Self {
        self.shape = value;
        self
    }
}
impl Widget for Avatar {
    fn build(&mut self, scope: &Scope) -> Result {
        // Validate both initial inputs before creating any visible node.
        self.validate()?;
        let name = text_source(scope, self.name.clone(), 256)?;
        let fallback = text_source(scope, self.fallback.clone(), 64)?;
        let width = diameter(self.common.size);
        let mut base = self.common.theme.label(self.common.size);
        base.display = Some(Display::InlineFlex);
        base.align = Some(Align::Center);
        base.justify = Some(Justify::Center);
        base.width = Some(Length::Px(width));
        base.height = Some(Length::Px(width));
        base.shrink = Some(0.0);
        base.radius = Some(match self.shape {
            AvatarShape::Circle => width / 2.0,
            AvatarShape::Square => self.common.theme.radius,
        });
        base.background = Some(self.common.theme.accent_soft);
        base.color = Some(self.common.theme.accent_text);
        base.font_weight = Some(600.0);
        base.white_space = Some(WhiteSpace::NoWrap);
        base.overflow_x = Some(Overflow::Hidden);
        base.overflow_y = Some(Overflow::Hidden);
        if let Some(overlap) = self.overlap {
            base.margin = Some(Edges {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: -overlap,
            });
            base.border_width = Some(Edges::all(2.0));
            base.border_style = Some(BorderStyle::Solid);
            base.border_color = Some(self.common.theme.surface);
        }
        let mut root =
            configure!(self.common, glory::ui::view(), scope, "sapid-avatar", base).name(name);
        if let Some(id) = self.id.take() {
            root = root.semantics(Some(Box::new(Semantics {
                id: Some(id),
                ..Default::default()
            })));
        }
        let mut text = glory::ui::text()
            .class("sapid-avatar-fallback")
            .selectable(false)
            .content(fallback);
        if let Some(source) = self.source.take() {
            let loaded = scope.cage(false);
            let mut previous = source.try_get()?;
            if source.is_reactive() {
                let source = source.clone();
                let weak = scope.downgrade();
                scope.effect(move || {
                    let Some(scope) = weak.upgrade() else { return };
                    let result = (|| -> Result {
                        let next = source.try_get()?;
                        // The Image driver reports invalid input and preserves
                        // its accepted resource. Keep that avatar visible too.
                        if crate::image::validate_source(&next).is_ok() && next != previous {
                            loaded.try_revise(|value| *value = false)?;
                            previous = next;
                        }
                        Ok(())
                    })();
                    if let Err(error) = result {
                        scope.report_error(error);
                    }
                });
            }
            text = text.style(loaded.map(|loaded| {
                Style::new().display(if *loaded {
                    Display::None
                } else {
                    Display::Block
                })
            }));
            let mut on_load = self.loaded.take();
            let mut on_error = self.failed.take();
            let weak = scope.downgrade();
            let error_scope = weak.clone();
            root = root.fill(
                crate::image::image(source)
                    .decorative()
                    .unstyled()
                    .class("sapid-avatar-image")
                    .fit(ObjectFit::Cover)
                    .style(loaded.map(|loaded| Style {
                        display: Some(if *loaded {
                            Display::Block
                        } else {
                            Display::None
                        }),
                        width: Some(Length::Percent(100.0)),
                        height: Some(Length::Percent(100.0)),
                        ..Style::new()
                    }))
                    .on_load(move |revision| {
                        let Some(scope) = weak.upgrade() else { return };
                        if let Err(error) = loaded.try_revise(|value| *value = true) {
                            scope.report_error(error);
                            return;
                        }
                        if let Some(callback) = &mut on_load {
                            callback(revision);
                        }
                    })
                    .on_error(move |revision| {
                        let Some(scope) = error_scope.upgrade() else {
                            return;
                        };
                        if let Err(error) = loaded.try_revise(|value| *value = false) {
                            scope.report_error(error);
                            return;
                        }
                        if let Some(callback) = &mut on_error {
                            callback(revision);
                        }
                    }),
            );
        }
        root.fill(text).show_in(scope)
    }
}
pub struct AvatarGroup {
    common: Common,
    namespace: String,
    avatars: Vec<(String, Avatar)>,
    visible: usize,
    overlap: f64,
    overflow_label: Box<dyn Fn(usize) -> String>,
}
pub fn avatar_group(namespace: impl Into<String>) -> AvatarGroup {
    AvatarGroup {
        common: Common::default(),
        namespace: namespace.into(),
        avatars: Vec::new(),
        visible: 4,
        overlap: 8.0,
        overflow_label: Box::new(|count| format!("{count} more people")),
    }
}
impl AvatarGroup {
    appearance!();
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn size(mut self, value: crate::Size) -> Self {
        self.common.size = value;
        self
    }
    pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn max_visible(mut self, value: usize) -> Self {
        self.visible = value;
        self
    }
    pub fn overlap(mut self, value: f64) -> Self {
        self.overlap = value;
        self
    }
    pub fn overflow_label(mut self, value: impl Fn(usize) -> String + 'static) -> Self {
        self.overflow_label = Box::new(value);
        self
    }
    pub fn avatar(mut self, key: impl Into<String>, value: Avatar) -> Self {
        if self.avatars.len() <= MAX_AVATAR_GROUP_ITEMS {
            self.avatars.push((key.into(), value));
        }
        self
    }
    pub fn avatars(mut self, values: impl IntoIterator<Item = (String, Avatar)>) -> Self {
        crate::document::extend_bounded(&mut self.avatars, values, MAX_AVATAR_GROUP_ITEMS);
        self
    }
}
impl Widget for AvatarGroup {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.avatars.len() > MAX_AVATAR_GROUP_ITEMS
            || !(1..=MAX_AVATAR_GROUP_ITEMS).contains(&self.visible)
            || !self.overlap.is_finite()
            || !(0.0..=24.0).contains(&self.overlap)
        {
            return Err(invalid(
                "avatar group accepts <=64 items, visible 1..=64, overlap finite in [0,24]",
            ));
        }
        let root_id = crate::composite::id("avatar-group", &self.namespace, "root", "root")?;
        let mut keys = HashSet::with_capacity(self.avatars.len());
        let mut ids = Vec::with_capacity(self.avatars.len());
        for (key, _) in &self.avatars {
            ids.push(crate::composite::id(
                "avatar-group",
                &self.namespace,
                "avatar",
                key,
            )?);
            if !keys.insert(key.as_str()) {
                return Err(invalid("duplicate avatar key"));
            }
        }
        for (_, avatar) in self.avatars.iter().take(self.visible) {
            avatar.validate()?;
        }
        let overflow = self.avatars.len().saturating_sub(self.visible);
        let overflow_name = if overflow > 0 {
            let label = (self.overflow_label)(overflow);
            valid_text(&label, 256)?;
            Some(label)
        } else {
            None
        };
        let mut visible = Vec::with_capacity(
            self.visible.min(self.avatars.len()) + usize::from(overflow > 0),
        );
        for (index, ((_, mut avatar), id)) in self.avatars.drain(..).zip(ids).enumerate() {
            if index >= self.visible {
                continue;
            }
            avatar.common.theme = avatar
                .theme
                .take()
                .unwrap_or_else(|| self.common.theme.clone());
            avatar.common.size = avatar.size.unwrap_or(self.common.size);
            avatar.common.unstyled |= self.common.unstyled;
            avatar.id = Some(id);
            avatar.overlap = Some(if index == 0 { 0.0 } else { self.overlap });
            visible.push(avatar);
        }
        if let Some(name) = overflow_name {
            let mut avatar = avatar(name).fallback(format!("+{overflow}"));
            avatar.common.theme = self.common.theme.clone();
            avatar.common.size = self.common.size;
            avatar.common.unstyled = self.common.unstyled;
            avatar.id = Some(crate::composite::id(
                "avatar-group",
                &self.namespace,
                "overflow",
                "overflow",
            )?);
            avatar.overlap = Some(self.overlap);
            visible.push(avatar.class("sapid-avatar-overflow"));
        }
        let base = Style {
            align: Some(Align::Center),
            ..Style::new().display(Display::Flex)
        };
        configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-avatar-group",
            base
        )
        .semantics(Some(Box::new(Semantics {
            id: Some(root_id),
            ..Default::default()
        })))
        .fill(visible)
        .show_in(scope)
    }
}
