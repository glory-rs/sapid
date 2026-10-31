use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    focused::demo_for("P3-12", theme)
}

#[allow(dead_code, unused_imports)]
mod focused {
    use glory::prelude::*;
    use sapid::Theme;

    pub fn demo_for(id: &str, theme: Theme) -> Box<dyn Widget> {
        match id {
            "P3-12" => Box::new(
                sapid::stack().theme(theme.clone()).gap(14.0).fill((
                    sapid::text().theme(theme.clone()).content("Account"),
                    sapid::separator()
                        .theme(theme.clone())
                        .name("Account details")
                        .length(Length::Percent(100.0)),
                    sapid::group().theme(theme.clone()).fill((
                        sapid::text().theme(theme.clone()).content("Profile"),
                        sapid::separator()
                            .theme(theme.clone())
                            .decorative(true)
                            .orientation(glory::render::Orientation::Vertical)
                            .length(Length::Px(24.0)),
                        sapid::text().theme(theme.clone()).content("Preferences"),
                    )),
                )),
            ),
            "P3-19" => Box::new(
                sapid::timeline("focused-release")
                    .theme(theme.clone())
                    .name("Release progress")
                    .active("build")
                    .item(
                        sapid::timeline_item("design")
                            .title("Design")
                            .timestamp("October 22")
                            .description("Agree on the semantic contract"),
                    )
                    .item(
                        sapid::timeline_item("build")
                            .title("Build")
                            .timestamp("October 23")
                            .description("Implement and verify widgets"),
                    )
                    .item(
                        sapid::timeline_item("ship")
                            .title("Ship")
                            .timestamp("October 24")
                            .description("Publish the release"),
                    ),
            ),
            _ => unreachable!("unsupported timeline demo"),
        }
    }
}
