//! Host services. UI/model code contains no browser objects or JavaScript UI.
use super::model::{ThemeRecord, ThemeStore};
use sapid::Theme;
pub const STORAGE_KEY: &str = "sapid.theme-studio.v1";
#[cfg(target_arch = "wasm32")]
fn storage() -> Result<web_sys::Storage, String> {
    web_sys::window()
        .ok_or("Missing window")?
        .local_storage()
        .map_err(|_| "Local storage is unavailable")?
        .ok_or_else(|| "Local storage is unavailable".into())
}
pub fn load() -> Result<ThemeStore, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let text = storage()?
            .get_item(STORAGE_KEY)
            .map_err(|_| "Could not read local themes")?;
        match text {
            None => Ok(ThemeStore::default()),
            Some(text) => {
                let store = ThemeStore::parse(&text)?;
                if ThemeStore::needs_upgrade(&text) {
                    let storage = storage()?;
                    let backup = format!("{STORAGE_KEY}.backup");
                    if storage
                        .get_item(&backup)
                        .map_err(|_| "Could not read local theme backup")?
                        .is_none()
                    {
                        storage
                            .set_item(&backup, &text)
                            .map_err(|_| "Could not back up local themes")?;
                    }
                    storage
                        .set_item(STORAGE_KEY, &store.json()?)
                        .map_err(|_| "Could not upgrade local themes")?;
                }
                Ok(store)
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok(ThemeStore::default())
    }
}
pub fn save(record: ThemeRecord) -> Result<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let mut store = load()?;
        let id = store.save(record)?;
        storage()?
            .set_item(STORAGE_KEY, &store.json()?)
            .map_err(|_| "Could not save: local storage quota or permission")?;
        Ok(id)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = record;
        Err("Saving is available in the browser".into())
    }
}
pub fn remove(id: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let mut store = load()?;
        store.themes.retain(|r| r.id != id);
        storage()?
            .set_item(STORAGE_KEY, &store.json()?)
            .map_err(|_| "Could not remove saved theme")?;
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = id;
        Err("Saving is available in the browser".into())
    }
}
pub fn navigate(url: &str) -> Result<(), String> {
    crate::showcase_host::navigate(url)
}
#[cfg(target_arch = "wasm32")]
fn style(id: &str, css: &str) -> Result<(), String> {
    let doc = web_sys::window()
        .and_then(|w| w.document())
        .ok_or("Missing document")?;
    let element = match doc.get_element_by_id(id) {
        Some(e) => e,
        None => {
            let e = doc
                .create_element("style")
                .map_err(|_| "Could not create theme stylesheet")?;
            e.set_id(id);
            doc.document_element()
                .ok_or("Missing document root")?
                .append_child(&e)
                .map_err(|_| "Could not attach theme stylesheet")?;
            e
        }
    };
    element.set_text_content(Some(css));
    Ok(())
}
pub fn preview(theme: &Theme) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        style(
            "sapid-studio-preview",
            &theme.stylesheet().map_err(|e| e.to_string())?,
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = theme;
        Ok(())
    }
}
pub fn install_catalog(store: &ThemeStore) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let mut css = String::new();
        for record in &store.themes {
            for mode in [sapid::ColorScheme::Light, sapid::ColorScheme::Dark] {
                if record.supports(mode) {
                    css.push_str(
                        &record
                            .theme(mode)?
                            .stylesheet()
                            .map_err(|e| e.to_string())?,
                    )
                }
            }
        }
        style("sapid-saved-themes", &css)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = store;
        Ok(())
    }
}
pub fn install_page(theme: &Theme) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        style(
            "sapid-active-theme",
            &theme.stylesheet().map_err(|e| e.to_string())?,
        )?;
        let doc = web_sys::window()
            .and_then(|w| w.document())
            .ok_or("Missing document")?;
        for element in [doc.document_element(), doc.body().map(Into::into)]
            .into_iter()
            .flatten()
        {
            let html = element
                .dyn_into::<web_sys::HtmlElement>()
                .map_err(|_| "Invalid host element")?;
            html.style()
                .set_property("background-color", &super::model::hex(theme.canvas))
                .map_err(|_| "Could not set page canvas")?;
            html.style()
                .set_property("color", &super::model::hex(theme.foreground))
                .map_err(|_| "Could not set page text")?;
            html.style()
                .set_property("color-scheme", theme.scheme.name())
                .map_err(|_| "Could not set color scheme")?;
        }
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = theme;
        Ok(())
    }
}
