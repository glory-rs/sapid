use glory::prelude::*;
use glory::style::FlexWrap;

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
        let clicks = scope.cage(0usize);
        let variant = scope.cage(Some("filled".to_owned()));
        let size = scope.cage(Some("medium".to_owned()));
        let disabled = scope.cage(false);
        let loading = scope.cage(false);
        let style_theme = theme.clone();
        let button_style = scope.bond(move || {
            let size = match size.get().as_deref() {
                Some("small") => sapid::Size::Small,
                Some("large") => sapid::Size::Large,
                _ => sapid::Size::Medium,
            };
            let control = style_theme.tokens.control(size);
            let text = style_theme
                .tokens
                .typography(sapid::TypographyRole::Control, size);
            let (background, foreground, border) = if disabled.get() || loading.get() {
                (
                    style_theme.disabled,
                    style_theme.disabled_text,
                    style_theme.border,
                )
            } else {
                match variant.get().as_deref() {
                    Some("outline") => (
                        style_theme.input,
                        style_theme.accent_text,
                        style_theme.accent,
                    ),
                    Some("light") => (
                        style_theme.accent_soft,
                        style_theme.accent_text,
                        style_theme.accent_soft,
                    ),
                    _ => (
                        style_theme.accent,
                        style_theme.on_accent,
                        style_theme.accent,
                    ),
                }
            };
            Style {
                background: Some(background),
                color: Some(foreground),
                border_color: Some(border),
                font_size: Some(text.font_size),
                line_height: Some(text.line_height),
                font_weight: text.weight,
                ..Style::new()
                    .min_height(Length::Px(control.min_height))
                    .padding(Edges {
                        top: control.padding_y,
                        bottom: control.padding_y,
                        left: control.padding_x,
                        right: control.padding_x,
                    })
                    .radius(style_theme.radius * control.radius_scale)
            }
        });
        let label = scope.bond(move || {
            if loading.get() {
                return "Loading…".to_owned();
            }
            match variant.get().as_deref() {
                Some("outline") => "Outline button",
                Some("light") => "Light button",
                _ => "Filled button",
            }
            .to_owned()
        });
        view()
            .class("site-button-playground")
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_wrap(FlexWrap::Wrap)
                    .gap(24.0),
            )
            .fill((
                column()
                    .class("site-playground-preview")
                    .style(Style {
                        grow: Some(1.0),
                        align: Some(Align::Center),
                        justify: Some(Justify::Center),
                        ..Style::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Column)
                            .gap(16.0)
                            .min_width(Length::Px(180.0))
                    })
                    .fill((
                        sapid::button()
                            .theme(theme.clone())
                            .class("site-playground-button")
                            .label(label)
                            .style(button_style)
                            .enabled(disabled.map(|v| !v))
                            .loading(loading)
                            .on_press(move || clicks.revise(|n| *n += 1)),
                        sapid::text()
                            .theme(theme.clone())
                            .content(clicks.map(|n| format!("Clicked {n} times"))),
                        sapid::group().theme(theme.clone()).fill(
                            [
                                ("Light variant", sapid::ButtonVariant::Light),
                                ("Outline variant", sapid::ButtonVariant::Outline),
                                ("Subtle variant", sapid::ButtonVariant::Subtle),
                            ]
                            .into_iter()
                            .map(|(label, variant)| {
                                sapid::button()
                                    .theme(theme.clone())
                                    .variant(variant)
                                    .label(label)
                                    .enabled(disabled.map(|value| !value))
                                    .loading(loading)
                                    .on_press(move || clicks.revise(|count| *count += 1))
                            })
                            .collect::<Vec<_>>(),
                        ),
                    )),
                column()
                    .class("site-playground-controls")
                    .style(
                        Style::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Column)
                            .gap(16.0)
                            .width(Length::Px(208.0))
                            .max_width(Limit::Percent(100.0)),
                    )
                    .fill((
                        sapid::field("showcase-button-variant")
                            .theme(theme.clone())
                            .label("Variant")
                            .control({
                                let theme = theme.clone();
                                move |field| {
                                    sapid::native_select("showcase-variant")
                                        .theme(theme.clone())
                                        .field(field)
                                        .option(sapid::select_option("filled", "Filled"))
                                        .option(sapid::select_option("outline", "Outline"))
                                        .option(sapid::select_option("light", "Light"))
                                        .value(variant)
                                        .on_value_change(move |next| {
                                            if next.is_some() {
                                                variant.revise(|v| *v = next);
                                            }
                                        })
                                }
                            }),
                        sapid::field("showcase-button-size")
                            .theme(theme.clone())
                            .label("Size")
                            .control({
                                let theme = theme.clone();
                                move |field| {
                                    sapid::native_select("showcase-size")
                                        .theme(theme.clone())
                                        .field(field)
                                        .option(sapid::select_option("small", "Sm"))
                                        .option(sapid::select_option("medium", "Md"))
                                        .option(sapid::select_option("large", "Lg"))
                                        .value(size)
                                        .on_value_change(move |next| {
                                            if next.is_some() {
                                                size.revise(|v| *v = next);
                                            }
                                        })
                                }
                            }),
                        sapid::checkbox()
                            .theme(theme.clone())
                            .label("Disabled")
                            .checked(disabled)
                            .on_checked_change(move |next| disabled.revise(|v| *v = next)),
                        sapid::checkbox()
                            .theme(theme.clone())
                            .label("Loading")
                            .checked(loading)
                            .on_checked_change(move |next| loading.revise(|v| *v = next)),
                    )),
            ))
            .show_in(scope)
    })
}
