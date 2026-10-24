use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use glory::render::Direction;
    use glory::style::{Color, Length, Style};
    use sapid::{Theme, alpha_slider, hue_slider};
    pub struct State {
        pub hue: Cage<f64>,
        pub alpha: Cage<f64>,
        pub accept: Cage<bool>,
        pub commit_accept: Cage<bool>,
        pub readonly: Cage<bool>,
        pub enabled: Cage<bool>,
        pub color: Cage<Color>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let hue = scope.cage(120.123456789);
        let alpha = scope.cage(0.25);
        let accept = scope.cage(true);
        let commit_accept = scope.cage(false);
        let readonly = scope.cage(false);
        let enabled = scope.cage(true);
        let color = scope.cage(Color(100, 150, 200, 255));
        let changes = scope.cage(0usize);
        let commits = scope.cage(0usize);
        let requested = scope.cage(0.);
        let hue_control = sapid::field("hue")
            .theme(theme.clone())
            .label("Hue channel")
            .description("HSV hue in degrees")
            .form_name("hue")
            .required(true)
            .readonly(readonly)
            .enabled(enabled)
            .control(move |field| {
                hue_slider("main")
                    .field(field)
                    .value(hue)
                    .step(0.25)
                    .class("main-hue")
                    .style(Style::new().width(Length::Px(320.)))
                    .on_value_change(move |v| {
                        changes.revise(|old| *old += 1);
                        requested.revise(|old| *old = v);
                        if accept.get_untracked() {
                            hue.revise(|old| *old = v)
                        }
                    })
                    .on_value_commit(move |v| {
                        commits.revise(|old| *old += 1);
                        if commit_accept.get_untracked() {
                            hue.revise(|old| *old = v)
                        }
                    })
            });
        let alpha_control = sapid::field("alpha")
            .theme(theme.clone())
            .label("Alpha channel")
            .form_name("alpha")
            .readonly(readonly)
            .enabled(enabled)
            .control(move |field| {
                alpha_slider("main")
                    .field(field)
                    .value(alpha)
                    .color(color)
                    .class("main-alpha")
                    .style(Style::new().width(Length::Px(320.)))
                    .on_value_change(move |v| {
                        changes.revise(|old| *old += 1);
                        requested.revise(|old| *old = v);
                        if accept.get_untracked() {
                            alpha.revise(|old| *old = v)
                        }
                    })
                    .on_value_commit(move |v| {
                        commits.revise(|old| *old += 1);
                        if commit_accept.get_untracked() {
                            alpha.revise(|old| *old = v)
                        }
                    })
            });
        let content = view().class("color-sliders-demo").fill((
            sapid::button().theme(theme.clone()).label("Before colors"),
            hue_control,
            alpha_control,
            hue_slider("rtl")
                .theme(theme.clone())
                .name("RTL hue")
                .direction(Direction::Rtl)
                .default_value(180.)
                .class("rtl-hue"),
            alpha_slider("unstyled")
                .name("Unstyled alpha")
                .default_value(0.5)
                .color(Color(0, 0, 255, 10))
                .unstyled()
                .class("unstyled-alpha"),
            sapid::text_field()
                .theme(theme.clone())
                .name("Color notes")
                .default_value("Retain 😀 caret")
                .class("color-notes"),
            text()
                .class("color-sliders-status")
                .content(scope.bond(move || {
                    format!(
                        "hue={};alpha={};request={};changes={};commits={}",
                        hue.get(),
                        alpha.get(),
                        requested.get(),
                        changes.get(),
                        commits.get()
                    )
                })),
            sapid::button().theme(theme).label("After colors"),
        ));
        (
            content,
            State {
                hue,
                alpha,
                accept,
                commit_accept,
                readonly,
                enabled,
                color,
            },
        )
    }
}
