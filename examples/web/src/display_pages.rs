#[cfg(target_arch = "wasm32")]
mod browser {
    use glory::web::WebApp;
    use glory::{Scope, Widget};
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! { static APP:RefCell<Option<WebApp>>=const {RefCell::new(None)}; }
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    fn mount(hydrate: bool, family: u8, hue: String, scheme: String) -> Result<(), JsValue> {
        let root = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("app"))
            .ok_or_else(|| js("missing root"))?;
        let theme = sapid::Theme::new(
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
        let build = move |_: &Scope| -> Box<dyn Widget> {
            if family == 1 {
                Box::new(crate::actions_form::actions_form(theme.clone()))
            } else if family == 2 {
                Box::new(crate::document_form::document_form(theme.clone()))
            } else if family == 3 {
                Box::new(crate::layout_form::layout_form(theme.clone()))
            } else if family == 4 {
                Box::new(crate::timeline_form::timeline_form(theme.clone()))
            } else if family == 5 {
                Box::new(crate::spoiler_form::spoiler_form(theme.clone()))
            } else if family == 6 {
                Box::new(crate::grid_form::grid_form(theme.clone()))
            } else if family == 7 {
                Box::new(crate::visually_hidden_form::visually_hidden_form(
                    theme.clone(),
                ))
            } else if family == 8 {
                Box::new(crate::avatar_form::avatar_form(theme.clone()))
            } else if family == 9 {
                Box::new(crate::image_form::image_form(theme.clone()))
            } else if family == 10 {
                Box::new(crate::button_form::button_form(theme.clone()))
            } else if family == 11 {
                Box::new(crate::alerts_form::alerts_form(theme.clone()))
            } else if family == 12 {
                Box::new(crate::graphics_form::graphics_form(theme.clone()))
            } else {
                Box::new(crate::display_form::display_form(theme.clone()))
            }
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        APP.with(|slot| *slot.borrow_mut() = Some(app));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn mount_display(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 0, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_actions(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 1, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_document(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 2, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_layout(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 3, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_timeline(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 4, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_spoiler(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 5, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_grid(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 6, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_visually_hidden(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 7, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_avatar(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 8, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_image(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 9, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_buttons(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 10, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_alerts(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 11, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_graphics(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 12, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn unmount_display() {
        let old = APP.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
