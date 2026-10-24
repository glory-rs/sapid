use crate::composite::{self, ContentPolicy, Factory};
use glory::render::{CollapseConfig, Event, EventKind, NodeId, Semantics};
use glory::style::{Edges, Overflow, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};

#[derive(Clone, Copy)]
pub(super) struct State {
    pub config: Cage<CollapseConfig>,
    presented: Cage<bool>,
    policy: ContentPolicy,
}
fn next_config(
    previous: CollapseConfig,
    open: bool,
    duration_millis: u16,
) -> Result<CollapseConfig> {
    let mut next = CollapseConfig {
        open,
        duration_millis,
        revision: previous.revision,
    };
    next.validate()?;
    if next != previous {
        next.revision = previous
            .revision
            .checked_add(1)
            .ok_or(glory::render::Error::CollapseRevisionExhausted)?;
    }
    Ok(next)
}
impl State {
    pub fn new(
        scope: &Scope,
        open: Lotus<bool>,
        duration: Lotus<u16>,
        animated: Lotus<bool>,
        policy: ContentPolicy,
    ) -> Result<Self> {
        let initial = CollapseConfig {
            open: open.try_get()?,
            duration_millis: duration.try_get()?,
            revision: 0,
        };
        initial.validate()?;
        let initial = CollapseConfig {
            duration_millis: if animated.try_get()? {
                initial.duration_millis
            } else {
                0
            },
            ..initial
        };
        let state = Self {
            config: scope.cage(initial),
            presented: scope.cage(initial.open || policy == ContentPolicy::KeepMounted),
            policy,
        };
        let completion = scope.capabilities()?.collapse_completion;
        let weak = scope.downgrade();
        scope.structural(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let duration = duration.try_get()?;
                CollapseConfig {
                    duration_millis: duration,
                    ..Default::default()
                }
                .validate()?;
                let duration = if animated.try_get()? { duration } else { 0 };
                let next = next_config(
                    glory::core::untracked(|| state.config.try_get())?,
                    open.try_get()?,
                    duration,
                )?;
                if glory::core::untracked(|| state.config.try_get())? != next {
                    state.config.try_revise(|config| *config = next)?;
                }
                let present = if policy == ContentPolicy::KeepMounted || next.open {
                    Some(true)
                } else if !completion {
                    Some(false)
                } else {
                    None
                };
                if let Some(present) = present
                    && glory::core::untracked(|| state.presented.try_get())? != present
                {
                    state.presented.try_revise(|value| *value = present)?;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        Ok(state)
    }
    fn complete(&self, revision: u64) -> Result {
        let current = self.config.try_get()?;
        if current.revision == revision
            && !current.open
            && self.policy == ContentPolicy::Unmount
            && self.presented.try_get()?
        {
            self.presented.try_revise(|value| *value = false)?;
        }
        Ok(())
    }
}
pub(super) struct Panel {
    pub id: glory::render::SemanticId,
    pub label: glory::render::SemanticId,
    pub state: State,
    pub trigger: Cage<Option<NodeId>>,
    pub reference: Option<Cage<Option<NodeId>>>,
    pub factory: Option<Factory>,
    pub unstyled: bool,
}
impl Widget for Panel {
    fn build(&mut self, scope: &Scope) -> Result {
        let state = self.state;
        let id = self.id.clone();
        let label = self.label.clone();
        let focused = scope.cage(false);
        let weak = scope.downgrade();
        let mut panel = glory::ui::view()
            .class("sapid-collapsible-panel")
            .style(
                Style::new()
                    .overflow_x(Overflow::Hidden)
                    .overflow_y(Overflow::Hidden),
            )
            .semantics(scope.bond(move || {
                let config = state.config.get();
                composite::metadata(Semantics {
                    id: Some(id.clone()),
                    labelled_by: Some(label.clone()),
                    hidden: !config.open,
                    inert: !config.open,
                    collapse: Some(config),
                    ..Default::default()
                })
            }))
            .on(EventKind::CollapseComplete, move |event| {
                if let Event::CollapseComplete { revision } = event
                    && let Err(error) = state.complete(revision)
                    && let Some(scope) = weak.upgrade()
                {
                    scope.report_error(error);
                }
            })
            .on(EventKind::FocusWithin, move |event| {
                if let Event::FocusWithin { focused: value, .. } = event {
                    focused.revise(|focused| *focused = value);
                }
            });
        if let Some(reference) = self.reference {
            panel = panel.node_ref(reference);
        }
        if let Some(factory) = self.factory.take() {
            let mut inner = glory::ui::view().class("sapid-collapsible-body");
            if !self.unstyled {
                // The clipping shell owns height and has no padding/border.
                // The normal 8px disclosure gap belongs inside the clip.
                inner = inner.style(Style::new().padding(Edges {
                    top: 20.,
                    right: 12.,
                    bottom: 12.,
                    left: 12.,
                }));
            }
            panel = panel.fill(inner.fill(glory::ui::when(state.presented, factory)));
        }
        panel.show_in(scope)?;
        let trigger = self.trigger;
        let mut previous = state.config.try_get()?.open;
        let weak = scope.downgrade();
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let now = state.config.try_get()?.open;
                if previous && !now && focused.try_get()? {
                    focused.try_revise(|value| *value = false)?;
                    composite::focus(&scope, trigger)?;
                }
                previous = now;
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn revision_only_advances_for_accepted_changes_and_exhaustion_is_typed() {
        let previous = CollapseConfig {
            open: false,
            duration_millis: 180,
            revision: u64::MAX,
        };
        assert_eq!(next_config(previous, false, 180).unwrap(), previous);
        assert!(matches!(
            next_config(previous, true, 180),
            Err(glory::ui::Error::Render(
                glory::render::Error::CollapseRevisionExhausted
            ))
        ));
        assert!(next_config(previous, false, 501).is_err());
        let previous = CollapseConfig {
            revision: 3,
            ..previous
        };
        assert_eq!(next_config(previous, true, 0).unwrap().revision, 4);
    }
}
