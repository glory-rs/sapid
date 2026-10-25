use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{Theme, field, select_group, select_option};

    pub struct State {
        pub value: Cage<Option<String>>,
        pub open: Cage<bool>,
        pub reject: Cage<bool>,
        pub reject_open: Cage<bool>,
        pub readonly: Cage<bool>,
        pub disabled: Cage<bool>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let value = scope.cage(Some("us".to_owned()));
        let open = scope.cage(false);
        let reject = scope.cage(false);
        let reject_open = scope.cage(false);
        let readonly = scope.cage(false);
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
                button().label("Before choices"),
                field("destination")
                    .theme(theme.clone())
                    .label("Destination")
                    .description("Choose your delivery region.")
                    .required(true)
                    .form_name("destination")
                    .control(move |field| {
                        sapid::select("destination")
                            .field(field)
                            .value(value)
                            .open(open)
                            .enabled(enabled)
                            .readonly(readonly)
                            .clearable(true)
                            .placeholder("Choose a destination")
                            .options([
                                select_option("us", "United States").value("US"),
                                select_option("ca", "Canada").value("CA"),
                                select_option("cl", "Chile").value("CL"),
                            ])
                            .group(select_group("Europe").options([
                                select_option("fr", "France").value("FR"),
                                select_option("de", "Germany").value("DE"),
                            ]))
                            .group(
                                select_group("Unavailable")
                                    .disabled(true)
                                    .option(select_option("jp", "Japan").value("JP")),
                            )
                            .on_value_change(move |proposal| {
                                proposals.revise(|count| *count += 1);
                                if !reject.get() {
                                    value.revise(|old| *old = proposal);
                                }
                            })
                            .on_open_change(move |proposal| {
                                if !reject_open.get() {
                                    open.revise(|old| *old = proposal)
                                }
                            })
                    }),
                sapid::select("long")
                    .theme(theme.clone())
                    .label("Jump to number")
                    .placeholder("Choose a number")
                    .options((0..80).map(|index| {
                        select_option(format!("n{index}"), format!("Number {index:02}"))
                    })),
                text().content(
                    scope.bond(move || format!("Select changes: {}", proposals.get())),
                ),
                button().label("After choices"),
            ));
        (
            sapid::form("custom-select-form")
                .theme(theme.clone())
                .fill(content),
            State {
                value,
                open,
                reject,
                reject_open,
                readonly,
                disabled,
            },
        )
    }
}
