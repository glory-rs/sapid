use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    //! Date picker fixtures use only explicit civil dates and host-owned layers.
    use glory::prelude::*;
    use sapid::prelude::*;
    fn d(v: &str) -> CivilDate {
        CivilDate::parse_iso(v).unwrap()
    }
    pub struct State {
        pub value: Cage<DateSelection>,
        pub open: Cage<bool>,
        pub accept: Cage<bool>,
        pub accept_open: Cage<bool>,
        pub enabled: Cage<bool>,
        pub readonly: Cage<bool>,
        pub visible: Cage<bool>,
        pub reference: Cage<Option<glory::render::NodeId>>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let provider = DatesProvider::new(d("2024-02-29"))
            .today(Some(d("2024-02-29")))
            .bounds(DateBounds::new(Some(d("2023-01-01")), Some(d("2025-12-31"))).unwrap());
        let value = scope.cage(DateSelection::Single(Some(d("2024-02-26"))));
        let open = scope.cage(false);
        let accept = scope.cage(true);
        let accept_open = scope.cage(true);
        let enabled = scope.cage(true);
        let readonly = scope.cage(false);
        let visible = scope.cage(true);
        let reference = scope.cage(None);
        let changes = scope.cage(0usize);
        let opens = scope.cage(0usize);
        let submits = scope.cage(0usize);
        let multi = scope.cage(DateSelection::empty(DatePickerMode::Multiple));
        let range = scope.cage(DateSelection::empty(DatePickerMode::Range));
        let rejected = scope.cage(0usize);
        let status = scope.bond(move || {
            format!(
                "value={};open={};changes={};opens={};submits={};multi={};range={};rejected={}",
                value.get().format_iso(),
                open.get(),
                changes.get(),
                opens.get(),
                submits.get(),
                multi.get().format_iso(),
                range.get().format_iso(),
                rejected.get()
            )
        });
        let field_theme = theme.clone();
        let primary = glory::ui::when(visible, move || {
            field("appointment")
                .label("Appointment")
                .description("Explicit civil date, no timezone")
                .required(true)
                .form_name("appointment")
                .theme(field_theme.clone())
                .control(move |f| {
                    date_picker_input("appointment-picker", provider)
                        .field(f)
                        .class("primary-date-picker")
                        .value(value)
                        .open(open)
                        .enabled(enabled)
                        .readonly(readonly)
                        .node_ref(reference)
                        .trigger_label("Choose appointment")
                        .title("Appointment dates")
                        .close_label("Close appointments")
                        .disable_weekday(Weekday::Saturday)
                        .disable_weekday(Weekday::Sunday)
                        .on_value_change(move |v| {
                            changes.revise(|n| *n += 1);
                            if accept.get_untracked() {
                                value.revise(|old| *old = v);
                            }
                        })
                        .on_open_change(move |v| {
                            opens.revise(|n| *n += 1);
                            if accept_open.get_untracked() {
                                open.revise(|old| *old = v);
                            }
                        })
                })
        });
        let full = SelectedDates::try_from_iter(
            (0..32).map(|i| d("2024-02-01").checked_add_days(i).unwrap()),
        )
        .unwrap();
        (
            glory::ui::view().class("date-picker-demo").fill((
                sapid::title(1)
                    .theme(theme.clone())
                    .content("DatePicker / DatePickerInput"),
                sapid::form("appointment-form")
                    .theme(theme.clone())
                    .on_submit(move |_| submits.revise(|n| *n += 1))
                    .fill((
                        primary,
                        submit_button()
                            .theme(theme.clone())
                            .label("Save appointment"),
                    )),
                sapid::text()
                    .theme(theme.clone())
                    .class("date-picker-status")
                    .content(status),
                date_picker("multiple", provider)
                    .mode(DatePickerMode::Multiple)
                    .theme(theme.clone())
                    .class("multiple-picker")
                    .name("Multiple dates")
                    .value(multi)
                    .on_value_change(move |v| multi.revise(|old| *old = v)),
                date_picker("range", provider)
                    .mode(DatePickerMode::Range)
                    .theme(theme.clone())
                    .class("range-picker")
                    .name("Range endpoints")
                    .value(range)
                    .exclude_date(d("2024-02-28"))
                    .on_value_change(move |v| range.revise(|old| *old = v)),
                date_picker("full", provider)
                    .mode(DatePickerMode::Multiple)
                    .theme(theme.clone())
                    .size(Size::ExtraSmall)
                    .class("full-picker")
                    .name("Bounded multiple dates")
                    .default_value(DateSelection::Multiple(full))
                    .on_reject(move |_| rejected.revise(|n| *n += 1)),
                text_field()
                    .theme(theme.clone())
                    .name("Unrelated picker draft")
                    .class("picker-draft")
                    .default_value("Stable draft"),
                glory::ui::button()
                    .label("Outside date picker")
                    .class("picker-outside"),
            )),
            State {
                value,
                open,
                accept,
                accept_open,
                enabled,
                readonly,
                visible,
                reference,
            },
        )
    }
}
