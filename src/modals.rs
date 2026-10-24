//! One bounded application-local modal queue, rendered by the existing Layers.
use crate::{
    Theme,
    composite::{Callback, Factory},
};
use glory::render::DismissReason;
use glory::ui::{Result, Widget};
use glory::{Cage, Scope};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

pub const MAX_PENDING_MODALS: usize = 32;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalOutcome {
    Closed,
    Confirmed,
    Cancelled,
    Dismissed(DismissReason),
}
/// A request is admitted only after all bounded text has been validated.
pub struct ModalRequest {
    title: String,
    description: String,
    confirmation: bool,
    confirm_label: String,
    close_label: String,
    content: Option<Factory>,
    result: Option<Box<dyn FnMut(ModalOutcome)>>,
}
pub fn modal_request(title: impl Into<String>, description: impl Into<String>) -> ModalRequest {
    request(title.into(), description.into(), false)
}
pub fn confirmation_request(
    title: impl Into<String>,
    description: impl Into<String>,
) -> ModalRequest {
    request(title.into(), description.into(), true)
}
fn request(title: String, description: String, confirmation: bool) -> ModalRequest {
    ModalRequest {
        title,
        description,
        confirmation,
        confirm_label: "Confirm".into(),
        close_label: if confirmation { "Cancel" } else { "Close" }.into(),
        content: None,
        result: None,
    }
}
impl ModalRequest {
    pub fn confirm_label(mut self, text: impl Into<String>) -> Self {
        self.confirm_label = text.into();
        self
    }
    pub fn close_label(mut self, text: impl Into<String>) -> Self {
        self.close_label = text.into();
        self
    }
    pub fn content<W: Widget>(mut self, mut factory: impl FnMut() -> W + 'static) -> Self {
        self.content = Some(Box::new(move || Box::new(factory())));
        self
    }
    pub fn on_result(mut self, callback: impl FnMut(ModalOutcome) + 'static) -> Self {
        self.result = Some(Box::new(callback));
        self
    }
}
#[derive(Clone)]
struct Entry {
    id: u64,
    title: String,
    description: String,
    confirmation: bool,
    confirm_label: String,
    close_label: String,
    content: Rc<RefCell<Option<Factory>>>,
    result: Callback<ModalOutcome>,
}
struct Controller {
    owner: glory::core::InstanceId,
    weak: glory::ui::WeakScope,
    live: Cell<bool>,
    next: Cell<u64>,
    entries: Cage<Vec<Entry>>,
    capacity: usize,
}
#[derive(Clone)]
pub struct ModalToken {
    controller: Weak<Controller>,
    id: u64,
}
#[derive(Clone)]
pub struct ModalsHandle(Rc<Controller>);
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn text(value: &str, max: usize) -> Result {
    if value.trim().is_empty() || value.len() > max || value.contains(['\0', '\r']) {
        Err(invalid(
            "modal text must be nonempty, canonical and within its byte budget",
        ))
    } else {
        Ok(())
    }
}
impl ModalsHandle {
    fn ensure(&self) -> Result {
        if self.0.live.get() && self.0.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn index(&self, token: &ModalToken) -> Result<Option<usize>> {
        self.ensure()?;
        if !Weak::ptr_eq(&token.controller, &Rc::downgrade(&self.0)) {
            return Err(invalid("modal token belongs to another manager"));
        }
        Ok(self.0.entries.try_with_untracked(|entries| {
            entries.iter().position(|entry| entry.id == token.id)
        })?)
    }
    /// Enqueues FIFO. At most one request has a mounted Layer at any instant.
    pub fn open(&self, request: ModalRequest) -> Result<ModalToken> {
        self.ensure()?;
        text(&request.title, 1024)?;
        text(&request.description, 16_384)?;
        text(&request.confirm_label, 256)?;
        text(&request.close_label, 256)?;
        if self.pending()? >= self.0.capacity {
            return Err(invalid("modal queue capacity exceeded"));
        }
        let id = self
            .0
            .next
            .get()
            .checked_add(1)
            .ok_or_else(|| invalid("modal identity exhausted"))?;
        let entry = Entry {
            id,
            title: request.title,
            description: request.description,
            confirmation: request.confirmation,
            confirm_label: request.confirm_label,
            close_label: request.close_label,
            content: Rc::new(RefCell::new(request.content)),
            result: Callback::new(request.result),
        };
        self.0
            .entries
            .try_revise(move |entries| entries.push(entry))?;
        self.0.next.set(id);
        Ok(ModalToken {
            controller: Rc::downgrade(&self.0),
            id,
        })
    }
    /// Updates accepted metadata in place; content factory and operation identity stay stable.
    pub fn update(
        &self,
        token: &ModalToken,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<bool> {
        let Some(index) = self.index(token)? else {
            return Ok(false);
        };
        let title = title.into();
        let description = description.into();
        text(&title, 1024)?;
        text(&description, 16_384)?;
        self.0.entries.try_revise(move |entries| {
            entries[index].title = title;
            entries[index].description = description;
        })?;
        Ok(true)
    }
    pub fn close(&self, token: &ModalToken) -> Result<bool> {
        self.resolve(token, ModalOutcome::Closed)
    }
    fn resolve(&self, token: &ModalToken, outcome: ModalOutcome) -> Result<bool> {
        let Some(index) = self.index(token)? else {
            return Ok(false);
        };
        // Removal and publication precede user callbacks; no queue/value borrow is held.
        let entry = self.0.entries.try_revise(|entries| entries.remove(index))?;
        self.notify(&entry, outcome);
        Ok(true)
    }
    fn notify(&self, entry: &Entry, outcome: ModalOutcome) {
        if let Some(scope) = self.0.weak.upgrade() {
            glory::core::untracked(|| {
                scope.owner().enter(|| {
                    entry.result.emit(self.0.owner, outcome);
                    entry.result.clear();
                })
            });
        } else {
            entry.result.clear();
        }
    }
    pub fn close_all(&self) -> Result<usize> {
        self.ensure()?;
        if self.pending()? == 0 {
            return Ok(0);
        }
        let entries = self.0.entries.try_revise(std::mem::take)?;
        let count = entries.len();
        for entry in entries {
            self.notify(&entry, ModalOutcome::Closed);
        }
        Ok(count)
    }
    pub fn pending(&self) -> Result<usize> {
        self.ensure()?;
        Ok(self.0.entries.try_with_untracked(Vec::len)?)
    }
}
pub struct ModalsManager {
    key: String,
    theme: Theme,
    capacity: usize,
    ready: Option<Box<dyn FnMut(ModalsHandle)>>,
}
pub fn modals_manager(key: impl Into<String>) -> ModalsManager {
    ModalsManager {
        key: key.into(),
        theme: Theme::default(),
        capacity: 16,
        ready: None,
    }
}
impl ModalsManager {
    pub fn theme(mut self, value: Theme) -> Self {
        self.theme = value;
        self
    }
    pub fn capacity(mut self, value: usize) -> Self {
        self.capacity = value;
        self
    }
    pub fn on_ready(mut self, callback: impl FnMut(ModalsHandle) + 'static) -> Self {
        self.ready = Some(Box::new(callback));
        self
    }
}
impl Widget for ModalsManager {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.key.is_empty()
            || self.key.len() > 32
            || !(1..=MAX_PENDING_MODALS).contains(&self.capacity)
        {
            return Err(invalid(
                "modal manager needs a key of 1..=32 bytes and capacity of 1..=32",
            ));
        }
        crate::composite::id("modals", &self.key, "root", "root")?;
        // A permanently closed host boundary makes unsupported Layer admission
        // fail at mount, before an imperative request can enter the queue.
        glory::ui::view()
            .role(glory::render::Role::Dialog)
            .focus(glory::render::FocusMode::Programmatic)
            .name("Queued dialogs")
            .class("sapid-modals-boundary")
            .semantics(Some(Box::new(glory::render::Semantics {
                id: Some(crate::composite::id(
                    "modals", &self.key, "host", "boundary",
                )?),
                hidden: true,
                layer: Some(glory::render::Layer::default()),
                ..Default::default()
            })))
            .show_in(scope)?;
        let entries = scope.cage(Vec::<Entry>::new());
        let controller = Rc::new(Controller {
            owner: scope.id(),
            weak: scope.downgrade(),
            live: Cell::new(true),
            next: Cell::new(0),
            entries,
            capacity: self.capacity,
        });
        let handle = ModalsHandle(controller.clone());
        let cleanup = controller.clone();
        scope.on_cleanup(move || cleanup.live.set(false))?;
        let current = scope.bond(move || {
            entries.with(|entries| entries.first().cloned().into_iter().collect::<Vec<_>>())
        });
        let key = self.key.clone();
        let theme = self.theme.clone();
        let command = handle.clone();
        glory::ui::each(
            current,
            |entry| entry.id,
            move |item, _| {
                let key = key.clone();
                let theme = theme.clone();
                let command = command.clone();
                glory::ui::component(move |scope| {
                    let entry = item.get();
                    let title = scope.bond(move || item.with(|entry| entry.title.clone()));
                    let description =
                        scope.bond(move || item.with(|entry| entry.description.clone()));
                    let token = ModalToken {
                        controller: Rc::downgrade(&command.0),
                        id: entry.id,
                    };
                    let content = entry.content.clone();
                    let factory = move || {
                        // Take the callback before user construction, permitting reentrant commands.
                        let callback = content.borrow_mut().take();
                        let result: Box<dyn Widget> = if let Some(mut callback) = callback {
                            let widget = callback();
                            *content.borrow_mut() = Some(callback);
                            widget
                        } else {
                            Box::new(glory::ui::component(|_| Ok(())))
                        };
                        result
                    };
                    let modal_key = format!("{key}-{}", entry.id);
                    let change = command.clone();
                    let changed = token.clone();
                    let weak = scope.downgrade();
                    let close = move |open: bool| {
                        if !open
                            && let Err(error) = change.close(&changed)
                            && let Some(scope) = weak.upgrade()
                        {
                            scope.report_error(error)
                        }
                    };
                    if entry.confirmation {
                        let confirmed = command.clone();
                        let confirmed_token = token.clone();
                        let weak_confirm = scope.downgrade();
                        let cancelled = command.clone();
                        let cancel_token = token.clone();
                        let weak_cancel = scope.downgrade();
                        crate::alert_dialog(modal_key)
                            .show_trigger(false)
                            .open(true)
                            .theme(theme.clone())
                            .title(title)
                            .description(description)
                            .confirm_label(entry.confirm_label)
                            .cancel_label(entry.close_label)
                            .content(factory)
                            .on_confirm(move || {
                                if let Err(error) =
                                    confirmed.resolve(&confirmed_token, ModalOutcome::Confirmed)
                                    && let Some(scope) = weak_confirm.upgrade()
                                {
                                    scope.report_error(error)
                                }
                            })
                            .on_cancel(move |reason| {
                                let outcome = match reason {
                                    crate::AlertCancelReason::Cancel => ModalOutcome::Cancelled,
                                    crate::AlertCancelReason::Dismiss(reason) => {
                                        ModalOutcome::Dismissed(reason)
                                    }
                                };
                                if let Err(error) = cancelled.resolve(&cancel_token, outcome)
                                    && let Some(scope) = weak_cancel.upgrade()
                                {
                                    scope.report_error(error)
                                }
                            })
                            .on_open_change(close)
                            .show_in(scope)
                    } else {
                        let dismissed = command.clone();
                        let dismissed_token = token.clone();
                        let weak_dismiss = scope.downgrade();
                        crate::dialog(modal_key)
                            .show_trigger(false)
                            .open(true)
                            .theme(theme.clone())
                            .title(title)
                            .description(description)
                            .close_label(entry.close_label)
                            .content(factory)
                            .on_dismiss(move |reason| {
                                if let Err(error) = dismissed
                                    .resolve(&dismissed_token, ModalOutcome::Dismissed(reason))
                                    && let Some(scope) = weak_dismiss.upgrade()
                                {
                                    scope.report_error(error)
                                }
                            })
                            .on_open_change(close)
                            .show_in(scope)
                    }
                })
            },
        )
        .show_in(scope)?;
        if let Some(mut callback) = self.ready.take() {
            callback(handle)
        }
        Ok(())
    }
}
