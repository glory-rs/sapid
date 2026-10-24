use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{
        Theme,
        menu::{
            context_menu, menu_checkbox, menu_item, menu_radio_group, menu_radio_item,
            menu_submenu,
        },
    };
    pub struct State {
        pub open: Cage<bool>,
        pub reject: Cage<bool>,
        pub disabled: Cage<bool>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let open = scope.cage(false);
        let reject = scope.cage(false);
        let disabled = scope.cage(false);
        let actions = scope.cage(0usize);
        let checks = scope.cage(false);
        let radio = scope.cage("one".to_string());
        let enabled = scope.bond(move || !disabled.get());
        let inner = context_menu("nested-context")
            .target_name("Nested area")
            .label("Nested actions")
            .theme(theme.clone())
            .target(button().label("Nested document"))
            .item(menu_item("inner").label("Nested action"));
        let main = context_menu("document-context")
            .target_name("Document area")
            .label("Document actions")
            .theme(theme.clone())
            .open(open)
            .enabled(enabled)
            .on_open_change(move |value| {
                if !reject.get() {
                    open.revise(|old| *old = value)
                }
            })
            .hover_delay_with(|| super::menu_form::local_delay(100))
            .target(
                view()
                    .style(
                        Style::new()
                            .padding(Edges::all(18.0))
                            .min_height(Length::Px(150.0))
                            .border_width(Edges::all(1.0)),
                    )
                    .fill((
                        text().content("Right-click this document area."),
                        button().label("Document button"),
                        input().name("Native context editor").value("Native draft"),
                        inner,
                    )),
            )
            .item(
                menu_item("copy")
                    .label("Copy document")
                    .on_activate(move || actions.revise(|v| *v += 1)),
            )
            .item(menu_item("disabled").label("Unavailable").enabled(false))
            .item(
                menu_checkbox("grid")
                    .label("Show grid")
                    .checked(checks)
                    .on_checked_change(move |v| checks.revise(|old| *old = v)),
            )
            .radio_group(
                menu_radio_group("mode")
                    .label("Mode")
                    .value(radio)
                    .on_value_change(move |v| radio.revise(|old| *old = v))
                    .items([
                        menu_radio_item("one", "Mode one"),
                        menu_radio_item("two", "Mode two"),
                    ]),
            )
            .submenu(
                menu_submenu("export").label("Export document").item(
                    menu_item("pdf")
                        .label("Export PDF")
                        .on_activate(move || actions.revise(|v| *v += 1)),
                ),
            );
        let override_editor = context_menu("editor-context")
            .target_name("Custom editor area")
            .label("Editor actions")
            .editor_policy(glory::render::ContextEditPolicy::Override)
            .theme(theme)
            .target(input().name("Custom context editor").value("Custom draft"))
            .item(menu_item("format").label("Format text"));
        (
            view()
                .style(
                    Style::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Column)
                        .gap(16.0),
                )
                .fill((
                    button().label("Before context"),
                    main,
                    button().label("After context"),
                    override_editor,
                    text().content(
                        scope.bond(move || format!("Context actions: {}", actions.get())),
                    ),
                )),
            State {
                open,
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
