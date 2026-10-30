//! Bounded, live-target guided steps using genuine anchored Dialog layers.
use crate::{
    appearance::Common,
    composite::{self, Callback},
    state::{Control, Value},
};
use glory::render::{
    Action, AnchorPlacement, DismissReason, Event, EventKind, FocusMode, Layer, LayerPlacement,
    NodeId, Query, QueryToken, QueryValue, Role, SemanticId, Semantics, TextKind,
};
use glory::style::{Edges, Length, Style};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Scope, Vary};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::{Rc, Weak},
};

pub const MAX_TOUR_STEPS: usize = 64;
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn label(value: &str) -> Result {
    if value.len() > 1024 || value.trim().is_empty() || value.contains(['\0', '\r']) {
        return Err(invalid("Tour text requires 1..=1024 canonical bytes"));
    }
    Ok(())
}
#[derive(Clone)]
struct Entry {
    node: Option<NodeId>,
    owner: glory::core::InstanceId,
}
struct Registry {
    key: String,
    owner: glory::core::InstanceId,
    pulse: Cage<()>,
    entries: RefCell<BTreeMap<String, Entry>>,
    attached: Cell<bool>,
}
/// One application-owned namespace containing only its currently mounted targets.
#[derive(Clone)]
pub struct TourContext(Rc<Registry>);
pub fn tour_context(scope: &Scope, key: impl Into<String>) -> Result<TourContext> {
    let key = key.into();
    composite::id("tour", &key, "context", "root")?;
    let state = Rc::new(Registry {
        key,
        owner: scope.id(),
        pulse: scope.cage(()),
        entries: RefCell::new(BTreeMap::new()),
        attached: Cell::new(false),
    });
    let weak = Rc::downgrade(&state);
    scope.on_cleanup(move || {
        if let Some(state) = weak.upgrade() {
            let entries = std::mem::take(&mut *state.entries.borrow_mut());
            drop(entries);
        }
    })?;
    Ok(TourContext(state))
}
impl TourContext {
    pub fn len(&self) -> usize {
        self.0.entries.borrow().len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn ensure(&self, scope: &Scope) -> Result {
        if !self.0.owner.is_alive() {
            return Err(glory::ui::Error::Gone);
        }
        if self.0.owner.runtime() != scope.id().runtime() {
            return Err(glory::render::Error::ForeignRuntime.into());
        }
        Ok(())
    }
    fn changed(&self) -> Result {
        if self.0.owner.is_alive() {
            self.0.pulse.try_revise(|_| ())?;
        }
        Ok(())
    }
}
pub struct TourTarget {
    context: TourContext,
    key: String,
    child: Option<Box<dyn Widget>>,
}
pub fn tour_target(context: &TourContext, key: impl Into<String>) -> TourTarget {
    TourTarget {
        context: context.clone(),
        key: key.into(),
        child: None,
    }
}
impl TourTarget {
    pub fn fill(mut self, child: impl Widget) -> Self {
        self.child = Some(Box::new(child));
        self
    }
}
impl Widget for TourTarget {
    fn build(&mut self, scope: &Scope) -> Result {
        self.context.ensure(scope)?;
        let id = composite::id("tour", &self.context.0.key, "target", &self.key)?;
        {
            let mut entries = self.context.0.entries.borrow_mut();
            if entries.contains_key(&self.key) {
                return Err(invalid("duplicate live Tour target"));
            }
            if entries.len() >= MAX_TOUR_STEPS {
                return Err(invalid("Tour live target budget exceeded"));
            }
            entries.insert(
                self.key.clone(),
                Entry {
                    node: None,
                    owner: scope.id(),
                },
            );
        }
        let ctx = self.context.clone();
        let key = self.key.clone();
        let owner = scope.id();
        scope.on_cleanup(move || {
            let removed = {
                let mut entries = ctx.0.entries.borrow_mut();
                if entries.get(&key).is_some_and(|e| e.owner == owner) {
                    entries.remove(&key)
                } else {
                    None
                }
            };
            if removed.is_some() {
                let _ = ctx.changed();
            }
        })?;
        let reference = scope.cage(None);
        let mut target = glory::ui::view()
            .class("sapid-tour-target")
            .focus(FocusMode::Programmatic)
            .node_ref(reference)
            .semantics(composite::metadata(Semantics {
                id: Some(id.clone()),
                ..Default::default()
            }));
        let ctx = self.context.clone();
        let weak = scope.downgrade();
        target = target.on(EventKind::Resize, move |_| {
            if let Err(error) = ctx.changed()
                && let Some(scope) = weak.upgrade()
            {
                scope.report_error(error);
            }
        });
        if let Some(child) = self.child.take() {
            target = target.fill(child);
        }
        target.show_in(scope)?;
        let node = reference
            .try_get()?
            .ok_or_else(|| invalid("Tour target failed to mount"))?;
        self.context
            .0
            .entries
            .borrow_mut()
            .get_mut(&self.key)
            .expect("reserved Tour target")
            .node = Some(node);
        self.context.changed()?;
        Ok(())
    }
}
/// A static declaration references one current target key, never an unmounted row.
#[derive(Clone, Debug)]
pub struct TourStep {
    pub key: String,
    pub title: String,
    pub description: String,
}
pub fn tour_step(
    key: impl Into<String>,
    title: impl Into<String>,
    description: impl Into<String>,
) -> TourStep {
    TourStep {
        key: key.into(),
        title: title.into(),
        description: description.into(),
    }
}
struct State {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    context: TourContext,
    steps: Vec<TourStep>,
    current: Control<usize>,
    input_current: Vary<usize>,
    open: Control<bool>,
    visible: Cage<bool>,
    changed: Callback<usize>,
    opened: Callback<bool>,
    cancelled: Callback<DismissReason>,
    lost: Callback<usize>,
    complete: Callback<()>,
    task: RefCell<Option<glory::core::TaskToken>>,
    pending: Cell<Option<QueryToken>>,
    ticket: Cell<u64>,
    loss: Cell<bool>,
    shown: Cell<Option<(usize, NodeId)>>,
}
impl State {
    fn index(&self) -> Result<usize> {
        let index = self.input_current.try_get()?;
        if index >= self.steps.len() {
            return Err(invalid("Tour current step is outside its declaration"));
        }
        Ok(index)
    }
    fn entry(&self) -> Result<Option<Entry>> {
        let key = &self.steps[self.index()?].key;
        Ok(self
            .context
            .0
            .entries
            .borrow()
            .get(key)
            .filter(|entry| entry.node.is_some())
            .cloned())
    }
    fn ensure(&self) -> Result {
        if self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn report(&self, result: Result) {
        if let Err(error) = result
            && let Some(scope) = self.weak.upgrade()
        {
            scope.report_error(error);
        }
    }
    fn cancel_work(&self) {
        let task = self.task.borrow_mut().take();
        if let Some(task) = task {
            task.cancel();
        }
        if let Some(token) = self.pending.take()
            && let Some(scope) = self.weak.upgrade()
        {
            let _ = scope.cancel_request(token);
        }
    }
    fn hide(&self) -> Result {
        self.shown.set(None);
        if self.visible.try_get()? {
            self.visible.try_revise(|visible| *visible = false)?;
        }
        Ok(())
    }
    fn missing(&self) -> Result {
        self.hide()?;
        if !self.loss.replace(true) {
            self.lost.emit(self.owner, self.index()?);
        }
        Ok(())
    }
    fn refresh(self: &Rc<Self>) -> Result {
        self.ensure()?;
        self.cancel_work();
        let current = self.index()?;
        if self.shown.get()
            != self
                .entry()?
                .and_then(|entry| entry.node.map(|node| (current, node)))
        {
            self.hide()?;
        }
        let ticket = self
            .ticket
            .get()
            .checked_add(1)
            .ok_or_else(|| invalid("Tour request identity exhausted"))?;
        self.ticket.set(ticket);
        if !self.open.source.try_get()? {
            self.hide()?;
            self.loss.set(false);
            return Ok(());
        }
        let Some(scope) = self.weak.upgrade() else {
            return Ok(());
        };
        let caps = scope.capabilities()?;
        // Deterministic construction/adoption: first observation always runs in a ready host task.
        if !caps.measure || !caps.scroll {
            return Ok(());
        }
        let weak = Rc::downgrade(self);
        let task = scope.spawn(async move {
            if let Some(state) = weak.upgrade() {
                let _finished = state.task.borrow_mut().take();
                state.report(state.measure(ticket));
            }
        })?;
        *self.task.borrow_mut() = Some(task);
        Ok(())
    }
    fn measure(self: &Rc<Self>, ticket: u64) -> Result {
        if self.ticket.get() != ticket || !self.open.source.try_get()? {
            return Ok(());
        }
        let Some(entry) = self.entry()? else {
            return self.missing();
        };
        let node = entry.node.expect("mounted Tour target");
        let Some(scope) = self.weak.upgrade() else {
            return Ok(());
        };
        if !entry.owner.is_alive() || !scope.is_active(node)? || scope.is_inert(node)? {
            return self.missing();
        }
        scope.action(Action::ScrollIntoView(node))?;
        let weak = Rc::downgrade(self);
        let index = self.index()?;
        let token = scope.request(Query::Bounds(node), move |reply| {
            let Some(state) = weak.upgrade() else { return; };
            if !state.owner.is_alive() || state.ticket.get() != ticket { return; }
            state.pending.set(None);
            state.report((|| {
                if !state.open.source.try_get()? || state.index()? != index || state.entry()?.is_none_or(|e| e.node != entry.node) { return Ok(()); }
                let Some(scope) = state.weak.upgrade() else { return Ok(()); };
                let visible = matches!(reply, Ok(QueryValue::Bounds(rect)) if rect.width > 0. && rect.height > 0.)
                    && scope.is_active(node)? && !scope.is_inert(node)?;
                if visible {
                    state.loss.set(false);
                    state.shown.set(Some((index, node)));
                    if !state.visible.try_get()? { state.visible.try_revise(|value| *value = true)?; }
                    Ok(())
                }
                else { state.missing() }
            })());
        })?;
        self.pending.set(Some(token));
        Ok(())
    }
    fn select(self: &Rc<Self>, index: usize) -> Result {
        self.ensure()?;
        if index >= self.steps.len() {
            return Err(invalid("Tour step is outside its declaration"));
        }
        if !self.open.source.try_get()? {
            return Ok(());
        }
        if self.index()? != index {
            self.current.accept(&index)?;
            self.changed.emit(self.owner, index);
        }
        Ok(())
    }
    fn close(&self, reason: Option<DismissReason>, finished: bool) -> Result {
        self.ensure()?;
        if !self.open.source.try_get()? {
            return Ok(());
        }
        self.open.accept(&false)?;
        if let Some(reason) = reason {
            self.cancelled.emit(self.owner, reason);
        }
        if finished {
            self.complete.emit(self.owner, ());
        }
        self.opened.emit(self.owner, false);
        Ok(())
    }
}
#[derive(Clone)]
pub struct TourHandle(Weak<State>);
impl TourHandle {
    fn state(&self) -> Result<Rc<State>> {
        self.0
            .upgrade()
            .filter(|state| state.owner.is_alive())
            .ok_or(glory::ui::Error::Gone)
    }
    pub fn refresh(&self) -> Result {
        self.state()?.refresh()
    }
    pub fn go_to(&self, index: usize) -> Result {
        self.state()?.select(index)
    }
    pub fn next(&self) -> Result {
        let state = self.state()?;
        let index = state.index()?;
        if index + 1 == state.steps.len() {
            state.close(None, true)
        } else {
            state.select(index + 1)
        }
    }
    pub fn previous(&self) -> Result {
        let state = self.state()?;
        state.select(state.index()?.saturating_sub(1))
    }
    pub fn cancel(&self) -> Result {
        self.state()?.close(Some(DismissReason::Gesture), false)
    }
}
pub struct Tour {
    key: String,
    context: TourContext,
    common: Common,
    steps: Vec<TourStep>,
    overflow: bool,
    current: Value<usize>,
    open: Value<bool>,
    placement: AnchorPlacement,
    changed: Option<Box<dyn FnMut(usize)>>,
    opened: Option<Box<dyn FnMut(bool)>>,
    cancelled: Option<Box<dyn FnMut(DismissReason)>>,
    lost: Option<Box<dyn FnMut(usize)>>,
    complete: Option<Box<dyn FnMut(())>>,
    ready: Option<Box<dyn FnMut(TourHandle)>>,
}
pub fn tour(context: &TourContext, key: impl Into<String>) -> Tour {
    Tour {
        key: key.into(),
        context: context.clone(),
        common: Common::default(),
        steps: Vec::new(),
        overflow: false,
        current: Value::default(),
        open: Value::default(),
        placement: AnchorPlacement::default(),
        changed: None,
        opened: None,
        cancelled: None,
        lost: None,
        complete: None,
        ready: None,
    }
}
impl Tour {
    pub fn step(mut self, step: TourStep) -> Self {
        if self.steps.len() < MAX_TOUR_STEPS
            && step.key.len() <= 64
            && SemanticId::new(&step.key).is_ok()
            && label(&step.title).is_ok()
            && label(&step.description).is_ok()
        {
            self.steps.push(step);
        } else {
            self.overflow = true;
        }
        self
    }
    pub fn steps(mut self, steps: impl IntoIterator<Item = TourStep>) -> Self {
        for step in steps {
            if self.steps.len() == MAX_TOUR_STEPS {
                self.overflow = true;
                break;
            }
            self = self.step(step);
            if self.overflow {
                break;
            }
        }
        self
    }
    pub fn current(mut self, value: impl Into<Vary<usize>>) -> Self {
        self.current.source = Some(value.into());
        self
    }
    pub fn default_current(mut self, value: usize) -> Self {
        self.current.default = value;
        self
    }
    pub fn open(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.open.source = Some(value.into());
        self
    }
    pub fn default_open(mut self, value: bool) -> Self {
        self.open.default = value;
        self
    }
    pub fn placement(mut self, value: AnchorPlacement) -> Self {
        self.placement = value;
        self
    }
    pub fn on_current_change(mut self, value: impl FnMut(usize) + 'static) -> Self {
        self.changed = Some(Box::new(value));
        self
    }
    pub fn on_open_change(mut self, value: impl FnMut(bool) + 'static) -> Self {
        self.opened = Some(Box::new(value));
        self
    }
    pub fn on_cancel(mut self, value: impl FnMut(DismissReason) + 'static) -> Self {
        self.cancelled = Some(Box::new(value));
        self
    }
    pub fn on_target_lost(mut self, value: impl FnMut(usize) + 'static) -> Self {
        self.lost = Some(Box::new(value));
        self
    }
    pub fn on_complete(mut self, mut value: impl FnMut() + 'static) -> Self {
        self.complete = Some(Box::new(move |_| value()));
        self
    }
    pub fn on_ready(mut self, value: impl FnMut(TourHandle) + 'static) -> Self {
        self.ready = Some(Box::new(value));
        self
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
}
impl Widget for Tour {
    fn build(&mut self, scope: &Scope) -> Result {
        self.context.ensure(scope)?;
        if self.overflow || self.steps.is_empty() {
            return Err(invalid("Tour requires 1..=64 steps"));
        }
        composite::validate_keys(&self.steps.iter().map(|s| s.key.clone()).collect::<Vec<_>>())?;
        for step in &self.steps {
            label(&step.title)?;
            label(&step.description)?;
        }
        self.placement.validate()?;
        let id = composite::id("tour", &self.key, "panel", "root")?;
        let title_id = composite::id("tour", &self.key, "title", "root")?;
        let desc_id = composite::id("tour", &self.key, "description", "root")?;
        let count = self.steps.len();
        let mut current = std::mem::take(&mut self.current).mount(scope);
        let input_current = current.source.clone();
        current.source = composite::validated(scope, &current.source, move |index| {
            if *index >= count {
                Err(invalid("Tour current step is outside its declaration"))
            } else {
                Ok(())
            }
        })?
        .into();
        let open = std::mem::take(&mut self.open).mount(scope);
        if self.context.0.attached.replace(true) {
            return Err(invalid("Tour context already has a live controller"));
        }
        let state = Rc::new(State {
            owner: scope.id(),
            weak: scope.downgrade(),
            context: self.context.clone(),
            steps: std::mem::take(&mut self.steps),
            current,
            input_current,
            open,
            visible: scope.cage(false),
            changed: Callback::new(self.changed.take()),
            opened: Callback::new(self.opened.take()),
            cancelled: Callback::new(self.cancelled.take()),
            lost: Callback::new(self.lost.take()),
            complete: Callback::new(self.complete.take()),
            task: RefCell::new(None),
            pending: Cell::new(None),
            ticket: Cell::new(0),
            loss: Cell::new(false),
            shown: Cell::new(None),
        });
        let weak = Rc::downgrade(&state);
        let ctx = self.context.clone();
        scope.on_cleanup(move || {
            ctx.0.attached.set(false);
            if let Some(state) = weak.upgrade() {
                state.cancel_work();
            }
        })?;
        let s = state.clone();
        let placement = self.placement;
        let labelled = title_id.clone();
        let described = desc_id.clone();
        let mut panel = glory::ui::view()
            .class("sapid-tour sapid-layer")
            .role(Role::Dialog)
            .focus(FocusMode::Programmatic)
            .semantics(scope.bond(move || {
                s.context.0.pulse.get();
                let target_key = &s.steps[s.current.source.get()].key;
                let anchor = composite::id("tour", &s.context.0.key, "target", target_key)
                    .expect("validated Tour target key");
                let entry = s.context.0.entries.borrow().get(target_key).cloned();
                composite::metadata(Semantics {
                    id: Some(id.clone()),
                    labelled_by: Some(labelled.clone()),
                    described_by: Some(described.clone()),
                    hidden: !s.open.source.get()
                        || !s.visible.get()
                        || entry.is_none_or(|e| e.node.is_none()),
                    layer: Some(Layer {
                        modal: false,
                        outside: false,
                        placement: LayerPlacement::Anchor(placement),
                        anchor: Some(anchor.clone()),
                        return_focus: Some(anchor),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            }));
        let s = state.clone();
        panel = panel.on(EventKind::Dismiss, move |event| {
            if let Event::Dismiss(reason) = event {
                if reason == DismissReason::AnchorLost {
                    s.report(s.missing());
                } else {
                    s.report(s.close(Some(reason), false));
                }
            }
        });
        if !self.common.unstyled {
            let mut style = Style::new()
                .width(Length::Px(320.))
                .padding(Edges::all(16.))
                .radius(self.common.theme.radius);
            style.background = Some(self.common.theme.surface);
            style.color = Some(self.common.theme.foreground);
            panel = panel.style(style);
        }
        let s = state.clone();
        let title = scope.bond(move || s.steps[s.current.source.get()].title.clone());
        let s = state.clone();
        let description = scope.bond(move || s.steps[s.current.source.get()].description.clone());
        let s = state.clone();
        let progress =
            scope.bond(move || format!("Step {} of {}", s.current.source.get() + 1, s.steps.len()));
        let h = TourHandle(Rc::downgrade(&state));
        let previous = h.clone();
        let next = h.clone();
        let cancel = h.clone();
        let s = state.clone();
        let previous_enabled = scope.bond(move || s.current.source.get() > 0);
        let s = state.clone();
        let next_label = scope.bond(move || {
            if s.current.source.get() + 1 == s.steps.len() {
                "Finish"
            } else {
                "Next step"
            }
            .to_string()
        });
        let weak = scope.downgrade();
        let previous_scope = weak.clone();
        let next_scope = weak.clone();
        panel
            .fill((
                glory::ui::text()
                    .kind(TextKind::Heading(2))
                    .content(title)
                    .semantics(composite::metadata(Semantics {
                        id: Some(title_id),
                        ..Default::default()
                    })),
                glory::ui::text()
                    .content(description)
                    .semantics(composite::metadata(Semantics {
                        id: Some(desc_id),
                        ..Default::default()
                    })),
                glory::ui::text()
                    .class("sapid-tour-progress")
                    .content(progress),
                glory::ui::button()
                    .label("Previous step")
                    .enabled(previous_enabled)
                    .on(EventKind::Activate, move |_| {
                        if let Err(error) = previous.previous()
                            && let Some(scope) = previous_scope.upgrade()
                        {
                            scope.report_error(error);
                        }
                    }),
                glory::ui::button()
                    .label(next_label)
                    .on(EventKind::Activate, move |_| {
                        if let Err(error) = next.next()
                            && let Some(scope) = next_scope.upgrade()
                        {
                            scope.report_error(error);
                        }
                    }),
                glory::ui::button()
                    .label("Cancel tour")
                    .on(EventKind::Activate, move |_| {
                        if let Err(error) = cancel.cancel()
                            && let Some(scope) = weak.upgrade()
                        {
                            scope.report_error(error);
                        }
                    }),
            ))
            .show_in(scope)?;
        let s = state.clone();
        scope.effect(move || {
            s.context.0.pulse.get();
            s.current.source.get();
            s.open.source.get();
            glory::core::untracked(|| s.report(s.refresh()));
        });
        if let Some(mut ready) = self.ready.take() {
            ready(h);
        }
        scope.retain(state)?;
        Ok(())
    }
}
