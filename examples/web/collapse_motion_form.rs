use glory::prelude::*;
use glory::render::EventKind;
use glory::style::FlexWrap;
use sapid::{ContentPolicy, Theme, button, collapsible, text_field};

pub fn collapse_motion_form(theme: Theme) -> impl Widget {
    component(move |scope| {
        let open = scope.cage(false);
        let animated = scope.cage(true);
        let duration = scope.cage(360u16);
        let tall = scope.cage(false);
        let accept = scope.cage(true);
        let proposals = scope.cage(0usize);
        let mounted = scope.cage(0usize);
        let cleaned = scope.cage(0usize);
        let order = scope.cage(String::new());
        let status = scope.bond(move || {
            format!(
                "Open: {}; mounted: {}; cleaned: {}; proposals: {}; order: {}",
                open.get(),
                mounted.get(),
                cleaned.get(),
                proposals.get(),
                order.get()
            )
        });
        let editor_theme = theme.clone();
        let animated_panel = collapsible("animated-details")
            .theme(theme.clone())
            .class("animated-unmount-collapse")
            .label("Animated details")
            .open(open)
            .height_transition(duration)
            .animated(animated)
            .content_policy(ContentPolicy::Unmount)
            .trigger_content(|| glory::ui::text().content("Show editable details"))
            .on_open_change(move |next| {
                proposals.revise(|value| *value += 1);
                order.revise(|order| *order = "typed".to_owned());
                if accept.get() {
                    open.revise(|value| *value = next);
                }
            })
            .on(EventKind::Activate, move |_| {
                order.revise(|order| order.push_str(" generic"))
            })
            .content(move || {
                let theme = editor_theme.clone();
                component(move |scope| {
                    mounted.revise(|value| *value += 1);
                    scope.on_cleanup(move || {
                        let _ = cleaned.try_revise(|value| *value += 1);
                    })?;
                    glory::ui::view()
                        .class("animated-collapse-content")
                        .style(tall.map(|tall| {
                            Style::new().height(Length::Px(if *tall { 180. } else { 96. }))
                        }))
                        .fill(glory::ui::text().content(
                            "This body stays owned until the current close actually completes.",
                        ))
                        .fill(
                            text_field()
                                .theme(theme.clone())
                                .class("animated-collapse-editor")
                                .name("Animated draft")
                                .default_value("Draft stays during reversal"),
                        )
                        .show_in(scope)
                })
            });
        let keep_theme = theme.clone();
        let kept = collapsible("kept-details")
            .theme(theme.clone())
            .class("animated-kept-collapse")
            .label("Kept details")
            .open(open)
            .height_transition(duration)
            .animated(animated)
            .content_policy(ContentPolicy::KeepMounted)
            .on_open_change(move |next| {
                if accept.get() {
                    open.revise(|value| *value = next);
                }
            })
            .content(move || {
                text_field()
                    .theme(keep_theme.clone())
                    .class("kept-collapse-editor")
                    .name("Retained draft")
                    .default_value("Retained note")
            });
        glory::ui::view()
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(20.),
            )
            .fill(animated_panel)
            .fill(kept)
            .fill(
                glory::ui::view()
                    .style(
                        Style::new()
                            .display(Display::Flex)
                            .flex_wrap(FlexWrap::Wrap)
                            .gap(8.),
                    )
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Open from source")
                            .on_press(move || open.revise(|value| *value = true)),
                    )
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Close from source")
                            .on_press(move || open.revise(|value| *value = false)),
                    )
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Toggle motion")
                            .on_press(move || animated.revise(|value| *value = !*value)),
                    )
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Toggle content height")
                            .on_press(move || tall.revise(|value| *value = !*value)),
                    )
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Toggle proposal acceptance")
                            .on_press(move || accept.revise(|value| *value = !*value)),
                    )
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Set duration zero")
                            .on_press(move || duration.revise(|value| *value = 0)),
                    )
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Set duration normal")
                            .on_press(move || duration.revise(|value| *value = 360)),
                    ),
            )
            .fill(
                glory::ui::text()
                    .class("collapse-motion-status")
                    .content(status),
            )
            .show_in(scope)
    })
}
