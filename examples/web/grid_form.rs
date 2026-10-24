use glory::style::{Length, Style};
use glory::ui::{Widget, component, text};
use sapid::{Theme, aspect_ratio, button, grid, grid_item, simple_grid, text_field};
pub fn grid_form(theme: Theme) -> impl Widget {
    component(move |scope| {
        let ratio = scope.cage(2.0);
        let cell = || Style {
            background: Some(theme.accent_soft),
            ..Style::new().min_height(Length::Px(40.0))
        };
        sapid::stack()
            .gap(16.0)
            .class("grid-fixture")
            .fill((
                grid("layout-showcase")
                    .theme(theme.clone())
                    .name("Four track layout")
                    .class("grid-main")
                    .columns(4)
                    .gap(8.0)
                    .item(
                        grid_item("wide")
                            .span(2)
                            .class("grid-wide")
                            .style(cell())
                            .fill(text().content("Two-column card")),
                    )
                    .item(
                        grid_item("draft")
                            .span(2)
                            .class("grid-draft-cell")
                            .style(cell())
                            .fill(
                                text_field()
                                    .theme(theme.clone())
                                    .name("Grid notes")
                                    .class("grid-draft")
                                    .default_value("Keep draft")
                                    .style(
                                        Style::new()
                                            .width(Length::Percent(100.0))
                                            .min_width(Length::Px(0.0)),
                                    ),
                            ),
                    )
                    .item(
                        grid_item("small")
                            .class("grid-small")
                            .style(cell())
                            .fill(text().content("One column")),
                    )
                    .item(
                        grid_item("tail")
                            .span(3)
                            .class("grid-tail")
                            .style(cell())
                            .fill(text().content("Three-column card")),
                    ),
                simple_grid("equal-cards")
                    .theme(theme.clone())
                    .class("grid-simple")
                    .columns(3)
                    .gap(8.0)
                    .items((0..5).map(|index| {
                        grid_item(format!("cell-{index}"))
                            .style(cell())
                            .fill(text().content(format!("Card {}", index + 1)))
                    })),
                aspect_ratio(ratio)
                    .theme(theme.clone())
                    .class("grid-ratio")
                    .width(Length::Px(200.0))
                    .style(Style {
                        background: Some(theme.accent_soft),
                        ..Style::new()
                    })
                    .fill(text().content("Ratio panel")),
                aspect_ratio(2.0)
                    .theme(theme.clone())
                    .class("grid-fixed-ratio")
                    .width(Length::Px(120.0))
                    .height(Length::Px(40.0))
                    .style(Style {
                        background: Some(theme.accent_soft),
                        ..Style::new()
                    })
                    .fill(text().content("Fixed axes")),
                button()
                    .theme(theme.clone())
                    .label("Change ratio")
                    .on_press(move || {
                        ratio.revise(|ratio| *ratio = if *ratio == 2.0 { 1.0 } else { 2.0 })
                    }),
            ))
            .show_in(scope)
    })
}
