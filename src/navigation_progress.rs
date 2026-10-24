//! Application-driven progress with one revocable, owner-scoped operation.
use crate::appearance::Common;
use glory::render::{NodeId, Semantics};
use glory::style::{Length, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope};
use std::{
    cell::Cell,
    rc::{Rc, Weak},
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavigationProgressSnapshot {
    pub active: bool,
    /// None is indeterminate; a completed operation retains Some(100.).
    pub value: Option<f64>,
}
#[derive(Clone, Copy, Default)]
struct State {
    generation: u64,
    snapshot: NavigationProgressSnapshot,
}
struct Controller {
    owner: glory::core::InstanceId,
    state: Cage<State>,
    next: Cell<u64>,
    live: Cell<bool>,
}
/// A generation-specific publication permit. Starting again revokes older permits.
#[derive(Clone)]
pub struct NavigationProgressToken {
    controller: Weak<Controller>,
    generation: u64,
}
#[derive(Clone)]
pub struct NavigationProgressHandle(Rc<Controller>);
impl NavigationProgressHandle {
    fn ensure(&self) -> Result {
        if self.0.live.get() && self.0.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    pub fn snapshot(&self) -> Result<NavigationProgressSnapshot> {
        self.ensure()?;
        Ok(glory::core::untracked(|| self.0.state.try_get())?.snapshot)
    }
    /// Begins an indeterminate operation, revoking any previous operation.
    pub fn start(&self) -> Result<NavigationProgressToken> {
        self.ensure()?;
        let generation = self
            .0
            .next
            .get()
            .checked_add(1)
            .ok_or_else(|| invalid("navigation progress generation exhausted"))?;
        self.0.state.try_revise(|state| {
            *state = State {
                generation,
                snapshot: NavigationProgressSnapshot {
                    active: true,
                    value: None,
                },
            }
        })?;
        self.0.next.set(generation);
        Ok(NavigationProgressToken {
            controller: Rc::downgrade(&self.0),
            generation,
        })
    }
    fn publish(
        &self,
        token: &NavigationProgressToken,
        value: Option<f64>,
        terminal: bool,
    ) -> Result<bool> {
        self.ensure()?;
        if !Weak::ptr_eq(&token.controller, &Rc::downgrade(&self.0)) {
            return Err(invalid(
                "navigation progress token belongs to another controller",
            ));
        }
        let current = glory::core::untracked(|| self.0.state.try_get())?;
        if current.generation != token.generation || !current.snapshot.active {
            return Ok(false);
        }
        let next = NavigationProgressSnapshot {
            active: !terminal,
            value,
        };
        if current.snapshot != next {
            self.0.state.try_revise(|state| state.snapshot = next)?;
        }
        Ok(true)
    }
    /// Publishes a finite percentage in 0..=100. Progress may move backwards.
    pub fn set(&self, token: &NavigationProgressToken, value: f64) -> Result<bool> {
        if !value.is_finite() || !(0.0..=100.0).contains(&value) {
            return Err(invalid("navigation progress must be finite and in 0..=100"));
        }
        self.publish(token, Some(value), false)
    }
    pub fn indeterminate(&self, token: &NavigationProgressToken) -> Result<bool> {
        self.publish(token, None, false)
    }
    /// Sets the terminal percentage to 100 and immediately hides the retained bar.
    pub fn finish(&self, token: &NavigationProgressToken) -> Result<bool> {
        self.publish(token, Some(100.), true)
    }
    /// Hides the bar and revokes this operation; no timers or requests are involved.
    pub fn cancel(&self, token: &NavigationProgressToken) -> Result<bool> {
        self.publish(token, None, true)
    }
}
pub struct NavigationProgress {
    common: Common,
    thickness: f64,
    ready: Option<Box<dyn FnMut(NavigationProgressHandle)>>,
}
pub fn navigation_progress() -> NavigationProgress {
    NavigationProgress {
        common: Common::default(),
        thickness: 3.,
        ready: None,
    }
}
impl NavigationProgress {
    pub fn on_ready(
        mut self,
        callback: impl FnMut(NavigationProgressHandle) + 'static,
    ) -> Self {
        self.ready = Some(Box::new(callback));
        self
    }
    pub fn name(mut self, value: impl Into<glory::Lotus<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn class(mut self, value: impl Into<glory::Lotus<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<glory::Lotus<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn thickness(mut self, value: f64) -> Self {
        self.thickness = value;
        self
    }
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
impl Widget for NavigationProgress {
    fn build(&mut self, scope: &Scope) -> Result {
        if !self.thickness.is_finite() || !(1.0..=128.0).contains(&self.thickness) {
            return Err(invalid("navigation progress thickness must be in 1..=128"));
        }
        let state = scope.cage(State::default());
        let controller = Rc::new(Controller {
            owner: scope.id(),
            state,
            next: Cell::new(0),
            live: Cell::new(true),
        });
        let cleanup = controller.clone();
        scope.on_cleanup(move || cleanup.live.set(false))?;
        let value = scope.bond(move || state.get().snapshot.value.unwrap_or(0.));
        let indeterminate = scope.bond(move || state.get().snapshot.value.is_none());
        let name = self
            .common
            .name
            .take()
            .unwrap_or_else(|| "Navigation progress".into());
        let (classes, styles) = self.common.appearance(
            scope,
            "sapid-navigation-progress",
            Style::new().width(Length::Percent(100.)),
        );
        let mut root = glory::ui::view()
            .class(classes)
            .style(styles)
            .semantics(scope.bond(move || {
                let active = state.get().snapshot.active;
                Some(Box::new(Semantics {
                    hidden: !active,
                    busy: Some(active),
                    ..Default::default()
                }))
            }));
        let mut bar = crate::progress(value)
            .indeterminate(indeterminate)
            .name(name)
            .theme(self.common.theme.clone())
            .thickness(self.thickness);
        if self.common.unstyled {
            bar = bar.unstyled()
        }
        if let Some(reference) = self.common.node_ref {
            root = root.node_ref(reference);
        }
        root.fill(bar).show_in(scope)?;
        if let Some(mut ready) = self.ready.take() {
            ready(NavigationProgressHandle(controller));
        }
        Ok(())
    }
}
