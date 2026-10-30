use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::prelude::*;
    fn d(s: &str) -> CivilDate {
        CivilDate::parse_iso(s).unwrap()
    }
    fn dt(s: &str) -> LocalDateTime {
        parse_local_date_time(s).unwrap()
    }
    fn data_for(window: ScheduleWindow, title: &str) -> ScheduleData {
        ScheduleData::new(
            ScheduleZone::Civil,
            window,
            (0..5).map(|i| {
                ScheduleEvent::new(
                    ScheduleKey::new(format!("event-{i}")).unwrap(),
                    if i == 0 {
                        title.to_owned()
                    } else {
                        format!("Event {i}")
                    },
                    ScheduleInterval::new(dt("2024-02-28T09:00"), dt("2024-02-29T10:00")).unwrap(),
                )
                .unwrap()
            }),
        )
        .unwrap()
    }
    pub struct State {
        pub data: Cage<ScheduleData>,
        pub accept: Cage<bool>,
        pub enabled: Cage<bool>,
        pub disable_on_navigation: Cage<bool>,
        pub readonly: Cage<bool>,
        pub visible: Cage<bool>,
        pub reference: Cage<Option<glory::render::NodeId>>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let data = scope.cage(data_for(
            ScheduleWindow::month(d("2024-02-01").year_month()).unwrap(),
            "Event 0",
        ));
        let selected = scope.cage(None);
        let accept = scope.cage(true);
        let enabled = scope.cage(true);
        let disable_on_navigation = scope.cage(false);
        let readonly = scope.cage(false);
        let visible = scope.cage(true);
        let reference = scope.cage(None);
        let changes = scope.cage(0usize);
        let requests = scope.cage(0usize);
        let seen = scope.cage(0usize);
        let status = scope.bond(move || {
            format!(
                "window={};selection={};changes={};seen={};requests={}",
                data.with(|v| v.window().start().date().to_string()),
                selected.with(|v| match v {
                    Some(ScheduleSelection::Slot { interval, .. }) =>
                        interval.start().date().to_string(),
                    Some(ScheduleSelection::Event { key, .. }) => key.as_str().to_owned(),
                    None => String::new(),
                }),
                changes.get(),
                seen.get(),
                requests.get()
            )
        });
        let primary_theme = theme.clone();
        let primary = glory::ui::when(visible, move || {
            month_view("work-month", data)
                .theme(primary_theme.clone())
                .class("primary-month")
                .name("Work month")
                .node_ref(reference)
                .selection(selected)
                .enabled(enabled)
                .readonly(readonly)
                .on_selection_change(move |v| {
                    changes.revise(|n| *n += 1);
                    if accept.get_untracked() {
                        selected.revise(|old| *old = Some(v));
                    }
                })
                .on_window_request(move |r| {
                    requests.revise(|n| *n += 1);
                    if accept.get_untracked() {
                        data.revise(|v| *v = data_for(r.window, "Event 0"));
                    }
                })
                .on(glory::render::EventKind::Activate, move |_| {
                    seen.revise(|v| *v = changes.get_untracked())
                })
                .on(glory::render::EventKind::GridNavigate, move |_| {
                    if disable_on_navigation.get_untracked() {
                        enabled.revise(|v| *v = false);
                    }
                })
        });
        let year = data_for(
            ScheduleWindow::year(CivilYear::new(2024).unwrap()).unwrap(),
            "Year event",
        );
        let month = data_for(
            ScheduleWindow::month(d("2024-02-01").year_month()).unwrap(),
            "Mobile event",
        );
        (
            glory::ui::view().class("schedule-calendar-demo").fill((
                sapid::title(1)
                    .theme(theme.clone())
                    .content("MonthView / YearView / MobileMonthView"),
                primary,
                sapid::text()
                    .theme(theme.clone())
                    .class("schedule-calendar-status")
                    .content(status),
                year_view("summary-year", year)
                    .theme(theme.clone())
                    .class("summary-year")
                    .name("Summary year"),
                mobile_month_view("mobile-month", month.clone())
                    .theme(theme.clone())
                    .class("mobile-month")
                    .name("Mobile month"),
                month_view("readonly-month", month.clone())
                    .theme(theme.clone())
                    .class("readonly-month")
                    .name("Readonly unstyled month")
                    .readonly(true)
                    .unstyled(),
                month_view("rtl-month", month)
                    .theme(theme.clone())
                    .class("rtl-month")
                    .name("RTL month")
                    .direction(glory::render::Direction::Rtl),
                text_field()
                    .theme(theme)
                    .name("Unrelated calendar schedule draft")
                    .default_value("Stable calendar schedule draft"),
                glory::ui::button()
                    .class("schedule-calendar-outside")
                    .label("Outside calendar schedule"),
            )),
            State {
                data,
                accept,
                enabled,
                disable_on_navigation,
                readonly,
                visible,
                reference,
            },
        )
    }
}
