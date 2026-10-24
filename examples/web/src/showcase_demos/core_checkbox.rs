use glory::prelude::*;

fn column() -> glory::ui::View {
    col().style(
        Style::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(16.0),
    )
}

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| {
        let checked = scope.cage(false);
        column()
            .fill((
                sapid::checkbox()
                    .theme(theme.clone())
                    .label("I agree to the terms and conditions")
                    .checked(checked)
                    .on_checked_change(move |next| checked.revise(|v| *v = next)),
                sapid::checkbox()
                    .theme(theme.clone())
                    .label("Disabled checkbox")
                    .enabled(false),
                sapid::text().theme(theme.clone()).content(checked.map(|v| {
                    if *v {
                        "Thank you for confirming."
                    } else {
                        "Your confirmation is optional in this demo."
                    }
                    .to_owned()
                })),
            ))
            .show_in(scope)
    })
}
