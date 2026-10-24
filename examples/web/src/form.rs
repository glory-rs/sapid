use glory::prelude::*;
use sapid::{button, checkbox, text_field};

pub struct State {
    pub value: Cage<String>,
    pub loading: Cage<bool>,
}

pub fn form(scope: &Scope) -> (impl Widget + use<>, State) {
    form_themed(scope, sapid::Theme::default())
}

pub fn form_themed(scope: &Scope, theme: sapid::Theme) -> (impl Widget + use<>, State) {
    let value = scope.cage("initial🙂".to_string());
    let checked = scope.cage(false);
    let loading = scope.cage(false);
    let presses = scope.cage(0_usize);
    let changes = scope.cage(0_usize);
    let commits = scope.cage(0_usize);
    let widget = col()
        .class("demo preview-card")
        .style(
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(16.0)
                .padding(Edges::all(24.0)),
        )
        .fill((
            text_field()
                .theme(theme.clone())
                .class("echo")
                .name("Controlled name")
                .value(value)
                .on_value_change(move |v| {
                    changes.revise(|n| *n += 1);
                    value.revise(|current| *current = v);
                })
                .on_commit(move |_| commits.revise(|n| *n += 1)),
            text_field()
                .theme(theme.clone())
                .class("locked")
                .name("Read only source")
                .value("locked"),
            text_field()
                .theme(theme.clone())
                .class("local")
                .name("Local notes")
                .multiline(true)
                .style(Style::new().min_height(Length::Px(88.0)))
                .default_value("local"),
            checkbox()
                .theme(theme.clone())
                .class("accept-check")
                .label("Accept notifications")
                .checked(checked)
                .on_checked_change(move |v| checked.revise(|current| *current = v)),
            checkbox()
                .theme(theme.clone())
                .class("reject-check")
                .label("Locked checkbox")
                .checked(false),
            checkbox()
                .theme(theme.clone())
                .class("local-check")
                .label("Local checkbox"),
            button()
                .theme(theme.clone())
                .class("action")
                .label("Save")
                .style(Style::new().width(Length::Px(120.0)))
                .loading(loading)
                .on_press(move || presses.revise(|n| *n += 1)),
            text().class("counts").content(scope.bond(move || {
                format!(
                    "presses:{} changes:{} commits:{}",
                    presses.get(),
                    changes.get(),
                    commits.get()
                )
            })),
            text()
                .class("check-state")
                .content(checked.map(|v| format!("checked:{v}"))),
        ));
    (widget, State { value, loading })
}
