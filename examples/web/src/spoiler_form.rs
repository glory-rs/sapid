use glory::prelude::*;
pub fn spoiler_form(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| {
        let long="Glory keeps semantic content stable while the host measures wrapping. This paragraph remains selectable in full. ".repeat(16);
        let content = scope.cage(long.clone());
        let open = scope.cage(false);
        let proposals = scope.cage(0_u32);
        let status = scope.bond(move || format!("Expansion proposals: {}", proposals.get()));
        sapid::stack()
            .class("spoiler-fixture")
            .gap(18.0)
            .fill((
                sapid::spoiler("local-spoiler")
                    .theme(theme.clone())
                    .class("local-spoiler")
                    .content(content)
                    .max_height(80.0),
                sapid::spoiler("controlled-spoiler")
                    .theme(theme.clone())
                    .class("controlled-spoiler")
                    .content(content)
                    .max_height(60.0)
                    .expanded(open)
                    .on_expanded_change(move |_| proposals.revise(|value| *value += 1)),
                text().class("spoiler-status").content(status),
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle expansion source")
                    .on_press(move || open.revise(|value| *value = !*value)),
                sapid::button()
                    .theme(theme.clone())
                    .label("Shorten text")
                    .on_press(move || content.revise(|text| *text = "Short content".into())),
                sapid::button()
                    .theme(theme.clone())
                    .label("Restore long text")
                    .on_press(move || content.revise(|text| *text = long.clone())),
            ))
            .show_in(scope)
    })
}
