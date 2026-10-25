use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::prelude::*;
    fn date(s: &str) -> CivilDate {
        CivilDate::parse_iso(s).unwrap()
    }
    fn dt(s: &str) -> LocalDateTime {
        parse_local_date_time(s).unwrap()
    }
    fn status(v: DateTimeStatus) -> &'static str {
        match v {
            DateTimeStatus::Empty => "Empty",
            DateTimeStatus::Incomplete => "Incomplete",
            DateTimeStatus::Valid(_) => "Valid",
            DateTimeStatus::OutOfBounds(_) => "OutOfBounds",
            DateTimeStatus::Ambiguous { .. } => "Ambiguous",
            DateTimeStatus::Gap => "Gap",
            DateTimeStatus::InvalidResolver => "InvalidResolver",
            DateTimeStatus::InvalidTime(_) => "InvalidTime",
        }
    }
    pub struct State {
        pub value: Cage<Option<LocalDateTime>>,
        pub accept: Cage<bool>,
        pub enabled: Cage<bool>,
        pub readonly: Cage<bool>,
        pub visible: Cage<bool>,
        pub reference: Cage<Option<glory::render::NodeId>>,
        pub reset: Cage<u64>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let value = scope.cage(Some(dt("2024-02-29T10:20")));
        let accept = scope.cage(true);
        let enabled = scope.cage(true);
        let readonly = scope.cage(false);
        let visible = scope.cage(true);
        let reference = scope.cage(None);
        let reset = scope.cage(0u64);
        let draft = scope.cage("Valid".to_owned());
        let changes = scope.cage(0usize);
        let commits = scope.cage(0usize);
        let resolved = scope.cage(String::new());
        let rejects = scope.cage(0usize);
        let submits = scope.cage(0usize);
        let submitted = scope.cage(String::new());
        let summary=scope.bond(move||format!("value={};draft={};changes={};commits={};resolved={};rejects={};submits={};submitted={}",value.get().map(format_local_date_time).unwrap_or_default(),draft.get(),changes.get(),commits.get(),resolved.get(),rejects.get(),submits.get(),submitted.get()));
        let field_theme = theme.clone();
        let primary = glory::ui::when(visible, move || {
            field("date-time-field")
                .theme(field_theme.clone())
                .label("Appointment date and time")
                .required(true)
                .control(move |f| {
                    let policy = DateTimePolicy::application(
                        AmbiguityPolicy::Reject,
                        |local| match local.time().hour() {
                            2 => LocalResolution::Gap,
                            1 => LocalResolution::Ambiguous {
                                earlier: ResolvedDateTime::from_fixed(
                                    local,
                                    FixedOffset::new(120).unwrap(),
                                ),
                                later: ResolvedDateTime::from_fixed(
                                    local,
                                    FixedOffset::new(60).unwrap(),
                                ),
                            },
                            _ => LocalResolution::Unique(ResolvedDateTime::from_fixed(
                                local,
                                FixedOffset::new(60).unwrap(),
                            )),
                        },
                    );
                    date_time_picker(
                        "appointment-date-time",
                        DatesProvider::new(date("2024-02-29")),
                        policy,
                    )
                    .field(f)
                    .class("primary-date-time")
                    .node_ref(reference)
                    .form_name("appointment")
                    .value(value)
                    .enabled(enabled)
                    .readonly(readonly)
                    .reset_revision(reset)
                    .on_value_change(move |v| {
                        changes.revise(|n| *n += 1);
                        if accept.get_untracked() {
                            value.revise(|old| *old = v)
                        }
                    })
                    .on_draft_change(move |v| draft.revise(|s| *s = status(v.status).into()))
                    .on_resolved(move |v| {
                        resolved.revise(|s| {
                            *s = match v.resolution {
                                DateTimeResolution::Civil => "civil".into(),
                                DateTimeResolution::Instant(v) => {
                                    format!("{}", v.offset().minutes())
                                }
                            }
                        })
                    })
                    .on_commit(move |_| commits.revise(|n| *n += 1))
                    .on_reject(move |_| rejects.revise(|n| *n += 1))
                })
        });
        let inline_value = scope.cage(None);
        let inline_draft = scope.cage("Empty".to_owned());
        let inline_summary = scope.bond(move || {
            format!(
                "value={};draft={}",
                inline_value
                    .get()
                    .map(format_local_date_time)
                    .unwrap_or_default(),
                inline_draft.get()
            )
        });
        (
            glory::ui::view().class("date-time-demo").fill((
                sapid::title(1)
                    .theme(theme.clone())
                    .content("DateTimePicker / InlineDateTimePicker"),
                sapid::form("date-time-form")
                    .theme(theme.clone())
                    .on_submit(move |_| {
                        submits.revise(|n| *n += 1);
                        submitted.revise(|s| {
                            *s = value
                                .get_untracked()
                                .map(format_local_date_time)
                                .unwrap_or_default()
                        })
                    })
                    .fill((
                        primary,
                        submit_button()
                            .theme(theme.clone())
                            .label("Save date and time"),
                    )),
                sapid::text()
                    .theme(theme.clone())
                    .class("date-time-summary")
                    .content(summary),
                inline_date_time_picker(
                    "inline-civil",
                    DatesProvider::new(date("2024-02-29")),
                    DateTimePolicy::Civil,
                )
                .class("inline-civil-date-time")
                .name("Inline civil appointment")
                .theme(theme.clone())
                .reset_revision(reset)
                .value(inline_value)
                .on_value_change(move |v| inline_value.revise(|s| *s = v))
                .on_draft_change(move |v| {
                    inline_draft.revise(|s| *s = status(v.status).into())
                }),
                sapid::text()
                    .theme(theme.clone())
                    .class("inline-date-time-summary")
                    .content(inline_summary),
                inline_date_time_picker(
                    "small-civil",
                    DatesProvider::new(date("2024-02-29")),
                    DateTimePolicy::FixedOffset(FixedOffset::new(480).unwrap()),
                )
                .class("small-date-time")
                .name("Small unstyled date time")
                .theme(theme.clone())
                .unstyled()
                .size(Size::ExtraSmall)
                .default_value(Some(dt("2024-02-29T13:05"))),
                text_field()
                    .theme(theme.clone())
                    .name("Unrelated date time draft")
                    .class("date-time-unrelated")
                    .default_value("Stable date time draft"),
                glory::ui::button()
                    .class("date-time-outside")
                    .label("Outside date and time"),
            )),
            State {
                value,
                accept,
                enabled,
                readonly,
                visible,
                reference,
                reset,
            },
        )
    }
}
