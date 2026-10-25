use glory::style::{Length, Style};
use glory::ui::{Widget, text, widget};
use sapid::{ContentPolicy, Theme, button, dialog, loading_overlay, overlay, text_field};
/// Matches the final Glory inert stream probe for an actual adoption check.
pub fn inert_stream_probe() -> impl Widget {
    use glory::render::{FocusMode, Layer, Role, SemanticId, Semantics};
    glory::ui::view()
        .class("stream-loading-region")
        .name("Streaming region")
        .semantics(Some(Box::new(Semantics {
            busy: Some(true),
            ..Default::default()
        })))
        .fill(
            glory::ui::view()
                .class("stream-inert-content")
                .semantics(Some(Box::new(Semantics {
                    inert: true,
                    ..Default::default()
                })))
                .fill((
                    glory::ui::button().label("Retained action"),
                    glory::ui::view()
                        .class("stream-inert-layer")
                        .role(Role::Dialog)
                        .focus(FocusMode::Programmatic)
                        .name("Retained dialog")
                        .semantics(Some(Box::new(Semantics {
                            id: Some(SemanticId::new("stream-inert-dialog").unwrap()),
                            layer: Some(Layer::default()),
                            ..Default::default()
                        })))
                        .fill(glory::ui::button().label("Layer action")),
                )),
        )
}
pub fn loading_overlay_form(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let loading = scope.cage(false);
        let visible = scope.cage(true);
        let actions = scope.cage(0usize);
        let value = scope.cage("Retained text".to_string());
        let inside_theme = theme.clone();
        let global_theme = theme.clone();
        sapid::stack()
            .theme(theme.clone())
            .gap(20.)
            .fill((
                loading_overlay(loading)
                    .theme(theme.clone())
                    .name("Preview editor")
                    .label("Loading preview")
                    .class("loading-region")
                    .style(Style::new().width(Length::Px(280.)))
                    .fill(
                        sapid::stack().theme(theme.clone()).gap(12.).fill((
                            text_field()
                                .theme(theme.clone())
                                .name("Region editor")
                                .value(value)
                                .on_value_change(move |next| {
                                    value.revise(|value| *value = next)
                                }),
                            button()
                                .theme(theme.clone())
                                .label("Run region action")
                                .on_press(move || actions.revise(|n| *n += 1)),
                            dialog("loading-region-dialog")
                                .theme(theme.clone())
                                .label("Open region dialog")
                                .title("Region dialog")
                                .content_policy(ContentPolicy::KeepMounted)
                                .content(move || {
                                    button()
                                        .theme(inside_theme.clone())
                                        .label("Load from region dialog")
                                        .on_press(move || loading.revise(|value| *value = true))
                                }),
                        )),
                    ),
                overlay()
                    .theme(theme.clone())
                    .visible(visible)
                    .class("decorative-region")
                    .style(Style::new().width(Length::Px(280.)))
                    .color(theme.accent)
                    .opacity(0.2)
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Decorated action")
                            .on_press(move || actions.revise(|n| *n += 1)),
                    ),
                button()
                    .theme(theme.clone())
                    .label("Start region loading")
                    .on_press(move || loading.revise(|value| *value = true)),
                button()
                    .theme(theme.clone())
                    .label("Finish region loading")
                    .on_press(move || loading.revise(|value| *value = false)),
                button()
                    .theme(theme.clone())
                    .label("Toggle decorative shade")
                    .on_press(move || visible.revise(|value| *value = !*value)),
                dialog("independent-loading-dialog")
                    .theme(theme.clone())
                    .label("Open independent dialog")
                    .title("Independent dialog")
                    .content(move || {
                        button()
                            .theme(global_theme.clone())
                            .label("Load background region")
                            .on_press(move || loading.revise(|value| *value = true))
                    }),
                text()
                    .class("loading-actions")
                    .content(actions.map(|count| format!("Actions: {count}"))),
                text().class("loading-value").content(value),
            ))
            .show_in(scope)
    })
}
