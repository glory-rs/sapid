use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    sapid::collapsible("showcase-details")
        .theme(theme.clone())
        .label("Show more information")
        .content(move || {
            sapid::text()
                .theme(theme.clone())
                .content("Build accessible interfaces with handwritten Rust builders.")
        })
}
