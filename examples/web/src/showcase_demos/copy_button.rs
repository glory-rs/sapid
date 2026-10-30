use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::copy_button_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    pub const COPY_SOURCE: &str = "Copy exact 😀 <script>\nsecond line\n";
    pub fn copy_button_form(theme: sapid::Theme) -> impl Widget {
        widget(move |scope| {
            let source = scope.cage(COPY_SOURCE.to_string());
            let state = scope.cage(sapid::CopyState::Idle);
            let enabled = scope.cage(true);
            let visible = scope.cage(true);
            let log = scope.cage(String::new());
            let calls = scope.cage(0usize);
            let inner = theme.clone();
            sapid::stack()
                .theme(theme.clone())
                .gap(12.)
                .fill((
                    glory::ui::when(visible, move || {
                        sapid::copy_button()
                            .theme(inner.clone())
                            .class("copy-demo")
                            .source(source)
                            .enabled(enabled)
                            .state_ref(state)
                            .feedback_delay(|| Box::pin(super::menu_form::local_delay(350)))
                            .on_result(move |outcome| {
                                calls.revise(|v| *v += 1);
                                log.revise(|v| *v = format!("typed:{outcome:?}:{:?}", state.get()));
                            })
                            .on(glory::render::EventKind::Clipboard, move |_| {
                                log.revise(|v| v.push_str(&format!(";generic:{:?}", state.get())))
                            })
                    }),
                    sapid::copy_button()
                        .theme(theme.clone())
                        .class("copy-persistent")
                        .label("Copy persistent feedback")
                        .source("Persistent source"),
                    sapid::copy_button()
                        .unstyled()
                        .class("copy-unstyled")
                        .label("Copy without theme")
                        .source("Unstyled source"),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Change copy source")
                        .on_press(move || source.revise(|v| *v = "Changed 😀\n".into())),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Reset copy source")
                        .on_press(move || source.revise(|v| *v = COPY_SOURCE.into())),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Toggle copy enabled")
                        .on_press(move || enabled.revise(|v| *v = !*v)),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Toggle copy owner")
                        .on_press(move || visible.revise(|v| *v = !*v)),
                    glory::ui::text()
                        .class("copy-state")
                        .content(scope.bond(move || format!("{:?}", state.get()))),
                    glory::ui::text().class("copy-log").content(log),
                    glory::ui::text()
                        .class("copy-calls")
                        .content(scope.bond(move || format!("Calls: {}", calls.get()))),
                    sapid::text_field()
                        .theme(theme.clone())
                        .name("Clipboard neighbor draft")
                        .default_value("Retained draft"),
                ))
                .show_in(scope)
        })
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
