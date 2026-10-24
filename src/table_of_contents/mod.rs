//! A live heading index and a bounded, explicitly owned scrolling document.
use crate::{appearance::Common, composite};
use glory::render::{
    Action, Current, Event, EventKind, NodeId, Query, QueryToken, QueryValue, Rect, Role,
    ScrollMetrics, SemanticId, Semantics,
};
use glory::style::{Length, Style};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Bond, Cage, Lotus, Scope};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};
mod model;
pub use model::{MAX_TOC_HEADINGS, TocEntry};
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
#[derive(Clone)]
struct Registered {
    info: TocEntry,
    node: NodeId,
    ticket: u64,
    scope: WeakScope,
}
#[derive(Clone)]
struct Viewport {
    node: NodeId,
    scope: WeakScope,
}
struct Batch {
    ticket: u64,
    tokens: Vec<QueryToken>,
    remaining: usize,
    viewport: Option<Rect>,
    metrics: Option<ScrollMetrics>,
    headings: Vec<(SemanticId, Option<Rect>)>,
    repeat: bool,
}
struct Index {
    key: String,
    owner: glory::core::InstanceId,
    weak: WeakScope,
    pulse: Cage<()>,
    active: Cage<Option<SemanticId>>,
    entries: RefCell<BTreeMap<SemanticId, Registered>>,
    viewport: RefCell<Option<Viewport>>,
    batch: RefCell<Option<Batch>>,
    next: Cell<u64>,
    next_entry: Cell<u64>,
    measured: Cell<bool>,
    offset: Cell<f64>,
    reveal: RefCell<Option<SemanticId>>,
    queued: Cell<bool>,
    task: RefCell<Option<glory::core::TaskToken>>,
}
#[derive(Clone)]
pub struct TocContext(Rc<Index>);
pub fn toc_context(scope: &Scope, key: impl Into<String>) -> Result<TocContext> {
    let key = key.into();
    composite::id("toc", &key, "root", "root")?;
    let state = Rc::new(Index {
        key,
        owner: scope.id(),
        weak: scope.downgrade(),
        pulse: scope.cage(()),
        active: scope.cage(None),
        entries: RefCell::new(BTreeMap::new()),
        viewport: RefCell::new(None),
        batch: RefCell::new(None),
        next: Cell::new(0),
        next_entry: Cell::new(0),
        measured: Cell::new(false),
        offset: Cell::new(0.),
        reveal: RefCell::new(None),
        queued: Cell::new(false),
        task: RefCell::new(None),
    });
    let weak = Rc::downgrade(&state);
    scope.on_cleanup(move || {
        if let Some(state) = weak.upgrade() {
            state.cancel();
            let entries = std::mem::take(&mut *state.entries.borrow_mut());
            drop(entries);
            state.viewport.borrow_mut().take();
        }
    })?;
    let weak = Rc::downgrade(&state);
    let pulse = state.pulse;
    let scope_weak = scope.downgrade();
    scope.effect(move || {
        pulse.get();
        if let (Some(state), Some(scope)) = (weak.upgrade(), scope_weak.upgrade())
            && let Err(error) = state.refresh()
        {
            scope.report_error(error)
        }
    });
    Ok(TocContext(state))
}
impl TocContext {
    pub fn entries(&self, scope: &Scope) -> Bond<Vec<TocEntry>> {
        let index = self.0.clone();
        scope.bond(move || {
            index.pulse.get();
            index
                .ordered()
                .into_iter()
                .map(|entry| entry.info)
                .collect()
        })
    }
    pub fn current(&self, scope: &Scope) -> Bond<Option<SemanticId>> {
        let active = self.0.active;
        scope.bond(move || active.get())
    }
    pub fn refresh(&self) -> Result {
        self.0.refresh()
    }
    pub fn reveal(&self, id: &SemanticId) -> Result {
        self.0.ensure()?;
        if !self.0.entries.borrow().contains_key(id) {
            return Err(invalid("cannot reveal an unregistered heading"));
        }
        let viewport = self
            .0
            .viewport
            .borrow()
            .clone()
            .ok_or(glory::ui::Error::Gone)?;
        let scope = viewport.scope.upgrade().ok_or(glory::ui::Error::Gone)?;
        let heading = self.0.entries.borrow()[id].node;
        if !scope.contains_node(viewport.node, heading)? {
            return Err(invalid("heading left its scrolling document"));
        }
        let capabilities = scope.capabilities()?;
        if !capabilities.measure || !capabilities.scroll {
            return Err(glory::render::Error::Unsupported(
                "heading reveal requires layout and scrolling",
            )
            .into());
        }
        *self.0.reveal.borrow_mut() = Some(id.clone());
        self.0.refresh()
    }
    pub fn len(&self) -> usize {
        self.0.entries.borrow().len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl Index {
    fn ensure(&self) -> Result {
        if self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn ensure_scope(&self, scope: &Scope) -> Result {
        self.ensure()?;
        if scope.id().runtime() != self.owner.runtime() {
            return Err(glory::render::Error::ForeignRuntime.into());
        }
        Ok(())
    }
    fn ordered(&self) -> Vec<Registered> {
        let mut values = self.entries.borrow().values().cloned().collect::<Vec<_>>();
        values.sort_by_key(|entry| (entry.info.order, entry.ticket));
        values
    }
    fn changed(&self) -> Result {
        self.cancel();
        // The construction seed must agree across measureless SSR and an
        // adopting host before that host can run its first measurement task.
        if !self.measured.get() {
            let first = self.ordered().first().map(|entry| entry.info.id.clone());
            if glory::core::untracked(|| self.active.try_get())? != first {
                self.active.try_revise(|value| *value = first)?;
            }
        }
        self.pulse.try_revise(|_| ())?;
        Ok(())
    }
    fn cancel(&self) {
        self.queued.set(false);
        let task = self.task.borrow_mut().take();
        if let Some(task) = task {
            task.cancel();
        }
        let work = self.batch.borrow_mut().take();
        let viewport = self.viewport.borrow().clone();
        if let (Some(work), Some(scope)) =
            (work, viewport.and_then(|value| value.scope.upgrade()))
        {
            for token in work.tokens {
                let _ = scope.cancel_request(token);
            }
        }
    }
    fn unregister(&self, id: &SemanticId) {
        let removed = self.entries.borrow_mut().remove(id);
        if removed.is_some() && self.owner.is_alive() {
            if self.active.get().as_ref() == Some(id) {
                let _ = self.active.try_revise(|value| *value = None);
            }
            if let Err(error) = self.changed()
                && let Some(scope) = self.weak.upgrade()
            {
                scope.report_error(error)
            }
        }
        drop(removed);
    }
    fn refresh(self: &Rc<Self>) -> Result {
        self.ensure()?;
        if let Some(work) = self.batch.borrow_mut().as_mut() {
            work.repeat = true;
            return Ok(());
        }
        if self.queued.get() {
            return Ok(());
        }
        let Some(viewport) = self.viewport.borrow().clone() else {
            return Ok(());
        };
        let Some(scope) = viewport.scope.upgrade() else {
            return Ok(());
        };
        let capabilities = scope.capabilities()?;
        if !capabilities.measure || !capabilities.scroll {
            let first = self.ordered().first().map(|entry| entry.info.id.clone());
            if self.active.get() != first {
                self.active.try_revise(|value| *value = first)?;
            }
            return Ok(());
        }
        self.queued.set(true);
        let weak = Rc::downgrade(self);
        let token = match scope.spawn(async move {
            let Some(state) = weak.upgrade() else { return };
            state.queued.set(false);
            let _completed = state.task.borrow_mut().take();
            if let Err(error) = state.measure() {
                state.cancel();
                if let Some(scope) = state.weak.upgrade() {
                    scope.report_error(error)
                }
            }
        }) {
            Ok(token) => token,
            Err(error) => {
                self.queued.set(false);
                return Err(error.into());
            }
        };
        if self.queued.get() {
            *self.task.borrow_mut() = Some(token);
        }
        Ok(())
    }
    fn measure(self: &Rc<Self>) -> Result {
        self.ensure()?;
        if let Some(work) = self.batch.borrow_mut().as_mut() {
            work.repeat = true;
            return Ok(());
        }
        let entries = self.ordered();
        let viewport = self.viewport.borrow().clone();
        let Some(viewport) = viewport else {
            return Ok(());
        };
        let Some(scope) = viewport.scope.upgrade() else {
            return Ok(());
        };
        for entry in &entries {
            if !scope.contains_node(viewport.node, entry.node)? {
                return Err(invalid("heading left its scrolling document"));
            }
        }
        let capabilities = scope.capabilities()?;
        if !capabilities.measure || !capabilities.scroll {
            let first = entries.first().map(|entry| entry.info.id.clone());
            if self.active.get() != first {
                self.active.try_revise(|value| *value = first)?;
            }
            return Ok(());
        }
        let ticket = self
            .next
            .get()
            .checked_add(1)
            .ok_or_else(|| invalid("heading measurement identity exhausted"))?;
        self.next.set(ticket);
        *self.batch.borrow_mut() = Some(Batch {
            ticket,
            tokens: Vec::with_capacity(entries.len() + 2),
            remaining: entries.len() + 2,
            viewport: None,
            metrics: None,
            headings: entries
                .iter()
                .map(|entry| (entry.info.id.clone(), None))
                .collect(),
            repeat: false,
        });
        let requests = std::iter::once((Query::Bounds(viewport.node), 0))
            .chain(std::iter::once((Query::ScrollMetrics(viewport.node), 1)))
            .chain(
                entries
                    .iter()
                    .enumerate()
                    .map(|(index, entry)| (Query::Bounds(entry.node), index + 2)),
            );
        for (query, index) in requests {
            let state = self.clone();
            let token = match scope.request(query, move |result| {
                let outcome = state.reply(ticket, index, result);
                if let Err(error) = outcome {
                    state.cancel();
                    if let Some(scope) = state.weak.upgrade() {
                        scope.report_error(error)
                    }
                }
            }) {
                Ok(token) => token,
                Err(error) => {
                    self.cancel();
                    return Err(error);
                }
            };
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
        let completed = {
            let mut slot = self.batch.borrow_mut();
            let Some(work) = slot.as_mut().filter(|work| work.ticket == ticket) else {
                return Ok(());
            };
            let value = result?;
            match (index, value) {
                (0, QueryValue::Bounds(rect)) => work.viewport = Some(rect),
                (1, QueryValue::ScrollMetrics(metrics)) => work.metrics = Some(metrics),
                (index, QueryValue::Bounds(rect)) if index >= 2 => {
                    work.headings[index - 2].1 = Some(rect)
                }
                _ => return Err(invalid("heading query response mismatch")),
            };
            work.remaining -= 1;
            if work.remaining == 0 {
                slot.take()
            } else {
                None
            }
        };
        let Some(work) = completed else { return Ok(()) };
        self.ensure()?;
        let viewport = work
            .viewport
            .ok_or_else(|| invalid("heading viewport geometry missing"))?;
        let metrics = work
            .metrics
            .ok_or_else(|| invalid("heading viewport metrics missing"))?;
        let headings = work
            .headings
            .into_iter()
            .filter_map(|(id, rect)| rect.map(|rect| (id, rect)))
            .collect::<Vec<_>>();
        let active = model::current(&headings, viewport, self.offset.get());
        self.measured.set(true);
        if self.active.get() != active {
            self.active.try_revise(|value| *value = active)?;
        }
        let reveal = self.reveal.borrow_mut().take();
        if let Some(reveal) = reveal
            && let Some((_, heading)) = headings.iter().find(|(id, _)| id == &reveal)
            && heading.height > 0.
        {
            if metrics.x < 0. || metrics.y < 0. {
                return Err(glory::render::Error::Unsupported(
                    "heading reveal on negative RTL/elastic offsets",
                )
                .into());
            }
            if let Some(viewport) = self.viewport.borrow().clone()
                && let Some(scope) = viewport.scope.upgrade()
            {
                scope.action(Action::ScrollTo {
                    node: viewport.node,
                    x: metrics.x,
                    y: (metrics.y + heading.y - work.viewport.unwrap().y - self.offset.get())
                        .clamp(
                            0.,
                            (metrics.content_height - metrics.viewport_height).max(0.),
                        ),
                })?;
            }
        }
        if work.repeat {
            self.refresh()?;
        }
        Ok(())
    }
}

pub struct TocHeading {
    context: TocContext,
    key: String,
    level: u8,
    order: Lotus<u32>,
    content: Lotus<String>,
    common: Common,
}
pub fn toc_heading(context: TocContext, key: impl Into<String>, level: u8) -> TocHeading {
    TocHeading {
        context,
        key: key.into(),
        level,
        order: 0.into(),
        content: String::new().into(),
        common: Common::default(),
    }
}
impl TocHeading {
    pub fn content(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.content = value.into();
        self
    }
    pub fn order(mut self, value: impl Into<Lotus<u32>>) -> Self {
        self.order = value.into();
        self
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn size(mut self, value: crate::Size) -> Self {
        self.common.size = value;
        self
    }
    pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
}
impl Widget for TocHeading {
    fn build(&mut self, scope: &Scope) -> Result {
        self.context.0.ensure_scope(scope)?;
        if !(1..=6).contains(&self.level) {
            return Err(invalid("registered headings require level 1..=6"));
        }
        if let Some(viewport) = self.context.0.viewport.borrow().as_ref()
            && !scope.is_within(viewport.node)?
        {
            return Err(invalid(
                "registered headings must belong to their scrolling document",
            ));
        }
        let id = composite::id("toc", &self.context.0.key, "heading", &self.key)?;
        {
            let entries = self.context.0.entries.borrow();
            if entries.len() >= MAX_TOC_HEADINGS {
                return Err(invalid("live heading index exceeds 256 headings"));
            }
            if entries.contains_key(&id) {
                return Err(invalid("live heading key is already registered"));
            }
        }
        let next = self
            .context
            .0
            .next_entry
            .get()
            .checked_add(1)
            .ok_or_else(|| invalid("heading registration identity exhausted"))?;
        let label =
            composite::validated(scope, &self.content, |value| model::validate_label(value))?;
        let level = self.level;
        let scale = self
            .common
            .theme
            .label(self.common.size)
            .font_size
            .unwrap_or(14.)
            / 14.;
        let size = [32., 28., 24., 20., 18., 16.][usize::from(level - 1)] * scale;
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let style = Style {
            font_size: Some(size),
            line_height: Some(size * 1.3),
            font_weight: Some(700.),
            color: Some(self.common.theme.foreground),
            margin: Some(glory::style::Edges::all(0.)),
            ..Style::new().font_family([glory::style::Family::System])
        };
        let (classes, style) =
            self.common
                .appearance(scope, "sapid-title sapid-toc-heading", style);
        let resize = Rc::downgrade(&self.context.0);
        let resize_scope = scope.downgrade();
        glory::ui::text()
            .class(classes)
            .style(style)
            .content(label)
            .kind(glory::render::TextKind::Heading(level))
            .on(EventKind::Resize, move |_| {
                if let (Some(state), Some(scope)) = (resize.upgrade(), resize_scope.upgrade())
                    && let Err(error) = state.refresh()
                {
                    scope.report_error(error)
                }
            })
            .node_ref(reference)
            .semantics(composite::metadata(Semantics {
                id: Some(id.clone()),
                ..Default::default()
            }))
            .show_in(scope)?;
        self.context.0.next_entry.set(next);
        self.context.0.entries.borrow_mut().insert(
            id.clone(),
            Registered {
                info: TocEntry {
                    id: id.clone(),
                    label: label.get(),
                    level,
                    order: self.order.get(),
                },
                node: reference.get().ok_or(glory::ui::Error::Gone)?,
                ticket: next,
                scope: scope.downgrade(),
            },
        );
        self.context.0.changed()?;
        let weak = Rc::downgrade(&self.context.0);
        let key = id.clone();
        scope.on_cleanup(move || {
            if let Some(state) = weak.upgrade() {
                state.unregister(&key)
            }
        })?;
        let weak = Rc::downgrade(&self.context.0);
        let key = id.clone();
        let order = self.order.clone();
        let weak_scope = scope.downgrade();
        scope.effect(move || {
            let next_label = label.get();
            let order = order.get();
            let Some(state) = weak.upgrade() else { return };
            let updated = {
                let mut entries = state.entries.borrow_mut();
                if let Some(entry) = entries.get_mut(&key)
                    && (entry.info.label != next_label || entry.info.order != order)
                {
                    entry.info.label = next_label;
                    entry.info.order = order;
                    true
                } else {
                    false
                }
            };
            if updated
                && let Err(error) = state.changed()
                && let Some(scope) = weak_scope.upgrade()
            {
                scope.report_error(error)
            }
        });
        Ok(())
    }
}

type DocumentFactory = Box<dyn FnMut(TocContext) -> Box<dyn Widget>>;
pub struct TocDocument {
    context: TocContext,
    key: String,
    common: Common,
    offset: f64,
    content: Option<DocumentFactory>,
}
pub fn toc_document(context: TocContext, key: impl Into<String>) -> TocDocument {
    TocDocument {
        context,
        key: key.into(),
        common: Common::default(),
        offset: 0.,
        content: None,
    }
}
impl TocDocument {
    pub fn content<W: Widget>(
        mut self,
        mut value: impl FnMut(TocContext) -> W + 'static,
    ) -> Self {
        self.content = Some(Box::new(move |context| Box::new(value(context))));
        self
    }
    pub fn offset(mut self, value: f64) -> Self {
        self.offset = value;
        self
    }
    pub fn name(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
}
impl Widget for TocDocument {
    fn build(&mut self, scope: &Scope) -> Result {
        self.context.0.ensure_scope(scope)?;
        if !self.offset.is_finite() || !(0. ..=1000.).contains(&self.offset) {
            return Err(invalid("heading offset requires 0..=1000 finite pixels"));
        }
        if self.context.0.viewport.borrow().is_some() {
            return Err(invalid(
                "a heading context accepts one live scrolling document",
            ));
        }
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let weak_scope = scope.downgrade();
        let custom_class = self.common.class.clone();
        let classes = scope.bond(move || format!("sapid-toc-document {}", custom_class.get()));
        let context = self.context.clone();
        let custom = self.common.style.clone();
        let style = scope.bond(move || {
            crate::appearance::merge(
                Style::new()
                    .height(Length::Px(400.))
                    .width(Length::Percent(100.)),
                custom.get(),
            )
        });
        let ready = self.context.clone();
        let observe = self.context.clone();
        let resize = self.context.clone();
        let weak_resize = scope.downgrade();
        let mut document = crate::scroll_area(format!("toc-{}", self.key))
            .theme(self.common.theme.clone())
            .class(classes)
            .style(style)
            .node_ref(reference)
            .on_ready(move |_| {
                if let Some(scope) = weak_scope.upgrade() {
                    let result = (|| -> Result {
                        let node = reference.try_get()?.ok_or(glory::ui::Error::Gone)?;
                        for entry in ready.0.ordered() {
                            if entry.scope.upgrade().is_none() || !scope.contains_node(node, entry.node)? {
                                return Err(invalid("registered headings must belong to their scrolling document"));
                            }
                        }
                        *ready.0.viewport.borrow_mut() = Some(Viewport {
                            node,
                            scope: scope.downgrade(),
                        });
                        ready.0.changed()
                    })();
                    if let Err(error) = result {
                        scope.report_error(error)
                    }
                }
            })
            .on_scroll_change(move |_| {
                if let Err(error) = observe.refresh()
                    && let Some(scope) = observe.0.weak.upgrade()
                {
                    scope.report_error(error)
                }
            })
            .on(EventKind::Resize, move |_| {
                if let Err(error) = resize.refresh()
                    && let Some(scope) = weak_resize.upgrade()
                {
                    scope.report_error(error)
                }
            });
        if let Some(name) = self.common.name.take() {
            document = document.name(name);
        }
        if let Some(mut factory) = self.content.take() {
            document = document.content(move |_| factory(context.clone()));
        }
        self.context.0.offset.set(self.offset);
        document.show_in(scope)?;
        let state = Rc::downgrade(&self.context.0);
        scope.on_cleanup(move || {
            if let Some(state) = state.upgrade() {
                state.cancel();
                state.viewport.borrow_mut().take();
                state.measured.set(false);
                let _ = state.active.try_revise(|value| *value = None);
            }
        })?;
        Ok(())
    }
}

struct NavRow {
    _owner: Scope,
    _mounted: glory::ui::Mounted,
    parent: Option<SemanticId>,
    row: NodeId,
    list: NodeId,
    link: NodeId,
    label: Cage<String>,
    children: Cage<bool>,
}
struct NavState {
    weak: WeakScope,
    context: TocContext,
    root: NodeId,
    rows: RefCell<BTreeMap<SemanticId, NavRow>>,
    focused: RefCell<Option<SemanticId>>,
    siblings: RefCell<BTreeMap<Option<SemanticId>, Vec<SemanticId>>>,
    pending_focus: Cage<Option<SemanticId>>,
    theme: crate::Theme,
    size: crate::Size,
    unstyled: bool,
    current: Lotus<Option<SemanticId>>,
}
impl NavState {
    fn remove_order(&self, id: &SemanticId, parent: &Option<SemanticId>) {
        let mut all = self.siblings.borrow_mut();
        if let Some(values) = all.get_mut(parent) {
            values.retain(|value| value != id);
            if values.is_empty() {
                all.remove(parent);
            }
        }
    }
    fn place(
        &self,
        scope: &Scope,
        id: &SemanticId,
        parent: Option<SemanticId>,
        before: Option<&SemanticId>,
    ) -> Result<bool> {
        let (old, node, target, next_node) = {
            let rows = self.rows.borrow();
            let row = &rows[id];
            (
                row.parent.clone(),
                row.row,
                parent.as_ref().map(|id| rows[id].list).unwrap_or(self.root),
                before.map(|id| rows[id].row),
            )
        };
        let unchanged = {
            let all = self.siblings.borrow();
            let values = &all[&old];
            let position = values
                .iter()
                .position(|value| value == id)
                .expect("live row order");
            old == parent && values.get(position + 1) == before
        };
        if unchanged {
            return Ok(false);
        }
        scope.place(node, Some(target), next_node)?;
        self.remove_order(id, &old);
        {
            let mut all = self.siblings.borrow_mut();
            let values = all.entry(parent.clone()).or_default();
            let position = before
                .map(|id| {
                    values
                        .iter()
                        .position(|value| value == id)
                        .expect("live next row")
                })
                .unwrap_or(values.len());
            values.insert(position, id.clone());
        }
        self.rows.borrow_mut().get_mut(id).expect("live row").parent = parent;
        Ok(true)
    }
    fn sync(self: &Rc<Self>, entries: Vec<TocEntry>) -> Result {
        let scope = self.weak.upgrade().ok_or(glory::ui::Error::Gone)?;
        let parents = model::parents(&entries);
        let focus = self.focused.borrow().clone();
        let existing = self.rows.borrow().keys().cloned().collect::<Vec<_>>();
        // Break only changed parent edges before retiring ancestors; unchanged
        // subtrees keep their actual order and never leave their live list.
        for (index, entry) in entries.iter().enumerate() {
            let parent = parents[index].map(|i| entries[i].id.clone());
            let old = self
                .rows
                .borrow()
                .get(&entry.id)
                .map(|row| row.parent.clone());
            if old
                .as_ref()
                .is_some_and(|old| old.is_some() && *old != parent)
            {
                self.place(&scope, &entry.id, None, None)?;
            }
        }
        let mut retired = Vec::new();
        for id in existing {
            if !entries.iter().any(|entry| entry.id == id)
                && let Some(row) = self.rows.borrow_mut().remove(&id)
            {
                self.remove_order(&id, &row.parent);
                retired.push(row);
            }
        }
        drop(retired);
        for entry in &entries {
            if self.rows.borrow().contains_key(&entry.id) {
                continue;
            }
            let owner = scope.child_in(self.root)?;
            let row_ref = owner.cage(None);
            let list_ref = owner.cage(None);
            let link_ref = owner.cage(None);
            let label = owner.cage(entry.label.clone());
            let children = owner.cage(false);
            let active = self.current.clone();
            let key = entry.id.clone();
            let current = owner.bond(move || {
                active
                    .get()
                    .filter(|id| id == &key)
                    .map(|_| Current::Location)
            });
            let context = Rc::downgrade(&self.context.0);
            let key = entry.id.clone();
            let weak_scope = owner.downgrade();
            let observed = Rc::downgrade(self);
            let focus_key = entry.id.clone();
            let mut link = crate::anchor()
                .href(format!("#{}", entry.id.as_str()))
                .label(label)
                .current(current)
                .node_ref(link_ref)
                .theme(self.theme.clone())
                .size(self.size)
                .on_navigate(move |_| {
                    if let Some(state) = context.upgrade() {
                        let context = TocContext(state);
                        if let Err(error) = context.reveal(&key)
                            && let Some(scope) = weak_scope.upgrade()
                        {
                            scope.report_error(error)
                        }
                    }
                })
                .on(EventKind::FocusChanged, move |event| {
                    if let Some(state) = observed.upgrade()
                        && let Event::FocusChanged(focused) = event
                    {
                        if focused {
                            *state.focused.borrow_mut() = Some(focus_key.clone());
                        } else if state.focused.borrow().as_ref() == Some(&focus_key) {
                            state.focused.borrow_mut().take();
                        }
                    }
                });
            if self.unstyled {
                link = link.unstyled();
            } else {
                let background = self.theme.accent_soft;
                let radius = self.theme.radius;
                link = link.style(owner.bond(move || {
                    let selected = current.get().is_some();
                    Style {
                        background: Some(if selected {
                            background
                        } else {
                            glory::style::Color(0, 0, 0, 0)
                        }),
                        font_weight: Some(if selected { 700. } else { 400. }),
                        radius: Some(radius),
                        padding: Some(glory::style::Edges::all(4.)),
                        ..Style::new()
                    }
                }));
            }
            let nested = glory::ui::view()
                .role(Role::List)
                .class("sapid-toc-children")
                .node_ref(list_ref)
                .style(composite::column(6.).padding(glory::style::Edges {
                    left: 16.,
                    top: 6.,
                    right: 0.,
                    bottom: 0.,
                }))
                .semantics(owner.bond(move || {
                    Some(Box::new(Semantics {
                        hidden: !children.get(),
                        ..Default::default()
                    }))
                }));
            let mounted = glory::ui::view()
                .role(Role::ListItem)
                .class("sapid-toc-item")
                .node_ref(row_ref)
                .fill((link, nested))
                .mount(&owner)?;
            let row = NavRow {
                row: row_ref.get().ok_or(glory::ui::Error::Gone)?,
                list: list_ref.get().ok_or(glory::ui::Error::Gone)?,
                link: link_ref.get().ok_or(glory::ui::Error::Gone)?,
                _owner: owner,
                _mounted: mounted,
                parent: None,
                label,
                children,
            };
            self.rows.borrow_mut().insert(entry.id.clone(), row);
            self.siblings
                .borrow_mut()
                .entry(None)
                .or_default()
                .push(entry.id.clone());
        }
        let mut next = BTreeMap::<Option<SemanticId>, SemanticId>::new();
        let mut moved = false;
        for (index, entry) in entries.iter().enumerate().rev() {
            let (label, count) = {
                let rows = self.rows.borrow();
                let row = &rows[&entry.id];
                (row.label, row.children)
            };
            if label.get() != entry.label {
                label.try_revise(|value| *value = entry.label.clone())?;
            }
            let has_children = parents.contains(&Some(index));
            if count.get() != has_children {
                count.try_revise(|value| *value = has_children)?;
            }
            let parent = parents[index].map(|i| entries[i].id.clone());
            moved |= self.place(&scope, &entry.id, parent.clone(), next.get(&parent))?;
            next.insert(parent, entry.id.clone());
        }
        if let Some(focus) = focus {
            if self.rows.borrow().contains_key(&focus) {
                if moved {
                    self.pending_focus
                        .try_revise(|value| *value = Some(focus))?;
                }
            } else {
                self.focused.borrow_mut().take();
                let target = entries.first().map(|entry| entry.id.clone());
                self.pending_focus.try_revise(|value| *value = target)?;
            }
        }
        Ok(())
    }
}

pub struct TableOfContents {
    context: TocContext,
    key: String,
    common: Common,
    current: Option<Lotus<Option<SemanticId>>>,
}
pub fn table_of_contents(context: TocContext, key: impl Into<String>) -> TableOfContents {
    TableOfContents {
        context,
        key: key.into(),
        common: Common::default(),
        current: None,
    }
}
impl TableOfContents {
    pub fn current(mut self, value: impl Into<Lotus<Option<SemanticId>>>) -> Self {
        self.current = Some(value.into());
        self
    }
    pub fn name(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn size(mut self, value: crate::Size) -> Self {
        self.common.size = value;
        self
    }
    pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
}
impl Widget for TableOfContents {
    fn build(&mut self, scope: &Scope) -> Result {
        self.context.0.ensure_scope(scope)?;
        let id = composite::id("toc", &self.context.0.key, "navigation", &self.key)?;
        let name = self
            .common
            .name
            .take()
            .unwrap_or_else(|| "On this page".into());
        let name = composite::validated(scope, &name, |value| model::validate_label(value))?;
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let list = scope.cage(None);
        let (classes, style) =
            self.common
                .appearance(scope, "sapid-table-of-contents", Style::new());
        glory::ui::view()
            .role(Role::Navigation)
            .name(name)
            .class(classes)
            .style(style)
            .node_ref(reference)
            .semantics(composite::metadata(Semantics {
                id: Some(id),
                ..Default::default()
            }))
            .fill(
                glory::ui::view()
                    .role(Role::List)
                    .class("sapid-toc-list")
                    .style(composite::column(6.))
                    .node_ref(list),
            )
            .show_in(scope)?;
        let active = self
            .current
            .take()
            .unwrap_or_else(|| self.context.current(scope).into());
        let state = Rc::new(NavState {
            weak: scope.downgrade(),
            context: self.context.clone(),
            root: list.get().ok_or(glory::ui::Error::Gone)?,
            rows: RefCell::new(BTreeMap::new()),
            focused: RefCell::new(None),
            siblings: RefCell::new(BTreeMap::new()),
            pending_focus: scope.cage(None),
            theme: self.common.theme.clone(),
            size: self.common.size,
            unstyled: self.common.unstyled,
            current: active,
        });
        let observe = state.clone();
        let entries = self.context.entries(scope);
        let weak = scope.downgrade();
        scope.structural(move || {
            let entries = entries.get();
            if let Err(error) = observe.sync(entries)
                && let Some(scope) = weak.upgrade()
            {
                scope.report_error(error)
            }
        });
        let focus = Rc::downgrade(&state);
        let weak = scope.downgrade();
        let pending = state.pending_focus;
        scope.effect(move || {
            let Some(key) = pending.get() else { return };
            if let (Some(state), Some(scope)) = (focus.upgrade(), weak.upgrade()) {
                let result = (|| -> Result {
                    pending.try_revise(|value| *value = None)?;
                    let node = state.rows.borrow().get(&key).map(|row| row.link);
                    composite::focus_node(&scope, node)
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                }
            }
        });
        let cleanup = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(state) = cleanup.upgrade() {
                let rows = std::mem::take(&mut *state.rows.borrow_mut());
                state.siblings.borrow_mut().clear();
                drop(rows);
            }
        })?;
        Ok(())
    }
}
