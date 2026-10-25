use glory::ui::{Widget, widget};
use sapid::{Theme, button, progress, ring_progress, semi_circle_progress};
pub fn progress_form(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let value = scope.cage(25.);
        let unknown = scope.cage(false);
        let text = scope.cage("Quarter complete".to_string());
        sapid::stack()
            .theme(theme.clone())
            .gap(20.0)
            .fill((
                progress(value)
                    .theme(theme.clone())
                    .name("File upload")
                    .value_text(text)
                    .indeterminate(unknown)
                    .size(240.)
                    .thickness(12.)
                    .class("progress-linear"),
                sapid::group().theme(theme.clone()).gap(24.0).fill((
                    ring_progress(value)
                        .theme(theme.clone())
                        .name("Circular upload")
                        .value_text(text)
                        .indeterminate(unknown)
                        .size(100.)
                        .thickness(10.)
                        .class("progress-ring"),
                    semi_circle_progress(value)
                        .theme(theme.clone())
                        .name("Half-circle upload")
                        .value_text(text)
                        .indeterminate(unknown)
                        .size(120.)
                        .thickness(12.)
                        .class("progress-semicircle"),
                )),
                progress(50.)
                    .theme(theme.clone())
                    .unstyled()
                    .name("Unstyled progress")
                    .size(200.)
                    .class("progress-unstyled"),
                button()
                    .theme(theme.clone())
                    .label("Advance progress")
                    .on_press(move || {
                        value.revise(|value| *value = 75.);
                        text.revise(|text| *text = "Three quarters complete".into());
                    }),
                button()
                    .theme(theme.clone())
                    .label("Toggle unknown progress")
                    .on_press(move || unknown.revise(|value| *value = !*value)),
                button()
                    .theme(theme.clone())
                    .label("Complete progress")
                    .on_press(move || {
                        value.revise(|value| *value = 100.);
                        text.revise(|text| *text = "Complete".into());
                    }),
                button()
                    .theme(theme.clone())
                    .label("Reset progress")
                    .on_press(move || {
                        value.revise(|value| *value = 0.);
                        text.revise(|text| *text = "Not started".into());
                    }),
            ))
            .show_in(scope)
    })
}
