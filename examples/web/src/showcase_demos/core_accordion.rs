use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    sapid::accordion("showcase-faq")
        .theme(theme.clone())
        .default_open(["shipping"])
        .item(
            sapid::accordion_item("shipping")
                .label("How long does shipping take?")
                .content({
                    let theme = theme.clone();
                    move || {
                        sapid::text()
                            .theme(theme.clone())
                            .content("Orders usually arrive within 3–5 business days.")
                    }
                }),
        )
        .item(
            sapid::accordion_item("returns")
                .label("What is your return policy?")
                .content({
                    let theme = theme.clone();
                    move || {
                        sapid::text()
                            .theme(theme.clone())
                            .content("You can request a return within 30 days.")
                    }
                }),
        )
        .item(
            sapid::accordion_item("support")
                .label("How can I contact support?")
                .content(move || {
                    sapid::text()
                        .theme(theme.clone())
                        .content("Send a message to the support team from your account.")
                }),
        )
}
