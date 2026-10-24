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
        let event = |key: &str, title: &str, start: &str, end: &str| {
            ScheduleEvent::new(
                ScheduleKey::new(key).unwrap(),
                title,
                ScheduleInterval::new(dt(start), dt(end)).unwrap(),
            )
            .unwrap()
        };
        ScheduleData::new(
            ScheduleZone::Civil,
            window,
            [
                event("planning", title, "2024-02-29T09:00", "2024-02-29T10:00"),
                event("review", "Review", "2024-02-29T09:30", "2024-02-29T10:30"),
                event(
                    "interview",
                    "Interview",
                    "2024-02-29T09:45",
                    "2024-02-29T10:30",
                ),
                ScheduleEvent::new(
                    ScheduleKey::new("holiday").unwrap(),
                    "Holiday",
                    ScheduleInterval::all_day(d("2024-03-01"), d("2024-03-02")).unwrap(),
                )
                .unwrap()
                .all_day()
                .unwrap(),
            ],
        )
        .unwrap()
    }
    pub struct State {
        pub data: Cage<ScheduleData>,
        pub accept: Cage<bool>,
        pub enabled: Cage<bool>,
        pub readonly: Cage<bool>,
        pub visible: Cage<bool>,
        pub reference: Cage<Option<glory::render::NodeId>>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let data = scope.cage(data_for(
            ScheduleWindow::days(d("2024-02-26"), 7).unwrap(),
            "Planning",
        ));
        let selected = scope.cage(None);
        let accept = scope.cage(true);
        let enabled = scope.cage(true);
        let readonly = scope.cage(false);
        let visible = scope.cage(true);
        let reference = scope.cage(None);
        let changes = scope.cage(0usize);
        let seen = scope.cage(0usize);
        let requests = scope.cage(0usize);
        let navigated = scope.cage(0usize);
        let summary = scope.bond(move || {
            format!(
                "window={};selection={};changes={};seen={};requests={};navigated={}",
                data.with(|v| v.window().start().date().to_string()),
                selected.with(|v| match v {
                    Some(ScheduleSelection::Slot { interval, .. }) =>
                        format_local_date_time(interval.start()),
                    Some(ScheduleSelection::Event { key, .. }) => key.as_str().to_owned(),
                    None => String::new(),
                }),
                changes.get(),
                seen.get(),
                requests.get(),
                navigated.get()
            )
        });
        let primary_theme = theme.clone();
        let primary = glory::ui::when(visible, move || {
            week_view("work-week", data)
                .theme(primary_theme.clone())
                .class("primary-week")
                .name("Work week")
                .node_ref(reference)
                .viewport_height(400.)
                .selection(selected)
                .enabled(enabled)
                .readonly(readonly)
                .on_selection_change(move |v| {
                    changes.revise(|n| *n += 1);
                    if accept.get_untracked() {
                        selected.revise(|old| *old = Some(v));
                    }
                })
                .on_window_request(move |request| {
                    requests.revise(|n| *n += 1);
                    if accept.get_untracked() {
                        data.revise(|v| *v = data_for(request.window, "Planning"));
                    }
                })
                .on(glory::render::EventKind::Activate, move |_| {
                    seen.revise(|v| *v = changes.get_untracked())
                })
                .on(glory::render::EventKind::GridNavigate, move |_| {
                    navigated.revise(|n| *n += 1)
                })
        });
        let day = data_for(
            ScheduleWindow::days(d("2024-02-29"), 1).unwrap(),
            "Day planning",
        );
        let rtl = data_for(
            ScheduleWindow::days(d("2024-02-26"), 7).unwrap(),
            "RTL planning",
        );
        (
            glory::ui::view().class("schedule-timeline-demo").fill((
                sapid::title(1)
                    .theme(theme.clone())
                    .content("DayView / WeekView"),
                primary,
                sapid::text()
                    .theme(theme.clone())
                    .class("schedule-timeline-status")
                    .content(summary),
                day_view("single-day", day.clone())
                    .theme(theme.clone())
                    .class("single-day")
                    .name("Leap day")
                    .viewport_height(240.),
                day_view("readonly-day", day)
                    .theme(theme.clone())
                    .class("readonly-day")
                    .name("Readonly unstyled day")
                    .readonly(true)
                    .unstyled()
                    .column_width(300.)
                    .viewport_height(200.),
                week_view("rtl-week", rtl)
                    .theme(theme.clone())
                    .class("rtl-week")
                    .name("RTL week")
                    .direction(glory::render::Direction::Rtl)
                    .viewport_height(200.),
                text_field()
                    .theme(theme)
                    .name("Unrelated timeline draft")
                    .default_value("Stable timeline draft"),
                glory::ui::button()
                    .class("schedule-timeline-outside")
                    .label("Outside timeline"),
            )),
            State {
                data,
                accept,
                enabled,
                readonly,
                visible,
                reference,
            },
        )
    }
}
