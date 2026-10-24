//! Native scrolling viewports with bounded, ordered application intents.
use crate::appearance::{Common, configure, merge, observe};
use crate::composite::{self, Callback};
use glory::render::{
    Action, Event, EventKind, FocusMode, NodeId, Orientation, Query, QueryValue, ScrollAxes,
    ScrollCommand, ScrollMetrics, ScrollMovement, Semantics,
};
use glory::style::{Length, Overflow, Style};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Bond, Cage, Lotus, Scope};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
};

pub const MAX_PENDING_SCROLL_INTENTS: usize = 128;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scrollbars {
    #[default]
    Auto,
    Always,
}
#[derive(Clone, Copy, Debug)]
enum Intent {
    Command(ScrollCommand),
    To(f64, f64),
    By(f64, f64),
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn position(
    metrics: ScrollMetrics,
    intents: &mut VecDeque<Intent>,
    step: f64,
) -> Result<(f64, f64)> {
    let mut x = metrics.x;
    let mut y = metrics.y;
    let max_x = (metrics.content_width - metrics.viewport_width).max(0.0);
    let max_y = (metrics.content_height - metrics.viewport_height).max(0.0);
    // The shared ScrollTo contract currently has a nonnegative assignment domain.
    // Raw metrics preserve RTL/elastic negatives rather than inventing a range.
    if x < 0.0 || y < 0.0 {
        return Err(glory::render::Error::Unsupported(
            "scroll commands on negative RTL/elastic offsets",
        )
        .into());
    }
    while let Some(intent) = intents.pop_front() {
        match intent {
            Intent::To(next_x, next_y) => {
                x = next_x.clamp(0.0, max_x);
                y = next_y.clamp(0.0, max_y);
            }
            Intent::By(dx, dy) => {
                x = (x + dx).clamp(0.0, max_x);
                y = (y + dy).clamp(0.0, max_y);
            }
            Intent::Command(command) => {
                let (value, max, page) = match command.axis {
                    Orientation::Horizontal => (&mut x, max_x, metrics.viewport_width),
                    Orientation::Vertical => (&mut y, max_y, metrics.viewport_height),
                };
                *value = match command.movement {
                    ScrollMovement::StepNegative => (*value - step).max(0.0),
                    ScrollMovement::StepPositive => (*value + step).min(max),
                    ScrollMovement::PageNegative => (*value - page).max(0.0),
                    ScrollMovement::PagePositive => (*value + page).min(max),
                    ScrollMovement::Start => 0.0,
                    ScrollMovement::End => max,
                }
                .clamp(0.0, max);
            }
        }
    }
    Ok((x, y))
}
struct Controller {
    weak: WeakScope,
    owner: glory::core::InstanceId,
    live: Cell<bool>,
    viewport: Cage<Option<NodeId>>,
    metrics: Cage<Option<ScrollMetrics>>,
    axes: ScrollAxes,
    step: f64,
    queue: RefCell<VecDeque<Intent>>,
    requesting: Cell<bool>,
    observed: Callback<ScrollMetrics>,
}
impl Controller {
    fn ensure(&self) -> Result {
        if self.live.get() && self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn request(self: &Rc<Self>) -> Result {
        self.ensure()?;
        if self.requesting.replace(true) {
            return Ok(());
        }
        let result = (|| {
            let scope = self.weak.upgrade().ok_or(glory::ui::Error::Gone)?;
            let node = self.viewport.try_get()?.ok_or(glory::ui::Error::Gone)?;
            let controller = self.clone();
            scope.request(Query::ScrollMetrics(node), move |result| {
                controller.complete(result)
            })?;
            Ok(())
        })();
        if result.is_err() {
            self.requesting.set(false);
            let queue = self.queue.replace(VecDeque::new());
            drop(queue);
        }
        result
    }
    fn enqueue(self: &Rc<Self>, intent: Intent) -> Result {
        self.ensure()?;
        {
            let mut queue = self.queue.borrow_mut();
            if queue.len() >= MAX_PENDING_SCROLL_INTENTS {
                return Err(invalid("pending scroll intent budget exceeded"));
            }
            queue.push_back(intent);
        }
        self.request()
    }
    fn complete(
        self: &Rc<Self>,
        result: std::result::Result<QueryValue, glory::render::Error>,
    ) {
        if self.ensure().is_err() {
            return;
        }
        self.requesting.set(false);
        let Some(scope) = self.weak.upgrade() else {
            return;
        };
        let mut intents = std::mem::take(&mut *self.queue.borrow_mut());
        let result = (|| -> Result {
            let QueryValue::ScrollMetrics(metrics) = result? else {
                return Err(invalid("scroll metrics result mismatch"));
            };
            metrics.validate()?;
            let assignment = (|| -> Result {
                if intents.is_empty() {
                    return Ok(());
                }
                let (x, y) = position(metrics, &mut intents, self.step)?;
                let node = self.viewport.try_get()?.ok_or(glory::ui::Error::Gone)?;
                scope.action(Action::ScrollTo { node, x, y })
            })();
            self.metrics.try_revise(|value| *value = Some(metrics))?;
            self.observed.emit(self.owner, metrics);
            assignment
        })();
        if let Err(error) = result {
            scope.report_error(error);
        }
    }
    fn retire(&self) {
        self.live.set(false);
        self.observed.clear();
        let queue = self.queue.replace(VecDeque::new());
        drop(queue);
    }
}
#[derive(Clone)]
pub struct ScrollHandle {
    inner: Rc<Controller>,
    metrics: Bond<Option<ScrollMetrics>>,
}
impl ScrollHandle {
    pub fn metrics(&self) -> Bond<Option<ScrollMetrics>> {
        self.metrics
    }
    pub fn refresh(&self) -> Result {
        self.inner.request()
    }
    pub fn command(&self, command: ScrollCommand) -> Result {
        self.inner.ensure()?;
        if !self.inner.axes.allows(command.axis) {
            return Err(invalid("scroll command uses an excluded axis"));
        }
        self.inner.enqueue(Intent::Command(command))
    }
    pub fn scroll_to(&self, x: f64, y: f64) -> Result {
        self.inner.ensure()?;
        if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
            return Err(invalid("scroll position must be finite and nonnegative"));
        }
        if x != 0.0 && !self.inner.axes.allows(Orientation::Horizontal)
            || y != 0.0 && !self.inner.axes.allows(Orientation::Vertical)
        {
            return Err(invalid("scroll position uses an excluded axis"));
        }
        self.inner.enqueue(Intent::To(x, y))
    }
    pub fn scroll_by(&self, x: f64, y: f64) -> Result {
        self.inner.ensure()?;
        if !x.is_finite() || !y.is_finite() {
            return Err(invalid("scroll delta must be finite"));
        }
        if x != 0.0 && !self.inner.axes.allows(Orientation::Horizontal)
            || y != 0.0 && !self.inner.axes.allows(Orientation::Vertical)
        {
            return Err(invalid("scroll delta uses an excluded axis"));
        }
        self.inner.enqueue(Intent::By(x, y))
    }
}
#[derive(Clone)]
pub struct ScrollContext {
    pub scroll: ScrollHandle,
    pub theme: crate::Theme,
    pub size: crate::Size,
    pub unstyled: bool,
}
type Factory = Box<dyn FnMut(ScrollContext) -> Box<dyn Widget>>;
type ScrollChange = Box<dyn FnMut((f64, f64))>;
pub struct ScrollArea {
    common: Common,
    key: String,
    axes: ScrollAxes,
    scrollbars: Scrollbars,
    keyboard: bool,
    step: f64,
    content: Option<Factory>,
    content_ref: Option<Cage<Option<NodeId>>>,
    observed: Option<Box<dyn FnMut(ScrollMetrics)>>,
    scroll_change: Option<ScrollChange>,
    ready: Option<Box<dyn FnMut(ScrollHandle)>>,
}
pub fn scroll_area(key: impl Into<String>) -> ScrollArea {
    ScrollArea {
        common: Common::default(),
        key: key.into(),
        axes: ScrollAxes::Vertical,
        scrollbars: Scrollbars::Auto,
        keyboard: true,
        step: 40.0,
        content: None,
        content_ref: None,
        observed: None,
        scroll_change: None,
        ready: None,
    }
}
macro_rules! methods {
    () => {
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
        pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
            self.common.class = value.into();
            self
        }
        pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
            self.common.style = value.into();
            self
        }
        pub fn name(mut self, value: impl Into<Lotus<String>>) -> Self {
            self.common.name = Some(value.into());
            self
        }
        pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
            self.common.node_ref = Some(value);
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
    };
}
impl ScrollArea {
    methods!();
    pub fn axes(mut self, value: ScrollAxes) -> Self {
        self.axes = value;
        self
    }
    pub fn scrollbars(mut self, value: Scrollbars) -> Self {
        self.scrollbars = value;
        self
    }
    pub fn keyboard(mut self, value: bool) -> Self {
        self.keyboard = value;
        self
    }
    pub fn step(mut self, value: f64) -> Self {
        self.step = value;
        self
    }
    pub fn content<W: Widget>(
        mut self,
        mut factory: impl FnMut(ScrollContext) -> W + 'static,
    ) -> Self {
        self.content = Some(Box::new(move |context| Box::new(factory(context))));
        self
    }
    pub fn content_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.content_ref = Some(value);
        self
    }
    pub fn on_metrics_change(mut self, callback: impl FnMut(ScrollMetrics) + 'static) -> Self {
        self.observed = Some(Box::new(callback));
        self
    }
    pub fn on_scroll_change(mut self, callback: impl FnMut((f64, f64)) + 'static) -> Self {
        self.scroll_change = Some(Box::new(callback));
        self
    }
    pub fn on_ready(mut self, callback: impl FnMut(ScrollHandle) + 'static) -> Self {
        self.ready = Some(Box::new(callback));
        self
    }
    fn controller(&mut self, scope: &Scope) -> Result<ScrollHandle> {
        composite::id("scroll", &self.key, "viewport", "viewport")?;
        if !self.step.is_finite() || self.step <= 0.0 || self.step > 1_000_000.0 {
            return Err(invalid("scroll step must be finite and in 0..=1000000"));
        }
        let metrics = scope.cage(None);
        let inner = Rc::new(Controller {
            weak: scope.downgrade(),
            owner: scope.id(),
            live: Cell::new(true),
            viewport: self.common.node_ref.unwrap_or_else(|| scope.cage(None)),
            metrics,
            axes: self.axes,
            step: self.step,
            queue: RefCell::new(VecDeque::new()),
            requesting: Cell::new(false),
            observed: Callback::new(self.observed.take()),
        });
        let cleanup = inner.clone();
        scope.on_cleanup(move || cleanup.retire())?;
        Ok(ScrollHandle {
            inner,
            metrics: scope.bond(move || metrics.get()),
        })
    }
    fn mount_view(&mut self, scope: &Scope, handle: ScrollHandle) -> Result {
        let owner = scope.id();
        let reference = handle.inner.viewport;
        let overflow = match self.scrollbars {
            Scrollbars::Auto => Overflow::Auto,
            Scrollbars::Always => Overflow::Scroll,
        };
        let mut structural = Style::new()
            .width(Length::Percent(100.0))
            .height(Length::Px(240.0))
            .overflow_x(if self.axes.allows(Orientation::Horizontal) {
                overflow
            } else {
                Overflow::Hidden
            })
            .overflow_y(if self.axes.allows(Orientation::Vertical) {
                overflow
            } else {
                Overflow::Hidden
            });
        if !self.common.unstyled {
            structural.color = Some(self.common.theme.foreground);
            structural.background = Some(self.common.theme.surface);
        }
        let custom = self.common.style.clone();
        let mut viewport = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-scroll-area",
            Style::new()
        )
        .style(scope.bond(move || merge(structural.clone(), custom.get())))
        .focus(if self.keyboard {
            FocusMode::TabStop
        } else {
            FocusMode::Programmatic
        })
        .node_ref(reference)
        .semantics(composite::metadata(Semantics {
            id: Some(composite::id("scroll", &self.key, "viewport", "viewport")?),
            scroll_keyboard: self.keyboard.then_some(self.axes),
            ..Default::default()
        }));
        if self.keyboard {
            let handle = handle.clone();
            let weak = scope.downgrade();
            let mut generic = self
                .common
                .handlers
                .remove(&EventKind::ScrollKeyboard)
                .unwrap_or_default();
            viewport = viewport.on(EventKind::ScrollKeyboard, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                if let Event::ScrollKeyboard { command, .. } = event
                    && let Err(error) = handle.command(command)
                {
                    scope.report_error(error);
                    return;
                }
                observe(&mut generic, owner, event);
            });
        }
        let callback = Callback::new(self.scroll_change.take());
        let cleanup = callback.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        for kind in [EventKind::Scroll, EventKind::Resize] {
            let handle = handle.clone();
            let weak = scope.downgrade();
            let callback = callback.clone();
            let mut generic = self.common.handlers.remove(&kind).unwrap_or_default();
            viewport = viewport.on(kind, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                if let Event::Scroll { x, y, .. } = event {
                    callback.emit(owner, (x, y));
                }
                if let Err(error) = handle.refresh() {
                    scope.report_error(error);
                    return;
                }
                observe(&mut generic, owner, event);
            });
        }
        for (kind, mut callbacks) in std::mem::take(&mut self.common.handlers) {
            viewport = viewport.on(kind, move |event| observe(&mut callbacks, owner, event));
        }
        viewport.show_in(scope)?;
        let parent = scope.child_in(reference.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        let handle_resize = handle.clone();
        let weak = scope.downgrade();
        let mut content =
            glory::ui::view()
                .class("sapid-scroll-content")
                .on(EventKind::Resize, move |_| {
                    if let Err(error) = handle_resize.refresh()
                        && let Some(scope) = weak.upgrade()
                    {
                        scope.report_error(error);
                    }
                });
        if let Some(reference) = self.content_ref {
            content = content.node_ref(reference);
        }
        if let Some(mut factory) = self.content.take() {
            let context = ScrollContext {
                scroll: handle.clone(),
                theme: self.common.theme.clone(),
                size: self.common.size,
                unstyled: self.common.unstyled,
            };
            content = content.fill(glory::ui::component(move |scope| {
                factory(context.clone()).show_in(scope)
            }));
        }
        content.show_in(&parent)?;
        scope.retain(parent)?;
        if let Some(mut callback) = self.ready.take() {
            callback(handle);
        }
        Ok(())
    }
}
impl Widget for ScrollArea {
    fn build(&mut self, scope: &Scope) -> Result {
        let handle = self.controller(scope)?;
        self.mount_view(scope, handle)
    }
}

pub struct Scroller {
    area: ScrollArea,
    axis: Orientation,
}
pub fn scroller(key: impl Into<String>) -> Scroller {
    Scroller {
        area: scroll_area(key).axes(ScrollAxes::Horizontal),
        axis: Orientation::Horizontal,
    }
}
impl Scroller {
    pub fn area(mut self, configure: impl FnOnce(ScrollArea) -> ScrollArea) -> Self {
        self.area = configure(self.area);
        self.area.axes = match self.axis {
            Orientation::Horizontal => ScrollAxes::Horizontal,
            Orientation::Vertical => ScrollAxes::Vertical,
        };
        self
    }
    pub fn axis(mut self, value: Orientation) -> Self {
        self.axis = value;
        self.area.axes = match value {
            Orientation::Horizontal => ScrollAxes::Horizontal,
            Orientation::Vertical => ScrollAxes::Vertical,
        };
        self
    }
    pub fn content<W: Widget>(
        mut self,
        factory: impl FnMut(ScrollContext) -> W + 'static,
    ) -> Self {
        self.area = self.area.content(factory);
        self
    }
}
impl Widget for Scroller {
    fn build(&mut self, scope: &Scope) -> Result {
        let handle = self.area.controller(scope)?;
        let metrics = handle.metrics();
        let axis = self.axis;
        for (label, movement, forward) in [
            ("Previous scroll page", ScrollMovement::PageNegative, false),
            ("Next scroll page", ScrollMovement::PagePositive, true),
        ] {
            let request = handle.clone();
            let weak = scope.downgrade();
            let enabled = scope.bond(move || {
                metrics.get().is_some_and(|metrics| {
                    let (offset, viewport, content) = match axis {
                        Orientation::Horizontal => {
                            (metrics.x, metrics.viewport_width, metrics.content_width)
                        }
                        Orientation::Vertical => {
                            (metrics.y, metrics.viewport_height, metrics.content_height)
                        }
                    };
                    offset >= 0.0
                        && if forward {
                            offset + viewport < content - 0.5
                        } else {
                            offset > 0.5
                        }
                })
            });
            crate::button()
                .theme(self.area.common.theme.clone())
                .size(self.area.common.size)
                .label(label)
                .enabled(enabled)
                .on_press(move || {
                    if let Err(error) = request.command(ScrollCommand { axis, movement })
                        && let Some(scope) = weak.upgrade()
                    {
                        scope.report_error(error);
                    }
                })
                .show_in(scope)?;
        }
        self.area.mount_view(scope, handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordered_clamping_is_not_an_unbounded_delta_sum() {
        let metrics = ScrollMetrics {
            y: 0.0,
            viewport_height: 100.0,
            content_height: 500.0,
            ..Default::default()
        };
        let mut queue = VecDeque::from([Intent::By(0.0, -100.0), Intent::By(0.0, 50.0)]);
        assert_eq!(position(metrics, &mut queue, 40.0).unwrap(), (0.0, 50.0));
        let mut queue = VecDeque::from([
            Intent::To(0.0, 900.0),
            Intent::By(0.0, -40.0),
            Intent::Command(ScrollCommand {
                axis: Orientation::Vertical,
                movement: ScrollMovement::PageNegative,
            }),
        ]);
        assert_eq!(position(metrics, &mut queue, 40.0).unwrap(), (0.0, 260.0));
        assert!(queue.is_empty());
    }
}
