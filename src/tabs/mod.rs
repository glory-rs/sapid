use glory::render::{Direction, Event, EventKind, FocusMode, Orientation, Role, Semantics};
use glory::ui::{Result, Widget};
use glory::{Scope, Vary};
use std::cell::Cell;
use std::rc::Rc;

use crate::appearance::{Common, common_methods, configure, observe};
use crate::composite::{
    self, Callback, ContentPolicy, Item, Members, Observers, Panel, item_methods,
};
use crate::state::{Control, Dirty, Value};

mod dynamic;
mod focus;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabActivation {
    #[default]
    Automatic,
    Manual,
}

pub struct Tab {
    inner: Item,
}
pub fn tab(key: impl Into<String>) -> Tab {
    Tab {
        inner: Item::new(key.into()),
    }
}
impl Tab {
    item_methods!();
    /// Rich content inside the semantic Tab; label remains its name.
    pub fn trigger_content<W: Widget>(
        mut self,
        mut factory: impl FnMut() -> W + 'static,
    ) -> Self {
        self.inner.trigger_content = Some(Box::new(move || Box::new(factory())));
        self
    }
}

pub struct Tabs {
    common: Common,
    key: String,
    items: Vec<Item>,
    dynamic: Option<composite::dynamic::Items>,
    value: Value<String>,
    on_change: Option<Box<dyn FnMut(String)>>,
    orientation: Orientation,
    direction: Direction,
    activation: TabActivation,
    looping: bool,
    policy: ContentPolicy,
}
pub fn tabs(key: impl Into<String>) -> Tabs {
    Tabs {
        common: Common::default(),
        key: key.into(),
        items: Vec::new(),
        dynamic: None,
        value: Value::default(),
        on_change: None,
        orientation: Orientation::Horizontal,
        direction: Direction::Ltr,
        activation: TabActivation::Automatic,
        looping: true,
        policy: ContentPolicy::KeepMounted,
    }
}
impl Tabs {
    common_methods!();
    pub fn tab(mut self, tab: Tab) -> Self {
        if self.items.len() <= 256 {
            self.items.push(tab.inner);
        }
        self
    }
    /// Reconcile at most 256 current stable keys. Factories run once for each
    /// newly admitted key under its short Owner; retained keys keep state.
    pub fn tabs(
        mut self,
        keys: impl Into<Vary<Vec<String>>>,
        mut factory: impl FnMut(&str) -> Tab + 'static,
    ) -> Self {
        self.dynamic = Some(composite::dynamic::Items {
            source: keys.into(),
            factory: Box::new(move |key| factory(key).inner),
        });
        self
    }
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
    pub fn value(mut self, value: impl Into<Vary<String>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.value.default = value.into();
        self
    }
    pub fn on_value_change(mut self, callback: impl FnMut(String) + 'static) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }
    pub fn orientation(mut self, axis: Orientation) -> Self {
        self.orientation = axis;
        self
    }
    pub fn activation(mut self, activation: TabActivation) -> Self {
        self.activation = activation;
        self
    }
    pub fn loop_focus(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }
    pub fn content_policy(mut self, policy: ContentPolicy) -> Self {
        self.policy = policy;
        self
    }
}

struct Selection {
    control: Control<String>,
    keys: Rc<Vec<String>>,
    callback: Callback<String>,
    pending: Cell<Option<usize>>,
    dirty: Dirty,
    owner: glory::core::InstanceId,
}
impl Selection {
    fn propose(&self, index: usize) -> Result {
        let old = self.control.source.try_get()?;
        if !self.keys.contains(&old) {
            return Err(glory::render::Error::InvalidProps("unknown tab value").into());
        }
        let next = &self.keys[index];
        if &old == next || self.pending.get() == Some(index) {
            return Ok(());
        }
        self.pending.set(Some(index));
        self.dirty.touch()?;
        self.control.accept(next)?;
        self.callback.emit(self.owner, next.clone());
        Ok(())
    }
}

impl Widget for Tabs {
    fn build(&mut self, scope: &Scope) -> Result {
        if let Some(input) = self.dynamic.take() {
            return dynamic::build(self, scope, input);
        }
        if self.items.len() > 256 {
            return Err(
                glory::render::Error::InvalidProps("tabs exceed the 256-item budget").into(),
            );
        }
        let root_id = composite::id("t", &self.key, "root", "root")?;
        let keys: Rc<Vec<_>> =
            Rc::new(self.items.iter().map(|item| item.key.clone()).collect());
        composite::validate_keys(&keys)?;
        if keys.is_empty() {
            return Err(
                glory::render::Error::InvalidProps("tabs require at least one item").into(),
            );
        }
        let members = Members::new(scope, &self.items, self.common.enabled.clone());
        let panel_references = Rc::new(
            self.items
                .iter()
                .map(|item| item.panel.unwrap_or_else(|| scope.cage(None)))
                .collect::<Vec<_>>(),
        );
        if self.value.source.is_none() && self.value.default.is_empty() {
            self.value.default = keys[members
                .flags()?
                .iter()
                .position(|value| *value)
                .unwrap_or(0)]
            .clone();
        }
        let control = std::mem::take(&mut self.value).mount(scope);
        let known = keys.clone();
        let selected = composite::validated(scope, &control.source, move |value| {
            if known.contains(value) {
                Ok(())
            } else {
                Err(glory::render::Error::InvalidProps("unknown tab value").into())
            }
        })?;
        let focused = scope.cage(0_usize);
        let within = scope.cage(false);
        let live = members.clone();
        let values = keys.clone();
        let roving = scope.bond(move || {
            let flags = live.flags().expect("live tab inputs");
            let chosen = selected
                .with(|value| values.iter().position(|key| key == value))
                .expect("validated tab");
            let current = if within.get() { focused.get() } else { chosen };
            if flags.get(current) == Some(&true) {
                current
            } else {
                flags.iter().position(|value| *value).unwrap_or(usize::MAX)
            }
        });
        let selection = Rc::new(Selection {
            control,
            keys: keys.clone(),
            callback: Callback::new(self.on_change.take()),
            pending: Cell::new(None),
            dirty: Dirty::new(scope),
            owner: scope.id(),
        });
        let reset = selection.clone();
        scope.effect(move || {
            if reset.dirty.take() {
                reset.pending.set(None);
            }
        });
        let restore_members = members.clone();
        let restore_panels = panel_references.clone();
        let restore_keys = keys.clone();
        let restore = scope.bond(move || {
            let chosen = selected
                .with(|value| restore_keys.iter().position(|key| key == value))
                .expect("validated tab");
            if restore_members.flags().expect("live tab inputs")[chosen] {
                restore_members.references[chosen].get()
            } else {
                // Focusing another enabled tab would auto-activate it, undoing
                // an explicit controlled selection of a disabled tab.
                restore_panels[chosen].get()
            }
        });
        let mut triggers: Vec<Box<dyn Widget>> = Vec::new();
        let mut panels: Vec<Box<dyn Widget>> = Vec::new();
        let owner = scope.id();
        let presses = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Activate)
                .unwrap_or_default(),
        );
        for (index, mut item) in self.items.drain(..).enumerate() {
            let trigger_id = composite::id("t", &self.key, "trigger", &item.key)?;
            let panel_id = composite::id("t", &self.key, "panel", &item.key)?;
            let key = item.key.clone();
            let open = scope.bond(move || selected.with(|value| value == &key));
            let id = trigger_id.clone();
            let target = panel_id.clone();
            let enabled = item.enabled;
            let root_enabled = self.common.enabled.clone();
            let mut trigger = glory::ui::button()
                .class("sapid-tab")
                .label(item.label.clone())
                .node_ref(members.references[index])
                .enabled(scope.bond(move || root_enabled.get() && enabled.get()))
                .focus(scope.bond(move || {
                    if roving.get() == index {
                        FocusMode::TabStop
                    } else {
                        FocusMode::Programmatic
                    }
                }))
                .semantics(scope.bond(move || {
                    composite::metadata(Semantics {
                        id: Some(id.clone()),
                        controls: Some(target.clone()),
                        tab: true,
                        selected: Some(open.get()),
                        navigation_item: true,
                        ..Default::default()
                    })
                }));
            if let Some(mut factory) = item.trigger_content.take() {
                trigger = trigger
                    .label("")
                    .name(item.label)
                    .fill(glory::ui::widget(move |scope| factory().show_in(scope)));
            }
            if !self.common.unstyled {
                let theme = self.common.theme.clone();
                let size = self.common.size;
                trigger = trigger.style(scope.bond(move || {
                    let mut style = theme.control(size);
                    if open.get() {
                        style.background = Some(theme.accent);
                        style.color = Some(theme.on_accent);
                    }
                    style
                }));
            }
            let automatic = self.activation == TabActivation::Automatic;
            let on_focus = selection.clone();
            let live = members.clone();
            let weak = scope.downgrade();
            trigger = trigger.on(EventKind::FocusChanged, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                if event == Event::FocusChanged(true) {
                    focused.revise(|value| *value = index);
                    if automatic {
                        let result = live.flags().and_then(|flags| {
                            if flags[index] {
                                on_focus.propose(index)
                            } else {
                                Ok(())
                            }
                        });
                        if let Err(error) = result {
                            scope.report_error(error);
                        }
                    }
                }
            });
            let on_press = selection.clone();
            let live = members.clone();
            let weak = scope.downgrade();
            let presses = presses.clone();
            trigger = trigger.on(EventKind::Activate, move |event| {
                if let Some(scope) = weak.upgrade() {
                    let result = live.flags().and_then(|flags| {
                        if flags[index] {
                            on_press.propose(index)
                        } else {
                            Ok(())
                        }
                    });
                    if let Err(error) = result {
                        scope.report_error(error);
                    }
                    presses.emit(owner, event);
                }
            });
            triggers.push(Box::new(trigger));
            panels.push(Box::new(Panel {
                id: panel_id,
                label: trigger_id,
                role: Some(Role::TabPanel),
                open: open.into(),
                trigger: members.references[index],
                restore: Some(restore.into()),
                reference: Some(panel_references[index]),
                factory: item.factory.take(),
                policy: self.policy,
                class: "sapid-tab-panel",
                unstyled: self.common.unstyled,
            }));
        }
        let mut list = glory::ui::view()
            .class("sapid-tab-list")
            .role(Role::TabList)
            .semantics(composite::metadata(Semantics {
                navigation: Some(self.orientation),
                navigation_direction: self.direction,
                ..Default::default()
            }))
            .style(
                glory::style::Style::new()
                    .display(glory::style::Display::Flex)
                    .flex_direction(if self.orientation == Orientation::Horizontal {
                        glory::style::FlexDirection::Row
                    } else {
                        glory::style::FlexDirection::Column
                    })
                    .gap(4.0),
            )
            .on(EventKind::FocusWithin, move |event| {
                if let Event::FocusWithin { focused, .. } = event {
                    within.revise(|value| *value = focused);
                }
            })
            .fill(triggers);
        if let Some(name) = self.common.name.take() {
            list = list.name(name);
        }
        let live = members.clone();
        let looping = self.looping;
        let weak = scope.downgrade();
        let mut observers = self
            .common
            .handlers
            .remove(&EventKind::Navigate)
            .unwrap_or_default();
        list = list.on(EventKind::Navigate, move |event| {
            if let Some(scope) = weak.upgrade() {
                if let Event::Navigate { from, command } = event
                    && let Err(error) = live.navigate(&scope, from, command, looping)
                {
                    scope.report_error(error);
                }
                observe(&mut observers, owner, event);
            }
        });
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-tabs",
            composite::column(8.0)
        )
        .semantics(composite::metadata(Semantics {
            id: Some(root_id),
            ..Default::default()
        }))
        .fill(list)
        .fill(panels);
        for (kind, mut observers) in std::mem::take(&mut self.common.handlers) {
            root = root.on(kind, move |event| observe(&mut observers, owner, event));
        }
        root.show_in(scope)?;
        members.register()
    }
}
