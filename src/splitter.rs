//! Two stable panes and a finite, focusable separator.
use crate::appearance::{Common, configure, merge};
use crate::composite::{self, Callback, Observers};
use crate::state::{Control, Value};
use glory::render::{
    Action, DragPhase, Event, EventKind, FocusMode, NodeId, Orientation, Query, QueryToken,
    QueryValue, RangeCommand, RangeValue, Role, SemanticId, Semantics,
};
use glory::style::{Display, Edges, FlexDirection, Length, Overflow, Style};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Scope, Vary};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Snapshot {
    value: f64,
    min: f64,
    max: f64,
    enabled: bool,
}
impl Snapshot {
    fn validate(&self) -> Result {
        RangeValue {
            min: self.min,
            max: self.max,
            value: self.value,
        }
        .validate()?;
        if self.min < 0.0 || self.max > 100.0 {
            return Err(invalid("splitter percentages must stay within 0..=100"));
        }
        Ok(())
    }
    fn range(self) -> RangeValue {
        RangeValue {
            min: self.min,
            max: self.max,
            value: self.value,
        }
    }
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
#[derive(Clone, Copy)]
struct Drag {
    ticket: u64,
    start: f64,
    value: f64,
    latest: f64,
    point: (f64, f64),
    extent: Option<f64>,
    phase: DragPhase,
    pending: bool,
}
struct Controller {
    weak: WeakScope,
    owner: glory::core::InstanceId,
    live: Cell<bool>,
    control: Control<f64>,
    local: bool,
    accepted: Cage<Snapshot>,
    axis: Orientation,
    step: f64,
    thickness: f64,
    root: Cage<Option<NodeId>>,
    handle: Cage<Option<NodeId>>,
    change: Callback<f64>,
    commit: Callback<f64>,
    range_events: Observers,
    drag_events: Observers,
    drag: RefCell<Option<Drag>>,
    pending: Cell<Option<QueryToken>>,
    next: Cell<u64>,
}
impl Controller {
    fn ensure(&self) -> Result {
        if self.live.get() && self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn propose(&self, value: f64) -> Result {
        self.ensure()?;
        let state = self.accepted.try_get()?;
        if !state.enabled {
            return Ok(());
        }
        let value = value.clamp(state.min, state.max);
        self.control.accept(&value)?;
        self.change.emit(self.owner, value);
        Ok(())
    }
    fn key(&self, command: RangeCommand) -> Result {
        let state = self.accepted.try_get()?;
        if !state.enabled {
            return Ok(());
        }
        command.validate(state.range())?;
        let current = if self.local {
            self.control.source.try_get()?.clamp(state.min, state.max)
        } else {
            state.value
        };
        let next = match command {
            RangeCommand::PageDecrease => (current - self.step * 10.0).max(state.min),
            RangeCommand::PageIncrease => (current + self.step * 10.0).min(state.max),
            RangeCommand::Decrease => (current - self.step).max(state.min),
            RangeCommand::Increase => (current + self.step).min(state.max),
            RangeCommand::Minimum => state.min,
            RangeCommand::Maximum => state.max,
            RangeCommand::Set(value) => value,
        };
        if next != current {
            self.propose(next)?;
            self.commit.emit(self.owner, next);
        }
        Ok(())
    }
    fn cancel(&self) {
        self.drag.borrow_mut().take();
        if let Some(token) = self.pending.take()
            && let Some(scope) = self.weak.upgrade()
            && let Err(error) = scope.cancel_request(token)
            && self.live.get()
            && self.owner.is_alive()
        {
            scope.report_error(error);
        }
    }
    fn retire(&self) {
        self.live.set(false);
        self.cancel();
        self.change.clear();
        self.commit.clear();
        self.range_events.clear();
        self.drag_events.clear();
    }
    fn coordinate(&self, x: f64, y: f64) -> f64 {
        if self.axis == Orientation::Horizontal {
            x
        } else {
            y
        }
    }
    fn apply_drag(&self) -> Result {
        let state = self.accepted.try_get()?;
        let drag = {
            let mut slot = self.drag.borrow_mut();
            let Some(drag) = slot.as_mut() else {
                return Ok(());
            };
            if !drag.pending || drag.extent.is_none() {
                return Ok(());
            }
            drag.pending = false;
            let result = *drag;
            if drag.phase == DragPhase::Drop {
                slot.take();
            }
            result
        };
        if !state.enabled {
            self.cancel();
            return Ok(());
        }
        let extent = drag.extent.unwrap();
        let value = (drag.value + (drag.latest - drag.start) / extent * 100.0)
            .clamp(state.min, state.max);
        self.propose(value)?;
        if drag.phase == DragPhase::Drop {
            self.commit.emit(self.owner, value);
        }
        let (x, y) = drag.point;
        // Generic observers get the same absolute axis coordinate after the
        // geometry-dependent typed proposal; pending moves coalesce to one.
        let from = self.handle.try_get()?.ok_or(glory::ui::Error::Gone)?;
        self.drag_events.emit(
            self.owner,
            Event::Drag {
                from,
                phase: drag.phase,
                x,
                y,
            },
        );
        Ok(())
    }
    fn pointer(self: &Rc<Self>, event: Event) -> Result {
        let Event::Drag { phase, x, y, .. } = event else {
            return Ok(());
        };
        self.ensure()?;
        if phase == DragPhase::Cancel {
            self.cancel();
            self.drag_events.emit(self.owner, event);
            return Ok(());
        }
        let state = self.accepted.try_get()?;
        if !state.enabled {
            self.cancel();
            return Ok(());
        }
        if phase == DragPhase::Start {
            self.cancel();
            let ticket = self
                .next
                .get()
                .checked_add(1)
                .ok_or_else(|| invalid("splitter drag ticket exhausted"))?;
            self.next.set(ticket);
            let coordinate = self.coordinate(x, y);
            *self.drag.borrow_mut() = Some(Drag {
                ticket,
                start: coordinate,
                value: state.value,
                latest: coordinate,
                point: (x, y),
                extent: None,
                phase: DragPhase::Start,
                pending: false,
            });
            let scope = self.weak.upgrade().ok_or(glory::ui::Error::Gone)?;
            let root = self.root.try_get()?.ok_or(glory::ui::Error::Gone)?;
            let controller = self.clone();
            let token = scope.request(Query::Bounds(root), move |result| {
                if controller.ensure().is_err() {
                    return;
                }
                if controller.drag.borrow().as_ref().map(|drag| drag.ticket) != Some(ticket) {
                    return;
                }
                controller.pending.set(None);
                let Some(scope) = controller.weak.upgrade() else {
                    return;
                };
                let result = (|| -> Result {
                    let QueryValue::Bounds(rect) = result? else {
                        return Err(invalid("splitter bounds response mismatch"));
                    };
                    let extent = (if controller.axis == Orientation::Horizontal {
                        rect.width
                    } else {
                        rect.height
                    }) - controller.thickness;
                    if !extent.is_finite() || extent <= 0.0 {
                        return Err(invalid("splitter needs positive available pane extent"));
                    }
                    {
                        let mut slot = controller.drag.borrow_mut();
                        let Some(drag) = slot.as_mut().filter(|drag| drag.ticket == ticket)
                        else {
                            return Ok(());
                        };
                        drag.extent = Some(extent);
                    }
                    controller.apply_drag()
                })();
                if let Err(error) = result {
                    controller.cancel();
                    scope.report_error(error);
                }
            })?;
            self.pending.set(Some(token));
            self.drag_events.emit(self.owner, event);
        } else {
            {
                let mut slot = self.drag.borrow_mut();
                let Some(drag) = slot.as_mut() else {
                    return Ok(());
                };
                drag.latest = self.coordinate(x, y);
                drag.point = (x, y);
                drag.phase = phase;
                drag.pending = true;
            }
            self.apply_drag()?;
        }
        Ok(())
    }
}

type Factory = Box<dyn FnMut() -> Box<dyn Widget>>;
pub struct Splitter {
    common: Common,
    key: String,
    value: Value<f64>,
    min: Vary<f64>,
    max: Vary<f64>,
    step: f64,
    axis: Orientation,
    thickness: f64,
    first: Option<Factory>,
    second: Option<Factory>,
    first_ref: Option<Cage<Option<NodeId>>>,
    second_ref: Option<Cage<Option<NodeId>>>,
    handle_ref: Option<Cage<Option<NodeId>>>,
    change: Option<Box<dyn FnMut(f64)>>,
    commit: Option<Box<dyn FnMut(f64)>>,
}
pub fn splitter(key: impl Into<String>) -> Splitter {
    Splitter {
        common: Common::default(),
        key: key.into(),
        value: Value {
            source: None,
            default: 50.0,
        },
        min: 10.0.into(),
        max: 90.0.into(),
        step: 1.0,
        axis: Orientation::Horizontal,
        thickness: 6.0,
        first: None,
        second: None,
        first_ref: None,
        second_ref: None,
        handle_ref: None,
        change: None,
        commit: None,
    }
}
impl Splitter {
    crate::appearance::common_methods!();
    pub fn value(mut self, value: impl Into<Vary<f64>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: f64) -> Self {
        self.value.default = value;
        self
    }
    pub fn min(mut self, value: impl Into<Vary<f64>>) -> Self {
        self.min = value.into();
        self
    }
    pub fn max(mut self, value: impl Into<Vary<f64>>) -> Self {
        self.max = value.into();
        self
    }
    pub fn step(mut self, value: f64) -> Self {
        self.step = value;
        self
    }
    pub fn axis(mut self, value: Orientation) -> Self {
        self.axis = value;
        self
    }
    pub fn thickness(mut self, value: f64) -> Self {
        self.thickness = value;
        self
    }
    pub fn first<W: Widget>(mut self, mut factory: impl FnMut() -> W + 'static) -> Self {
        self.first = Some(Box::new(move || Box::new(factory())));
        self
    }
    pub fn second<W: Widget>(mut self, mut factory: impl FnMut() -> W + 'static) -> Self {
        self.second = Some(Box::new(move || Box::new(factory())));
        self
    }
    pub fn first_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.first_ref = Some(value);
        self
    }
    pub fn second_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.second_ref = Some(value);
        self
    }
    pub fn handle_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.handle_ref = Some(value);
        self
    }
    pub fn on_value_change(mut self, callback: impl FnMut(f64) + 'static) -> Self {
        self.change = Some(Box::new(callback));
        self
    }
    pub fn on_value_commit(mut self, callback: impl FnMut(f64) + 'static) -> Self {
        self.commit = Some(Box::new(callback));
        self
    }
}
impl Widget for Splitter {
    fn build(&mut self, scope: &Scope) -> Result {
        let first_id = composite::id("splitter", &self.key, "pane", "first")?;
        let second_id = composite::id("splitter", &self.key, "pane", "second")?;
        let handle_id = composite::id("splitter", &self.key, "handle", "handle")?;
        if !self.step.is_finite()
            || self.step <= 0.0
            || self.step > 100.0
            || !self.thickness.is_finite()
            || self.thickness <= 0.0
            || self.thickness > 128.0
        {
            return Err(invalid(
                "splitter needs step in (0,100] and thickness in (0,128]",
            ));
        }
        if self.common.focus != FocusMode::Auto {
            return Err(invalid("splitter owns its handle tab stop"));
        }
        let local = self.value.source.is_none();
        let axis = self.axis;
        let handle_name = self
            .common
            .name
            .clone()
            .unwrap_or_else(|| "Resize panes".into());
        let control = std::mem::take(&mut self.value).mount(scope);
        let source = control.source.clone();
        let min = self.min.clone();
        let max = self.max.clone();
        let enabled = self.common.enabled.clone();
        let raw: Vary<Snapshot> = scope
            .bond(move || Snapshot {
                value: source.get(),
                min: min.get(),
                max: max.get(),
                enabled: enabled.get(),
            })
            .into();
        let accepted = composite::validated(scope, &raw, Snapshot::validate)?;
        let root = scope.cage(None);
        let handle = self.handle_ref.unwrap_or_else(|| scope.cage(None));
        let controller = Rc::new(Controller {
            weak: scope.downgrade(),
            owner: scope.id(),
            live: Cell::new(true),
            control,
            local,
            accepted,
            axis: self.axis,
            step: self.step,
            thickness: self.thickness,
            root,
            handle,
            change: Callback::new(self.change.take()),
            commit: Callback::new(self.commit.take()),
            range_events: Observers::new(
                self.common
                    .handlers
                    .remove(&EventKind::Range)
                    .unwrap_or_default(),
            ),
            drag_events: Observers::new(
                self.common
                    .handlers
                    .remove(&EventKind::Drag)
                    .unwrap_or_default(),
            ),
            drag: RefCell::new(None),
            pending: Cell::new(None),
            next: Cell::new(0),
        });
        let cleanup = controller.clone();
        scope.on_cleanup(move || cleanup.retire())?;
        let outer_ref = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let custom = self.common.style.clone();
        let mut outer = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-splitter",
            Style::new()
                .width(Length::Percent(100.0))
                .height(Length::Px(300.0))
        )
        .style(scope.bond(move || {
            merge(
                Style::new()
                    .width(Length::Percent(100.0))
                    .height(Length::Px(300.0)),
                custom.get(),
            )
        }))
        .node_ref(outer_ref);
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            outer = outer.on(kind, move |event| {
                crate::appearance::observe(&mut handlers, owner, event)
            });
        }
        outer.show_in(scope)?;
        let parent = scope.child_in(outer_ref.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        glory::ui::view()
            .class("sapid-splitter-frame")
            .node_ref(root)
            .style(
                Style::new()
                    .width(Length::Percent(100.0))
                    .height(Length::Percent(100.0))
                    .display(Display::Flex)
                    .flex_direction(if self.axis == Orientation::Horizontal {
                        FlexDirection::Row
                    } else {
                        FlexDirection::Column
                    })
                    .gap(0.0)
                    .padding(Edges::all(0.0))
                    .border_width(Edges::all(0.0))
                    .margin(Edges::all(0.0)),
            )
            .show_in(&parent)?;
        let frame = scope.child_in(root.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        let first_focused = scope.cage(false);
        let second_focused = scope.cage(false);
        for (first, id, reference, factory, focused) in [
            (
                true,
                first_id.clone(),
                self.first_ref,
                self.first.take(),
                first_focused,
            ),
            (
                false,
                second_id,
                self.second_ref,
                self.second.take(),
                second_focused,
            ),
        ] {
            if !first {
                build_handle(
                    &frame,
                    &self.common,
                    controller.clone(),
                    first_id.clone(),
                    handle_id.clone(),
                    handle_name.clone(),
                )?;
            }
            let mut pane = glory::ui::view()
                .name(if first { "First pane" } else { "Second pane" })
                .class(if first {
                    "sapid-splitter-first"
                } else {
                    "sapid-splitter-second"
                })
                .semantics(scope.bond(move || {
                    composite::metadata(Semantics {
                        id: Some(id.clone()),
                        hidden: if first {
                            accepted.get().value == 0.0
                        } else {
                            accepted.get().value == 100.0
                        },
                        ..Default::default()
                    })
                }))
                .style(scope.bond(move || {
                    Style {
                        grow: Some(if first {
                            accepted.get().value
                        } else {
                            100.0 - accepted.get().value
                        }),
                        shrink: Some(1.0),
                        ..Style::new()
                            .width(if axis == Orientation::Horizontal {
                                Length::Px(0.0)
                            } else {
                                Length::Percent(100.0)
                            })
                            .height(if axis == Orientation::Vertical {
                                Length::Px(0.0)
                            } else {
                                Length::Percent(100.0)
                            })
                            .min_width(Length::Px(0.0))
                            .min_height(Length::Px(0.0))
                            .margin(Edges::all(0.0))
                            .overflow_x(Overflow::Hidden)
                            .overflow_y(Overflow::Hidden)
                    }
                }))
                .on(EventKind::FocusWithin, move |event| {
                    if let Event::FocusWithin { focused: value, .. } = event {
                        focused.revise(|old| *old = value);
                    }
                });
            if let Some(reference) = reference {
                pane = pane.node_ref(reference);
            }
            if let Some(mut factory) = factory {
                pane = pane.fill(glory::ui::widget(move |scope| factory().show_in(scope)));
            }
            pane.show_in(&frame)?;
        }
        let weak = scope.downgrade();
        let adapt = controller.clone();
        scope.effect(move || {
            let state = accepted.get();
            if !state.enabled {
                adapt.cancel();
            }
            if state.enabled
                && (state.value == 0.0 && first_focused.get()
                    || state.value == 100.0 && second_focused.get())
                && let Some(scope) = weak.upgrade()
                && let Some(handle) = handle.get_untracked()
            {
                first_focused.revise(|v| *v = false);
                second_focused.revise(|v| *v = false);
                if let Err(error) = scope.action(Action::Focus(handle)) {
                    scope.report_error(error);
                }
            }
        });
        scope.retain(parent)?;
        scope.retain(frame)?;
        Ok(())
    }
}
fn build_handle(
    scope: &Scope,
    common: &Common,
    controller: Rc<Controller>,
    first: SemanticId,
    id: SemanticId,
    name: Vary<String>,
) -> Result {
    let accepted = controller.accepted;
    let axis = controller.axis;
    let thickness = controller.thickness;
    let range = controller.clone();
    let drag = controller.clone();
    let weak = scope.downgrade();
    let drag_weak = weak.clone();
    let color = common.theme.border;
    let unstyled = common.unstyled;
    glory::ui::view()
        .role(Role::Separator)
        .name(name)
        .focus(scope.bond(move || {
            if accepted.get().enabled {
                FocusMode::TabStop
            } else {
                FocusMode::Programmatic
            }
        }))
        .class("sapid-splitter-handle")
        .node_ref(controller.handle)
        .semantics(scope.bond(move || {
            let state = accepted.get();
            composite::metadata(Semantics {
                id: Some(id.clone()),
                splitter: Some(state.range()),
                separator_axis: Some(if axis == Orientation::Horizontal {
                    Orientation::Vertical
                } else {
                    Orientation::Horizontal
                }),
                controls: Some(first.clone()),
                disabled: !state.enabled,
                ..Default::default()
            })
        }))
        .style(Style {
            shrink: Some(0.0),
            background: (!unstyled).then_some(color),
            ..Style::new()
                .width(if axis == Orientation::Horizontal {
                    Length::Px(thickness)
                } else {
                    Length::Percent(100.0)
                })
                .height(if axis == Orientation::Horizontal {
                    Length::Percent(100.0)
                } else {
                    Length::Px(thickness)
                })
                .margin(Edges::all(0.0))
        })
        .on(EventKind::Range, move |event| {
            if let Event::Range { command, .. } = event
                && let Err(error) = range.key(command)
            {
                if let Some(scope) = weak.upgrade() {
                    scope.report_error(error);
                }
                return;
            }
            range.range_events.emit(range.owner, event);
        })
        .on(EventKind::Drag, move |event| {
            if let Err(error) = drag.pointer(event)
                && let Some(scope) = drag_weak.upgrade()
            {
                drag.cancel();
                scope.report_error(error);
            }
        })
        .show_in(scope)
}
