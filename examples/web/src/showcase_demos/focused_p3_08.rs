use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    focused::demo_for("P3-08", theme)
}

#[allow(dead_code, unused_imports)]
mod focused {
    use glory::prelude::*;
    use sapid::{KbdKey, KbdPlatform, Theme};

    pub fn demo_for(id: &str, theme: Theme) -> Box<dyn Widget> {
        match id {
            "P3-07" => {
                Box::new(
                    sapid::text_list()
                        .theme(theme.clone())
                        .ordered(true)
                        .name("Getting started")
                        .item(sapid::text_list_item().fill(text().content("Create an application")))
                        .item(sapid::text_list_item().fill((
                            text().content("Compose widgets"),
                            sapid::text_list().theme(theme.clone()).item(
                                sapid::text_list_item().fill(text().content("Keep keys stable")),
                            ),
                        )))
                        .item(sapid::text_list_item().fill(text().content("Choose a host"))),
                )
            }
            "P3-08" => Box::new(
                sapid::blockquote()
                    .theme(theme.clone())
                    .fill(text().content(
                        "Make interfaces simpler by giving each piece of UI a clear semantic role.",
                    ))
                    .citation(text().content("Sapid design principles")),
            ),
            "P3-09" => Box::new(
                sapid::stack().theme(theme.clone()).gap(18.0).fill((
                    sapid::text()
                        .theme(theme.clone())
                        .content("Inline code: let glory = 42;"),
                    sapid::code()
                        .theme(theme.clone())
                        .content("let glory = 42;"),
                    sapid::mark().theme(theme.clone()).content("Marked text"),
                    sapid::highlight()
                        .theme(theme.clone())
                        .content("Highlight only the important word in this sentence.")
                        .query("important"),
                )),
            ),
            "P3-16" => Box::new(
                sapid::group().theme(theme.clone()).fill((
                    sapid::kbd()
                        .theme(theme.clone())
                        .platform(KbdPlatform::Mac)
                        .key(KbdKey::Meta),
                    sapid::text().theme(theme.clone()).content("+"),
                    sapid::kbd().theme(theme.clone()).content("K"),
                    sapid::text().theme(theme.clone()).content("to open search"),
                )),
            ),
            "P3-18" => Box::new(
                sapid::stack().theme(theme.clone()).gap(20.0).fill((
                    sapid::item()
                        .theme(theme.clone())
                        .title("Sapid")
                        .description("Composable semantic widgets for Glory")
                        .media(sapid::badge().theme(theme.clone()).content("RS")),
                    sapid::data_list()
                        .theme(theme.clone())
                        .name("Project metadata")
                        .item(
                            sapid::data_list_item()
                                .term("Package")
                                .fill(text().content("sapid")),
                        )
                        .item(
                            sapid::data_list_item()
                                .term("Host")
                                .fill(text().content("Web, SSR and native")),
                        ),
                )),
            ),
            _ => unreachable!("unsupported document demo"),
        }
    }
}
