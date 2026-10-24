use glory::render::{Direction, Event, EventKind, Orientation, Role, Semantics};
use glory::ui::{Result, Widget};
use glory::{Lotus, Scope};
use std::collections::HashSet;
use std::rc::Rc;

use crate::appearance::{Common, common_methods, configure, observe};
use crate::composite::{
    self, Callback, ContentPolicy, Item, Members, Observers, Panel, item_methods,
};
use crate::state::Value;

#[path = "accordion_dynamic.rs"]
mod dynamic;

pub struct AccordionItem {
    inner: Item,
}
pub fn accordion_item(key: impl Into<String>) -> AccordionItem {
    AccordionItem {
        inner: Item::new(key.into()),
    }
}
impl AccordionItem {
    item_methods!();
    /// Rich content inside the semantic Button; label remains its name.
    pub fn trigger_content<W: Widget>(
        mut self,
        mut factory: impl FnMut() -> W + 'static,
    ) -> Self {
        self.inner.trigger_content = Some(Box::new(move || Box::new(factory())));
        self
    }
}

pub struct Accordion {
    common: Common,
    key: String,
    items: Vec<Item>,
    dynamic: Option<composite::dynamic::Items>,
    open: Value<Vec<String>>,
    on_change: Option<Box<dyn FnMut(Vec<String>)>>,
    multiple: bool,
    collapsible: bool,
    looping: bool,
    heading: u8,
    regions: bool,
    policy: ContentPolicy,
    direction: Direction,
}
pub fn accordion(key: impl Into<String>) -> Accordion {
    Accordion {
        common: Common::default(),
        key: key.into(),
        items: Vec::new(),
        dynamic: None,
        open: Value::default(),
        on_change: None,
        multiple: false,
        collapsible: true,
        looping: true,
        heading: 3,
        regions: false,
        policy: ContentPolicy::Unmount,
        direction: Direction::Ltr,
    }
}
impl Accordion {
    common_methods!();
    pub fn item(mut self, item: AccordionItem) -> Self {
        if self.items.len() <= 256 {
            self.items.push(item.inner);
        }
        self
    }
    /// A bounded keyed sequence; factories run once per newly mounted key.
    pub fn items(
        mut self,
        keys: impl Into<Lotus<Vec<String>>>,
        mut factory: impl FnMut(&str) -> AccordionItem + 'static,
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
    pub fn open(mut self, value: impl Into<Lotus<Vec<String>>>) -> Self {
        self.open.source = Some(value.into());
        self
    }
    pub fn default_open(mut self, keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.open.default = keys.into_iter().take(257).map(Into::into).collect();
        self
    }
    pub fn on_open_change(mut self, callback: impl FnMut(Vec<String>) + 'static) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }
    pub fn multiple(mut self, value: bool) -> Self {
        self.multiple = value;
        self
    }
    pub fn collapsible(mut self, value: bool) -> Self {
        self.collapsible = value;
        self
    }
    pub fn loop_focus(mut self, value: bool) -> Self {
        self.looping = value;
        self
    }
    pub fn heading_level(mut self, level: u8) -> Self {
        self.heading = level;
        self
    }
    pub fn regions(mut self, value: bool) -> Self {
        self.regions = value;
        self
    }
    pub fn content_policy(mut self, policy: ContentPolicy) -> Self {
        self.policy = policy;
        self
    }
}

fn valid_open(
    value: &[String],
    known: &HashSet<String>,
    multiple: bool,
    collapsible: bool,
) -> Result {
    if value.len() > 256 {
        return Err(glory::render::Error::InvalidProps(
            "accordion open keys exceed the 256-item budget",
        )
        .into());
    }
    let unique: HashSet<_> = value.iter().collect();
    if unique.len() != value.len()
        || value.iter().any(|key| !known.contains(key))
        || (!multiple && value.len() > 1)
        || (!collapsible && !known.is_empty() && value.is_empty())
    {
        return Err(glory::render::Error::InvalidProps("invalid accordion open keys").into());
    }
    Ok(())
}

impl Widget for Accordion {
    fn build(&mut self, scope: &Scope) -> Result {
        if let Some(input) = self.dynamic.take() {
            return dynamic::build(self, scope, input);
        }
        if self.items.len() > 256 {
            return Err(glory::render::Error::InvalidProps(
                "accordion exceeds the 256-item budget",
            )
            .into());
        }
        let root_id = composite::id("a", &self.key, "root", "root")?;
        let keys: Vec<_> = self.items.iter().map(|item| item.key.clone()).collect();
        composite::validate_keys(&keys)?;
        if !(1..=6).contains(&self.heading) {
            return Err(
                glory::render::Error::InvalidProps("heading level must be 1..=6").into(),
            );
        }
        if self.open.source.is_none()
            && !self.collapsible
            && self.open.default.is_empty()
            && let Some(first) = keys.first()
        {
            self.open.default.push(first.clone());
        }
        let control = std::mem::take(&mut self.open).mount(scope);
        let known: Rc<HashSet<_>> = Rc::new(keys.iter().cloned().collect());
        let multiple = self.multiple;
        let collapsible = self.collapsible;
        let check_known = known.clone();
        let selected = composite::validated(scope, &control.source, move |value| {
            valid_open(value, &check_known, multiple, collapsible)
        })?;
        let callback = Callback::new(self.on_change.take());
        let members = Members::new(scope, &self.items, self.common.enabled.clone());
        let mut children: Vec<Box<dyn Widget>> = Vec::new();
        let owner = scope.id();
        let presses = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Activate)
                .unwrap_or_default(),
        );
        for (index, mut item) in self.items.drain(..).enumerate() {
            let trigger_id = composite::id("a", &self.key, "trigger", &item.key)?;
            let panel_id = composite::id("a", &self.key, "panel", &item.key)?;
            let key = item.key.clone();
            let open = scope.bond(move || selected.with(|value| value.contains(&key)));
            let id = trigger_id.clone();
            let target = panel_id.clone();
            let enabled = item.enabled.clone();
            let root_enabled = self.common.enabled.clone();
            let current = item.key.clone();
            let mut trigger = glory::ui::button()
                .class("sapid-accordion-trigger")
                .label(item.label.clone())
                .node_ref(members.references[index])
                .enabled(scope.bond(move || root_enabled.get() && enabled.get()))
                .semantics(scope.bond(move || {
                    composite::metadata(Semantics {
                        id: Some(id.clone()),
                        controls: Some(target.clone()),
                        expanded: Some(open.get()),
                        navigation_item: true,
                        disabled: !collapsible
                            && selected
                                .with(|value| value.len() == 1 && value.contains(&current)),
                        ..Default::default()
                    })
                }));
            if let Some(mut factory) = item.trigger_content.take() {
                trigger = trigger
                    .label("")
                    .name(item.label)
                    .fill(glory::ui::component(move |scope| factory().show_in(scope)));
            }
            if !self.common.unstyled {
                trigger = trigger.style(self.common.theme.control(self.common.size));
            }
            let state = control.clone();
            let callback = callback.clone();
            let known = known.clone();
            let key = item.key;
            let weak = scope.downgrade();
            let presses = presses.clone();
            trigger = trigger.on(EventKind::Activate, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    let mut next = state.source.try_get()?;
                    valid_open(&next, &known, multiple, collapsible)?;
                    if let Some(at) = next.iter().position(|value| value == &key) {
                        if !collapsible && next.len() == 1 {
                            return Ok(());
                        }
                        next.remove(at);
                    } else {
                        if !multiple {
                            next.clear();
                        }
                        next.push(key.clone());
                    }
                    state.accept(&next)?;
                    callback.emit(owner, next);
                    presses.emit(owner, event);
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                }
            });
            let header = glory::ui::view()
                .role(Role::Heading(self.heading))
                .fill(trigger);
            let panel = Panel {
                id: panel_id,
                label: trigger_id,
                role: self.regions.then_some(Role::Section),
                open: open.into(),
                trigger: members.references[index],
                restore: None,
                reference: item.panel,
                factory: item.factory.take(),
                policy: self.policy,
                class: "sapid-accordion-panel",
                unstyled: self.common.unstyled,
            };
            children.push(Box::new(
                glory::ui::view()
                    .class("sapid-accordion-item")
                    .fill((header, panel)),
            ));
        }
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-accordion",
            composite::column(8.0)
        )
        .semantics(composite::metadata(Semantics {
            id: Some(root_id),
            navigation: Some(Orientation::Vertical),
            navigation_direction: self.direction,
            ..Default::default()
        }))
        .fill(children);
        let mut observers = self
            .common
            .handlers
            .remove(&EventKind::Navigate)
            .unwrap_or_default();
        let live = members.clone();
        let looping = self.looping;
        let weak = scope.downgrade();
        root = root.on(EventKind::Navigate, move |event| {
            if let Some(scope) = weak.upgrade() {
                if let Event::Navigate { from, command } = event
                    && let Err(error) = live.navigate(&scope, from, command, looping)
                {
                    scope.report_error(error);
                }
                observe(&mut observers, owner, event);
            }
        });
        for (kind, mut observers) in std::mem::take(&mut self.common.handlers) {
            root = root.on(kind, move |event| observe(&mut observers, owner, event));
        }
        root.show_in(scope)?;
        members.register()
    }
}
