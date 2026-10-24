//! Browser attachment for the Rust-rendered documentation site.
#[cfg(target_arch = "wasm32")]
mod browser {
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {
        static APP: RefCell<Option<glory::web::WebApp>> = const { RefCell::new(None) };
    }
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    /// Route, theme and hydration are initial application inputs. Navigation,
    /// search, menus, controls and code views are built by Glory/Sapid in Rust.
    #[wasm_bindgen]
    pub fn mount_showcase(
        hydrate: bool,
        selection: String,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        let root = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("app"))
            .ok_or_else(|| js("showcase mount root is missing"))?;
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
        // Dispose a previous application before binding its root again. The
        // outer #app is host-owned and is never constructed by this widget.
        let old = APP.with(|slot| slot.borrow_mut().take());
        drop(old);
        let widget = crate::showcase::site(theme, selection);
        let app = if hydrate {
            glory::web::hydrate(root, widget).map_err(js)?
        } else {
            glory::web::launch(root, widget).map_err(js)?
        };
        APP.with(|slot| *slot.borrow_mut() = Some(app));
        Ok(())
    }
}
