use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use glory::render::{Action, Event, EventKind, NodeId, Role, SemanticId, Semantics};
use glory::style::{Display, Edges, FlexDirection, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};

/// Panel shells and their relations always exist. This controls the lifetime of
/// the content inside the shell, independently from its semantic visibility.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ContentPolicy {
    #[default]
    Unmount,
    KeepMounted,
}

pub(crate) type Factory = Box<dyn FnMut() -> Box<dyn Widget>>;
type ChangeHandler<T> = Box<dyn FnMut(T)>;

pub(crate) struct Callback<T>(Rc<RefCell<Option<ChangeHandler<T>>>>);
impl<T> Clone for Callback<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T> Callback<T> {
    pub fn new(callback: Option<ChangeHandler<T>>) -> Self {
        Self(Rc::new(RefCell::new(callback)))
    }
    pub fn emit(&self, owner: glory::core::InstanceId, value: T) {
        if !owner.is_alive() {
            self.clear();
            return;
        }
        let callback = self.0.borrow_mut().take();
        if let Some(mut callback) = callback {
            callback(value);
            if owner.is_alive() {
                *self.0.borrow_mut() = Some(callback);
            }
        }
    }
    pub fn clear(&self) {
        let callback = self.0.borrow_mut().take();
        drop(callback);
    }
}

type EventHandlers = Vec<Box<dyn FnMut(Event)>>;
#[derive(Clone)]
pub(crate) struct Observers(Rc<RefCell<EventHandlers>>);
impl Observers {
    pub fn new(handlers: EventHandlers) -> Self {
        Self(Rc::new(RefCell::new(handlers)))
    }
    pub fn emit(&self, owner: glory::core::InstanceId, event: Event) {
        let mut handlers = std::mem::take(&mut *self.0.borrow_mut());
        crate::appearance::observe(&mut handlers, owner, event);
        if owner.is_alive() {
            *self.0.borrow_mut() = handlers;
        }
    }
    pub fn clear(&self) {
        let handlers = std::mem::take(&mut *self.0.borrow_mut());
        drop(handlers);
    }
}

pub(crate) fn id(kind: &str, namespace: &str, part: &str, item: &str) -> Result<SemanticId> {
    for key in [namespace, item] {
        if key.len() > 64 {
            return Err(glory::render::Error::InvalidProps(
                "widget keys are limited to 64 bytes",
            )
            .into());
        }
        SemanticId::new(key)?;
    }
    Ok(SemanticId::new(format!(
        "sapid-{kind}-{}_{namespace}-{part}-{}_{item}",
        namespace.len(),
        item.len()
    ))?)
}

pub(crate) fn metadata(value: Semantics) -> Option<Box<Semantics>> {
    Some(Box::new(value))
}

pub(crate) fn column(gap: f64) -> Style {
    Style::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(gap)
}

pub(crate) fn validate_keys(keys: &[String]) -> Result {
    let mut seen = HashSet::new();
    for key in keys {
        SemanticId::new(key)?;
        if key.len() > 64 || !seen.insert(key) {
            return Err(glory::render::Error::InvalidProps(
                "widget item keys must be unique bounded tokens",
            )
            .into());
        }
    }
    Ok(())
}

/// Validate external state before publishing it to any child binding. Retain
/// only the current value; invalid program updates leave the last valid UI.
pub(crate) fn validated<T: Clone + PartialEq + 'static>(
    scope: &Scope,
    source: &Lotus<T>,
    check: impl Fn(&T) -> Result + 'static,
) -> Result<Cage<T>> {
    let initial = source.try_with(|value| {
        check(value)?;
        Ok::<T, glory::ui::Error>(value.clone())
    })??;
    let current = scope.cage(initial);
    // Lotus::Value owns T directly. Clone its source only after validation,
    // otherwise even entering this helper could duplicate an oversized value.
    let source = source.clone();
    let weak = scope.downgrade();
    scope.structural(move || {
        let Some(scope) = weak.upgrade() else { return };
        let result = (|| -> Result {
            let next = source.try_with(|value| {
                check(value)?;
                Ok::<T, glory::ui::Error>(value.clone())
            })??;
            if glory::core::untracked(|| current.try_with(|old| old != &next))? {
                current.try_revise(|old| *old = next)?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            scope.report_error(error);
        }
    });
    Ok(current)
}

pub(crate) struct Members {
    pub references: Vec<Cage<Option<NodeId>>>,
    pub enabled: Vec<Lotus<bool>>,
    pub root_enabled: Lotus<bool>,
    index: RefCell<HashMap<NodeId, usize>>,
}
impl Members {
    pub fn new(scope: &Scope, items: &[Item], root_enabled: Lotus<bool>) -> Rc<Self> {
        Rc::new(Self {
            references: items
                .iter()
                .map(|item| item.trigger.unwrap_or_else(|| scope.cage(None)))
                .collect(),
            enabled: items.iter().map(|item| item.enabled.clone()).collect(),
            root_enabled,
            index: RefCell::new(HashMap::new()),
        })
    }
    pub fn register(&self) -> Result {
        let mut index = self.index.borrow_mut();
        for (i, reference) in self.references.iter().enumerate() {
            index.insert(reference.try_get()?.ok_or(glory::ui::Error::Gone)?, i);
        }
        Ok(())
    }
    pub fn flags(&self) -> Result<Vec<bool>> {
        let root = self.root_enabled.try_get()?;
        self.enabled
            .iter()
            .map(|value| Ok(root && value.try_get()?))
            .collect()
    }
    pub fn locate(&self, from: NodeId) -> Option<usize> {
        self.index.borrow().get(&from).copied()
    }
    pub fn navigate(
        &self,
        scope: &Scope,
        from: NodeId,
        command: glory::render::Navigation,
        looping: bool,
    ) -> Result {
        let Some(current) = self.locate(from) else {
            return Ok(());
        };
        if let Some(next) = destination(current, command, &self.flags()?, looping) {
            focus(scope, self.references[next])?;
        }
        Ok(())
    }
}

pub(crate) fn focus(scope: &Scope, target: Cage<Option<NodeId>>) -> Result {
    focus_node(scope, target.try_get()?)
}
pub(crate) fn focus_node(scope: &Scope, target: Option<NodeId>) -> Result {
    if !scope.capabilities()?.focus {
        return Ok(());
    }
    if let Some(target) = target
        && scope.is_active(target)?
    {
        match scope.action(Action::Focus(target)) {
            Ok(()) | Err(glory::ui::Error::Render(glory::render::Error::Unsupported(_))) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

pub(crate) struct Panel {
    pub id: SemanticId,
    pub label: SemanticId,
    pub role: Option<Role>,
    pub open: Lotus<bool>,
    pub trigger: Cage<Option<NodeId>>,
    pub restore: Option<Lotus<Option<NodeId>>>,
    pub reference: Option<Cage<Option<NodeId>>>,
    pub factory: Option<Factory>,
    pub policy: ContentPolicy,
    pub class: &'static str,
    pub unstyled: bool,
}
impl Widget for Panel {
    fn build(&mut self, scope: &Scope) -> Result {
        let open = self.open.clone();
        let id = self.id.clone();
        let label = self.label.clone();
        let focused = scope.cage(false);
        let mut panel = glory::ui::view()
            .class(self.class)
            .semantics(scope.bond(move || {
                metadata(Semantics {
                    id: Some(id.clone()),
                    labelled_by: Some(label.clone()),
                    hidden: !open.get(),
                    ..Semantics::default()
                })
            }))
            .on(EventKind::FocusWithin, move |event| {
                if let Event::FocusWithin { focused: value, .. } = event {
                    focused.revise(|v| *v = value);
                }
            });
        if let Some(role) = self.role {
            panel = panel.role(role);
        }
        if let Some(reference) = self.reference {
            panel = panel.node_ref(reference);
        }
        if !self.unstyled {
            panel = panel.style(Style::new().padding(Edges::all(12.0)));
        }
        if self.role == Some(Role::TabPanel) {
            panel = panel.focus(glory::render::FocusMode::TabStop);
        }
        if let Some(mut factory) = self.factory.take() {
            panel = match self.policy {
                ContentPolicy::KeepMounted => {
                    panel.fill(glory::ui::component(move |scope| factory().show_in(scope)))
                }
                ContentPolicy::Unmount => {
                    panel.fill(glory::ui::when(self.open.clone(), factory))
                }
            };
        }
        panel.show_in(scope)?;
        let open = self.open.clone();
        let mut previous = open.try_get()?;
        let restore = self.restore.clone().unwrap_or_else(|| self.trigger.into());
        let weak = scope.downgrade();
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let now = open.try_get()?;
                if previous && !now && focused.try_get()? {
                    // Disposal can retire the origin before a native blur is
                    // delivered. Do not carry that focus into a later opening.
                    focused.try_revise(|value| *value = false)?;
                    focus_node(&scope, restore.try_get()?)?;
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

/// One statically configured member; only mounted members are held, never a
/// history of visited data. Dynamic large collections remain Glory List's job.
pub(crate) struct Item {
    pub key: String,
    pub label: Lotus<String>,
    pub enabled: Lotus<bool>,
    pub factory: Option<Factory>,
    pub trigger: Option<Cage<Option<NodeId>>>,
    pub panel: Option<Cage<Option<NodeId>>>,
}
impl Item {
    pub fn new(key: String) -> Self {
        Self {
            label: key.clone().into(),
            key,
            enabled: true.into(),
            factory: None,
            trigger: None,
            panel: None,
        }
    }
}

macro_rules! item_methods {
    () => {
        pub fn label(mut self, label: impl Into<glory::Lotus<String>>) -> Self {
            self.inner.label = label.into();
            self
        }
        pub fn enabled(mut self, enabled: impl Into<glory::Lotus<bool>>) -> Self {
            self.inner.enabled = enabled.into();
            self
        }
        pub fn content<W: glory::Widget>(
            mut self,
            mut factory: impl FnMut() -> W + 'static,
        ) -> Self {
            self.inner.factory = Some(Box::new(move || Box::new(factory())));
            self
        }
        pub fn trigger_ref(
            mut self,
            reference: glory::Cage<Option<glory::render::NodeId>>,
        ) -> Self {
            self.inner.trigger = Some(reference);
            self
        }
        pub fn panel_ref(
            mut self,
            reference: glory::Cage<Option<glory::render::NodeId>>,
        ) -> Self {
            self.inner.panel = Some(reference);
            self
        }
    };
}
pub(crate) use item_methods;

pub(crate) fn destination(
    current: usize,
    command: glory::render::Navigation,
    enabled: &[bool],
    looping: bool,
) -> Option<usize> {
    use glory::render::Navigation;
    let first = || enabled.iter().position(|v| *v);
    let last = || enabled.iter().rposition(|v| *v);
    match command {
        Navigation::First => first(),
        Navigation::Last => last(),
        Navigation::Next => ((current + 1)..enabled.len())
            .find(|i| enabled[*i])
            .or_else(|| if looping { first() } else { None }),
        Navigation::Previous => (0..current)
            .rev()
            .find(|i| enabled[*i])
            .or_else(|| if looping { last() } else { None }),
        _ => None,
    }
}

#[cfg(test)]
mod bounded_validation_tests {
    use super::*;
    use std::cell::Cell;
    struct Value {
        length: usize,
        clones: Rc<Cell<usize>>,
    }
    impl Clone for Value {
        fn clone(&self) -> Self {
            self.clones.set(self.clones.get() + 1);
            Self {
                length: self.length,
                clones: self.clones.clone(),
            }
        }
    }
    impl PartialEq for Value {
        fn eq(&self, other: &Self) -> bool {
            self.length == other.length
        }
    }
    struct Probe(Cage<usize>);
    impl Drop for Probe {
        fn drop(&mut self) {
            self.0.revise(|value| *value += 1);
        }
    }
    fn check(value: &Value) -> Result {
        if value.length > 3 {
            Err(glory::render::Error::InvalidProps("bounded validation test").into())
        } else {
            Ok(())
        }
    }
    #[test]
    fn oversized_constant_lotus_is_borrowed_without_even_one_clone() {
        let app = glory::App::memory();
        let clones = Rc::new(Cell::new(0));
        let source = Lotus::Value(Value {
            length: 1_000_000,
            clones: clones.clone(),
        });
        assert!(validated(app.scope(), &source, check).is_err());
        assert_eq!(clones.get(), 0);
        assert_eq!(app.stats().nodes, 0);
    }
    #[test]
    fn invalid_initial_source_never_clones_and_drops_check_capture_outside_value_borrow() {
        let app = glory::App::memory();
        let clones = Rc::new(Cell::new(0));
        let dropped = app.scope().cage(0);
        let probe = Probe(dropped);
        let source = app.scope().cage(Value {
            length: 1_000_000,
            clones: clones.clone(),
        });
        assert!(
            validated(app.scope(), &source.into(), move |value| {
                let _ = &probe;
                check(value)
            })
            .is_err()
        );
        assert_eq!(clones.get(), 0);
        assert_eq!(dropped.get(), 1);
    }
    #[test]
    fn invalid_structural_updates_never_clone_recover_and_retire_with_owner() {
        let app = glory::App::memory();
        let owner = app.scope().child().unwrap();
        let clones = Rc::new(Cell::new(0));
        let dropped = app.scope().cage(0);
        let probe = Probe(dropped);
        let source = app.scope().cage(Value {
            length: 1,
            clones: clones.clone(),
        });
        let accepted = validated(&owner, &source.into(), move |value| {
            let _ = &probe;
            check(value)
        })
        .unwrap();
        app.flush().unwrap();
        let initial = clones.get();
        source.revise(|value| value.length = 1_000_000);
        assert!(app.flush().is_err());
        assert_eq!(clones.get(), initial);
        assert_eq!(accepted.with(|value| value.length), 1);
        source.revise(|value| value.length = 2);
        app.flush().unwrap();
        assert_eq!(clones.get(), initial + 1);
        assert_eq!(accepted.with(|value| value.length), 2);
        owner.dispose();
        assert!(!accepted.is_alive());
        assert_eq!(dropped.get(), 1);
        let retired = clones.get();
        source.revise(|value| value.length = 3);
        app.flush().unwrap();
        assert_eq!(clones.get(), retired);
    }
}
