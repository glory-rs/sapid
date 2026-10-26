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
        let value = scope.cage(String::new());
        let hint = scope.cage("Ada Lovelace".to_owned());
        column()
            .fill((
                sapid::field("showcase-name")
                    .theme(theme.clone())
                    .label("Your name")
                    .description("This value is controlled by a Cage.")
                    .control({
                        let theme = theme.clone();
                        move |field| {
                            sapid::text_field()
                                .theme(theme.clone())
                                .field(field)
                                .value(value)
                                .placeholder(hint)
                                .on_value_change(move |next| value.revise(|v| *v = next))
                        }
                    }),
                sapid::text().theme(theme.clone()).content(value.map(|v| {
                    if v.is_empty() {
                        "Start typing above".to_owned()
                    } else {
                        format!("Hello, {v}!")
                    }
                })),
            ))
            .fill(
                sapid::group().theme(theme.clone()).fill(
                    sapid::button()
                        .theme(theme.clone())
                        .variant(sapid::ButtonVariant::Subtle)
                        .size(sapid::Size::Small)
                        .label("Change hint")
                        .on_press(move || {
                            hint.revise(|value| {
                                *value = if value == "Ada Lovelace" {
                                    "Grace Hopper"
                                } else {
                                    "Ada Lovelace"
                                }
                                .to_owned()
                            })
                        }),
                ),
            )
            .show_in(scope)
    })
}
