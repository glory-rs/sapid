use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use glory::render::TagSeparators;
    use sapid::{FieldOptions, FormState, TagDuplicatePolicy, TagsInputState, Theme};

    pub struct State {
        pub tags: Cage<TagsInputState>,
        pub pills: Cage<TagsInputState>,
        pub reject: Cage<bool>,
        pub readonly: Cage<bool>,
        pub disabled: Cage<bool>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let tags = scope.cage(TagsInputState {
            values: vec!["Rust".into()],
            search: String::new(),
        });
        let pills = scope.cage(TagsInputState {
            values: vec!["stable".into()],
            search: String::new(),
        });
        let reject = scope.cage(false);
        let readonly = scope.cage(false);
        let disabled = scope.cage(false);
        let enabled = scope.bond(move || !disabled.get());
        let changes = scope.cage(0usize);
        let commits = scope.cage(0usize);
        let removes = scope.cage(0usize);
        let submits = scope.cage(0usize);
        let rejection = scope.cage(String::new());
        let model = FormState::new(scope);
        let registered = model
            .register(
                scope,
                "skills",
                tags,
                FieldOptions::default().disabled(disabled).validate(
                    |value: &TagsInputState| {
                        if value.values.is_empty() {
                            vec!["Add at least one skill".into()]
                        } else {
                            vec![]
                        }
                    },
                ),
            )
            .expect("fixture tags field");
        let error = registered.state();
        let errors = scope.bond(move || error.get().errors.join(". "));
        let content = view()
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(16.0),
            )
            .fill((
                button().label("Before tags"),
                sapid::field("skills")
                    .theme(theme.clone())
                    .label("Skills")
                    .description(
                        "Enter, comma and semicolon add a free tag. Up to three skills.",
                    )
                    .form_name("skills")
                    .required(true)
                    .error(errors)
                    .control(move |field| {
                        sapid::tags_input("skills")
                            .field(field)
                            .state(tags)
                            .readonly(readonly)
                            .enabled(enabled)
                            .max_tags(3)
                            .separators(TagSeparators::EnterCommaSemicolon)
                            .on_change(move |proposal| {
                                changes.revise(|count| *count += 1);
                                if !reject.get() {
                                    tags.revise(|value| *value = proposal);
                                }
                            })
                            .on_commit(move |_| commits.revise(|count| *count += 1))
                            .on_remove(move |_| removes.revise(|count| *count += 1))
                            .on_reject(move |reason| {
                                rejection
                                    .revise(|value| *value = format!("Rejected: {reason:?}"))
                            })
                    }),
                sapid::field("labels")
                    .theme(theme.clone())
                    .label("Labels")
                    .description("Pill values may repeat; each is submitted separately.")
                    .form_name("labels")
                    .control(move |field| {
                        sapid::pills_input("labels")
                            .field(field)
                            .state(pills)
                            .readonly(readonly)
                            .enabled(enabled)
                            .duplicate_policy(TagDuplicatePolicy::Allow)
                            .on_change(move |proposal| pills.revise(|value| *value = proposal))
                    }),
                text().content(scope.bond(move || {
                    format!(
                        "Changes: {}, commits: {}, removes: {}, submits: {}",
                        changes.get(),
                        commits.get(),
                        removes.get(),
                        submits.get()
                    )
                })),
                text().content(rejection),
                sapid::submit_button()
                    .theme(theme.clone())
                    .label("Submit tags"),
                button().label("After tags"),
            ));
        (
            sapid::form("free-tags")
                .theme(theme.clone())
                .on_submit(move |_| {
                    model
                        .submit(move |token| {
                            submits.revise(|count| *count += 1);
                            token.complete(Ok(()));
                        })
                        .expect("fixture tags submit");
                })
                .fill(content),
            State {
                tags,
                pills,
                reject,
                readonly,
                disabled,
            },
        )
    }
}
