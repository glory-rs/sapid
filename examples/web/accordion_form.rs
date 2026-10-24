use glory::prelude::*;
use glory::render::{Direction, EventKind};
use glory::style::FlexWrap;
use sapid::{ContentPolicy, Theme, accordion, accordion_item, button, text_field};

pub fn accordion_form(theme: Theme) -> impl Widget {
    component(move |scope| {
        let keys = scope.cage(vec![
            "alpha".to_owned(),
            "beta".to_owned(),
            "gamma".to_owned(),
        ]);
        let open = scope.cage(vec!["alpha".to_owned()]);
        let accept = scope.cage(false);
        let enabled = scope.cage(true);
        let beta_enabled = scope.cage(false);
        let chinese = scope.cage(false);
        let proposals = scope.cage(0usize);
        let factories = scope.cage(0usize);
        let order = scope.cage(String::new());
        let status = scope.bond(move || {
            format!(
                "Proposals: {}; factories: {}; order: {}",
                proposals.get(),
                factories.get(),
                order.get()
            )
        });
        let control_theme = theme.clone();
        let local_theme = theme.clone();
        let controlled = accordion("keyed-sections")
            .theme(theme.clone())
            .class("controlled-dynamic-accordion")
            .direction(Direction::Rtl)
            .enabled(enabled)
            .content_policy(ContentPolicy::KeepMounted)
            .regions(true)
            .open(open)
            .on_open_change(move |next| {
                proposals.revise(|value| *value += 1);
                order.revise(|value| {
                    value.clear();
                    value.push_str("typed");
                });
                if accept.get() {
                    open.revise(|value| *value = next);
                }
            })
            .on(EventKind::Activate, move |_| {
                order.revise(|value| value.push_str(" generic"))
            })
            .items(keys, move |key| {
                factories.revise(|value| *value += 1);
                let word = key.to_owned();
                let label = chinese.map(move |chinese| {
                    if *chinese {
                        format!("章节 {word}")
                    } else {
                        format!("Section {word}")
                    }
                });
                let item_enabled: Lotus<bool> = if key == "beta" {
                    beta_enabled.into()
                } else {
                    true.into()
                };
                let body_theme = control_theme.clone();
                let class = format!("dynamic-editor-{key}");
                accordion_item(key)
                    .label(label)
                    .enabled(item_enabled)
                    .trigger_content(move || {
                        component(move |scope| {
                            glory::ui::text().content("Details: ").show_in(scope)?;
                            glory::ui::text()
                                .class("dynamic-rich-summary")
                                .content(label)
                                .show_in(scope)
                        })
                    })
                    .content(move || {
                        text_field()
                            .theme(body_theme.clone())
                            .class(class.clone())
                            .name(label)
                            .default_value("Initial draft")
                    })
            });
        let local = accordion("local-keyed-sections")
            .theme(theme.clone())
            .class("local-dynamic-accordion")
            .collapsible(false)
            .content_policy(ContentPolicy::Unmount)
            .items(keys, move |key| {
                let label = format!("Local {key}");
                let body_theme = local_theme.clone();
                let class = format!("local-dynamic-editor-{key}");
                accordion_item(key).label(label.clone()).content(move || {
                    text_field()
                        .theme(body_theme.clone())
                        .name(label.clone())
                        .class(class.clone())
                })
            });
        let controls = glory::ui::view()
            .class("dynamic-accordion-controls")
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_wrap(FlexWrap::Wrap)
                    .gap(8.),
            )
            .fill((
                button()
                    .theme(theme.clone())
                    .label("Rotate members")
                    .on_press(move || {
                        keys.revise(|keys| {
                            if !keys.is_empty() {
                                keys.rotate_right(1);
                            }
                        })
                    }),
                button()
                    .theme(theme.clone())
                    .label("Remove alpha")
                    .on_press(move || keys.revise(|keys| keys.retain(|key| key != "alpha"))),
                button()
                    .theme(theme.clone())
                    .label("Add alpha")
                    .on_press(move || {
                        keys.revise(|keys| {
                            if !keys.iter().any(|key| key == "alpha") {
                                keys.push("alpha".to_owned());
                            }
                        })
                    }),
                button()
                    .theme(theme.clone())
                    .label("Empty members")
                    .on_press(move || keys.revise(Vec::clear)),
                button()
                    .theme(theme.clone())
                    .label("Restore members")
                    .on_press(move || {
                        keys.revise(|keys| {
                            *keys =
                                vec!["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()]
                        })
                    }),
                button()
                    .theme(theme.clone())
                    .label("Toggle proposal acceptance")
                    .on_press(move || accept.revise(|value| *value = !*value)),
                button()
                    .theme(theme.clone())
                    .label("Toggle root enabled")
                    .on_press(move || enabled.revise(|value| *value = !*value)),
                button()
                    .theme(theme.clone())
                    .label("Toggle beta enabled")
                    .on_press(move || beta_enabled.revise(|value| *value = !*value)),
                button()
                    .theme(theme.clone())
                    .label("Toggle language")
                    .on_press(move || chinese.revise(|value| *value = !*value)),
                button()
                    .theme(theme.clone())
                    .label("Open gamma from source")
                    .on_press(move || open.revise(|keys| *keys = vec!["gamma".to_owned()])),
            ));
        sapid::stack()
            .gap(16.)
            .class("accordion-dynamic-fixture")
            .fill((
                controlled,
                glory::ui::text()
                    .class("dynamic-accordion-status")
                    .content(status),
                controls,
                local,
            ))
            .show_in(scope)
    })
}
