use glory::render::Orientation;
use glory::style::Length;
use glory::ui::{Widget, component, text};
use sapid::{Theme, TimelineProgress, button, separator, text_field, timeline, timeline_item};
pub fn timeline_form(theme: Theme) -> impl Widget {
    component(move |scope| {
        let progress = scope.cage(TimelineProgress::Active("build".into()));
        glory::ui::col()
            .class("timeline-fixture")
            .fill((
                separator()
                    .theme(theme.clone())
                    .name("Stage boundary")
                    .length(Length::Px(240.0)),
                separator()
                    .theme(theme.clone())
                    .name("Vertical boundary")
                    .orientation(Orientation::Vertical)
                    .length(Length::Px(32.0)),
                separator()
                    .theme(theme.clone())
                    .class("decorative-boundary")
                    .decorative(true)
                    .length(Length::Px(240.0)),
                timeline("release-progress")
                    .theme(theme.clone())
                    .name("Release progress")
                    .progress(progress)
                    .item(
                        timeline_item("design")
                            .title("Design")
                            .timestamp("2026-10-22")
                            .description("Agree on the document contract"),
                    )
                    .item(
                        timeline_item("build")
                            .title("Build")
                            .timestamp("2026-10-23")
                            .description("Implement and verify components")
                            .marker(text().content("Build marker"))
                            .fill(
                                text_field()
                                    .theme(theme.clone())
                                    .class("timeline-draft")
                                    .name("Build notes")
                                    .default_value("Build notes"),
                            ),
                    )
                    .item(
                        timeline_item("ship")
                            .title("Ship")
                            .timestamp("2026-10-24")
                            .description("Publish the release"),
                    ),
                button()
                    .theme(theme.clone())
                    .label("Advance timeline")
                    .on_press(move || {
                        progress
                            .revise(|value| *value = TimelineProgress::Active("ship".into()))
                    }),
                button()
                    .theme(theme.clone())
                    .label("Complete timeline")
                    .on_press(move || {
                        progress.revise(|value| *value = TimelineProgress::Complete)
                    }),
                button()
                    .theme(theme.clone())
                    .label("Reset timeline")
                    .on_press(move || {
                        progress.revise(|value| *value = TimelineProgress::Pending)
                    }),
            ))
            .show_in(scope)
    })
}
