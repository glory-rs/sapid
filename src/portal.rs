//! Explicit semantic targets with source-owned, movable boxless content.
use crate::composite::{self, Callback, Factory};
use glory::render::{Descriptor, NodeId, Props, Semantics};
use glory::ui::{Mounted, Result, WeakScope, Widget};
use glory::{Scope, Vary};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet},
    rc::{Rc, Weak},
};
pub const MAX_PORTAL_TARGETS: usize = 128;
pub const MAX_PORTALS: usize = 1024;
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn key(value: &str) -> Result {
    if value.len() > 64 {
        return Err(invalid("Portal keys require at most 64 bytes"));
    }
    glory::render::SemanticId::new(value)?;
    Ok(())
}
#[derive(Clone, Copy)]
struct Target {
    owner: glory::core::InstanceId,
    node: Option<NodeId>,
}
struct Source {
    key: String,
    state: Weak<State>,
}
#[derive(Default)]
struct Tables {
    targets: BTreeMap<String, Target>,
    sources: BTreeMap<u64, Source>,
    subscribers: BTreeMap<String, BTreeSet<u64>>,
}
struct Registry {
    owner: glory::core::InstanceId,
    key: String,
    next: Cell<u64>,
    tables: RefCell<Tables>,
}
/// A bounded namespace within one App. It contains only live declarations.
#[derive(Clone)]
pub struct PortalContext(Rc<Registry>);
pub fn portal_context(scope: &Scope, namespace: impl Into<String>) -> Result<PortalContext> {
    let namespace = namespace.into();
    key(&namespace)?;
    let registry = Rc::new(Registry {
        owner: scope.id(),
        key: namespace,
        next: Cell::new(0),
        tables: RefCell::new(Tables::default()),
    });
    let weak = Rc::downgrade(&registry);
    scope.on_cleanup(move || {
        if let Some(registry) = weak.upgrade() {
            let tables = std::mem::take(&mut *registry.tables.borrow_mut());
            drop(tables);
        }
    })?;
    Ok(PortalContext(registry))
}
impl PortalContext {
    fn ensure(&self, scope: &Scope) -> Result {
        if !self.0.owner.is_alive() {
            return Err(glory::ui::Error::Gone);
        }
        if self.0.owner.runtime() != scope.id().runtime() {
            return Err(glory::render::Error::ForeignRuntime.into());
        }
        Ok(())
    }
    pub fn targets(&self) -> usize {
        self.0.tables.borrow().targets.len()
    }
    pub fn sources(&self) -> usize {
        self.0.tables.borrow().sources.len()
    }
}
impl Registry {
    fn source(&self, desired: String) -> Result<u64> {
        let mut tables = self.tables.borrow_mut();
        if tables.sources.len() >= MAX_PORTALS {
            return Err(invalid("Portal source budget exceeded"));
        }
        let id = self
            .next
            .get()
            .checked_add(1)
            .ok_or_else(|| invalid("Portal registration identity exhausted"))?;
        self.next.set(id);
        tables
            .subscribers
            .entry(desired.clone())
            .or_default()
            .insert(id);
        tables.sources.insert(
            id,
            Source {
                key: desired,
                state: Weak::new(),
            },
        );
        Ok(id)
    }
    fn remove_source(&self, id: u64) {
        let (source, bucket) = {
            let mut tables = self.tables.borrow_mut();
            let source = tables.sources.remove(&id);
            let bucket = source.as_ref().and_then(|source| {
                let empty = tables.subscribers.get_mut(&source.key).is_some_and(|set| {
                    set.remove(&id);
                    set.is_empty()
                });
                empty
                    .then(|| tables.subscribers.remove(&source.key))
                    .flatten()
            });
            (source, bucket)
        };
        drop(source);
        drop(bucket);
    }
    fn select(&self, id: u64, key: &str) -> Result {
        let removed = {
            let mut tables = self.tables.borrow_mut();
            let source = tables.sources.get_mut(&id).ok_or(glory::ui::Error::Gone)?;
            if source.key == key {
                return Ok(());
            }
            let old = std::mem::replace(&mut source.key, key.to_owned());
            let empty = tables.subscribers.get_mut(&old).is_some_and(|set| {
                set.remove(&id);
                set.is_empty()
            });
            let removed = empty.then(|| tables.subscribers.remove(&old)).flatten();
            tables
                .subscribers
                .entry(key.to_owned())
                .or_default()
                .insert(id);
            removed
        };
        drop(removed);
        Ok(())
    }
    fn notify(&self, key: &str, retiring: Option<NodeId>) {
        let states = {
            let tables = self.tables.borrow();
            tables
                .subscribers
                .get(key)
                .into_iter()
                .flat_map(|ids| ids.iter())
                .filter_map(|id| tables.sources.get(id).map(|source| source.state.clone()))
                .collect::<Vec<_>>()
        };
        for weak in states {
            if let Some(state) = weak.upgrade()
                && let Some(scope) = state.weak.upgrade()
            {
                // Target cleanup runs under a retired Owner. Existing source work
                // explicitly enters that other still-live Owner before mutation.
                let result = scope.owner().enter(|| {
                    if let Some(target) = retiring {
                        state.detach(Some(target), true)
                    } else {
                        state.sync()
                    }
                });
                state.report(result);
            }
        }
    }
}
pub struct PortalTarget {
    context: PortalContext,
    key: String,
    child: Option<Box<dyn Widget>>,
}
pub fn portal_target(context: &PortalContext, key: impl Into<String>) -> PortalTarget {
    PortalTarget {
        context: context.clone(),
        key: key.into(),
        child: None,
    }
}
impl PortalTarget {
    pub fn fill(mut self, widget: impl Widget) -> Self {
        self.child = Some(Box::new(widget));
        self
    }
}
impl Widget for PortalTarget {
    fn build(&mut self, scope: &Scope) -> Result {
        self.context.ensure(scope)?;
        key(&self.key)?;
        let name = composite::id("portal", &self.context.0.key, "target", &self.key)?;
        {
            let mut tables = self.context.0.tables.borrow_mut();
            if tables.targets.contains_key(&self.key) {
                return Err(invalid("duplicate live Portal target"));
            }
            if tables.targets.len() >= MAX_PORTAL_TARGETS {
                return Err(invalid("Portal target budget exceeded"));
            }
            tables.targets.insert(
                self.key.clone(),
                Target {
                    owner: scope.id(),
                    node: None,
                },
            );
        }
        let registry = self.context.0.clone();
        let selected = self.key.clone();
        let owner = scope.id();
        scope.on_cleanup(move || {
            let removed = {
                let mut tables = registry.tables.borrow_mut();
                if tables
                    .targets
                    .get(&selected)
                    .is_some_and(|target| target.owner == owner)
                {
                    tables.targets.remove(&selected)
                } else {
                    None
                }
            };
            if let Some(target) = removed {
                registry.notify(&selected, target.node);
            }
        })?;
        let mut descriptor = Descriptor::new(Props::View);
        descriptor.classes.push("sapid-portal-target".into());
        descriptor.semantics = composite::metadata(Semantics {
            id: Some(name),
            ..Default::default()
        });
        // The target and its retirement callback have exactly the same Owner.
        let node = scope.create(descriptor)?;
        scope.defer_descendant_disposal(node)?;
        // Publish the staged identity before child factories run. A child may
        // construct a foreign-owned source; even if that child fails, cleanup
        // must know the exact retiring target and park outside its subtree.
        self.context
            .0
            .tables
            .borrow_mut()
            .targets
            .get_mut(&self.key)
            .expect("reserved target")
            .node = Some(node);
        if let Some(child) = self.child.take() {
            let content = scope.child_in(node)?;
            child.show_in(&content)?;
            scope.retain(content)?;
        }
        self.context.0.notify(&self.key, None);
        Ok(())
    }
}
struct State {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    context: PortalContext,
    serial: u64,
    desired: glory::Cage<String>,
    enabled: Vary<bool>,
    origin: NodeId,
    content: RefCell<Option<Mounted>>,
    current: Cell<Option<NodeId>>,
    lost: Callback<String>,
}
impl State {
    fn report(&self, result: Result) {
        if let Err(error) = result
            && let Some(scope) = self.weak.upgrade()
        {
            scope.report_error(error);
        }
    }
    fn root(&self) -> Option<NodeId> {
        self.content
            .borrow()
            .as_ref()
            .and_then(|mounted| mounted.roots().first().copied())
    }
    fn detach(&self, retiring: Option<NodeId>, report_loss: bool) -> Result {
        let Some(scope) = self.weak.upgrade() else {
            return Ok(());
        };
        let Some(root) = self.root() else {
            return Ok(());
        };
        let was_attached = self.current.get().is_some();
        let origin_live = scope.is_active(self.origin).unwrap_or(false);
        let origin_dies = retiring
            .is_some_and(|target| scope.contains_node(target, self.origin).unwrap_or(false));
        let parent = (origin_live && !origin_dies).then_some(self.origin);
        scope.place(root, parent, None)?;
        if let Some(content) = self.content.borrow().as_ref() {
            content.suspend()?;
        }
        self.current.set(None);
        if was_attached && report_loss {
            self.lost.emit(self.owner, self.desired.try_get()?);
        }
        Ok(())
    }
    fn sync(&self) -> Result {
        if !self.owner.is_alive() {
            return Err(glory::ui::Error::Gone);
        }
        let Some(scope) = self.weak.upgrade() else {
            return Ok(());
        };
        let desired = self.desired.try_get()?;
        self.context.0.select(self.serial, &desired)?;
        let target = self
            .context
            .0
            .tables
            .borrow()
            .targets
            .get(&desired)
            .copied()
            .filter(|target| target.owner.is_alive())
            .and_then(|target| target.node);
        if !self.enabled.try_get()? || target.is_none() {
            return self.detach(None, target.is_none());
        }
        let target = target.expect("checked current target");
        let Some(root) = self.root() else {
            return Ok(());
        };
        if self.current.get() == Some(target) {
            return Ok(());
        }
        scope.place(root, Some(target), None)?;
        self.current.set(Some(target));
        Ok(())
    }
    fn clear(&self) {
        let content = self.content.borrow_mut().take();
        drop(content);
        self.lost.clear();
    }
}
pub struct Portal {
    context: PortalContext,
    target: Vary<String>,
    enabled: Vary<bool>,
    factory: Option<Factory>,
    lost: Option<Box<dyn FnMut(String)>>,
}
pub fn portal(context: &PortalContext, target: impl Into<Vary<String>>) -> Portal {
    Portal {
        context: context.clone(),
        target: target.into(),
        enabled: true.into(),
        factory: None,
        lost: None,
    }
}
impl Portal {
    pub fn content<W: Widget>(mut self, mut factory: impl FnMut() -> W + 'static) -> Self {
        self.factory = Some(Box::new(move || Box::new(factory())));
        self
    }
    pub fn enabled(mut self, enabled: impl Into<Vary<bool>>) -> Self {
        self.enabled = enabled.into();
        self
    }
    pub fn on_target_lost(mut self, callback: impl FnMut(String) + 'static) -> Self {
        self.lost = Some(Box::new(callback));
        self
    }
}
impl Widget for Portal {
    fn build(&mut self, scope: &Scope) -> Result {
        self.context.ensure(scope)?;
        let mut factory = self
            .factory
            .take()
            .ok_or_else(|| invalid("Portal requires content"))?;
        let desired = composite::validated(scope, &self.target, |target| key(target))?;
        let serial = self.context.0.source(desired.try_get()?)?;
        let registry = self.context.0.clone();
        scope.on_cleanup(move || registry.remove_source(serial))?;
        let origin = scope.create(Descriptor::new(Props::Group))?;
        let content_scope = scope.child_in(origin)?;
        let mounted = content_scope.mount_with(|| glory::ui::group().fill(factory()))?;
        mounted.suspend()?;
        scope.retain(content_scope)?;
        let state = Rc::new(State {
            owner: scope.id(),
            weak: scope.downgrade(),
            context: self.context.clone(),
            serial,
            desired,
            enabled: self.enabled.clone(),
            origin,
            content: RefCell::new(Some(mounted)),
            current: Cell::new(None),
            lost: Callback::new(self.lost.take()),
        });
        self.context
            .0
            .tables
            .borrow_mut()
            .sources
            .get_mut(&serial)
            .expect("reserved source")
            .state = Rc::downgrade(&state);
        let weak = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(state) = weak.upgrade() {
                state.clear();
            }
        })?;
        let live = state.clone();
        scope.structural(move || {
            live.desired.get();
            live.enabled.get();
            live.report(live.sync());
        });
        scope.retain(state)?;
        Ok(())
    }
}
