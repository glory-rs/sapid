use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{JsonLimits, Theme, json_input};
    pub struct State {
        pub value: Cage<String>,
        pub reject: Cage<bool>,
        pub readonly: Cage<bool>,
        pub enabled: Cage<bool>,
        pub error: Cage<String>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let value = scope.cage(String::from(
            r#"{"z":1e999,"a":9007199254740993123456789,"z":"😀"}"#,
        ));
        let reject = scope.cage(false);
        let readonly = scope.cage(false);
        let enabled = scope.cage(true);
        let error = scope.cage(String::new());
        let changes = scope.cage(0usize);
        let formats = scope.cage(0usize);
        let validity = scope.cage(String::new());
        let main = sapid::field("json-payload")
            .theme(theme.clone())
            .label("Payload")
            .description("JSON request document")
            .form_name("payload")
            .required(true)
            .error(error)
            .readonly(readonly)
            .enabled(enabled)
            .control(move |field| {
                json_input("payload")
                    .field(field)
                    .value(value)
                    .class("payload-json")
                    .on_value_change(move |next| {
                        changes.revise(|old| *old += 1);
                        if !reject.get_untracked() {
                            value.revise(|old| *old = next)
                        }
                    })
                    .on_format(move |_| formats.revise(|old| *old += 1))
                    .on_validation(move |state| {
                        validity.revise(|old| *old = format!("{state:?}"))
                    })
            });
        let root = view().class("json-demo").fill((
            sapid::button().theme(theme.clone()).label("Before JSON"),
            main,
            json_input("local")
                .theme(theme.clone())
                .name("Local draft")
                .class("local-json")
                .default_value("{")
                .format_label("Format local"),
            json_input("limited")
                .theme(theme.clone())
                .name("Limited JSON")
                .class("limited-json")
                .default_value("[1,2,3]")
                .limits(JsonLimits { bytes: 8, depth: 8 })
                .format_label("Format limited"),
            json_input("unstyled")
                .theme(theme.clone())
                .name("Unstyled JSON")
                .class("unstyled-json")
                .default_value("[]")
                .unstyled()
                .show_formatter(false),
            text().class("json-status").content(scope.bond(move || {
                format!(
                    "changes={};formats={};state={}",
                    changes.get(),
                    formats.get(),
                    validity.get()
                )
            })),
            sapid::button().theme(theme).label("After JSON"),
        ));
        (
            root,
            State {
                value,
                reject,
                readonly,
                enabled,
                error,
            },
        )
    }
}
