//! One bounded two-dimensional pointer controller shared by ColorPicker hosts.
use super::PickerState;
use glory::render::{DragPhase, Event, NodeId, Query, QueryToken, QueryValue, Rect};
use glory::ui::{Result, WeakScope};
use glory::{Cage, Scope};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone, Copy)]
struct Work {
    ticket: u64,
    start: (f64, f64),
    latest: (f64, f64),
    phase: DragPhase,
    first: bool,
}
pub(super) struct Pad {
    state: Rc<PickerState>,
    weak: WeakScope,
    reference: Cage<Option<NodeId>>,
    work: RefCell<Option<Work>>,
    active: Cell<Option<u64>>,
    pending: Cell<Option<QueryToken>>,
    next: Cell<u64>,
}
impl Pad {
    pub fn new(
        scope: &Scope,
        state: Rc<PickerState>,
        reference: Cage<Option<NodeId>>,
    ) -> Result<Rc<Self>> {
        let value = Rc::new(Self {
            state,
            weak: scope.downgrade(),
            reference,
            work: RefCell::new(None),
            active: Cell::new(None),
            pending: Cell::new(None),
            next: Cell::new(0),
        });
        let weak = Rc::downgrade(&value);
        scope.on_cleanup(move || {
            if let Some(value) = weak.upgrade() {
                value.cancel();
            }
        })?;
        Ok(value)
    }
    fn cancel(&self) {
        self.active.set(None);
        self.work.borrow_mut().take();
        if let Some(token) = self.pending.take()
            && let Some(scope) = self.weak.upgrade()
            && let Err(error) = scope.cancel_request(token)
        {
            scope.report_error(error)
        }
    }
    fn point(&self, ticket: u64, rect: Rect, point: (f64, f64), phase: DragPhase) -> Result {
        if self.active.get() != Some(ticket) || !self.state.ready() {
            return Ok(());
        }
        if !rect.width.is_finite()
            || !rect.height.is_finite()
            || rect.width <= 0.
            || rect.height <= 0.
        {
            return Ok(());
        }
        let mut x = ((point.0 - rect.x) / rect.width).clamp(0., 1.);
        if self.state.direction == glory::render::Direction::Rtl {
            x = 1. - x;
        }
        let value = 1. - ((point.1 - rect.y) / rect.height).clamp(0., 1.);
        let mut next = self.state.current();
        next.saturation = x;
        next.value = value;
        self.state.propose(next, phase == DragPhase::Drop)?;
        if let Some(from) = self.reference.try_get()? {
            self.state.drag_events.emit(
                self.state.owner,
                Event::Drag {
                    from,
                    phase,
                    x: point.0,
                    y: point.1,
                },
            );
        }
        Ok(())
    }
    fn apply(&self, ticket: u64, rect: Rect) -> Result {
        let Some(work) = self
            .work
            .borrow_mut()
            .take()
            .filter(|work| work.ticket == ticket)
        else {
            return Ok(());
        };
        if work.first {
            self.point(ticket, rect, work.start, DragPhase::Start)?;
        }
        if work.phase != DragPhase::Start {
            self.point(ticket, rect, work.latest, work.phase)?;
        }
        if work.phase == DragPhase::Drop && self.active.get() == Some(ticket) {
            self.active.set(None);
        }
        Ok(())
    }
    pub fn event(self: &Rc<Self>, event: Event) -> Result {
        let Event::Drag { phase, x, y, .. } = event else {
            return Ok(());
        };
        if phase == DragPhase::Cancel || !self.state.ready() {
            self.cancel();
            if phase == DragPhase::Cancel {
                self.state.drag_events.emit(self.state.owner, event);
            }
            return Ok(());
        }
        if phase == DragPhase::Start {
            self.cancel();
            let ticket =
                self.next
                    .get()
                    .checked_add(1)
                    .ok_or(glory::render::Error::InvalidProps(
                        "color drag ticket exhausted",
                    ))?;
            self.next.set(ticket);
            self.active.set(Some(ticket));
            *self.work.borrow_mut() = Some(Work {
                ticket,
                start: (x, y),
                latest: (x, y),
                phase,
                first: true,
            });
        } else {
            let Some(ticket) = self.active.get() else {
                return Ok(());
            };
            let mut work = self.work.borrow_mut();
            if let Some(work) = work.as_mut() {
                work.latest = (x, y);
                work.phase = phase;
            } else {
                *work = Some(Work {
                    ticket,
                    start: (x, y),
                    latest: (x, y),
                    phase,
                    first: false,
                });
            }
        }
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
            if state.active.get() != Some(ticket) {
                return;
            }
            state.pending.set(None);
            let result = (|| -> Result {
                let QueryValue::Bounds(rect) = result? else {
                    return Err(glory::render::Error::InvalidProps(
                        "color plane bounds response mismatch",
                    )
                    .into());
                };
                state.apply(ticket, rect)
            })();
            if let Err(error) = result {
                state.cancel();
                if let Some(scope) = state.weak.upgrade() {
                    scope.report_error(error)
                }
            }
        })?;
        if self.active.get() == Some(ticket) {
            self.pending.set(Some(token));
        } else {
            scope.cancel_request(token)?;
        }
        Ok(())
    }
}
