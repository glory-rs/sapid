use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    sapid::tabs("showcase-tabs")
        .theme(theme.clone())
        .name("Account settings")
        .default_value("gallery")
        .tab(sapid::tab("gallery").label("Gallery").content({
            let theme = theme.clone();
            move || {
                sapid::text()
                    .theme(theme.clone())
                    .content("Gallery tab content")
            }
        }))
        .tab(sapid::tab("messages").label("Messages").content({
            let theme = theme.clone();
            move || {
                sapid::text()
                    .theme(theme.clone())
                    .content("Messages tab content")
            }
        }))
        .tab(sapid::tab("settings").label("Settings").content(move || {
            sapid::text()
                .theme(theme.clone())
                .content("Settings tab content")
        }))
}
