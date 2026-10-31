use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    focused::demo_for("P6-10", theme)
}

#[allow(dead_code, unused_imports)]
mod focused {
    use glory::prelude::*;
    use sapid::Theme;

    pub fn demo_for(id: &str, theme: Theme) -> Box<dyn Widget> {
        match id {
            "P2-07" => Box::new(
                sapid::button_group()
                    .theme(theme.clone())
                    .name("Text alignment")
                    .buttons([
                        sapid::button()
                            .theme(theme.clone())
                            .variant(sapid::ButtonVariant::Secondary)
                            .label("Left"),
                        sapid::button()
                            .theme(theme.clone())
                            .variant(sapid::ButtonVariant::Secondary)
                            .label("Center"),
                        sapid::button()
                            .theme(theme.clone())
                            .variant(sapid::ButtonVariant::Secondary)
                            .label("Right"),
                    ]),
            ),
            "P6-10" => Box::new(
                sapid::stack().theme(theme.clone()).gap(20.0).fill((
                    sapid::toolbar("focused-editing")
                        .theme(theme.clone())
                        .name("Editing tools")
                        .group(
                            sapid::toolbar_group("focused-clipboard")
                                .name("Clipboard")
                                .button(sapid::button().label("Cut"))
                                .button(sapid::button().label("Copy")),
                        )
                        .group(
                            sapid::toolbar_group("focused-history")
                                .name("History")
                                .button(sapid::button().label("Undo"))
                                .button(sapid::button().label("Redo")),
                        ),
                    sapid::action_bar("focused-selection")
                        .theme(theme.clone())
                        .name("Selection actions")
                        .selected_count(3)
                        .group(
                            sapid::toolbar_group("focused-bulk")
                                .name("Bulk operations")
                                .button(sapid::button().label("Archive"))
                                .button(sapid::button().label("Delete")),
                        ),
                )),
            ),
            _ => unreachable!("unsupported toolbar demo"),
        }
    }
}
