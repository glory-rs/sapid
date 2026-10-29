#[path = "../../fixtures/dsl_demo.rs"]
pub mod fixture;

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::fixture;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static APP: RefCell<Option<WebApp>> = const {RefCell::new(None)};}

    #[wasm_bindgen]
    pub fn dsl_mount(hydrate: bool) -> Result<(), JsValue> {
        let root = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("app")
            .unwrap();
        let make = || fixture::demo(sapid::Theme::light());
        let app = if hydrate {
            glory::web::hydrate(root, make())
                .map_err(|error| JsValue::from_str(&error.to_string()))
        } else {
            glory::web::launch(root, make())
                .map_err(|error| JsValue::from_str(&error.to_string()))
        }?;
        APP.with(|slot| *slot.borrow_mut() = Some(app));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn dsl_close() -> Result<usize, JsValue> {
        let app = APP
            .with(|slot| slot.borrow_mut().take())
            .ok_or("not mounted")?;
        let runtime = app.app().shared_runtime();
        app.app()
            .clear()
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        drop(app);
        Ok(runtime.stats().nodes)
    }
}
