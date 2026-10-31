use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    focused::demo_for("P2-05", theme)
}

#[allow(dead_code, unused_imports)]
mod focused {
    use glory::prelude::*;
    use sapid::{CheckState, ChipMode, Theme};

    pub fn demo_for(id: &str, theme: Theme) -> Box<dyn Widget> {
        match id {
            "P0-10" => Box::new(
                sapid::field("focused-collection")
                    .theme(theme.clone())
                    .label("Choose a fruit")
                    .description("Use arrow keys or type the beginning of an option.")
                    .control(|field| {
                        sapid::radio_group("focused-collection-options")
                            .field(field)
                            .orientation(glory::render::Orientation::Horizontal)
                            .typeahead(sapid::TypeaheadConfig {
                                timeout_millis: 700,
                                query_bytes: 32,
                            })
                            .default_value("apple")
                            .items([
                                sapid::radio("apple").label("Apple"),
                                sapid::radio("apricot").label("Apricot"),
                                sapid::radio("banana").label("Banana"),
                                sapid::radio("blueberry").label("Blueberry"),
                            ])
                    }),
            ),
            "P2-01" => Box::new(widget(move |scope| {
                let mixed = scope.cage(CheckState::Mixed);
                let selected = scope.cage(vec!["read".to_owned()]);
                sapid::stack()
                    .theme(theme.clone())
                    .gap(20.0)
                    .fill((
                        sapid::field("focused-mixed")
                            .theme(theme.clone())
                            .label("Partial selection")
                            .description("The parent checkbox preserves its indeterminate state.")
                            .control(move |field| {
                                sapid::mixed_checkbox()
                                    .field(field)
                                    .state(mixed)
                                    .on_state_change(move |state| {
                                        mixed.revise(|value| *value = state)
                                    })
                            }),
                        sapid::checkbox_group("focused-permissions")
                            .theme(theme.clone())
                            .label("Permissions")
                            .select_all_label("All permissions")
                            .selected(selected)
                            .items([
                                sapid::checkbox_item("read", "Read"),
                                sapid::checkbox_item("write", "Write"),
                                sapid::checkbox_item("admin", "Admin").enabled(false),
                            ])
                            .on_selected_change(move |value| {
                                selected.revise(|current| *current = value)
                            }),
                    ))
                    .show_in(scope)
            })),
            "P2-02" => Box::new(
                sapid::stack().theme(theme.clone()).gap(18.0).fill((
                    sapid::field("focused-notifications")
                        .theme(theme.clone())
                        .label("Notifications")
                        .description("Receive product updates by email.")
                        .control(|field| sapid::switch().field(field).default_checked(true)),
                    sapid::field("focused-autosave")
                        .theme(theme.clone())
                        .label("Autosave")
                        .control(|field| sapid::switch().field(field)),
                    sapid::field("focused-locked-switch")
                        .theme(theme.clone())
                        .label("Managed by your organization")
                        .enabled(false)
                        .control(|field| sapid::switch().field(field)),
                )),
            ),
            "P2-03" => Box::new(
                sapid::field("focused-delivery")
                    .theme(theme.clone())
                    .label("Delivery")
                    .description("Choose one delivery option.")
                    .required(true)
                    .control(|field| {
                        sapid::radio_group("focused-delivery-options")
                            .field(field)
                            .default_value("standard")
                            .items([
                                sapid::radio("standard").label("Standard"),
                                sapid::radio("express").label("Express"),
                                sapid::radio("unavailable")
                                    .label("Unavailable")
                                    .enabled(false),
                            ])
                    }),
            ),
            "P2-05" => Box::new(widget(move |scope| {
                let interval = scope.cage("monthly".to_owned());
                sapid::field("focused-billing")
                    .theme(theme.clone())
                    .label("Billing interval")
                    .control(move |field| {
                        sapid::segmented_control("focused-billing-options")
                            .field(field)
                            .value(interval)
                            .segments([
                                sapid::segment("monthly").label("Monthly"),
                                sapid::segment("yearly").label("Yearly"),
                                sapid::segment("lifetime").label("Lifetime"),
                            ])
                            .on_value_change(move |value| {
                                interval.revise(|current| *current = value)
                            })
                    })
                    .show_in(scope)
            })),
            "P2-06" => Box::new(widget(move |scope| {
                let languages = scope.cage(vec!["rust".to_owned()]);
                sapid::field("focused-languages")
                    .theme(theme.clone())
                    .label("Languages")
                    .description("Choose all that apply.")
                    .control(move |field| {
                        sapid::chip_group("focused-language-options")
                            .mode(ChipMode::Multiple)
                            .field(field)
                            .values(languages)
                            .items([
                                sapid::chip_item("rust", "Rust"),
                                sapid::chip_item("wasm", "Wasm"),
                                sapid::chip_item("typescript", "TypeScript"),
                            ])
                            .on_values_change(move |value| {
                                languages.revise(|current| *current = value)
                            })
                    })
                    .show_in(scope)
            })),
            _ => unreachable!("unsupported choices demo"),
        }
    }
}
