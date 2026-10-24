#[path = "../alerts_form.rs"]
pub mod alerts_form;
#[path = "../actions_form.rs"]
pub mod actions_form;
#[path = "../avatar_form.rs"]
pub mod avatar_form;
#[path = "../button_form.rs"]
pub mod button_form;
pub mod choices_form;
pub mod composite_form;
#[path = "../display_form.rs"]
pub mod display_form;
pub mod display_pages;
#[path = "../document_form.rs"]
pub mod document_form;
pub mod fields_form;
pub mod form;
pub mod forms_form;
#[path = "../graphics_form.rs"]
pub mod graphics_form;
#[path = "../grid_form.rs"]
pub mod grid_form;
pub mod hints_form;
#[path = "../image_form.rs"]
pub mod image_form;
#[path = "../layout_form.rs"]
pub mod layout_form;
pub mod menu_form;
pub mod navigation_form;
pub mod native_select_form;
pub mod overlay_form;
pub mod scroll_area_form;
pub mod shell_form;
#[path = "../spoiler_form.rs"]
pub mod spoiler_form;
pub mod stepper_form;
#[path = "../timeline_form.rs"]
pub mod timeline_form;
pub mod toggles_form;
pub mod typography_form;
#[path = "../visually_hidden_form.rs"]
pub mod visually_hidden_form;

#[cfg(target_arch = "wasm32")]
mod browser {
    use crate::form::State;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;

    thread_local! { static APP: RefCell<Option<(WebApp, State)>> = const { RefCell::new(None) }; }
    thread_local! { static COMPOSITES: RefCell<Option<(WebApp, crate::composite_form::State)>> = const { RefCell::new(None) }; }
    thread_local! { static OVERLAYS: RefCell<Option<(WebApp, crate::overlay_form::State)>> = const { RefCell::new(None) }; }
    thread_local! { static FIELDS: RefCell<Option<(WebApp, crate::fields_form::State)>> = const { RefCell::new(None) }; }
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }

    fn page_theme() -> sapid::Theme {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element());
        sapid::Theme::new(
            if root
                .as_ref()
                .and_then(|e| e.get_attribute("data-hue"))
                .as_deref()
                == Some("green")
            {
                sapid::Hue::Green
            } else {
                sapid::Hue::Blue
            },
            if root
                .as_ref()
                .and_then(|e| e.get_attribute("data-mode"))
                .as_deref()
                == Some("dark")
            {
                sapid::ColorScheme::Dark
            } else {
                sapid::ColorScheme::Light
            },
        )
    }

    #[wasm_bindgen]
    pub fn mount(hydrate: bool) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing root"))?;
        let mut state = None;
        let build = |scope: &glory::Scope| {
            let (widget, source) = crate::form::form_themed(scope, page_theme());
            state = Some(source);
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
    pub fn set_loading(value: bool) -> Result {
        APP.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            state.loading.revise(|current| *current = value);
            app.app().flush().map_err(js)
        })
    }

    #[wasm_bindgen]
    pub fn set_value(value: String) -> Result {
        APP.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            state.value.revise(|current| *current = value);
            app.app().flush().map_err(js)
        })
    }

    #[wasm_bindgen]
    pub fn unmount() {
        let old = FIELDS.with(|slot| slot.borrow_mut().take());
        drop(old);
        let old = OVERLAYS.with(|slot| slot.borrow_mut().take());
        drop(old);
        let old = APP.with(|slot| slot.borrow_mut().take());
        drop(old);
        let old = COMPOSITES.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
    #[wasm_bindgen]
    pub fn input_text(value: String) -> Result {
        FIELDS.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.bio.revise(|old| *old = value);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn input_state(kind: String, value: bool) -> Result {
        FIELDS.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "visible" => state.visible.revise(|old| *old = value),
                "reject" => state.reject.revise(|old| *old = value),
                _ => return Err(js("unknown input state")),
            }
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn mount_fields(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
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
        let mut state = None;
        let build = |scope: &glory::Scope| {
            let (widget, source) = crate::fields_form::form(scope, theme.clone());
            state = Some(source);
            widget
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        FIELDS.with(|slot| *slot.borrow_mut() = Some((app, state.unwrap())));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn field_state(kind: String, value: bool) -> Result {
        FIELDS.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "error" => state.error.revise(|v| {
                    *v = if value {
                        "Enter a valid work email address.".into()
                    } else {
                        String::new()
                    }
                }),
                "readonly" => state.readonly.revise(|v| *v = value),
                "disabled" => state.disabled.revise(|v| *v = value),
                "required" => state.required.revise(|v| *v = value),
                _ => return Err(js("unknown field state")),
            };
            app.app().flush().map_err(js)
        })
    }

    #[wasm_bindgen]
    pub fn mount_overlays(hydrate: bool) -> Result {
        mount_overlay_state(hydrate, false)
    }
    #[wasm_bindgen]
    pub fn mount_overlays_initial(hydrate: bool) -> Result {
        mount_overlay_state(hydrate, true)
    }
    fn mount_overlay_state(hydrate: bool, initial: bool) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing root"))?;
        let mut state = None;
        let build = |scope: &glory::Scope| {
            let (widget, source) =
                crate::overlay_form::form_themed(scope, initial, page_theme());
            state = Some(source);
            widget
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        OVERLAYS.with(|slot| *slot.borrow_mut() = Some((app, state.unwrap())));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn overlay_open(value: bool) -> Result {
        OVERLAYS.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.open.revise(|current| *current = value);
            app.app().flush().map_err(js)
        })
    }

    #[wasm_bindgen]
    pub fn mount_composites(hydrate: bool) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing root"))?;
        let mut state = None;
        let build = |scope: &glory::Scope| {
            let (widget, source) = crate::composite_form::form_themed(scope, page_theme());
            state = Some(source);
            widget
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        COMPOSITES.with(|slot| *slot.borrow_mut() = Some((app, state.unwrap())));
        Ok(())
    }

    #[wasm_bindgen]
    pub fn composite_open(value: bool) -> Result {
        COMPOSITES.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            // These test commands enter from JS outside Glory's host turn.
            // Observe earlier queued native focus events before a later write.
            app.app().pump().map_err(js)?;
            state.open.revise(|current| *current = value);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn composite_value(value: String) -> Result {
        COMPOSITES.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.selected.revise(|current| *current = value);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn composite_disabled(value: bool) -> Result {
        COMPOSITES.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.disabled.revise(|current| *current = value);
            app.app().flush().map_err(js)
        })
    }
}
