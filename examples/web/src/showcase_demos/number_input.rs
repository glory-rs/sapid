use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use glory::render::EventKind;
    use sapid::{Decimal, NumberLocale, NumberValue, Theme, number_input};
    fn d(value: &str) -> Decimal {
        Decimal::parse(value).unwrap()
    }
    pub struct State {
        pub value: Cage<NumberValue>,
        pub accept: Cage<bool>,
        pub commit_accept: Cage<bool>,
        pub enabled: Cage<bool>,
        pub readonly: Cage<bool>,
        pub error: Cage<String>,
        pub reference: Cage<Option<glory::render::NodeId>>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let value = scope.cage(NumberValue::Decimal(d("0.1")));
        let accept = scope.cage(true);
        let commit_accept = scope.cage(false);
        let enabled = scope.cage(true);
        let readonly = scope.cage(false);
        let error = scope.cage(String::new());
        let reference = scope.cage(None);
        let request = scope.cage(String::new());
        let changes = scope.cage(0usize);
        let commits = scope.cage(0usize);
        let submits = scope.cage(0usize);
        let rejected = scope.cage(0usize);
        let stage = scope.cage(false);
        let ordered = scope.cage(true);
        let status = scope.bond(move || {
            format!(
                "value={:?};request={};changes={};commits={};submits={};rejected={};ordered={}",
                value.get(),
                request.get(),
                changes.get(),
                commits.get(),
                submits.get(),
                rejected.get(),
                ordered.get()
            )
        });
        let primary = sapid::field("number-amount")
            .theme(theme.clone())
            .label("Amount")
            .description("Exact decimal with 18 fractional digits")
            .form_name("amount")
            .required(true)
            .error(error)
            .enabled(enabled)
            .readonly(readonly)
            .control(move |field| {
                number_input()
                    .field(field)
                    .class("primary-number-input")
                    .value(value)
                    .min(Some(d("0")))
                    .max(Some(d("10")))
                    .step(d("0.1"))
                    .page_step(d("1"))
                    .node_ref(reference)
                    .on_value_change(move |v| {
                        request.revise(|r| *r = format!("{v:?}"));
                        changes.revise(|n| *n += 1);
                        stage.revise(|v| *v = true);
                        if accept.get_untracked() {
                            value.revise(|old| *old = v)
                        }
                    })
                    .on_commit(move |v| {
                        commits.revise(|n| *n += 1);
                        if accept.get_untracked() || commit_accept.get_untracked() {
                            value.revise(|old| *old = v)
                        }
                    })
                    .on_reject(move |_| rejected.revise(|n| *n += 1))
                    .on(EventKind::Spin, move |_| {
                        ordered.revise(|v| *v &= stage.get_untracked());
                        stage.revise(|v| *v = false)
                    })
            });
        (
            sapid::form("number-demo")
                .theme(theme.clone())
                .on_submit(move |_| submits.revise(|n| *n += 1))
                .fill(
                    glory::ui::view().class("number-input-demo").fill((
                        sapid::title(1).theme(theme.clone()).content("NumberInput"),
                        primary,
                        sapid::text()
                            .theme(theme.clone())
                            .class("number-input-status")
                            .content(status),
                        number_input()
                            .theme(theme.clone())
                            .name("German amount")
                            .class("german-number-input")
                            .locale(NumberLocale::DeDe)
                            .grouping(true)
                            .default_value(d("1234.5"))
                            .step(d("0.5")),
                        number_input()
                            .theme(theme.clone())
                            .name("Readonly amount")
                            .default_value(d("2.5"))
                            .readonly(true),
                        number_input()
                            .theme(theme.clone())
                            .name("Plain amount")
                            .class("plain-number-input")
                            .default_value(d("2.5"))
                            .unstyled(),
                        number_input()
                            .theme(theme.clone())
                            .name("Fine decimal")
                            .class("fine-number-input")
                            .default_value(d("0.000000000000000001"))
                            .step(d("0.000000000000000001"))
                            .controls(false),
                        glory::ui::input()
                            .name("Number draft")
                            .value("Stable text draft")
                            .class("number-input-draft"),
                    )),
                ),
            State {
                value,
                accept,
                commit_accept,
                enabled,
                readonly,
                error,
                reference,
            },
        )
    }
}
