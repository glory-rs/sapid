use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{Theme, hover_card, tooltip};

    pub struct State {
        pub tip: Cage<bool>,
        pub card: Cage<bool>,
        pub disabled: Cage<bool>,
        pub reject: Cage<bool>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let tip = scope.cage(false);
        let card = scope.cage(false);
        let disabled = scope.cage(false);
        let reject = scope.cage(false);
        let dismissed = scope.cage(0usize);
        let followed = scope.cage(0usize);
        let enabled = scope.bond(move || !disabled.get());
        let widgets = view()
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(18.0),
            )
            .fill((
                tooltip("save-tip")
                    .label("Save draft")
                    .text("Your draft is saved privately until you publish it.")
                    .theme(theme.clone())
                    .open(tip)
                    .enabled(enabled)
                    .open_delay_with(|| super::menu_form::local_delay(80))
                    .close_delay_with(|| super::menu_form::local_delay(120))
                    .on_open_change(move |value| {
                        if !reject.get() {
                            tip.revise(|old| *old = value);
                        }
                    })
                    .on_dismiss(move |_| dismissed.revise(|value| *value += 1)),
                hover_card("person-card")
                    .label("About Ada")
                    .name("Ada profile")
                    .theme(theme.clone())
                    .open(card)
                    .open_delay_with(|| super::menu_form::local_delay(80))
                    .close_delay_with(|| super::menu_form::local_delay(160))
                    .on_open_change(move |value| {
                        if !reject.get() {
                            card.revise(|old| *old = value);
                        }
                    })
                    .content(move || {
                        view()
                            .style(
                                Style::new()
                                    .display(Display::Flex)
                                    .flex_direction(FlexDirection::Column)
                                    .gap(12.0),
                            )
                            .fill((
                                text().content("Ada builds thoughtful developer tools."),
                                link().label("Read Ada profile").target("#ada"),
                                button()
                                    .label("Follow Ada")
                                    .on(activate, move |_| followed.revise(|value| *value += 1)),
                            ))
                    }),
                tooltip("search-tip")
                    .text("Search by title or author.")
                    .theme(theme)
                    .target(|target| target.input(input().name("Search").value("initial query"))),
                text().content(scope.bond(move || format!("Dismissed tips: {}", dismissed.get()))),
                text().content(scope.bond(move || format!("Followed: {}", followed.get()))),
                button().label("Outside hints"),
            ));
        (
            widgets,
            State {
                tip,
                card,
                disabled,
                reject,
            },
        )
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
