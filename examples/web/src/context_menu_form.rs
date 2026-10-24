use glory::prelude::*;
use sapid::{
    Theme,
    menu::{
        context_menu, menu_checkbox, menu_item, menu_radio_group, menu_radio_item, menu_submenu,
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
                text()
                    .content(scope.bond(move || format!("Context actions: {}", actions.get()))),
            )),
        State {
            open,
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
    pub fn mount_context_menu(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> std::result::Result<(), JsValue> {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing context root"))?;
        let theme = Theme::new(
            if hue == "green" {
                sapid::Hue::Green
            } else {
                sapid::Hue::Blue
            },
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
    pub fn context_menu_state(kind: String, value: bool) -> std::result::Result<(), JsValue> {
        APP.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("context unmounted"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "open" => state.open.revise(|old| *old = value),
                "reject" => state.reject.revise(|old| *old = value),
                "disabled" => state.disabled.revise(|old| *old = value),
                _ => return Err(js("unknown context state")),
            };
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_context_menu() {
        APP.with(|slot| drop(slot.borrow_mut().take()));
    }
}
