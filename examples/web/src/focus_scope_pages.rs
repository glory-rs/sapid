#[cfg(target_arch = "wasm32")]
mod browser {
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! { static PAIR: RefCell<[Option<WebApp>;2]> = const { RefCell::new([None,None]) }; }
    #[wasm_bindgen]
    pub fn mount_focus_scope_pair() -> Result<(), JsValue> {
        let old = PAIR.with(|pair| std::mem::take(&mut *pair.borrow_mut()));
        drop(old);
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("missing document"))?;
        for (index, prefix) in ["left", "right"].into_iter().enumerate() {
            let root = document
                .get_element_by_id(&format!("scope-{prefix}"))
                .ok_or_else(|| JsValue::from_str("missing scope root"))?;
            let app = glory::web::launch(
                root,
                crate::focus_scope_form::focus_scope_form_with_prefix(
                    sapid::Theme::blue_light(),
                    prefix,
                ),
            )
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
            PAIR.with(|pair| pair.borrow_mut()[index] = Some(app));
        }
        Ok(())
    }
    #[wasm_bindgen]
    pub fn retire_focus_scope_app(index: usize) -> Result<(), JsValue> {
        if index > 1 {
            return Err(JsValue::from_str("invalid scope App index"));
        }
        let app = PAIR.with(|pair| pair.borrow_mut()[index].take());
        drop(app);
        Ok(())
    }
}
#[cfg(target_arch = "wasm32")]
pub use browser::*;
