use glory::prelude::*;
use sapid::{Theme, portal, portal_context, portal_target};
pub fn portal_form(theme: Theme) -> impl Widget {
    portal_with_missing(theme, false)
}
pub fn portal_missing_form(theme: Theme) -> impl Widget {
    portal_with_missing(theme, true)
}
fn portal_with_missing(theme: Theme, missing: bool) -> impl Widget {
    widget(move |scope| {
        let context = portal_context(scope, "example")?;
        let chosen = scope.cage("a".to_string());
        let enabled = scope.cage(true);
        let a = scope.cage(!missing);
        let b = scope.cage(true);
        let source = scope.cage(true);
        let factories = scope.cage(0usize);
        let losses = scope.cage(0usize);
        let value = scope.cage("Keep my portal draft".to_owned());
        let target_a = context.clone();
        let target_b = context.clone();
        let sources = context.clone();
        let source_theme = theme.clone();
        view()
            .fill((
                sapid::button()
                    .theme(theme.clone())
                    .label("Move portal to A")
                    .on_press(move || chosen.revise(|v| *v = "a".to_string())),
                sapid::button()
                    .theme(theme.clone())
                    .label("Move portal to B")
                    .on_press(move || chosen.revise(|v| *v = "b".to_string())),
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle target A")
                    .on_press(move || a.revise(|v| *v = !*v)),
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle target B")
                    .on_press(move || b.revise(|v| *v = !*v)),
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle portal enabled")
                    .on_press(move || enabled.revise(|v| *v = !*v)),
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle portal source")
                    .on_press(move || source.revise(|v| *v = !*v)),
                text().class("portal-status").content(scope.bond(move || {
                    format!(
                        "target={} enabled={} source={} factories={} losses={}",
                        chosen.get(),
                        enabled.get(),
                        source.get(),
                        factories.get(),
                        losses.get()
                    )
                })),
                view()
                    .class("portal-destinations")
                    .style(Style::new().display(glory::style::Display::Flex).gap(24.))
                    .fill((
                        view()
                            .class("portal-destination-a")
                            .style(
                                Style::new()
                                    .width(Length::Px(400.))
                                    .min_height(Length::Px(220.))
                                    .padding(glory::style::Edges::all(12.)),
                            )
                            .fill(when(a, move || {
                                portal_target(&target_a, "a").fill(text().content("Destination A"))
                            })),
                        view()
                            .class("portal-destination-b")
                            .style(
                                Style::new()
                                    .width(Length::Px(400.))
                                    .min_height(Length::Px(220.))
                                    .padding(glory::style::Edges::all(12.)),
                            )
                            .fill(when(b, move || {
                                portal_target(&target_b, "b").fill(text().content("Destination B"))
                            })),
                    )),
                view()
                    .class("portal-source-location")
                    .fill(when(source, move || {
                        let theme = source_theme.clone();
                        portal(&sources, chosen)
                            .enabled(enabled)
                            .on_target_lost(move |_| losses.revise(|v| *v += 1))
                            .content(move || {
                                factories.revise(|v| *v += 1);
                                let theme = theme.clone();
                                widget(move |scope| {
                                    let count = scope.cage(0usize);
                                    view()
                                        .class("portal-content")
                                        .fill((
                                            sapid::text_field()
                                                .theme(theme.clone())
                                                .name("Portaled draft")
                                                .value(value)
                                                .on_value_change(move |next| {
                                                    value.revise(|v| *v = next)
                                                }),
                                            sapid::button()
                                                .theme(theme.clone())
                                                .label("Increment portal counter")
                                                .on_press(move || count.revise(|v| *v += 1)),
                                            text().class("portal-local-counter").content(
                                                scope.bond(move || {
                                                    format!("source counter={}", count.get())
                                                }),
                                            ),
                                        ))
                                        .show_in(scope)
                                })
                            })
                    })),
            ))
            .show_in(scope)
    })
}
