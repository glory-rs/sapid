use glory::prelude::*;
use sapid::{OverflowHandle, Theme, button, overflow_item, overflow_list, text_field};

pub fn overflow_list_form(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let width = scope.cage(520.);
        let enabled = scope.cage(true);
        let long = scope.cage(false);
        let revision = scope.cage(0u64);
        let visible = scope.cage(2usize);
        let accept = scope.cage(true);
        let proposals = scope.cage(0usize);
        let source = scope.cage(
            ["home", "report", "note", "settings", "share"]
                .map(str::to_owned)
                .to_vec(),
        );
        let factories = scope.cage(0usize);
        let cleaned = scope.cage(0usize);
        let clicks = scope.cage(0usize);
        let handle = scope.cage(None::<OverflowHandle>);
        let status=scope.bond(move||{
            let handle=handle.get();let count=handle.as_ref().map_or(0,|handle|handle.visible_count().get());
            let hidden=handle.as_ref().map_or(0,|handle|handle.overflow_count().get());
            let pending=handle.as_ref().map_or(0,|handle|handle.pending_queries().get());
            format!("Visible: {count}; hidden: {hidden}; pending: {pending}; factories: {}; cleaned: {}; proposals: {}; clicks: {}",factories.get(),cleaned.get(),proposals.get(),clicks.get())
        });
        let item_theme = theme.clone();
        let list = overflow_list("responsive-actions")
            .class("demo-overflow-list")
            .name("Responsive action strip")
            .theme(theme.clone())
            .style(width.map(|width| {
                Style::new()
                    .width(Length::Px(*width))
                    .max_width(glory::style::Limit::Percent(100.))
            }))
            .enabled(enabled)
            .initial_visible(2)
            .visible_count(visible)
            .handle_ref(handle)
            .measurement_revision(revision)
            .overflow_label("More actions")
            .on_visible_count_change(move |count| {
                proposals.revise(|value| *value += 1);
                if accept.get() {
                    visible.revise(|value| *value = count);
                }
            })
            .dynamic_items(source, move |key| {
                factories.revise(|value| *value += 1);
                let key = key.to_owned();
                let theme = item_theme.clone();
                let mut item = overflow_item(key.clone());
                if key == "note" {
                    item = item.width(160.);
                }
                item.content(move || {
                    let key = key.clone();
                    let theme = theme.clone();
                    widget(move |scope| {
                        scope.on_cleanup(move || {
                            let _ = cleaned.try_revise(|value| *value += 1);
                        })?;
                        if key == "note" {
                            text_field()
                                .theme(theme.clone())
                                .class("overflow-draft")
                                .name("Overflow draft")
                                .style(
                                    Style::new()
                                        .width(Length::Percent(100.))
                                        .min_width(Length::Px(0.)),
                                )
                                .default_value("Retained draft")
                                .show_in(scope)
                        } else {
                            let label: Lotus<String> = if key == "report" {
                                long.map(|long| {
                                    if *long {
                                        "A much longer annual report action".to_owned()
                                    } else {
                                        "Quarterly report".to_owned()
                                    }
                                })
                                .into()
                            } else {
                                match key.as_str() {
                                    "home" => "Home",
                                    "settings" => "Settings",
                                    _ => "Share link",
                                }
                                .to_owned()
                                .into()
                            };
                            button()
                                .theme(theme.clone())
                                .label(label)
                                .class(format!("overflow-action-{key}"))
                                .on_press(move || clicks.revise(|value| *value += 1))
                                .show_in(scope)
                        }
                    })
                })
            });
        glory::ui::view()
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(20.),
            )
            .fill(list)
            .fill(
                glory::ui::view()
                    .style(Style {
                        flex_wrap: Some(glory::style::FlexWrap::Wrap),
                        ..Style::new().display(Display::Flex).gap(8.)
                    })
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Narrow list")
                            .on_press(move || width.revise(|value| *value = 240.)),
                    )
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Widen list")
                            .on_press(move || width.revise(|value| *value = 640.)),
                    )
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Toggle action label")
                            .on_press(move || {
                                long.revise(|value| *value = !*value);
                                revision.revise(|value| *value += 1);
                            }),
                    )
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Reverse items")
                            .on_press(move || source.revise(|keys| keys.reverse())),
                    )
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Remove draft")
                            .on_press(move || {
                                source.revise(|keys| keys.retain(|key| key != "note"))
                            }),
                    )
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Restore draft")
                            .on_press(move || {
                                source.revise(|keys| {
                                    if !keys.iter().any(|key| key == "note") {
                                        keys.push("note".into());
                                    }
                                })
                            }),
                    )
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Toggle list enabled")
                            .on_press(move || enabled.revise(|value| *value = !*value)),
                    )
                    .fill(
                        button()
                            .theme(theme.clone())
                            .label("Toggle capacity acceptance")
                            .on_press(move || accept.revise(|value| *value = !*value)),
                    ),
            )
            .fill(
                glory::ui::text()
                    .class("overflow-list-status")
                    .content(status),
            )
            .show_in(scope)
    })
}
