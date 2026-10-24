use glory::prelude::*;
use glory::render::{FocusScopeConfig, SemanticId};
use sapid::{Theme, focus_scope};

pub fn focus_scope_form(theme: Theme) -> impl Widget {
    focus_scope_form_with_prefix(theme, "scope")
}
pub fn focus_scope_form_with_prefix(theme: Theme, prefix: &'static str) -> impl Widget {
    widget(move |scope| {
        let active = scope.cage(false);
        let inner = scope.cage(false);
        let mounted = scope.cage(true);
        let loop_tab = scope.cage(true);
        let contain = scope.cage(true);
        let auto = scope.cage(true);
        let clicks = scope.cage(0usize);
        let configuration = scope.bond(move || FocusScopeConfig {
            active: active.get(),
            loop_tab: loop_tab.get(),
            contain_focus: contain.get(),
            auto_focus: auto.get(),
            ..Default::default()
        });
        let theme_inner = theme.clone();
        view()
            .fill((
                sapid::button()
                    .theme(theme.clone())
                    .label("Activate scope")
                    .on_press(move || active.revise(|value| *value = true)),
                sapid::button()
                    .theme(theme.clone())
                    .label("Deactivate scope")
                    .on_press(move || active.revise(|value| *value = false)),
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle scope owner")
                    .on_press(move || mounted.revise(|value| *value = !*value)),
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle loop policy")
                    .on_press(move || loop_tab.revise(|value| *value = !*value)),
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle containment policy")
                    .on_press(move || contain.revise(|value| *value = !*value)),
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle autofocus policy")
                    .on_press(move || auto.revise(|value| *value = !*value)),
                text()
                    .class("focus-scope-status")
                    .content(scope.bond(move || {
                        format!(
                            "active={} inner={} owner={} clicks={}",
                            active.get(),
                            inner.get(),
                            mounted.get(),
                            clicks.get()
                        )
                    })),
                when(mounted, move || {
                    focus_scope()
                        .configuration(configuration)
                        .semantic_id(SemanticId::new(format!("{prefix}-outer")).unwrap())
                        .style(
                            Style::new()
                                .padding(glory::style::Edges::all(12.))
                                .width(Length::Px(540.)),
                        )
                        .fill((
                            sapid::text_field()
                                .theme(theme_inner.clone())
                                .name("Scope draft")
                                .default_value("Keep scope draft"),
                            sapid::button()
                                .theme(theme_inner.clone())
                                .label("Enable nested scope")
                                .on_press(move || inner.revise(|value| *value = true)),
                            focus_scope()
                                .active(inner)
                                .semantic_id(
                                    SemanticId::new(format!("{prefix}-inner")).unwrap(),
                                )
                                .fill((
                                    sapid::text_field()
                                        .theme(theme_inner.clone())
                                        .name("Nested draft")
                                        .default_value("Nested draft"),
                                    sapid::button()
                                        .theme(theme_inner.clone())
                                        .label("Close nested scope")
                                        .on_press(move || inner.revise(|value| *value = false)),
                                )),
                            sapid::button()
                                .theme(theme_inner.clone())
                                .label("Scope last control")
                                .on_press(move || clicks.revise(|value| *value += 1)),
                        ))
                }),
                sapid::button()
                    .theme(theme.clone())
                    .label("After scope")
                    .on_press(move || clicks.revise(|value| *value += 1)),
            ))
            .show_in(scope)
    })
}
