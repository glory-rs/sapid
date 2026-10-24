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
            } else if family == 13 {
                Box::new(crate::progress_form::progress_form(theme.clone()))
            } else if family == 14 {
                Box::new(crate::navigation_progress_form::navigation_progress_form(
                    theme.clone(),
                ))
            } else if family == 15 {
                Box::new(crate::spinner_form::spinner_form(theme.clone()))
            } else if family == 16 {
                Box::new(crate::modals_form::modals_form(theme.clone()))
            } else if family == 17 {
                Box::new(crate::loading_overlay_form::loading_overlay_form(
                    theme.clone(),
                ))
            } else if family == 19 {
                Box::new(crate::sheet_form::sheet_form(theme.clone()))
            } else if family == 23 {
                Box::new(crate::accordion_form::accordion_form(theme.clone()))
            } else if family == 24 {
                Box::new(crate::tabs_dynamic_form::tabs_dynamic_form(theme.clone()))
            } else if family == 25 {
                Box::new(crate::collapse_motion_form::collapse_motion_form(
                    theme.clone(),
                ))
            } else if family == 22 {
                Box::new(crate::color_swatch_form::color_swatch_form(theme.clone()))
            } else if family == 21 {
                Box::new(crate::toolbar_form::toolbar_form(theme.clone()))
            } else if family == 37 {
                Box::new(crate::data_table_server_form::data_table_server_form(
                    theme.clone(),
                ))
            } else if family == 35 {
                Box::new(crate::data_table_form::data_table_form(theme.clone()))
            } else if family == 31 {
                Box::new(crate::table_form::table_form(theme.clone()))
            } else if family == 30 {
                Box::new(crate::table_of_contents_form::table_of_contents_form(
                    theme.clone(),
                ))
            } else if family == 27 {
                Box::new(crate::floating_indicator_form::floating_indicator_form(
                    theme.clone(),
                ))
            } else if family == 26 {
                Box::new(crate::overflow_list_form::overflow_list_form(theme.clone()))
            } else if family == 32 {
                Box::new(crate::translation_form::translation_form(theme.clone()))
            } else if family == 33 {
                Box::new(crate::text_motion_form::text_motion_form(theme.clone()))
            } else if family == 34 {
                Box::new(crate::code_form::code_form(theme.clone()))
            } else if family == 18 {
                Box::new(crate::notifications_form::notifications_form(theme.clone()))
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
    pub fn mount_progress(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 13, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_navigation_progress(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 14, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_spinner(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 15, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_loading_overlay(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 17, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_inert_stream_probe() -> Result<(), JsValue> {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("inert-stream-root"))
            .ok_or_else(|| js("missing inert stream root"))?;
        let app = glory::web::hydrate(root, crate::loading_overlay_form::inert_stream_probe())
            .map_err(js)?;
        APP.with(|slot| *slot.borrow_mut() = Some(app));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn mount_modals(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 16, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_notifications(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 18, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_toolbar(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 21, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_color_swatch(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 22, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_sheet(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 19, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_accordion(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 23, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_tabs_dynamic(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 24, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_collapse_motion(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 25, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_data_table_server(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 37, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_data_table(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 35, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_table(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 31, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_table_of_contents(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 30, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_overflow_list(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 26, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_floating_indicator(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 27, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_translation(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 32, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_text_motion(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 33, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_code(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 34, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_collapse_stream_probe() -> Result<(), JsValue> {
        let root = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("collapse-stream-root"))
            .ok_or_else(|| js("missing collapse stream root"))?;
        let app =
            glory::web::hydrate(root, crate::collapse_stream_probe::collapse_stream_probe())
                .map_err(js)?;
        APP.with(|slot| *slot.borrow_mut() = Some(app));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn mount_sheet_stream_probe() -> Result<(), JsValue> {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("sheet-stream-root"))
            .ok_or_else(|| js("missing sheet stream root"))?;
        let app =
            glory::web::hydrate(root, crate::sheet_form::sheet_stream_probe()).map_err(js)?;
        APP.with(|slot| *slot.borrow_mut() = Some(app));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn unmount_display() {
        let old = APP.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
