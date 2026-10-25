use glory::prelude::*;
use sapid::{Theme, field, select_group, select_option};

pub struct State {
    pub value: Cage<Option<String>>,
    pub open: Cage<bool>,
    pub reject: Cage<bool>,
    pub reject_open: Cage<bool>,
    pub readonly: Cage<bool>,
    pub disabled: Cage<bool>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let value = scope.cage(Some("us".to_owned()));
    let open = scope.cage(false);
    let reject = scope.cage(false);
    let reject_open = scope.cage(false);
    let readonly = scope.cage(false);
    let disabled = scope.cage(false);
    let proposals = scope.cage(0usize);
    let enabled = scope.bond(move || !disabled.get());
    let content = view()
        .style(
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(16.0),
        )
        .fill((
            button().label("Before choices"),
            field("destination")
                .theme(theme.clone())
                .label("Destination")
                .description("Choose your delivery region.")
                .required(true)
                .form_name("destination")
                .control(move |field| {
                    sapid::select("destination")
                        .field(field)
                        .value(value)
                        .open(open)
                        .enabled(enabled)
                        .readonly(readonly)
                        .clearable(true)
                        .placeholder("Choose a destination")
                        .options([
                            select_option("us", "United States").value("US"),
                            select_option("ca", "Canada").value("CA"),
                            select_option("cl", "Chile").value("CL"),
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
                            proposals.revise(|count| *count += 1);
                            if !reject.get() {
                                value.revise(|old| *old = proposal);
                            }
                        })
                        .on_open_change(move |proposal| {
                            if !reject_open.get() {
                                open.revise(|old| *old = proposal)
                            }
                        })
                }),
            sapid::select("long")
                .theme(theme.clone())
                .label("Jump to number")
                .placeholder("Choose a number")
                .options((0..80).map(|index| {
                    select_option(format!("n{index}"), format!("Number {index:02}"))
                })),
            text().content(scope.bond(move || format!("Select changes: {}", proposals.get()))),
            button().label("After choices"),
        ));
    (
        sapid::form("custom-select-form")
            .theme(theme.clone())
            .fill(content),
        State {
            value,
            open,
            reject,
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
    thread_local! {static SELECT:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_select(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing select root"))?;
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
        SELECT.with(|slot| *slot.borrow_mut() = Some((app, state.unwrap())));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn select_state(kind: String, value: String) -> Result {
        SELECT.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("select unmounted"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "value" => state
                    .value
                    .revise(|old| *old = if value == "none" { None } else { Some(value) }),
                "open" => state.open.revise(|old| *old = value == "true"),
                "reject" => state.reject.revise(|old| *old = value == "true"),
                "reject_open" => state.reject_open.revise(|old| *old = value == "true"),
                "readonly" => state.readonly.revise(|old| *old = value == "true"),
                "disabled" => state.disabled.revise(|old| *old = value == "true"),
                _ => return Err(js("unknown select state")),
            };
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_select() {
        let old = SELECT.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
#[cfg(target_arch = "wasm32")]
pub use browser::*;
