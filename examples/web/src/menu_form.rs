use glory::prelude::*;
use sapid::{
    Theme, dropdown_menu, menu_checkbox, menu_item, menu_radio_group, menu_radio_item,
    menu_submenu,
};
pub struct State {
    pub open: Cage<bool>,
    pub checked: Cage<bool>,
    pub radio: Cage<String>,
    pub reject: Cage<bool>,
    pub disabled: Cage<bool>,
    pub share: Cage<bool>,
    pub export: Cage<bool>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let open = scope.cage(false);
    let checked = scope.cage(false);
    let radio = scope.cage("comfortable".to_string());
    let reject = scope.cage(false);
    let disabled = scope.cage(false);
    let share = scope.cage(false);
    let export = scope.cage(false);
    let actions = scope.cage(0usize);
    let enabled = scope.bond(move || !disabled.get());
    let tools = dropdown_menu("tools")
        .label("Tools")
        .theme(theme.clone())
        .open(open)
        .enabled(enabled)
        .on_open_change(move |value| {
            if !reject.get() {
                open.revise(|old| *old = value);
            }
        })
        .hover_delay_with(hover_delay)
        .item(
            menu_item("new")
                .label("New document")
                .on_activate(move || actions.revise(|count| *count += 1)),
        )
        .item(
            menu_item("unavailable")
                .label("Unavailable action")
                .enabled(false),
        )
        .item(
            menu_checkbox("grid")
                .label("Show grid")
                .checked(checked)
                .on_checked_change(move |value| {
                    if !reject.get() {
                        checked.revise(|old| *old = value);
                    }
                }),
        )
        .radio_group(
            menu_radio_group("density")
                .label("Density")
                .value(radio)
                .on_value_change(move |value| {
                    if !reject.get() {
                        radio.revise(|old| *old = value);
                    }
                })
                .items([
                    menu_radio_item("comfortable", "Comfortable"),
                    menu_radio_item("compact", "Compact"),
                ]),
        )
        .separator("break")
        .submenu(
            menu_submenu("share")
                .label("Share")
                .open(share)
                .on_open_change(move |value| {
                    if !reject.get() {
                        share.revise(|old| *old = value);
                    }
                })
                .item(
                    menu_item("email")
                        .label("Email link")
                        .on_activate(move || actions.revise(|count| *count += 1)),
                )
                .submenu(
                    menu_submenu("more").label("More sharing").item(
                        menu_item("copy")
                            .label("Copy link")
                            .on_activate(move || actions.revise(|count| *count += 1)),
                    ),
                ),
        )
        .submenu(
            menu_submenu("export")
                .label("Export")
                .open(export)
                .on_open_change(move |value| {
                    if !reject.get() {
                        export.revise(|old| *old = value);
                    }
                })
                .items([menu_item("pdf").label("PDF"), menu_item("png").label("PNG")]),
        );
    let widgets = view()
        .style(
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(16.0),
        )
        .fill((
            text().content(scope.bond(move || format!("Actions performed: {}", actions.get()))),
            view().field_part(glory::render::FieldPart::Form).fill((
                button().label("Before tools"),
                tools,
                button().label("After tools"),
            )),
            button().label("Outside tools"),
            dropdown_menu("rtl-tools")
                .label("RTL tools")
                .theme(theme)
                .direction(glory::render::Direction::Rtl)
                .item(menu_item("start").label("RTL start"))
                .submenu(
                    menu_submenu("more")
                        .label("RTL more")
                        .item(menu_item("child").label("RTL child")),
                ),
        ));
    (
        widgets,
        State {
            open,
            checked,
            radio,
            reject,
            disabled,
            share,
            export,
        },
    )
}
fn hover_delay() -> impl std::future::Future<Output = ()> {
    local_delay(120)
}
#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn local_delay(_millis: i32) {}
#[cfg(target_arch = "wasm32")]
pub(crate) fn local_delay(millis: i32) -> impl std::future::Future<Output = ()> {
    browser::Delay::new(millis)
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
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
    thread_local! { static MENU:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)}; }
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_menu(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("app"))
            .ok_or_else(|| js("missing menu root"))?;
        let theme = crate::themes::select(
            if hue == "green" { "green" } else { "blue" },
            if scheme == "dark" {
                sapid::ColorScheme::Dark
            } else {
                sapid::ColorScheme::Light
            },
        );
        let mut state = None;
        let build = |scope: &Scope| {
            let (widget, source) = form(scope, theme);
            state = Some(source);
            widget
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        MENU.with(|slot| *slot.borrow_mut() = Some((app, state.unwrap())));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn menu_state(kind: String, value: bool) -> Result {
        MENU.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("menu unmounted"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "open" => state.open.revise(|old| *old = value),
                "checked" => state.checked.revise(|old| *old = value),
                "radio" => state
                    .radio
                    .revise(|old| *old = if value { "compact" } else { "comfortable" }.into()),
                "reject" => state.reject.revise(|old| *old = value),
                "disabled" => state.disabled.revise(|old| *old = value),
                "share" => state.share.revise(|old| *old = value),
                "export" => state.export.revise(|old| *old = value),
                _ => return Err(js("unknown menu state")),
            };
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_menu() {
        MENU.with(|slot| drop(slot.borrow_mut().take()));
    }
}
