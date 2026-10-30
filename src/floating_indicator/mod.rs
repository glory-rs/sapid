//! A bounded decoration following registered semantic boxes without focus edits.
use crate::appearance::{Common, configure, merge, observe};
use crate::composite::{self, Callback};
use glory::render::{
    Event, EventKind, GraphicProps, GraphicStroke, GraphicTransition, NodeId, Query, QueryToken,
    QueryValue, Rect, Semantics, StrokeCap,
};
use glory::style::{Color, Length, PointerEvents, Position, Style};
use glory::ui::{Children, IntoChildren, Result, WeakScope, Widget};
use glory::{Bond, Cage, Scope, Vary};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};
mod model;
pub use model::MAX_FLOATING_TARGETS;
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn key(value: &str) -> Result {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(invalid(
            "floating target keys require 1..=64 ASCII letters/digits/_/-",
        ));
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
struct Snapshot {
    geometry: Option<model::Geometry>,
    transition: bool,
}
#[derive(Clone)]
struct Target {
    node: NodeId,
    owner: glory::core::InstanceId,
    serial: u64,
}
struct Batch {
    ticket: u64,
    key: String,
    serial: u64,
    tokens: Vec<QueryToken>,
    remaining: usize,
    area: Option<Rect>,
    target: Option<Rect>,
    repeat: bool,
}
struct Controller {
    weak: WeakScope,
    owner: glory::core::InstanceId,
    container: Cage<Option<NodeId>>,
    targets: RefCell<BTreeMap<String, Target>>,
    next_target: Cell<u64>,
    pulse: Cage<()>,
    active: Cage<Option<String>>,
    visible: Vary<bool>,
    snapshot: Cage<Snapshot>,
    pending: Cage<usize>,
    task: Cell<Option<glory::core::TaskToken>>,
    scheduled: Cell<bool>,
    batch: RefCell<Option<Batch>>,
    next: Cell<u64>,
    callback: Callback<Option<Rect>>,
}
#[derive(Clone)]
pub struct FloatingIndicatorContext {
    inner: Rc<Controller>,
}
impl FloatingIndicatorContext {
    pub fn refresh(&self) -> Result {
        self.inner.schedule()
    }
    pub fn registered(&self) -> usize {
        self.inner.targets.borrow().len()
    }
    pub fn pending_queries(&self, scope: &Scope) -> Bond<usize> {
        let pending = self.inner.pending;
        scope.bond(move || pending.get())
    }
}
impl Controller {
    fn scope(&self) -> Result<Scope> {
        if !self.owner.is_alive() {
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
        self.scheduled.set(false);
        if let Some(task) = self.task.take() {
            task.cancel();
        }
        let batch = self.batch.borrow_mut().take();
        if let (Some(batch), Some(scope)) = (batch, self.weak.upgrade()) {
            for token in batch.tokens {
                let _ = scope.cancel_request(token);
            }
        }
        let _ = self.pending.try_revise(|value| *value = 0);
    }
    fn hide(&self) -> Result {
        let old = glory::core::untracked(|| self.snapshot.try_get())?;
        if old.geometry.is_some() {
            self.snapshot.try_revise(|value| {
                *value = Snapshot {
                    geometry: None,
                    transition: false,
                }
            })?;
            self.callback.emit(self.owner, None)
        }
        Ok(())
    }
    fn changed(self: &Rc<Self>) -> Result {
        self.cancel();
        self.pulse.try_revise(|_| ())?;
        Ok(())
    }
    fn unregister(self: &Rc<Self>, key: &str, serial: u64) {
        let removed = {
            let mut targets = self.targets.borrow_mut();
            if targets
                .get(key)
                .is_some_and(|target| target.serial == serial)
            {
                targets.remove_entry(key)
            } else {
                None
            }
        };
        if removed.is_some()
            && self.owner.is_alive()
            && let Err(error) = self.changed()
        {
            self.report(error)
        }
        drop(removed);
    }
    fn schedule(self: &Rc<Self>) -> Result {
        let scope = self.scope()?;
        if !self.visible.try_get()? || !scope.capabilities()?.measure {
            self.cancel();
            return self.hide();
        }
        if let Some(batch) = self.batch.borrow_mut().as_mut() {
            batch.repeat = true;
            return Ok(());
        }
        if self.scheduled.replace(true) {
            return Ok(());
        }
        let weak = Rc::downgrade(self);
        let task = scope.spawn(async move {
            if let Some(state) = weak.upgrade() {
                state.task.set(None);
                state.scheduled.set(false);
                if let Err(error) = state.measure() {
                    state.cancel();
                    state.report(error)
                }
            }
        })?;
        self.task.set(Some(task));
        Ok(())
    }
    fn measure(self: &Rc<Self>) -> Result {
        let scope = self.scope()?;
        let Some(key) = self.active.try_get()? else {
            return self.hide();
        };
        let target = self.targets.borrow().get(&key).cloned();
        let Some(target) = target else {
            return self.hide();
        };
        if !target.owner.is_alive() {
            return self.hide();
        }
        let area = self.container.try_get()?.ok_or(glory::ui::Error::Gone)?;
        if !scope.contains_node(area, target.node)? {
            self.hide()?;
            return Err(invalid(
                "floating target moved outside its live measurement area",
            ));
        }
        if !scope.is_active(area)? || !scope.is_active(target.node)? {
            return self.hide();
        }
        let ticket = self
            .next
            .get()
            .checked_add(1)
            .ok_or_else(|| invalid("floating observation identity exhausted"))?;
        self.next.set(ticket);
        *self.batch.borrow_mut() = Some(Batch {
            ticket,
            key,
            serial: target.serial,
            tokens: Vec::with_capacity(2),
            remaining: 2,
            area: None,
            target: None,
            repeat: false,
        });
        self.pending.try_revise(|value| *value = 2)?;
        for (index, node) in [area, target.node].into_iter().enumerate() {
            let weak = Rc::downgrade(self);
            let token = scope.request(Query::Bounds(node), move |result| {
                if let Some(state) = weak.upgrade()
                    && let Err(error) = state.reply(ticket, index, result)
                {
                    state.cancel();
                    state.report(error)
                }
            })?;
            if let Some(batch) = self
                .batch
                .borrow_mut()
                .as_mut()
                .filter(|batch| batch.ticket == ticket)
            {
                batch.tokens.push(token)
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
        let (complete, remaining) = {
            let mut slot = self.batch.borrow_mut();
            let Some(batch) = slot.as_mut().filter(|batch| batch.ticket == ticket) else {
                return Ok(());
            };
            let QueryValue::Bounds(rect) = result? else {
                return Err(invalid("floating query response mismatch"));
            };
            if index == 0 {
                batch.area = Some(rect)
            } else {
                batch.target = Some(rect)
            }
            batch.remaining -= 1;
            let remaining = batch.remaining;
            (if remaining == 0 { slot.take() } else { None }, remaining)
        };
        self.pending.try_revise(|value| *value = remaining)?;
        let Some(batch) = complete else { return Ok(()) };
        self.scope()?;
        if !self
            .targets
            .borrow()
            .get(&batch.key)
            .is_some_and(|target| target.serial == batch.serial && target.owner.is_alive())
            || self.active.try_get()?.as_deref() != Some(batch.key.as_str())
        {
            return Ok(());
        }
        let geometry = model::geometry(
            batch
                .area
                .ok_or_else(|| invalid("floating area bounds missing"))?,
            batch
                .target
                .ok_or_else(|| invalid("floating target bounds missing"))?,
        )?;
        let old = self.snapshot.try_get()?;
        if old.geometry != geometry {
            self.snapshot.try_revise(|value| {
                *value = Snapshot {
                    geometry,
                    transition: old.geometry.is_some() && geometry.is_some(),
                }
            })?;
            self.callback
                .emit(self.owner, geometry.map(|geometry| geometry.target));
        }
        if batch.repeat && self.owner.is_alive() {
            self.schedule()?;
        }
        Ok(())
    }
}

pub struct FloatingIndicatorTarget {
    context: FloatingIndicatorContext,
    key: String,
    children: Children,
    class: Vary<String>,
    style: Vary<Style>,
    reference: Option<Cage<Option<NodeId>>>,
}
pub fn floating_indicator_target(
    context: FloatingIndicatorContext,
    key: impl Into<String>,
) -> FloatingIndicatorTarget {
    FloatingIndicatorTarget {
        context,
        key: key.into(),
        children: ().into_children(),
        class: "".into(),
        style: Style::new().into(),
        reference: None,
    }
}
impl FloatingIndicatorTarget {
    pub fn fill(mut self, value: impl IntoChildren) -> Self {
        self.children = value.into_children();
        self
    }
    pub fn class(mut self, value: impl Into<Vary<String>>) -> Self {
        self.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Vary<Style>>) -> Self {
        self.style = value.into();
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.reference = Some(value);
        self
    }
}
impl Widget for FloatingIndicatorTarget {
    fn build(&mut self, scope: &Scope) -> Result {
        key(&self.key)?;
        let state = &self.context.inner;
        state.scope()?;
        let area = state.container.try_get()?.ok_or(glory::ui::Error::Gone)?;
        if !scope.is_within(area)? {
            return Err(invalid(
                "floating targets must be constructed inside their measurement area",
            ));
        }
        {
            let targets = state.targets.borrow();
            if targets.len() >= MAX_FLOATING_TARGETS {
                return Err(invalid("floating indicator exceeds 128 current targets"));
            }
            if targets.contains_key(&self.key) {
                return Err(invalid("floating target key is already registered"));
            }
        }
        let reference = self.reference.unwrap_or_else(|| scope.cage(None));
        let class = self.class.clone();
        let weak = Rc::downgrade(state);
        glory::ui::view()
            .class(scope.bond(move || format!("sapid-floating-target {}", class.get())))
            .style({
                let style = self.style.clone();
                scope.bond(move || {
                    merge(
                        Style::new().display(glory::style::Display::InlineFlex),
                        style.get(),
                    )
                })
            })
            .node_ref(reference)
            .on(EventKind::Resize, move |_| {
                if let Some(state) = weak.upgrade()
                    && let Err(error) = state.schedule()
                {
                    state.report(error)
                }
            })
            .fill(std::mem::replace(&mut self.children, ().into_children()))
            .show_in(scope)?;
        let node = reference.try_get()?.ok_or(glory::ui::Error::Gone)?;
        let serial = state
            .next_target
            .get()
            .checked_add(1)
            .ok_or_else(|| invalid("floating registration identity exhausted"))?;
        state.next_target.set(serial);
        state.targets.borrow_mut().insert(
            self.key.clone(),
            Target {
                node,
                owner: scope.id(),
                serial,
            },
        );
        let weak = Rc::downgrade(state);
        let key = self.key.clone();
        scope.on_cleanup(move || {
            if let Some(state) = weak.upgrade() {
                state.unregister(&key, serial)
            }
        })?;
        state.changed()
    }
}

type Content = Box<dyn FnMut(FloatingIndicatorContext) -> Box<dyn Widget>>;
pub struct FloatingIndicator {
    common: Common,
    key: String,
    active: Vary<Option<String>>,
    visible: Vary<bool>,
    animated: Vary<bool>,
    duration: Vary<u16>,
    revision: Vary<u64>,
    color: Option<Vary<Color>>,
    thickness: f64,
    content: Option<Content>,
    reference: Option<Cage<Option<FloatingIndicatorContext>>>,
    changed: Option<Box<dyn FnMut(Option<Rect>)>>,
}
pub fn floating_indicator(key: impl Into<String>) -> FloatingIndicator {
    FloatingIndicator {
        common: Common::default(),
        key: key.into(),
        active: None.into(),
        visible: true.into(),
        animated: true.into(),
        duration: 180u16.into(),
        revision: 0u64.into(),
        color: None,
        thickness: 2.,
        content: None,
        reference: None,
        changed: None,
    }
}
impl FloatingIndicator {
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
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
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
    pub fn active(mut self, value: impl Into<Vary<Option<String>>>) -> Self {
        self.active = value.into();
        self
    }
    pub fn visible(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.visible = value.into();
        self
    }
    pub fn animated(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.animated = value.into();
        self
    }
    pub fn duration(mut self, value: impl Into<Vary<u16>>) -> Self {
        self.duration = value.into();
        self
    }
    pub fn measurement_revision(mut self, value: impl Into<Vary<u64>>) -> Self {
        self.revision = value.into();
        self
    }
    pub fn color(mut self, value: impl Into<Vary<Color>>) -> Self {
        self.color = Some(value.into());
        self
    }
    pub fn thickness(mut self, value: f64) -> Self {
        self.thickness = value;
        self
    }
    pub fn content<W: Widget>(
        mut self,
        mut factory: impl FnMut(FloatingIndicatorContext) -> W + 'static,
    ) -> Self {
        self.content = Some(Box::new(move |context| Box::new(factory(context))));
        self
    }
    pub fn context_ref(mut self, value: Cage<Option<FloatingIndicatorContext>>) -> Self {
        self.reference = Some(value);
        self
    }
    pub fn on_geometry_change(mut self, value: impl FnMut(Option<Rect>) + 'static) -> Self {
        self.changed = Some(Box::new(value));
        self
    }
    pub fn on(mut self, kind: EventKind, callback: impl FnMut(Event) + 'static) -> Self {
        self.common
            .handlers
            .entry(kind)
            .or_default()
            .push(Box::new(callback));
        self
    }
}
impl Widget for FloatingIndicator {
    fn build(&mut self, scope: &Scope) -> Result {
        composite::id("floating", &self.key, "root", "root")?;
        if !self.thickness.is_finite() || !(0.000001..=16.).contains(&self.thickness) {
            return Err(invalid(
                "floating stroke width must be finite and in 0.000001..=16",
            ));
        }
        let active = composite::validated(scope, &self.active, |value| {
            if let Some(value) = value {
                key(value)?;
            }
            Ok(())
        })?;
        let duration = composite::validated(scope, &self.duration, |value| {
            Ok(GraphicTransition {
                duration_millis: *value,
            }
            .validate()?)
        })?;
        let container = scope.cage(None);
        let pulse = scope.cage(());
        let snapshot = scope.cage(Snapshot {
            geometry: None,
            transition: false,
        });
        let pending = scope.cage(0);
        let state = Rc::new(Controller {
            weak: scope.downgrade(),
            owner: scope.id(),
            container,
            targets: RefCell::new(BTreeMap::new()),
            next_target: Cell::new(0),
            pulse,
            active,
            visible: self.visible.clone(),
            snapshot,
            pending,
            task: Cell::new(None),
            scheduled: Cell::new(false),
            batch: RefCell::new(None),
            next: Cell::new(0),
            callback: Callback::new(self.changed.take()),
        });
        let weak = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(state) = weak.upgrade() {
                state.cancel();
                state.callback.clear();
                let targets = std::mem::take(&mut *state.targets.borrow_mut());
                drop(targets);
            }
        })?;
        let context = FloatingIndicatorContext {
            inner: state.clone(),
        };
        let mut factory = self.content.take();
        let color = self
            .color
            .clone()
            .unwrap_or_else(|| self.common.theme.accent.into());
        let stroke = self.thickness;
        let painted = !self.common.unstyled || self.color.is_some();
        let animated = self.animated.clone();
        let graphic = glory::ui::graphic()
            .class("sapid-floating-indicator-shape")
            .style(Style {
                width: Some(Length::Percent(100.)),
                height: Some(Length::Percent(100.)),
                pointer_events: Some(PointerEvents::None),
                ..Style::new()
            })
            .drawing(scope.bond(move || {
                let current = snapshot.get();
                let geometry = current.geometry;
                let rect = geometry.map_or(
                    Rect {
                        x: 0.,
                        y: 0.,
                        width: 0.,
                        height: 0.,
                    },
                    |geometry| geometry.target,
                );
                GraphicProps {
                    width: geometry.map_or(1., |geometry| geometry.width),
                    height: geometry.map_or(1., |geometry| geometry.height),
                    commands: model::rectangle(rect),
                    fill: None,
                    stroke: painted.then(|| GraphicStroke {
                        color: color.get(),
                        width: stroke,
                        cap: StrokeCap::Butt,
                    }),
                    alternative: None,
                    animation: None,
                    transition: (current.transition && animated.get() && duration.get() > 0).then(
                        || GraphicTransition {
                            duration_millis: duration.get(),
                        },
                    ),
                }
            }));
        let origin = glory::ui::view()
            .class("sapid-floating-area")
            .node_ref(container)
            .style(Style {
                position: Some(Position::Relative),
                ..Style::new()
            });
        let custom = self.common.style.clone();
        let unstyled = self.common.unstyled;
        let theme = self.common.theme.clone();
        let resize = Rc::downgrade(&state);
        let owner = scope.id();
        let mut generic = self
            .common
            .handlers
            .remove(&EventKind::Resize)
            .unwrap_or_default();
        let root_reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-floating-indicator",
            Style::new()
        )
        .node_ref(root_reference)
        .style(scope.bond(move || {
            merge(
                if unstyled {
                    Style::new()
                } else {
                    Style {
                        color: Some(theme.foreground),
                        ..Style::new()
                    }
                },
                custom.get(),
            )
        }))
        .on(EventKind::Resize, move |event| {
            if let Some(state) = resize.upgrade()
                && let Err(error) = state.schedule()
            {
                state.report(error)
            }
            observe(&mut generic, owner, event)
        });
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)?;
        let parent = scope.child_in(root_reference.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        origin.show_in(&parent)?;
        let area_scope = scope.child_in(container.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        if let Some(factory) = factory.as_mut() {
            factory(context.clone()).show_in(&area_scope)?;
        }
        glory::ui::view()
            .class("sapid-floating-indicator-marker")
            .style(Style {
                position: Some(Position::Absolute),
                top: Some(Length::Px(0.)),
                left: Some(Length::Px(0.)),
                width: Some(Length::Percent(100.)),
                height: Some(Length::Percent(100.)),
                pointer_events: Some(PointerEvents::None),
                ..Style::new()
            })
            .semantics(scope.bond(move || {
                composite::metadata(Semantics {
                    hidden: snapshot.get().geometry.is_none(),
                    ..Default::default()
                })
            }))
            .fill(graphic)
            .show_in(&area_scope)?;
        if let Some(reference) = self.reference {
            reference.try_revise(|value| *value = Some(context))?;
        }
        let refresh = Rc::downgrade(&state);
        let visible = self.visible.clone();
        let revision = self.revision.clone();
        scope.effect(move || {
            let _ = (active.get(), pulse.get(), visible.get(), revision.get());
            if let Some(state) = refresh.upgrade() {
                state.cancel();
                if let Err(error) = state.schedule() {
                    state.report(error)
                }
            }
        });
        scope.retain((parent, area_scope, state))?;
        Ok(())
    }
}
