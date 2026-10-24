use glory::prelude::*;
pub fn text_motion_form(theme: sapid::Theme) -> impl Widget {
    component(move |scope| {
        let paused = scope.cage(false);
        let animated = scope.cage(true);
        let short = scope.cage(false);
        let wide = scope.cage(false);
        let amount = scope.cage(1234.25);
        let content = scope.bond(move || {
            if short.get() {
                "Short status".to_owned()
            } else {
                "Latest updates: native and browser motion preserve stable accessible text"
                    .to_owned()
            }
        });
        glory::ui::view()
            .class("text-motion-demo")
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(16.),
            )
            .fill((
                sapid::marquee()
                    .theme(theme.clone())
                    .class("text-motion-left")
                    .text(content)
                    .period_millis(1200u32)
                    .paused(paused)
                    .animated(animated)
                    .style(wide.map(|wide| {
                        Style::new()
                            .width(Length::Px(if *wide { 640. } else { 180. }))
                            .max_width(glory::style::Limit::Percent(100.))
                    })),
                sapid::marquee()
                    .theme(theme.clone())
                    .class("text-motion-right")
                    .text("Reverse direction: current finite text moves to the right")
                    .direction(sapid::MarqueeDirection::Right)
                    .period_millis(1200u32)
                    .paused(paused)
                    .animated(animated)
                    .style(
                        Style::new()
                            .width(Length::Px(180.))
                            .max_width(glory::style::Limit::Percent(100.)),
                    ),
                sapid::rolling_number()
                    .theme(theme.clone())
                    .class("text-motion-amount")
                    .value(amount)
                    .min_fraction_digits(2)
                    .max_fraction_digits(2)
                    .duration_millis(800u16)
                    .paused(paused)
                    .animated(animated),
                sapid::rolling_number()
                    .theme(theme.clone())
                    .class("text-motion-euro")
                    .value(amount)
                    .locale(sapid::NumberLocale::DeDe)
                    .format(sapid::NumberFormat::Currency(sapid::Currency::Eur))
                    .duration_millis(800u16)
                    .paused(paused)
                    .animated(animated),
                sapid::text_field()
                    .theme(theme.clone())
                    .name("Text motion draft")
                    .default_value("Preserve this draft"),
                glory::ui::view()
                    .style(Style {
                        flex_wrap: Some(glory::style::FlexWrap::Wrap),
                        ..Style::new().display(Display::Flex).gap(8.)
                    })
                    .fill((
                        sapid::button()
                            .theme(theme.clone())
                            .label("Pause or resume text motion")
                            .on_press(move || paused.revise(|value| *value = !*value)),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Stop or start text motion")
                            .on_press(move || animated.revise(|value| *value = !*value)),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Change marquee text")
                            .on_press(move || short.revise(|value| *value = !*value)),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Change marquee viewport")
                            .on_press(move || wide.revise(|value| *value = !*value)),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Set amount 1298.75")
                            .on_press(move || amount.revise(|value| *value = 1298.75)),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Set amount minus nine")
                            .on_press(move || amount.revise(|value| *value = -9.)),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Set amount twelve")
                            .on_press(move || amount.revise(|value| *value = 12.)),
                    )),
            ))
            .show_in(scope)
    })
}
