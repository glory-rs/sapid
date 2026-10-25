use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::spinner_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::ui::{Widget, widget};
    use sapid::{Theme, button, spinner};
    pub fn spinner_form(theme: Theme) -> impl Widget {
        widget(move |scope| {
            let visible = scope.cage(true);
            let animated = scope.cage(true);
            sapid::stack()
                .theme(theme.clone())
                .gap(20.)
                .fill((
                    sapid::group().theme(theme.clone()).gap(24.).fill((
                        spinner()
                            .theme(theme.clone())
                            .name("Loading files")
                            .diameter(48.)
                            .thickness(6.)
                            .visible(visible)
                            .animated(animated)
                            .class("spinner-controlled"),
                        sapid::loader()
                            .theme(theme.clone())
                            .decorative()
                            .diameter(32.)
                            .class("spinner-decorative"),
                        spinner()
                            .theme(theme.clone())
                            .unstyled()
                            .name("Unstyled loading")
                            .diameter(40.)
                            .class("spinner-unstyled"),
                    )),
                    button()
                        .theme(theme.clone())
                        .label("Toggle spinner animation")
                        .on_press(move || animated.revise(|value| *value = !*value)),
                    button()
                        .theme(theme.clone())
                        .label("Toggle spinner visibility")
                        .on_press(move || visible.revise(|value| *value = !*value)),
                ))
                .show_in(scope)
        })
    }
}
