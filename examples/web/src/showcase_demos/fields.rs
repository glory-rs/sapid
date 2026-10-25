use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use glory::render::EventKind;
    use sapid::{Theme, button, checkbox, field, fieldset, text_field};

    pub struct State {
        pub error: Cage<String>,
        pub readonly: Cage<bool>,
        pub disabled: Cage<bool>,
        pub required: Cage<bool>,
        pub value: Cage<String>,
        pub bio: Cage<String>,
        pub visible: Cage<bool>,
        pub reject: Cage<bool>,
    }
    fn column(gap: f64) -> Style {
        Style::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(gap)
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let error = scope.cage(String::new());
        let readonly = scope.cage(false);
        let disabled = scope.cage(false);
        let required = scope.cage(true);
        let value = scope.cage("alex@studio.design".to_string());
        let edits = scope.cage(0_usize);
        let shared = scope.cage(false);
        let bio = scope.cage("Write a little about yourself.".to_string());
        let visible = scope.cage(false);
        let reject = scope.cage(false);
        let advanced = fieldset("advanced-inputs")
            .legend("Input compositions")
            .disabled(disabled)
            .theme(theme.clone())
            .fill((
                field("biography")
                    .label("Biography")
                    .description("Grows with your text, up to five lines.")
                    .theme(theme.clone())
                    .readonly(readonly)
                    .error(error)
                    .control(move |context| {
                        sapid::textarea()
                            .field(context)
                            .class("input-bio")
                            .autosize(true)
                            .min_rows(2)
                            .max_rows(5)
                            .value(bio)
                            .on_value_change(move |v| {
                                if !reject.get() {
                                    bio.revise(|old| *old = v)
                                }
                            })
                    }),
                field("account-password")
                    .label("Account password")
                    .description("Reveal without replacing the editor.")
                    .theme(theme.clone())
                    .readonly(readonly)
                    .control(move |context| {
                        sapid::password_input()
                            .field(context)
                            .class("input-password")
                            .default_value("example-password")
                            .visible(visible)
                            .on_visibility_change(move |v| {
                                if !reject.get() {
                                    visible.revise(|old| *old = v)
                                }
                            })
                    }),
                field("website")
                    .label("Website")
                    .theme(theme.clone())
                    .control({
                        let theme = theme.clone();
                        move |context| {
                            sapid::input_group()
                                .theme(theme.clone())
                                .class("input-website-group")
                                .start(
                                    sapid::input_addon()
                                        .theme(theme.clone())
                                        .content("https://"),
                                )
                                .input(
                                    text_field()
                                        .field(context)
                                        .class("input-website")
                                        .value("studio.design"),
                                )
                                .end(
                                    sapid::input_section().theme(theme.clone()).fill(
                                        button()
                                            .theme(theme.clone())
                                            .variant(sapid::ButtonVariant::Secondary)
                                            .label("Check address")
                                            .on_press(move || edits.revise(|n| *n += 1)),
                                    ),
                                )
                        }
                    }),
            ));
        let name_field = field("showcase-email")
            .label("Email address")
            .description("We will only use this to send important account updates.")
            .error(error)
            .required(required)
            .readonly(readonly)
            .theme(theme.clone())
            .control(move |context| {
                text_field()
                    .field(context)
                    .class("field-email")
                    .value(value)
                    .on_value_change(move |v| {
                        edits.revise(|n| *n += 1);
                        value.revise(|old| *old = v);
                    })
            });
        let consent = field("showcase-consent")
            .description("Your preference can be changed at any time in account settings.")
            .error(error.map(|message| {
                if message.is_empty() {
                    String::new()
                } else {
                    "Please confirm your preference.".into()
                }
            }))
            .readonly(readonly)
            .required(required)
            .theme(theme.clone())
            .control(move |context| {
                checkbox()
                    .field(context)
                    .class("field-consent")
                    .label("Receive product updates")
                    .checked(shared)
                    .on_checked_change(move |v| shared.revise(|old| *old = v))
            });
        let profile = fieldset("profile")
            .legend("Personal information")
            .disabled(disabled)
            .theme(theme.clone())
            .fill((
                field("profile-name")
                    .label("Full name")
                    .description("As it should appear on your profile.")
                    .theme(theme.clone())
                    .control(|context| {
                        text_field()
                            .field(context)
                            .class("fieldset-name")
                            .default_value("Alex Morgan")
                    }),
                field("profile-company")
                    .label("Company")
                    .theme(theme.clone())
                    .control(|context| {
                        text_field()
                            .field(context)
                            .class("fieldset-company")
                            .default_value("Studio North")
                    }),
                field("profile-public")
                    .label("Public profile")
                    .theme(theme.clone())
                    .control(|context| {
                        checkbox()
                            .field(context)
                            .class("fieldset-check")
                            .label("Visible to other members")
                    }),
                button()
                    .theme(theme.clone())
                    .class("fieldset-button")
                    .label("Save profile"),
            ));
        let standalone_label = SemanticId::new("standalone-label").unwrap();
        let standalone_control = SemanticId::new("standalone-control").unwrap();
        let standalone = col().style(column(8.0)).fill((
            sapid::label(standalone_label.clone(), standalone_control.clone())
                .theme(theme.clone())
                .content("Standalone control label"),
            glory::ui::input()
                .class("standalone-input")
                .style(Style {
                    background: Some(theme.input),
                    color: Some(theme.foreground),
                    ..Style::new()
                        .padding(Edges::all(8.0))
                        .border_width(Edges::all(1.0))
                        .border_style(BorderStyle::Solid)
                        .border_color(theme.border)
                })
                .semantics(Some(Box::new(Semantics {
                    id: Some(standalone_control),
                    labelled_by: Some(standalone_label),
                    ..Default::default()
                }))),
        ));
        let widget = col().class("fields-demo").style(column(0.0)).fill((
        view().class("preview-card").fill((
            view().class("preview-header").fill((text().class("preview-title").content("Interactive preview"),text().content("Controlled · required · accessible"))),
            view().class("preview-body").fill(col().class("field-preview-group").style(column(22.0)).fill((name_field,consent,
                text().class("field-demo-status").content(edits.map(|n| format!("Accepted edits: {n}"))),
            ))),
        )),
        view().class("sample-actions").fill((
            glory::ui::button().class("toggle-error").label("Toggle error").on(EventKind::Activate, move |_| error.revise(|v| *v = if v.is_empty() { "Enter a valid work email address.".into() } else { String::new() })),
            glory::ui::button().class("toggle-readonly").label("Toggle read-only").on(EventKind::Activate, move |_| readonly.revise(|v| *v = !*v)),
            glory::ui::button().class("toggle-required").label("Toggle required").on(EventKind::Activate, move |_| required.revise(|v| *v = !*v)),
        )),
        text().class("sample-note").content("The preview uses real Glory controls. Toggle states to inspect label associations, focus, and error descriptions."),
        text().kind(TextKind::Heading(2)).class("group-heading").semantics(Some(Box::new(Semantics { id: Some(SemanticId::new("groups").unwrap()), ..Default::default() }))).content("Group related fields"),
        text().class("section-copy").content("Fieldset keeps related controls together. Its disabled state applies to every control in the group."),
        view().class("preview-card").fill((
            view().class("preview-header").fill((text().class("preview-title").content("Fieldset"),text().content("Native grouping · inherited disabled state"))),
            view().class("preview-body").fill(profile),
        )),
        view().class("sample-actions").fill(glory::ui::button().class("toggle-fieldset").label("Toggle fieldset disabled").on(EventKind::Activate,move |_| disabled.revise(|v| *v = !*v))),
        text().kind(TextKind::Heading(2)).class("standalone-heading").semantics(Some(Box::new(Semantics { id: Some(SemanticId::new("parts").unwrap()), ..Default::default() }))).content("Composable label parts"),
        text().class("section-copy").content("Label, Description and Error are also available as standalone builders, using stable semantic IDs."),
        view().class("preview-card").fill(view().class("preview-body").fill(standalone)),
        col().fill((text().kind(TextKind::Heading(2)).semantics(Some(Box::new(Semantics { id: Some(SemanticId::new("input-widgets").unwrap()), ..Default::default() }))).content("Textarea, password and input groups"),
        text().class("section-copy").content("Intrinsic multiline sizing, a named password visibility action, and prefixes with independent keyboard actions."),
        view().class("preview-card").fill(view().class("preview-body").fill(advanced)))),
    ));
        (
            widget,
            State {
                error,
                readonly,
                disabled,
                required,
                value,
                bio,
                visible,
                reject,
            },
        )
    }
}
