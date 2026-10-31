use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use glory::render::{Direction, Event, EventKind, Orientation};
    use glory::style::{Length, Style};
    use sapid::{Theme, range_slider};
    pub struct State {
        pub value: Cage<[f64; 2]>,
        pub accept: Cage<bool>,
        pub commit_accept: Cage<bool>,
        pub enabled: Cage<bool>,
        pub readonly: Cage<bool>,
        pub error: Cage<String>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let value = scope.cage([25., 75.]);
        let accept = scope.cage(true);
        let commit_accept = scope.cage(false);
        let enabled = scope.cage(true);
        let readonly = scope.cage(false);
        let error = scope.cage(String::new());
        let changes = scope.cage(0usize);
        let commits = scope.cage(0usize);
        let request = scope.cage([25., 75.]);
        let phase = scope.cage(String::from("idle"));
        let stage = scope.cage(false);
        let ordered = scope.cage(true);
        let status = scope.bond(move || {
            format!(
                "value={:?};request={:?};changes={};commits={};phase={};ordered={}",
                value.get(),
                request.get(),
                changes.get(),
                commits.get(),
                phase.get(),
                ordered.get()
            )
        });
        let value_text = scope.bond(move || value.get().map(|v| Some(format!("{v} units"))));
        let horizontal = sapid::field("price")
            .theme(theme.clone())
            .label("Price range")
            .description("Choose units from zero to one hundred")
            .form_name("price")
            .required(true)
            .error(error)
            .enabled(enabled)
            .readonly(readonly)
            .control(move |field| {
                range_slider("price")
                    .field(field)
                    .labels(["Minimum price".into(), "Maximum price".into()])
                    .minimum_gap(10.)
                    .class("horizontal-range-slider")
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
        let vertical = range_slider("vertical")
            .theme(theme.clone())
            .name("Vertical range")
            .labels(["Minimum vertical".into(), "Maximum vertical".into()])
            .minimum_gap(10.)
            .orientation(Orientation::Vertical)
            .default_value([25., 75.])
            .class("vertical-range-slider")
            .style(Style::new().height(Length::Px(180.)));
        let rtl = range_slider("rtl")
            .theme(theme.clone())
            .name("RTL range")
            .labels(["Minimum RTL".into(), "Maximum RTL".into()])
            .minimum_gap(10.)
            .direction(Direction::Rtl)
            .default_value([25., 75.])
            .class("rtl-range-slider")
            .style(Style::new().width(Length::Px(300.)));
        (
            glory::ui::view().class("range-slider-demo").fill((
                sapid::title(1).theme(theme.clone()).content("Range Slider"),
                horizontal,
                sapid::text()
                    .theme(theme.clone())
                    .class("range-slider-status")
                    .content(status),
                vertical,
                rtl,
                glory::ui::input()
                    .name("Range slider draft")
                    .class("range-slider-draft")
                    .value("Preserve this range draft"),
                sapid::button()
                    .theme(theme.clone())
                    .label("After range sliders"),
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
