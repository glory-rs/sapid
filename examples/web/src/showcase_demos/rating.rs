use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use glory::render::{Event, EventKind};
    use sapid::{RatingPrecision, Theme, rating};
    pub struct State {
        pub value: Cage<f64>,
        pub accept: Cage<bool>,
        pub enabled: Cage<bool>,
        pub readonly: Cage<bool>,
        pub error: Cage<String>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let value = scope.cage(2.5);
        let accept = scope.cage(true);
        let enabled = scope.cage(true);
        let readonly = scope.cage(false);
        let error = scope.cage(String::new());
        let request = scope.cage(2.5);
        let changes = scope.cage(0usize);
        let stage = scope.cage(false);
        let ordered = scope.cage(true);
        let status = scope.bond(move || {
            format!(
                "value={};request={};changes={};ordered={}",
                value.get(),
                request.get(),
                changes.get(),
                ordered.get()
            )
        });
        let control = sapid::field("quality")
            .theme(theme.clone())
            .label("Quality")
            .description("Choose zero to five stars in half-star increments")
            .enabled(enabled)
            .readonly(readonly)
            .required(true)
            .error(error)
            .control(move |context| {
                rating("quality")
                    .field(context)
                    .class("primary-rating")
                    .precision(RatingPrecision::Half)
                    .clearable(true)
                    .form_name("quality")
                    .value(value)
                    .on_value_change(move |next| {
                        request.revise(|v| *v = next);
                        changes.revise(|v| *v += 1);
                        stage.revise(|v| *v = true);
                        if accept.get_untracked() {
                            value.revise(|v| *v = next)
                        }
                    })
                    .on(EventKind::CheckedChanged, move |event| {
                        if event == Event::CheckedChanged(true) {
                            ordered.revise(|v| *v &= stage.get_untracked());
                            stage.revise(|v| *v = false)
                        }
                    })
            });
        (
            glory::ui::view().class("rating-demo").fill((
                sapid::title(1).theme(theme.clone()).content("Rating"),
                control,
                sapid::text()
                    .theme(theme.clone())
                    .class("rating-status")
                    .content(status),
                rating("whole")
                    .theme(theme.clone())
                    .name("Whole stars")
                    .default_value(3.)
                    .class("whole-rating"),
                rating("readonly")
                    .theme(theme.clone())
                    .name("Readonly stars")
                    .precision(RatingPrecision::Half)
                    .default_value(3.5)
                    .readonly(true)
                    .class("readonly-rating"),
                rating("plain")
                    .theme(theme.clone())
                    .name("Unstyled stars")
                    .precision(RatingPrecision::Half)
                    .default_value(2.5)
                    .unstyled()
                    .class("plain-rating"),
                rating("display")
                    .theme(theme.clone())
                    .name("Current score")
                    .precision(RatingPrecision::Half)
                    .display_only()
                    .value(value)
                    .class("display-rating"),
                glory::ui::input()
                    .name("Rating draft")
                    .class("rating-draft")
                    .value("Preserve this rating draft"),
                sapid::button().theme(theme.clone()).label("After ratings"),
            )),
            State {
                value,
                accept,
                enabled,
                readonly,
                error,
            },
        )
    }
}
