use glory::ui::{Widget, component, view};
use sapid::{FieldOptions, Theme, question, question_step, questionnaire};
pub fn questionnaire_form(theme: Theme) -> impl Widget {
    component(move |scope| {
        let name = scope.cage(String::new());
        let permission = scope.cage(false);
        let active = scope.cage(0usize);
        let allow = scope.cage(true);
        let proposals = scope.cage(0usize);
        let completed = scope.cage(0usize);
        let result = scope.cage("No answers submitted".to_owned());
        let schema = questionnaire("profile")
            .theme(theme.clone())
            .class("questionnaire-primary")
            .title("Application questionnaire")
            .active(active)
            .step(
                question_step("identity", "Identity").question(
                    question(
                        "name",
                        name,
                        FieldOptions::default().validate(|value: &String| {
                            if value.trim().is_empty() {
                                vec!["Name is required".into()]
                            } else if value.len() > 256 {
                                vec!["Name exceeds 256 bytes".into()]
                            } else {
                                Vec::new()
                            }
                        }),
                        |field, control| {
                            let value = field.value();
                            sapid::text_field()
                                .field(control)
                                .value(value)
                                .on_value_change(move |proposal| {
                                    value.revise(|value| *value = proposal);
                                    field.changed().unwrap();
                                })
                        },
                    )
                    .label("Your name")
                    .description("Application-owned typed String answer")
                    .required(true),
                ),
            )
            .step(
                question_step("permission", "Permission").question(
                    question(
                        "permission",
                        permission,
                        FieldOptions::default().validate(|value: &bool| {
                            if *value {
                                Vec::new()
                            } else {
                                vec!["Permission is required".into()]
                            }
                        }),
                        |field, control| {
                            let value = field.value();
                            sapid::checkbox()
                                .field(control)
                                .label("Grant permission")
                                .checked(value)
                                .on_checked_change(move |proposal| {
                                    value.revise(|value| *value = proposal);
                                    field.changed().unwrap();
                                })
                        },
                    )
                    .label("Permission")
                    .description("Application-owned typed bool answer"),
                ),
            )
            .on_step_change(move |proposal| {
                proposals.revise(|count| *count += 1);
                if allow.get() {
                    active.revise(|value| *value = proposal)
                }
            })
            .on_complete(move |token| {
                if token.is_current() {
                    completed.revise(|count| *count += 1);
                    result.revise(|result| {
                        *result = format!(
                            "Accepted [{}]; permission {}",
                            name.get_untracked(),
                            permission.get_untracked()
                        )
                    });
                    token.complete(Ok(()));
                }
            });
        view()
            .class("questionnaire-fixture")
            .style(
                glory::style::Style::new()
                    .width(glory::style::Length::Percent(100.))
                    .max_width(glory::style::Limit::Px(760.))
                    .display(glory::style::Display::Flex)
                    .flex_direction(glory::style::FlexDirection::Column)
                    .gap(12.),
            )
            .fill((
                schema,
                sapid::button()
                    .theme(theme.clone())
                    .label("Invalidate previous answer")
                    .on_press(move || name.revise(String::clear)),
                sapid::button()
                    .theme(theme.clone())
                    .label("Refuse step changes")
                    .on_press(move || allow.revise(|value| *value = false)),
                sapid::button()
                    .theme(theme.clone())
                    .label("Allow step changes")
                    .on_press(move || allow.revise(|value| *value = true)),
                sapid::text()
                    .theme(theme.clone())
                    .class("questionnaire-status")
                    .content(scope.bond(move || {
                        format!(
                            "Step {}; proposals {}; completed {}",
                            active.get(),
                            proposals.get(),
                            completed.get()
                        )
                    })),
                sapid::text()
                    .theme(theme.clone())
                    .class("questionnaire-result")
                    .content(result),
            ))
            .show_in(scope)
    })
}
