use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    focused::demo_for("P3-02", theme)
}

#[allow(dead_code, unused_imports)]
mod focused {
    use glory::prelude::*;
    use sapid::Theme;

    fn tile(theme: &Theme, label: &str) -> sapid::Box {
        sapid::box_()
            .theme(theme.clone())
            .style(Style {
                background: Some(theme.accent_soft),
                color: Some(theme.accent_text),
                ..Style::new().padding(Edges::all(16.0)).radius(theme.radius)
            })
            .fill(text().content(label.to_owned()))
    }

    pub fn demo_for(id: &str, theme: Theme) -> Box<dyn Widget> {
        match id {
            "P3-02" => Box::new(
                sapid::container()
                    .theme(theme.clone())
                    .max_width(Limit::Px(420.0))
                    .fill(
                        sapid::stack().theme(theme.clone()).fill((
                            sapid::center()
                                .theme(theme.clone())
                                .height(Length::Px(140.0))
                                .fill(tile(&theme, "Centered content")),
                            sapid::space().theme(theme.clone()).height(Length::Px(20.0)),
                            sapid::text()
                                .theme(theme.clone())
                                .content("A bounded container with intentional space."),
                        )),
                    ),
            ),
            "P3-03" => Box::new(
                sapid::stack().theme(theme.clone()).gap(20.0).fill((
                    sapid::grid("focused-grid")
                        .theme(theme.clone())
                        .columns(2)
                        .gap(12.0)
                        .items((1..=4).map(|index| {
                            sapid::grid_item(format!("grid-{index}"))
                                .theme(theme.clone())
                                .fill(tile(&theme, &format!("Grid {index}")))
                        })),
                    sapid::simple_grid("focused-simple-grid")
                        .theme(theme.clone())
                        .columns(3)
                        .gap(8.0)
                        .items((1..=3).map(|index| {
                            sapid::grid_item(format!("simple-{index}"))
                                .theme(theme.clone())
                                .fill(tile(&theme, &format!("Simple {index}")))
                        })),
                )),
            ),
            _ => unreachable!("unsupported responsive demo"),
        }
    }
}
