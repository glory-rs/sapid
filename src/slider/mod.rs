//! One finite numeric range, with application-owned controlled values.
use crate::appearance::Common;
mod mapping;
use crate::composite::{self, Callback, Observers};
use crate::field::{FieldControl, control_metadata};
use crate::state::{Control, Value};
use glory::render::{
    Direction, DragPhase, Event, EventKind, FocusMode, NodeId, Orientation, Query, QueryToken,
    QueryValue, RangeCommand, RangeValue, Rect, Role, SliderValue,
};
use glory::style::{Edges, Length, Position, Style};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Lotus, Scope};
use mapping::Mapping;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
#[derive(Clone, Debug, PartialEq)]
struct Snapshot {
    range: SliderValue,
    enabled: bool,
    readonly: bool,
    mapping: Mapping,
}
impl Snapshot {
    fn validate(&self) -> Result {
        self.range.range.validate()?;
        self.range.validate()?;
        self.mapping
            .validate(self.range.range, self.range.step, self.range.page_step)?;
        if [self.range.range.min, self.range.range.max]
            .into_iter()
            .any(|v| v.abs() > 1e12)
            || self.range.step > 1e12
            || self.range.page_step > 1e12
        {
            return Err(invalid(
                "slider coordinates and steps are bounded to +/-1000000000000",
            ));
        }
        Ok(())
    }
    fn domain_matches(&self, other: &Self) -> bool {
        self.range.range.min == other.range.range.min
            && self.range.range.max == other.range.range.max
            && self.range.axis == other.range.axis
            && self.range.direction == other.range.direction
            && self.range.step == other.range.step
            && self.range.page_step == other.range.page_step
            && self.enabled == other.enabled
            && self.readonly == other.readonly
            && self.mapping == other.mapping
    }
    fn snap(&self, value: f64) -> f64 {
        let range = self.range.range;
        let value = value.clamp(range.min, range.max);
        if value == range.min || value == range.max {
            return value;
        }
        let remainder = (value - range.min) % self.range.step;
        let lower = value - remainder;
        let upper = (lower + self.range.step).min(range.max);
        if value - lower < upper - value {
            lower.max(range.min)
        } else {
            upper
        }
    }
    fn point(&self, rect: Rect, x: f64, y: f64) -> Result<Option<f64>> {
        Ok(self
            .mapping
            .point(
                rect,
                x,
                y,
                self.range.axis,
                self.range.direction,
                self.range.range,
            )?
            .map(|value| self.snap(value)))
    }
}
#[derive(Clone, Copy)]
struct Drag {
    ticket: u64,
    start: (f64, f64),
    latest: (f64, f64),
    phase: DragPhase,
    start_pending: bool,
    pending: bool,
}
struct Controller {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    live: Cell<bool>,
    control: Control<f64>,
    local: bool,
    accepted: Cage<Snapshot>,
    reference: Cage<Option<NodeId>>,
    changed: Callback<f64>,
    committed: Callback<f64>,
    range_events: Observers,
    drag_events: Observers,
    expected: Cell<Option<f64>>,
    confirmed: Cell<Option<(f64, Option<u64>)>>,
    drag: RefCell<Option<Drag>>,
    active: Cell<Option<u64>>,
    pending: Cell<Option<QueryToken>>,
    next: Cell<u64>,
    last_point: Cell<Option<f64>>,
}
impl Controller {
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
    fn cancel(&self) {
        self.active.set(None);
        self.last_point.set(None);
        self.drag.borrow_mut().take();
        let token = self.pending.take();
        if let Some(token) = token
            && let Some(scope) = self.weak.upgrade()
            && let Err(error) = scope.cancel_request(token)
            && self.owner.is_alive()
        {
            scope.report_error(error)
        }
    }
    fn retire(&self) {
        self.live.set(false);
        self.cancel();
        self.changed.clear();
        self.committed.clear();
        self.range_events.clear();
        self.drag_events.clear();
    }
    fn propose(&self, value: f64) -> Result {
        self.ensure()?;
        let state = self.accepted.try_get()?;
        if !state.enabled || state.readonly {
            return Ok(());
        }
        RangeCommand::Set(value).validate(state.range.range)?;
        let before = self.source_stamp()?;
        self.expected.set(Some(value));
        self.confirmed.set(None);
        self.control.accept(&value)?;
        self.changed.emit(self.owner, value);
        self.confirm_acceptance(value, before)
    }
    fn confirm_acceptance(&self, value: f64, before: (f64, Option<u64>)) -> Result {
        if self.ensure().is_ok() && self.expected.get() == Some(value) {
            let after = self.source_stamp()?;
            // A past rejected request must never become a cursor merely
            // because a later external write happens to contain that number.
            if after.0 == value && before != after {
                self.confirmed.set(Some(after));
            }
        }
        Ok(())
    }
    fn commit(&self, value: f64) -> Result {
        let before = self.source_stamp()?;
        self.committed.emit(self.owner, value);
        self.confirm_acceptance(value, before)
    }
    fn source_stamp(&self) -> Result<(f64, Option<u64>)> {
        let value = self.control.source.try_get()?;
        let version = match self.control.source {
            Lotus::Cage(source) => Some(source.try_version()?),
            _ => None,
        };
        Ok((value, version))
    }
    fn key(&self, command: RangeCommand) -> Result {
        self.ensure()?;
        self.cancel();
        let state = self.accepted.try_get()?;
        if !state.enabled || state.readonly {
            return Ok(());
        }
        command.validate(state.range.range)?;
        let current = if self.local {
            self.control.source.try_get()?
        } else if let Some(confirmed) = self.confirmed.get()
            && self.source_stamp()? == confirmed
        {
            confirmed.0
        } else {
            state.range.range.value
        };
        let range = state.range.range;
        let value = state.mapping.key(
            current,
            command,
            range,
            state.range.step,
            state.range.page_step,
        );
        let value = if state.mapping.retains_increment(command) {
            value
        } else {
            state.snap(value)
        };
        if value != current {
            self.propose(value)?;
            self.commit(value)?
        }
        Ok(())
    }
    fn publish_drag(
        &self,
        ticket: u64,
        point: (f64, f64),
        phase: DragPhase,
        rect: Rect,
    ) -> Result {
        if self.active.get() != Some(ticket) || self.ensure().is_err() {
            return Ok(());
        }
        let state = self.accepted.try_get()?;
        if !state.enabled || state.readonly {
            self.cancel();
            return Ok(());
        }
        let mapped = state.point(rect, point.0, point.1)?;
        if let Some(value) = mapped {
            self.last_point.set(Some(value));
        }
        let current = if self.local {
            self.control.source.try_get()?
        } else {
            state.range.range.value
        };
        let value = mapped.or(self.last_point.get()).unwrap_or(current);
        if value != current {
            self.propose(value)?
        }
        if self.active.get() != Some(ticket) || self.ensure().is_err() {
            return Ok(());
        }
        if phase == DragPhase::Drop {
            self.commit(value)?
        }
        if self.active.get() == Some(ticket)
            && let Some(from) = self.reference.try_get()?
        {
            self.drag_events.emit(
                self.owner,
                Event::Drag {
                    from,
                    phase,
                    x: point.0,
                    y: point.1,
                },
            )
        }
        Ok(())
    }
    fn apply_geometry(&self, ticket: u64, rect: Rect) -> Result {
        let work = {
            let mut slot = self.drag.borrow_mut();
            let Some(drag) = slot.as_mut().filter(|drag| drag.ticket == ticket) else {
                return Ok(());
            };
            if !drag.pending {
                return Ok(());
            }
            let work = *drag;
            drag.pending = false;
            drag.start_pending = false;
            if work.phase == DragPhase::Drop {
                slot.take();
            }
            work
        };
        if work.start_pending {
            self.publish_drag(ticket, work.start, DragPhase::Start, rect)?
        }
        if work.phase != DragPhase::Start {
            self.publish_drag(ticket, work.latest, work.phase, rect)?
        }
        if work.phase == DragPhase::Drop && self.active.get() == Some(ticket) {
            self.active.set(None)
        }
        Ok(())
    }
    fn request_geometry(self: &Rc<Self>) -> Result {
        if self.pending.get().is_some() {
            return Ok(());
        }
        let Some(ticket) = self.active.get() else {
            return Ok(());
        };
        let scope = self.weak.upgrade().ok_or(glory::ui::Error::Gone)?;
        let node = self.reference.try_get()?.ok_or(glory::ui::Error::Gone)?;
        let state = self.clone();
        let token = scope.request(Query::Bounds(node), move |result| {
            if state.ensure().is_err() || state.active.get() != Some(ticket) {
                return;
            }
            state.pending.set(None);
            let result = (|| -> Result {
                let QueryValue::Bounds(rect) = result? else {
                    return Err(invalid("slider bounds response mismatch"));
                };
                state.apply_geometry(ticket, rect)
            })();
            if let Err(error) = result {
                state.cancel();
                state.report(error)
            }
        })?;
        if self.active.get() == Some(ticket) {
            self.pending.set(Some(token))
        } else {
            scope.cancel_request(token)?;
        }
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
        let accepted = self.accepted.try_get()?;
        if !accepted.enabled || accepted.readonly {
            self.cancel();
            return Ok(());
        }
        if phase == DragPhase::Start {
            self.cancel();
            let ticket = self
                .next
                .get()
                .checked_add(1)
                .ok_or_else(|| invalid("slider drag ticket exhausted"))?;
            self.next.set(ticket);
            self.active.set(Some(ticket));
            *self.drag.borrow_mut() = Some(Drag {
                ticket,
                start: (x, y),
                latest: (x, y),
                phase,
                start_pending: true,
                pending: true,
            });
        } else {
            let mut slot = self.drag.borrow_mut();
            let Some(drag) = slot.as_mut() else {
                return Ok(());
            };
            drag.latest = (x, y);
            drag.phase = phase;
            drag.pending = true;
        }
        self.request_geometry()
    }
}

pub struct Slider {
    key: String,
    common: Common,
    value: Value<f64>,
    min: Lotus<f64>,
    max: Lotus<f64>,
    step: Lotus<f64>,
    page: Option<Lotus<f64>>,
    text: Lotus<Option<String>>,
    axis: Orientation,
    direction: Direction,
    field: Option<FieldControl>,
    required: Lotus<bool>,
    readonly: Lotus<bool>,
    invalid: Lotus<bool>,
    inherit_theme: bool,
    inherit_size: bool,
    changed: Option<Box<dyn FnMut(f64)>>,
    committed: Option<Box<dyn FnMut(f64)>>,
    track_content: Option<Box<dyn Widget>>,
    thumb_color: Option<Lotus<glory::style::Color>>,
    mapping: Mapping,
    default_text: bool,
}
pub fn slider(key: impl Into<String>) -> Slider {
    Slider {
        key: key.into(),
        common: Common::default(),
        value: Value::default(),
        min: 0.0.into(),
        max: 100.0.into(),
        step: 1.0.into(),
        page: None,
        text: None.into(),
        axis: Orientation::Horizontal,
        direction: Direction::Ltr,
        field: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
        inherit_theme: true,
        inherit_size: true,
        changed: None,
        committed: None,
        track_content: None,
        thumb_color: None,
        mapping: Mapping::Linear,
        default_text: false,
    }
}
impl Slider {
    pub(crate) fn angular(mut self) -> Self {
        self.mapping = Mapping::Angle;
        self.default_text = true;
        self.min = 0.0.into();
        self.max = 360.0.into();
        self.page = Some(15.0.into());
        self
    }
    pub(crate) fn track_content(mut self, value: impl Widget + 'static) -> Self {
        self.track_content = Some(Box::new(value));
        self
    }
    pub(crate) fn thumb_color(mut self, value: impl Into<Lotus<glory::style::Color>>) -> Self {
        self.thumb_color = Some(value.into());
        self
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self.inherit_theme = false;
        self
    }
    pub fn size(mut self, value: crate::Size) -> Self {
        self.common.size = value;
        self.inherit_size = false;
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
    pub fn enabled(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.common.enabled = value.into();
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
    pub fn value(mut self, value: impl Into<Lotus<f64>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: f64) -> Self {
        self.value.default = value;
        self
    }
    pub fn min(mut self, value: impl Into<Lotus<f64>>) -> Self {
        self.min = value.into();
        self
    }
    pub fn max(mut self, value: impl Into<Lotus<f64>>) -> Self {
        self.max = value.into();
        self
    }
    pub fn step(mut self, value: impl Into<Lotus<f64>>) -> Self {
        self.step = value.into();
        self
    }
    pub fn page_step(mut self, value: impl Into<Lotus<f64>>) -> Self {
        self.page = Some(value.into());
        self
    }
    pub fn value_text(mut self, value: impl Into<Lotus<Option<String>>>) -> Self {
        self.text = value.into();
        self.default_text = false;
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
    pub fn field(mut self, value: FieldControl) -> Self {
        self.field = Some(value);
        self
    }
    pub fn required(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.required = value.into();
        self
    }
    pub fn readonly(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn invalid(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.invalid = value.into();
        self
    }
    pub fn on_value_change(mut self, callback: impl FnMut(f64) + 'static) -> Self {
        self.changed = Some(Box::new(callback));
        self
    }
    pub fn on_value_commit(mut self, callback: impl FnMut(f64) + 'static) -> Self {
        self.committed = Some(Box::new(callback));
        self
    }
}
impl Widget for Slider {
    fn build(&mut self, scope: &Scope) -> Result {
        let mapping = self.mapping;
        let mut id = composite::id(
            if mapping == Mapping::Angle {
                "angle-slider"
            } else {
                "slider"
            },
            &self.key,
            "control",
            "root",
        )?;
        if let Some(field) = &self.field {
            id = field.id.clone();
            if self.inherit_theme {
                self.common.theme = field.theme.clone()
            }
            if self.inherit_size {
                self.common.size = field.size
            }
            self.common.unstyled |= field.unstyled;
        }
        let local = self.value.source.is_none();
        let control = std::mem::take(&mut self.value).mount(scope);
        let enabled = self.field.as_ref().map_or_else(
            || self.common.enabled.clone(),
            |field| field.enabled(scope, self.common.enabled.clone()),
        );
        let metadata = control_metadata(
            scope,
            self.field.as_ref(),
            self.common.name.is_some(),
            self.required.clone(),
            self.readonly.clone(),
            self.invalid.clone(),
        );
        if self.common.name.is_none()
            && metadata.try_with(|metadata| {
                metadata
                    .as_ref()
                    .is_none_or(|metadata| metadata.labelled_by.is_none())
            })?
        {
            self.common.name = Some(
                if mapping == Mapping::Angle {
                    "Angle"
                } else {
                    "Value"
                }
                .into(),
            )
        }
        let source = control.source.clone();
        let min = self.min.clone();
        let max = self.max.clone();
        let step = self.step.clone();
        let page = self.page.clone();
        let text = std::mem::replace(&mut self.text, None.into());
        let field_metadata = metadata.clone();
        let axis = self.axis;
        let direction = self.direction;
        let default_text = self.default_text;
        let read = move || -> Result<Snapshot> {
            let step = step.try_get()?;
            let page = page
                .as_ref()
                .map_or(Ok((step * 10.0).min(1e12)), Lotus::try_get)?;
            let mut snapshot = Snapshot {
                range: SliderValue {
                    range: RangeValue {
                        min: min.try_get()?,
                        max: max.try_get()?,
                        value: source.try_get()?,
                    },
                    axis,
                    direction,
                    step,
                    page_step: page,
                    value_text: None,
                },
                mapping,
                enabled: enabled.try_get()?,
                readonly: field_metadata.try_with(|metadata| {
                    metadata.as_ref().is_some_and(|metadata| metadata.readonly)
                })?,
            };
            snapshot.validate()?;
            snapshot.range.value_text = text.try_with(|text| {
                if text
                    .as_ref()
                    .is_some_and(|text| text.len() > 1024 || text.trim().is_empty())
                {
                    return Err(invalid("slider value text requires 1..=1024 bytes"));
                }
                Ok::<_, glory::ui::Error>(text.clone())
            })??;
            if default_text && snapshot.range.value_text.is_none() {
                snapshot.range.value_text =
                    Some(format!("{} degrees", snapshot.range.range.value));
            }
            Ok(snapshot)
        };
        let accepted = scope.cage(read()?);
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let range_events = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Range)
                .unwrap_or_default(),
        );
        let drag_events = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Drag)
                .unwrap_or_default(),
        );
        let controller = Rc::new(Controller {
            owner: scope.id(),
            weak: scope.downgrade(),
            live: Cell::new(true),
            control,
            local,
            accepted,
            reference,
            changed: Callback::new(self.changed.take()),
            committed: Callback::new(self.committed.take()),
            range_events,
            drag_events,
            expected: Cell::new(None),
            confirmed: Cell::new(None),
            drag: RefCell::new(None),
            active: Cell::new(None),
            pending: Cell::new(None),
            next: Cell::new(0),
            last_point: Cell::new(None),
        });
        let cleanup = Rc::downgrade(&controller);
        scope.on_cleanup(move || {
            if let Some(controller) = cleanup.upgrade() {
                controller.retire()
            }
        })?;
        let state = controller.clone();
        scope.structural(move || {
            let result = (|| -> Result {
                let next = read()?;
                let previous = state.accepted.try_get()?;
                if next != previous {
                    if !previous.domain_matches(&next)
                        || previous.range.range.value != next.range.range.value
                            && state.expected.get() != Some(next.range.range.value)
                    {
                        state.cancel()
                    }
                    state.accepted.try_revise(|value| *value = next)?;
                    state.expected.set(None);
                    state.confirmed.set(None);
                }
                Ok(())
            })();
            if let Err(error) = result {
                state.cancel();
                state.report(error)
            }
        });
        let size = match self.common.size {
            crate::Size::ExtraSmall => 24.0,
            crate::Size::Small => 30.0,
            crate::Size::Medium => 36.0,
            crate::Size::Large => 42.0,
            crate::Size::ExtraLarge => 50.0,
        };
        let horizontal = axis == Orientation::Horizontal;
        let base = Style::new()
            .position(Position::Relative)
            .width(Length::Px(if mapping == Mapping::Angle {
                size * 3. + 12.
            } else if horizontal {
                240.0
            } else {
                size
            }))
            .height(Length::Px(if mapping == Mapping::Angle {
                size * 3. + 12.
            } else if horizontal {
                size
            } else {
                240.0
            }));
        let default_width = base.width;
        let default_height = base.height;
        let (classes, styles) = self.common.appearance(
            scope,
            if mapping == Mapping::Angle {
                "sapid-angle-slider"
            } else {
                "sapid-slider"
            },
            base,
        );
        let geometry = scope.bond(move || {
            let mut style = styles.get();
            style.width = style.width.or(default_width);
            style.height = style.height.or(default_height);
            style.position = Some(Position::Relative);
            style.padding = Some(Edges::all(0.0));
            style.border_width = Some(Edges::all(0.0));
            style
        });
        let mut root = glory::ui::view()
            .role(Role::Slider)
            .class(classes)
            .style(geometry)
            .node_ref(reference)
            .focus(scope.bond(move || {
                if accepted.get().enabled {
                    FocusMode::TabStop
                } else {
                    FocusMode::Programmatic
                }
            }))
            .semantics(scope.bond(move || {
                let state = accepted.get();
                let mut metadata = metadata.get().map(|metadata| *metadata).unwrap_or_default();
                metadata.id = Some(id.clone());
                if let Some(control) = metadata.form_control.as_mut()
                    && control.value.is_none()
                {
                    control.value = Some(state.range.range.value.to_string())
                }
                metadata.slider = Some(state.range);
                metadata.disabled = !state.enabled;
                composite::metadata(metadata)
            }));
        if let Some(name) = self.common.name.clone() {
            root = root.name(name)
        }
        let state = controller.clone();
        let weak = scope.downgrade();
        root = root.on(EventKind::Range, move |event| {
            if let Event::Range { command, .. } = event
                && let Err(error) = state.key(command)
            {
                if let Some(scope) = weak.upgrade() {
                    scope.report_error(error)
                }
                return;
            }
            state.range_events.emit(state.owner, event)
        });
        let state = controller.clone();
        root = root.on(EventKind::Drag, move |event| {
            if let Err(error) = state.pointer(event) {
                state.report(error)
            }
        });
        let resize = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Resize)
                .unwrap_or_default(),
        );
        let state = controller.clone();
        root = root.on(EventKind::Resize, move |event| {
            state.cancel();
            resize.emit(state.owner, event)
        });
        let owner = scope.id();
        for (kind, handlers) in std::mem::take(&mut self.common.handlers) {
            let observers = Observers::new(handlers);
            root = root.on(kind, move |event| observers.emit(owner, event))
        }
        root.show_in(scope)?;
        let parent = scope.child_in(reference.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        if mapping == Mapping::Angle {
            draw_angle(
                scope,
                &parent,
                accepted,
                self.common.theme.clone(),
                self.thumb_color.clone(),
                self.common.unstyled,
            )?;
            scope.retain(parent)?;
            return Ok(());
        }
        let track_color = self.common.theme.border;
        let fill_color = self.common.theme.accent;
        let thumb_color = self.common.theme.surface;
        let unstyled = self.common.unstyled;
        let custom_track = self.track_content.is_some();
        let preview = self.thumb_color.clone();
        let line = move |filled: bool| {
            let state = accepted.get();
            let mut fraction = state.range.range.fraction().unwrap_or(0.0) * 100.0;
            if filled {
                fraction = fraction.clamp(0.0, 100.0)
            } else {
                fraction = 100.0
            }
            let mut style = Style::new().position(Position::Absolute);
            if horizontal {
                style.left = Some(Length::Px(0.0));
                style.top = Some(Length::Px(size / 2.0 - 2.0));
                style.width = Some(Length::Percent(fraction));
                style.height = Some(Length::Px(4.0));
                if direction == Direction::Rtl {
                    style.left = None;
                    style.right = Some(Length::Px(0.0));
                }
            } else {
                style.bottom = Some(Length::Px(0.0));
                style.left = Some(Length::Px(size / 2.0 - 2.0));
                style.height = Some(Length::Percent(fraction));
                style.width = Some(Length::Px(4.0));
            }
            if !unstyled {
                if filled || !custom_track {
                    style.background = Some(if filled { fill_color } else { track_color });
                }
                style.radius = Some(2.0);
            }
            style
        };
        let track = glory::ui::view()
            .class("sapid-slider-track")
            .style(scope.bond(move || line(false)));
        if let Some(content) = self.track_content.take() {
            track.fill(content).show_in(&parent)?;
        } else {
            track.show_in(&parent)?;
            glory::ui::view()
                .class("sapid-slider-fill")
                .style(scope.bond(move || line(true)))
                .show_in(&parent)?;
        }
        let thumb = size * 0.5;
        glory::ui::view()
            .class("sapid-slider-thumb")
            .style(scope.bond(move || {
                let state = accepted.get();
                let mut fraction = state.range.range.fraction().unwrap_or(0.0) * 100.0;
                if horizontal && direction == Direction::Rtl {
                    fraction = 100.0 - fraction
                }
                let mut style = Style::new()
                    .position(Position::Absolute)
                    .width(Length::Px(thumb))
                    .height(Length::Px(thumb));
                if horizontal {
                    style.left = Some(Length::Percent(fraction));
                    style.top = Some(Length::Px((size - thumb) / 2.0));
                    style.margin = Some(Edges {
                        top: 0.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: -thumb / 2.0,
                    });
                } else {
                    style.bottom = Some(Length::Percent(fraction));
                    style.left = Some(Length::Px((size - thumb) / 2.0));
                    style.margin = Some(Edges {
                        top: 0.0,
                        right: 0.0,
                        bottom: -thumb / 2.0,
                        left: 0.0,
                    });
                }
                if !unstyled || preview.is_some() {
                    style.background = Some(preview.as_ref().map_or(thumb_color, Lotus::get));
                }
                if !unstyled {
                    style.border_color = Some(fill_color);
                    style.border_style = Some(glory::style::BorderStyle::Solid);
                    style.border_width = Some(Edges::all(2.0));
                    style.radius = Some(thumb);
                }
                style
            }))
            .show_in(&parent)?;
        scope.retain(parent)?;
        Ok(())
    }
}

fn circle(x: f64, y: f64, r: f64) -> Vec<glory::render::PathCommand> {
    use glory::render::{GraphicPoint as P, PathCommand as C};
    let k = r * 0.552_284_749_830_793_6;
    vec![
        C::Move(P::new(x + r, y)),
        C::Cubic {
            first: P::new(x + r, y + k),
            second: P::new(x + k, y + r),
            end: P::new(x, y + r),
        },
        C::Cubic {
            first: P::new(x - k, y + r),
            second: P::new(x - r, y + k),
            end: P::new(x - r, y),
        },
        C::Cubic {
            first: P::new(x - r, y - k),
            second: P::new(x - k, y - r),
            end: P::new(x, y - r),
        },
        C::Cubic {
            first: P::new(x + k, y - r),
            second: P::new(x + r, y - k),
            end: P::new(x + r, y),
        },
        C::Close,
    ]
}
fn draw_angle(
    scope: &Scope,
    parent: &Scope,
    state: Cage<Snapshot>,
    theme: crate::Theme,
    preview: Option<Lotus<glory::style::Color>>,
    unstyled: bool,
) -> Result {
    use glory::render::{GraphicPoint, GraphicProps, GraphicStroke, PathCommand};
    let accent = if unstyled {
        theme.foreground
    } else {
        theme.accent
    };
    let ring = if unstyled {
        theme.foreground
    } else {
        theme.border
    };
    let style = Style::new()
        .position(Position::Absolute)
        .left(Length::Px(0.))
        .top(Length::Px(0.))
        .width(Length::Percent(100.))
        .height(Length::Percent(100.));
    glory::ui::graphic()
        .class("sapid-angle-slider-ring")
        .style(style.clone())
        .drawing(GraphicProps {
            width: 100.,
            height: 100.,
            commands: circle(50., 50., 40.),
            stroke: Some(GraphicStroke {
                color: ring,
                width: 2.,
                cap: Default::default(),
            }),
            ..Default::default()
        })
        .show_in(parent)?;
    glory::ui::graphic()
        .class("sapid-angle-slider-hand")
        .style(style.clone())
        .drawing(scope.bond(move || {
            let radians = state.with(|s| s.range.range.value.to_radians());
            GraphicProps {
                width: 100.,
                height: 100.,
                commands: vec![
                    PathCommand::Move(GraphicPoint::new(50., 50.)),
                    PathCommand::Line(GraphicPoint::new(
                        50. + 40. * radians.sin(),
                        50. - 40. * radians.cos(),
                    )),
                ],
                stroke: Some(GraphicStroke {
                    color: accent,
                    width: 2.,
                    cap: glory::render::StrokeCap::Round,
                }),
                ..Default::default()
            }
        }))
        .show_in(parent)?;
    glory::ui::graphic()
        .class("sapid-angle-slider-thumb")
        .style(style)
        .drawing(scope.bond(move || {
            let radians = state.with(|s| s.range.range.value.to_radians());
            GraphicProps {
                width: 100.,
                height: 100.,
                commands: circle(50. + 40. * radians.sin(), 50. - 40. * radians.cos(), 5.),
                fill: Some(preview.as_ref().map_or(theme.surface, Lotus::get)),
                stroke: Some(GraphicStroke {
                    color: accent,
                    width: 2.,
                    cap: Default::default(),
                }),
                ..Default::default()
            }
        }))
        .show_in(parent)?;
    Ok(())
}
