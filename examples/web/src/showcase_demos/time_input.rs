use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    //! One native editor per time input, explicit finite wall-clock formats.
    use glory::prelude::*;
    use sapid::prelude::*;
    fn t(value: &str) -> LocalTime {
        LocalTime::parse_iso(value).unwrap()
    }
    pub struct State {
        pub value: Cage<Option<LocalTime>>,
        pub accept: Cage<bool>,
        pub readonly: Cage<bool>,
        pub enabled: Cage<bool>,
        pub visible: Cage<bool>,
        pub reference: Cage<Option<glory::render::NodeId>>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let value = scope.cage(Some(t("09:30")));
        let accept = scope.cage(true);
        let readonly = scope.cage(false);
        let enabled = scope.cage(true);
        let visible = scope.cage(true);
        let reference = scope.cage(None);
        let changes = scope.cage(0usize);
        let commits = scope.cage(0usize);
        let rejects = scope.cage(0usize);
        let submits = scope.cage(0usize);
        let submitted = scope.cage(String::new());
        let draft = scope.cage("Valid".to_owned());
        let order = scope.cage(String::new());
        let status = scope.bond(move || {
        format!(
            "value={};draft={};changes={};commits={};rejects={};submits={};submitted={};order={}",
            value.get().map(|v| v.to_string()).unwrap_or_default(),
            draft.get(),
            changes.get(),
            commits.get(),
            rejects.get(),
            submits.get(),
            submitted.get(),
            order.get()
        )
    });
        let field_theme = theme.clone();
        let primary = glory::ui::when(visible, move || {
            field("meeting-time")
                .theme(field_theme.clone())
                .label("Meeting time")
                .description("Wall-clock time, 09:00–17:00")
                .form_name("meeting_time")
                .required(true)
                .control(move |f| {
                    time_input("meeting-time")
                        .field(f)
                        .class("primary-time-input")
                        .value(value)
                        .readonly(readonly)
                        .enabled(enabled)
                        .node_ref(reference)
                        .bounds(TimeBounds::new(Some(t("09:00")), Some(t("17:00"))).unwrap())
                        .on_draft_change(move |v| {
                            draft.revise(|old| {
                                *old = match v.status {
                                    TimeDraftStatus::Empty => "Empty",
                                    TimeDraftStatus::Incomplete => "Incomplete",
                                    TimeDraftStatus::Valid(_) => "Valid",
                                    TimeDraftStatus::Invalid => "Invalid",
                                    TimeDraftStatus::OutOfBounds(_) => "OutOfBounds",
                                    TimeDraftStatus::TooLong => "TooLong",
                                }
                                .into()
                            });
                            order.revise(|old| *old = "draft".into())
                        })
                        .on_value_change(move |next| {
                            changes.revise(|n| *n += 1);
                            order.revise(|v| v.push_str(",value"));
                            if accept.get_untracked() {
                                value.revise(|old| *old = next)
                            }
                        })
                        .on_commit(move |_| commits.revise(|n| *n += 1))
                        .on_reject(move |_| rejects.revise(|n| *n += 1))
                        .on(glory::render::EventKind::InputEdit, move |_| {
                            order.revise(|v| v.push_str(",generic"))
                        })
                })
        });
        (
            glory::ui::view().class("time-input-demo").fill((
                sapid::title(1)
                    .theme(theme.clone())
                    .content("TimeInput / TimeValue"),
                sapid::form("time-form")
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
                        submit_button().theme(theme.clone()).label("Save time"),
                    )),
                sapid::text()
                    .theme(theme.clone())
                    .class("time-input-status")
                    .content(status),
                time_value(value)
                    .theme(theme.clone())
                    .class("time-value")
                    .empty_label("No meeting time"),
                time_input("twelve-hour")
                    .theme(theme.clone())
                    .name("Twelve-hour seconds")
                    .class("time-twelve")
                    .format(TimeFormat::new(
                        HourCycle::H12,
                        TimePrecision::Second,
                        DateLocale::En,
                    ))
                    .default_value(Some(t("13:05:06"))),
                time_input("chinese-time")
                    .theme(theme.clone())
                    .name("Chinese time")
                    .class("time-chinese")
                    .format(TimeFormat::new(
                        HourCycle::H12,
                        TimePrecision::Second,
                        DateLocale::ZhCn,
                    ))
                    .default_value(Some(t("00:00:01"))),
                time_input("unstyled-time")
                    .theme(theme.clone())
                    .unstyled()
                    .size(Size::ExtraSmall)
                    .name("Unstyled small time")
                    .class("time-unstyled")
                    .default_value(Some(t("10:00"))),
                text_field()
                    .theme(theme.clone())
                    .name("Unrelated time draft")
                    .class("time-draft")
                    .default_value("Stable time draft"),
                glory::ui::button()
                    .label("Outside time input")
                    .class("time-outside"),
            )),
            State {
                value,
                accept,
                readonly,
                enabled,
                visible,
                reference,
            },
        )
    }
}
