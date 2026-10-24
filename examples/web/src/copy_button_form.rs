use glory::prelude::*;
pub const COPY_SOURCE: &str = "Copy exact 😀 <script>\nsecond line\n";
pub fn copy_button_form(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| {
        let source = scope.cage(COPY_SOURCE.to_string());
        let state = scope.cage(sapid::CopyState::Idle);
        let enabled = scope.cage(true);
        let visible = scope.cage(true);
        let log = scope.cage(String::new());
        let calls = scope.cage(0usize);
        let inner = theme.clone();
        sapid::stack()
            .gap(12.)
            .fill((
                glory::ui::when(visible, move || {
                    sapid::copy_button()
                        .theme(inner.clone())
                        .class("copy-demo")
                        .source(source)
                        .enabled(enabled)
                        .state_ref(state)
                        .feedback_delay(|| Box::pin(crate::menu_form::local_delay(350)))
                        .on_result(move |outcome| {
                            calls.revise(|v| *v += 1);
                            log.revise(|v| *v = format!("typed:{outcome:?}:{:?}", state.get()));
                        })
                        .on(glory::render::EventKind::Clipboard, move |_| {
                            log.revise(|v| v.push_str(&format!(";generic:{:?}", state.get())))
                        })
                }),
                sapid::copy_button()
                    .theme(theme.clone())
                    .class("copy-persistent")
                    .label("Copy persistent feedback")
                    .source("Persistent source"),
                sapid::copy_button()
                    .unstyled()
                    .class("copy-unstyled")
                    .label("Copy without theme")
                    .source("Unstyled source"),
                sapid::button()
                    .theme(theme.clone())
                    .label("Change copy source")
                    .on_press(move || source.revise(|v| *v = "Changed 😀\n".into())),
                sapid::button()
                    .theme(theme.clone())
                    .label("Reset copy source")
                    .on_press(move || source.revise(|v| *v = COPY_SOURCE.into())),
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle copy enabled")
                    .on_press(move || enabled.revise(|v| *v = !*v)),
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle copy owner")
                    .on_press(move || visible.revise(|v| *v = !*v)),
                glory::ui::text()
                    .class("copy-state")
                    .content(scope.bond(move || format!("{:?}", state.get()))),
                glory::ui::text().class("copy-log").content(log),
                glory::ui::text()
                    .class("copy-calls")
                    .content(scope.bond(move || format!("Calls: {}", calls.get()))),
                sapid::text_field()
                    .theme(theme.clone())
                    .name("Clipboard neighbor draft")
                    .default_value("Retained draft"),
            ))
            .show_in(scope)
    })
}
