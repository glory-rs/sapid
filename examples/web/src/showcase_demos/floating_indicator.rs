use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::floating_indicator_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{Theme, button, floating_indicator, floating_indicator_target, text_field};
    pub fn floating_indicator_form(theme: Theme) -> impl Widget {
        widget(move |scope| {
            let active = scope.cage(Some("one".to_owned()));
            let vertical = scope.cage(false);
            let long = scope.cage(false);
            let visible = scope.cage(true);
            let animated = scope.cage(true);
            let duration = scope.cage(360u16);
            let revision = scope.cage(0u64);
            let keys = scope.cage(["one", "two", "draft"].map(str::to_owned).to_vec());
            let geometry = scope.cage(None::<glory::render::Rect>);
            let updates = scope.cage(0usize);
            let status = scope.bond(move || {
                format!(
                    "Active: {}; measured: {}; updates: {}",
                    active.get().unwrap_or_else(|| "none".into()),
                    geometry.get().map_or_else(
                        || "none".into(),
                        |rect| format!(
                            "{:.1},{:.1},{:.1},{:.1}",
                            rect.x, rect.y, rect.width, rect.height
                        )
                    ),
                    updates.get()
                )
            });
            let content_theme = theme.clone();
            let indicator = floating_indicator("workspaces")
                .theme(theme.clone())
                .class("demo-floating-indicator")
                .active(active)
                .visible(visible)
                .animated(animated)
                .duration(duration)
                .measurement_revision(revision)
                .style(
                    Style::new()
                        .width(Length::Px(640.))
                        .max_width(glory::style::Limit::Percent(100.))
                        .overflow_x(glory::style::Overflow::Hidden),
                )
                .on_geometry_change(move |rect| {
                    geometry.revise(|value| *value = rect);
                    updates.revise(|value| *value += 1);
                })
                .content(move |context| {
                    let theme = content_theme.clone();
                    glory::ui::view()
                        .class("floating-demo-layout")
                        .style(vertical.map(|vertical| {
                            Style {
                                align: Some(glory::style::Align::Start),
                                flex_wrap: Some(glory::style::FlexWrap::Wrap),
                                ..Style::new()
                                    .display(Display::Flex)
                                    .flex_direction(if *vertical {
                                        FlexDirection::Column
                                    } else {
                                        FlexDirection::Row
                                    })
                                    .gap(12.)
                            }
                        }))
                        .fill(glory::ui::each(keys).key(|key| key.clone()).children(
                            move |entry| {
                                let key = entry.value;
                                let context = context.clone();
                                let theme = theme.clone();
                                widget(move |scope| {
                                    let key = key.try_get()?;
                                    let class = format!("floating-target-{key}");
                                    if key == "draft" {
                                        floating_indicator_target(context.clone(), key)
                                            .class(class)
                                            .style(Style::new().width(Length::Px(180.)))
                                            .fill(
                                                text_field()
                                                    .theme(theme.clone())
                                                    .class("floating-draft")
                                                    .name("Floating draft")
                                                    .default_value("Keep this draft")
                                                    .style(
                                                        Style::new()
                                                            .width(Length::Percent(100.))
                                                            .min_width(Length::Px(0.)),
                                                    ),
                                            )
                                            .show_in(scope)
                                    } else {
                                        let next = key.clone();
                                        let label: Vary<String> = if key == "one" {
                                            long.map(|long| {
                                                if *long {
                                                    "All projects and recent work".to_owned()
                                                } else {
                                                    "Projects".to_owned()
                                                }
                                            })
                                            .into()
                                        } else {
                                            "Teams".to_owned().into()
                                        };
                                        floating_indicator_target(context.clone(), key)
                                            .class(class)
                                            .fill(
                                                button()
                                                    .theme(theme.clone())
                                                    .label(label)
                                                    .on_press(move || {
                                                        active.revise(|value| {
                                                            *value = Some(next.clone())
                                                        })
                                                    }),
                                            )
                                            .show_in(scope)
                                    }
                                })
                            },
                        ))
                });
            glory::ui::view()
                .style(
                    Style::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Column)
                        .gap(20.),
                )
                .fill(indicator)
                .fill(
                    glory::ui::view()
                        .style(Style {
                            flex_wrap: Some(glory::style::FlexWrap::Wrap),
                            ..Style::new().display(Display::Flex).gap(8.)
                        })
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Select projects")
                                .on_press(move || {
                                    active.revise(|value| *value = Some("one".into()))
                                }),
                        )
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Select teams")
                                .on_press(move || {
                                    active.revise(|value| *value = Some("two".into()))
                                }),
                        )
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Select draft")
                                .on_press(move || {
                                    active.revise(|value| *value = Some("draft".into()))
                                }),
                        )
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Toggle indicator direction")
                                .on_press(move || {
                                    vertical.revise(|value| *value = !*value);
                                    revision.revise(|value| *value += 1);
                                }),
                        )
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Reverse indicator targets")
                                .on_press(move || {
                                    keys.revise(|keys| keys.reverse());
                                    revision.revise(|value| *value += 1);
                                }),
                        )
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Toggle target label")
                                .on_press(move || long.revise(|value| *value = !*value)),
                        )
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Toggle indicator visibility")
                                .on_press(move || visible.revise(|value| *value = !*value)),
                        )
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Toggle indicator motion")
                                .on_press(move || animated.revise(|value| *value = !*value)),
                        )
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Set indicator duration zero")
                                .on_press(move || duration.revise(|value| *value = 0)),
                        )
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Remove projects target")
                                .on_press(move || {
                                    keys.revise(|keys| keys.retain(|key| key != "one"))
                                }),
                        )
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Restore projects target")
                                .on_press(move || {
                                    keys.revise(|keys| {
                                        if !keys.iter().any(|key| key == "one") {
                                            keys.push("one".into());
                                        }
                                    })
                                }),
                        ),
                )
                .fill(
                    glory::ui::text()
                        .class("floating-indicator-status")
                        .content(status),
                )
                .show_in(scope)
        })
    }
}
