//! Pressed buttons and finite single/multiple value groups.
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use glory::render::{Event, EventKind, FocusMode, NodeId, Orientation, Role, Semantics};
use glory::style::{Display, FlexDirection, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};

use crate::appearance::{Common, common_methods, configure, merge, observe};
use crate::composite::{self, Callback, Observers};
use crate::state::{Control, Value};

pub const MAX_TOGGLE_GROUP_ITEMS: usize = 1024;

pub struct Toggle {
    common: Common,
    value: Value<bool>,
    label: Lotus<String>,
    on_change: Option<Box<dyn FnMut(bool)>>,
}
pub fn toggle() -> Toggle {
    Toggle {
        common: Common::default(),
        value: Value::default(),
        label: "".into(),
        on_change: None,
    }
}
impl Toggle {
    common_methods!();
    pub fn label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.label = value.into();
        self
    }
    pub fn pressed(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_pressed(mut self, value: bool) -> Self {
        self.value.default = value;
        self
    }
    pub fn on_pressed_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }
}
fn appearance(common: &Common, scope: &Scope, pressed: Lotus<bool>) -> Lotus<Style> {
    let theme = common.theme.clone();
    let size = common.size;
    let unstyled = common.unstyled;
    let overrides = common.style.clone();
    scope
        .bond(move || {
            let mut style = if unstyled {
                Style::new()
            } else {
                theme.control(size)
            };
            if !unstyled && pressed.get() {
                style.background = Some(theme.accent);
                style.color = Some(theme.on_accent);
                style.border_color = Some(theme.accent);
            }
            merge(style, overrides.get())
        })
        .into()
}
impl Widget for Toggle {
    fn build(&mut self, scope: &Scope) -> Result {
        let control = std::mem::take(&mut self.value).mount(scope);
        control.source.try_get()?;
        let pressed = control.source.clone();
        let style = appearance(&self.common, scope, pressed.clone());
        let enabled = self.common.enabled.clone();
        let mut button = configure!(
            self.common,
            glory::ui::button(),
            scope,
            "sapid-button sapid-toggle",
            Style::new()
        )
        .label(self.label.clone())
        .enabled(enabled.clone())
        .style(style)
        .semantics(scope.bond(move || {
            composite::metadata(Semantics {
                pressed: Some(pressed.get()),
                ..Default::default()
            })
        }));
        let callback = Callback::new(self.on_change.take());
        let mut observers = self
            .common
            .handlers
            .remove(&EventKind::Activate)
            .unwrap_or_default();
        let owner = scope.id();
        let weak = scope.downgrade();
        button = button.on(EventKind::Activate, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                if enabled.try_get()? {
                    let proposal = !control.source.try_get()?;
                    control.accept(&proposal)?;
                    callback.emit(owner, proposal);
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
                return;
            }
            observe(&mut observers, owner, event);
        });
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            button = button.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        button.show_in(scope)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleMode {
    #[default]
    Single,
    Multiple,
}
pub struct ToggleItem {
    key: String,
    common: Common,
    label: Lotus<String>,
}
pub fn toggle_item(key: impl Into<String>) -> ToggleItem {
    let key = key.into();
    ToggleItem {
        label: key.clone().into(),
        key,
        common: Common::default(),
    }
}
impl ToggleItem {
    pub fn label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.label = value.into();
        self
    }
    pub fn name(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn enabled(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.common.enabled = value.into();
        self
    }
    pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
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
pub struct ToggleGroup {
    key: String,
    common: Common,
    mode: ToggleMode,
    value: Value<Vec<String>>,
    items: Vec<ToggleItem>,
    orientation: Orientation,
    looping: bool,
    on_change: Option<Box<dyn FnMut(Vec<String>)>>,
}
pub fn toggle_group(key: impl Into<String>) -> ToggleGroup {
    ToggleGroup {
        key: key.into(),
        common: Common::default(),
        mode: ToggleMode::Single,
        value: Value::default(),
        items: Vec::new(),
        orientation: Orientation::Horizontal,
        looping: true,
        on_change: None,
    }
}
impl ToggleGroup {
    common_methods!();
    pub fn mode(mut self, value: ToggleMode) -> Self {
        self.mode = value;
        self
    }
    pub fn value(mut self, value: impl Into<Lotus<Vec<String>>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.default = value
            .into_iter()
            .take(MAX_TOGGLE_GROUP_ITEMS + 1)
            .map(Into::into)
            .collect();
        self
    }
    pub fn item(mut self, value: ToggleItem) -> Self {
        if self.items.len() <= MAX_TOGGLE_GROUP_ITEMS {
            self.items.push(value);
        }
        self
    }
    pub fn items(mut self, values: impl IntoIterator<Item = ToggleItem>) -> Self {
        crate::document::extend_bounded(&mut self.items, values, MAX_TOGGLE_GROUP_ITEMS);
        self
    }
    pub fn orientation(mut self, value: Orientation) -> Self {
        self.orientation = value;
        self
    }
    pub fn loop_focus(mut self, value: bool) -> Self {
        self.looping = value;
        self
    }
    pub fn on_value_change(mut self, callback: impl FnMut(Vec<String>) + 'static) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }
}
struct GroupControl {
    control: Control<Vec<String>>,
    keys: Rc<Vec<String>>,
    mode: ToggleMode,
    flags: Vec<Lotus<bool>>,
    root_enabled: Lotus<bool>,
    references: Rc<Vec<Cage<Option<NodeId>>>>,
    callback: Callback<Vec<String>>,
    owner: glory::core::InstanceId,
}
fn validate_selection(keys: &[String], mode: ToggleMode, value: &[String]) -> Result {
    if value.len() > keys.len() || mode == ToggleMode::Single && value.len() > 1 {
        return Err(
            glory::render::Error::InvalidProps("invalid toggle group selection size").into(),
        );
    }
    let mut unique = HashSet::new();
    if value
        .iter()
        .any(|key| !keys.contains(key) || !unique.insert(key))
    {
        return Err(glory::render::Error::InvalidProps(
            "toggle selection contains unknown or duplicate keys",
        )
        .into());
    }
    Ok(())
}
impl GroupControl {
    fn flags(&self) -> Result<Vec<bool>> {
        let root = self.root_enabled.try_get()?;
        self.flags
            .iter()
            .map(|flag| Ok(root && flag.try_get()?))
            .collect()
    }
    fn activate(&self, index: usize) -> Result {
        if !self.flags()?[index] {
            return Ok(());
        }
        let mut value = self.control.source.try_with(|value| {
            validate_selection(&self.keys, self.mode, value)?;
            Ok::<_, glory::ui::Error>(value.clone())
        })??;
        let key = &self.keys[index];
        if let Some(selected) = value.iter().position(|value| value == key) {
            value.remove(selected);
        } else {
            if self.mode == ToggleMode::Single {
                value.clear();
            }
            value.push(key.clone());
        }
        self.control.accept(&value)?;
        self.callback.emit(self.owner, value);
        Ok(())
    }
}
impl Widget for ToggleGroup {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.items.is_empty()
            || self.items.len() > MAX_TOGGLE_GROUP_ITEMS
            || self.value.default.len() > MAX_TOGGLE_GROUP_ITEMS
            || self.common.focus != FocusMode::Auto
        {
            return Err(glory::render::Error::InvalidProps(
                "toggle groups require 1..=1024 items and own their tab stops",
            )
            .into());
        }
        let id = composite::id("toggle-group", &self.key, "root", "root")?;
        let keys = Rc::new(
            self.items
                .iter()
                .map(|item| item.key.clone())
                .collect::<Vec<_>>(),
        );
        composite::validate_keys(&keys)?;
        let control = std::mem::take(&mut self.value).mount(scope);
        let valid_keys = keys.clone();
        let mode = self.mode;
        let selected = composite::validated(scope, &control.source, move |value| {
            validate_selection(&valid_keys, mode, value)
        })?;
        let references = Rc::new(
            self.items
                .iter()
                .map(|item| item.common.node_ref.unwrap_or_else(|| scope.cage(None)))
                .collect::<Vec<_>>(),
        );
        let flags = self
            .items
            .iter()
            .map(|item| item.common.enabled.clone())
            .collect::<Vec<_>>();
        let controller = Rc::new(GroupControl {
            control,
            keys: keys.clone(),
            mode,
            flags: flags.clone(),
            root_enabled: self.common.enabled.clone(),
            references: references.clone(),
            callback: Callback::new(self.on_change.take()),
            owner: scope.id(),
        });
        let focused = scope.cage(usize::MAX);
        let within = scope.cage(false);
        let roving_controller = controller.clone();
        let roving = scope.bond(move || {
            let flags = roving_controller.flags().expect("live toggle flags");
            let focused = focused.get();
            if within.get() && flags.get(focused) == Some(&true) {
                return focused;
            }
            let current = selected.get();
            roving_controller
                .keys
                .iter()
                .enumerate()
                .find_map(|(index, key)| {
                    (flags[index] && current.contains(key)).then_some(index)
                })
                .or_else(|| flags.iter().position(|enabled| *enabled))
                .unwrap_or(usize::MAX)
        });
        let owner = scope.id();
        let group_presses = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Activate)
                .unwrap_or_default(),
        );
        let mut children: Vec<Box<dyn Widget>> = Vec::new();
        for (index, mut item) in self.items.drain(..).enumerate() {
            item.common.theme = self.common.theme.clone();
            item.common.size = self.common.size;
            item.common.unstyled |= self.common.unstyled;
            let key = item.key.clone();
            let pressed = scope.bond(move || selected.with(|values| values.contains(&key)));
            let style = appearance(&item.common, scope, pressed.into());
            let enabled = item.common.enabled.clone();
            let root_enabled = self.common.enabled.clone();
            let mut button = configure!(
                item.common,
                glory::ui::button(),
                scope,
                "sapid-button sapid-toggle sapid-toggle-item",
                Style::new()
            )
            .label(item.label)
            .style(style)
            .enabled(scope.bond(move || root_enabled.get() && enabled.get()))
            .node_ref(references[index])
            .focus(scope.bond(move || {
                if roving.get() == index {
                    FocusMode::TabStop
                } else {
                    FocusMode::Programmatic
                }
            }))
            .semantics(scope.bond(move || {
                composite::metadata(Semantics {
                    pressed: Some(pressed.get()),
                    navigation_item: true,
                    ..Default::default()
                })
            }));
            let mut activate_observers = item
                .common
                .handlers
                .remove(&EventKind::Activate)
                .unwrap_or_default();
            let live = controller.clone();
            let weak = scope.downgrade();
            let group_presses = group_presses.clone();
            button = button.on(EventKind::Activate, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                if let Err(error) = live.activate(index) {
                    scope.report_error(error);
                    return;
                }
                observe(&mut activate_observers, owner, event.clone());
                group_presses.emit(owner, event);
            });
            let mut focus_observers = item
                .common
                .handlers
                .remove(&EventKind::FocusChanged)
                .unwrap_or_default();
            button = button.on(EventKind::FocusChanged, move |event| {
                if event == Event::FocusChanged(true) {
                    focused.revise(|current| *current = index);
                }
                observe(&mut focus_observers, owner, event);
            });
            for (kind, mut handlers) in item.common.handlers {
                button = button.on(kind, move |event| observe(&mut handlers, owner, event));
            }
            children.push(Box::new(button));
        }
        let mut root = configure!(
            self.common,
            glory::ui::view().role(Role::Group),
            scope,
            "sapid-toggle-group",
            Style::new()
                .display(Display::Flex)
                .flex_direction(if self.orientation == Orientation::Horizontal {
                    FlexDirection::Row
                } else {
                    FlexDirection::Column
                })
                .gap(4.0)
        )
        .semantics(composite::metadata(Semantics {
            id: Some(id),
            navigation: Some(self.orientation),
            ..Default::default()
        }))
        .fill(children);
        let within_observers = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::FocusWithin)
                .unwrap_or_default(),
        );
        root = root.on(EventKind::FocusWithin, move |event| {
            if let Event::FocusWithin { focused, .. } = event {
                within.revise(|value| *value = focused);
            }
            within_observers.emit(owner, event);
        });
        let index = Rc::new(std::cell::RefCell::new(HashMap::<NodeId, usize>::new()));
        let lookup = index.clone();
        let looping = self.looping;
        let live = controller.clone();
        let weak = scope.downgrade();
        let mut navigation_observers = self
            .common
            .handlers
            .remove(&EventKind::Navigate)
            .unwrap_or_default();
        root = root.on(EventKind::Navigate, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            if let Event::Navigate { from, command } = event {
                let current = lookup.borrow().get(&from).copied();
                let result = (|| -> Result {
                    if let Some(current) = current
                        && let Some(next) =
                            composite::destination(current, command, &live.flags()?, looping)
                    {
                        // Roving focus follows an actual FocusChanged event; a queued
                        // action is never reported as actual focus.
                        composite::focus(&scope, live.references[next])?;
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                    return;
                }
            }
            observe(&mut navigation_observers, owner, event);
        });
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)?;
        let entries = references
            .iter()
            .enumerate()
            .map(|(index, reference)| {
                Ok((reference.try_get()?.ok_or(glory::ui::Error::Gone)?, index))
            })
            .collect::<Result<Vec<_>>>()?;
        index.borrow_mut().extend(entries);
        Ok(())
    }
}
