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
                text().content(scope.bond(move || format!("Menu actions: {}", actions.get()))),
            )),
        State {
            value,
            reject,
            disabled,
        },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static APP:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_menubar(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> std::result::Result<(), JsValue> {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing menubar root"))?;
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
            let (widget, value) = form(scope, theme);
            state = Some(value);
            widget
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        APP.with(|slot| *slot.borrow_mut() = Some((app, state.unwrap())));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn menubar_state(kind: String, value: bool) -> std::result::Result<(), JsValue> {
        APP.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("menubar unmounted"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "reject" => state.reject.revise(|old| *old = value),
                "disabled" => state.disabled.revise(|old| *old = value),
                "view" => state
                    .value
                    .revise(|old| *old = value.then(|| "view".into())),
                _ => return Err(js("unknown menubar state")),
            };
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_menubar() {
        APP.with(|slot| drop(slot.borrow_mut().take()));
    }
}
