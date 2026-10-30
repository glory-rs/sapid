use super::*;
use crate::collection::{Collection, CollectionLimits};
use crate::composite::dynamic::{self, Items};

fn check_open(keys: &[String], multiple: bool) -> Result {
    dynamic::check_keys(keys)?;
    if !multiple && keys.len() > 1 {
        return Err(glory::render::Error::InvalidProps(
            "single accordion accepts at most one open key",
        )
        .into());
    }
    Ok(())
}

pub(super) fn build(widget: &mut Accordion, scope: &Scope, input: Items) -> Result {
    if !widget.items.is_empty() || !(1..=6).contains(&widget.heading) {
        return Err(glory::render::Error::InvalidProps(
            "dynamic accordion requires a valid heading and no static items",
        )
        .into());
    }
    let root_id = composite::id("a", &widget.key, "root", "root")?;
    let configs = dynamic::mount(scope, input)?;
    let local = widget.open.source.is_none();
    let control = std::mem::take(&mut widget.open).mount(scope);
    let multiple = widget.multiple;
    let source = composite::validated(scope, &control.source, move |keys| {
        check_open(keys, multiple)
    })?;
    let selected = scope.cage(Vec::<String>::new());
    let repair = control.clone();
    let collapsible = widget.collapsible;
    let weak = scope.downgrade();
    scope.structural(move || {
        let Some(scope) = weak.upgrade() else { return };
        let result = (|| -> Result {
            let configs = configs.try_get()?;
            let source = source.try_get()?;
            let mut next: Vec<_> = source
                .into_iter()
                .filter(|key| configs.iter().any(|config| &config.key == key))
                .collect();
            if local && !collapsible && next.is_empty() {
                let mut first = None;
                for config in &configs {
                    if config.enabled.try_get()? {
                        first = Some(config);
                        break;
                    }
                }
                if let Some(config) = first.or_else(|| configs.first()) {
                    next.push(config.key.clone());
                }
            }
            // Observe the accepted projection, never subscribe this normalizer
            // to the raw local source it may repair before validation publishes.
            glory::core::untracked(|| repair.accept(&next))?;
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
    let callback = Callback::new(widget.on_change.take());
    let presses = Observers::new(
        widget
            .common
            .handlers
            .remove(&EventKind::Activate)
            .unwrap_or_default(),
    );
    let namespace = widget.key.clone();
    let root_enabled = widget.common.enabled.clone();
    let theme = widget.common.theme.clone();
    let size = widget.common.size;
    let unstyled = widget.common.unstyled;
    let heading = widget.heading;
    let regions = widget.regions;
    let policy = widget.policy;
    let owner = scope.id();
    let registrations = collection.clone();
    let focus = dynamic::FocusRestore::new(
        scope,
        configs,
        selected.map(|keys| keys.first().cloned()).into(),
        root_enabled.clone(),
    );
    let rows = glory::ui::each(configs)
        .key(|config| config.key.clone())
        .children(move |entry| {
            let row = entry.value;
            let namespace = namespace.clone();
            let root_enabled = root_enabled.clone();
            let theme = theme.clone();
            let control = control.clone();
            let callback = callback.clone();
            let presses = presses.clone();
            let collection = registrations.clone();
            glory::ui::widget(move |scope| {
                let config = row.try_get()?;
                let trigger_id = composite::id("a", &namespace, "trigger", &config.key)?;
                let panel_id = composite::id("a", &namespace, "panel", &config.key)?;
                let root_enabled = root_enabled.clone();
                let item_enabled = config.enabled.clone();
                let enabled = scope.bond(move || root_enabled.get() && item_enabled.get());
                collection.register_ordered(
                    scope,
                    config.key.clone(),
                    config.trigger,
                    enabled,
                    config.label,
                    config.position,
                )?;
                let key = config.key.clone();
                let open = scope.bond(move || selected.with(|keys| keys.contains(&key)));
                let id = trigger_id.clone();
                let target = panel_id.clone();
                let mut trigger = glory::ui::button()
                    .class("sapid-accordion-trigger")
                    .label(config.label)
                    .enabled(enabled)
                    .node_ref(config.trigger)
                    .semantics(scope.bond(move || {
                        composite::metadata(Semantics {
                            id: Some(id.clone()),
                            controls: Some(target.clone()),
                            expanded: Some(open.get()),
                            navigation_item: true,
                            disabled: !collapsible
                                && open.get()
                                && selected.with(|keys| keys.len() == 1),
                            ..Default::default()
                        })
                    }));
                if !unstyled {
                    trigger = trigger.style(theme.control(size));
                }
                if let Some(mut factory) = config.take_trigger_content() {
                    trigger = trigger
                        .label("")
                        .name(config.label)
                        .fill(glory::ui::widget(move |scope| factory().show_in(scope)));
                }
                let control = control.clone();
                let callback = callback.clone();
                let presses = presses.clone();
                let key = config.key.clone();
                let weak = scope.downgrade();
                trigger = trigger.on(EventKind::Activate, move |event| {
                    let Some(scope) = weak.upgrade() else { return };
                    let result = (|| -> Result {
                        let mut next = selected.try_get()?;
                        if let Some(index) = next.iter().position(|old| old == &key) {
                            if !collapsible && next.len() == 1 {
                                return Ok(());
                            }
                            next.remove(index);
                        } else {
                            if !multiple {
                                next.clear();
                            }
                            next.push(key.clone());
                        }
                        control.accept(&next)?;
                        callback.emit(owner, next);
                        presses.emit(owner, event);
                        Ok(())
                    })();
                    if let Err(error) = result {
                        scope.report_error(error);
                    }
                });
                let panel = Panel {
                    id: panel_id,
                    label: trigger_id,
                    role: regions.then_some(Role::Section),
                    open: open.into(),
                    trigger: config.trigger,
                    restore: None,
                    reference: Some(config.panel),
                    factory: config.take_content(),
                    policy,
                    class: "sapid-accordion-panel",
                    unstyled,
                };
                let focused_key = config.key.clone();
                let weak = scope.downgrade();
                glory::ui::view()
                    .class("sapid-accordion-item")
                    .on(EventKind::FocusWithin, move |event| {
                        if let Event::FocusWithin { focused, .. } = event
                            && let Err(error) = focus.observe(&focused_key, focused)
                            && let Some(scope) = weak.upgrade()
                        {
                            scope.report_error(error);
                        }
                    })
                    .fill((
                        glory::ui::view().role(Role::Heading(heading)).fill(trigger),
                        panel,
                    ))
                    .show_in(scope)
            })
        });
    let mut observers = widget
        .common
        .handlers
        .remove(&EventKind::Navigate)
        .unwrap_or_default();
    let looping = widget.looping;
    let mut focus_observers = widget
        .common
        .handlers
        .remove(&EventKind::FocusWithin)
        .unwrap_or_default();
    let weak_focus = scope.downgrade();
    let weak = scope.downgrade();
    let mut root = configure!(
        widget.common,
        glory::ui::view(),
        scope,
        "sapid-accordion",
        composite::column(8.0)
    )
    .semantics(composite::metadata(Semantics {
        id: Some(root_id),
        navigation: Some(Orientation::Vertical),
        navigation_direction: widget.direction,
        ..Default::default()
    }))
    .fill(rows)
    .on(EventKind::FocusWithin, move |event| {
        if let Some(scope) = weak_focus.upgrade() {
            if matches!(event, Event::FocusWithin { focused: false, .. })
                && let Err(error) = focus.leave()
            {
                scope.report_error(error);
            }
            observe(&mut focus_observers, owner, event);
        }
    })
    .on(EventKind::Navigate, move |event| {
        if let Some(scope) = weak.upgrade() {
            let result = (|| -> Result {
                if let Event::Navigate { from, command } = event {
                    let mounted = collection.mounted()?;
                    if let Some(index) = mounted.iter().position(|item| item.node == from) {
                        let flags: Vec<_> = mounted.iter().map(|item| item.enabled).collect();
                        if let Some(next) = composite::destination(index, command, &flags, looping)
                        {
                            composite::focus_node(&scope, Some(mounted[next].node))?;
                        }
                    }
                }
                observe(&mut observers, owner, event);
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        }
    });
    for (kind, mut observers) in std::mem::take(&mut widget.common.handlers) {
        root = root.on(kind, move |event| observe(&mut observers, owner, event));
    }
    root.show_in(scope)
}
