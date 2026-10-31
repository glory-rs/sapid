use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    focused::demo_for("P3-20", theme)
}

#[allow(dead_code, unused_imports)]
mod focused {
    use glory::prelude::*;
    use sapid::Theme;

    pub fn demo_for(id: &str, theme: Theme) -> Box<dyn Widget> {
        match id {
            "P3-11" => Box::new(
                sapid::stack().theme(theme.clone()).gap(18.0).fill((
                    sapid::card()
                        .theme(theme.clone())
                        .title("Project settings")
                        .description("A card groups related content and actions.")
                        .header(sapid::badge().theme(theme.clone()).content("Active"))
                        .fill(
                            sapid::text()
                                .theme(theme.clone())
                                .content("Manage your workspace preferences."),
                        )
                        .footer(
                            sapid::button()
                                .theme(theme.clone())
                                .size(sapid::Size::Small)
                                .label("Save changes"),
                        ),
                    sapid::paper()
                        .theme(theme.clone())
                        .bordered(true)
                        .shadow(sapid::ShadowSize::Small)
                        .fill(
                            sapid::text()
                                .theme(theme.clone())
                                .content("A lightweight paper surface"),
                        ),
                )),
            ),
            "P3-17" => Box::new(
                sapid::empty_state()
                    .theme(theme.clone())
                    .illustration(sapid::badge().theme(theme.clone()).content("Inbox"))
                    .title("No messages yet")
                    .description("Start a conversation to see it here.")
                    .actions(
                        sapid::button()
                            .theme(theme.clone())
                            .label("Compose message"),
                    ),
            ),
            "P3-20" => Box::new(
                sapid::stack().theme(theme.clone()).gap(12.0).fill((
                    sapid::number_formatter()
                        .theme(theme.clone())
                        .value(1234.5)
                        .min_fraction_digits(2)
                        .max_fraction_digits(2),
                    sapid::number_formatter()
                        .theme(theme.clone())
                        .value(1234.5)
                        .locale(sapid::NumberLocale::DeDe)
                        .format(sapid::NumberFormat::Currency(sapid::Currency::Eur)),
                    sapid::number_formatter()
                        .theme(theme.clone())
                        .value(0.125)
                        .format(sapid::NumberFormat::Percent)
                        .max_fraction_digits(1),
                )),
            ),
            _ => unreachable!("unsupported display demo"),
        }
    }
}
