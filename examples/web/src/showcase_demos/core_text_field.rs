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
            .show_in(scope)
    })
}
