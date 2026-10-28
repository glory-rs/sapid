//! Mounted navigation links and disclosures; no menu roles or implicit routing.
use crate::appearance::{Common, configure, observe};
use crate::collection::{Collection, CollectionLimits, RovingFocus};
use crate::composite::{self, Callback, Observers};
use crate::state::{Control, DelayFactory, OwnerDelay, Value};
use glory::core::InstanceId;
use glory::render::{
    Action, Change, Direction, DismissReason, Event, EventKind, Layer, LayerFocus,
    LayerPlacement, NavigationPanel, NavigationPanelCommand, NodeId, Orientation, Patch,
    PointerPhase, Role, SemanticId, Semantics,
};
use glory::style::{Display, Edges, FlexDirection, Length, Style};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Bond, Cage, Scope, Vary};
use std::{
    cell::{Cell, RefCell},
    future::Future,
    rc::{Rc, Weak},
};

mod items;
pub use items::{
    NavigationMenuLink, NavigationMenuPanel, navigation_menu_link, navigation_menu_panel,
};
pub const MAX_NAVIGATION_MENU_ITEMS: usize = 128;
pub const MAX_NAVIGATION_MENU_DEPTH: usize = 8;
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn valid_key(key: &str) -> Result {
    if key.is_empty()
        || key.len() > 64
        || !key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        Err(invalid("navigation keys must be bounded ASCII tokens"))
    } else {
        Ok(())
    }
}
fn valid_path(path: &[String]) -> Result {
    if path.len() > MAX_NAVIGATION_MENU_DEPTH {
        return Err(invalid("navigation paths exceed eight levels"));
    }
    for key in path {
        valid_key(key)?
    }
    Ok(())
}
fn label(scope: &Scope, source: &Vary<String>) -> Result<Cage<String>> {
    composite::validated(scope, source, |label| {
        if label.is_empty() || label.len() > 4096 {
            Err(invalid("navigation labels require 1..=4096 bytes"))
        } else {
            Ok(())
        }
    })
}
type Factory = Box<dyn FnMut(NavigationMenuContext, &Scope) -> Result>;
#[derive(Clone)]
struct Entry {
    token: u64,
    key: String,
    parent: Option<u64>,
    owner: InstanceId,
    level: Rc<Level>,
    reference: Cage<Option<NodeId>>,
    panel: Option<Cage<Option<NodeId>>>,
    opened: Cage<bool>,
    trigger_id: SemanticId,
    panel_id: SemanticId,
    enabled: Vary<bool>,
}
struct Registry {
    entries: Vec<Entry>,
    focusables: Vec<Focusable>,
    next: u64,
    closed: bool,
}
#[derive(Clone)]
struct Focusable {
    token: u64,
    owner: InstanceId,
    reference: Cage<Option<NodeId>>,
}
struct Level {
    parent: Option<u64>,
    depth: usize,
    container: Cage<Option<NodeId>>,
    collection: Collection,
    roving: RovingFocus,
}
#[derive(Clone)]
enum Focus {
    First(u64),
    Last(u64),
    Trigger(u64),
}
struct FocusIntent {
    path: Vec<String>,
    focus: Focus,
}
#[derive(Clone)]
struct HoverIntent {
    path: Vec<String>,
    baseline: Vec<String>,
    close: bool,
    target: Option<u64>,
}
struct Root {
    owner: InstanceId,
    weak: WeakScope,
    live: Cell<bool>,
    namespace: String,
    control: Control<Vec<String>>,
    accepted: Cage<Vec<String>>,
    registry: RefCell<Registry>,
    pulse: Cage<()>,
    change: Callback<Vec<String>>,
    direction: Direction,
    hover: bool,
    delay: Rc<OwnerDelay>,
    factory: Option<DelayFactory>,
    hover_intent: RefCell<Option<HoverIntent>>,
    focus: RefCell<Option<FocusIntent>>,
    focus_pulse: Cage<()>,
    focused: Cell<Option<NodeId>>,
    focused_member: Cell<Option<u64>>,
    focus_loss: Cell<bool>,
}
impl Root {
    fn observe_focus(&self, node: NodeId, focused: bool, fallback: Option<u64>) -> Result {
        self.ensure()?;
        if focused {
            if self.focused.get() != Some(node) {
                let mut member = self
                    .snapshot()?
                    .iter()
                    .find(|entry| entry.reference.try_get().ok().flatten() == Some(node))
                    .map(|entry| entry.token)
                    .or(fallback);
                let focusables = self.registry.borrow().focusables.clone();
                if let Some(token) = focusables
                    .iter()
                    .filter(|item| item.owner.is_alive())
                    .find(|item| item.reference.try_get().ok().flatten() == Some(node))
                    .map(|item| item.token)
                {
                    member = Some(token)
                }
                self.focused.set(Some(node));
                self.focused_member.set(member);
            }
            self.focus_loss.set(false);
        } else if self.focused.get() == Some(node) {
            self.focused.set(None);
            self.focused_member.set(None);
            self.focus_loss.set(true);
        }
        self.focus_pulse.try_revise(|_| ())?;
        Ok(())
    }
    fn ensure(&self) -> Result {
        if self.live.get() && self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn report(&self, error: glory::ui::Error) {
        if let Some(scope) = self.weak.upgrade() {
            scope.report_error(error)
        }
    }
    fn snapshot(&self) -> Result<Vec<Entry>> {
        self.ensure()?;
        self.pulse.try_with(|_| ())?;
        Ok(self
            .registry
            .borrow()
            .entries
            .iter()
            .filter(|entry| entry.owner.is_alive())
            .cloned()
            .collect())
    }
    fn entry(&self, token: u64) -> Result<Entry> {
        self.snapshot()?
            .into_iter()
            .find(|entry| entry.token == token)
            .ok_or(glory::ui::Error::Gone)
    }
    fn path(&self, token: u64) -> Result<Vec<String>> {
        let entries = self.snapshot()?;
        let mut path = Vec::new();
        let mut cursor = Some(token);
        while let Some(token) = cursor {
            let entry = entries
                .iter()
                .find(|entry| entry.token == token)
                .ok_or(glory::ui::Error::Gone)?;
            path.push(entry.key.clone());
            cursor = entry.parent;
            if path.len() > MAX_NAVIGATION_MENU_DEPTH {
                return Err(invalid("navigation depth exceeded"));
            }
        }
        path.reverse();
        Ok(path)
    }
    fn live_path(&self) -> Result<Vec<String>> {
        let accepted = self.accepted.try_get()?;
        let entries = self.snapshot()?;
        let mut parent = None;
        let mut path = Vec::new();
        for key in accepted {
            let Some(entry) = entries.iter().find(|entry| {
                entry.parent == parent && entry.key == key && entry.panel.is_some()
            }) else {
                break;
            };
            path.push(key);
            parent = Some(entry.token)
        }
        Ok(path)
    }
    fn register(self: &Rc<Self>, scope: &Scope, mut entry: Entry) -> Result<Entry> {
        self.ensure()?;
        valid_key(&entry.key)?;
        if scope.id().runtime() != self.owner.runtime() {
            return Err(glory::render::Error::ForeignRuntime.into());
        }
        let token = {
            let mut registry = self.registry.borrow_mut();
            if registry.closed {
                return Err(glory::ui::Error::Gone);
            }
            if registry.entries.len() + registry.focusables.len() >= MAX_NAVIGATION_MENU_ITEMS {
                return Err(invalid("navigation has at most 128 live entries"));
            }
            if registry
                .entries
                .iter()
                .any(|other| other.parent == entry.parent && other.key == entry.key)
            {
                return Err(invalid("navigation sibling keys must be distinct"));
            }
            registry.next = registry
                .next
                .checked_add(1)
                .ok_or_else(|| invalid("navigation identity exhausted"))?;
            registry.next
        };
        entry.token = token;
        entry.trigger_id =
            SemanticId::new(format!("sapid-nav-{}-item-{token}", self.namespace))?;
        entry.panel_id =
            SemanticId::new(format!("sapid-nav-{}-panel-{token}", self.namespace))?;
        self.registry.borrow_mut().entries.push(entry.clone());
        self.pulse.try_revise(|_| ())?;
        let root = Rc::downgrade(self);
        scope.on_cleanup(move || {
            if let Some(root) = root.upgrade() {
                let removed = {
                    let mut registry = root.registry.borrow_mut();
                    registry
                        .entries
                        .iter()
                        .position(|entry| entry.token == token)
                        .map(|index| registry.entries.remove(index))
                };
                drop(removed);
                if root.focused_member.get() == Some(token) {
                    root.focused.set(None);
                    root.focused_member.set(None);
                    root.focus_loss.set(true);
                    let _ = root.focus_pulse.try_revise(|_| ());
                }
                let _ = root.pulse.try_revise(|_| ());
            }
        })?;
        Ok(entry)
    }
    fn cancel_hover(&self) {
        self.hover_intent.borrow_mut().take();
        self.delay.cancel()
    }
    fn register_focusable(
        self: &Rc<Self>,
        scope: &Scope,
        reference: Cage<Option<NodeId>>,
    ) -> Result {
        self.ensure()?;
        let token = {
            let mut registry = self.registry.borrow_mut();
            if registry.closed {
                return Err(glory::ui::Error::Gone);
            }
            if registry.entries.len() + registry.focusables.len() >= MAX_NAVIGATION_MENU_ITEMS {
                return Err(invalid("navigation has at most 128 live registrations"));
            }
            let token = registry
                .next
                .checked_add(1)
                .ok_or_else(|| invalid("navigation identity exhausted"))?;
            registry.next = token;
            registry.focusables.push(Focusable {
                token,
                owner: scope.id(),
                reference,
            });
            token
        };
        let weak = Rc::downgrade(self);
        scope.on_cleanup(move || {
            if let Some(root) = weak.upgrade() {
                let _removed = {
                    let mut registry = root.registry.borrow_mut();
                    registry
                        .focusables
                        .iter()
                        .position(|item| item.token == token)
                        .map(|index| registry.focusables.remove(index))
                };
                if root.focused_member.get() == Some(token) {
                    root.focused.set(None);
                    root.focused_member.set(None);
                    root.focus_loss.set(true);
                    let _ = root.focus_pulse.try_revise(|_| ());
                }
                let _ = root.pulse.try_revise(|_| ());
            }
        })?;
        self.pulse.try_revise(|_| ())?;
        Ok(())
    }
    fn retire(&self) {
        self.live.set(false);
        self.cancel_hover();
        self.focus.borrow_mut().take();
        self.change.clear();
        let entries = {
            let mut registry = self.registry.borrow_mut();
            registry.closed = true;
            (
                std::mem::take(&mut registry.entries),
                std::mem::take(&mut registry.focusables),
            )
        };
        drop(entries)
    }
    fn propose(&self, path: Vec<String>, focus: Option<Focus>) -> Result {
        self.ensure()?;
        valid_path(&path)?;
        self.cancel_hover();
        let same = self.live_path()? == path;
        let source_same = self
            .control
            .source
            .try_with(|source| source.len() <= MAX_NAVIGATION_MENU_DEPTH && source == &path)?;
        if !same || !source_same {
            self.control.accept(&path)?;
            self.change.emit(self.owner, path.clone());
        }
        if !self.live.get() || !self.owner.is_alive() {
            return Ok(());
        }
        let accepted = self
            .control
            .source
            .try_with(|source| valid_path(source).is_ok() && source == &path)?;
        *self.focus.borrow_mut() = if accepted {
            focus.map(|focus| FocusIntent { path, focus })
        } else {
            None
        };
        self.focus_pulse.try_revise(|_| ())?;
        Ok(())
    }
    fn open(&self, token: u64, last: bool, focus: bool) -> Result {
        let entry = self.entry(token)?;
        if entry.panel.is_none() || !entry.enabled.try_get()? {
            return Ok(());
        }
        self.propose(
            self.path(token)?,
            focus.then_some(if last {
                Focus::Last(token)
            } else {
                Focus::First(token)
            }),
        )
    }
    fn close(&self, token: u64, focus: bool) -> Result {
        let mut path = self.path(token)?;
        path.pop();
        self.propose(path, focus.then_some(Focus::Trigger(token)))
    }
    fn activate(&self, token: u64) -> Result {
        let entry = self.entry(token)?;
        if !entry.enabled.try_get()? {
            return Ok(());
        }
        if entry.panel.is_some() {
            if entry.opened.try_get()? {
                self.close(token, false)
            } else {
                self.open(token, false, false)
            }
        } else {
            self.propose(Vec::new(), None)
        }
    }
    fn panel_command(&self, entry: &Entry, command: NavigationPanelCommand) -> Result {
        match command {
            NavigationPanelCommand::First => self.open(entry.token, false, true),
            NavigationPanelCommand::Last => self.open(entry.token, true, true),
            NavigationPanelCommand::Open => self.open(entry.token, false, true),
            NavigationPanelCommand::Close => entry
                .parent
                .map_or(Ok(()), |parent| self.close(parent, true)),
        }
    }
    fn navigate(
        &self,
        level: &Level,
        from: NodeId,
        command: glory::render::Navigation,
    ) -> Result {
        self.ensure()?;
        self.cancel_hover();
        let Some(destination) = level.roving.destination(from, command)? else {
            return Ok(());
        };
        level.roving.accept(&destination)?;
        if let Some(scope) = self.weak.upgrade() {
            scope.action(Action::Focus(destination.node))?;
        }
        if level.parent.is_none() && !self.live_path()?.is_empty() {
            let entry = self.snapshot()?.into_iter().find(|entry| {
                entry.reference.try_get().ok().flatten() == Some(destination.node)
            });
            if let Some(entry) = entry {
                if entry.panel.is_some() {
                    self.open(entry.token, false, false)?
                } else {
                    self.propose(Vec::new(), None)?
                }
            }
        }
        Ok(())
    }
    fn hover(self: &Rc<Self>, path: Vec<String>, close: bool) -> Result {
        if !self.hover {
            return Ok(());
        }
        let baseline = self.live_path()?;
        if self
            .hover_intent
            .borrow()
            .as_ref()
            .is_some_and(|pending| pending.path == path && pending.close == close)
        {
            return Ok(());
        }
        if path == baseline {
            self.cancel_hover();
            return Ok(());
        }
        let mut parent = None;
        let entries = self.snapshot()?;
        for key in &path {
            let entry = entries
                .iter()
                .find(|entry| {
                    entry.parent == parent && entry.key == *key && entry.panel.is_some()
                })
                .ok_or(glory::ui::Error::Gone)?;
            parent = Some(entry.token)
        }
        let intent = HoverIntent {
            path,
            baseline,
            close,
            target: parent,
        };
        *self.hover_intent.borrow_mut() = Some(intent.clone());
        // The effect must subscribe to this intent's live member and source,
        // including when no hover existed during its initial evaluation.
        self.focus_pulse.try_revise(|_| ())?;
        let root = Rc::downgrade(self);
        let scope = self.weak.upgrade().ok_or(glory::ui::Error::Gone)?;
        self.delay.run(&scope, self.factory.as_ref(), move || {
            if let Some(root) = root.upgrade() {
                let pending = root.hover_intent.borrow_mut().take();
                if let Some(pending) = pending
                    && pending.path == intent.path
                    && root.live_path()? == pending.baseline
                    && root.control.source.try_with(|source| {
                        valid_path(source).is_ok() && source == &pending.baseline
                    })?
                    && pending.target.is_none_or(|token| {
                        root.entry(token).is_ok_and(|entry| {
                            entry.enabled.try_get().unwrap_or(false)
                                && root.path(token).is_ok_and(|path| path == pending.path)
                        })
                    })
                {
                    root.propose(pending.path, None)?;
                }
            }
            Ok(())
        })
    }
    fn metadata(&self, entry: &Entry, open: bool) -> Option<Box<Semantics>> {
        composite::metadata(Semantics {
            id: Some(entry.panel_id.clone()),
            hidden: !open,
            navigation: Some(Orientation::Vertical),
            navigation_direction: self.direction,
            navigation_panel: Some(NavigationPanel::Content),
            layer: open.then(|| Layer {
                modal: false,
                focus: LayerFocus::Preserve,
                placement: if entry.parent.is_none() {
                    if self.direction == Direction::Rtl {
                        LayerPlacement::Anchor(glory::render::AnchorPlacement::new(
                            glory::render::AnchorSide::Bottom,
                            glory::render::AnchorAlign::End,
                        ))
                    } else {
                        LayerPlacement::Anchor(glory::render::AnchorPlacement::new(
                            glory::render::AnchorSide::Bottom,
                            glory::render::AnchorAlign::Start,
                        ))
                    }
                } else if self.direction == Direction::Rtl {
                    LayerPlacement::Anchor(
                        glory::render::AnchorPlacement::new(
                            glory::render::AnchorSide::Left,
                            glory::render::AnchorAlign::Start,
                        )
                        .gap(4.0),
                    )
                } else {
                    LayerPlacement::Anchor(
                        glory::render::AnchorPlacement::new(
                            glory::render::AnchorSide::Right,
                            glory::render::AnchorAlign::Start,
                        )
                        .gap(4.0),
                    )
                },
                anchor: Some(entry.trigger_id.clone()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
    fn reconcile(&self) -> Result {
        self.ensure()?;
        let path = self.control.source.try_with(|path| {
            valid_path(path)?;
            Ok::<_, glory::ui::Error>(path.clone())
        })??;
        let mut entries = self.snapshot()?;
        let mut parent = None;
        for key in &path {
            let entry = entries
                .iter()
                .find(|entry| {
                    entry.parent == parent && entry.key == *key && entry.panel.is_some()
                })
                .ok_or_else(|| invalid("navigation path refers to an unknown mounted panel"))?;
            parent = Some(entry.token)
        }
        // Admit newly opening layers before closing the accepted branch. Native
        // rejection therefore preserves the closed/previous accepted display.
        entries.sort_by_key(|entry| {
            !self
                .path(entry.token)
                .is_ok_and(|entry_path| path.starts_with(&entry_path))
        });
        let scope = self.weak.upgrade().ok_or(glory::ui::Error::Gone)?;
        for entry in entries.iter().filter(|entry| entry.panel.is_some()) {
            let open = path.starts_with(&self.path(entry.token)?);
            if open == entry.opened.get_untracked() {
                continue;
            }
            let panel = entry
                .panel
                .unwrap()
                .try_get()?
                .ok_or(glory::ui::Error::Gone)?;
            scope.patch(
                panel,
                Patch::Semantics(Change::Set(self.metadata(entry, open))),
            )?;
            entry.opened.try_revise(|value| *value = open)?;
        }
        if self.accepted.get_untracked() != path {
            self.accepted.try_revise(|value| *value = path)?;
        }
        Ok(())
    }
    fn focus_pending(&self) -> Result {
        self.ensure()?;
        let pending = self.focus.borrow_mut().take();
        let Some(pending) = pending else {
            return Ok(());
        };
        if pending.path != self.live_path()? {
            return Ok(());
        }
        let (token, last, trigger) = match pending.focus {
            Focus::First(token) => (token, false, false),
            Focus::Last(token) => (token, true, false),
            Focus::Trigger(token) => (token, false, true),
        };
        let Ok(entry) = self.entry(token) else {
            return Ok(());
        };
        let target = if trigger {
            if entry.enabled.try_get()? {
                entry.reference.try_get()?
            } else {
                entry
                    .level
                    .collection
                    .mounted()?
                    .into_iter()
                    .find(|member| member.enabled)
                    .map(|member| member.node)
            }
        } else {
            let entries = self.snapshot()?;
            let level = entries
                .iter()
                .find(|child| child.parent == Some(token))
                .map(|child| child.level.clone());
            if let Some(level) = level {
                let mut members = level
                    .collection
                    .mounted()?
                    .into_iter()
                    .filter(|member| member.enabled);
                if last {
                    members.next_back()
                } else {
                    members.next()
                }
                .map(|member| member.node)
            } else {
                None
            }
        };
        if let Some(target) = target
            && let Some(scope) = self.weak.upgrade()
        {
            scope.action(Action::Focus(target))?
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct NavigationMenuContext {
    root: Weak<Root>,
    level: Rc<Level>,
    theme: crate::Theme,
    size: crate::Size,
    unstyled: bool,
    open: Bond<bool>,
}
impl NavigationMenuContext {
    /// Bind a custom control's focus lifetime to this adapter's Owner. Pass the
    /// same node_ref to the widget. This does not add a roving navigation item.
    pub fn track_focusable<W: Widget>(
        &self,
        reference: Cage<Option<NodeId>>,
        widget: W,
    ) -> NavigationFocusable<W> {
        NavigationFocusable {
            context: self.clone(),
            reference,
            widget: Some(widget),
        }
    }
    fn root(&self, scope: &Scope) -> Result<Rc<Root>> {
        let root = self.root.upgrade().ok_or(glory::ui::Error::Gone)?;
        root.ensure()?;
        let container = self
            .level
            .container
            .try_get()?
            .ok_or(glory::ui::Error::Gone)?;
        if !scope.is_within(container)? {
            return Err(invalid(
                "navigation context must be a live construction ancestor",
            ));
        }
        Ok(root)
    }
    /// Actual accepted visibility; suitable for explicitly conditional children.
    pub fn open(&self) -> Bond<bool> {
        self.open
    }
}
pub struct NavigationFocusable<W: Widget> {
    context: NavigationMenuContext,
    reference: Cage<Option<NodeId>>,
    widget: Option<W>,
}
impl<W: Widget> Widget for NavigationFocusable<W> {
    fn build(&mut self, scope: &Scope) -> Result {
        let root = self.context.root(scope)?;
        self.widget
            .take()
            .ok_or(glory::ui::Error::Gone)?
            .show_in(scope)?;
        let node = self
            .reference
            .try_get()?
            .ok_or_else(|| invalid("tracked navigation control must populate its node_ref"))?;
        let probe = scope.child_in(node)?;
        let container = self
            .context
            .level
            .container
            .try_get()?
            .ok_or(glory::ui::Error::Gone)?;
        if !probe.is_within(container)? {
            return Err(invalid(
                "tracked navigation control must belong to this content",
            ));
        }
        drop(probe);
        root.register_focusable(scope, self.reference)
    }
}
pub struct NavigationMenu {
    key: String,
    common: Common,
    path: Value<Vec<String>>,
    invalid_default: bool,
    content: Option<Factory>,
    changed: Option<Box<dyn FnMut(Vec<String>)>>,
    direction: Direction,
    hover: bool,
    delay: Option<DelayFactory>,
}
pub fn navigation_menu(key: impl Into<String>) -> NavigationMenu {
    NavigationMenu {
        key: key.into(),
        common: Common::default(),
        path: Value::default(),
        invalid_default: false,
        content: None,
        changed: None,
        direction: Direction::Ltr,
        hover: false,
        delay: None,
    }
}
impl NavigationMenu {
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn size(mut self, value: crate::Size) -> Self {
        self.common.size = value;
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn class(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Vary<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
    pub fn on(mut self, kind: EventKind, handler: impl FnMut(Event) + 'static) -> Self {
        self.common
            .handlers
            .entry(kind)
            .or_default()
            .push(Box::new(handler));
        self
    }
    pub fn direction(mut self, value: Direction) -> Self {
        self.direction = value;
        self
    }
    pub fn open_path(mut self, value: impl Into<Vary<Vec<String>>>) -> Self {
        self.path.source = Some(value.into());
        self
    }
    pub fn default_open_path<S: Into<String>>(
        mut self,
        values: impl IntoIterator<Item = S>,
    ) -> Self {
        self.invalid_default = false;
        self.path.default.clear();
        for value in values.into_iter().take(MAX_NAVIGATION_MENU_DEPTH + 1) {
            let value = value.into();
            if valid_key(&value).is_err() {
                self.invalid_default = true
            } else {
                self.path.default.push(value)
            }
        }
        self.invalid_default |= self.path.default.len() > MAX_NAVIGATION_MENU_DEPTH;
        self
    }
    pub fn on_open_path_change(mut self, callback: impl FnMut(Vec<String>) + 'static) -> Self {
        self.changed = Some(Box::new(callback));
        self
    }
    pub fn hover(mut self, value: bool) -> Self {
        self.hover = value;
        self
    }
    pub fn hover_delay_with<F: Fn() -> Fut + 'static, Fut: Future<Output = ()> + 'static>(
        mut self,
        factory: F,
    ) -> Self {
        self.hover = true;
        self.delay = Some(Rc::new(move || Box::pin(factory())));
        self
    }
    pub fn content<W: glory::ui::IntoChildren>(
        mut self,
        mut factory: impl FnMut(NavigationMenuContext) -> W + 'static,
    ) -> Self {
        self.content = Some(Box::new(move |context, scope| {
            glory::ui::group().fill(factory(context)).show_in(scope)
        }));
        self
    }
}
impl Widget for NavigationMenu {
    fn build(&mut self, scope: &Scope) -> Result {
        valid_key(&self.key)?;
        if self.invalid_default {
            return Err(invalid("invalid navigation default path"));
        }
        let control = std::mem::take(&mut self.path).mount(scope);
        control.source.try_with(|path| valid_path(path))??;
        let root = Rc::new(Root {
            owner: scope.id(),
            weak: scope.downgrade(),
            live: Cell::new(true),
            namespace: self.key.clone(),
            control,
            accepted: scope.cage(Vec::new()),
            registry: RefCell::new(Registry {
                entries: Vec::new(),
                focusables: Vec::new(),
                next: 0,
                closed: false,
            }),
            pulse: scope.cage(()),
            change: Callback::new(self.changed.take()),
            direction: self.direction,
            hover: self.hover,
            delay: OwnerDelay::new(scope)?,
            factory: self.delay.clone(),
            hover_intent: RefCell::new(None),
            focus: RefCell::new(None),
            focus_pulse: scope.cage(()),
            focused: Cell::new(None),
            focused_member: Cell::new(None),
            focus_loss: Cell::new(false),
        });
        let cleanup = Rc::downgrade(&root);
        scope.on_cleanup(move || {
            if let Some(root) = cleanup.upgrade() {
                root.retire()
            }
        })?;
        if self.common.name.is_none() {
            self.common.name = Some("Main navigation".into())
        }
        let outer_ref = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let mut outer = configure!(
            self.common,
            glory::ui::view().role(Role::Navigation),
            scope,
            "sapid-navigation-menu",
            Style::new()
        )
        .node_ref(outer_ref);
        let focus_observers = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::FocusWithin)
                .unwrap_or_default(),
        );
        let state = root.clone();
        let owner = scope.id();
        outer = outer.on(EventKind::FocusWithin, move |event| {
            if let Event::FocusWithin { node, focused } = event
                && let Err(error) = state.observe_focus(node, focused, None)
            {
                state.report(error);
                return;
            }
            focus_observers.emit(owner, event)
        });
        if self.hover {
            let observers = Observers::new(
                self.common
                    .handlers
                    .remove(&EventKind::PointerWithin)
                    .unwrap_or_default(),
            );
            let state = root.clone();
            outer = outer.on(EventKind::PointerWithin, move |event| {
                let result = match event {
                    Event::PointerWithin {
                        phase: PointerPhase::Leave,
                        ..
                    } => state.hover(Vec::new(), true),
                    Event::PointerWithin {
                        phase: PointerPhase::Move,
                        ..
                    } => {
                        let close = state
                            .hover_intent
                            .borrow()
                            .as_ref()
                            .is_some_and(|pending| pending.close);
                        if close {
                            state.cancel_hover()
                        }
                        Ok(())
                    }
                    _ => Ok(()),
                };
                if let Err(error) = result {
                    state.report(error);
                    return;
                }
                observers.emit(owner, event)
            });
        }
        let mut handlers = std::mem::take(&mut self.common.handlers);
        let navigate_observers =
            Observers::new(handlers.remove(&EventKind::Navigate).unwrap_or_default());
        for (kind, mut handlers) in handlers {
            outer = outer.on(kind, move |event| observe(&mut handlers, owner, event))
        }
        outer.show_in(scope)?;
        let parent = scope.child_in(outer_ref.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        let container = scope.cage(None);
        let collection = Collection::new(
            scope,
            CollectionLimits {
                items: MAX_NAVIGATION_MENU_ITEMS,
                label_bytes: 4096,
            },
        )?;
        let level = Rc::new(Level {
            parent: None,
            depth: 0,
            container,
            roving: RovingFocus::new(scope, collection.clone(), true)?,
            collection,
        });
        let state = root.clone();
        let current = level.clone();
        glory::ui::view()
            .role(Role::Group)
            .class("sapid-navigation-menu-list")
            .node_ref(container)
            .semantics(composite::metadata(Semantics {
                navigation: Some(Orientation::Horizontal),
                navigation_direction: self.direction,
                ..Default::default()
            }))
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .gap(4.0)
                    .margin(Edges::all(0.0))
                    .padding(Edges::all(0.0)),
            )
            .on(EventKind::Navigate, move |event| {
                if let Event::Navigate { from, command } = event
                    && let Err(error) = state.navigate(&current, from, command)
                {
                    state.report(error);
                    return;
                }
                navigate_observers.emit(owner, event)
            })
            .show_in(&parent)?;
        let list_scope = scope.child_in(container.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        let list_ref = scope.cage(None);
        glory::ui::view()
            .role(Role::List)
            .node_ref(list_ref)
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .gap(4.0)
                    .margin(Edges::all(0.0))
                    .padding(Edges::all(0.0)),
            )
            .show_in(&list_scope)?;
        let content_scope =
            scope.child_in(list_ref.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        let state = Rc::downgrade(&root);
        let open = scope.bond(move || {
            state
                .upgrade()
                .is_some_and(|root| root.live_path().is_ok_and(|path| !path.is_empty()))
        });
        if let Some(factory) = self.content.as_mut() {
            factory(
                NavigationMenuContext {
                    root: Rc::downgrade(&root),
                    level,
                    theme: self.common.theme.clone(),
                    size: self.common.size,
                    unstyled: self.common.unstyled,
                    open,
                },
                &content_scope,
            )?
        }
        root.reconcile()?;
        let state = root.clone();
        scope.structural(move || {
            if let Err(error) = state.reconcile() {
                state.report(error)
            }
        });
        let state = root.clone();
        scope.effect(move || {
            let _ = state.focus_pulse.try_with(|_| ());
            let _ = state.accepted.try_with(|_| ());
            let _ = state.pulse.try_with(|_| ());
            if let Some(node) = state.focused.get()
                && let Some(scope) = state.weak.upgrade()
            {
                match scope.is_active(node) {
                    Ok(true) => {}
                    // Accepted hiding is not a new user focus-loss request.
                    // The pending keyboard return target may now be revealed.
                    Ok(false) => {
                        state.focused.set(None);
                        state.focused_member.set(None);
                    }
                    Err(_) => {
                        state.focused.set(None);
                        state.focused_member.set(None);
                        state.focus_loss.set(true);
                    }
                }
            }
            if state.focus_loss.replace(false)
                && state.focused.get().is_none()
                && let Err(error) = state.propose(Vec::new(), None)
            {
                state.report(error)
            }
            let cancel = state.hover_intent.borrow().as_ref().is_some_and(|pending| {
                state.live_path().is_ok_and(|path| path != pending.baseline)
                    || !state
                        .control
                        .source
                        .try_with(|source| {
                            valid_path(source).is_ok() && source == &pending.baseline
                        })
                        .unwrap_or(false)
                    || pending
                        .target
                        .is_some_and(|token| state.entry(token).is_err())
            });
            if cancel {
                state.cancel_hover()
            }
            if let Err(error) = state.focus_pending() {
                state.report(error)
            }
        });
        scope.retain(parent)?;
        scope.retain(list_scope)?;
        scope.retain(content_scope)?;
        Ok(())
    }
}
