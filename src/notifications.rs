//! Bounded notifications with a single owner-bound timer for visible deadlines.
use crate::{AlertSeverity, Announcement, Theme, composite::Callback, state::OwnerDelay};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Scope};
use std::{
    cell::Cell,
    future::Future,
    rc::{Rc, Weak},
    time::Duration,
};

pub const MAX_NOTIFICATIONS: usize = 64;
pub const MAX_VISIBLE_NOTIFICATIONS: usize = 8;
/// Host time and cancellable sleeping are explicitly supplied by the application.
/// These functions are host services, not reactive computations.
#[derive(Clone)]
pub struct NotificationClock {
    now: Rc<dyn Fn() -> Duration>,
    sleep: Rc<dyn Fn(Duration) -> glory::core::LocalFuture>,
}
impl NotificationClock {
    pub fn new<F: Future<Output = ()> + 'static>(
        now: impl Fn() -> Duration + 'static,
        sleep: impl Fn(Duration) -> F + 'static,
    ) -> Self {
        Self {
            now: Rc::new(now),
            sleep: Rc::new(move |duration| Box::pin(sleep(duration))),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationCloseReason {
    Dismissed,
    Action,
    Expired,
    Replaced,
    Cleared,
}
type NotificationAction = (glory::Lotus<String>, Box<dyn FnMut()>);
/// A standalone live message with a real optional action and independent close.
pub struct Notification {
    inner: crate::Alert,
    theme: Theme,
    unstyled: bool,
    class: glory::Lotus<String>,
    action: Option<NotificationAction>,
}
pub fn notification(key: impl Into<String>) -> Notification {
    Notification {
        inner: crate::alert(key).closable(true),
        theme: Theme::default(),
        unstyled: false,
        class: "".into(),
        action: None,
    }
}
impl Notification {
    pub fn title(mut self, value: impl Into<glory::Lotus<String>>) -> Self {
        self.inner = self.inner.title(value);
        self
    }
    pub fn content(mut self, value: impl Into<glory::Lotus<String>>) -> Self {
        self.inner = self.inner.content(value);
        self
    }
    pub fn visible(mut self, value: impl Into<glory::Lotus<bool>>) -> Self {
        self.inner = self.inner.visible(value);
        self
    }
    pub fn closable(mut self, value: bool) -> Self {
        self.inner = self.inner.closable(value);
        self
    }
    pub fn close_label(mut self, value: impl Into<glory::Lotus<String>>) -> Self {
        self.inner = self.inner.close_label(value);
        self
    }
    pub fn on_close(mut self, callback: impl FnMut() + 'static) -> Self {
        self.inner = self.inner.on_close(callback);
        self
    }
    pub fn action(
        mut self,
        label: impl Into<glory::Lotus<String>>,
        callback: impl FnMut() + 'static,
    ) -> Self {
        self.action = Some((label.into(), Box::new(callback)));
        self
    }
    pub fn theme(mut self, value: Theme) -> Self {
        self.theme = value.clone();
        self.inner = self.inner.theme(value);
        self
    }
    pub fn severity(mut self, value: AlertSeverity) -> Self {
        self.inner = self.inner.severity(value);
        self
    }
    pub fn announcement(mut self, value: Announcement) -> Self {
        self.inner = self.inner.announcement(value);
        self
    }
    pub fn class(mut self, value: impl Into<glory::Lotus<String>>) -> Self {
        self.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<glory::Lotus<glory::style::Style>>) -> Self {
        self.inner = self.inner.style(value);
        self
    }
    pub fn name(mut self, value: impl Into<glory::Lotus<String>>) -> Self {
        self.inner = self.inner.name(value);
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<glory::render::NodeId>>) -> Self {
        self.inner = self.inner.node_ref(value);
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.unstyled = true;
        self.inner = self.inner.unstyled();
        self
    }
}
impl Widget for Notification {
    fn build(&mut self, scope: &Scope) -> Result {
        let class = self.class.clone();
        let mut inner = std::mem::replace(&mut self.inner, crate::alert(String::new()))
            .class(scope.bond(move || format!("sapid-notification {}", class.get())));
        if let Some((label, callback)) = self.action.take() {
            let label = crate::composite::validated(scope, &label, |label| text(label, 256))?;
            let mut action = crate::button()
                .theme(self.theme.clone())
                .label(label)
                .on_press(callback);
            if self.unstyled {
                action = action.unstyled()
            }
            inner = inner.fill(action);
        }
        inner.build(scope)
    }
}
pub struct NotificationRequest {
    key: String,
    title: String,
    message: String,
    severity: AlertSeverity,
    announcement: Announcement,
    timeout: Option<Duration>,
    action: Option<(String, Box<dyn FnMut()>)>,
    close: Option<Box<dyn FnMut(NotificationCloseReason)>>,
}
pub fn notification_request(
    key: impl Into<String>,
    title: impl Into<String>,
    message: impl Into<String>,
) -> NotificationRequest {
    NotificationRequest {
        key: key.into(),
        title: title.into(),
        message: message.into(),
        severity: AlertSeverity::Info,
        announcement: Announcement::Polite,
        timeout: None,
        action: None,
        close: None,
    }
}
impl NotificationRequest {
    pub fn severity(mut self, value: AlertSeverity) -> Self {
        self.severity = value;
        self
    }
    pub fn announcement(mut self, value: Announcement) -> Self {
        self.announcement = value;
        self
    }
    pub fn timeout(mut self, value: Duration) -> Self {
        self.timeout = Some(value);
        self
    }
    pub fn action(
        mut self,
        label: impl Into<String>,
        callback: impl FnMut() + 'static,
    ) -> Self {
        self.action = Some((label.into(), Box::new(callback)));
        self
    }
    pub fn on_close(mut self, callback: impl FnMut(NotificationCloseReason) + 'static) -> Self {
        self.close = Some(Box::new(callback));
        self
    }
}
#[derive(Clone)]
struct Entry {
    id: u64,
    key: String,
    title: String,
    message: String,
    severity: AlertSeverity,
    announcement: Announcement,
    remaining: Option<Duration>,
    deadline: Option<Duration>,
    action: Option<(String, Callback<()>)>,
    close: Callback<NotificationCloseReason>,
}
#[derive(Default)]
struct Queue {
    entries: Vec<Entry>,
    paused: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Stamp {
    id: u64,
    deadline: Duration,
}
struct Controller {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    live: Cell<bool>,
    next: Cell<u64>,
    queue: Cage<Queue>,
    reschedule: Cage<()>,
    capacity: usize,
    visible: usize,
    clock: Option<NotificationClock>,
    delay: Rc<OwnerDelay>,
    scheduled: Cell<Option<Stamp>>,
    last_now: Cell<Duration>,
    hover: Cell<Option<u64>>,
    focus: Cell<Option<u64>>,
}
#[derive(Clone)]
pub struct NotificationToken {
    controller: Weak<Controller>,
    id: u64,
}
#[derive(Clone)]
pub struct NotificationsHandle(Rc<Controller>);
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn text(value: &str, max: usize) -> Result {
    if value.trim().is_empty() || value.len() > max || value.contains(['\0', '\r']) {
        Err(invalid(
            "notification text must be nonempty, canonical and within its byte budget",
        ))
    } else {
        Ok(())
    }
}
impl NotificationsHandle {
    fn ensure(&self) -> Result {
        if self.0.live.get() && self.0.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn index(&self, token: &NotificationToken) -> Result<Option<usize>> {
        self.ensure()?;
        if !Weak::ptr_eq(&token.controller, &Rc::downgrade(&self.0)) {
            return Err(invalid("notification token belongs to another toaster"));
        }
        Ok(self
            .0
            .queue
            .try_with_untracked(|q| q.entries.iter().position(|e| e.id == token.id))?)
    }
    /// Replaces an existing complete key at its FIFO position, revoking its old token.
    pub fn show(&self, request: NotificationRequest) -> Result<NotificationToken> {
        self.ensure()?;
        text(&request.key, 64)?;
        text(&request.title, 256)?;
        text(&request.message, 4096)?;
        if let Some((label, _)) = &request.action {
            text(label, 256)?
        }
        if let Some(timeout) = request.timeout
            && (self.0.clock.is_none()
                || timeout.is_zero()
                || timeout > Duration::from_secs(86_400))
        {
            return Err(invalid(
                "notification timeout requires a clock and a duration in 0..=24h",
            ));
        }
        let existing = self
            .0
            .queue
            .try_with_untracked(|q| q.entries.iter().position(|e| e.key == request.key))?;
        if existing.is_none() && self.pending()? == self.0.capacity {
            return Err(invalid("notification queue capacity exceeded"));
        }
        let id = self
            .0
            .next
            .get()
            .checked_add(1)
            .ok_or_else(|| invalid("notification identity exhausted"))?;
        let action = request.action.map(|(label, mut callback)| {
            (label, Callback::new(Some(Box::new(move |()| callback()))))
        });
        let entry = Entry {
            id,
            key: request.key,
            title: request.title,
            message: request.message,
            severity: request.severity,
            announcement: request.announcement,
            remaining: request.timeout,
            deadline: None,
            action,
            close: Callback::new(request.close),
        };
        let old = self.0.queue.try_revise(move |q| {
            if let Some(index) = existing {
                Some(std::mem::replace(&mut q.entries[index], entry))
            } else {
                q.entries.push(entry);
                None
            }
        })?;
        self.0.next.set(id);
        if existing.is_some_and(|index| index < self.0.visible) {
            self.0.cancel_timer()
        }
        if let Some(old) = old {
            self.0
                .notify(&old, NotificationCloseReason::Replaced, false)
        }
        Ok(NotificationToken {
            controller: Rc::downgrade(&self.0),
            id,
        })
    }
    pub fn update(
        &self,
        token: &NotificationToken,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<bool> {
        let Some(index) = self.index(token)? else {
            return Ok(false);
        };
        let title = title.into();
        let message = message.into();
        text(&title, 256)?;
        text(&message, 4096)?;
        self.0.queue.try_revise(move |q| {
            q.entries[index].title = title;
            q.entries[index].message = message;
        })?;
        Ok(true)
    }
    pub fn dismiss(&self, token: &NotificationToken) -> Result<bool> {
        self.resolve(token, NotificationCloseReason::Dismissed, false)
    }
    fn resolve(
        &self,
        token: &NotificationToken,
        reason: NotificationCloseReason,
        action: bool,
    ) -> Result<bool> {
        let Some(index) = self.index(token)? else {
            return Ok(false);
        };
        let entry = self.0.queue.try_revise(|q| q.entries.remove(index))?;
        if index < self.0.visible {
            self.0.cancel_timer()
        }
        self.0.notify(&entry, reason, action);
        Ok(true)
    }
    pub fn clear(&self) -> Result<usize> {
        self.ensure()?;
        if self.pending()? == 0 {
            return Ok(0);
        }
        let entries = self
            .0
            .queue
            .try_revise(|q| std::mem::take(&mut q.entries))?;
        self.0.cancel_timer();
        let count = entries.len();
        for entry in entries {
            self.0
                .notify(&entry, NotificationCloseReason::Cleared, false)
        }
        Ok(count)
    }
    pub fn pending(&self) -> Result<usize> {
        self.ensure()?;
        Ok(self.0.queue.try_with_untracked(|q| q.entries.len())?)
    }
    /// Pauses remaining visible lifetimes at the next flush/host boundary.
    pub fn pause(&self, paused: bool) -> Result {
        self.set_pause(4, paused)
    }
    fn set_pause(&self, bit: u8, paused: bool) -> Result {
        self.ensure()?;
        let old = self.0.queue.try_with_untracked(|q| q.paused)?;
        let next = if paused { old | bit } else { old & !bit };
        if next == old {
            return Ok(());
        }
        self.0.queue.try_revise(|q| q.paused = next)?;
        self.0.cancel_timer();
        Ok(())
    }
    fn activity(&self, bit: u8, id: u64, active: bool) -> Result {
        self.ensure()?;
        let slot = if bit == 1 {
            &self.0.hover
        } else {
            &self.0.focus
        };
        if !active && slot.get() != Some(id) {
            return Ok(());
        }
        let previous = self.0.queue.try_with_untracked(|q| q.paused)?;
        let next = if active {
            previous | bit
        } else {
            previous & !bit
        };
        let origin = active.then_some(id);
        if next == previous {
            slot.set(origin);
            return Ok(());
        }
        self.0.queue.try_revise(|q| {
            q.paused = next;
            slot.set(origin);
        })?;
        // Publish the live origin before cancellation: a future destructor may
        // reenter and retire this row, whose cleanup must see that origin.
        self.0.cancel_timer();
        Ok(())
    }
}
impl Controller {
    fn cancel_timer(&self) {
        self.scheduled.set(None);
        self.delay.cancel();
    }
    fn notify(&self, entry: &Entry, reason: NotificationCloseReason, action: bool) {
        if let Some(scope) = self.weak.upgrade() {
            glory::core::untracked(|| {
                scope.owner().enter(|| {
                    if action && let Some((_, callback)) = &entry.action {
                        callback.emit(self.owner, ());
                    }
                    entry.close.emit(self.owner, reason);
                    if let Some((_, callback)) = &entry.action {
                        callback.clear()
                    }
                    entry.close.clear();
                })
            });
        } else {
            if let Some((_, callback)) = &entry.action {
                callback.clear()
            }
            entry.close.clear()
        }
    }
    fn now(&self) -> Result<Duration> {
        let clock = self
            .clock
            .as_ref()
            .ok_or_else(|| invalid("notification clock missing"))?;
        let now = (clock.now)();
        if now < self.last_now.get() {
            return Err(invalid("notification clock regressed"));
        }
        self.last_now.set(now);
        Ok(now)
    }
    fn synchronize(self: &Rc<Self>) -> Result {
        if !self.live.get() || !self.owner.is_alive() {
            return Ok(());
        }
        let version = self.queue.try_version()?;
        let (paused, times) = self.queue.try_with_untracked(|q| {
            (
                q.paused != 0,
                q.entries
                    .iter()
                    .take(self.visible)
                    .filter_map(|e| e.remaining.map(|remaining| (e.id, remaining, e.deadline)))
                    .collect::<Vec<_>>(),
            )
        })?;
        if times.is_empty() {
            self.cancel_timer();
            return Ok(());
        }
        let now = self.now()?;
        if !self.owner.is_alive() || self.queue.try_version()? != version {
            return Ok(());
        }
        let mut changes = Vec::new();
        for &(id, remaining, deadline) in &times {
            match (paused, deadline) {
                (true, Some(deadline)) => {
                    changes.push((id, deadline.saturating_sub(now), None))
                }
                (false, None) => changes.push((
                    id,
                    remaining,
                    Some(
                        now.checked_add(remaining)
                            .ok_or_else(|| invalid("notification deadline exhausted"))?,
                    ),
                )),
                _ => {}
            }
        }
        if !changes.is_empty() {
            self.queue.try_revise(|q| {
                for (id, remaining, deadline) in changes {
                    if let Some(e) =
                        q.entries.iter_mut().take(self.visible).find(|e| e.id == id)
                    {
                        e.remaining = Some(remaining);
                        e.deadline = deadline;
                    }
                }
            })?;
            self.cancel_timer();
            return Ok(());
        }
        if paused {
            self.cancel_timer();
            return Ok(());
        }
        let stamp = times
            .iter()
            .filter_map(|&(id, _, deadline)| deadline.map(|deadline| Stamp { id, deadline }))
            .min_by_key(|stamp| stamp.deadline);
        if self.scheduled.get() == stamp {
            return Ok(());
        }
        let Some(stamp) = stamp else {
            self.cancel_timer();
            return Ok(());
        };
        let scope = self.weak.upgrade().ok_or(glory::ui::Error::Gone)?;
        let clock = self
            .clock
            .clone()
            .ok_or_else(|| invalid("notification clock missing"))?;
        let delay = stamp.deadline.saturating_sub(now);
        let factory: crate::state::DelayFactory = Rc::new(move || (clock.sleep)(delay));
        let weak = Rc::downgrade(self);
        self.scheduled.set(Some(stamp));
        let result = self.delay.run(&scope, Some(&factory), move || {
            if let Some(controller) = weak.upgrade() {
                controller.wake(stamp)?
            }
            Ok(())
        });
        if result.is_err() && self.scheduled.get() == Some(stamp) {
            self.scheduled.set(None)
        }
        result
    }
    fn wake(self: &Rc<Self>, stamp: Stamp) -> Result {
        if !self.live.get() || !self.owner.is_alive() || self.scheduled.get() != Some(stamp) {
            return Ok(());
        }
        self.scheduled.set(None);
        let version = self.queue.try_version()?;
        let now = self.now()?;
        if !self.owner.is_alive() || self.queue.try_version()? != version {
            return Ok(());
        }
        let expired = self.queue.try_with_untracked(|q| {
            if q.paused != 0 {
                Vec::new()
            } else {
                q.entries
                    .iter()
                    .take(self.visible)
                    .filter(|e| e.deadline.is_some_and(|deadline| deadline <= now))
                    .map(|e| e.id)
                    .collect::<Vec<_>>()
            }
        })?;
        if !expired.is_empty() {
            let removed = self.queue.try_revise(|q| {
                let mut removed = Vec::new();
                let mut index = 0;
                while index < q.entries.len() {
                    if expired.contains(&q.entries[index].id) {
                        removed.push(q.entries.remove(index));
                    } else {
                        index += 1
                    }
                }
                removed
            })?;
            for entry in removed {
                self.notify(&entry, NotificationCloseReason::Expired, false)
            }
        } else {
            self.reschedule.try_revise(|_| ())?;
        }
        Ok(())
    }
}

pub struct Toaster {
    key: String,
    theme: Theme,
    capacity: usize,
    visible: usize,
    clock: Option<NotificationClock>,
    hover_pause: bool,
    ready: Option<Box<dyn FnMut(NotificationsHandle)>>,
}
pub fn toaster(key: impl Into<String>) -> Toaster {
    Toaster {
        key: key.into(),
        theme: Theme::default(),
        capacity: 32,
        visible: 4,
        clock: None,
        hover_pause: false,
        ready: None,
    }
}
pub fn notifications(key: impl Into<String>) -> Toaster {
    toaster(key)
}
impl Toaster {
    pub fn theme(mut self, value: Theme) -> Self {
        self.theme = value;
        self
    }
    pub fn capacity(mut self, value: usize) -> Self {
        self.capacity = value;
        self
    }
    pub fn visible_limit(mut self, value: usize) -> Self {
        self.visible = value;
        self
    }
    pub fn clock(mut self, value: NotificationClock) -> Self {
        self.clock = Some(value);
        self
    }
    pub fn pause_on_hover(mut self, value: bool) -> Self {
        self.hover_pause = value;
        self
    }
    pub fn on_ready(mut self, callback: impl FnMut(NotificationsHandle) + 'static) -> Self {
        self.ready = Some(Box::new(callback));
        self
    }
}
impl Widget for Toaster {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.key.is_empty()
            || self.key.len() > 32
            || !(1..=MAX_NOTIFICATIONS).contains(&self.capacity)
            || !(1..=MAX_VISIBLE_NOTIFICATIONS).contains(&self.visible)
            || self.visible > self.capacity
        {
            return Err(invalid(
                "toaster needs a 1..=32 byte key, capacity 1..=64 and visible limit 1..=min(8,capacity)",
            ));
        }
        crate::composite::id("notifications", &self.key, "root", "root")?;
        let queue = scope.cage(Queue::default());
        let reschedule = scope.cage(());
        let delay = OwnerDelay::new(scope)?;
        let controller = Rc::new(Controller {
            owner: scope.id(),
            weak: scope.downgrade(),
            live: Cell::new(true),
            next: Cell::new(0),
            queue,
            reschedule,
            capacity: self.capacity,
            visible: self.visible,
            clock: self.clock.clone(),
            delay,
            scheduled: Cell::new(None),
            last_now: Cell::new(Duration::ZERO),
            hover: Cell::new(None),
            focus: Cell::new(None),
        });
        let handle = NotificationsHandle(controller.clone());
        let cleanup = controller.clone();
        scope.on_cleanup(move || {
            cleanup.live.set(false);
            cleanup.cancel_timer();
        })?;
        let visible = self.visible;
        let items = scope.bond(move || {
            queue.with(|q| q.entries.iter().take(visible).cloned().collect::<Vec<_>>())
        });
        let controller_effect = controller.clone();
        let weak = scope.downgrade();
        scope.effect(move || {
            reschedule.get();
            queue.with(|q| {
                let _ = q.paused;
            });
            if let Err(error) = controller_effect.synchronize()
                && let Some(scope) = weak.upgrade()
            {
                scope.report_error(error)
            }
        });
        let mut root = glory::ui::view()
            .name("Notifications")
            .class("sapid-toaster")
            .semantics(Some(Box::new(glory::render::Semantics {
                id: Some(crate::composite::id(
                    "notifications",
                    &self.key,
                    "root",
                    "root",
                )?),
                ..Default::default()
            })))
            .style(crate::composite::column(12.));
        let hover = self.hover_pause && self.clock.is_some();
        if hover {
            root = root.on(glory::render::EventKind::PointerWithin, |_| {});
        }
        let command = handle.clone();
        let namespace = self.key.clone();
        let theme = self.theme.clone();
        root.fill(glory::ui::each(
            items,
            |entry| entry.id,
            move |item, _| {
                let command = command.clone();
                let namespace = namespace.clone();
                let theme = theme.clone();
                glory::ui::component(move |scope| {
                    let entry = item.get();
                    let token = NotificationToken {
                        controller: Rc::downgrade(&command.0),
                        id: entry.id,
                    };
                    let title = scope.bond(move || item.with(|e| e.title.clone()));
                    let message = scope.bond(move || item.with(|e| e.message.clone()));
                    let dismiss = command.clone();
                    let dismissed = token.clone();
                    let weak = scope.downgrade();
                    let mut row = crate::notification(format!("{namespace}-{}", entry.id))
                        .theme(theme.clone())
                        .title(title)
                        .content(message)
                        .severity(entry.severity)
                        .announcement(entry.announcement)
                        .closable(true)
                        .close_label("Dismiss notification")
                        .on_close(move || {
                            if let Err(error) = dismiss.dismiss(&dismissed)
                                && let Some(scope) = weak.upgrade()
                            {
                                scope.report_error(error)
                            }
                        });
                    if let Some((label, _)) = &entry.action {
                        let action = command.clone();
                        let token = token.clone();
                        let weak = scope.downgrade();
                        row = row.action(label.clone(), move || {
                            if let Err(error) =
                                action.resolve(&token, NotificationCloseReason::Action, true)
                                && let Some(scope) = weak.upgrade()
                            {
                                scope.report_error(error)
                            }
                        });
                    }
                    let cleanup = command.clone();
                    let id = entry.id;
                    scope.on_cleanup(move || {
                        let _ = cleanup.activity(1, id, false);
                        let _ = cleanup.activity(2, id, false);
                    })?;
                    let mut wrapper = glory::ui::view().class("sapid-notification-row");
                    for (kind, bit) in [
                        (glory::render::EventKind::FocusWithin, 2),
                        (glory::render::EventKind::PointerWithin, 1),
                    ] {
                        if bit == 1 && !hover {
                            continue;
                        }
                        let activity = command.clone();
                        let weak = scope.downgrade();
                        wrapper = wrapper.on(kind, move |event| {
                            let active = match event {
                                glory::render::Event::FocusWithin { focused, .. } => focused,
                                glory::render::Event::PointerWithin { phase, .. } => {
                                    phase != glory::render::PointerPhase::Leave
                                }
                                _ => return,
                            };
                            if let Err(error) = activity.activity(bit, id, active)
                                && let Some(scope) = weak.upgrade()
                            {
                                scope.report_error(error)
                            }
                        });
                    }
                    wrapper.fill(row).show_in(scope)
                })
            },
        ))
        .show_in(scope)?;
        if let Some(mut callback) = self.ready.take() {
            callback(handle)
        }
        Ok(())
    }
}
