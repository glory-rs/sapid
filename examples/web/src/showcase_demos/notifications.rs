use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::notifications_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{
        NotificationClock, NotificationToken, NotificationsHandle, Theme, notification_request,
    };
    use std::{cell::RefCell, rc::Rc, time::Duration};
    #[derive(Default)]
    struct Controls {
        handle: Option<NotificationsHandle>,
        last: Option<NotificationToken>,
    }
    #[cfg(target_arch = "wasm32")]
    fn clock() -> NotificationClock {
        let performance = web_sys::window().unwrap().performance().unwrap();
        NotificationClock::new(
            move || Duration::from_secs_f64(performance.now() / 1000.),
            move |duration| {
                super::menu_form::local_delay(
                    duration.as_millis().min(i32::MAX as u128).max(1) as i32
                )
            },
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn clock() -> NotificationClock {
        NotificationClock::new(|| Duration::ZERO, |_| std::future::pending())
    }
    pub fn notifications_form(theme: Theme) -> impl Widget {
        widget(move |scope| {
            let control = Rc::new(RefCell::new(Controls::default()));
            let ready = control.clone();
            let log = scope.cage("None".to_string());
            let count = scope.cage(0usize);
            let actions = scope.cage(0usize);
            let mut controls = Vec::new();
            for (label, operation) in [
                ("Show persistent notification", 0),
                ("Show timed notification", 1),
                ("Replace saved notification", 2),
                ("Update saved notification", 3),
                ("Clear notifications", 4),
                ("Queue three timed notifications", 5),
                ("Pause notifications", 6),
                ("Resume notifications", 7),
            ] {
                let control = control.clone();
                let weak = scope.downgrade();
                controls.push(sapid::button().theme(theme.clone()).label(label).on_press(
                    move || {
                        if let Err(error) = command(&control, operation, log, count, actions)
                            && let Some(scope) = weak.upgrade()
                        {
                            scope.report_error(error)
                        }
                    },
                ));
            }
            sapid::stack()
                .theme(theme.clone())
                .gap(16.)
                .fill((
                    sapid::group().theme(theme.clone()).fill(controls),
                    sapid::text_field()
                        .theme(theme.clone())
                        .name("Outside notification editor")
                        .default_value("Retained outside draft"),
                    glory::ui::text()
                        .class("notifications-pending")
                        .content(scope.bond(move || format!("Pending: {}", count.get()))),
                    glory::ui::text().class("notifications-result").content(log),
                    glory::ui::text()
                        .class("notifications-actions")
                        .content(scope.bond(move || format!("Actions: {}", actions.get()))),
                    sapid::affix("notifications-fixture")
                        .theme(theme.clone())
                        .style(Style::new().width(Length::Px(320.)))
                        .fill(
                            sapid::toaster("demo")
                                .theme(theme.clone())
                                .capacity(8)
                                .visible_limit(2)
                                .clock(clock())
                                .pause_on_hover(true)
                                .on_ready(move |handle| {
                                    ready.borrow_mut().handle = Some(handle)
                                }),
                        ),
                ))
                .show_in(scope)
        })
    }
    fn command(
        control: &Rc<RefCell<Controls>>,
        operation: u8,
        log: Cage<String>,
        count: Cage<usize>,
        actions: Cage<usize>,
    ) -> glory::ui::Result {
        let Some(handle) = control.borrow().handle.clone() else {
            return Ok(());
        };
        match operation {
            0 | 1 | 2 | 5 => {
                let total = if operation == 5 { 3 } else { 1 };
                for index in 0..total {
                    let observed = handle.clone();
                    let close = move |reason| {
                        log.revise(|value| *value = format!("{reason:?}"));
                        if let Ok(pending) = observed.pending() {
                            count.revise(|value| *value = pending)
                        }
                    };
                    let (key, title) = match operation {
                        0 => ("saved".to_string(), "Saved notification".to_string()),
                        1 => ("timed".into(), "Timed notification".into()),
                        2 => ("saved".into(), "Replacement notification".into()),
                        _ => (
                            format!("queued-{index}"),
                            format!("Queued notification {}", index + 1),
                        ),
                    };
                    let mut request =
                        notification_request(key, title, "Changes are stored on this device.")
                            .on_close(close);
                    if operation == 1 || operation == 5 {
                        request = request.timeout(Duration::from_millis(900))
                    } else {
                        request = request
                            .action("Undo save", move || actions.revise(|value| *value += 1))
                    }
                    let token = handle.show(request)?;
                    if operation == 0 || operation == 2 {
                        control.borrow_mut().last = Some(token)
                    }
                }
            }
            3 => {
                let token = control.borrow().last.clone();
                if let Some(token) = token {
                    handle.update(
                        &token,
                        "Updated notification",
                        "Accepted message changed without replacing the row.",
                    )?;
                }
            }
            4 => {
                handle.clear()?;
            }
            6 => handle.pause(true)?,
            _ => handle.pause(false)?,
        }
        count.revise(|value| *value = handle.pending().unwrap_or(0));
        Ok(())
    }
}
#[allow(dead_code)]
mod menu_form {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn local_delay(_millis: i32) {}
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn local_delay(millis: i32) -> impl std::future::Future<Output = ()> {
        browser::Delay::new(millis)
    }
    #[cfg(target_arch = "wasm32")]
    mod browser {
        use std::cell::{Cell, RefCell};
        use std::future::Future;
        use std::pin::Pin;
        use std::rc::Rc;
        use std::task::{Context, Poll, Waker};
        use wasm_bindgen::JsCast;
        use wasm_bindgen::prelude::*;
        struct TimerState {
            done: Cell<bool>,
            waker: RefCell<Option<Waker>>,
        }
        pub(super) struct Delay {
            millis: i32,
            state: Rc<TimerState>,
            timer: Option<i32>,
            callback: Option<Closure<dyn FnMut()>>,
        }
        impl Delay {
            pub fn new(millis: i32) -> Self {
                Self {
                    millis,
                    state: Rc::new(TimerState {
                        done: Cell::new(false),
                        waker: RefCell::new(None),
                    }),
                    timer: None,
                    callback: None,
                }
            }
        }
        impl Future for Delay {
            type Output = ();
            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                if self.state.done.get() {
                    return Poll::Ready(());
                }
                let previous = self.state.waker.borrow_mut().replace(cx.waker().clone());
                drop(previous);
                if self.timer.is_none() {
                    let state = self.state.clone();
                    let callback = Closure::new(move || {
                        state.done.set(true);
                        let waker = state.waker.borrow_mut().take();
                        if let Some(waker) = waker {
                            waker.wake();
                        }
                    });
                    self.timer = Some(
                        web_sys::window()
                            .unwrap()
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                callback.as_ref().unchecked_ref(),
                                self.millis,
                            )
                            .unwrap(),
                    );
                    self.callback = Some(callback);
                }
                Poll::Pending
            }
        }
        impl Drop for Delay {
            fn drop(&mut self) {
                if let Some(timer) = self.timer.take()
                    && let Some(window) = web_sys::window()
                {
                    window.clear_timeout_with_handle(timer);
                }
            }
        }
    }
}
