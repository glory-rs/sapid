use glory::prelude::*;
use sapid::{Theme, field, select_group, select_option};

pub struct State {
    pub value: Cage<Option<String>>,
    pub search: Cage<String>,
    pub open: Cage<bool>,
    pub reject: Cage<bool>,
    pub reject_search: Cage<bool>,
    pub reject_open: Cage<bool>,
    pub readonly: Cage<bool>,
    pub disabled: Cage<bool>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let value = scope.cage(Some("ca".to_owned()));
    let search = scope.cage("Canada".to_owned());
    let open = scope.cage(false);
    let reject = scope.cage(false);
    let reject_search = scope.cage(false);
    let reject_open = scope.cage(false);
    let readonly = scope.cage(false);
    let disabled = scope.cage(false);
    let changed = scope.cage(0usize);
    let searched = scope.cage(0usize);
    let submitted = scope.cage(0usize);
    let enabled = scope.bond(move || !disabled.get());
    let content = view()
        .style(
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(16.0),
        )
        .fill((
            button().label("Before combobox"),
            field("editable-country")
                .theme(theme.clone())
                .label("Country")
                .description("Type a country, then choose a suggestion.")
                .required(true)
                .form_name("country")
                .control(move |field| {
                    sapid::combobox("country")
                        .field(field)
                        .value(value)
                        .search(search)
                        .open(open)
                        .enabled(enabled)
                        .readonly(readonly)
                        .options([
                            select_option("ca", "Canada").value("CA"),
                            select_option("ic", "Canary Islands").value("IC"),
                        ])
                        .group(select_group("Europe").options([
                            select_option("fr", "France").value("FR"),
                            select_option("de", "Germany").value("DE"),
                        ]))
                        .group(
                            select_group("Unavailable")
                                .disabled(true)
                                .option(select_option("jp", "Japan").value("JP")),
                        )
                        .on_value_change(move |proposal| {
                            changed.revise(|value| *value += 1);
                            if !reject.get() {
                                value.revise(|value| *value = proposal);
                            }
                        })
                        .on_search_change(move |proposal| {
                            searched.revise(|value| *value += 1);
                            if !reject_search.get() {
                                search.revise(|value| *value = proposal);
                            }
                        })
                        .on_open_change(move |proposal| {
                            if !reject_open.get() {
                                open.revise(|value| *value = proposal)
                            }
                        })
                }),
            text().content(scope.bond(move || {
                format!(
                    "Changes: {}, edits: {}, submits: {}",
                    changed.get(),
                    searched.get(),
                    submitted.get()
                )
            })),
            sapid::submit_button()
                .theme(theme.clone())
                .label("Submit country"),
            button().label("After combobox"),
        ));
    (
        sapid::form("combobox-form")
            .theme(theme.clone())
            .on_submit(move |_| submitted.revise(|value| *value += 1))
            .fill(content),
        State {
            value,
            search,
            open,
            reject,
            reject_search,
            reject_open,
            readonly,
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
    thread_local! {static COMBOBOX:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_combobox(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing combobox root"))?;
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
            let (widget, source) = form(scope, theme);
            state = Some(source);
            widget
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        COMBOBOX.with(|slot| *slot.borrow_mut() = Some((app, state.unwrap())));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn combobox_state(kind: String, value: String) -> Result {
        COMBOBOX.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("combobox unmounted"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "value" => state
                    .value
                    .revise(|old| *old = if value == "none" { None } else { Some(value) }),
                "search" => state.search.revise(|old| *old = value),
                "open" => state.open.revise(|old| *old = value == "true"),
                "reject" => state.reject.revise(|old| *old = value == "true"),
                "reject_search" => state.reject_search.revise(|old| *old = value == "true"),
                "reject_open" => state.reject_open.revise(|old| *old = value == "true"),
                "readonly" => state.readonly.revise(|old| *old = value == "true"),
                "disabled" => state.disabled.revise(|old| *old = value == "true"),
                _ => return Err(js("unknown combobox state")),
            };
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_combobox() {
        let old = COMBOBOX.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
#[cfg(target_arch = "wasm32")]
pub use browser::*;
