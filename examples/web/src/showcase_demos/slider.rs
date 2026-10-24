use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use glory::render::{Direction, Event, EventKind, Orientation};
    use glory::style::{Length, Style};
    use sapid::{Theme, slider};
    pub struct State {
        pub value: Cage<f64>,
        pub accept: Cage<bool>,
        pub commit_accept: Cage<bool>,
        pub enabled: Cage<bool>,
        pub readonly: Cage<bool>,
        pub error: Cage<String>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let value = scope.cage(25.);
        let accept = scope.cage(true);
        let commit_accept = scope.cage(false);
        let enabled = scope.cage(true);
        let readonly = scope.cage(false);
        let error = scope.cage(String::new());
        let changes = scope.cage(0usize);
        let commits = scope.cage(0usize);
        let request = scope.cage(25.);
        let phase = scope.cage(String::from("idle"));
        let stage = scope.cage(false);
        let ordered = scope.cage(true);
        let status = scope.bond(move || {
            format!(
                "value={:.2};request={:.2};changes={};commits={};phase={};ordered={}",
                value.get(),
                request.get(),
                changes.get(),
                commits.get(),
                phase.get(),
                ordered.get()
            )
        });
        let value_text = scope.bond(move || Some(format!("{} units", value.get())));
        let horizontal = sapid::field("volume")
            .theme(theme.clone())
            .label("Volume")
            .description("Choose units from zero to one hundred")
            .form_name("volume")
            .required(true)
            .error(error)
            .enabled(enabled)
            .readonly(readonly)
            .control(move |field| {
                slider("volume")
                    .field(field)
                    .class("horizontal-slider")
                    .value(value)
                    .step(1.)
                    .page_step(10.)
                    .value_text(value_text)
                    .style(Style::new().width(Length::Px(400.)))
                    .on_value_change(move |next| {
                        changes.revise(|v| *v += 1);
                        request.revise(|v| *v = next);
                        stage.revise(|v| *v = true);
                        if accept.get_untracked() {
                            value.revise(|v| *v = next)
                        }
                    })
                    .on_value_commit(move |next| {
                        commits.revise(|v| *v += 1);
                        if commit_accept.get_untracked() {
                            value.revise(|v| *v = next)
                        }
                    })
                    .on(EventKind::Range, move |_| {
                        ordered.revise(|v| *v &= stage.get_untracked());
                        stage.revise(|v| *v = false)
                    })
                    .on(EventKind::Drag, move |event| {
                        if let Event::Drag { phase: current, .. } = event {
                            phase.revise(|v| *v = format!("{current:?}"))
                        }
                    })
            });
        let vertical = slider("vertical")
            .theme(theme.clone())
            .name("Vertical level")
            .orientation(Orientation::Vertical)
            .default_value(25.)
            .class("vertical-slider")
            .style(Style::new().height(Length::Px(180.)));
        let rtl = slider("rtl")
            .theme(theme.clone())
            .name("RTL level")
            .direction(Direction::Rtl)
            .default_value(50.)
            .class("rtl-slider")
            .style(Style::new().width(Length::Px(300.)));
        (
            glory::ui::view().class("slider-demo").fill((
                sapid::title(1).theme(theme.clone()).content("Slider"),
                horizontal,
                sapid::text()
                    .theme(theme)
                    .class("slider-status")
                    .content(status),
                vertical,
                rtl,
                glory::ui::input()
                    .name("Slider draft")
                    .class("slider-draft")
                    .value("Preserve this slider draft"),
                glory::ui::button().label("After sliders"),
            )),
            State {
                value,
                accept,
                commit_accept,
                enabled,
                readonly,
                error,
            },
        )
    }
}
