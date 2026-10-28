//! Bounded, measured inline widgets moved into the existing Popover host.
use crate::appearance::{Common, common_methods, configure, merge, observe};
use crate::composite::{self, Callback, ContentPolicy};
use crate::state::{Control, Value};
use glory::render::{
    Action, Change, Event, EventKind, NodeId, Patch, Query, QueryToken, QueryValue, Semantics,
};
use glory::style::{
    Align, Display, FlexDirection, FlexWrap, Length, Limit, Overflow, PointerEvents, Position,
    Style, WhiteSpace,
};
use glory::ui::{Mounted, Result, WeakScope, Widget};
use glory::{Bond, Cage, Scope, Vary};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

mod model;
pub use model::{MAX_OVERFLOW_ITEMS, MAX_OVERFLOW_QUERIES};
type Factory = Box<dyn FnMut() -> Box<dyn Widget>>;
type KeyFactory = Box<dyn FnMut(&str) -> OverflowItem>;
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn keys(keys: &[String]) -> Result {
    if keys.len() > MAX_OVERFLOW_ITEMS {
        return Err(invalid("overflow list exceeds 128 registered items"));
    }
    composite::validate_keys(keys)
}

/// A current atomic inline item. Intrinsic children must not depend on the
/// available inline width. Fixed pixel width is available for flexible editors.
pub struct OverflowItem {
    key: String,
    content: Option<Factory>,
    width: Option<Vary<f64>>,
}
pub fn overflow_item(key: impl Into<String>) -> OverflowItem {
    OverflowItem {
        key: key.into(),
        content: None,
        width: None,
    }
}
impl OverflowItem {
    pub fn content<W: Widget>(mut self, mut factory: impl FnMut() -> W + 'static) -> Self {
        self.content = Some(Box::new(move || Box::new(factory())));
        self
    }
    /// An explicit finite pixel box; percentage/fr sizing is not an intrinsic
    /// measurement policy and is intentionally absent from this API.
    pub fn width(mut self, value: impl Into<Vary<f64>>) -> Self {
        self.width = Some(value.into());
        self
    }
}

pub struct OverflowList {
    common: Common,
    key: String,
    items: Vec<OverflowItem>,
    dynamic: Option<(Vary<Vec<String>>, KeyFactory)>,
    initial: usize,
    visible: Option<Vary<usize>>,
    changed: Option<Box<dyn FnMut(usize)>>,
    gap: Vary<f64>,
    label: Vary<String>,
    open: Value<bool>,
    open_changed: Option<Box<dyn FnMut(bool)>>,
    revision: Vary<u64>,
    popup: bool,
    handle: Option<Cage<Option<OverflowHandle>>>,
}
pub fn overflow_list(key: impl Into<String>) -> OverflowList {
    OverflowList {
        common: Common::default(),
        key: key.into(),
        items: Vec::new(),
        dynamic: None,
        initial: 3,
        visible: None,
        changed: None,
        gap: 8.0.into(),
        label: "More items".into(),
        open: Value::default(),
        open_changed: None,
        revision: 0u64.into(),
        popup: true,
        handle: None,
    }
}
impl OverflowList {
    common_methods!();
    pub fn item(mut self, item: OverflowItem) -> Self {
        if self.items.len() <= MAX_OVERFLOW_ITEMS {
            self.items.push(item)
        }
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = OverflowItem>) -> Self {
        let remaining = (MAX_OVERFLOW_ITEMS + 1).saturating_sub(self.items.len());
        self.items.extend(items.into_iter().take(remaining));
        self
    }
    /// Keys describe only the bounded current registrations, not a business
    /// collection to scan/window. Retained keys keep their actual widget Owner.
    pub fn dynamic_items(
        mut self,
        keys: impl Into<Vary<Vec<String>>>,
        factory: impl FnMut(&str) -> OverflowItem + 'static,
    ) -> Self {
        self.dynamic = Some((keys.into(), Box::new(factory)));
        self
    }
    /// Deterministic SSR/client-first-shape prefix before physical observation.
    pub fn initial_visible(mut self, value: usize) -> Self {
        self.initial = value;
        self
    }
    pub fn visible_count(mut self, value: impl Into<Vary<usize>>) -> Self {
        self.visible = Some(value.into());
        self
    }
    pub fn on_visible_count_change(mut self, callback: impl FnMut(usize) + 'static) -> Self {
        self.changed = Some(Box::new(callback));
        self
    }
    pub fn gap(mut self, value: impl Into<Vary<f64>>) -> Self {
        self.gap = value.into();
        self
    }
    pub fn overflow_label(mut self, value: impl Into<Vary<String>>) -> Self {
        self.label = value.into();
        self
    }
    pub fn open(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.open.source = Some(value.into());
        self
    }
    pub fn on_open_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.open_changed = Some(Box::new(callback));
        self
    }
    /// Refresh hidden intrinsic content after its application-owned text/font
    /// inputs change; ordinary visible geometry and container Resize are observed.
    pub fn measurement_revision(mut self, value: impl Into<Vary<u64>>) -> Self {
        self.revision = value.into();
        self
    }
    /// Explicitly omit overflow presentation. This renders one ordinary inline
    /// row on native; it does not advertise a native top-layer implementation.
    pub fn overflow(mut self, value: bool) -> Self {
        self.popup = value;
        self
    }
    pub fn handle_ref(mut self, value: Cage<Option<OverflowHandle>>) -> Self {
        self.handle = Some(value);
        self
    }
}

struct Row {
    key: String,
    mounted: Mounted,
    node: NodeId,
    parent: Cell<NodeId>,
}
struct Batch {
    ticket: u64,
    tokens: Vec<QueryToken>,
    remaining: usize,
    available: Option<f64>,
    trigger: Option<f64>,
    widths: Vec<Option<f64>>,
    repeat: bool,
}
struct Controller {
    weak: WeakScope,
    owner: glory::core::InstanceId,
    live: Cell<bool>,
    popup: bool,
    line: Cell<Option<NodeId>>,
    strip: Cell<Option<NodeId>>,
    lane: Cell<Option<NodeId>>,
    panel: Cell<Option<NodeId>>,
    trigger: Cage<Option<NodeId>>,
    more_box: Cell<Option<NodeId>>,
    rows: RefCell<Vec<Rc<Row>>>,
    batch: RefCell<Option<Batch>>,
    next: Cell<u64>,
    task: RefCell<Option<glory::core::TaskToken>>,
    scheduled: Cell<bool>,
    building: Cell<bool>,
    focus_task: RefCell<Option<glory::core::TaskToken>>,
    focus_query: Cell<Option<QueryToken>>,
    focus_pending: Cell<bool>,
    refresh_after_focus: Cell<bool>,
    visible: Control<usize>,
    open: Control<bool>,
    gap: Cage<f64>,
    enabled: Vary<bool>,
    count: Cage<usize>,
    total: Cage<usize>,
    more_visible: Cage<bool>,
    more_focus: Cell<bool>,
    handoff: Cell<bool>,
    focus: RefCell<Option<(String, NodeId)>>,
    last_width: Cell<Option<f64>>,
    last_trigger: Cell<Option<f64>>,
    changed: Callback<usize>,
    open_changed: Callback<bool>,
    pending: Cage<usize>,
}
#[derive(Clone)]
pub struct OverflowHandle {
    inner: Rc<Controller>,
    visible: Bond<usize>,
    hidden: Bond<usize>,
    pending: Bond<usize>,
}
impl OverflowHandle {
    pub fn visible_count(&self) -> Bond<usize> {
        self.visible
    }
    pub fn overflow_count(&self) -> Bond<usize> {
        self.hidden
    }
    pub fn pending_queries(&self) -> Bond<usize> {
        self.pending
    }
    pub fn refresh(&self) -> Result {
        self.inner.schedule()
    }
    pub fn set_open(&self, open: bool) -> Result {
        self.inner.set_open(open)
    }
}
impl Controller {
    fn sync_pending(&self) -> Result {
        let count = self
            .batch
            .borrow()
            .as_ref()
            .map_or(0, |work| work.remaining)
            + usize::from(self.focus_query.get().is_some());
        if count > MAX_OVERFLOW_QUERIES {
            return Err(invalid("overflow pending query budget exceeded"));
        }
        if glory::core::untracked(|| self.pending.try_get())? != count {
            self.pending.try_revise(|value| *value = count)?;
        }
        Ok(())
    }
    fn scope(&self) -> Result<Scope> {
        if !self.live.get() || !self.owner.is_alive() {
            return Err(glory::ui::Error::Gone);
        }
        self.weak.upgrade().ok_or(glory::ui::Error::Gone)
    }
    fn report(&self, error: glory::ui::Error) {
        if let Some(scope) = self.weak.upgrade() {
            scope.report_error(error)
        }
    }
    fn cancel(&self) {
        let task = self.task.borrow_mut().take();
        self.scheduled.set(false);
        if let Some(task) = task {
            task.cancel();
        }
        let work = self.batch.borrow_mut().take();
        if let (Some(work), Some(scope)) = (work, self.weak.upgrade()) {
            for token in work.tokens {
                let _ = scope.cancel_request(token);
            }
        }
        let _ = self.sync_pending();
    }
    fn retire(&self) {
        self.live.set(false);
        self.cancel();
        self.cancel_focus();
        self.changed.clear();
        self.open_changed.clear();
        let rows = self.rows.replace(Vec::new());
        drop(rows);
        self.focus.borrow_mut().take();
    }
    fn schedule(self: &Rc<Self>) -> Result {
        let scope = self.scope()?;
        if !self.popup || !scope.capabilities()?.measure || !self.enabled.try_get()? {
            return Ok(());
        }
        if self.focus_pending.get() {
            self.refresh_after_focus.set(true);
            return Ok(());
        }
        if let Some(batch) = self.batch.borrow_mut().as_mut() {
            batch.repeat = true;
            return Ok(());
        }
        if self.scheduled.replace(true) {
            return Ok(());
        }
        let weak = Rc::downgrade(self);
        let token = scope.spawn(async move {
            if let Some(state) = weak.upgrade() {
                state.scheduled.set(false);
                let _completed = state.task.borrow_mut().take();
                if let Err(error) = state.measure() {
                    state.cancel();
                    let _ = state.reconcile(false);
                    state.report(error)
                }
            }
        })?;
        *self.task.borrow_mut() = Some(token);
        Ok(())
    }
    fn set_open(self: &Rc<Self>, open: bool) -> Result {
        self.scope()?;
        if open
            && (self.count.try_get()? >= self.total.try_get()? || !self.enabled.try_get()?)
        {
            return Ok(());
        }
        if self.open.source.try_get()? == open {
            return Ok(());
        }
        self.open.accept(&open)?;
        self.open_changed.emit(self.owner, open);
        Ok(())
    }
    fn cancel_focus(&self) {
        self.focus_pending.set(false);
        if let Some(task) = self.focus_task.borrow_mut().take() {
            task.cancel();
        }
        if let (Some(token), Some(scope)) = (self.focus_query.take(), self.weak.upgrade()) {
            let _ = scope.cancel_request(token);
        }
        let _ = self.sync_pending();
    }
    fn confirm_focus(self: &Rc<Self>, target: NodeId) -> Result {
        self.cancel_focus();
        self.focus_pending.set(true);
        let scope = self.scope()?;
        let weak = Rc::downgrade(self);
        let task = scope.spawn(async move {
            let Some(state) = weak.upgrade() else { return };
            let _completed = state.focus_task.borrow_mut().take();
            if !state.focus_pending.get() {
                return;
            }
            let result = (|| -> Result {
                let scope = state.scope()?;
                let reply = Rc::downgrade(&state);
                let token = scope.request(Query::Focus(target), move |result| {
                    if let Some(state) = reply.upgrade() {
                        state.focus_query.set(None);
                        state.focus_pending.set(false);
                        state.handoff.set(false);
                        state
                            .more_focus
                            .set(matches!(result, Ok(QueryValue::Focus(true))));
                        let _ = state.sync_pending();
                        if let Err(error) = state.reconcile(false) {
                            state.report(error)
                        }
                        if state.refresh_after_focus.replace(false)
                            && let Err(error) = state.schedule()
                        {
                            state.report(error)
                        }
                    }
                })?;
                state.focus_query.set(Some(token));
                state.sync_pending()?;
                Ok(())
            })();
            if let Err(error) = result {
                state.cancel_focus();
                state.handoff.set(false);
                state.report(error)
            }
        })?;
        *self.focus_task.borrow_mut() = Some(task);
        Ok(())
    }
    fn reconcile(self: &Rc<Self>, force: bool) -> Result {
        let scope = self.scope()?;
        let rows = self.rows.borrow().clone();
        let count = if self.popup {
            self.visible.source.try_get()?.min(rows.len())
        } else {
            rows.len()
        };
        if self.popup && count == rows.len() {
            self.open.accept(&false)?;
        }
        let open = self.popup && self.open.source.try_get()? && count < rows.len();
        let focused = self.focus.borrow().clone();
        let lost = focused
            .as_ref()
            .is_some_and(|(key, _)| !rows.iter().any(|row| &row.key == key));
        let hidden = focused.as_ref().is_some_and(|(key, _)| {
            !open && rows.iter().skip(count).any(|row| &row.key == key)
        });
        let can_focus = self.enabled.try_get()? && scope.capabilities()?.focus;
        if (lost || hidden) && can_focus {
            self.handoff.set(true);
        } else if !can_focus {
            self.handoff.set(false);
            self.focus.borrow_mut().take();
        }
        let more =
            self.popup && (count < rows.len() || self.more_focus.get() || self.handoff.get());
        if self.more_visible.try_get()? != more {
            if let Some(node) = self.more_box.get() {
                scope.patch(
                    node,
                    Patch::Semantics(Change::Set(composite::metadata(Semantics {
                        inert: !more,
                        ..Default::default()
                    }))),
                )?;
            }
            self.more_visible.try_revise(|value| *value = more)?;
        }
        if self.count.try_get()? != count {
            self.count.try_revise(|value| *value = count)?;
        }
        let strip = self.strip.get().ok_or(glory::ui::Error::Gone)?;
        let panel = self.panel.get();
        let mut moved_focus = false;
        for (index, row) in rows.iter().enumerate() {
            let parent = if index < count {
                strip
            } else {
                panel.ok_or(glory::ui::Error::Gone)?
            };
            if force || row.parent.get() != parent {
                moved_focus |= focused.as_ref().is_some_and(|(key, _)| key == &row.key);
                scope.place(row.node, Some(parent), None)?;
                row.parent.set(parent);
            }
        }
        if can_focus {
            if lost || hidden {
                if let Some(trigger) = self.trigger.try_get()? {
                    self.focus.borrow_mut().take();
                    scope.action(Action::Focus(trigger))?;
                    self.confirm_focus(trigger)?;
                }
            } else if moved_focus
                && let Some((key, node)) = focused
                && (rows
                    .iter()
                    .any(|row| row.key == key && row.parent.get() == strip)
                    || open)
            {
                scope.action(Action::Focus(node))?;
            }
        }
        Ok(())
    }
    fn measure(self: &Rc<Self>) -> Result {
        let scope = self.scope()?;
        if self.focus_pending.get() {
            self.refresh_after_focus.set(true);
            return Ok(());
        }
        if self.building.get() {
            return self.schedule();
        }
        if self.batch.borrow().is_some() {
            return Ok(());
        }
        let rows = self.rows.borrow().clone();
        let line = self.line.get().ok_or(glory::ui::Error::Gone)?;
        let trigger = self.trigger.try_get()?.ok_or(glory::ui::Error::Gone)?;
        let lane = self.lane.get().ok_or(glory::ui::Error::Gone)?;
        let open = self.open.source.try_get()? && self.count.try_get()? < rows.len();
        // Only currently hidden rows enter the inert, transparent measurement
        // lane. Visible/focused widgets keep their actual presentation parent.
        if !open {
            for row in rows.iter().skip(self.count.try_get()?) {
                if row.parent.get() != lane {
                    scope.place(row.node, Some(lane), None)?;
                    row.parent.set(lane);
                }
            }
        }
        let ticket = self
            .next
            .get()
            .checked_add(1)
            .ok_or_else(|| invalid("overflow measurement identity exhausted"))?;
        self.next.set(ticket);
        *self.batch.borrow_mut() = Some(Batch {
            ticket,
            tokens: Vec::with_capacity(rows.len() + 2),
            remaining: rows.len() + 2,
            available: None,
            trigger: None,
            widths: vec![None; rows.len()],
            repeat: false,
        });
        self.pending.try_revise(|value| *value = rows.len() + 2)?;
        for (index, node) in [line, trigger]
            .into_iter()
            .chain(rows.iter().map(|row| row.node))
            .enumerate()
        {
            let weak = Rc::downgrade(self);
            let token = scope.request(Query::Bounds(node), move |result| {
                if let Some(state) = weak.upgrade()
                    && let Err(error) = state.reply(ticket, index, result)
                {
                    state.cancel();
                    let _ = state.reconcile(false);
                    state.report(error)
                }
            })?;
            if let Some(work) = self
                .batch
                .borrow_mut()
                .as_mut()
                .filter(|work| work.ticket == ticket)
            {
                work.tokens.push(token);
            } else {
                scope.cancel_request(token)?;
            }
        }
        Ok(())
    }
    fn reply(
        self: &Rc<Self>,
        ticket: u64,
        index: usize,
        result: std::result::Result<QueryValue, glory::render::Error>,
    ) -> Result {
        let work = {
            let mut slot = self.batch.borrow_mut();
            let Some(work) = slot.as_mut().filter(|work| work.ticket == ticket) else {
                return Ok(());
            };
            let QueryValue::Bounds(rect) = result? else {
                return Err(invalid("overflow query response mismatch"));
            };
            model::extent(rect.width)?;
            match index {
                0 => work.available = Some(rect.width),
                1 => work.trigger = Some(rect.width),
                index => work.widths[index - 2] = Some(rect.width),
            }
            work.remaining -= 1;
            if work.remaining == 0 {
                slot.take()
            } else {
                None
            }
        };
        self.sync_pending()?;
        let Some(work) = work else { return Ok(()) };
        self.scope()?;
        self.pending.try_revise(|value| *value = 0)?;
        let widths = work
            .widths
            .into_iter()
            .map(|width| width.ok_or_else(|| invalid("overflow item geometry missing")))
            .collect::<Result<Vec<_>>>()?;
        let available = work
            .available
            .ok_or_else(|| invalid("overflow container geometry missing"))?;
        // A hidden ancestor has no intrinsic observation. Keep the accepted
        // prefix; a later positive Resize starts a fresh bounded observation.
        if available > 0. {
            let count = model::capacity(
                &widths,
                available,
                work.trigger
                    .ok_or_else(|| invalid("overflow trigger geometry missing"))?,
                self.gap.try_get()?,
                self.more_focus.get(),
            )?;
            let current = self.visible.source.try_get()?.min(widths.len());
            if count != current {
                self.visible.accept(&count)?;
                self.changed.emit(self.owner, count);
            }
        }
        self.reconcile(false)?;
        if work.repeat {
            self.schedule()?;
        }
        Ok(())
    }
}

fn row(
    scope: &Scope,
    state: &Rc<Controller>,
    item: OverflowItem,
    parent: NodeId,
) -> Result<Rc<Row>> {
    composite::validate_keys(std::slice::from_ref(&item.key))?;
    if let Some(width) = &item.width {
        model::extent(width.try_get()?)?;
    }
    let owner = scope.child_in(parent)?;
    let reference = owner.cage(None);
    let OverflowItem {
        key,
        mut content,
        width,
    } = item;
    let focus_key = key.clone();
    let focus = Rc::downgrade(state);
    let resize = Rc::downgrade(state);
    let last_width = Cell::new(None);
    let accepted_width = width
        .as_ref()
        .map(|width| composite::validated(&owner, width, |width| model::extent(*width)))
        .transpose()?;
    let width_state = Rc::downgrade(state);
    let child = glory::ui::view()
        .class("sapid-overflow-item")
        .node_ref(reference)
        .style(owner.bond(move || {
            let mut style = Style::new().display(Display::InlineFlex);
            style.shrink = Some(0.0);
            style.grow = Some(0.0);
            style.white_space = Some(WhiteSpace::NoWrap);
            if let Some(width) = accepted_width {
                style.width = Some(Length::Px(width.get()));
            }
            style
        }))
        .on(EventKind::FocusWithin, move |event| {
            if let (Some(state), Event::FocusWithin { node, focused }) =
                (focus.upgrade(), event)
            {
                if focused {
                    *state.focus.borrow_mut() = Some((focus_key.clone(), node));
                } else if state
                    .focus
                    .borrow()
                    .as_ref()
                    .is_some_and(|(current, _)| current == &focus_key)
                {
                    state.focus.borrow_mut().take();
                }
            }
        })
        .on(EventKind::Resize, move |event| {
            if let (Some(state), Event::Resize { size, .. }) = (resize.upgrade(), event)
                && size.width > 0.
                && last_width.replace(Some(size.width)) != Some(size.width)
                && state.batch.borrow().is_none()
                && let Err(error) = state.schedule()
            {
                state.report(error)
            }
        })
        .fill(glory::ui::widget(move |scope| {
            if let Some(factory) = content.as_mut() {
                factory().show_in(scope)
            } else {
                Ok(())
            }
        }));
    let mounted = child.mount(&owner)?;
    let node = reference.try_get()?.ok_or(glory::ui::Error::Gone)?;
    if let Some(value) = accepted_width {
        let weak = owner.downgrade();
        owner.effect(move || {
            if let (Some(scope), Some(state)) = (weak.upgrade(), width_state.upgrade()) {
                let result = (|| -> Result {
                    let _ = value.try_get()?;
                    state.schedule()
                })();
                if let Err(error) = result {
                    scope.report_error(error)
                }
            }
        });
    }
    // Mounted owns its child Scope. The intermediate construction Owner is
    // retained by that scope, preserving resources and bindings through moves.
    mounted.scope().retain(owner)?;
    Ok(Rc::new(Row {
        key,
        mounted,
        node,
        parent: Cell::new(parent),
    }))
}

fn dynamic_row(
    scope: &Scope,
    state: &Rc<Controller>,
    key: &str,
    factory: &mut KeyFactory,
    parent: NodeId,
) -> Result<Rc<Row>> {
    let owner = scope.child()?;
    let item = owner.owner().enter(|| factory(key));
    if item.key != key {
        return Err(invalid("overflow factory returned a different key"));
    }
    let row = row(&owner, state, item, parent)?;
    row.mounted.scope().retain(owner)?;
    Ok(row)
}

impl Widget for OverflowList {
    fn build(&mut self, scope: &Scope) -> Result {
        composite::id("overflow", &self.key, "root", "root")?;
        if self.initial > MAX_OVERFLOW_ITEMS {
            return Err(invalid("overflow initial prefix exceeds 128 items"));
        }
        if self.dynamic.is_some() && !self.items.is_empty() {
            return Err(invalid("overflow items must be static or dynamic"));
        }
        let initial_keys = if let Some((source, _)) = &self.dynamic {
            source.try_with(|value| {
                keys(value)?;
                Ok::<_, glory::ui::Error>(value.clone())
            })??
        } else {
            if self.items.len() > MAX_OVERFLOW_ITEMS
                || self.items.iter().any(|item| item.key.len() > 64)
            {
                return Err(invalid("overflow static item budget exceeded"));
            }
            let value = self
                .items
                .iter()
                .map(|item| item.key.clone())
                .collect::<Vec<_>>();
            keys(&value)?;
            value
        };
        let raw_gap = self.gap.clone();
        model::extent(raw_gap.try_get()?)?;
        let gap = scope.cage(raw_gap.try_get()?);
        let label = composite::validated(scope, &self.label, |label| {
            if label.is_empty() || label.len() > 256 {
                return Err(invalid("overflow label must contain 1..=256 bytes"));
            }
            Ok(())
        })?;
        let controlled = self
            .visible
            .take()
            .map(|source| {
                composite::validated(scope, &source, |count| {
                    if *count > MAX_OVERFLOW_ITEMS {
                        return Err(invalid("overflow controlled prefix exceeds 128 items"));
                    }
                    Ok(())
                })
                .map(Vary::from)
            })
            .transpose()?;
        let mut visible = Value {
            source: controlled,
            default: self.initial,
        };
        if !self.popup {
            visible.default = initial_keys.len();
        }
        let visible = visible.mount(scope);
        if visible.source.try_get()? > MAX_OVERFLOW_ITEMS {
            return Err(invalid("overflow controlled prefix exceeds 128 items"));
        }
        let count = scope.cage(if self.popup {
            visible.source.try_get()?.min(initial_keys.len())
        } else {
            initial_keys.len()
        });
        let total = scope.cage(initial_keys.len());
        let more = scope.cage(self.popup && count.get() < total.get());
        let open = std::mem::take(&mut self.open).mount(scope);
        let pending = scope.cage(0);
        let state = Rc::new(Controller {
            weak: scope.downgrade(),
            owner: scope.id(),
            live: Cell::new(true),
            popup: self.popup,
            line: Cell::new(None),
            strip: Cell::new(None),
            lane: Cell::new(None),
            panel: Cell::new(None),
            trigger: scope.cage(None),
            more_box: Cell::new(None),
            rows: RefCell::new(Vec::new()),
            batch: RefCell::new(None),
            next: Cell::new(0),
            task: RefCell::new(None),
            scheduled: Cell::new(false),
            building: Cell::new(false),
            focus_task: RefCell::new(None),
            focus_query: Cell::new(None),
            focus_pending: Cell::new(false),
            refresh_after_focus: Cell::new(false),
            visible,
            open,
            gap,
            enabled: self.common.enabled.clone(),
            count,
            total,
            more_visible: more,
            more_focus: Cell::new(false),
            handoff: Cell::new(false),
            focus: RefCell::new(None),
            last_width: Cell::new(None),
            last_trigger: Cell::new(None),
            changed: Callback::new(self.changed.take()),
            open_changed: Callback::new(self.open_changed.take()),
            pending,
        });
        let weak = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(state) = weak.upgrade() {
                state.retire()
            }
        })?;
        let enabled = self.common.enabled.clone();
        let root_ref = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let custom = self.common.style.clone();
        let unstyled = self.common.unstyled;
        let theme = self.common.theme.clone();
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-overflow-list",
            Style::new()
        )
        .node_ref(root_ref)
        .style(scope.bond(move || {
            let mut style = if unstyled {
                Style::new()
            } else {
                Style {
                    color: Some(theme.foreground),
                    ..Style::new()
                }
            };
            style = merge(style, custom.get());
            style.position.get_or_insert(Position::Relative);
            style.overflow_x.get_or_insert(Overflow::Hidden);
            style.overflow_y.get_or_insert(Overflow::Hidden);
            style.min_width.get_or_insert(Length::Px(0.));
            style
        }))
        .semantics(scope.bond(move || {
            composite::metadata(Semantics {
                inert: !enabled.get(),
                ..Default::default()
            })
        }));
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)?;
        let parent = scope.child_in(root_ref.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        let line_ref = scope.cage(None);
        let line_state = Rc::downgrade(&state);
        glory::ui::view()
            .class("sapid-overflow-line")
            .node_ref(line_ref)
            .style(scope.bond(move || {
                Style {
                    gap: Some(gap.get()),
                    align: Some(Align::Start),
                    ..Style::new()
                        .display(Display::Flex)
                        .width(Length::Percent(100.))
                }
            }))
            .on(EventKind::Resize, move |event| {
                if let (Some(state), Event::Resize { size, .. }) = (line_state.upgrade(), event)
                    && state.last_width.replace(Some(size.width)) != Some(size.width)
                    && let Err(error) = state.schedule()
                {
                    state.report(error)
                }
            })
            .show_in(&parent)?;
        let line = line_ref.try_get()?.ok_or(glory::ui::Error::Gone)?;
        state.line.set(Some(line));
        let line_scope = scope.child_in(line)?;
        let strip_ref = scope.cage(None);
        glory::ui::view()
            .class("sapid-overflow-strip")
            .node_ref(strip_ref)
            .style(scope.bond(move || Style {
                gap: Some(gap.get()),
                shrink: Some(0.0),
                align: Some(Align::Start),
                ..Style::new().display(if count.get() == 0 {
                    Display::None
                } else {
                    Display::Flex
                })
            }))
            .show_in(&line_scope)?;
        state.strip.set(strip_ref.try_get()?);
        let lane_ref = scope.cage(None);
        glory::ui::view()
            .class("sapid-overflow-measure")
            .node_ref(lane_ref)
            .semantics(composite::metadata(Semantics {
                inert: true,
                ..Default::default()
            }))
            .style(Style {
                display: Some(Display::Flex),
                flex_wrap: Some(FlexWrap::NoWrap),
                position: Some(Position::Absolute),
                width: Some(Length::Percent(100.)),
                top: Some(Length::Px(0.)),
                left: Some(Length::Px(0.)),
                opacity: Some(0.0),
                overflow_x: Some(Overflow::Hidden),
                overflow_y: Some(Overflow::Hidden),
                pointer_events: Some(PointerEvents::None),
                align: Some(Align::Start),
                ..Style::new()
            })
            .show_in(&parent)?;
        state.lane.set(lane_ref.try_get()?);
        if self.popup {
            let more_ref = scope.cage(None);
            let more_state = Rc::downgrade(&state);
            glory::ui::view()
                .class("sapid-overflow-more")
                .node_ref(more_ref)
                .semantics(scope.bond(move || {
                    composite::metadata(Semantics {
                        inert: !more.get(),
                        ..Default::default()
                    })
                }))
                .style(scope.bond(move || Style {
                    position: Some(if more.get() {
                        Position::Relative
                    } else {
                        Position::Absolute
                    }),
                    opacity: Some(if more.get() { 1.0 } else { 0.0 }),
                    pointer_events: Some(if more.get() {
                        PointerEvents::Auto
                    } else {
                        PointerEvents::None
                    }),
                    min_width: Some(Length::Px(0.)),
                    max_width: Some(Limit::Percent(100.)),
                    overflow_x: Some(Overflow::Hidden),
                    overflow_y: Some(Overflow::Hidden),
                    shrink: Some(1.0),
                    ..Style::new()
                }))
                .show_in(&line_scope)?;
            state.more_box.set(more_ref.try_get()?);
            let more_scope =
                scope.child_in(more_ref.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
            let panel_ref = scope.cage(None);
            let open = state.open.source.clone();
            let on_open = Rc::downgrade(&state);
            let trigger = state.trigger;
            let popup = crate::popover(format!("{}-more", self.key))
                .theme(self.common.theme.clone())
                .size(self.common.size)
                .label(scope.bond(move || {
                    format!(
                        "{} ({} hidden)",
                        label.get(),
                        total.get().saturating_sub(count.get())
                    )
                }))
                .open(scope.bond(move || open.get() && count.get() < total.get()))
                .enabled(self.common.enabled.clone())
                .show_close(false)
                .content_policy(ContentPolicy::KeepMounted)
                .trigger_ref(trigger)
                .on_open_change(move |open| {
                    if let Some(state) = on_open.upgrade()
                        && let Err(error) = state.set_open(open)
                    {
                        state.report(error)
                    }
                })
                .content(move || {
                    glory::ui::view()
                        .class("sapid-overflow-panel")
                        .node_ref(panel_ref)
                        .style(Style {
                            align: Some(Align::Start),
                            flex_direction: Some(FlexDirection::Column),
                            gap: Some(8.0),
                            ..Style::new().display(Display::Flex)
                        })
                });
            let popup = if self.common.unstyled {
                popup.unstyled()
            } else {
                popup
            };
            popup.show_in(&more_scope)?;
            scope.on(
                trigger.try_get()?.ok_or(glory::ui::Error::Gone)?,
                EventKind::FocusChanged,
                move |event| {
                    if let (Some(state), Event::FocusChanged(focused)) =
                        (more_state.upgrade(), event)
                    {
                        state.more_focus.set(focused);
                        state.handoff.set(false);
                        state.cancel_focus();
                        let _ = state.sync_pending();
                        if let Err(error) =
                            state.reconcile(false).and_then(|_| state.schedule())
                        {
                            state.report(error)
                        }
                    }
                },
            )?;
            let resize = Rc::downgrade(&state);
            scope.on(
                trigger.try_get()?.ok_or(glory::ui::Error::Gone)?,
                EventKind::Resize,
                move |event| {
                    if let (Some(state), Event::Resize { size, .. }) = (resize.upgrade(), event)
                        && size.width > 0.
                        && state.last_trigger.replace(Some(size.width)) != Some(size.width)
                        && state.batch.borrow().is_none()
                        && let Err(error) = state.schedule()
                    {
                        state.report(error)
                    }
                },
            )?;
            state.panel.set(panel_ref.try_get()?);
            scope.retain(more_scope)?;
        }
        let strip = state.strip.get().ok_or(glory::ui::Error::Gone)?;
        let mut rows = Vec::with_capacity(initial_keys.len());
        if let Some((_, factory)) = &mut self.dynamic {
            for (index, key) in initial_keys.iter().enumerate() {
                let target = if index < count.get() {
                    strip
                } else {
                    state.panel.get().ok_or(glory::ui::Error::Gone)?
                };
                rows.push(dynamic_row(scope, &state, key, factory, target)?);
            }
        } else {
            for (index, item) in std::mem::take(&mut self.items).into_iter().enumerate() {
                let target = if index < count.get() {
                    strip
                } else {
                    state.panel.get().ok_or(glory::ui::Error::Gone)?
                };
                rows.push(row(scope, &state, item, target)?);
            }
        }
        *state.rows.borrow_mut() = rows;
        let handle = OverflowHandle {
            inner: state.clone(),
            visible: scope.bond(move || count.get()),
            hidden: scope.bond(move || total.get().saturating_sub(count.get())),
            pending: scope.bond(move || pending.get()),
        };
        if let Some(reference) = self.handle {
            reference.try_revise(|value| *value = Some(handle))?;
        }
        let config = Rc::downgrade(&state);
        let visible = state.visible.source.clone();
        let open = state.open.source.clone();
        let enabled = state.enabled.clone();
        scope.structural(move || {
            let _ = (visible.get(), open.get(), enabled.get());
            if let Some(state) = config.upgrade() {
                if visible.get() > MAX_OVERFLOW_ITEMS {
                    state.report(invalid("overflow controlled prefix exceeds 128 items"));
                    return;
                }
                state.cancel();
                if let Err(error) = state.reconcile(false).and_then(|_| state.schedule()) {
                    state.report(error)
                }
            }
        });
        if let Some((source, mut factory)) = self.dynamic.take() {
            let config = Rc::downgrade(&state);
            let owner = scope.downgrade();
            scope.structural(move || {
                let Some(state) = config.upgrade() else {
                    return;
                };
                let Some(scope) = owner.upgrade() else { return };
                let result = (|| -> Result {
                    let next = source.try_with(|value| {
                        keys(value)?;
                        Ok::<_, glory::ui::Error>(value.clone())
                    })??;
                    let previous = state.rows.borrow().clone();
                    if previous.iter().map(|row| &row.key).eq(next.iter()) {
                        return Ok(());
                    }
                    state.cancel();
                    state.building.set(true);
                    let mut old = previous
                        .into_iter()
                        .map(|row| (row.key.clone(), row))
                        .collect::<HashMap<_, _>>();
                    let mut rows = Vec::with_capacity(next.len());
                    let result = (|| -> Result {
                        for key in &next {
                            rows.push(if let Some(row) = old.remove(key) {
                                row
                            } else {
                                dynamic_row(&scope, &state, key, &mut factory, strip)?
                            });
                        }
                        Ok(())
                    })();
                    state.building.set(false);
                    result?;
                    let retired = state.rows.replace(rows);
                    drop(retired);
                    drop(old);
                    total.try_revise(|value| *value = next.len())?;
                    state.reconcile(true)?;
                    state.schedule()
                })();
                if let Err(error) = result {
                    state.report(error)
                }
            });
        }
        let refresh = Rc::downgrade(&state);
        let revision = self.revision.clone();
        let weak = scope.downgrade();
        scope.effect(move || {
            let _ = revision.get();
            let next = raw_gap.get();
            if let (Some(state), Some(scope)) = (refresh.upgrade(), weak.upgrade()) {
                let result = (|| -> Result {
                    model::extent(next)?;
                    if gap.try_get()? != next {
                        gap.try_revise(|value| *value = next)?;
                    }
                    state.schedule()
                })();
                if let Err(error) = result {
                    scope.report_error(error)
                }
            }
        });
        scope.retain((parent, line_scope, state))?;
        Ok(())
    }
}
