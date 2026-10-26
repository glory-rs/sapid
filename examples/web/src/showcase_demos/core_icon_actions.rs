use glory::prelude::*;
use sapid::{ActionIconVariant, Size, Theme};

pub fn demo(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let enabled = scope.cage(true);
        let activations = scope.cage(0usize);
        let variants = [
            ("Filled", ActionIconVariant::Filled),
            ("Light", ActionIconVariant::Light),
            ("Outline", ActionIconVariant::Outline),
            ("Subtle", ActionIconVariant::Subtle),
            ("Default", ActionIconVariant::Default),
        ]
        .into_iter()
        .map(|(name, variant)| {
            sapid::stack()
                .theme(theme.clone())
                .gap(8.0)
                .style(Style {
                    align: Some(Align::Center),
                    ..Style::new()
                })
                .fill((
                    sapid::action_icon(format!("{name} icon action"))
                        .theme(theme.clone())
                        .variant(variant)
                        .enabled(enabled)
                        .icon(text().content("↻").selectable(false))
                        .on_press(move || activations.revise(|count| *count += 1)),
                    sapid::text()
                        .theme(theme.clone())
                        .variant(sapid::TextVariant::Caption)
                        .content(name),
                ))
        })
        .collect::<Vec<_>>();
        let sizes = [
            Size::ExtraSmall,
            Size::Small,
            Size::Medium,
            Size::Large,
            Size::ExtraLarge,
        ]
        .into_iter()
        .enumerate()
        .map(|(index, size)| {
            sapid::close_button()
                .theme(theme.clone())
                .size(size)
                .name(format!("Close size {}", index + 1))
                .enabled(enabled)
                .on_press(move || activations.revise(|count| *count += 1))
        })
        .collect::<Vec<_>>();
        sapid::stack()
            .theme(theme.clone())
            .gap(24.0)
            .fill((
                sapid::stack().theme(theme.clone()).gap(12.0).fill((
                    sapid::text()
                        .theme(theme.clone())
                        .content("Icon action variants"),
                    sapid::group().theme(theme.clone()).gap(24.0).fill(variants),
                )),
                sapid::stack().theme(theme.clone()).gap(10.0).fill((
                    sapid::text()
                        .theme(theme.clone())
                        .content("Close button sizes"),
                    sapid::group().theme(theme.clone()).fill(sizes),
                )),
                sapid::group().theme(theme.clone()).fill((
                    sapid::checkbox()
                        .theme(theme.clone())
                        .label("Disabled")
                        .checked(enabled.map(|value| !value))
                        .on_checked_change(move |value| {
                            enabled.revise(|current| *current = !value)
                        }),
                    sapid::text()
                        .theme(theme.clone())
                        .variant(sapid::TextVariant::Small)
                        .content(activations.map(|count| format!("Activated {count} times"))),
                )),
            ))
            .show_in(scope)
    })
}
