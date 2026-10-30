//! One live gesture and one cancellable canvas request, shared by its handles.
use super::*;
use glory::render::{DragPhase, Event, NodeId, Query, QueryToken, QueryValue, Rect};
#[derive(Clone)]
struct Gesture {
    ticket: u64,
    version: u64,
    target: EditTarget,
    anchor: crate::CivilDate,
    kind: ScheduleEditKind,
    start: (f64, f64),
    latest: (f64, f64),
    from: NodeId,
    phase: DragPhase,
    rect: Option<Rect>,
}
pub(super) struct Drag {
    root: Rc<ResourceController>,
    weak: WeakScope,
    canvas: Cage<Option<NodeId>>,
    gesture: RefCell<Option<Gesture>>,
    pending: Cell<Option<QueryToken>>,
    next: Cell<u64>,
    axis: ResourceAxis,
    direction: glory::render::Direction,
    date_days: usize,
    step: u16,
    column_width: glory::Bond<f64>,
}
pub(super) struct Config {
    pub axis: ResourceAxis,
    pub direction: glory::render::Direction,
    pub date_days: usize,
    pub step: u16,
    pub column_width: glory::Bond<f64>,
}
impl Drag {
    pub(super) fn root_report(&self, result: Result) {
        self.root.report(result)
    }
    pub(super) fn new(
        scope: &Scope,
        root: Rc<ResourceController>,
        canvas: Cage<Option<NodeId>>,
        config: Config,
    ) -> Result<Rc<Self>> {
        let Config {
            axis,
            direction,
            date_days,
            step,
            column_width,
        } = config;
        let value = Rc::new(Self {
            root,
            weak: scope.downgrade(),
            canvas,
            gesture: RefCell::new(None),
            pending: Cell::new(None),
            next: Cell::new(0),
            axis,
            direction,
            date_days,
            step,
            column_width,
        });
        let weak = Rc::downgrade(&value);
        scope.on_cleanup(move || {
            if let Some(s) = weak.upgrade() {
                s.cancel();
            }
        })?;
        let weak = Rc::downgrade(&value);
        scope.effect(move || {
            if let Some(s) = weak.upgrade() {
                // Any newly accepted configuration revokes its old geometry.
                s.root.accepted.with(|_| ());
                s.root.enabled.get();
                s.root.readonly.get();
                let stale = s
                    .gesture
                    .borrow()
                    .as_ref()
                    .is_some_and(|g| s.root.accepted.try_version().ok() != Some(g.version));
                if stale || !s.root.ready().unwrap_or(false) {
                    s.cancel()
                }
            }
        });
        Ok(value)
    }
    pub(super) fn cancel(&self) {
        self.gesture.borrow_mut().take();
        if let Some(token) = self.pending.take()
            && let Some(scope) = self.weak.upgrade()
            && let Err(e) = scope.cancel_request(token)
        {
            scope.report_error(e)
        }
    }
    pub(super) fn event(
        self: &Rc<Self>,
        target: EditTarget,
        anchor: crate::CivilDate,
        kind: ScheduleEditKind,
        event: Event,
    ) -> Result {
        let Event::Drag { from, phase, x, y } = event else {
            return Ok(());
        };
        if phase == DragPhase::Cancel || !self.root.ready()? {
            self.cancel();
            self.root.drag_events.emit(self.root.owner, event);
            return Ok(());
        }
        if phase == DragPhase::Start {
            self.cancel();
            let ticket = self.next.get().checked_add(1).ok_or_else(invalid)?;
            self.next.set(ticket);
            let version = self.root.accepted.try_version()?;
            *self.gesture.borrow_mut() = Some(Gesture {
                ticket,
                version,
                target,
                anchor,
                kind,
                start: (x, y),
                latest: (x, y),
                from,
                phase,
                rect: None,
            });
            let scope = self.weak.upgrade().ok_or(glory::ui::Error::Gone)?;
            let canvas = self.canvas.try_get()?.ok_or(glory::ui::Error::Gone)?;
            let weak = Rc::downgrade(self);
            let token = scope.request(Query::Bounds(canvas), move |result| {
                let Some(s) = weak.upgrade() else { return };
                if s.gesture.borrow().as_ref().map(|g| g.ticket) != Some(ticket) {
                    return;
                }
                s.pending.set(None);
                let rect = match result {
                    Ok(QueryValue::Bounds(rect)) if rect.width > 0. && rect.height > 0. => rect,
                    _ => {
                        s.cancel();
                        return;
                    }
                };
                if let Some(g) = s.gesture.borrow_mut().as_mut() {
                    g.rect = Some(rect)
                }
                s.root.report(s.finish());
            })?;
            if self.gesture.borrow().as_ref().map(|g| g.ticket) == Some(ticket) {
                self.pending.set(Some(token))
            } else {
                scope.cancel_request(token)?;
            }
            self.root.drag_events.emit(self.root.owner, event);
        } else {
            let waiting = {
                let mut slot = self.gesture.borrow_mut();
                let Some(g) = slot.as_mut().filter(|g| g.from == from) else {
                    return Ok(());
                };
                g.latest = (x, y);
                g.phase = phase;
                g.rect.is_none()
            };
            if phase == DragPhase::Drop {
                if !waiting {
                    self.finish()?
                }
            } else {
                self.root.drag_events.emit(self.root.owner, event)
            }
        }
        Ok(())
    }
    fn finish(&self) -> Result {
        let ready = self
            .gesture
            .borrow()
            .as_ref()
            .is_some_and(|g| g.phase == DragPhase::Drop && g.rect.is_some());
        if !ready {
            return Ok(());
        }
        let g = self.gesture.borrow_mut().take().unwrap();
        if !self.root.ready()? || self.root.accepted.try_version()? != g.version {
            return Ok(());
        }
        let rect = g.rect.unwrap();
        let count = self
            .root
            .accepted
            .try_with(|m| m.visible_resources().len() * self.date_days)?;
        if count == 0 {
            return Ok(());
        }
        let (coordinate, delta, extent, gutter) = if self.axis == ResourceAxis::Columns {
            (
                g.latest.0 - rect.x,
                g.latest.1 - g.start.1,
                self.column_width.get(),
                64.,
            )
        } else {
            (
                g.latest.1 - rect.y,
                g.latest.0 - g.start.0,
                self.column_width.get(),
                32.,
            )
        };
        let mut column = ((coordinate - gutter) / extent)
            .floor()
            .clamp(0., (count - 1) as f64) as usize;
        if self.axis == ResourceAxis::Columns && self.direction == glory::render::Direction::Rtl {
            column = count - 1 - column
        }
        let mut delta = delta;
        if self.axis == ResourceAxis::Rows && self.direction == glory::render::Direction::Rtl {
            delta = -delta
        }
        let unit = if self.axis == ResourceAxis::Columns {
            32.
        } else {
            64.
        };
        let minutes = (delta / unit)
            .round()
            .clamp(-6_000_000_000., 6_000_000_000.) as i64
            * i64::from(self.step);
        let (resource, day) = self.root.accepted.try_with(|m| {
            (
                m.visible_resources()[column / self.date_days].key().clone(),
                m.data()
                    .window()
                    .start()
                    .date()
                    .checked_add_days((column % self.date_days) as i32)
                    .unwrap(),
            )
        })?;
        let minutes = minutes + i64::from(g.anchor.days_until(day)) * 1440;
        let proposed = match g.kind {
            ScheduleEditKind::Move => g.target.interval.shifted_minutes(minutes),
            ScheduleEditKind::ResizeStart => g.target.interval.resize_start_by_minutes(minutes),
            ScheduleEditKind::ResizeEnd => g.target.interval.resize_end_by_minutes(minutes),
        };
        if let Ok(proposed) = proposed {
            let resource = if g.kind == ScheduleEditKind::Move {
                resource
            } else {
                g.target.resource.clone()
            };
            self.root.proposal(ScheduleEdit {
                event: g.target.event,
                candidate: g.target.candidate,
                kind: g.kind,
                original: g.target.interval,
                proposed,
                original_resource: Some(g.target.resource),
                proposed_resource: Some(resource),
            })?;
        }
        self.root.drag_events.emit(
            self.root.owner,
            Event::Drag {
                from: g.from,
                phase: g.phase,
                x: g.latest.0,
                y: g.latest.1,
            },
        );
        Ok(())
    }
}
