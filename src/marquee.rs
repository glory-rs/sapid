//! Finite text-only marquee; the host owns presentation time and preference.
use crate::appearance::{Common, configure, merge, observe};
use crate::composite;
use glory::render::{
    EventKind, NodeId, Query, QueryToken, QueryValue, Rect, Semantics, TranslationKind,
    TranslationMotion, TranslationOffset,
};
use glory::style::{Display, FlexDirection, Length, Overflow, Style, WhiteSpace};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Scope, Vary};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
pub const MAX_MARQUEE_TEXT_BYTES: usize = 4096;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarqueeDirection {
    #[default]
    Left,
    Right,
}
pub struct Marquee {
    common: Common,
    text: Vary<String>,
    paused: Vary<bool>,
    animated: Vary<bool>,
    period: Vary<u32>,
    revision: Vary<u64>,
    direction: MarqueeDirection,
    gap: f64,
}
pub fn marquee() -> Marquee {
    Marquee {
        common: Common::default(),
        text: "".into(),
        paused: false.into(),
        animated: true.into(),
        period: 8000.into(),
        revision: 0.into(),
        direction: MarqueeDirection::Left,
        gap: 24.,
    }
}
impl Marquee {
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
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
    pub fn text(mut self, value: impl Into<Vary<String>>) -> Self {
        self.text = value.into();
        self
    }
    pub fn paused(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.paused = value.into();
        self
    }
    /// Stop clears current motion; resuming begins a new loop at its start.
    pub fn animated(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.animated = value.into();
        self
    }
    pub fn period_millis(mut self, value: impl Into<Vary<u32>>) -> Self {
        self.period = value.into();
        self
    }
    pub fn measurement_revision(mut self, value: impl Into<Vary<u64>>) -> Self {
        self.revision = value.into();
        self
    }
    pub fn direction(mut self, value: MarqueeDirection) -> Self {
        self.direction = value;
        self
    }
    pub fn gap(mut self, value: f64) -> Self {
        self.gap = value;
        self
    }
    pub fn on(
        mut self,
        kind: EventKind,
        callback: impl FnMut(glory::render::Event) + 'static,
    ) -> Self {
        self.common
            .handlers
            .entry(kind)
            .or_default()
            .push(Box::new(callback));
        self
    }
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn text(value: &str) -> Result {
    if value.len() > MAX_MARQUEE_TEXT_BYTES {
        return Err(invalid("marquee text exceeds 4096 UTF-8 bytes"));
    }
    Ok(())
}
struct Batch {
    ticket: u64,
    tokens: Vec<QueryToken>,
    boxes: [Option<Rect>; 2],
    repeat: bool,
}
struct Measure {
    weak: WeakScope,
    owner: glory::core::InstanceId,
    root: Cage<Option<NodeId>>,
    first: Cage<Option<NodeId>>,
    distance: Cage<Option<f64>>,
    enabled: Vary<bool>,
    gap: f64,
    next: Cell<u64>,
    batch: RefCell<Option<Batch>>,
    task: Cell<Option<glory::core::TaskToken>>,
    scheduled: Cell<bool>,
}
impl Measure {
    fn report(&self, error: glory::ui::Error) {
        if let Some(scope) = self.weak.upgrade() {
            scope.report_error(error);
        }
    }
    fn stop(&self) {
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
    }
    fn reset(&self) -> Result {
        if glory::core::untracked(|| self.distance.try_get())?.is_some() {
            self.distance.try_revise(|value| *value = None)?;
        }
        Ok(())
    }
    fn schedule(self: &Rc<Self>) -> Result {
        let scope = self.weak.upgrade().ok_or(glory::ui::Error::Gone)?;
        if !self.owner.is_alive() || !self.enabled.try_get()? || !scope.capabilities()?.measure
        {
            self.stop();
            return self.reset();
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
                    state.stop();
                    let _ = state.reset();
                    state.report(error);
                }
            }
        })?;
        self.task.set(Some(task));
        Ok(())
    }
    fn measure(self: &Rc<Self>) -> Result {
        let scope = self.weak.upgrade().ok_or(glory::ui::Error::Gone)?;
        let root = self.root.try_get()?.ok_or(glory::ui::Error::Gone)?;
        let first = self.first.try_get()?.ok_or(glory::ui::Error::Gone)?;
        if !scope.is_active(root)? || !scope.is_active(first)? {
            return self.reset();
        }
        if !scope.contains_node(root, first)? {
            return Err(invalid(
                "marquee measurement item moved outside its viewport",
            ));
        }
        let ticket = self
            .next
            .get()
            .checked_add(1)
            .ok_or_else(|| invalid("marquee observation identity exhausted"))?;
        self.next.set(ticket);
        *self.batch.borrow_mut() = Some(Batch {
            ticket,
            tokens: Vec::with_capacity(2),
            boxes: [None, None],
            repeat: false,
        });
        for (index, node) in [root, first].into_iter().enumerate() {
            let weak = Rc::downgrade(self);
            let token = scope.request(Query::Bounds(node), move |result| {
                if let Some(state) = weak.upgrade()
                    && let Err(error) = state.reply(ticket, index, result)
                {
                    state.stop();
                    let _ = state.reset();
                    state.report(error);
                }
            })?;
            if let Some(batch) = self
                .batch
                .borrow_mut()
                .as_mut()
                .filter(|batch| batch.ticket == ticket)
            {
                batch.tokens.push(token);
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
        let complete = {
            let mut slot = self.batch.borrow_mut();
            let Some(batch) = slot.as_mut().filter(|batch| batch.ticket == ticket) else {
                return Ok(());
            };
            let QueryValue::Bounds(rect) = result? else {
                return Err(invalid("marquee query response mismatch"));
            };
            batch.boxes[index] = Some(rect);
            if batch.boxes.iter().all(Option::is_some) {
                slot.take()
            } else {
                None
            }
        };
        let Some(batch) = complete else { return Ok(()) };
        if !self.owner.is_alive() {
            return Ok(());
        }
        let root = batch.boxes[0].unwrap();
        let first = batch.boxes[1].unwrap();
        if [root.width, root.height, first.width, first.height]
            .into_iter()
            .any(|value| !value.is_finite() || !(0. ..=1_000_000.).contains(&value))
        {
            return Err(invalid(
                "marquee measured boxes require finite dimensions in 0..=1000000",
            ));
        }
        let distance = first.width + self.gap;
        if distance > 1_000_000. {
            return Err(invalid("marquee cycle exceeds 1000000 CSS pixels"));
        }
        let next = (root.width > 0. && first.height > 0. && first.width > root.width)
            .then_some(distance);
        if self.distance.try_get()? != next {
            self.distance.try_revise(|value| *value = next)?;
        }
        if batch.repeat {
            self.schedule()?;
        }
        Ok(())
    }
}
impl Widget for Marquee {
    fn build(&mut self, scope: &Scope) -> Result {
        if !self.gap.is_finite() || !(0. ..=4096.).contains(&self.gap) {
            return Err(invalid("marquee gap must be finite in 0..=4096"));
        }
        let content = composite::validated(scope, &self.text, |value| text(value))?;
        let period = composite::validated(scope, &self.period, |value| {
            if (16..=60_000).contains(value) {
                Ok(())
            } else {
                Err(invalid("marquee period must be in 16..=60000 milliseconds"))
            }
        })?;
        let distance = scope.cage(None);
        let outer = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let root = scope.cage(None);
        let first = scope.cage(None);
        let state = Rc::new(Measure {
            weak: scope.downgrade(),
            owner: scope.id(),
            root,
            first,
            distance,
            enabled: self.animated.clone(),
            gap: self.gap,
            next: Cell::new(0),
            batch: RefCell::new(None),
            task: Cell::new(None),
            scheduled: Cell::new(false),
        });
        let resize = Rc::downgrade(&state);
        let owner = scope.id();
        let mut observers = self
            .common
            .handlers
            .remove(&EventKind::Resize)
            .unwrap_or_default();
        let custom = self.common.style.clone();
        let themed = if self.common.unstyled {
            Style::new()
        } else {
            self.common.theme.label(self.common.size)
        };
        let mut widget = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-marquee",
            Style::new()
        )
        .node_ref(outer)
        .style(scope.bond(move || merge(themed.clone(), custom.get())))
        .on(EventKind::Resize, move |event| {
            if let Some(state) = resize.upgrade()
                && let Err(error) = state.schedule()
            {
                state.report(error);
            }
            observe(&mut observers, owner, event)
        });
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            widget = widget.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        widget.show_in(scope)?;
        let parent = scope.child_in(outer.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        crate::visually_hidden().content(content).show_in(&parent)?;
        glory::ui::view()
            .class("sapid-marquee-viewport")
            .node_ref(root)
            .style(
                Style::new()
                    .width(Length::Percent(100.))
                    .display(Display::Block)
                    .overflow_x(Overflow::Hidden)
                    .overflow_y(Overflow::Hidden),
            )
            .show_in(&parent)?;
        let area = scope.child_in(root.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        let animated = self.animated.clone();
        let paused = self.paused.clone();
        let direction = self.direction;
        let metadata = scope.bond(move || {
            Some(Box::new(Semantics {
                inert: true,
                translation: distance.get().filter(|_| animated.get()).map(|distance| {
                    let (from, to) = match direction {
                        MarqueeDirection::Left => (0., -distance),
                        MarqueeDirection::Right => (-distance, 0.),
                    };
                    TranslationMotion {
                        kind: TranslationKind::Loop {
                            from: TranslationOffset::new(from, 0.),
                            to: TranslationOffset::new(to, 0.),
                            period_millis: period.get(),
                        },
                        paused: paused.get(),
                    }
                }),
                ..Default::default()
            }))
        });
        let resize = Rc::downgrade(&state);
        let copies = self.animated.clone();
        glory::ui::view()
            .class("sapid-marquee-track")
            .semantics(metadata)
            .style(Style {
                display: Some(Display::InlineFlex),
                flex_direction: Some(FlexDirection::Row),
                gap: Some(self.gap),
                white_space: Some(WhiteSpace::NoWrap),
                ..Style::new()
            })
            .fill((
                glory::ui::view()
                    .class("sapid-marquee-first")
                    .node_ref(first)
                    .style(Style {
                        shrink: Some(0.),
                        ..Style::new()
                    })
                    .on(EventKind::Resize, move |_| {
                        if let Some(state) = resize.upgrade()
                            && let Err(error) = state.schedule()
                        {
                            state.report(error);
                        }
                    })
                    .fill(glory::ui::text().content(content).style(Style {
                        white_space: Some(WhiteSpace::NoWrap),
                        ..Style::new()
                    })),
                glory::ui::view()
                    .class("sapid-marquee-copy")
                    .semantics(scope.bond(move || {
                        Some(Box::new(Semantics {
                            hidden: distance.get().is_none() || !copies.get(),
                            ..Default::default()
                        }))
                    }))
                    .style(Style {
                        shrink: Some(0.),
                        ..Style::new()
                    })
                    .fill(glory::ui::text().content(content).style(Style {
                        white_space: Some(WhiteSpace::NoWrap),
                        ..Style::new()
                    })),
            ))
            .show_in(&area)?;
        let refresh = Rc::downgrade(&state);
        let animated = self.animated.clone();
        let revision = self.revision.clone();
        scope.effect(move || {
            let _ = (content.get(), animated.get(), revision.get());
            if let Some(state) = refresh.upgrade() {
                state.stop();
                if let Err(error) = state.reset() {
                    state.report(error);
                    return;
                }
                if let Err(error) = state.schedule() {
                    state.report(error);
                }
            }
        });
        let retired = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(state) = retired.upgrade() {
                state.stop();
            }
        })?;
        scope.retain(area)?;
        scope.retain(parent)?;
        scope.retain(state)?;
        Ok(())
    }
}
