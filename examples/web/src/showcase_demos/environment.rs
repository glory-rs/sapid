use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::environment_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::render::{Direction, MotionPreference, Orientation};
    use glory::style::{Display, FlexDirection, Length, Style};
    use glory::ui::Widget;
    use sapid::{
        Locale, LocaleMessage, Size, Theme, ThemeTokens, TypographyMetrics, TypographyRole,
    };

    pub fn environment_form(theme: Theme) -> impl Widget {
        let tokens = ThemeTokens::default()
            .with_spacing(Size::Medium, 24.)
            .unwrap()
            .with_control(
                Size::Medium,
                sapid::ControlMetrics {
                    min_height: 48.,
                    padding_x: 18.,
                    padding_y: 8.,
                    radius_scale: 2.,
                },
            )
            .unwrap()
            .with_typography(
                TypographyRole::Control,
                Size::Medium,
                TypographyMetrics {
                    font_size: 19.,
                    line_height: 28.,
                    weight: Some(500.),
                },
            )
            .unwrap()
            .with_typography(
                TypographyRole::Heading2,
                Size::Medium,
                TypographyMetrics {
                    font_size: 35.,
                    line_height: 43.,
                    weight: Some(650.),
                },
            )
            .unwrap();
        sapid::theme_provider(theme.with_tokens(tokens))
            .direction(Direction::Rtl)
            .locale(Locale::arabic())
            .motion(MotionPreference::Reduce)
            .class("environment-provider")
            .style(Style::new().padding(glory::style::Edges::all(20.)))
            .content(|context| {
                let theme = context.theme.clone();
                let nested = context
                    .provider()
                    .direction(Direction::Ltr)
                    .locale(Locale::chinese())
                    .class("environment-nested")
                    .content(|context| {
                        sapid::button()
                            .theme(context.theme)
                            .label(context.locale.message(LocaleMessage::Close))
                            .class("environment-localized")
                    });
                let row = glory::ui::view()
                    .class("environment-row")
                    .style(Style {
                        display: Some(Display::Flex),
                        flex_direction: Some(FlexDirection::Row),
                        gap: Some(context.spacing(Size::Medium)),
                        ..Default::default()
                    })
                    .fill((
                        sapid::button()
                            .theme(theme.clone())
                            .label(context.message(LocaleMessage::Open))
                            .class("environment-first"),
                        sapid::button()
                            .theme(theme.clone())
                            .label(context.message(LocaleMessage::Next))
                            .class("environment-second"),
                    ));
                let duration = context.transition_duration(250).unwrap();
                sapid::stack()
                    .theme(theme.clone())
                    .gap(context.spacing(Size::Medium))
                    .fill((
                        sapid::title(2)
                            .theme(theme.clone())
                            .content("Environment tokens")
                            .class("environment-heading"),
                        row,
                        sapid::text_field()
                            .theme(theme.clone())
                            .name("Environment draft")
                            .default_value("Draft 🙂")
                            .style(Style::new().width(Length::Px(260.))),
                        sapid::radio_group("environment-navigation")
                            .theme(theme.clone())
                            .label("Explicit RTL navigation")
                            .default_value("first")
                            .orientation(Orientation::Horizontal)
                            .direction(context.direction)
                            .items([
                                sapid::radio("first").label("أول"),
                                sapid::radio("second").label("ثاني"),
                                sapid::radio("third").label("ثالث"),
                            ]),
                        nested,
                        sapid::collapsible("environment-motion")
                            .theme(theme.clone())
                            .height_transition(duration as u16)
                            .default_open(true)
                            .content({
                                let theme = theme.clone();
                                move || {
                                    sapid::text()
                                        .theme(theme.clone())
                                        .content("Reduced motion content")
                                }
                            }),
                    ))
            })
    }
}
