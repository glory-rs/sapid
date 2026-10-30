use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{CommandHandle, CommandResults, Theme, command, command_action, command_group};

    pub struct State {
        pub open: Cage<bool>,
        pub reject_open: Cage<bool>,
        pub reject_search: Cage<bool>,
        pub disabled: Cage<bool>,
        pub async_handle: Cage<Option<CommandHandle>>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let open = scope.cage(false);
        let reject_open = scope.cage(false);
        let reject_search = scope.cage(false);
        let disabled = scope.cage(false);
        let query = scope.cage(String::new());
        let root = scope.cage(None);
        let shortcut = scope.cage(None::<CommandHandle>);
        let weak = scope.downgrade();
        scope.effect(move || {
            if let (Some(target), Some(handle)) = (root.get(), shortcut.get())
                && let Err(error) = handle.register_shortcut(target)
                && let Some(scope) = weak.upgrade()
            {
                scope.report_error(error)
            }
        });
        let async_handle = scope.cage(None);
        let actions = scope.cage(String::new());
        let palette = command("application-commands")
            .label("Application commands")
            .theme(theme.clone())
            .open(open)
            .search(query)
            .enabled(scope.bond(move || !disabled.get()))
            .on_open_change(move |value| {
                if !reject_open.get() {
                    open.revise(|old| *old = value)
                }
            })
            .on_search_change(move |value| {
                if !reject_search.get() {
                    query.revise(|old| *old = value)
                }
            })
            .on_action(move |key| actions.revise(|old| *old = key))
            .on_ready(move |handle| shortcut.revise(|old| *old = Some(handle)))
            .group(command_group("files", "Files"))
            .actions([
                command_action("new", "New document")
                    .group("files")
                    .keywords("fresh create"),
                command_action("blocked", "Unavailable action")
                    .group("files")
                    .disabled(true),
                command_action("open", "Open document").group("files"),
                command_action("settings", "Settings").description("Application preferences"),
            ]);
        let many =
            command("many-commands")
                .label("Many commands")
                .theme(theme.clone())
                .actions((0..256).map(|index| {
                    command_action(format!("action-{index}"), format!("Action {index}"))
                }))
                .on_action(move |key| actions.revise(|old| *old = key));
        let asynchronous = command("remote-commands")
            .label("Async commands")
            .theme(theme)
            .on_ready(move |handle| async_handle.revise(|old| *old = Some(handle)))
            .load_with(|query| async move {
                super::menu_form::local_delay(250).await;
                if query == "error" {
                    Err("Command search failed".into())
                } else {
                    Ok(CommandResults::new(vec![command_action(
                        "remote",
                        format!("Remote {query}"),
                    )]))
                }
            })
            .on_action(move |key| actions.revise(|old| *old = format!("async-{key}")));
        (
            view()
                .node_ref(root)
                .style(
                    Style::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Column)
                        .gap(16.0),
                )
                .fill((
                    button().label("Before command"),
                    palette,
                    input().name("Native shortcut editor").value("Keep draft"),
                    asynchronous,
                    many,
                    text().content(scope.bond(move || format!("Executed: {}", actions.get()))),
                    button().label("After command"),
                )),
            State {
                open,
                reject_open,
                reject_search,
                disabled,
                async_handle,
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
