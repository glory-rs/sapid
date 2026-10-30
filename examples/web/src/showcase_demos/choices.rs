use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{
        CheckState, ChipMode, Theme, checkbox_group, checkbox_item, chip, chip_group, chip_item,
        field, mixed_checkbox, radio, radio_group, segment, segmented_control, switch,
    };

    pub struct State {
        pub mixed: Cage<CheckState>,
        pub switched: Cage<bool>,
        pub selected: Cage<Vec<String>>,
        pub reject: Cage<bool>,
        pub disabled: Cage<bool>,
        pub readonly: Cage<bool>,
        pub radio: Cage<String>,
        pub segment: Cage<String>,
        pub featured: Cage<bool>,
        pub channel: Cage<String>,
        pub languages: Cage<Vec<String>>,
        pub search: Cage<String>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let mixed = scope.cage(CheckState::Mixed);
        let switched = scope.cage(false);
        let selected = scope.cage(vec!["read".to_owned(), "inherited".to_owned()]);
        let reject = scope.cage(false);
        let disabled = scope.cage(false);
        let readonly = scope.cage(false);
        let radio_value = scope.cage("standard".to_owned());
        let segment_value = scope.cage("monthly".to_owned());
        let featured = scope.cage(false);
        let channel = scope.cage("email".to_owned());
        let languages = scope.cage(Vec::<String>::new());
        let search = scope.cage("alpha".to_owned());
        let enabled = scope.bond(move || !disabled.get());
        let widgets =
            view()
                .style(
                    Style::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Column)
                        .gap(24.0),
                )
                .fill((
                    field("mixed-choice")
                        .label("Partial selection")
                        .description("Native indeterminate selection")
                        .theme(theme.clone())
                        .readonly(readonly)
                        .enabled(enabled)
                        .control(move |field| {
                            mixed_checkbox().field(field).state(mixed).on_state_change(
                                move |state| {
                                    if !reject.get() {
                                        mixed.revise(|old| *old = state);
                                    }
                                },
                            )
                        }),
                    field("notification-switch")
                        .label("Notifications")
                        .description("Email updates")
                        .required(true)
                        .theme(theme.clone())
                        .readonly(readonly)
                        .enabled(enabled)
                        .control(move |field| {
                            switch().field(field).checked(switched).on_checked_change(
                                move |value| {
                                    if !reject.get() {
                                        switched.revise(|old| *old = value);
                                    }
                                },
                            )
                        }),
                    checkbox_group("permissions")
                        .theme(theme.clone())
                        .label("Permissions")
                        .select_all_label("All editable permissions")
                        .enabled(enabled)
                        .selected(selected)
                        .items([
                            checkbox_item("read", "Read"),
                            checkbox_item("write", "Write"),
                            checkbox_item("inherited", "Inherited").enabled(false),
                        ])
                        .on_selected_change(move |value| {
                            if !reject.get() {
                                selected.revise(|old| *old = value);
                            }
                        }),
                ));
        let widgets = view()
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(24.0),
            )
            .fill((
                widgets,
                field("featured-chip")
                    .label("Featured chip")
                    .theme(theme.clone())
                    .readonly(readonly)
                    .enabled(enabled)
                    .control(move |field| {
                        chip()
                            .field(field)
                            .label("Featured")
                            .checked(featured)
                            .on_checked_change(move |value| {
                                if !reject.get() {
                                    featured.revise(|old| *old = value);
                                }
                            })
                    }),
                field("channel-chips")
                    .label("Contact channel")
                    .theme(theme.clone())
                    .readonly(readonly)
                    .enabled(enabled)
                    .control(move |field| {
                        chip_group("channel-choice")
                            .field(field)
                            .value(channel)
                            .items([
                                chip_item("email", "Email"),
                                chip_item("sms", "SMS"),
                                chip_item("push", "Push").enabled(false),
                            ])
                            .on_value_change(move |value| {
                                if !reject.get() {
                                    channel.revise(|old| *old = value);
                                }
                            })
                    }),
                field("language-chips")
                    .label("Languages")
                    .theme(theme.clone())
                    .required(true)
                    .readonly(readonly)
                    .enabled(enabled)
                    .control(move |field| {
                        chip_group("languages-choice")
                            .mode(ChipMode::Multiple)
                            .field(field)
                            .values(languages)
                            .items([
                                chip_item("rust", "Rust"),
                                chip_item("wasm", "Wasm"),
                                chip_item("cpp", "C++").enabled(false),
                            ])
                            .on_values_change(move |value| {
                                if !reject.get() {
                                    languages.revise(|old| *old = value);
                                }
                            })
                    }),
                field("billing-segment")
                    .label("Billing interval")
                    .theme(theme.clone())
                    .readonly(readonly)
                    .enabled(enabled)
                    .control(move |field| {
                        segmented_control("billing-choice")
                            .field(field)
                            .form_name("billing")
                            .value(segment_value)
                            .segments([
                                segment("monthly").label("Monthly"),
                                segment("weekly").label("Weekly").enabled(false),
                                segment("yearly").label("Yearly"),
                            ])
                            .on_value_change(move |value| {
                                if !reject.get() {
                                    segment_value.revise(|old| *old = value);
                                }
                            })
                    }),
                radio_group("search-choice")
                    .label("Searchable options")
                    .theme(theme.clone())
                    .orientation(glory::render::Orientation::Horizontal)
                    .direction(glory::render::Direction::Rtl)
                    .typeahead(sapid::TypeaheadConfig {
                        timeout_millis: 100,
                        query_bytes: 32,
                    })
                    .enabled(enabled)
                    .readonly(readonly)
                    .value(search)
                    .items([
                        radio("alpha").label("Alpha"),
                        radio("apple").label("Apple"),
                        radio("apricot").label("Apricot").enabled(false),
                        radio("beta").label("Beta"),
                    ])
                    .on_value_change(move |value| {
                        if !reject.get() {
                            search.revise(|old| *old = value);
                        }
                    }),
                field("delivery-radio")
                    .label("Delivery")
                    .description("Choose a delivery option")
                    .required(true)
                    .theme(theme)
                    .readonly(readonly)
                    .enabled(enabled)
                    .control(move |field| {
                        radio_group("delivery-choice")
                            .field(field)
                            .form_name("delivery")
                            .value(radio_value)
                            .items([
                                radio("standard").label("Standard"),
                                radio("unavailable").label("Unavailable").enabled(false),
                                radio("express").label("Express"),
                            ])
                            .on_value_change(move |value| {
                                if !reject.get() {
                                    radio_value.revise(|old| *old = value);
                                }
                            })
                    }),
                button().label("After choices"),
            ));
        (
            widgets,
            State {
                mixed,
                switched,
                selected,
                reject,
                disabled,
                readonly,
                radio: radio_value,
                segment: segment_value,
                featured,
                channel,
                languages,
                search,
            },
        )
    }
}
