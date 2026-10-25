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
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static CALENDAR:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(v: impl std::fmt::Display) -> JsValue {
        v.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_calendar(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing calendar root"))?;
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
            let (widget, value) = form(scope, theme);
            state = Some(value);
            widget
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        let old = CALENDAR.with(|slot| slot.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn calendar_policy(
        accept: bool,
        accept_month: bool,
        enabled: bool,
        readonly: bool,
    ) -> Result {
        CALENDAR.with(|slot| {
            let slot = slot.borrow();
            let (app, s) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            s.accept.revise(|v| *v = accept);
            s.accept_month.revise(|v| *v = accept_month);
            s.enabled.revise(|v| *v = enabled);
            s.readonly.revise(|v| *v = readonly);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn calendar_value(value: String) -> Result {
        let next = if value.is_empty() {
            None
        } else {
            Some(CivilDate::parse_iso(&value).map_err(js)?)
        };
        CALENDAR.with(|slot| {
            let slot = slot.borrow();
            let (app, s) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            s.value.revise(|v| *v = next);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn calendar_month(year: u16, month: u8) -> Result {
        let next = YearMonth::new(year, month).map_err(js)?;
        CALENDAR.with(|slot| {
            let slot = slot.borrow();
            let (app, s) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            s.month.revise(|v| *v = next);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn calendar_locale(chinese: bool, sunday: bool) -> Result {
        CALENDAR.with(|slot| {
            let slot = slot.borrow();
            let (app, s) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            s.provider.revise(|v| {
                *v = v
                    .locale(if chinese {
                        DateLocale::ZhCn
                    } else {
                        DateLocale::En
                    })
                    .week_start(if sunday {
                        Weekday::Sunday
                    } else {
                        Weekday::Monday
                    })
            });
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn calendar_visible(visible: bool) -> Result {
        CALENDAR.with(|slot| {
            let slot = slot.borrow();
            let (app, s) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            s.visible.revise(|v| *v = visible);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn calendar_is_retired() -> bool {
        CALENDAR.with(|slot| {
            slot.borrow()
                .as_ref()
                .is_none_or(|(_, s)| s.reference.get().is_none())
        })
    }
    #[wasm_bindgen]
    pub fn unmount_calendar() -> Result {
        let old = CALENDAR.with(|slot| slot.borrow_mut().take());
        if let Some((app, _)) = old {
            app.app().clear().map_err(js)?;
        }
        Ok(())
    }
}
