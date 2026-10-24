use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    sapid::dialog("showcase-dialog")
        .theme(theme.clone())
        .label("Open modal")
        .title("Edit your profile")
        .description("Update your display name and close the dialog when finished.")
        .content(move || {
            sapid::field("showcase-profile")
                .theme(theme.clone())
                .label("Display name")
                .control({
                    let theme = theme.clone();
                    move |field| {
                        sapid::text_field()
                            .theme(theme.clone())
                            .field(field)
                            .default_value("Alex Morgan")
                    }
                })
        })
}
