//! Owner-scoped mounted collection, roving focus and explicit-clock typeahead.
//! Only mounted registrations are retained. No business source is enumerated.
use std::cell::RefCell;
use std::rc::Rc;

use glory::render::{Navigation, NodeId, SemanticId, validate_typeahead};
use glory::ui::Result;
use glory::{Cage, Lotus, Scope};

#[derive(Clone, Copy, Debug)]
pub struct CollectionLimits {
    pub items: usize,
    pub label_bytes: usize,
}
impl Default for CollectionLimits {
    fn default() -> Self {
        Self {
            items: 1024,
            label_bytes: 256,
        }
    }
}

/// An opaque registration identity. Reusing a key never reuses this identity.
#[derive(Clone, Debug)]
pub struct CollectionMember {
    collection: Rc<()>,
    serial: u64,
}
impl PartialEq for CollectionMember {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.collection, &other.collection) && self.serial == other.serial
    }
}
impl Eq for CollectionMember {}

#[derive(Clone)]
struct Entry {
    member: CollectionMember,
    key: String,
    reference: Cage<Option<NodeId>>,
    enabled: Lotus<bool>,
    label: Lotus<String>,
    position: Option<Lotus<usize>>,
}
struct Registry {
    entries: Vec<Entry>,
    next: u64,
    closed: bool,
}

/// One current mounted item, never an item from a virtual collection source.
#[derive(Clone, Debug)]
pub struct MountedItem {
    pub member: CollectionMember,
    pub key: String,
    pub node: NodeId,
    pub enabled: bool,
}

#[derive(Clone)]
pub struct Collection {
    registry: Rc<RefCell<Registry>>,
    identity: Rc<()>,
    pulse: Cage<()>,
    owner: glory::core::InstanceId,
    limits: CollectionLimits,
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
impl Collection {
    pub fn new(scope: &Scope, limits: CollectionLimits) -> Result<Self> {
        if !(1..=1024).contains(&limits.items) || !(1..=4096).contains(&limits.label_bytes) {
            return Err(invalid("collection limits exceed the mounted budget"));
        }
        let registry = Rc::new(RefCell::new(Registry {
            entries: Vec::new(),
            next: 0,
            closed: false,
        }));
        let weak = Rc::downgrade(&registry);
        scope.on_cleanup(move || {
            if let Some(registry) = weak.upgrade() {
                let entries = {
                    let mut registry = registry.borrow_mut();
                    registry.closed = true;
                    std::mem::take(&mut registry.entries)
                };
                // Lotus/configuration destructors may reenter the application.
                drop(entries);
            }
        })?;
        Ok(Self {
            registry,
            identity: Rc::new(()),
            pulse: scope.cage(()),
            owner: scope.id(),
            limits,
        })
    }

    /// Register explicitly in the item's mounted Owner. The returned token does
    /// not own that Scope; item cleanup immediately removes the registration.
    pub fn register(
        &self,
        scope: &Scope,
        key: impl Into<String>,
        reference: Cage<Option<NodeId>>,
        enabled: impl Into<Lotus<bool>>,
        label: impl Into<Lotus<String>>,
    ) -> Result<CollectionMember> {
        self.register_inner(
            scope,
            key.into(),
            reference,
            enabled.into(),
            label.into(),
            None,
        )
    }

    /// Explicit keyed-row order. A reordered row keeps its registration and
    /// Owner while keyboard traversal observes its current logical position.
    pub fn register_ordered(
        &self,
        scope: &Scope,
        key: impl Into<String>,
        reference: Cage<Option<NodeId>>,
        enabled: impl Into<Lotus<bool>>,
        label: impl Into<Lotus<String>>,
        position: impl Into<Lotus<usize>>,
    ) -> Result<CollectionMember> {
        self.register_inner(
            scope,
            key.into(),
            reference,
            enabled.into(),
            label.into(),
            Some(position.into()),
        )
    }

    fn register_inner(
        &self,
        scope: &Scope,
        key: String,
        reference: Cage<Option<NodeId>>,
        enabled: Lotus<bool>,
        label: Lotus<String>,
        position: Option<Lotus<usize>>,
    ) -> Result<CollectionMember> {
        if key.len() > 64 {
            return Err(invalid("collection keys exceed 64 bytes"));
        }
        SemanticId::new(&key)?;
        if !self.owner.is_alive() || !scope.id().is_alive() {
            return Err(glory::ui::Error::Gone);
        }
        if self.owner.runtime() != scope.id().runtime() {
            return Err(glory::render::Error::ForeignRuntime.into());
        }
        label.try_with(|label| {
            if label.len() > self.limits.label_bytes {
                Err(invalid("collection label exceeds its byte budget"))
            } else {
                Ok(())
            }
        })??;
        if !self.owner.is_alive() || !scope.id().is_alive() {
            return Err(glory::ui::Error::Gone);
        }
        if reference
            .try_get()?
            .is_some_and(|node| node.runtime() != self.owner.runtime())
        {
            return Err(glory::render::Error::ForeignRuntime.into());
        }
        let member = {
            let mut registry = self.registry.borrow_mut();
            if registry.closed {
                return Err(glory::ui::Error::Gone);
            }
            if registry.entries.len() == self.limits.items
                || registry.entries.iter().any(|entry| entry.key == key)
            {
                return Err(invalid(
                    "duplicate collection key or mounted budget exhausted",
                ));
            }
            let serial = registry
                .next
                .checked_add(1)
                .ok_or_else(|| invalid("collection registration identity exhausted"))?;
            registry.next = serial;
            let member = CollectionMember {
                collection: self.identity.clone(),
                serial,
            };
            registry.entries.push(Entry {
                member: member.clone(),
                key,
                reference,
                enabled,
                label,
                position,
            });
            member
        };
        let weak = Rc::downgrade(&self.registry);
        let retired = member.clone();
        let pulse = self.pulse;
        let cleanup = scope.on_cleanup(move || {
            if let Some(registry) = weak.upgrade() {
                let removed = {
                    let mut registry = registry.borrow_mut();
                    registry
                        .entries
                        .iter()
                        .position(|entry| entry.member == retired)
                        .map(|index| registry.entries.remove(index))
                };
                let changed = removed.is_some();
                drop(removed);
                if changed {
                    let _ = pulse.try_revise(|_| ());
                }
            }
        });
        if let Err(error) = cleanup {
            let removed = {
                let mut registry = self.registry.borrow_mut();
                registry
                    .entries
                    .iter()
                    .position(|entry| entry.member == member)
                    .map(|index| registry.entries.remove(index))
            };
            drop(removed);
            return Err(error.into());
        }
        self.pulse.try_revise(|_| ())?;
        Ok(member)
    }

    fn entries(&self) -> Result<Vec<Entry>> {
        self.pulse.try_get()?;
        if !self.owner.is_alive() {
            return Err(glory::ui::Error::Gone);
        }
        let registry = self.registry.borrow();
        if registry.closed {
            return Err(glory::ui::Error::Gone);
        }
        Ok(registry.entries.clone())
    }
    pub fn mounted(&self) -> Result<Vec<MountedItem>> {
        let mut items = self
            .entries()?
            .into_iter()
            .enumerate()
            .filter_map(|(registered, entry)| match entry.reference.try_get() {
                Ok(Some(node)) => Some(
                    entry
                        .enabled
                        .try_get()
                        .and_then(|enabled| {
                            Ok((
                                entry
                                    .position
                                    .as_ref()
                                    .map(Lotus::try_get)
                                    .transpose()?
                                    .unwrap_or(registered),
                                registered,
                                MountedItem {
                                    member: entry.member,
                                    key: entry.key,
                                    node,
                                    enabled,
                                },
                            ))
                        })
                        .map_err(Into::into),
                ),
                Ok(None) => None,
                Err(error) => Some(Err(error.into())),
            })
            .collect::<Result<Vec<_>>>()?;
        items.sort_by_key(|(position, registered, _)| (*position, *registered));
        Ok(items.into_iter().map(|(_, _, item)| item).collect())
    }
    pub fn registered_len(&self) -> Result<usize> {
        self.pulse.try_get()?;
        if !self.owner.is_alive() {
            return Err(glory::ui::Error::Gone);
        }
        let registry = self.registry.borrow();
        if registry.closed {
            return Err(glory::ui::Error::Gone);
        }
        Ok(registry.entries.len())
    }
}

/// The tab entry is an intent. Actual focus is observed separately by the host.
#[derive(Clone)]
pub struct RovingFocus {
    collection: Collection,
    current: Cage<Option<CollectionMember>>,
    looping: bool,
}
impl RovingFocus {
    pub fn new(scope: &Scope, collection: Collection, looping: bool) -> Result<Self> {
        if !scope.id().is_alive() {
            return Err(glory::ui::Error::Gone);
        }
        if collection.owner.runtime() != scope.id().runtime() {
            return Err(glory::render::Error::ForeignRuntime.into());
        }
        collection.registered_len()?;
        Ok(Self {
            collection,
            current: scope.cage(None),
            looping,
        })
    }
    pub fn current(&self) -> Result<Option<CollectionMember>> {
        Ok(self.current.try_get()?)
    }
    pub fn is_tab_stop(&self, member: &CollectionMember) -> Result<bool> {
        Ok(self.current()?.as_ref() == Some(member))
    }
    pub fn observe(&self, member: &CollectionMember) -> Result {
        if self
            .collection
            .mounted()?
            .iter()
            .any(|item| &item.member == member && item.enabled)
        {
            self.current
                .try_revise(|current| *current = Some(member.clone()))?;
        }
        Ok(())
    }
    /// Preserve the current entry unless a preferred live enabled key is given
    /// or that entry retired/disabled. This never claims that focus moved.
    pub fn repair(&self, preferred: Option<&str>) -> Result {
        let items = self.collection.mounted()?;
        let current = self.current()?;
        let next = preferred
            .and_then(|key| items.iter().find(|item| item.key == key && item.enabled))
            .or_else(|| {
                items
                    .iter()
                    .find(|item| Some(&item.member) == current.as_ref() && item.enabled)
            })
            .or_else(|| items.iter().find(|item| item.enabled))
            .map(|item| item.member.clone());
        if next != current {
            self.current.try_revise(|current| *current = next)?;
        }
        Ok(())
    }
    pub fn destination(
        &self,
        from: NodeId,
        command: Navigation,
    ) -> Result<Option<MountedItem>> {
        let items = self.collection.mounted()?;
        let Some(current) = items.iter().position(|item| item.node == from) else {
            return Ok(None);
        };
        let flags = items.iter().map(|item| item.enabled).collect::<Vec<_>>();
        Ok(
            crate::composite::destination(current, command, &flags, self.looping)
                .map(|index| items[index].clone()),
        )
    }
    pub fn accept(&self, item: &MountedItem) -> Result {
        self.observe(&item.member)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TypeaheadConfig {
    pub timeout_millis: u64,
    pub query_bytes: usize,
}
impl Default for TypeaheadConfig {
    fn default() -> Self {
        Self {
            timeout_millis: 500,
            query_bytes: 64,
        }
    }
}
/// Only the active bounded query and its last explicit clock are retained.
pub struct Typeahead {
    config: TypeaheadConfig,
    query: String,
    last: Option<u64>,
}
impl Typeahead {
    pub fn new(config: TypeaheadConfig) -> Result<Self> {
        if !(1..=60_000).contains(&config.timeout_millis)
            || !(1..=256).contains(&config.query_bytes)
        {
            return Err(invalid("invalid typeahead timeout or query budget"));
        }
        Ok(Self {
            config,
            query: String::new(),
            last: None,
        })
    }
    pub fn query(&self) -> &str {
        &self.query
    }
    pub fn clear(&mut self) {
        self.query.clear();
        self.last = None;
    }
    pub fn search(
        &mut self,
        collection: &Collection,
        from: NodeId,
        text: &str,
        at_millis: u64,
    ) -> Result<Option<MountedItem>> {
        validate_typeahead(text, at_millis)?;
        if self.last.is_some_and(|last| at_millis < last) {
            return Err(invalid("typeahead clock regressed"));
        }
        let character = text.to_lowercase();
        let expired = self
            .last
            .is_none_or(|last| at_millis - last > self.config.timeout_millis);
        let repeated = !expired && self.query == character;
        let query = if expired || repeated {
            character
        } else {
            format!("{}{character}", self.query)
        };
        if query.len() > self.config.query_bytes {
            return Err(invalid("typeahead query budget exhausted"));
        }
        let entries = collection.entries()?;
        let mut candidates = Vec::with_capacity(entries.len());
        for entry in entries {
            let Some(node) = entry.reference.try_get()? else {
                continue;
            };
            let enabled = entry.enabled.try_get()?;
            let matches = entry.label.try_with(|label| {
                if label.len() > collection.limits.label_bytes {
                    return Err(invalid("collection label exceeds its byte budget"));
                }
                // Lowercase only this bounded mounted label, outside registry borrows.
                Ok(enabled && label.to_lowercase().starts_with(&query))
            })??;
            candidates.push((
                MountedItem {
                    member: entry.member,
                    key: entry.key,
                    node,
                    enabled,
                },
                matches,
            ));
        }
        // Invalid label/access/clock/budget leaves the previous query untouched.
        let Some(current) = candidates.iter().position(|(item, _)| item.node == from) else {
            return Ok(None);
        };
        let next = if expired || repeated {
            current + 1
        } else {
            current
        };
        let result = (0..candidates.len())
            .map(|offset| (next + offset) % candidates.len())
            .find(|index| candidates[*index].1)
            .map(|index| candidates[index].0.clone());
        self.query = query;
        self.last = Some(at_millis);
        Ok(result)
    }
}
