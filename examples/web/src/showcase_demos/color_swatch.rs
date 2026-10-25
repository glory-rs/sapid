use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::color_swatch_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use glory::style::Color;
    pub fn color_swatch_form(theme: sapid::Theme) -> impl Widget {
        widget(move |scope| {
            let color = scope.cage(Color(255, 0, 0, 128));
            let selected = scope.cage(false);
            let accept = scope.cage(false);
            let enabled = scope.cage(true);
            let requested = scope.cage(false);
            sapid::stack()
                .theme(theme.clone())
                .gap(16.)
                .fill((
                    sapid::group().theme(theme.clone()).fill((
                        sapid::color_swatch(Color(255, 0, 0, 128))
                            .theme(theme.clone())
                            .name("Half transparent red")
                            .class("passive-red"),
                        sapid::color_swatch(Color(0, 0, 0, 0))
                            .theme(theme.clone())
                            .name("Fully transparent")
                            .class("passive-clear"),
                        sapid::color_swatch(Color(0, 100, 255, 255))
                            .theme(theme.clone())
                            .name("Round blue")
                            .round(true)
                            .side(40.),
                        sapid::color_swatch(Color(0, 0, 0, 0))
                            .unstyled()
                            .name("Unstyled transparent")
                            .side(40.)
                            .round(true),
                    )),
                    sapid::group().theme(theme.clone()).fill((
                        sapid::color_swatch(color)
                            .theme(theme.clone())
                            .class("controlled-swatch")
                            .pressed(selected)
                            .enabled(enabled)
                            .on_pressed_change(move |value| {
                                requested.revise(|v| *v = value);
                                if accept.get() {
                                    selected.revise(|v| *v = value)
                                }
                            }),
                        sapid::color_swatch(Color(30, 150, 60, 255))
                            .theme(theme.clone())
                            .name("Local green selection")
                            .default_pressed(false),
                    )),
                    glory::ui::text()
                        .class("swatch-proposal")
                        .content(scope.bond(move || format!("Requested: {}", requested.get()))),
                    sapid::group().theme(theme.clone()).fill((
                        sapid::button()
                            .theme(theme.clone())
                            .label("Accept swatch selection")
                            .on_press(move || accept.revise(|v| *v = true)),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Toggle swatch enabled")
                            .on_press(move || enabled.revise(|v| *v = !*v)),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Change swatch color")
                            .on_press(move || color.revise(|v| *v = Color(0, 0, 255, 128))),
                    )),
                    sapid::text_field()
                        .theme(theme.clone())
                        .name("Outside swatch editor")
                        .default_value("Retained color draft"),
                ))
                .show_in(scope)
        })
    }
}
