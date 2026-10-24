use glory::prelude::*;
use glory::render::{
    Query, QueryValue, Semantics, TranslationKind, TranslationMotion, TranslationOffset,
};
fn to(x: f64) -> TranslationMotion {
    TranslationMotion {
        kind: TranslationKind::To {
            target: TranslationOffset::new(x, 0.),
            duration_millis: 800,
        },
        paused: false,
    }
}
pub fn translation_form(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| {
        let motion = scope.cage(Some(to(0.)));
        let visible = scope.cage(true);
        let reference = scope.cage(None);
        let queried = scope.cage("Not observed".to_string());
        let semantic = scope.bond(move || {
            Some(Box::new(Semantics {
                translation: motion.get(),
                hidden: !visible.get(),
                inert: true,
                ..Default::default()
            }))
        });
        let observe_scope = scope.downgrade();
        glory::ui::view()
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(12.),
            )
            .fill((
                sapid::visually_hidden().content("Balance 123"),
                glory::ui::view()
                    .class("translation-clip")
                    .style(Style {
                        width: Some(Length::Px(80.)),
                        height: Some(Length::Px(60.)),
                        overflow_x: Some(glory::style::Overflow::Hidden),
                        overflow_y: Some(glory::style::Overflow::Hidden),
                        background: Some(Color(255, 255, 255, 255)),
                        opacity: Some(0.5),
                        ..Style::new()
                    })
                    .fill(
                        glory::ui::view()
                            .class("translation-parent")
                            .semantics(Some(Box::new(Semantics {
                                inert: true,
                                translation: Some(to(12.)),
                                ..Default::default()
                            })))
                            .fill(
                                glory::ui::view()
                                    .class("translation-cube")
                                    .node_ref(reference)
                                    .semantics(semantic)
                                    .style(Style {
                                        width: Some(Length::Px(20.)),
                                        height: Some(Length::Px(20.)),
                                        background: Some(Color(255, 0, 0, 255)),
                                        ..Style::new()
                                    }),
                            ),
                    ),
                glory::ui::view()
                    .semantics(Some(Box::new(Semantics {
                        inert: true,
                        ..Default::default()
                    })))
                    .fill((text().content("Balance 123"), text().content("Balance 123"))),
                sapid::text_field()
                    .theme(theme.clone())
                    .name("Motion focus probe")
                    .default_value("Retained motion focus"),
                glory::ui::view()
                    .style(Style {
                        flex_wrap: Some(glory::style::FlexWrap::Wrap),
                        ..Style::new().display(Display::Flex).gap(8.)
                    })
                    .fill((
                        sapid::button()
                            .theme(theme.clone())
                            .label("Move forty")
                            .on_press(move || motion.revise(|value| *value = Some(to(40.)))),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Reverse target")
                            .on_press(move || motion.revise(|value| *value = Some(to(-20.)))),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Toggle translation pause")
                            .on_press(move || {
                                motion.revise(|value| {
                                    if let Some(value) = value {
                                        value.paused = !value.paused;
                                    }
                                })
                            }),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Clear translation")
                            .on_press(move || motion.revise(|value| *value = None)),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Repeat translation")
                            .on_press(move || {
                                motion.revise(|value| {
                                    *value = Some(TranslationMotion {
                                        kind: TranslationKind::Loop {
                                            from: TranslationOffset::ZERO,
                                            to: TranslationOffset::new(100., 0.),
                                            period_millis: 800,
                                        },
                                        paused: false,
                                    })
                                })
                            }),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Toggle translation visibility")
                            .on_press(move || visible.revise(|value| *value = !*value)),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Observe translation bounds")
                            .on_press(move || {
                                if let Some(scope) = observe_scope.upgrade()
                                    && let Some(node) = reference.get()
                                {
                                    let _ = scope.request(Query::Bounds(node), move |value| {
                                        if let Ok(QueryValue::Bounds(rect)) = value {
                                            queried.revise(|value| {
                                                *value = format!("{:.2},{:.2}", rect.x, rect.y)
                                            });
                                        }
                                    });
                                }
                            }),
                    )),
                text().class("translation-query-status").content(queried),
            ))
            .show_in(scope)
    })
}
