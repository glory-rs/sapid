//! Bounded keyed configuration shared by the trigger and panel renderers.
use super::{Factory, Item};
use glory::ui::Result;
use glory::{Cage, Scope, Vary};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub(crate) const LIMIT: usize = 256;
pub(crate) struct Items {
    pub source: Vary<Vec<String>>,
    pub factory: Box<dyn FnMut(&str) -> Item>,
}
pub(crate) struct Config {
    _owner: Scope,
    pub key: String,
    pub label: Cage<String>,
    pub enabled: Vary<bool>,
    pub trigger: Cage<Option<glory::render::NodeId>>,
    pub panel: Cage<Option<glory::render::NodeId>>,
    pub position: Cage<usize>,
    content: RefCell<Option<Factory>>,
    trigger_content: RefCell<Option<Factory>>,
}
impl Config {
    pub fn take_content(&self) -> Option<Factory> {
        self.content.borrow_mut().take()
    }
    pub fn take_trigger_content(&self) -> Option<Factory> {
        self.trigger_content.borrow_mut().take()
    }
}
pub(crate) fn check_keys(keys: &[String]) -> Result {
    if keys.len() > LIMIT {
        return Err(glory::render::Error::InvalidProps(
            "dynamic composite exceeds the 256-item budget",
        )
        .into());
    }
    super::validate_keys(keys)
}
fn create(
    scope: &Scope,
    key: &str,
    position: usize,
    factory: &mut dyn FnMut(&str) -> Item,
) -> Result<Rc<Config>> {
    let owner = scope.child()?;
    let item = owner.owner().enter(|| factory(key));
    if item.key != key {
        return Err(glory::render::Error::InvalidProps(
            "dynamic composite factory returned a different key",
        )
        .into());
    }
    let label = super::validated(&owner, &item.label, |label| {
        if label.is_empty() || label.len() > 256 {
            return Err(glory::render::Error::InvalidProps(
                "dynamic composite labels must contain 1..=256 bytes",
            )
            .into());
        }
        Ok(())
    })?;
    item.enabled.try_get()?;
    let trigger = item.trigger.unwrap_or_else(|| owner.cage(None));
    let panel = item.panel.unwrap_or_else(|| owner.cage(None));
    let position = owner.cage(position);
    Ok(Rc::new(Config {
        _owner: owner,
        key: item.key,
        label,
        enabled: item.enabled,
        trigger,
        panel,
        position,
        content: RefCell::new(item.factory),
        trigger_content: RefCell::new(item.trigger_content),
    }))
}

/// Configuration factories run once per newly admitted key, outside registry
/// borrows. Only current configurations are retained; removing a key releases
/// its short Owner after both keyed renderers have retired their row.
pub(crate) fn mount(scope: &Scope, mut input: Items) -> Result<Cage<Vec<Rc<Config>>>> {
    let initial = input.source.try_with(|keys| {
        check_keys(keys)?;
        Ok::<_, glory::ui::Error>(keys.clone())
    })??;
    let mut configs = Vec::with_capacity(initial.len());
    for (position, key) in initial.iter().enumerate() {
        configs.push(create(scope, key, position, &mut input.factory)?);
    }
    let current = scope.cage(configs);
    let weak = scope.downgrade();
    scope.structural(move || {
        let Some(scope) = weak.upgrade() else { return };
        let result = (|| -> Result {
            let keys = input.source.try_with(|keys| {
                check_keys(keys)?;
                Ok::<_, glory::ui::Error>(keys.clone())
            })??;
            let previous = glory::core::untracked(|| current.try_get())?;
            if previous.len() == keys.len()
                && previous.iter().zip(&keys).all(|(old, key)| &old.key == key)
            {
                return Ok(());
            }
            let mut retained: HashMap<_, _> = previous
                .into_iter()
                .map(|config| (config.key.clone(), config))
                .collect();
            let mut next = Vec::with_capacity(keys.len());
            for (position, key) in keys.iter().enumerate() {
                next.push(match retained.remove(key) {
                    Some(config) => config,
                    None => create(&scope, key, position, &mut input.factory)?,
                });
            }
            for (position, config) in next.iter().enumerate() {
                if glory::core::untracked(|| config.position.try_get())? != position {
                    config.position.try_revise(|value| *value = position)?;
                }
            }
            current.try_revise(|value| *value = next)?;
            Ok(())
        })();
        if let Err(error) = result {
            scope.report_error(error);
        }
    });
    Ok(current)
}

/// Retain only the currently observed member key. A removed focused member
/// hands focus to a live enabled trigger after keyed row reconciliation.
#[derive(Clone, Copy)]
pub(crate) struct FocusRestore {
    observed: Cage<Option<String>>,
    pending: Cage<bool>,
}
impl FocusRestore {
    pub fn new(
        scope: &Scope,
        configs: Cage<Vec<Rc<Config>>>,
        preferred: Vary<Option<String>>,
        enabled: Vary<bool>,
    ) -> Self {
        let state = Self {
            observed: scope.cage(None::<String>),
            pending: scope.cage(false),
        };
        let weak = scope.downgrade();
        scope.structural(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let current = configs.try_get()?;
                let focused = glory::core::untracked(|| state.observed.try_get())?;
                if focused.is_some_and(|key| !current.iter().any(|config| config.key == key)) {
                    state.observed.try_revise(|key| *key = None)?;
                    state.pending.try_revise(|pending| *pending = true)?;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        let weak = scope.downgrade();
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                if !state.pending.try_get()? {
                    return Ok(());
                }
                let configs = configs.try_get()?;
                let preferred = preferred.try_get()?;
                let mut candidates = Vec::new();
                if enabled.try_get()? {
                    for config in &configs {
                        if config.enabled.try_get()? {
                            candidates.push(config);
                        }
                    }
                }
                let target = preferred
                    .as_ref()
                    .and_then(|key| {
                        candidates.iter().find(|config| &config.key == key).copied()
                    })
                    .or_else(|| candidates.first().copied());
                let node = target
                    .map(|config| config.trigger.try_get())
                    .transpose()?
                    .flatten();
                if target.is_some() && node.is_none() {
                    return Ok(());
                }
                state.pending.try_revise(|pending| *pending = false)?;
                super::focus_node(&scope, node)
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        state
    }
    pub fn observe(&self, key: &str, focused: bool) -> Result {
        let old = self.observed.try_get()?;
        if focused && old.as_deref() != Some(key) {
            self.observed
                .try_revise(|value| *value = Some(key.to_owned()))?;
        } else if !focused && old.as_deref() == Some(key) {
            self.observed.try_revise(|value| *value = None)?;
        }
        Ok(())
    }
    pub fn leave(&self) -> Result {
        if self.observed.try_get()?.is_some() {
            self.observed.try_revise(|value| *value = None)?;
        }
        if self.pending.try_get()? {
            self.pending.try_revise(|pending| *pending = false)?;
        }
        Ok(())
    }
}
