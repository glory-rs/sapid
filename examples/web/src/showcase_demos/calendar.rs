use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    //! Explicit civil dates; the fixture never samples a browser clock or locale.
    use glory::prelude::*;
    use glory::render::EventKind;
    use sapid::prelude::*;
    use std::boxed::Box;
    fn d(value: &str) -> CivilDate {
        CivilDate::parse_iso(value).unwrap()
    }
    pub struct State {
        pub value: Cage<Option<CivilDate>>,
        pub month: Cage<YearMonth>,
        pub provider: Cage<DatesProvider>,
        pub accept: Cage<bool>,
        pub accept_month: Cage<bool>,
        pub enabled: Cage<bool>,
        pub readonly: Cage<bool>,
        pub visible: Cage<bool>,
        pub reference: Cage<Option<glory::render::NodeId>>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let provider = scope.cage(
            DatesProvider::new(d("2024-02-29"))
                .today(Some(d("2024-02-29")))
                .bounds(DateBounds::new(Some(d("2023-01-01")), Some(d("2025-12-31"))).unwrap()),
        );
        let value = scope.cage(Some(d("2024-02-28")));
        let month = scope.cage(d("2024-02-01").year_month());
        let accept = scope.cage(true);
        let accept_month = scope.cage(true);
        let enabled = scope.cage(true);
        let readonly = scope.cage(false);
        let visible = scope.cage(true);
        let reference = scope.cage(None);
        let changes = scope.cage(0usize);
        let months = scope.cage(0usize);
        let staged = scope.cage(false);
        let ordered = scope.cage(true);
        let grid_events = scope.cage(0usize);
        let status = scope.bond(move || {
            format!(
                "value={};month={};changes={};months={};ordered={};grid={}",
                value
                    .get()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "none".into()),
                month.get(),
                changes.get(),
                months.get(),
                ordered.get(),
                grid_events.get()
            )
        });
        let primary_theme = theme.clone();
        let primary = glory::ui::when(visible, move || {
            calendar("primary-calendar", provider)
                .theme(primary_theme.clone())
                .class("primary-calendar")
                .name("Choose a date")
                .value(value)
                .month(month)
                .enabled(enabled)
                .readonly(readonly)
                .node_ref(reference)
                .disable_weekday(Weekday::Saturday)
                .disable_weekday(Weekday::Sunday)
                .exclude_date(d("2024-03-01"))
                .on_value_change(move |next| {
                    changes.revise(|n| *n += 1);
                    staged.revise(|v| *v = true);
                    if accept.get_untracked() {
                        value.revise(|v| *v = next);
                    }
                })
                .on_month_change(move |next| {
                    months.revise(|n| *n += 1);
                    staged.revise(|v| *v = true);
                    if accept_month.get_untracked() {
                        month.revise(|v| *v = next);
                    }
                })
                .on(EventKind::Activate, move |_| {
                    if !readonly.get_untracked() {
                        ordered.revise(|v| *v &= staged.get_untracked());
                        staged.revise(|v| *v = false);
                    }
                })
        });
        let protocol = glory::ui::view()
            .role(Role::Grid)
            .name("Grid input isolation")
            .class("calendar-protocol-grid")
            .semantics(Some(Box::new(Semantics {
                grid_navigation: Some(glory::render::Direction::Ltr),
                ..Default::default()
            })))
            .on(EventKind::GridNavigate, move |_| {
                grid_events.revise(|n| *n += 1)
            })
            .fill(
                glory::ui::view().role(Role::Row).fill((
                    glory::ui::view().role(Role::GridCell).fill(
                        glory::ui::button()
                            .label("Protocol day")
                            .class("calendar-protocol-button"),
                    ),
                    glory::ui::view().role(Role::GridCell).fill(
                        glory::ui::input()
                            .name("Nested grid draft")
                            .class("calendar-nested-draft")
                            .value("Editable"),
                    ),
                )),
            );
        (
            glory::ui::view().class("calendar-demo").fill((
                sapid::title(1)
                    .theme(theme.clone())
                    .content("Calendar / MiniCalendar"),
                primary,
                sapid::text()
                    .theme(theme.clone())
                    .class("calendar-status")
                    .content(status),
                mini_calendar(
                    "mini-calendar",
                    DatesProvider::new(d("2024-02-29"))
                        .week_start(Weekday::Sunday)
                        .today(Some(d("2024-02-29"))),
                )
                .theme(theme.clone())
                .class("rtl-mini-calendar")
                .name("Compact RTL calendar")
                .direction(glory::render::Direction::Rtl),
                mini_calendar(
                    "edge-calendar",
                    DatesProvider::new(CivilDate::MIN).week_start(Weekday::Sunday),
                )
                .theme(theme.clone())
                .name("First supported month")
                .class("edge-calendar")
                .unstyled(),
                protocol,
                glory::ui::input()
                    .name("Unrelated draft")
                    .class("calendar-draft")
                    .value("Stable draft"),
            )),
            State {
                value,
                month,
                provider,
                accept,
                accept_month,
                enabled,
                readonly,
                visible,
                reference,
            },
        )
    }
}
