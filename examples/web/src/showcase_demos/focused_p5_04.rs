use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    focused::demo_for("P5-04", theme)
}

#[allow(dead_code, unused_imports)]
mod focused {
    use glory::prelude::*;
    use sapid::Theme;

    pub fn demo_for(id: &str, theme: Theme) -> Box<dyn Widget> {
        match id {
            "P5-03" => Box::new(
                sapid::tooltip("focused-save-tip")
                    .theme(theme.clone())
                    .label("Save draft")
                    .text("Your draft is saved privately until you publish it."),
            ),
            "P5-04" => Box::new(
                sapid::hover_card("focused-profile-card")
                    .theme(theme.clone())
                    .label("About Ada")
                    .name("Ada profile")
                    .content(move || {
                        sapid::stack().theme(theme.clone()).gap(10.0).fill((
                            sapid::text()
                                .theme(theme.clone())
                                .content("Ada builds thoughtful developer tools."),
                            sapid::anchor()
                                .theme(theme.clone())
                                .label("Read Ada's profile")
                                .href("#ada"),
                        ))
                    }),
            ),
            _ => unreachable!("unsupported hints demo"),
        }
    }
}
