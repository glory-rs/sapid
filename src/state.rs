use std::cell::{Cell, RefCell};
use std::rc::Rc;

use glory::core::Lotus;
use glory::ui::Result;
use glory::{Cage, Scope};

/// Configuration has exactly one source of truth. Defaults never switch a
/// controlled widget back to local ownership, regardless of setter order.
pub(crate) struct Value<T: 'static> {
    pub source: Option<Lotus<T>>,
    pub default: T,
}

impl<T: Default> Default for Value<T> {
    fn default() -> Self {
        Self {
            source: None,
            default: T::default(),
        }
    }
}

impl<T: Clone> Value<T> {
    pub fn mount(self, scope: &Scope) -> Control<T> {
        match self.source {
            Some(source) => Control {
                source,
                local: None,
            },
            None => {
                let local = scope.cage(self.default);
                Control {
                    source: local.into(),
                    local: Some(local),
                }
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct Control<T: 'static> {
    pub source: Lotus<T>,
    local: Option<Cage<T>>,
}

impl<T: Clone + PartialEq> Control<T> {
    pub fn accept(&self, proposal: &T) -> Result {
        if let Some(local) = self.local
            && local.try_with(|current| current != proposal)?
        {
            local.try_revise(|current| *current = proposal.clone())?;
        }
        Ok(())
    }
}

/// A host tick can contain several edits at the same input revision. Reconcile
/// after the whole tick, never invalidate later events inside an earlier handler.
#[derive(Clone)]
pub(crate) struct Dirty {
    pulse: Cage<()>,
    pending: Rc<Cell<bool>>,
}

impl Dirty {
    pub fn new(scope: &Scope) -> Self {
        Self {
            pulse: scope.cage(()),
            pending: Rc::new(Cell::new(false)),
        }
    }

    pub fn touch(&self) -> Result {
        self.pending.set(true);
        self.pulse.try_revise(|_| ())?;
        Ok(())
    }

    pub fn take(&self) -> bool {
        self.pulse.get();
        self.pending.replace(false)
    }
}

pub(crate) type DelayFactory = Rc<dyn Fn() -> glory::core::LocalFuture>;

/// One cancellable, Owner-owned pending operation. The generation is local to
/// the live controller; completion/cancellation never keeps a visited history.
pub(crate) struct OwnerDelay {
    pending: RefCell<Option<glory::core::TaskToken>>,
    ticket: Cell<Option<u64>>,
    next: Cell<u64>,
}
impl OwnerDelay {
    pub fn new(scope: &Scope) -> Result<Rc<Self>> {
        let value = Rc::new(Self {
            pending: RefCell::new(None),
            ticket: Cell::new(None),
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
    pub fn cancel(&self) {
        self.ticket.set(None);
        let task = self.pending.borrow_mut().take();
        if let Some(task) = task {
            task.cancel();
        }
    }
    pub fn run(
        self: &Rc<Self>,
        scope: &Scope,
        factory: Option<&DelayFactory>,
        callback: impl FnOnce() -> Result + 'static,
    ) -> Result {
        self.cancel();
        let Some(factory) = factory else {
            return callback();
        };
        let ticket =
            self.next
                .get()
                .checked_add(1)
                .ok_or(glory::render::Error::InvalidProps(
                    "widget delay identity exhausted",
                ))?;
        self.next.set(ticket);
        self.ticket.set(Some(ticket));
        // User future construction and cancellation run without state borrows.
        let future = factory();
        if !scope.id().is_alive() || self.ticket.get() != Some(ticket) {
            return Ok(());
        }
        let weak = Rc::downgrade(self);
        let owner = scope.downgrade();
        let task = scope.spawn(async move {
            future.await;
            if let (Some(state), Some(scope)) = (weak.upgrade(), owner.upgrade())
                && state.ticket.get() == Some(ticket)
            {
                state.ticket.set(None);
                let _completed = state.pending.borrow_mut().take();
                if let Err(error) = callback() {
                    scope.report_error(error);
                }
            }
        })?;
        if self.ticket.get() == Some(ticket) {
            let _previous = self.pending.borrow_mut().replace(task);
        } else {
            task.cancel();
        }
        Ok(())
    }
}
