use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    sapid::drawer("showcase-drawer")
        .theme(theme.clone())
        .label("Open drawer")
        .title("Your workspace")
        .content(move || {
            sapid::text()
                .theme(theme.clone())
                .content("Keep related information and actions within reach.")
        })
}
