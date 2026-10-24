use glory::style::{Edges, Length, Style};
use glory::ui::{Widget, widget};
use sapid::{Icon, IndicatorPosition, Theme, ThemeIconVariant, button, indicator, theme_icon};
pub fn graphics_form(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let visible = scope.cage(true);
        let count = scope.cage("3".to_string());
        let icon = scope.cage(Icon::Check);
        let name = scope.cage("Completed".to_string());
        sapid::stack()
            .gap(24.0)
            .fill((
                sapid::group()
                    .gap(20.0)
                    .style(Style::new().padding(Edges::all(16.0)))
                    .fill((
                        indicator()
                            .theme(theme.clone())
                            .visible(visible)
                            .label(count)
                            .diameter(20.0)
                            .name("Unread messages")
                            .class("graphics-indicator")
                            .fill(button().theme(theme.clone()).label("Inbox")),
                        indicator()
                            .theme(theme.clone())
                            .position(IndicatorPosition::BottomLeft)
                            .class("graphics-dot")
                            .fill(glory::ui::text().content("Presence dot")),
                        indicator()
                            .unstyled()
                            .class("graphics-indicator-unstyled")
                            .fill(glory::ui::text().content("Unstyled marker")),
                    )),
                sapid::group().gap(12.0).fill((
                    theme_icon(icon)
                        .name(name)
                        .theme(theme.clone())
                        .class("graphics-dynamic"),
                    theme_icon(Icon::Info)
                        .name("Information")
                        .variant(ThemeIconVariant::Outline)
                        .theme(theme.clone())
                        .class("graphics-info"),
                    theme_icon(Icon::Close)
                        .theme(theme.clone())
                        .variant(ThemeIconVariant::Solid)
                        .class("graphics-decorative"),
                    theme_icon(Icon::ArrowRight)
                        .theme(theme.clone())
                        .unstyled()
                        .class("graphics-unstyled"),
                )),
                button()
                    .theme(theme.clone())
                    .label("Update markers")
                    .style(Style::new().width(Length::Px(180.0)))
                    .on_press(move || {
                        count.revise(|value| *value = "4".into());
                        icon.revise(|value| *value = Icon::Close);
                        name.revise(|value| *value = "Closed".into());
                    }),
                button()
                    .theme(theme.clone())
                    .label("Toggle marker")
                    .style(Style::new().width(Length::Px(180.0)))
                    .on_press(move || visible.revise(|value| *value = !*value)),
            ))
            .show_in(scope)
    })
}
