use glory::prelude::*;
use sapid::{FieldOptions, FormState, Theme, field, select_group, select_option};

pub struct State {
    pub free: Cage<String>,
    pub values: Cage<Vec<String>>,
    pub search: Cage<String>,
    pub reject_free: Cage<bool>,
    pub reject_values: Cage<bool>,
    pub reject_search: Cage<bool>,
    pub readonly: Cage<bool>,
    pub disabled: Cage<bool>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let free = scope.cage("Custom city".to_owned());
    let values = scope.cage(vec!["ca".to_owned()]);
    let search = scope.cage(String::new());
    let reject_free = scope.cage(false);
    let reject_values = scope.cage(false);
    let reject_search = scope.cage(false);
    let readonly = scope.cage(false);
    let disabled = scope.cage(false);
    let enabled = scope.bond(move || !disabled.get());
    let submitted = scope.cage(0usize);
    let edits = scope.cage(0usize);
    let commits = scope.cage(0usize);
    let changed = scope.cage(0usize);
    let commands = std::rc::Rc::new(std::cell::RefCell::new(
        None::<sapid::multi_select::MultiSelectHandle>,
    ));
    let ready = commands.clone();
    let select_all = commands.clone();
    let clear_all = commands.clone();
    let command_enabled = scope.bond(move || enabled.get() && !readonly.get());
    let weak_all = scope.downgrade();
    let weak_clear = scope.downgrade();
    let model = FormState::new(scope);
    let selected_field = model
        .register(
            scope,
            "countries",
            values,
            FieldOptions::default()
                .disabled(disabled)
                .validate(|values: &Vec<String>| {
                    if values.is_empty() {
                        vec!["Choose at least one country".into()]
                    } else {
                        vec![]
                    }
                }),
        )
        .expect("bounded choice fixture");
    let selected_state = selected_field.state();
    let errors = scope.bond(move || selected_state.get().errors.join(". "));
    let content = view()
        .style(
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(16.0),
        )
        .fill((
            button().label("Before choices"),
            field("free-city")
                .theme(theme.clone())
                .label("City")
                .description("Choose a suggestion or enter any city.")
                .required(true)
                .form_name("city")
                .control(move |field| {
                    sapid::autocomplete("city")
                        .field(field)
                        .value(free)
                        .readonly(readonly)
                        .enabled(enabled)
                        .option(select_option("paris", "Paris suggestion").value("Paris"))
                        .option(select_option("rome", "Rome suggestion").value("Roma"))
                        .option(select_option("blocked", "Unavailable city").disabled(true))
                        .on_value_change(move |value| {
                            edits.revise(|count| *count += 1);
                            if !reject_free.get() {
                                free.revise(|current| *current = value);
                            }
                        })
                        .on_commit(move |_| commits.revise(|count| *count += 1))
                }),
            field("many-countries")
                .theme(theme)
                .label("Countries")
                .description("Search, choose several, or remove a tag.")
                .required(true)
                .form_name("countries")
                .error(errors)
                .control(move |field| {
                    sapid::multi_select("countries")
                        .on_ready({
                            let ready = ready.clone();
                            move |handle| *ready.borrow_mut() = Some(handle)
                        })
                        .field(field)
                        .value(values)
                        .search(search)
                        .readonly(readonly)
                        .enabled(enabled)
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
                            changed.revise(|count| *count += 1);
                            if !reject_values.get() {
                                values.revise(|current| *current = proposal);
                            }
                        })
                        .on_search_change(move |proposal| {
                            if !reject_search.get() {
                                search.revise(|current| *current = proposal);
                            }
                        })
                }),
            button()
                .label("Select filtered countries")
                .enabled(command_enabled)
                .on(glory::render::EventKind::Activate, move |_| {
                    let command = select_all.borrow().clone();
                    if let Some(command) = command
                        && let Err(error) = command.select_all()
                        && let Some(scope) = weak_all.upgrade()
                    {
                        scope.report_error(error);
                    }
                }),
            button()
                .label("Clear selected countries")
                .enabled(command_enabled)
                .on(glory::render::EventKind::Activate, move |_| {
                    let command = clear_all.borrow().clone();
                    if let Some(command) = command
                        && let Err(error) = command.clear_all()
                        && let Some(scope) = weak_clear.upgrade()
                    {
                        scope.report_error(error);
                    }
                }),
            text().content(scope.bond(move || {
                format!(
                    "Edits: {}, commits: {}, changes: {}, submits: {}",
                    edits.get(),
                    commits.get(),
                    changed.get(),
                    submitted.get()
                )
            })),
            sapid::submit_button().label("Submit choices"),
            button().label("After choices"),
        ));
    (
        sapid::form("choice-extras")
            .on_submit(move |_| {
                model
                    .submit(move |token| {
                        submitted.revise(|count| *count += 1);
                        token.complete(Ok(()));
                    })
                    .expect("fixture submit");
            })
            .fill(content),
        State {
            free,
            values,
            search,
            reject_free,
            reject_values,
            reject_search,
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
    thread_local! {static EXTRAS:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_choice_extras(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing extras root"))?;
        let theme = Theme::new(
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
        EXTRAS.with(|slot| *slot.borrow_mut() = Some((app, state.unwrap())));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn choice_extras_state(kind: String, value: String) -> Result {
        EXTRAS.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("extras unmounted"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "free" => state.free.revise(|current| *current = value),
                "values" => state.values.revise(|current| {
                    *current = if value.is_empty() {
                        vec![]
                    } else {
                        value.split(',').map(str::to_owned).collect()
                    }
                }),
                "search" => state.search.revise(|current| *current = value),
                "reject_free" => state
                    .reject_free
                    .revise(|current| *current = value == "true"),
                "reject_values" => state
                    .reject_values
                    .revise(|current| *current = value == "true"),
                "reject_search" => state
                    .reject_search
                    .revise(|current| *current = value == "true"),
                "readonly" => state.readonly.revise(|current| *current = value == "true"),
                "disabled" => state.disabled.revise(|current| *current = value == "true"),
                _ => return Err(js("unknown extras state")),
            }
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_choice_extras() {
        let old = EXTRAS.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
#[cfg(target_arch = "wasm32")]
pub use browser::*;
