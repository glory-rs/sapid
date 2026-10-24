use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{Theme, menu_checkbox, menu_item, menu_submenu, menubar, menubar_menu};

    pub struct State {
        pub value: Cage<Option<String>>,
        pub reject: Cage<bool>,
        pub disabled: Cage<bool>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let value = scope.cage(None::<String>);
        let reject = scope.cage(false);
        let disabled = scope.cage(false);
        let checked = scope.cage(false);
        let actions = scope.cage(0usize);
        let bar = menubar("application")
            .name("Application menu")
            .theme(theme.clone())
            .value(value)
            .enabled(scope.bond(move || !disabled.get()))
            .on_value_change(move |next| {
                if !reject.get() {
                    value.revise(|old| *old = next);
                }
            })
            .hover_delay_with(|| super::menu_form::local_delay(100))
            .menus([
                menubar_menu("file")
                    .label("File")
                    .item(
                        menu_item("new")
                            .label("New file")
                            .on_activate(move || actions.revise(|v| *v += 1)),
                    )
                    .item(
                        menu_item("unavailable")
                            .label("Unavailable file")
                            .enabled(false),
                    )
                    .submenu(
                        menu_submenu("export")
                            .label("Export file")
                            .item(menu_item("pdf").label("Export PDF")),
                    ),
                menubar_menu("disabled")
                    .label("Unavailable root")
                    .enabled(false)
                    .item(menu_item("unused").label("Unused")),
                menubar_menu("edit")
                    .label("Edit")
                    .item(menu_item("undo").label("Undo edit"))
                    .item(menu_item("redo").label("Redo edit")),
                menubar_menu("view")
                    .label("View")
                    .item(
                        menu_checkbox("grid")
                            .label("Show grid")
                            .checked(checked)
                            .on_checked_change(move |next| checked.revise(|old| *old = next)),
                    )
                    .item(menu_item("zoom").label("Zoom view")),
            ]);
        let rtl = menubar("rtl-app")
            .name("RTL menu")
            .theme(theme)
            .direction(glory::render::Direction::Rtl)
            .menus([
                menubar_menu("one")
                    .label("RTL one")
                    .item(menu_item("first").label("RTL first item")),
                menubar_menu("two")
                    .label("RTL two")
                    .item(menu_item("second").label("RTL second item")),
            ]);
        (
            view()
                .style(
                    Style::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Column)
                        .gap(18.0),
                )
                .fill((
                    button().label("Before menubar"),
                    bar,
                    button().label("After menubar"),
                    rtl,
                    input().name("Menubar draft").value("Draft text"),
                    text().content(
                        scope.bond(move || format!("Menu actions: {}", actions.get())),
                    ),
                )),
            State {
                value,
                reject,
                disabled,
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
