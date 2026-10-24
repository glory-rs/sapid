use glory::prelude::*;
use glory::render::{Direction, EventKind, Orientation};
use glory::style::FlexWrap;
use sapid::{ContentPolicy, TabActivation, Theme, button, tab, tabs, text_field};

pub fn tabs_dynamic_form(theme: Theme) -> impl Widget {
    component(move |scope| {
        let keys = scope.cage(vec![
            "alpha".to_owned(),
            "beta".to_owned(),
            "gamma".to_owned(),
        ]);
        let value = scope.cage("alpha".to_owned());
        let accept = scope.cage(false);
        let enabled = scope.cage(true);
        let beta_enabled = scope.cage(false);
        let chinese = scope.cage(false);
        let proposals = scope.cage(0usize);
        let factories = scope.cage(0usize);
        let order = scope.cage(String::new());
        let status = scope.bond(move || {
            format!(
                "Proposals: {}; factories: {}; value: {}; order: {}",
                proposals.get(),
                factories.get(),
                value.get(),
                order.get()
            )
        });
        let editor_theme = theme.clone();
        let controlled = tabs("keyed-tabs")
            .name("Document sections")
            .class("controlled-dynamic-tabs")
            .theme(theme.clone())
            .direction(Direction::Rtl)
            .enabled(enabled)
            .value(value)
            .content_policy(ContentPolicy::KeepMounted)
            .on_value_change(move |next| {
                proposals.revise(|value| *value += 1);
                order.revise(|value| *value = "typed".to_owned());
                if accept.get() {
                    value.revise(|value| *value = next);
                }
            })
            .on(EventKind::Activate, move |_| {
                order.revise(|value| value.push_str(" generic"))
            })
            .tabs(keys, move |key| {
                factories.revise(|value| *value += 1);
                let word = key.to_owned();
                let label = chinese.map(move |chinese| {
                    if *chinese {
                        format!("章节 {word}")
                    } else {
                        format!("Section {word}")
                    }
                });
                let enabled: Lotus<bool> = if key == "beta" {
                    beta_enabled.into()
                } else {
                    true.into()
                };
                let theme = editor_theme.clone();
                let class = format!("dynamic-tab-editor-{key}");
                tab(key)
                    .label(label)
                    .enabled(enabled)
                    .trigger_content(move || {
                        component(move |scope| {
                            glory::ui::text().content("↳ ").show_in(scope)?;
                            glory::ui::text()
                                .class("dynamic-tab-caption")
                                .content(label)
                                .show_in(scope)
                        })
                    })
                    .content(move || {
                        text_field()
                            .theme(theme.clone())
                            .class(class.clone())
                            .name(label)
                            .default_value("Initial draft")
                    })
            });
        let local_theme = theme.clone();
        let local = tabs("local-keyed-tabs")
            .name("Manual chapters")
            .theme(theme.clone())
            .class("local-dynamic-tabs")
            .orientation(Orientation::Vertical)
            .activation(TabActivation::Manual)
            .content_policy(ContentPolicy::Unmount)
            .tabs(keys, move |key| {
                let label = format!("Local {key}");
                let class = format!("local-dynamic-tab-editor-{key}");
                let theme = local_theme.clone();
                tab(key).label(label.clone()).content(move || {
                    text_field()
                        .theme(theme.clone())
                        .class(class.clone())
                        .name(label.clone())
                        .default_value("Local draft")
                })
            });
        let action_theme = theme.clone();
        glory::ui::view()
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(20.),
            )
            .fill(glory::ui::text().content("Automatic controlled tabs · RTL"))
            .fill(controlled)
            .fill(glory::ui::text().content("Manual local tabs · vertical"))
            .fill(local)
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
                            .theme(action_theme.clone())
                            .label("Rotate members")
                            .on_press(move || {
                                keys.revise(|keys| {
                                    if !keys.is_empty() {
                                        keys.rotate_right(1);
                                    }
                                });
                            }),
                    )
                    .fill(
                        button()
                            .theme(action_theme.clone())
                            .label("Remove alpha")
                            .on_press(move || {
                                keys.revise(|keys| keys.retain(|key| key != "alpha"));
                            }),
                    )
                    .fill(
                        button()
                            .theme(action_theme.clone())
                            .label("Add alpha")
                            .on_press(move || {
                                keys.revise(|keys| {
                                    if !keys.iter().any(|key| key == "alpha") {
                                        keys.push("alpha".to_owned());
                                    }
                                });
                            }),
                    )
                    .fill(
                        button()
                            .theme(action_theme.clone())
                            .label("Empty members")
                            .on_press(move || keys.revise(Vec::clear)),
                    )
                    .fill(
                        button()
                            .theme(action_theme.clone())
                            .label("Restore members")
                            .on_press(move || {
                                keys.revise(|keys| {
                                    *keys = vec![
                                        "alpha".to_owned(),
                                        "beta".to_owned(),
                                        "gamma".to_owned(),
                                    ]
                                });
                            }),
                    )
                    .fill(
                        button()
                            .theme(action_theme.clone())
                            .label("Toggle proposal acceptance")
                            .on_press(move || accept.revise(|value| *value = !*value)),
                    )
                    .fill(
                        button()
                            .theme(action_theme.clone())
                            .label("Toggle root enabled")
                            .on_press(move || enabled.revise(|value| *value = !*value)),
                    )
                    .fill(
                        button()
                            .theme(action_theme.clone())
                            .label("Toggle beta enabled")
                            .on_press(move || beta_enabled.revise(|value| *value = !*value)),
                    )
                    .fill(
                        button()
                            .theme(action_theme.clone())
                            .label("Toggle language")
                            .on_press(move || chinese.revise(|value| *value = !*value)),
                    )
                    .fill(
                        button()
                            .theme(action_theme.clone())
                            .label("Select alpha from source")
                            .on_press(move || {
                                value.revise(|value| *value = "alpha".to_owned())
                            }),
                    )
                    .fill(
                        button()
                            .theme(action_theme)
                            .label("Select gamma from source")
                            .on_press(move || {
                                value.revise(|value| *value = "gamma".to_owned())
                            }),
                    ),
            )
            .fill(
                glory::ui::text()
                    .class("dynamic-tabs-status")
                    .content(status),
            )
            .show_in(scope)
    })
}
