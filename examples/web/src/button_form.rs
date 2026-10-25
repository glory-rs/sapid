use glory::prelude::*;
pub fn button_form(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| {
        let loading = scope.cage(false);
        let label = scope.cage("Save draft".to_string());
        let order = scope.cage(String::new());
        let typed = order;
        let generic = order;
        sapid::stack()
            .theme(theme.clone())
            .class("button-enhancements-fixture")
            .gap(14.0)
            .fill((
                sapid::group().theme(theme.clone()).fill((
                    sapid::button()
                        .theme(theme.clone())
                        .size(sapid::Size::ExtraSmall)
                        .label("Extra small")
                        .class("button-xs"),
                    sapid::button()
                        .theme(theme.clone())
                        .size(sapid::Size::ExtraLarge)
                        .label("Extra large")
                        .class("button-xl"),
                )),
                sapid::button()
                    .theme(theme.clone())
                    .class("section-button")
                    .label(label)
                    .left_section(text().content("★"))
                    .right_section(text().content("Ctrl+S"))
                    .loading(loading)
                    .on(glory::render::EventKind::Activate, move |_| {
                        generic.revise(|value| value.push_str("generic;"))
                    })
                    .on_press(move || typed.revise(|value| value.push_str("typed;"))),
                text().class("button-order").content(order),
                sapid::button()
                    .theme(theme.clone())
                    .class("section-unstyled")
                    .unstyled()
                    .label("Plain action")
                    .left_section(text().content("+")),
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle button loading")
                    .on_press(move || loading.revise(|value| *value = !*value)),
                sapid::button()
                    .theme(theme.clone())
                    .label("Update button label")
                    .on_press(move || label.revise(|value| *value = "Save changes".into())),
            ))
            .show_in(scope)
    })
}
