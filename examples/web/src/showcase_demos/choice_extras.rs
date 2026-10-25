use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
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
                    .theme(theme.clone())
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
                sapid::submit_button()
                    .theme(theme.clone())
                    .label("Submit choices"),
                button().label("After choices"),
            ));
        (
            sapid::form("choice-extras")
                .theme(theme.clone())
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
}
