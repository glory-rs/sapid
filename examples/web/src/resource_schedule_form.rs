use glory::prelude::*;
use sapid::prelude::*;
fn date(s: &str) -> CivilDate {
    CivilDate::parse_iso(s).unwrap()
}
fn dt(s: &str) -> LocalDateTime {
    parse_local_date_time(s).unwrap()
}
fn key(s: &str) -> ScheduleKey {
    ScheduleKey::new(s).unwrap()
}
fn data_for(
    window: ScheduleWindow,
    page: ScheduleResourceWindow,
    title: &str,
) -> ResourceScheduleData {
    let make = |id: &str, title: &str, start: &str, end: &str, resource: &str, permissions| {
        ScheduleEvent::new(
            key(id),
            title,
            ScheduleInterval::new(dt(start), dt(end)).unwrap(),
        )
        .unwrap()
        .resource(key(resource))
        .permissions(permissions)
    };
    let permissions = SchedulePermissions {
        select: true,
        move_event: true,
        resize_event: true,
    };
    ResourceScheduleData::new(
        ScheduleData::new(
            ScheduleZone::Civil,
            window,
            [
                make(
                    "planning",
                    title,
                    "2024-02-29T09:00",
                    "2024-02-29T10:00",
                    "room0",
                    permissions,
                ),
                make(
                    "review",
                    "Review",
                    "2024-02-29T09:00",
                    "2024-02-29T10:00",
                    "room1",
                    SchedulePermissions {
                        select: false,
                        move_event: true,
                        resize_event: false,
                    },
                ),
                ScheduleEvent::new(
                    key("holiday"),
                    "Holiday",
                    ScheduleInterval::all_day(date("2024-02-29"), date("2024-03-01")).unwrap(),
                )
                .unwrap()
                .all_day()
                .unwrap()
                .resource(key("room1"))
                .permissions(permissions),
            ],
        )
        .unwrap(),
        ScheduleResources::new((0..5).map(|i| {
            ScheduleResource::new(key(&format!("room{i}")), format!("Room {i}")).unwrap()
        }))
        .unwrap(),
        page,
    )
    .unwrap()
}
fn accept_edit(source: Cage<ResourceScheduleData>, edit: &ScheduleEdit) {
    source.revise(|old| {
        let events = old.data().events().iter().map(|event| {
            if event.key() != &edit.event {
                return event.clone();
            }
            let mut next = ScheduleEvent::new(event.key().clone(), event.title(), edit.proposed)
                .unwrap()
                .permissions(event.edit_permissions())
                .resource(edit.proposed_resource.clone().unwrap());
            if event.is_all_day() {
                next = next.all_day().unwrap()
            }
            if let Some(rule) = event.recurrence() {
                next = next.recurring(rule).unwrap()
            }
            next.except_dates(event.exceptions().iter().copied())
                .unwrap()
        });
        *old = ResourceScheduleData::new(
            ScheduleData::new(old.data().zone().clone(), old.data().window(), events).unwrap(),
            old.resources().clone(),
            old.resource_window(),
        )
        .unwrap();
    });
}
pub struct State {
    pub data: Cage<ResourceScheduleData>,
    pub month: Cage<ResourceScheduleData>,
    pub accept: Cage<bool>,
    pub enabled: Cage<bool>,
    pub readonly: Cage<bool>,
    pub visible: Cage<bool>,
    pub reference: Cage<Option<glory::render::NodeId>>,
    pub selected: Cage<Option<ScheduleSelection>>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let data = scope.cage(data_for(
        ScheduleWindow::days(date("2024-02-29"), 1).unwrap(),
        ScheduleResourceWindow::new(0, 2).unwrap(),
        "Planning",
    ));
    let month = scope.cage(data_for(
        ScheduleWindow::month(YearMonth::new(2024, 2).unwrap()).unwrap(),
        ScheduleResourceWindow::new(0, 2).unwrap(),
        "Month planning",
    ));
    let accept = scope.cage(true);
    let enabled = scope.cage(true);
    let readonly = scope.cage(false);
    let visible = scope.cage(true);
    let reference = scope.cage(None);
    let selected = scope.cage(None);
    let changes = scope.cage(0usize);
    let edits = scope.cage(0usize);
    let requests = scope.cage(0usize);
    let seen = scope.cage(0usize);
    let last = scope.cage(String::new());
    let navigated = scope.cage(0usize);
    let status=scope.bond(move ||format!("first={};count={};selection={};changes={};edits={};requests={};seen={};last={};navigated={}",data.with(|m|m.resource_window().first()),data.with(|m|m.resource_window().count()),selected.with(|v|match v {Some(ScheduleSelection::Event{key,..})=>key.as_str().to_owned(),Some(ScheduleSelection::Slot{interval,resource})=>format!("{}:{}",resource.as_ref().map_or("",|r|r.as_str()),format_local_date_time(interval.start())),None=>String::new()}),changes.get(),edits.get(),requests.get(),seen.get(),last.get(),navigated.get()));
    let main_theme = theme.clone();
    let primary = glory::ui::when(visible, move || {
        resources_day_view("resource-day", data)
            .theme(main_theme.clone())
            .class("primary-resources")
            .name("Resource day")
            .node_ref(reference)
            .viewport_height(300.)
            .enabled(enabled)
            .readonly(readonly)
            .selection(selected)
            .on_selection_change(move |v| {
                changes.revise(|n| *n += 1);
                if accept.get_untracked() {
                    selected.revise(|old| *old = Some(v))
                }
            })
            .on_window_request(move |r| {
                requests.revise(|n| *n += 1);
                if accept.get_untracked() {
                    data.revise(|old| *old = data_for(r.window, r.resource_window, "Planning"))
                }
            })
            .on_edit(move |e| {
                edits.revise(|n| *n += 1);
                last.revise(|v| {
                    *v = format!(
                        "{}:{}",
                        e.proposed_resource.as_ref().unwrap().as_str(),
                        format_local_date_time(e.proposed.start())
                    )
                });
                if accept.get_untracked() {
                    accept_edit(data, &e)
                }
            })
            .on(glory::render::EventKind::Activate, move |_| {
                seen.revise(|n| *n = changes.get_untracked() + edits.get_untracked())
            })
            .on(glory::render::EventKind::Drag, move |event| {
                if matches!(
                    event,
                    Event::Drag {
                        phase: glory::render::DragPhase::Drop,
                        ..
                    }
                ) {
                    seen.revise(|n| *n = changes.get_untracked() + edits.get_untracked())
                }
            })
            .on(glory::render::EventKind::GridNavigate, move |_| {
                navigated.revise(|n| *n += 1)
            })
    });
    let rows = resources_day_view("resource-rows", data)
        .theme(theme.clone())
        .class("row-resources")
        .name("Resource rows")
        .axis(ResourceAxis::Rows)
        .viewport_height(260.)
        .enabled(enabled)
        .readonly(readonly)
        .on_edit(move |e| {
            edits.revise(|n| *n += 1);
            last.revise(|v| {
                *v = format!(
                    "{}:{}",
                    e.proposed_resource.as_ref().unwrap().as_str(),
                    format_local_date_time(e.proposed.start())
                )
            });
            if accept.get_untracked() {
                accept_edit(data, &e)
            }
        });
    let month_widget = resources_month_view("resource-month", month)
        .theme(theme.clone())
        .class("month-resources")
        .name("Resource month")
        .viewport_height(360.)
        .enabled(enabled)
        .readonly(readonly)
        .on_edit(move |e| {
            edits.revise(|n| *n += 1);
            last.revise(|v| {
                *v = format!(
                    "{}:{}",
                    e.proposed_resource.as_ref().unwrap().as_str(),
                    format_local_date_time(e.proposed.start())
                )
            });
            if accept.get_untracked() {
                accept_edit(month, &e)
            }
        });
    let week = data_for(
        ScheduleWindow::days(date("2024-02-26"), 7).unwrap(),
        ScheduleResourceWindow::new(0, 2).unwrap(),
        "Week planning",
    );
    let rtl = data_for(
        ScheduleWindow::days(date("2024-02-29"), 1).unwrap(),
        ScheduleResourceWindow::new(0, 2).unwrap(),
        "RTL planning",
    );
    let unstyled = rtl.clone();
    (
        glory::ui::view().class("resource-schedule-demo").fill((
            sapid::title(1)
                .theme(theme.clone())
                .content("Resource schedule · Day / Week / Month"),
            primary,
            sapid::text()
                .theme(theme.clone())
                .class("resource-schedule-status")
                .content(status),
            text_field()
                .theme(theme.clone())
                .name("Unrelated resource draft")
                .default_value("Stable resource draft"),
            rows,
            month_widget,
            resources_week_view("resource-week", week)
                .theme(theme.clone())
                .name("Resource week")
                .class("week-resources")
                .viewport_height(250.),
            resources_day_view("resource-rtl", rtl)
                .theme(theme.clone())
                .name("RTL resource day")
                .class("rtl-resources")
                .direction(glory::render::Direction::Rtl)
                .viewport_height(220.),
            resources_day_view("resource-unstyled", unstyled)
                .theme(theme)
                .name("Unstyled resource day")
                .class("unstyled-resources")
                .unstyled()
                .readonly(true)
                .viewport_height(220.),
        )),
        State {
            data,
            month,
            accept,
            enabled,
            readonly,
            visible,
            reference,
            selected,
        },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    type Result = std::result::Result<(), JsValue>;
    thread_local! {static APP:RefCell<Option<(glory::web::WebApp,State)>>=const {RefCell::new(None)};}
    fn js(v: impl std::fmt::Display) -> JsValue {
        v.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_resource_schedule(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing resource root"))?;
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
        let old = APP.with(|v| v.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    fn update(f: impl FnOnce(&State)) -> Result {
        APP.with(|v| {
            let v = v.borrow();
            let (app, state) = v.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            f(state);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn resource_schedule_policy(accept: bool, enabled: bool, readonly: bool) -> Result {
        update(|s| {
            s.accept.revise(|v| *v = accept);
            s.enabled.revise(|v| *v = enabled);
            s.readonly.revise(|v| *v = readonly)
        })
    }
    #[wasm_bindgen]
    pub fn resource_schedule_visible(value: bool) -> Result {
        update(|s| s.visible.revise(|v| *v = value))
    }
    #[wasm_bindgen]
    pub fn resource_schedule_is_retired() -> bool {
        APP.with(|s| {
            s.borrow()
                .as_ref()
                .is_none_or(|(_, s)| s.reference.get().is_none())
        })
    }
    #[wasm_bindgen]
    pub fn resource_schedule_reset() -> Result {
        update(|s| {
            s.data.revise(|v| {
                *v = data_for(
                    ScheduleWindow::days(date("2024-02-29"), 1).unwrap(),
                    ScheduleResourceWindow::new(0, 2).unwrap(),
                    "Planning",
                )
            });
            s.month.revise(|v| {
                *v = data_for(
                    ScheduleWindow::month(YearMonth::new(2024, 2).unwrap()).unwrap(),
                    ScheduleResourceWindow::new(0, 2).unwrap(),
                    "Month planning",
                )
            });
            s.selected.revise(|v| *v = None)
        })
    }
    #[wasm_bindgen]
    pub fn resource_schedule_title(title: String) -> Result {
        if title.len() > 256 || title.contains(['\0', '\r', '\n']) {
            return Err(js("title budget"));
        }
        update(|s| {
            s.data
                .revise(|old| *old = data_for(old.data().window(), old.resource_window(), &title))
        })
    }
}
