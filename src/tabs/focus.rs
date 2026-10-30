//! Current-member focus repair with a bounded host confirmation.
use crate::composite::dynamic::Config;
use glory::render::{Action, NodeId, Query, QueryToken};
use glory::ui::Result;
use glory::{Cage, Scope, Vary};
use std::rc::Rc;

#[derive(Clone, Copy)]
pub(crate) struct FocusRestore {
    observed: Cage<Option<String>>,
    pending: Cage<bool>,
    restoring: Cage<Option<(NodeId, String)>>,
    task: Cage<Option<glory::core::TaskToken>>,
    query: Cage<Option<QueryToken>>,
    requester: Cage<glory::ui::WeakScope>,
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
            restoring: scope.cage(None),
            task: scope.cage(None),
            query: scope.cage(None),
            requester: scope.cage(scope.downgrade()),
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
                let restoring = glory::core::untracked(|| state.restoring.try_get())?;
                if restoring.is_some_and(|(_, key)| !current.iter().any(|config| config.key == key))
                {
                    state.cancel()?;
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
                    .and_then(|key| candidates.iter().find(|config| &config.key == key).copied())
                    .or_else(|| candidates.first().copied());
                let node = target
                    .map(|config| config.trigger.try_get())
                    .transpose()?
                    .flatten();
                if target.is_some() && node.is_none() {
                    return Ok(());
                }
                state.pending.try_revise(|pending| *pending = false)?;
                state.cancel()?;
                if let (Some(config), Some(node)) = (target, node) {
                    state.restore(&scope, node, &config.key)?;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        state
    }
    fn cancel(&self) -> Result {
        let task = glory::core::untracked(|| self.task.try_get())?;
        self.task.try_revise(|task| *task = None)?;
        if let Some(task) = task {
            task.cancel();
        }
        let query = glory::core::untracked(|| self.query.try_get())?;
        self.query.try_revise(|query| *query = None)?;
        if let Some(query) = query
            && let Some(requester) = glory::core::untracked(|| self.requester.try_get())?.upgrade()
        {
            requester.cancel_request(query)?;
        }
        self.restoring.try_revise(|value| *value = None)?;
        Ok(())
    }
    fn restore(self, scope: &Scope, node: NodeId, key: &str) -> Result {
        if !scope.capabilities()?.focus || !scope.is_active(node)? {
            return Ok(());
        }
        match scope.action(Action::Focus(node)) {
            Ok(()) => {}
            Err(glory::ui::Error::Render(glory::render::Error::Unsupported(_))) => {
                return Ok(());
            }
            Err(error) => return Err(error),
        }
        self.restoring
            .try_revise(|value| *value = Some((node, key.to_owned())))?;
        let weak = scope.downgrade();
        // Host::spawn never polls synchronously: the current semantic flush
        // commits the action before this task observes the host. Host focus
        // events precede its query reply, consuming the marker when accepted;
        // the reply clears it even when physical focus was refused.
        let task = scope.owner().spawn(async move {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                self.task.try_revise(|task| *task = None)?;
                if !self.matches(node)? {
                    return Ok(());
                }
                let token = scope.request(Query::Focus(node), move |_| {
                    let _ = self.query.try_revise(|query| *query = None);
                    let _ = self.restoring.try_revise(|value| *value = None);
                })?;
                self.query.try_revise(|query| *query = Some(token))?;
                Ok(())
            })();
            if let Err(error) = result {
                let _ = self.cancel();
                scope.report_error(error);
            }
        });
        match task {
            Ok(task) => self.task.try_revise(|slot| *slot = Some(task))?,
            Err(glory::core::AccessError::NoHost) => {
                self.restoring.try_revise(|value| *value = None)?;
            }
            Err(error) => {
                self.restoring.try_revise(|value| *value = None)?;
                return Err(error.into());
            }
        }
        Ok(())
    }
    fn matches(&self, node: NodeId) -> Result<bool> {
        Ok(glory::core::untracked(|| {
            self.restoring
                .try_with(|value| value.as_ref().is_some_and(|(target, _)| *target == node))
        })?)
    }
    /// A single observed focus gain may suppress automatic activation after a
    /// membership repair. This does not manufacture a focus observation.
    pub fn consume(&self, node: NodeId) -> Result<bool> {
        let restoring = self.matches(node)?;
        self.cancel()?;
        Ok(restoring)
    }
    pub fn observe(&self, key: &str, focused: bool) -> Result {
        if focused {
            self.cancel()?;
        }
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
        self.cancel()?;
        Ok(())
    }
}
