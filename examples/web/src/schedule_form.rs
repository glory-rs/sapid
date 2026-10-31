use glory::prelude::*;
use sapid::prelude::*;
fn d(s: &str) -> CivilDate {
    CivilDate::parse_iso(s).unwrap()
}
fn dt(s: &str) -> LocalDateTime {
    parse_local_date_time(s).unwrap()
}
pub fn data_for(window: ScheduleWindow, title: &str) -> ScheduleData {
    let timed = |key: &str, title: &str, start: &str, end: &str| {
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
            timed("planning", title, "2024-02-29T09:00", "2024-02-29T10:00")
                .resource(ScheduleKey::new("room-a").unwrap()),
            timed(
                "standup",
                "Daily standup",
                "2024-01-01T09:30",
                "2024-01-01T10:00",
            )
            .recurring(
                ScheduleRecurrence::new(RecurrenceFrequency::Daily, 1, RecurrenceEnd::Never)
                    .unwrap(),
            )
            .unwrap(),
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
    pub selected: Cage<Option<ScheduleSelection>>,
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
    let requests = scope.cage(0usize);
    let revision = scope.cage(0u64);
    let seen = scope.cage(0usize);
    let summary = scope.bond(move || {
        format!(
            "window={};selection={};changes={};seen={};requests={};revision={}",
            data.with(|v| v.window().start().date().to_string()),
            selected.with(|v| match v {
                Some(ScheduleSelection::Event { key, interval, .. }) => format!(
                    "{}@{}",
                    key.as_str(),
                    format_local_date_time(interval.start())
                ),
                Some(ScheduleSelection::Slot { interval, .. }) =>
                    format_local_date_time(interval.start()),
                None => String::new(),
            }),
            changes.get(),
            seen.get(),
            requests.get(),
            revision.get()
        )
    });
    let primary_theme = theme.clone();
    let primary = glory::ui::when(visible, move || {
        agenda_view("work-agenda", data)
            .theme(primary_theme.clone())
            .class("primary-agenda")
            .name("Work agenda")
            .style(glory::style::Style::new().width(glory::style::Length::Percent(100.)))
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
            .on_window_request(move |request| {
                requests.revise(|n| *n += 1);
                revision.revise(|n| *n = request.revision);
                if accept.get_untracked() {
                    data.revise(|v| *v = data_for(request.window, "Planning"));
                }
            })
            .on(glory::render::EventKind::Activate, move |_| {
                seen.revise(|n| *n = changes.get_untracked())
            })
    });
    (
        glory::ui::view().class("schedule-demo").fill((
            sapid::title(1)
                .theme(theme.clone())
                .content("Schedule / AgendaView"),
            primary,
            sapid::text()
                .theme(theme.clone())
                .class("schedule-status")
                .content(summary),
            agenda_view(
                "empty-agenda",
                ScheduleData::new(
                    ScheduleZone::Civil,
                    ScheduleWindow::days(d("2024-02-29"), 1).unwrap(),
                    [],
                )
                .unwrap(),
            )
            .theme(theme.clone())
            .class("empty-agenda")
            .name("Empty agenda"),
            agenda_view(
                "small-agenda",
                data_for(
                    ScheduleWindow::days(d("2024-02-29"), 1).unwrap(),
                    "Small planning",
                ),
            )
            .theme(theme.clone())
            .class("small-agenda")
            .name("Small unstyled agenda")
            .unstyled()
            .size(Size::ExtraSmall),
            agenda_view(
                "chinese-agenda",
                data_for(
                    ScheduleWindow::days(d("2024-03-01"), 1).unwrap(),
                    "Planning",
                ),
            )
            .theme(theme.clone())
            .class("chinese-agenda")
            .name("Chinese agenda")
            .locale(DateLocale::ZhCn)
            .readonly(true),
            text_field()
                .theme(theme.clone())
                .name("Unrelated schedule draft")
                .class("schedule-draft")
                .default_value("Stable schedule draft"),
            sapid::button().theme(theme.clone())
                .class("schedule-outside")
                .label("Outside schedule"),
        )),
        State {
            data,
            selected,
            accept,
            enabled,
            readonly,
            visible,
            reference,
        },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static APP:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(v: impl std::fmt::Display) -> JsValue {
        v.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_schedule(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing schedule root"))?;
        let theme = crate::themes::select(
            if hue == "green" { "green" } else { "blue" },
            if scheme == "dark" {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            },
        );
        let mut state = None;
        let build = |scope: &Scope| {
            let (w, s) = form(scope, theme);
            state = Some(s);
            w
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        let old = APP.with(|s| s.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    fn update(f: impl FnOnce(&State)) -> Result {
        APP.with(|s| {
            let s = s.borrow();
            let (app, state) = s.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            f(state);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn schedule_policy(accept: bool, enabled: bool, readonly: bool) -> Result {
        update(|s| {
            s.accept.revise(|v| *v = accept);
            s.enabled.revise(|v| *v = enabled);
            s.readonly.revise(|v| *v = readonly);
        })
    }
    #[wasm_bindgen]
    pub fn schedule_program_title(title: String) -> Result {
        if title.len() > 256 {
            return Err(js("title budget"));
        }
        update(|s| {
            let window = s.data.with(|v| v.window());
            s.data.revise(|v| *v = data_for(window, &title));
        })
    }
    #[wasm_bindgen]
    pub fn schedule_reset() -> Result {
        update(|s| {
            s.data.revise(|v| {
                *v = data_for(
                    ScheduleWindow::days(d("2024-02-26"), 7).unwrap(),
                    "Planning",
                )
            });
            s.selected.revise(|v| *v = None);
        })
    }
    #[wasm_bindgen]
    pub fn schedule_visible(value: bool) -> Result {
        update(|s| s.visible.revise(|v| *v = value))
    }
    #[wasm_bindgen]
    pub fn schedule_is_retired() -> bool {
        APP.with(|s| {
            s.borrow()
                .as_ref()
                .is_none_or(|(_, s)| s.reference.get().is_none())
        })
    }
    #[wasm_bindgen]
    pub fn unmount_schedule() -> Result {
        let old = APP.with(|s| s.borrow_mut().take());
        if let Some((app, _)) = old {
            app.app().clear().map_err(js)?;
        }
        Ok(())
    }
}
