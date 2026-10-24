//! Shared display fixture: include from the fixture application and SSR path.
use glory::ui::{Widget, text, widget};
use sapid::{BadgeVariant, Theme, badge, button, card, empty_state, paper, pill};

pub fn display_form(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let visible = scope.cage(true);
        let proposals = scope.cage(0_u32);
        let proposals_text =
            scope.bond(move || format!("Remove proposals: {}", proposals.get()));
        card()
            .theme(theme.clone())
            .class("display-fixture")
            .title("Display widgets")
            .description("Passive tags and surfaces; actions remain native controls.")
            .header(
                sapid::group().fill((
                    badge().theme(theme.clone()).content("Soft <tag>"),
                    badge()
                        .theme(theme.clone())
                        .variant(BadgeVariant::Solid)
                        .content("Solid"),
                    badge()
                        .theme(theme.clone())
                        .variant(BadgeVariant::Outline)
                        .content("Outline"),
                )),
            )
            .fill((
                pill()
                    .theme(theme.clone())
                    .class("local-pill")
                    .content("Local tag")
                    .removable("Remove local tag"),
                pill()
                    .theme(theme.clone())
                    .class("controlled-pill")
                    .content("Controlled tag")
                    .removable("Request controlled removal")
                    .visible(visible)
                    .on_remove(move || proposals.revise(|value| *value += 1)),
                button()
                    .theme(theme.clone())
                    .label("Toggle controlled visibility")
                    .on_press(move || visible.revise(|value| *value = !*value)),
                text().class("display-proposals").content(proposals_text),
                paper()
                    .theme(theme.clone())
                    .shadow(sapid::ShadowSize::Medium)
                    .bordered(true)
                    .fill(text().content("Paper surface")),
                empty_state()
                    .theme(theme.clone())
                    .title("No messages")
                    .description("Start a conversation")
                    .illustration(text().content("Inbox"))
                    .actions(button().theme(theme.clone()).label("Compose")),
                badge().unstyled().content("Unstyled tag"),
                pill()
                    .theme(theme.clone())
                    .class("disabled-pill")
                    .content("Disabled tag")
                    .removable("Remove disabled tag")
                    .enabled(false),
                sapid::stack().gap(4.0).fill((
                    sapid::number_formatter()
                        .theme(theme.clone())
                        .class("number-us")
                        .value(1234.5)
                        .min_fraction_digits(2)
                        .max_fraction_digits(2),
                    sapid::number_formatter()
                        .theme(theme.clone())
                        .class("number-eur")
                        .value(1234.5)
                        .locale(sapid::NumberLocale::DeDe)
                        .format(sapid::NumberFormat::Currency(sapid::Currency::Eur)),
                    sapid::number_formatter()
                        .theme(theme.clone())
                        .class("number-percent")
                        .value(0.125)
                        .format(sapid::NumberFormat::Percent)
                        .max_fraction_digits(1),
                )),
            ))
            .footer(text().content("Footer <content>"))
            .show_in(scope)
    })
}
