use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{Theme, field, native_select, select_group, select_option};

    pub struct State {
        pub selected: Cage<Option<String>>,
        pub many: Cage<Vec<String>>,
        pub reject: Cage<bool>,
        pub disabled: Cage<bool>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let selected = scope.cage(Some("starter".to_owned()));
        let many = scope.cage(vec!["orange".into()]);
        let reject = scope.cage(false);
        let disabled = scope.cage(false);
        let proposals = scope.cage(0usize);
        let enabled = scope.bond(move || !disabled.get());
        let content = view()
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(16.0),
            )
            .fill((
                button().label("Before select"),
                field("native-plan")
                    .theme(theme.clone())
                    .label("Subscription plan")
                    .description("Choose a billing tier.")
                    .form_name("plan")
                    .required(true)
                    .control(move |field| {
                        native_select("plan")
                            .field(field)
                            .value(selected)
                            .enabled(enabled)
                            .placeholder("Choose a plan")
                            .options([
                                select_option("starter", "Starter").value("start-value"),
                                select_option("pro", "Professional").value("pro-value"),
                            ])
                            .group(select_group("Unavailable").disabled(true).option(
                                select_option("enterprise", "Enterprise").value("enterprise-value"),
                            ))
                            .on_value_change(move |value| {
                                proposals.revise(|count| *count += 1);
                                if !reject.get() {
                                    selected.revise(|old| *old = value);
                                }
                            })
                    }),
                native_select("fruit")
                    .theme(theme.clone())
                    .label("Favourite fruit")
                    .form_name("fruit")
                    .rows(4)
                    .values(many)
                    .options([
                        select_option("apple", "Apple").value("red"),
                        select_option("orange", "Orange").value("orange-value"),
                        select_option("pear", "Pear").value("green"),
                        select_option("blocked", "Blocked").disabled(true),
                    ])
                    .on_values_change(move |value| {
                        if !reject.get() {
                            many.revise(|old| *old = value)
                        }
                    }),
                native_select("empty")
                    .theme(theme.clone())
                    .label("Optional empty choice")
                    .form_name("empty")
                    .placeholder("Nothing selected")
                    .option(select_option("empty-value", "Explicit empty value").value("")),
                text()
                    .content(scope.bond(move || format!("Select proposals: {}", proposals.get()))),
                button().label("After select"),
            ));
        (
            sapid::form("native-select-form")
                .theme(theme.clone())
                .fill(content),
            State {
                selected,
                many,
                reject,
                disabled,
            },
        )
    }
}
