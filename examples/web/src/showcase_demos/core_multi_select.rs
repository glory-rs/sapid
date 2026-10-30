use glory::prelude::*;
use sapid::{ButtonVariant, MultiSelectHandle, Theme, select_option};
use std::{cell::RefCell, rc::Rc};

pub fn demo(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let values = scope.cage(vec![
            "ca".to_owned(),
            "ic".to_owned(),
            "fr".to_owned(),
            "de".to_owned(),
        ]);
        let handle = Rc::new(RefCell::new(None::<MultiSelectHandle>));
        let ready = handle.clone();
        let all = handle.clone();
        let clear = handle.clone();
        let enabled = scope.cage(true);
        let readonly = scope.cage(false);
        let commands_enabled = scope.bond(move || enabled.get() && !readonly.get());
        let weak_all = scope.downgrade();
        let weak_clear = scope.downgrade();
        let status = scope.bond(move || format!("{} countries selected", values.with(Vec::len)));
        sapid::stack()
            .theme(theme.clone())
            .gap(18.0)
            .fill((
                sapid::field("showcase-countries")
                    .theme(theme.clone())
                    .label("Countries")
                    .description("Search, choose several, or remove a tag.")
                    .control(move |field| {
                        let ready = ready.clone();
                        sapid::multi_select("showcase-countries")
                            .field(field)
                            .value(values)
                            .max_values(4)
                            .clearable(true)
                            .clear_label("Clear countries")
                            .placeholder("Search countries")
                            .enabled(enabled)
                            .readonly(readonly)
                            .options([
                                select_option("ca", "Canada"),
                                select_option("ic", "Canary Islands"),
                                select_option("fr", "France"),
                                select_option("de", "Germany"),
                                select_option("jp", "Japan").disabled(true),
                            ])
                            .on_value_change(move |proposal| {
                                values.revise(|current| *current = proposal)
                            })
                            .on_ready(move |command| *ready.borrow_mut() = Some(command))
                    }),
                sapid::group().theme(theme.clone()).fill((
                    sapid::button()
                        .theme(theme.clone())
                        .size(sapid::Size::Small)
                        .variant(ButtonVariant::Secondary)
                        .label("Select filtered countries")
                        .enabled(commands_enabled)
                        .on_press(move || {
                            let command = all.borrow().clone();
                            if let Some(command) = command
                                && let Err(error) = command.select_all()
                                && let Some(scope) = weak_all.upgrade()
                            {
                                scope.report_error(error);
                            }
                        }),
                    sapid::button()
                        .theme(theme.clone())
                        .size(sapid::Size::Small)
                        .variant(ButtonVariant::Secondary)
                        .label("Clear selected countries")
                        .enabled(commands_enabled)
                        .on_press(move || {
                            let command = clear.borrow().clone();
                            if let Some(command) = command
                                && let Err(error) = command.clear_all()
                                && let Some(scope) = weak_clear.upgrade()
                            {
                                scope.report_error(error);
                            }
                        }),
                )),
                sapid::text()
                    .theme(theme.clone())
                    .variant(sapid::TextVariant::Small)
                    .content(status),
                sapid::group().theme(theme.clone()).fill((
                    sapid::checkbox()
                        .theme(theme.clone())
                        .label("Disabled")
                        .checked(enabled.map(|value| !value))
                        .on_checked_change(move |value| {
                            enabled.revise(|current| *current = !value)
                        }),
                    sapid::checkbox()
                        .theme(theme.clone())
                        .label("Read only")
                        .checked(readonly)
                        .on_checked_change(move |value| {
                            readonly.revise(|current| *current = value)
                        }),
                )),
                limited_choices(theme.clone()),
            ))
            .show_in(scope)
    })
}

fn limited_choices(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let values = scope.cage(vec!["rust".to_owned()]);
        sapid::field("showcase-tools")
            .theme(theme.clone())
            .label("Tools")
            .description(
                "Select up to two tools. Selected options are hidden; clearing keeps your search.",
            )
            .control(move |field| {
                sapid::multi_select("showcase-tools")
                    .field(field)
                    .value(values)
                    .max_values(2)
                    .hide_picked_options(true)
                    .clearable(true)
                    .clear_label("Clear tools")
                    .placeholder("Search tools")
                    .group(
                        sapid::select_group("Runtime")
                            .option(select_option("rust", "Rust"))
                            .option(select_option("glory", "Glory")),
                    )
                    .group(sapid::select_group("Design").option(select_option("sapid", "Sapid")))
                    .on_value_change(move |proposal| values.revise(|current| *current = proposal))
            })
            .show_in(scope)
    })
}
