use glory::prelude::*;
use glory::render::{Event, NodeId};
use sapid::{
    NotificationClock, NotificationCloseReason as Reason, NotificationsHandle,
    notification_request, toaster,
};
use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
    time::Duration,
};
#[derive(Clone, Default)]
struct Time {
    now: Rc<Cell<Duration>>,
    waker: Rc<RefCell<Option<Waker>>>,
    created: Rc<Cell<usize>>,
    dropped: Rc<Cell<usize>>,
    early: Rc<Cell<bool>>,
}
struct Sleep {
    time: Time,
    deadline: Duration,
}
impl Future for Sleep {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.time.early.replace(false) || self.time.now.get() >= self.deadline {
            Poll::Ready(())
        } else {
            *self.time.waker.borrow_mut() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
impl Drop for Sleep {
    fn drop(&mut self) {
        self.time.dropped.set(self.time.dropped.get() + 1)
    }
}
impl Time {
    fn clock(&self) -> NotificationClock {
        let now = self.clone();
        let sleep = self.clone();
        NotificationClock::new(
            move || now.now.get(),
            move |delay| {
                sleep.created.set(sleep.created.get() + 1);
                Sleep {
                    time: sleep.clone(),
                    deadline: sleep.now.get() + delay,
                }
            },
        )
    }
    fn set(&self, millis: u64) {
        self.now.set(Duration::from_millis(millis));
        if let Some(waker) = self.waker.borrow_mut().take() {
            waker.wake()
        }
    }
}
fn fixture(
    visible: usize,
) -> (
    App,
    Rc<glory::core::executor::Executor>,
    Time,
    NotificationsHandle,
) {
    let executor = Rc::new(glory::core::executor::Executor::new(1));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let time = Time::default();
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    app.mount(
        toaster("tests")
            .clock(time.clock())
            .pause_on_hover(false)
            .visible_limit(visible)
            .on_ready(move |h| *saved.borrow_mut() = Some(h)),
    )
    .unwrap();
    let handle = out.borrow_mut().take().unwrap();
    (app, executor, time, handle)
}
fn tick(app: &App, executor: &glory::core::executor::Executor, time: &Time, millis: u64) {
    time.set(millis);
    executor.poll_one();
    app.flush().unwrap();
    assert!(executor.len() <= 1)
}
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = std::collections::VecDeque::from(tree.roots().to_vec());
    while let Some(id) = pending.pop_front() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            return id;
        }
        pending.extend(&node.children)
    }
    panic!("missing {class}")
}
fn button(app: &App, label: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = std::collections::VecDeque::from(tree.roots().to_vec());
    while let Some(id) = pending.pop_front() {
        let node = tree.node(id).unwrap();
        if matches!(node.descriptor.view().props,glory::render::Props::Button(p)if p.label==label) {
            return id;
        }
        pending.extend(&node.children)
    }
    panic!("missing {label}")
}
#[test]
fn one_timer_covers_visible_deadlines_and_queued_lifetimes_start_on_promotion() {
    let (app, executor, time, handle) = fixture(1);
    let log = Rc::new(RefCell::new(Vec::new()));
    for key in ["first", "second"] {
        let saved = log.clone();
        handle
            .show(
                notification_request(key, key, "Details")
                    .timeout(Duration::from_millis(100))
                    .on_close(move |r| saved.borrow_mut().push(r)),
            )
            .unwrap();
    }
    app.flush().unwrap();
    assert_eq!(executor.len(), 1);
    assert!(time.waker.borrow().is_none(), "spawn synchronously polled");
    executor.poll_one();
    tick(&app, &executor, &time, 100);
    assert_eq!(handle.pending().unwrap(), 1);
    assert_eq!(time.created.get(), 2);
    executor.poll_one();
    tick(&app, &executor, &time, 199);
    assert_eq!(handle.pending().unwrap(), 1);
    tick(&app, &executor, &time, 200);
    assert_eq!(handle.pending().unwrap(), 0);
    assert!(executor.is_empty());
    assert_eq!(*log.borrow(), vec![Reason::Expired, Reason::Expired]);
}
#[test]
fn pause_retains_remaining_duration_and_early_provider_completion_cannot_expire() {
    let (app, executor, time, handle) = fixture(2);
    handle
        .show(notification_request("a", "Title", "Details").timeout(Duration::from_millis(100)))
        .unwrap();
    app.flush().unwrap();
    time.early.set(true);
    executor.poll_one();
    app.flush().unwrap();
    assert_eq!(handle.pending().unwrap(), 1);
    assert_eq!(executor.len(), 1);
    executor.poll_one();
    tick(&app, &executor, &time, 40);
    handle.pause(true).unwrap();
    app.flush().unwrap();
    assert!(executor.is_empty());
    time.set(200);
    handle.pause(false).unwrap();
    app.flush().unwrap();
    executor.poll_one();
    tick(&app, &executor, &time, 259);
    assert_eq!(handle.pending().unwrap(), 1);
    tick(&app, &executor, &time, 260);
    assert_eq!(handle.pending().unwrap(), 0);
}
#[test]
fn removing_a_focused_row_releases_its_pause_without_a_retired_focus_event() {
    let (app, executor, time, handle) = fixture(2);
    let first = handle
        .show(notification_request("persistent", "First", "Details"))
        .unwrap();
    handle
        .show(
            notification_request("timed", "Second", "Details").timeout(Duration::from_millis(100)),
        )
        .unwrap();
    app.flush().unwrap();
    executor.poll_one();
    time.set(10);
    let row = find(&app, "sapid-notification-row");
    let close = button(&app, "Dismiss notification");
    app.dispatch(
        row,
        Event::FocusWithin {
            node: close,
            focused: true,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert!(executor.is_empty());
    handle.dismiss(&first).unwrap();
    app.flush().unwrap();
    assert_eq!(executor.len(), 1);
    executor.poll_one();
    tick(&app, &executor, &time, 100);
    assert_eq!(handle.pending().unwrap(), 0);
}
#[test]
fn replacement_revokes_old_token_preserves_fifo_and_results_can_reenter() {
    let (app, _executor, _time, handle) = fixture(1);
    let log = Rc::new(RefCell::new(Vec::new()));
    let saved = log.clone();
    let command = handle.clone();
    let old = handle
        .show(
            notification_request("key", "Old", "Details").on_close(move |reason| {
                saved.borrow_mut().push(reason);
                assert_eq!(command.pending().unwrap(), 1);
            }),
        )
        .unwrap();
    app.flush().unwrap();
    let token = handle
        .show(notification_request("key", "Replacement", "New details"))
        .unwrap();
    assert!(!handle.dismiss(&old).unwrap());
    assert_eq!(*log.borrow(), vec![Reason::Replaced]);
    assert_eq!(handle.pending().unwrap(), 1);
    app.flush().unwrap();
    let root = find(&app, "sapid-notification-row");
    handle
        .update(&token, "Updated", "Accepted message")
        .unwrap();
    app.flush().unwrap();
    assert_eq!(find(&app, "sapid-notification-row"), root);
    assert!(handle.dismiss(&token).unwrap());
    app.flush().unwrap();
    assert_eq!(handle.pending().unwrap(), 0);
}
#[test]
fn action_removes_before_callbacks_and_does_not_repeat_on_late_activation() {
    let (app, _executor, _time, handle) = fixture(1);
    let log = Rc::new(RefCell::new(Vec::new()));
    let action_log = log.clone();
    let close_log = log.clone();
    let command = handle.clone();
    handle
        .show(
            notification_request("action", "Message", "Details")
                .action("Undo", move || {
                    assert_eq!(command.pending().unwrap(), 0);
                    action_log.borrow_mut().push("action");
                })
                .on_close(move |reason| {
                    assert_eq!(reason, Reason::Action);
                    close_log.borrow_mut().push("close");
                }),
        )
        .unwrap();
    app.flush().unwrap();
    app.dispatch(button(&app, "Undo"), Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(*log.borrow(), vec!["action", "close"]);
    assert_eq!(handle.pending().unwrap(), 0);
}
#[test]
fn admission_budget_foreign_tokens_invalid_data_and_clock_requirements_are_atomic() {
    let (app, _executor, _time, handle) = fixture(1);
    let (_other, _, _, foreign) = fixture(1);
    for request in [
        notification_request("", "Title", "Message"),
        notification_request("a", "x".repeat(257), "Message"),
        notification_request("a", "Title", "x".repeat(4097)),
        notification_request("a", "Title", "Message").timeout(Duration::ZERO),
    ] {
        assert!(handle.show(request).is_err());
        assert_eq!(handle.pending().unwrap(), 0)
    }
    let token = handle
        .show(notification_request("a", "Title", "Message"))
        .unwrap();
    assert!(foreign.dismiss(&token).is_err());
    assert!(handle.update(&token, "", "Message").is_err());
    assert_eq!(handle.pending().unwrap(), 1);
    app.flush().unwrap();
    let standalone = App::memory();
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    standalone
        .mount(toaster("no-clock").on_ready(move |h| *saved.borrow_mut() = Some(h)))
        .unwrap();
    assert!(
        out.borrow()
            .as_ref()
            .unwrap()
            .show(notification_request("timed", "Title", "Message").timeout(Duration::from_secs(1)))
            .is_err()
    );
}
#[test]
fn full_queue_rejects_new_keys_but_replacement_keeps_its_fifo_slot() {
    let (app, _, _, handle) = fixture(1);
    let mut first = None;
    for index in 0..32 {
        let token = handle
            .show(notification_request(
                format!("entry-{index}"),
                format!("Title {index}"),
                "Message",
            ))
            .unwrap();
        if index == 0 {
            first = Some(token);
        }
    }
    assert!(
        handle
            .show(notification_request("overflow", "Overflow", "Message"))
            .is_err()
    );
    assert_eq!(handle.pending().unwrap(), 32);
    let replacement = handle
        .show(notification_request(
            "entry-0",
            "Replacement first",
            "Message",
        ))
        .unwrap();
    assert!(!handle.dismiss(&first.unwrap()).unwrap());
    assert_eq!(handle.pending().unwrap(), 32);
    app.flush().unwrap();
    assert!(
        glory::ssr::render(
            sapid::notification("standalone")
                .title("Independent")
                .content("Message")
                .action("Keep open", || {})
        )
        .unwrap()
        .contains("Keep open")
    );
    assert!(handle.dismiss(&replacement).unwrap());
    app.flush().unwrap();
    assert_eq!(handle.pending().unwrap(), 31);
    let tree = app.tree();
    assert!(
        matches!(tree.node(find(&app,"sapid-alert-title")).unwrap().descriptor.view().props,glory::render::Props::Text(p) if p.content=="Title 1")
    );
}
#[test]
fn owner_cleanup_cancels_timer_and_drops_pending_callbacks_without_notification() {
    let executor = Rc::new(glory::core::executor::Executor::new(1));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let time = Time::default();
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    let count = Rc::new(Cell::new(0));
    let callbacks = count.clone();
    let mounted = app
        .scope()
        .mount_with(|| {
            toaster("retire")
                .clock(time.clock())
                .pause_on_hover(false)
                .on_ready(move |h| *saved.borrow_mut() = Some(h))
        })
        .unwrap();
    let handle = out.borrow_mut().take().unwrap();
    let token = handle
        .show(
            notification_request("retire", "Title", "Message")
                .timeout(Duration::from_secs(1))
                .on_close(move |_| callbacks.set(callbacks.get() + 1)),
        )
        .unwrap();
    app.flush().unwrap();
    executor.poll_one();
    let late = time.waker.borrow().clone().unwrap();
    drop(mounted);
    app.flush().unwrap();
    assert!(executor.is_empty());
    assert!(app.tree().is_empty());
    assert_eq!(count.get(), 0);
    assert!(handle.dismiss(&token).is_err());
    late.wake();
    executor.poll_one();
    assert_eq!(count.get(), 0);
}
#[test]
fn repeated_show_dismiss_has_no_retained_rows_or_timer_history() {
    let (app, executor, _time, handle) = fixture(1);
    let base = app.tree().len();
    for _ in 0..64 {
        let token = handle
            .show(notification_request("bounded", "Title", "Message"))
            .unwrap();
        app.flush().unwrap();
        handle.dismiss(&token).unwrap();
        app.flush().unwrap();
        assert_eq!(app.tree().len(), base)
    }
    assert_eq!(handle.pending().unwrap(), 0);
    assert!(executor.is_empty());
    assert_eq!(app.stats().queries, 0);
}
#[test]
fn cancellation_destructor_can_flush_row_retirement_without_restoring_its_focus_pause() {
    type CleanupSlot = RefCell<Option<Box<dyn FnOnce()>>>;
    type Cleanup = Rc<CleanupSlot>;
    struct ReentrantSleep {
        inner: Sleep,
        cleanup: std::rc::Weak<CleanupSlot>,
    }
    impl Future for ReentrantSleep {
        type Output = ();
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            Pin::new(&mut self.inner).poll(cx)
        }
    }
    impl Drop for ReentrantSleep {
        fn drop(&mut self) {
            if let Some(cleanup) = self.cleanup.upgrade() {
                let callback = cleanup.borrow_mut().take();
                if let Some(callback) = callback {
                    callback()
                }
            }
        }
    }
    let executor = Rc::new(glory::core::executor::Executor::new(1));
    let app = Rc::new(App::with_host(
        executor.clone(),
        glory::render::memory::Memory::new,
    ));
    let time = Time::default();
    let cleanup: Cleanup = Rc::new(RefCell::new(None));
    let weak = Rc::downgrade(&cleanup);
    let now = time.clone();
    let sleep = time.clone();
    let clock = NotificationClock::new(
        move || now.now.get(),
        move |delay| ReentrantSleep {
            inner: Sleep {
                time: sleep.clone(),
                deadline: sleep.now.get() + delay,
            },
            cleanup: weak.clone(),
        },
    );
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    app.mount(
        toaster("reentrant-drop")
            .clock(clock)
            .pause_on_hover(false)
            .on_ready(move |h| *saved.borrow_mut() = Some(h)),
    )
    .unwrap();
    let handle = out.borrow_mut().take().unwrap();
    handle
        .show(notification_request("old", "Old", "Details").timeout(Duration::from_millis(100)))
        .unwrap();
    app.flush().unwrap();
    executor.poll_one();
    let row = find(&app, "sapid-notification-row");
    let close = button(&app, "Dismiss notification");
    let command = handle.clone();
    let callback_app = app.clone();
    *cleanup.borrow_mut() = Some(Box::new(move || {
        command.clear().unwrap();
        callback_app.flush().unwrap();
        command
            .show(notification_request("new", "New", "Details").timeout(Duration::from_millis(100)))
            .unwrap();
        callback_app.flush().unwrap();
    }));
    app.dispatch(
        row,
        Event::FocusWithin {
            node: close,
            focused: true,
        },
    )
    .unwrap();
    assert_eq!(handle.pending().unwrap(), 1);
    assert_eq!(executor.len(), 1);
    executor.poll_one();
    tick(&app, &executor, &time, 100);
    assert_eq!(handle.pending().unwrap(), 0);
}
