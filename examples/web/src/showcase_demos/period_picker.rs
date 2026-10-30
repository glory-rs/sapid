use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    //! Joint month/year acceptance fixture; each report exercises both period types.
    use glory::prelude::*;
    use sapid::prelude::*;
    fn date(v: &str) -> CivilDate {
        CivilDate::parse_iso(v).unwrap()
    }
    fn year(v: u16) -> CivilYear {
        CivilYear::new(v).unwrap()
    }
    fn month(v: u8) -> YearMonth {
        YearMonth::new(2024, v).unwrap()
    }
    pub struct State {
        pub month: Cage<MonthSelection>,
        pub year: Cage<YearSelection>,
        pub page: Cage<CivilYear>,
        pub accept: Cage<bool>,
        pub accept_page: Cage<bool>,
        pub enabled: Cage<bool>,
        pub readonly: Cage<bool>,
        pub visible: Cage<bool>,
        pub reference: Cage<Option<glory::render::NodeId>>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let provider = DatesProvider::new(date("2024-02-29")).today(Some(date("2024-02-29")));
        let bounded = provider
            .bounds(DateBounds::new(Some(date("2023-01-01")), Some(date("2025-12-31"))).unwrap());
        let m = scope.cage(MonthSelection::Single(Some(month(2))));
        let y = scope.cage(YearSelection::Single(Some(year(2024))));
        let page = scope.cage(year(2024));
        let accept = scope.cage(true);
        let accept_page = scope.cage(true);
        let enabled = scope.cage(true);
        let readonly = scope.cage(false);
        let visible = scope.cage(true);
        let reference = scope.cage(None);
        let changes = scope.cage(0usize);
        let years = scope.cage(0usize);
        let submits = scope.cage(0usize);
        let rejects = scope.cage(0usize);
        let range = scope.cage(MonthSelection::empty(DatePickerMode::Range));
        let multi = scope.cage(YearSelection::empty(DatePickerMode::Multiple));
        let status = scope.bond(move || {
        format!(
            "month={};year={};page={};changes={};years={};submits={};range={};multi={};rejects={}",
            m.get().format_iso(),
            y.get().format_iso(),
            page.get(),
            changes.get(),
            years.get(),
            submits.get(),
            range.get().format_iso(),
            multi.get().format_iso(),
            rejects.get()
        )
    });
        let field_theme = theme.clone();
        let primary = glory::ui::when(visible, move || {
            field("period-month-field")
                .theme(field_theme.clone())
                .label("Billing month")
                .form_name("billing_month")
                .required(true)
                .control(move |f| {
                    month_picker_input("billing-month", bounded)
                        .field(f)
                        .class("primary-month-picker")
                        .value(m)
                        .year(page)
                        .enabled(enabled)
                        .readonly(readonly)
                        .node_ref(reference)
                        .trigger_label("Choose billing month")
                        .title("Billing months")
                        .close_label("Close billing months")
                        .on_value_change(move |v| {
                            changes.revise(|n| *n += 1);
                            if accept.get_untracked() {
                                m.revise(|old| *old = v)
                            }
                        })
                        .on_year_change(move |v| {
                            years.revise(|n| *n += 1);
                            if accept_page.get_untracked() {
                                page.revise(|old| *old = v)
                            }
                        })
                })
        });
        let full = SelectedYears::try_from_iter((2024..2056).map(year)).unwrap();
        (
            glory::ui::view().class("period-picker-demo").fill((
                sapid::title(1)
                    .theme(theme.clone())
                    .content("MonthPicker / YearPicker"),
                sapid::form("period-form")
                    .theme(theme.clone())
                    .on_submit(move |_| submits.revise(|n| *n += 1))
                    .fill((
                        primary,
                        year_picker_input("billing-year", provider)
                            .theme(theme.clone())
                            .class("primary-year-picker")
                            .name("Billing year")
                            .form_name("billing_year")
                            .value(y)
                            .enabled(enabled)
                            .readonly(readonly)
                            .trigger_label("Choose billing year")
                            .title("Billing years")
                            .close_label("Close billing years")
                            .on_value_change(move |v| {
                                changes.revise(|n| *n += 1);
                                if accept.get_untracked() {
                                    y.revise(|old| *old = v)
                                }
                            }),
                        submit_button().theme(theme.clone()).label("Save periods"),
                    )),
                sapid::text()
                    .theme(theme.clone())
                    .class("period-picker-status")
                    .content(status),
                month_picker("month-range", provider)
                    .theme(theme.clone())
                    .class("month-range")
                    .name("Month range")
                    .mode(DatePickerMode::Range)
                    .value(range)
                    .exclude_value(month(2))
                    .on_value_change(move |v| range.revise(|old| *old = v)),
                year_picker("year-multiple", provider)
                    .theme(theme.clone())
                    .class("year-multiple")
                    .name("Multiple years")
                    .mode(DatePickerMode::Multiple)
                    .value(multi)
                    .on_value_change(move |v| multi.revise(|old| *old = v)),
                year_picker("year-full", provider)
                    .theme(theme.clone())
                    .class("year-full")
                    .size(Size::ExtraSmall)
                    .mode(DatePickerMode::Multiple)
                    .default_value(YearSelection::Multiple(full))
                    .on_reject(move |_| rejects.revise(|n| *n += 1)),
                month_picker("month-rtl", provider)
                    .theme(theme.clone())
                    .class("month-rtl")
                    .direction(glory::render::Direction::Rtl),
                year_picker("year-min", DatesProvider::new(CivilDate::MIN))
                    .theme(theme.clone())
                    .class("year-min"),
                text_field()
                    .theme(theme.clone())
                    .name("Unrelated period draft")
                    .class("period-draft")
                    .default_value("Stable period draft"),
                glory::ui::button()
                    .label("Outside periods")
                    .class("period-outside"),
            )),
            State {
                month: m,
                year: y,
                page,
                accept,
                accept_page,
                enabled,
                readonly,
                visible,
                reference,
            },
        )
    }
}
