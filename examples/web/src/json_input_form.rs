use glory::prelude::*;
use sapid::{JsonLimits, Theme, json_input};
pub struct State {
    pub value: Cage<String>,
    pub reject: Cage<bool>,
    pub readonly: Cage<bool>,
    pub enabled: Cage<bool>,
    pub error: Cage<String>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let value = scope.cage(String::from(
        r#"{"z":1e999,"a":9007199254740993123456789,"z":"😀"}"#,
    ));
    let reject = scope.cage(false);
    let readonly = scope.cage(false);
    let enabled = scope.cage(true);
    let error = scope.cage(String::new());
    let changes = scope.cage(0usize);
    let formats = scope.cage(0usize);
    let validity = scope.cage(String::new());
    let main = sapid::field("json-payload")
        .theme(theme.clone())
        .label("Payload")
        .description("JSON request document")
        .form_name("payload")
        .required(true)
        .error(error)
        .readonly(readonly)
        .enabled(enabled)
        .control(move |field| {
            json_input("payload")
                .field(field)
                .value(value)
                .class("payload-json")
                .on_value_change(move |next| {
                    changes.revise(|old| *old += 1);
                    if !reject.get_untracked() {
                        value.revise(|old| *old = next)
                    }
                })
                .on_format(move |_| formats.revise(|old| *old += 1))
                .on_validation(move |state| validity.revise(|old| *old = format!("{state:?}")))
        });
    let root = view().class("json-demo").fill((
        sapid::button().theme(theme.clone()).label("Before JSON"),
        main,
        json_input("local")
            .theme(theme.clone())
            .name("Local draft")
            .class("local-json")
            .default_value("{")
            .format_label("Format local"),
        json_input("limited")
            .theme(theme.clone())
            .name("Limited JSON")
            .class("limited-json")
            .default_value("[1,2,3]")
            .limits(JsonLimits { bytes: 8, depth: 8 })
            .format_label("Format limited"),
        json_input("unstyled")
            .theme(theme.clone())
            .name("Unstyled JSON")
            .class("unstyled-json")
            .default_value("[]")
            .unstyled()
            .show_formatter(false),
        text().class("json-status").content(scope.bond(move || {
            format!(
                "changes={};formats={};state={}",
                changes.get(),
                formats.get(),
                validity.get()
            )
        })),
        sapid::button().theme(theme).label("After JSON"),
    ));
    (
        root,
        State {
            value,
            reject,
            readonly,
            enabled,
            error,
        },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static JSON: RefCell<Option<(WebApp, State)>> = const { RefCell::new(None) };}
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_json_input(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("app"))
            .ok_or_else(|| js("missing JSON root"))?;
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
            let (widget, model) = form(scope, theme);
            state = Some(model);
            widget
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        JSON.with(|slot| *slot.borrow_mut() = Some((app, state.unwrap())));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn json_input_state(kind: String, value: String) -> Result {
        JSON.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("JSON unmounted"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "value" => state.value.revise(|old| *old = value),
                "error" => state.error.revise(|old| *old = value),
                "reject" => state.reject.revise(|old| *old = value == "true"),
                "readonly" => state.readonly.revise(|old| *old = value == "true"),
                "enabled" => state.enabled.revise(|old| *old = value == "true"),
                _ => return Err(js("unknown JSON fixture state")),
            }
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_json_input() {
        drop(JSON.with(|slot| slot.borrow_mut().take()));
    }
}
#[cfg(target_arch = "wasm32")]
pub use browser::*;
