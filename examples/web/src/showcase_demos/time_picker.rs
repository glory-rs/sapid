use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    //! Segmented native spinbuttons and actual time Grid share one finite value.
    use glory::prelude::*;
    use sapid::prelude::*;
    fn t(s: &str) -> LocalTime {
        LocalTime::parse_iso(s).unwrap()
    }
    pub struct State {
        pub value: Cage<Option<LocalTime>>,
        pub accept: Cage<bool>,
        pub enabled: Cage<bool>,
        pub readonly: Cage<bool>,
        pub visible: Cage<bool>,
        pub reference: Cage<Option<glory::render::NodeId>>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let value = scope.cage(Some(t("09:30:00")));
        let accept = scope.cage(true);
        let enabled = scope.cage(true);
        let readonly = scope.cage(false);
        let visible = scope.cage(true);
        let reference = scope.cage(None);
        let changes = scope.cage(0usize);
        let commits = scope.cage(0usize);
        let rejects = scope.cage(0usize);
        let draft = scope.cage("Valid".to_owned());
        let submits = scope.cage(0usize);
        let submitted = scope.cage(String::new());
        let format = TimeFormat::new(HourCycle::H24, TimePrecision::Second, DateLocale::En);
        let spec = TimeGridSpec::new(t("09:00"), t("11:30"), 1800, 3).unwrap();
        let status = scope.bond(move || {
            format!(
                "value={};draft={};changes={};commits={};rejects={};submits={};submitted={}",
                value.get().map(|v| v.to_string()).unwrap_or_default(),
                draft.get(),
                changes.get(),
                commits.get(),
                rejects.get(),
                submits.get(),
                submitted.get()
            )
        });
        let field_theme = theme.clone();
        let primary = glory::ui::when(visible, move || {
            field("appointment-parts")
                .theme(field_theme.clone())
                .label("Appointment time")
                .required(true)
                .control(move |f| {
                    time_picker("appointment-time")
                        .field(f)
                        .class("primary-time-picker")
                        .form_name("appointment")
                        .format(format)
                        .grid(spec)
                        .value(value)
                        .node_ref(reference)
                        .enabled(enabled)
                        .readonly(readonly)
                        .bounds(TimeBounds::new(Some(t("09:00")), Some(t("17:00"))).unwrap())
                        .disable_time(t("10:00"))
                        .on_value_change(move |v| {
                            changes.revise(|n| *n += 1);
                            if accept.get_untracked() {
                                value.revise(|old| *old = v)
                            }
                        })
                        .on_commit(move |_| commits.revise(|n| *n += 1))
                        .on_reject(move |_| rejects.revise(|n| *n += 1))
                        .on_draft_change(move |v| {
                            draft.revise(|old| {
                                *old = match v.status {
                                    TimePickerStatus::Empty => "Empty",
                                    TimePickerStatus::Incomplete => "Incomplete",
                                    TimePickerStatus::Valid(_) => "Valid",
                                    TimePickerStatus::Invalid => "Invalid",
                                    TimePickerStatus::OutOfBounds(_) => "OutOfBounds",
                                    TimePickerStatus::Unavailable(_) => "Unavailable",
                                    TimePickerStatus::TooLong => "TooLong",
                                }
                                .into()
                            })
                        })
                })
        });
        (
            glory::ui::view().class("time-picker-demo").fill((
                sapid::title(1)
                    .theme(theme.clone())
                    .content("TimePicker / TimeGrid"),
                sapid::form("appointment-form")
                    .theme(theme.clone())
                    .on_submit(move |_| {
                        submits.revise(|n| *n += 1);
                        submitted.revise(|s| {
                            *s = value
                                .get_untracked()
                                .map(|v| v.to_string())
                                .unwrap_or_default()
                        })
                    })
                    .fill((
                        primary,
                        submit_button()
                            .theme(theme.clone())
                            .label("Save appointment time"),
                    )),
                sapid::text()
                    .theme(theme.clone())
                    .class("time-picker-status")
                    .content(status),
                time_value(value)
                    .format(format)
                    .theme(theme.clone())
                    .class("time-picker-summary")
                    .empty_label("No appointment time"),
                time_picker("twelve-segments")
                    .theme(theme.clone())
                    .class("twelve-time-picker")
                    .name("Twelve-hour picker")
                    .format(TimeFormat::new(
                        HourCycle::H12,
                        TimePrecision::Second,
                        DateLocale::En,
                    ))
                    .default_value(Some(t("13:05:06"))),
                time_picker("chinese-segments")
                    .theme(theme.clone())
                    .class("chinese-time-picker")
                    .name("Chinese picker")
                    .format(TimeFormat::new(
                        HourCycle::H12,
                        TimePrecision::Minute,
                        DateLocale::ZhCn,
                    ))
                    .default_value(Some(t("00:00"))),
                time_picker("small-segments")
                    .theme(theme.clone())
                    .class("small-time-picker")
                    .name("Small unstyled picker")
                    .unstyled()
                    .size(Size::ExtraSmall)
                    .default_value(Some(t("01:02"))),
                time_grid("rtl-time-slots")
                    .theme(theme.clone())
                    .class("rtl-time-grid")
                    .name("RTL time grid")
                    .spec(spec)
                    .direction(glory::render::Direction::Rtl),
                text_field()
                    .theme(theme.clone())
                    .name("Unrelated segmented draft")
                    .class("time-picker-draft")
                    .default_value("Stable segmented draft"),
                sapid::button()
                    .theme(theme.clone())
                    .label("Outside segmented time")
                    .class("time-picker-outside"),
            )),
            State {
                value,
                accept,
                enabled,
                readonly,
                visible,
                reference,
            },
        )
    }
}
