use glory::prelude::*;
use sapid::{Theme, field, native_select, select_group, select_option};

pub struct State {
    pub selected: Cage<Option<String>>,
    pub many: Cage<Vec<String>>,
    pub reject: Cage<bool>,
    pub disabled: Cage<bool>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let selected = scope.cage(Some("starter".to_owned()));
    let many = scope.cage(vec!["orange".into()]);
    let reject = scope.cage(false);
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
            button().label("Before select"),
            field("native-plan")
                .theme(theme.clone())
                .label("Subscription plan")
                .description("Choose a billing tier.")
                .form_name("plan")
                .required(true)
                .control(move |field| {
                    native_select("plan")
                        .field(field)
                        .value(selected)
                        .enabled(enabled)
                        .placeholder("Choose a plan")
                        .options([
                            select_option("starter", "Starter").value("start-value"),
                            select_option("pro", "Professional").value("pro-value"),
                        ])
                        .group(select_group("Unavailable").disabled(true).option(
                            select_option("enterprise", "Enterprise").value("enterprise-value"),
                        ))
                        .on_value_change(move |value| {
                            proposals.revise(|count| *count += 1);
                            if !reject.get() {
                                selected.revise(|old| *old = value);
                            }
                        })
                }),
            native_select("fruit")
                .theme(theme.clone())
                .label("Favourite fruit")
                .form_name("fruit")
                .rows(4)
                .values(many)
                .options([
                    select_option("apple", "Apple").value("red"),
                    select_option("orange", "Orange").value("orange-value"),
                    select_option("pear", "Pear").value("green"),
                    select_option("blocked", "Blocked").disabled(true),
                ])
                .on_values_change(move |value| {
                    if !reject.get() {
                        many.revise(|old| *old = value)
                    }
                }),
            native_select("empty")
                .theme(theme.clone())
                .label("Optional empty choice")
                .form_name("empty")
                .placeholder("Nothing selected")
                .option(select_option("empty-value", "Explicit empty value").value("")),
            text().content(scope.bond(move || format!("Select proposals: {}", proposals.get()))),
            button().label("After select"),
        ));
    (
        sapid::form("native-select-form")
            .theme(theme.clone())
            .fill(content),
        State {
            selected,
            many,
            reject,
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
    thread_local! { static SELECT:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)}; }
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_native_select(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing native select root"))?;
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
    pub fn native_select_state(kind: String, value: String) -> Result {
        SELECT.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("select unmounted"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "selected" => state
                    .selected
                    .revise(|old| *old = if value == "none" { None } else { Some(value) }),
                "reject" => state.reject.revise(|old| *old = value == "true"),
                "disabled" => state.disabled.revise(|old| *old = value == "true"),
                _ => return Err(js("unknown select state")),
            };
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_native_select() {
        let old = SELECT.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
#[cfg(target_arch = "wasm32")]
pub use browser::*;
