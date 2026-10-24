use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    sapid::popover("showcase-popover")
        .theme(theme.clone())
        .label("Toggle popover")
        .title("Popover title")
        .content(move || {
            sapid::text()
                .theme(theme.clone())
                .content("A popover keeps additional information beside its trigger.")
        })
}
