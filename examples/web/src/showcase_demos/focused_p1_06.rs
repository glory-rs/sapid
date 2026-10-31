use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    focused::demo_for("P1-06", theme)
}

#[allow(dead_code, unused_imports)]
mod focused {
    use glory::prelude::*;
    use sapid::Theme;

    pub fn demo_for(id: &str, theme: Theme) -> Box<dyn Widget> {
        match id {
        "P1-01" => Box::new(
            sapid::field("focused-field-email")
                .theme(theme.clone())
                .label("Email address")
                .description("We will only use this for important account updates.")
                .required(true)
                .control(|field| {
                    sapid::text_field()
                        .field(field)
                        .default_value("alex@studio.design")
                }),
        ),
        "P1-02" => Box::new(
            sapid::field("focused-field-parts")
                .theme(theme.clone())
                .label("Work email")
                .description("Use the address associated with your workspace.")
                .error("Enter a valid email address, including the @ sign.")
                .required(true)
                .control(|field| sapid::text_field().field(field).default_value("alex@studio")),
        ),
        "P1-03" => Box::new(widget(move |scope| {
            let disabled = scope.cage(false);
            sapid::stack()
                .theme(theme.clone())
                .gap(16.0)
                .fill((
                    sapid::fieldset("focused-profile")
                        .theme(theme.clone())
                        .legend("Personal information")
                        .disabled(disabled)
                        .fill((
                            sapid::field("focused-profile-name")
                                .theme(theme.clone())
                                .label("Your name")
                                .control(|field| {
                                    sapid::text_field().field(field).default_value("Ada Lovelace")
                                }),
                            sapid::field("focused-profile-email")
                                .theme(theme.clone())
                                .label("Email")
                                .control(|field| {
                                    sapid::text_field()
                                        .field(field)
                                        .default_value("ada@example.com")
                                }),
                        )),
                    sapid::group().theme(theme.clone()).fill(
                        sapid::button()
                            .theme(theme.clone())
                            .variant(sapid::ButtonVariant::Secondary)
                            .size(sapid::Size::Small)
                            .label("Toggle fieldset disabled")
                            .on_press(move || disabled.revise(|value| *value = !*value)),
                    ),
                ))
                .show_in(scope)
        })),
        "P1-04" => Box::new(
            sapid::field("focused-biography")
                .theme(theme.clone())
                .label("Biography")
                .description("Autosizes from three to six lines as you write.")
                .control(|field| {
                    sapid::textarea()
                        .field(field)
                        .autosize(true)
                        .min_rows(3)
                        .max_rows(6)
                        .default_value(
                            "Tell us a little about yourself.\nThis textarea grows with its content.",
                        )
                }),
        ),
        "P1-05" => Box::new(
            sapid::field("focused-password")
                .theme(theme.clone())
                .label("Account password")
                .description("Use the visibility action without losing your input.")
                .required(true)
                .control(|field| {
                    sapid::password_input()
                        .field(field)
                        .default_value("example-password")
                }),
        ),
        "P1-06" => Box::new(
            sapid::field("focused-website")
                .theme(theme.clone())
                .label("Website")
                .description("Addons and actions stay aligned with the input.")
                .control(move |field| {
                    sapid::input_group()
                        .theme(theme.clone())
                        .start(sapid::input_addon().theme(theme.clone()).content("https://"))
                        .input(sapid::text_field().field(field).default_value("studio.design"))
                        .end(
                            sapid::input_section().theme(theme.clone()).fill(
                                sapid::button()
                                    .theme(theme.clone())
                                    .variant(sapid::ButtonVariant::Secondary)
                                    .size(sapid::Size::Small)
                                    .label("Check"),
                            ),
                        )
                }),
        ),
        _ => unreachable!("unsupported fields demo"),
    }
    }
}
