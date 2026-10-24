use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
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
                    .theme(theme)
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
                sapid::submit_button().label("Submit country"),
                button().label("After combobox"),
            ));
        (
            sapid::form("combobox-form")
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
}
