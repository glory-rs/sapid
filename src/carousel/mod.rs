//! An index-addressed page window. At most seven slide Owners are mounted.
use crate::appearance::{Common, common_methods, merge, observe};
use crate::composite::Callback;
use crate::state::{Control, DelayFactory, OwnerDelay, Value};
use glory::render::{
    Direction, DragPhase, Event, EventKind, FocusMode, Navigation, NodeId, Orientation, Query,
    QueryToken, QueryValue, Semantics, SwipeCapture,
};
use glory::style::{
    BorderStyle, Display, Edges, FlexDirection, Length, Limit, Overflow, Style,
};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Scope, Vary};
use std::{cell::Cell, rc::Rc};
mod model;
pub use model::{MAX_CAROUSEL_COUNT, MAX_CAROUSEL_OVERSCAN};
use model::{Page, Step};
type Factory = Box<dyn FnMut(usize) -> Box<dyn Widget>>;
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}

pub struct Carousel {
    common: Common,
    count: Vary<usize>,
    current: Value<usize>,
    looping: bool,
    overscan: usize,
    axis: Orientation,
    direction: Direction,
    changed: Option<Box<dyn FnMut(usize)>>,
    slide: Option<Factory>,
    delay: Option<DelayFactory>,
    paused: Vary<bool>,
    hover_pause: bool,
}
pub fn carousel() -> Carousel {
    Carousel {
        common: Common {
            name: Some("Carousel".into()),
            focus: FocusMode::TabStop,
            ..Default::default()
        },
        count: 0usize.into(),
        current: Value::default(),
        looping: false,
        overscan: 1,
        axis: Orientation::Horizontal,
        direction: Direction::Ltr,
        changed: None,
        slide: None,
        delay: None,
        paused: false.into(),
        hover_pause: true,
    }
}
impl Carousel {
    common_methods!();
    pub fn count(mut self, value: impl Into<Vary<usize>>) -> Self {
        self.count = value.into();
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
    pub fn looping(mut self, value: bool) -> Self {
        self.looping = value;
        self
    }
    pub fn overscan(mut self, value: usize) -> Self {
        self.overscan = value;
        self
    }
    pub fn orientation(mut self, value: Orientation) -> Self {
        self.axis = value;
        self
    }
    pub fn direction(mut self, value: Direction) -> Self {
        self.direction = value;
        self
    }
    pub fn on_current_change(mut self, callback: impl FnMut(usize) + 'static) -> Self {
        self.changed = Some(Box::new(callback));
        self
    }
    /// Called once for each newly mounted numeric index, under its short Owner.
    /// Retained indices preserve editors; leaving the seven-slide window retires them.
    pub fn render_slide<W: Widget>(
        mut self,
        mut factory: impl FnMut(usize) -> W + 'static,
    ) -> Self {
        self.slide = Some(Box::new(move |index| Box::new(factory(index))));
        self
    }
    /// Supply one actual delay future per tick. No timer/task exists by default.
    /// Pause controls and focus containment revoke the current wait immediately.
    pub fn autoplay(mut self, delay: impl Fn() -> glory::core::LocalFuture + 'static) -> Self {
        self.delay = Some(Rc::new(delay));
        self
    }
    pub fn paused(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.paused = value.into();
        self
    }
    /// Hover observation is a real host capability. Native callers can explicitly
    /// disable it while retaining focus/explicit pause and their injected clock.
    pub fn pause_on_hover(mut self, value: bool) -> Self {
        self.hover_pause = value;
        self
    }
}
struct Controller {
    weak: WeakScope,
    owner: glory::core::InstanceId,
    page: Cage<Page>,
    control: Control<usize>,
    enabled: Vary<bool>,
    changed: Callback<usize>,
    root: Cage<Option<NodeId>>,
    extent: Cell<Option<f64>>,
    query: Cell<Option<QueryToken>>,
    task: Cell<Option<glory::core::TaskToken>>,
    measuring: Cell<bool>,
    repeat: Cell<bool>,
    swipe: Cell<Option<(Page, f64, u64)>>,
    axis: Orientation,
    direction: Direction,
    delay: Rc<OwnerDelay>,
    pulse: Cage<()>,
}
impl Controller {
    fn report(&self, error: glory::ui::Error) {
        if let Some(scope) = self.weak.upgrade() {
            scope.report_error(error)
        }
    }
    fn propose(&self, proposal: usize) -> Result {
        if !self.owner.is_alive() || !self.enabled.try_get()? {
            return Ok(());
        }
        let page = glory::core::untracked(|| self.page.try_get())?;
        if proposal >= page.count || proposal == page.current {
            return Ok(());
        }
        self.control.accept(&proposal)?;
        self.changed.emit(self.owner, proposal);
        Ok(())
    }
    fn step(&self, step: Step) -> Result {
        let page = glory::core::untracked(|| self.page.try_get())?;
        if let Some(next) = page.propose(step) {
            self.propose(next)?;
        }
        Ok(())
    }
    fn cancel_measurement(&self) {
        self.measuring.set(false);
        self.repeat.set(false);
        if let Some(task) = self.task.take() {
            task.cancel();
        }
        if let Some(token) = self.query.take()
            && let Some(scope) = self.weak.upgrade()
        {
            let _ = scope.cancel_request(token);
        }
        self.extent.set(None);
    }
    fn measure(self: &Rc<Self>) -> Result {
        if self.measuring.replace(true) {
            self.repeat.set(true);
            return Ok(());
        }
        self.repeat.set(false);
        let weak = Rc::downgrade(self);
        let scope = self.weak.upgrade().ok_or(glory::ui::Error::Gone)?;
        let task = scope.spawn(async move {
            let Some(state) = weak.upgrade() else {
                return;
            };
            state.task.take();
            let result = (|| -> Result {
                let Some(scope) = state.weak.upgrade() else {
                    return Ok(());
                };
                let Some(node) = state.root.try_get()? else {
                    return Ok(());
                };
                let weak = Rc::downgrade(&state);
                let token = scope.request(Query::Bounds(node), move |result| {
                    let Some(state) = weak.upgrade() else {
                        return;
                    };
                    state.query.set(None);
                    state.measuring.set(false);
                    if state.repeat.replace(false) {
                        if let Err(error) = state.measure() {
                            state.report(error);
                        }
                        return;
                    }
                    if let Ok(QueryValue::Bounds(rect)) = result {
                        let value = match state.axis {
                            Orientation::Horizontal => rect.width,
                            Orientation::Vertical => rect.height,
                        };
                        state
                            .extent
                            .set((value.is_finite() && value > 0.).then_some(value));
                    }
                })?;
                state.query.set(Some(token));
                Ok(())
            })();
            if let Err(error) = result {
                state.measuring.set(false);
                state.report(error);
            }
        })?;
        self.task.set(Some(task));
        Ok(())
    }
}
impl Widget for Carousel {
    fn build(&mut self, scope: &Scope) -> Result {
        let owner = scope.child()?;
        let controlled = self.current.source.is_some();
        let current = std::mem::take(&mut self.current).mount(&owner);
        let initial = Page {
            count: self.count.get(),
            current: current.source.get(),
            looping: self.looping,
            overscan: self.overscan,
        }
        .validate()?;
        let page = owner.cage(initial);
        let root = owner.cage(None);
        let state = Rc::new(Controller {
            weak: owner.downgrade(),
            owner: owner.id(),
            page,
            control: current.clone(),
            enabled: self.common.enabled.clone(),
            changed: Callback::new(self.changed.take()),
            root,
            extent: Cell::new(None),
            query: Cell::new(None),
            task: Cell::new(None),
            measuring: Cell::new(false),
            repeat: Cell::new(false),
            swipe: Cell::new(None),
            axis: self.axis,
            direction: self.direction,
            delay: OwnerDelay::new(&owner)?,
            pulse: owner.cage(()),
        });
        let cleanup = Rc::downgrade(&state);
        owner.on_cleanup(move || {
            if let Some(state) = cleanup.upgrade() {
                state.cancel_measurement();
                state.delay.cancel();
                state.changed.clear();
                state.swipe.set(None);
            }
        })?;
        let count = self.count.clone();
        let looping = self.looping;
        let overscan = self.overscan;
        let reconcile = state.clone();
        owner.structural(move || {
            let result = (|| -> Result {
                let count = count.get();
                let mut index = current.source.get();
                if !controlled && index >= count {
                    index = count.saturating_sub(1);
                    current.accept(&index)?;
                }
                let next = Page {
                    count,
                    current: index,
                    looping,
                    overscan,
                }
                .validate()?;
                if page.try_get()? != next {
                    reconcile.swipe.set(None);
                    page.try_revise(|value| *value = next)?;
                }
                Ok(())
            })();
            if let Err(error) = result {
                reconcile.report(error);
            }
        });
        let mut base = Style::new()
            .min_width(Length::Px(0.))
            .max_width(Limit::Percent(100.))
            .overflow_x(Overflow::Hidden);
        if !self.common.unstyled {
            base.background = Some(self.common.theme.surface);
            base.color = Some(self.common.theme.foreground);
            base.border_width = Some(Edges::all(1.));
            base.border_style = Some(BorderStyle::Solid);
            base.border_color = Some(self.common.theme.border);
            base.radius = Some(self.common.theme.radius);
        }
        let external = self.common.node_ref;
        self.common.node_ref = Some(root);
        let enabled = self.common.enabled.clone();
        let axis = self.axis;
        let direction = self.direction;
        let (classes, styles) = self.common.appearance(&owner, "sapid-carousel", base);
        let style = owner.bond(move || {
            merge(
                Style::new()
                    .min_width(Length::Px(0.))
                    .max_width(Limit::Percent(100.))
                    .overflow_x(Overflow::Hidden),
                styles.get(),
            )
        });
        let mut container = glory::ui::view()
            .class(classes)
            .style(style)
            .focus(self.common.focus)
            .node_ref(root)
            .semantics(owner.bond(move || {
                Some(Box::new(Semantics {
                    navigation: Some(axis),
                    navigation_direction: direction,
                    swipe_capture: Some(SwipeCapture { axis, revision: 0 }),
                    disabled: !enabled.get(),
                    ..Default::default()
                }))
            }));
        if let Some(name) = self.common.name.take() {
            container = container.name(name);
        }
        let local = state.clone();
        let mut observers = self
            .common
            .handlers
            .remove(&EventKind::Navigate)
            .unwrap_or_default();
        container = container.on(EventKind::Navigate, move |event| {
            if let Event::Navigate { from, command } = event
                && local.root.try_get().ok().flatten() == Some(from)
            {
                let step = match command {
                    Navigation::Previous => Some(Step::Previous),
                    Navigation::Next => Some(Step::Next),
                    Navigation::First => Some(Step::First),
                    Navigation::Last => Some(Step::Last),
                    _ => None,
                };
                if let Some(step) = step
                    && let Err(error) = local.step(step)
                {
                    local.report(error);
                }
            }
            observe(&mut observers, local.owner, event);
        });
        let local = state.clone();
        let mut observers = self
            .common
            .handlers
            .remove(&EventKind::Swipe)
            .unwrap_or_default();
        container = container.on(EventKind::Swipe, move |event| {
            let result = (|| -> Result {
                if let Event::Swipe {
                    phase,
                    x,
                    y,
                    revision,
                    ..
                } = event
                {
                    let point = if local.axis == Orientation::Horizontal {
                        x
                    } else {
                        y
                    };
                    match phase {
                        DragPhase::Start => {
                            local
                                .swipe
                                .set(Some((local.page.try_get()?, point, revision)))
                        }
                        DragPhase::Cancel => local.swipe.set(None),
                        DragPhase::Drop => {
                            if let Some((start, position, captured)) = local.swipe.take()
                                && captured == revision
                                && start == local.page.try_get()?
                                && let Some(extent) = local.extent.get()
                            {
                                // Vertical pages use top-to-bottom order independently of text direction.
                                let direction = if local.axis == Orientation::Vertical {
                                    Direction::Ltr
                                } else {
                                    local.direction
                                };
                                if let Some(next) =
                                    start.swipe(point - position, extent, direction)?
                                {
                                    local.propose(next)?;
                                }
                            }
                        }
                        DragPhase::Move => {}
                    }
                }
                Ok(())
            })();
            if let Err(error) = result {
                local.report(error);
            }
            observe(&mut observers, local.owner, event);
        });
        if owner.capabilities()?.measure {
            let local = state.clone();
            let mut observers = self
                .common
                .handlers
                .remove(&EventKind::Resize)
                .unwrap_or_default();
            container = container.on(EventKind::Resize, move |event| {
                local.swipe.set(None);
                local.extent.set(None);
                if let Err(error) = local.measure() {
                    local.report(error);
                }
                observe(&mut observers, local.owner, event);
            });
        }
        let paused = owner.cage(false);
        let focused = owner.cage(false);
        let hovered = owner.cage(false);
        if self.delay.is_some() {
            let mut observers = self
                .common
                .handlers
                .remove(&EventKind::FocusWithin)
                .unwrap_or_default();
            let id = owner.id();
            container = container.on(EventKind::FocusWithin, move |event| {
                if let Event::FocusWithin { focused: value, .. } = event {
                    focused.revise(|old| *old = value);
                }
                observe(&mut observers, id, event);
            });
            if self.hover_pause {
                let mut observers = self
                    .common
                    .handlers
                    .remove(&EventKind::PointerWithin)
                    .unwrap_or_default();
                let id = owner.id();
                container = container.on(EventKind::PointerWithin, move |event| {
                    if let Event::PointerWithin { phase, .. } = event {
                        hovered
                            .revise(|old| *old = phase != glory::render::PointerPhase::Leave);
                    }
                    observe(&mut observers, id, event);
                });
            }
        }
        for (kind, mut observers) in std::mem::take(&mut self.common.handlers) {
            let id = owner.id();
            container = container.on(kind, move |event| observe(&mut observers, id, event));
        }
        container.show_in(&owner)?;
        let node = root.try_get()?.ok_or(glory::ui::Error::Gone)?;
        if let Some(external) = external {
            external.try_revise(|value| *value = Some(node))?;
            owner.on_cleanup(move || {
                let _ = external.try_revise(|value| {
                    if *value == Some(node) {
                        *value = None;
                    }
                });
            })?;
        }
        let child = owner.child_in(node)?;
        let indices = owner.bond(move || page.get().window());
        let current_page = page;
        let mut factory = self
            .slide
            .take()
            .ok_or_else(|| invalid("carousel requires an index slide factory"))?;
        glory::ui::view()
            .class("sapid-carousel-viewport")
            .style(Style::new().min_width(Length::Px(0.)))
            .fill(glory::ui::each(
                indices,
                |index| *index,
                move |index, _| {
                    let index = index.get();
                    let mut content = Some(factory(index));
                    glory::ui::widget(move |scope| {
                        let current = current_page;
                        glory::ui::view()
                            .class(format!("sapid-carousel-slide sapid-carousel-slide-{index}"))
                            .name(scope.bond(move || {
                                format!("Slide {} of {}", index + 1, current.get().count)
                            }))
                            .semantics(scope.bond(move || {
                                let hidden = current.get().current != index;
                                Some(Box::new(Semantics {
                                    hidden,
                                    inert: hidden,
                                    ..Default::default()
                                }))
                            }))
                            .fill(content.take().ok_or(glory::ui::Error::Gone)?)
                            .show_in(scope)?;
                        Ok(())
                    })
                },
            ))
            .show_in(&child)?;
        let control_ref = owner.cage(None);
        glory::ui::view()
            .node_ref(control_ref)
            .class("sapid-carousel-controls")
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .gap(8.)
                    .padding(Edges::all(8.)),
            )
            .show_in(&child)?;
        let controls = child.child_in(control_ref.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        for (step, label, class) in [
            (Step::Previous, "Previous slide", "sapid-carousel-previous"),
            (Step::Next, "Next slide", "sapid-carousel-next"),
        ] {
            let enabled = self.common.enabled.clone();
            let local = state.clone();
            let mut button = crate::button()
                .label(label)
                .class(class)
                .theme(self.common.theme.clone())
                .size(self.common.size)
                .enabled(
                    owner.bond(move || enabled.get() && page.get().propose(step).is_some()),
                )
                .on_press(move || {
                    if let Err(error) = local.step(step) {
                        local.report(error);
                    }
                });
            if self.common.unstyled {
                button = button.unstyled();
            }
            button.show_in(&controls)?;
        }
        glory::ui::text()
            .class("sapid-carousel-position")
            .content(owner.bond(move || {
                let page = page.get();
                if page.count == 0 {
                    "No slides".into()
                } else {
                    format!("{} / {}", page.current + 1, page.count)
                }
            }))
            .show_in(&controls)?;
        if let Some(delay) = self.delay.clone() {
            let mut button = crate::button()
                .class("sapid-carousel-pause")
                .theme(self.common.theme.clone())
                .label(owner.bond(move || {
                    if paused.get() {
                        "Resume autoplay".to_owned()
                    } else {
                        "Pause autoplay".to_owned()
                    }
                }))
                .on_press(move || paused.revise(|value| *value = !*value));
            if self.common.unstyled {
                button = button.unstyled();
            }
            button.show_in(&controls)?;
            let source_pause = self.paused.clone();
            let local = state.clone();
            let weak = owner.downgrade();
            if owner.has_task_host()? {
                owner.effect(move || {
                    local.pulse.get();
                    let page = local.page.get();
                    let blocked = paused.get()
                        || focused.get()
                        || hovered.get()
                        || source_pause.get()
                        || !local.enabled.get()
                        || page.propose(Step::Next).is_none();
                    local.delay.cancel();
                    if !blocked && let Some(scope) = weak.upgrade() {
                        let state = local.clone();
                        if let Err(error) = local.delay.run(&scope, Some(&delay), move || {
                            state.step(Step::Next)?;
                            state.pulse.try_revise(|_| ())?;
                            Ok(())
                        }) {
                            local.report(error);
                        }
                    }
                });
            }
        }
        if owner.capabilities()?.measure {
            state.measure()?;
        }
        scope.retain(controls)?;
        scope.retain(child)?;
        scope.retain(owner)?;
        Ok(())
    }
}
