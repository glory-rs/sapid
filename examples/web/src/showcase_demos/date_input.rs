use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use glory::render::{EventKind, InputEdit};
    use sapid::prelude::*;
    fn d(v: &str) -> CivilDate {
        CivilDate::parse_iso(v).unwrap()
    }
    fn label(s: DateDraftStatus) -> &'static str {
        match s {
            DateDraftStatus::Empty => "empty",
            DateDraftStatus::Incomplete => "incomplete",
            DateDraftStatus::Valid(_) => "valid",
            DateDraftStatus::Invalid => "invalid",
            DateDraftStatus::OutOfBounds(_) => "outside",
            DateDraftStatus::TooLong => "too-long",
        }
    }
    pub struct State {
        pub value: Cage<Option<CivilDate>>,
        pub accept: Cage<bool>,
        pub enabled: Cage<bool>,
        pub readonly: Cage<bool>,
        pub visible: Cage<bool>,
        pub reference: Cage<Option<glory::render::NodeId>>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let provider = DatesProvider::new(d("2024-02-29"))
            .today(Some(d("2024-02-29")))
            .bounds(DateBounds::new(Some(d("2024-01-01")), Some(d("2024-12-31"))).unwrap());
        let value = scope.cage(Some(d("2024-02-26")));
        let accept = scope.cage(true);
        let enabled = scope.cage(true);
        let readonly = scope.cage(false);
        let visible = scope.cage(true);
        let reference = scope.cage(None);
        let changes = scope.cage(0usize);
        let commits = scope.cage(0usize);
        let parses = scope.cage(0usize);
        let rejects = scope.cage(0usize);
        let draft = scope.cage(String::new());
        let state = scope.cage("valid".to_owned());
        let ordered = scope.cage(true);
        let submits = scope.cage(0usize);
        let submitted = scope.cage(String::new());
        let status=scope.bond(move ||format!("value={};state={};changes={};commits={};parses={};rejects={};ordered={};submits={};submitted={}",value.get().map(|v|v.to_string()).unwrap_or_default(),state.get(),changes.get(),commits.get(),parses.get(),rejects.get(),ordered.get(),submits.get(),submitted.get()));
        let primary_theme = theme.clone();
        let primary = glory::ui::when(visible, move || {
            field("date-input-field")
                .label("Editable appointment")
                .description("Exact ISO civil date")
                .required(true)
                .form_name("appointment")
                .theme(primary_theme.clone())
                .control(move |field| {
                    date_input("primary-date-input", provider)
                        .field(field)
                        .class("primary-date-input")
                        .node_ref(reference)
                        .value(value)
                        .enabled(enabled)
                        .readonly(readonly)
                        .picker_label("Choose editable appointment")
                        .picker_title("Editable appointment dates")
                        .parser(move |v| {
                            parses.revise(|n| *n += 1);
                            parse_iso_date_draft(v)
                        })
                        .on_draft_change(move |v| {
                            draft.revise(|old| *old = v.text);
                            state.revise(|old| *old = label(v.status).into());
                        })
                        .on_value_change(move |v| {
                            changes.revise(|n| *n += 1);
                            if accept.get_untracked() {
                                value.revise(|old| *old = v);
                            }
                        })
                        .on_commit(move |_| commits.revise(|n| *n += 1))
                        .on_reject(move |s| {
                            rejects.revise(|n| *n += 1);
                            state.revise(|v| *v = label(s).into());
                        })
                        .on(EventKind::InputEdit, move |event| {
                            if let Event::InputEdit(InputEdit::Value {
                                value,
                                composing: false,
                                ..
                            }) = event
                            {
                                ordered.revise(|v| *v &= draft.get_untracked() == value);
                            }
                        })
                })
        });
        (
            glory::ui::view().class("date-input-demo").fill((
                sapid::title(1).theme(theme.clone()).content("DateInput"),
                sapid::form("date-input-form")
                    .theme(theme.clone())
                    .on_submit(move |_| {
                        submits.revise(|n| *n += 1);
                        submitted.revise(|v| {
                            *v = value
                                .get_untracked()
                                .map(|d| d.to_string())
                                .unwrap_or_default()
                        });
                    })
                    .fill((
                        primary,
                        submit_button()
                            .theme(theme.clone())
                            .label("Save civil date"),
                    )),
                sapid::text()
                    .theme(theme.clone())
                    .class("date-input-status")
                    .content(status),
                date_input("custom-date", provider)
                    .with_picker(false)
                    .theme(theme.clone())
                    .name("Custom date parser")
                    .class("custom-date-input")
                    .parser(|v| {
                        if v == "29/02/2024" {
                            DateParse::Date(d("2024-02-29"))
                        } else {
                            parse_iso_date_draft(v)
                        }
                    }),
                date_input("unstyled-date", provider)
                    .with_picker(false)
                    .theme(theme.clone())
                    .unstyled()
                    .size(Size::ExtraSmall)
                    .name("Unstyled date")
                    .class("unstyled-date-input")
                    .default_value(Some(d("2024-02-29"))),
                text_field()
                    .theme(theme.clone())
                    .name("Unrelated date draft")
                    .class("date-draft-editor")
                    .default_value("Stable draft"),
                sapid::button()
                    .theme(theme.clone())
                    .label("Outside editable date")
                    .class("date-input-outside"),
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
