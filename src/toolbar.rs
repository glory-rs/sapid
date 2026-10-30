//! Finite grouped actions with one roving entry and an explicit selection bar.
use crate::appearance::{Common, configure, observe};
use crate::composite;
use glory::render::{Direction, Event, EventKind, Orientation, Role, Semantics};
use glory::style::{BorderStyle, Display, Edges, FlexDirection, FlexWrap, Length, Style};
use glory::ui::{Result, Widget};
use glory::{Scope, Vary};

pub const MAX_TOOLBAR_ACTIONS: usize = 128;
pub const MAX_TOOLBAR_GROUPS: usize = 32;
pub struct ToolbarGroup {
    key: String,
    name: Option<Vary<String>>,
    buttons: Vec<crate::Button>,
}
pub fn toolbar_group(key: impl Into<String>) -> ToolbarGroup {
    ToolbarGroup {
        key: key.into(),
        name: None,
        buttons: Vec::new(),
    }
}
impl ToolbarGroup {
    pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.name = Some(value.into());
        self
    }
    pub fn button(mut self, value: crate::Button) -> Self {
        crate::document::extend_bounded(
            &mut self.buttons,
            std::iter::once(value),
            MAX_TOOLBAR_ACTIONS,
        );
        self
    }
    pub fn buttons(mut self, value: impl IntoIterator<Item = crate::Button>) -> Self {
        crate::document::extend_bounded(&mut self.buttons, value, MAX_TOOLBAR_ACTIONS);
        self
    }
}
pub struct Toolbar {
    key: String,
    common: Common,
    groups: Vec<ToolbarGroup>,
    orientation: Orientation,
    direction: Direction,
    looping: bool,
    visible: Vary<bool>,
}
pub fn toolbar(key: impl Into<String>) -> Toolbar {
    let common = Common {
        name: Some("Actions".into()),
        ..Default::default()
    };
    Toolbar {
        key: key.into(),
        common,
        groups: Vec::new(),
        orientation: Orientation::Horizontal,
        direction: Direction::Ltr,
        looping: true,
        visible: true.into(),
    }
}
impl Toolbar {
    pub fn group(mut self, value: ToolbarGroup) -> Self {
        crate::document::extend_bounded(
            &mut self.groups,
            std::iter::once(value),
            MAX_TOOLBAR_GROUPS,
        );
        self
    }
    pub fn groups(mut self, value: impl IntoIterator<Item = ToolbarGroup>) -> Self {
        crate::document::extend_bounded(&mut self.groups, value, MAX_TOOLBAR_GROUPS);
        self
    }
    pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn size(mut self, value: crate::Size) -> Self {
        self.common.size = value;
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
    pub fn enabled(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.common.enabled = value.into();
        self
    }
    pub fn visible(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.visible = value.into();
        self
    }
    pub fn orientation(mut self, value: Orientation) -> Self {
        self.orientation = value;
        self
    }
    pub fn direction(mut self, value: Direction) -> Self {
        self.direction = value;
        self
    }
    pub fn loop_focus(mut self, value: bool) -> Self {
        self.looping = value;
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn node_ref(mut self, value: glory::Cage<Option<glory::render::NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
    pub fn on(mut self, kind: EventKind, callback: impl FnMut(Event) + 'static) -> Self {
        self.common
            .handlers
            .entry(kind)
            .or_default()
            .push(Box::new(callback));
        self
    }
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn label(value: &str) -> Result {
    if value.trim().is_empty() || value.len() > 256 || value.contains(['\0', '\r', '\n']) {
        Err(invalid(
            "toolbar names must be nonempty canonical text within 256 bytes",
        ))
    } else {
        Ok(())
    }
}
impl Widget for Toolbar {
    fn build(&mut self, scope: &Scope) -> Result {
        let id = composite::id("toolbar", &self.key, "root", "root")?;
        let total = self
            .groups
            .iter()
            .map(|group| group.buttons.len())
            .sum::<usize>();
        if self.groups.is_empty()
            || self.groups.len() > MAX_TOOLBAR_GROUPS
            || !(1..=MAX_TOOLBAR_ACTIONS).contains(&total)
            || self.groups.iter().any(|group| group.buttons.is_empty())
        {
            return Err(invalid(
                "toolbar needs 1..=32 nonempty groups and 1..=128 total actions",
            ));
        }
        let mut keys = std::collections::HashSet::new();
        for group in &self.groups {
            composite::id("toolbar", &self.key, "group", &group.key)?;
            if !keys.insert(&group.key) {
                return Err(invalid("duplicate toolbar group key"));
            }
        }
        let name = self
            .common
            .name
            .as_ref()
            .ok_or_else(|| invalid("toolbar requires a name"))?;
        self.common.name = Some(composite::validated(scope, name, |value| label(value))?.into());
        let collection = crate::Collection::new(
            scope,
            crate::CollectionLimits {
                items: MAX_TOOLBAR_ACTIONS,
                label_bytes: 1,
            },
        )?;
        let roving = crate::RovingFocus::new(scope, collection.clone(), self.looping)?;
        let axis = match self.orientation {
            Orientation::Horizontal => FlexDirection::Row,
            Orientation::Vertical => FlexDirection::Column,
        };
        let mut groups = Vec::with_capacity(self.groups.len());
        for mut group in self.groups.drain(..) {
            let group_id = composite::id("toolbar", &self.key, "group", &group.key)?;
            let mut buttons = Vec::with_capacity(group.buttons.len());
            for (index, button) in group.buttons.drain(..).enumerate() {
                buttons.push(
                    button
                        .in_group(
                            scope,
                            self.common.theme.clone(),
                            self.common.size,
                            self.common.enabled.clone(),
                            self.common.unstyled,
                            Style::new(),
                        )
                        .in_toolbar(
                            scope,
                            collection.clone(),
                            roving.clone(),
                            format!("{}-{index}", group.key),
                        )?,
                );
            }
            let mut view = glory::ui::view()
                .class("sapid-toolbar-group")
                .role(Role::Group)
                .style(
                    Style::new()
                        .display(Display::Flex)
                        .flex_direction(axis)
                        .gap(4.)
                        .min_width(Length::Px(0.)),
                )
                .semantics(Some(Box::new(Semantics {
                    id: Some(group_id),
                    ..Default::default()
                })))
                .fill(buttons);
            if let Some(name) = group.name.take() {
                view = view.name(composite::validated(scope, &name, |value| label(value))?)
            }
            groups.push(view);
        }
        let theme = &self.common.theme;
        let mut base = Style::new()
            .display(Display::Flex)
            .flex_direction(axis)
            .flex_wrap(if self.orientation == Orientation::Horizontal {
                FlexWrap::Wrap
            } else {
                FlexWrap::NoWrap
            })
            .gap(12.)
            .padding(Edges::all(6.))
            .min_width(Length::Px(0.))
            .border_width(Edges::all(1.))
            .border_style(BorderStyle::Solid)
            .border_color(theme.border)
            .radius(theme.radius);
        base.background = Some(theme.surface);
        let visible = self.visible.clone();
        let orientation = self.orientation;
        let direction = self.direction;
        let mut root = configure!(self.common, glory::ui::view(), scope, "sapid-toolbar", base)
            .role(Role::Toolbar)
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    id: Some(id.clone()),
                    hidden: !visible.get(),
                    navigation: Some(orientation),
                    navigation_direction: direction,
                    ..Default::default()
                }))
            }))
            .fill(groups);
        let mut observers = self
            .common
            .handlers
            .remove(&EventKind::Navigate)
            .unwrap_or_default();
        let navigation = roving.clone();
        let weak = scope.downgrade();
        let owner = scope.id();
        root = root.on(EventKind::Navigate, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                if let Event::Navigate { from, command } = event
                    && let Some(next) = navigation.destination(from, command)?
                {
                    composite::focus_node(&scope, Some(next.node))?;
                    navigation.accept(&next)?
                }
                observe(&mut observers, owner, event);
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error)
            }
        });
        for (kind, mut callbacks) in std::mem::take(&mut self.common.handlers) {
            root = root.on(kind, move |event| observe(&mut callbacks, owner, event));
        }
        root.show_in(scope)?;
        roving.repair(None)?;
        let weak = scope.downgrade();
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            if let Err(error) = roving.repair(None) {
                scope.report_error(error)
            }
        });
        Ok(())
    }
}

pub struct ActionBar {
    toolbar: Toolbar,
    count: Vary<usize>,
    visible: Option<Vary<bool>>,
    clear: Option<Box<dyn FnMut()>>,
    clear_label: Vary<String>,
}
pub fn action_bar(key: impl Into<String>) -> ActionBar {
    ActionBar {
        toolbar: toolbar(key).name("Selected actions"),
        count: 0usize.into(),
        visible: None,
        clear: None,
        clear_label: "Clear selection".into(),
    }
}
impl ActionBar {
    pub fn group(mut self, value: ToolbarGroup) -> Self {
        self.toolbar = self.toolbar.group(value);
        self
    }
    pub fn selected_count(mut self, value: impl Into<Vary<usize>>) -> Self {
        self.count = value.into();
        self
    }
    pub fn visible(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.visible = Some(value.into());
        self
    }
    pub fn on_clear_selection(mut self, value: impl FnMut() + 'static) -> Self {
        self.clear = Some(Box::new(value));
        self
    }
    pub fn clear_label(mut self, value: impl Into<Vary<String>>) -> Self {
        self.clear_label = value.into();
        self
    }
    pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.toolbar = self.toolbar.name(value);
        self
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.toolbar = self.toolbar.theme(value);
        self
    }
    pub fn size(mut self, value: crate::Size) -> Self {
        self.toolbar = self.toolbar.size(value);
        self
    }
    pub fn class(mut self, value: impl Into<Vary<String>>) -> Self {
        self.toolbar = self.toolbar.class(value);
        self
    }
    pub fn style(mut self, value: impl Into<Vary<Style>>) -> Self {
        self.toolbar = self.toolbar.style(value);
        self
    }
    pub fn direction(mut self, value: Direction) -> Self {
        self.toolbar = self.toolbar.direction(value);
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.toolbar = self.toolbar.unstyled();
        self
    }
}
impl Widget for ActionBar {
    fn build(&mut self, scope: &Scope) -> Result {
        let count = self.count.clone();
        let enabled = self.toolbar.common.enabled.clone();
        self.toolbar.common.enabled = scope.bond(move || count.get() > 0 && enabled.get()).into();
        let count = self.count.clone();
        let visible = self
            .visible
            .take()
            .unwrap_or_else(|| scope.bond(move || count.get() > 0).into());
        self.toolbar.visible = visible.clone();
        if let Some(mut callback) = self.clear.take() {
            let label = composite::validated(scope, &self.clear_label, |value| label(value))?;
            let count = self.count.clone();
            self.toolbar =
                self.toolbar
                    .take_for_build()
                    .group(toolbar_group("clear-selection").button(
                        crate::button().label(label).on_press(move || {
                            if count.get() > 0 {
                                callback()
                            }
                        }),
                    ));
        }
        let count = self.count.clone();
        let bar = self.toolbar.take_for_build();
        glory::ui::view()
            .class("sapid-action-bar")
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    hidden: !visible.get(),
                    ..Default::default()
                }))
            }))
            .fill((
                glory::ui::text()
                    .class("sapid-action-bar-count")
                    .content(scope.bond(move || format!("{} selected", count.get()))),
                bar,
            ))
            .show_in(scope)
    }
}
impl Toolbar {
    fn take_for_build(&mut self) -> Self {
        std::mem::replace(self, toolbar(String::new()))
    }
}
