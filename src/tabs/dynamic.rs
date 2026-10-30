use super::*;
use crate::collection::{Collection, CollectionLimits, CollectionMember, RovingFocus};
use crate::composite::dynamic::{self, Items};
use glory::Cage;
use std::cell::RefCell;

struct Selection {
    control: Control<String>,
    selected: Cage<Option<String>>,
    callback: Callback<String>,
    pending: RefCell<Option<CollectionMember>>,
    dirty: Dirty,
    owner: glory::core::InstanceId,
}
impl Selection {
    fn propose(&self, key: &str, member: &CollectionMember) -> Result {
        if self
            .selected
            .try_with(|selected| selected.as_deref() == Some(key))?
            || self.pending.borrow().as_ref() == Some(member)
        {
            return Ok(());
        }
        let previous = self.pending.replace(Some(member.clone()));
        drop(previous);
        self.dirty.touch()?;
        let next = key.to_owned();
        self.control.accept(&next)?;
        self.callback.emit(self.owner, next);
        Ok(())
    }
}

fn check_value(value: &str) -> Result {
    if value.len() > 64 {
        return Err(glory::render::Error::InvalidProps("tab value exceeds 64 bytes").into());
    }
    if !value.is_empty() {
        glory::render::SemanticId::new(value)?;
    }
    Ok(())
}

pub(super) fn build(widget: &mut Tabs, scope: &Scope, input: Items) -> Result {
    if !widget.items.is_empty() {
        return Err(
            glory::render::Error::InvalidProps("dynamic tabs cannot contain static tabs").into(),
        );
    }
    let root_id = composite::id("t", &widget.key, "root", "root")?;
    let configs = dynamic::mount(scope, input)?;
    let local = widget.value.source.is_none();
    let control = std::mem::take(&mut widget.value).mount(scope);
    let source = composite::validated(scope, &control.source, |value| check_value(value))?;
    let selected = scope.cage(None::<String>);
    let repair = control.clone();
    let weak = scope.downgrade();
    scope.structural(move || {
        let Some(scope) = weak.upgrade() else { return };
        let result = (|| -> Result {
            let configs = configs.try_get()?;
            let source = source.try_get()?;
            let mut next = configs
                .iter()
                .find(|config| config.key == source)
                .map(|config| config.key.clone());
            if local && next.is_none() {
                for config in &configs {
                    if config.enabled.try_get()? {
                        next = Some(config.key.clone());
                        break;
                    }
                }
                if next.is_none() {
                    next = configs.first().map(|config| config.key.clone());
                }
            }
            if local {
                glory::core::untracked(|| repair.accept(&next.clone().unwrap_or_default()))?;
            }
            if glory::core::untracked(|| selected.try_with(|old| old != &next))? {
                selected.try_revise(|old| *old = next)?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            scope.report_error(error);
        }
    });
    let collection = Collection::new(
        scope,
        CollectionLimits {
            items: dynamic::LIMIT,
            label_bytes: 256,
        },
    )?;
    let roving = RovingFocus::new(scope, collection.clone(), widget.looping)?;
    let within = scope.cage(false);
    let live = collection.clone();
    let repair = roving.clone();
    let weak = scope.downgrade();
    scope.structural(move || {
        let Some(scope) = weak.upgrade() else { return };
        let result = (|| -> Result {
            // Track current membership, order and enabled fields, not the
            // roving intent that this reconciliation may itself repair.
            live.mounted()?;
            let preferred = if within.try_get()? {
                None
            } else {
                selected.try_get()?
            };
            glory::core::untracked(|| repair.repair(preferred.as_deref()))
        })();
        if let Err(error) = result {
            scope.report_error(error);
        }
    });
    let selection = Rc::new(Selection {
        control,
        selected,
        callback: Callback::new(widget.on_change.take()),
        pending: RefCell::new(None),
        dirty: Dirty::new(scope),
        owner: scope.id(),
    });
    let reset = selection.clone();
    scope.effect(move || {
        if reset.dirty.take() {
            let previous = reset.pending.take();
            drop(previous);
        }
    });
    let restore_root_enabled = widget.common.enabled.clone();
    let restore = scope.bond(move || {
        let selected = selected.get();
        configs.with(|configs| {
            configs
                .iter()
                .find(|config| Some(&config.key) == selected.as_ref())
                .and_then(|config| {
                    if restore_root_enabled.get() && config.enabled.get() {
                        config.trigger.get()
                    } else {
                        config.panel.get()
                    }
                })
        })
    });
    let focus = super::focus::FocusRestore::new(
        scope,
        configs,
        selected.into(),
        widget.common.enabled.clone(),
    );
    let presses = Observers::new(
        widget
            .common
            .handlers
            .remove(&EventKind::Activate)
            .unwrap_or_default(),
    );
    let root_enabled = widget.common.enabled.clone();
    let theme = widget.common.theme.clone();
    let size = widget.common.size;
    let unstyled = widget.common.unstyled;
    let namespace = widget.key.clone();
    let automatic = widget.activation == TabActivation::Automatic;
    let owner = scope.id();
    let registrations = collection.clone();
    let navigation = roving.clone();
    let triggers = glory::ui::each(configs)
        .key(|config| config.key.clone())
        .children(move |entry| {
            let row = entry.value;
            let namespace = namespace.clone();
            let root_enabled = root_enabled.clone();
            let theme = theme.clone();
            let collection = registrations.clone();
            let roving = navigation.clone();
            let selection = selection.clone();
            let presses = presses.clone();
            glory::ui::widget(move |scope| {
                let config = row.try_get()?;
                let trigger_id = composite::id("t", &namespace, "trigger", &config.key)?;
                let panel_id = composite::id("t", &namespace, "panel", &config.key)?;
                let root_enabled = root_enabled.clone();
                let item_enabled = config.enabled.clone();
                let enabled = scope.bond(move || root_enabled.get() && item_enabled.get());
                let member = collection.register_ordered(
                    scope,
                    config.key.clone(),
                    config.trigger,
                    enabled,
                    config.label,
                    config.position,
                )?;
                let key = config.key.clone();
                let open =
                    scope.bond(move || selected.with(|selected| selected.as_ref() == Some(&key)));
                let id = trigger_id.clone();
                let target = panel_id.clone();
                let tab_roving = roving.clone();
                let tab_member = member.clone();
                let mut trigger = glory::ui::button()
                    .class("sapid-tab")
                    .label(config.label)
                    .enabled(enabled)
                    .node_ref(config.trigger)
                    .focus(scope.bond(move || {
                        if tab_roving
                            .is_tab_stop(&tab_member)
                            .expect("live tab registration")
                        {
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
                if !unstyled {
                    let theme = theme.clone();
                    trigger = trigger.style(scope.bond(move || {
                        let mut style = theme.control(size);
                        if open.get() {
                            style.background = Some(theme.accent);
                            style.color = Some(theme.on_accent);
                        }
                        style
                    }));
                }
                if let Some(mut factory) = config.take_trigger_content() {
                    trigger = trigger
                        .label("")
                        .name(config.label)
                        .fill(glory::ui::widget(move |scope| factory().show_in(scope)));
                }
                let focused_key = config.key.clone();
                let focused_member = member.clone();
                let focused_selection = selection.clone();
                let focused_roving = roving.clone();
                let reference = config.trigger;
                let weak = scope.downgrade();
                trigger = trigger.on(EventKind::FocusChanged, move |event| {
                    let Some(scope) = weak.upgrade() else { return };
                    let result = (|| -> Result {
                        if let Event::FocusChanged(focused) = event {
                            if focused {
                                let restoring = reference
                                    .try_get()?
                                    .map(|node| focus.consume(node))
                                    .transpose()?
                                    .unwrap_or(false);
                                focused_roving.observe(&focused_member)?;
                                if automatic && !restoring && enabled.try_get()? {
                                    focused_selection.propose(&focused_key, &focused_member)?;
                                }
                            }
                            focus.observe(&focused_key, focused)?;
                        }
                        Ok(())
                    })();
                    if let Err(error) = result {
                        scope.report_error(error);
                    }
                });
                let key = config.key.clone();
                let member = member.clone();
                let selection = selection.clone();
                let presses = presses.clone();
                let weak = scope.downgrade();
                trigger = trigger.on(EventKind::Activate, move |event| {
                    let Some(scope) = weak.upgrade() else { return };
                    let result = (|| -> Result {
                        if enabled.try_get()? {
                            selection.propose(&key, &member)?;
                        }
                        presses.emit(owner, event);
                        Ok(())
                    })();
                    if let Err(error) = result {
                        scope.report_error(error);
                    }
                });
                trigger.show_in(scope)
            })
        });
    let namespace = widget.key.clone();
    let policy = widget.policy;
    let panels = glory::ui::each(configs)
        .key(|config| config.key.clone())
        .children(move |entry| {
            let row = entry.value;
            let namespace = namespace.clone();
            glory::ui::widget(move |scope| {
                let config = row.try_get()?;
                let trigger_id = composite::id("t", &namespace, "trigger", &config.key)?;
                let panel_id = composite::id("t", &namespace, "panel", &config.key)?;
                let key = config.key.clone();
                let open =
                    scope.bond(move || selected.with(|selected| selected.as_ref() == Some(&key)));
                let panel = Panel {
                    id: panel_id,
                    label: trigger_id,
                    role: Some(Role::TabPanel),
                    open: open.into(),
                    trigger: config.trigger,
                    restore: Some(restore.into()),
                    reference: Some(config.panel),
                    factory: config.take_content(),
                    policy,
                    class: "sapid-tab-panel",
                    unstyled,
                };
                let key = config.key.clone();
                let weak = scope.downgrade();
                glory::ui::view()
                    .class("sapid-tab-panel-item")
                    .on(EventKind::FocusWithin, move |event| {
                        if let Event::FocusWithin { focused, .. } = event
                            && let Some(scope) = weak.upgrade()
                            && let Err(error) = focus.observe(&key, focused)
                        {
                            scope.report_error(error);
                        }
                    })
                    .fill(panel)
                    .show_in(scope)
            })
        });
    let mut list = glory::ui::view()
        .class("sapid-tab-list")
        .role(Role::TabList)
        .semantics(composite::metadata(Semantics {
            navigation: Some(widget.orientation),
            navigation_direction: widget.direction,
            ..Default::default()
        }))
        .style(
            glory::style::Style::new()
                .display(glory::style::Display::Flex)
                .flex_direction(if widget.orientation == Orientation::Horizontal {
                    glory::style::FlexDirection::Row
                } else {
                    glory::style::FlexDirection::Column
                })
                .gap(4.),
        )
        .on(EventKind::FocusWithin, move |event| {
            if let Event::FocusWithin { focused, .. } = event {
                within.revise(|value| *value = focused);
            }
        })
        .fill(triggers);
    if let Some(name) = widget.common.name.take() {
        list = list.name(name);
    }
    let mut navigate_observers = widget
        .common
        .handlers
        .remove(&EventKind::Navigate)
        .unwrap_or_default();
    let weak = scope.downgrade();
    list = list.on(EventKind::Navigate, move |event| {
        if let Some(scope) = weak.upgrade() {
            let result = (|| -> Result {
                if let Event::Navigate { from, command } = event
                    && let Some(next) = roving.destination(from, command)?
                {
                    composite::focus_node(&scope, Some(next.node))?;
                }
                observe(&mut navigate_observers, owner, event);
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        }
    });
    let mut focus_observers = widget
        .common
        .handlers
        .remove(&EventKind::FocusWithin)
        .unwrap_or_default();
    let weak = scope.downgrade();
    let mut root = configure!(
        widget.common,
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
    .fill(panels)
    .on(EventKind::FocusWithin, move |event| {
        if let Some(scope) = weak.upgrade() {
            if matches!(event, Event::FocusWithin { focused: false, .. })
                && let Err(error) = focus.leave()
            {
                scope.report_error(error);
            }
            observe(&mut focus_observers, owner, event);
        }
    });
    for (kind, mut observers) in std::mem::take(&mut widget.common.handlers) {
        root = root.on(kind, move |event| observe(&mut observers, owner, event));
    }
    root.show_in(scope)
}
