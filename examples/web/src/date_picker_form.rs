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
    let full =
        SelectedDates::try_from_iter((0..32).map(|i| d("2024-02-01").checked_add_days(i).unwrap()))
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
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static DATE_PICKER:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(v: impl std::fmt::Display) -> JsValue {
        v.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_date_picker(hydrate: bool, hue: String, scheme: String) -> Result {
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
        let old = DATE_PICKER.with(|slot| slot.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn date_picker_policy(
        accept: bool,
        accept_open: bool,
        enabled: bool,
        readonly: bool,
    ) -> Result {
        DATE_PICKER.with(|slot| {
            let slot = slot.borrow();
            let (app, s) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            s.accept.revise(|v| *v = accept);
            s.accept_open.revise(|v| *v = accept_open);
            s.enabled.revise(|v| *v = enabled);
            s.readonly.revise(|v| *v = readonly);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn date_picker_value(spelling: String) -> Result {
        let next = DateSelection::parse_iso(DatePickerMode::Single, &spelling).map_err(js)?;
        DATE_PICKER.with(|slot| {
            let slot = slot.borrow();
            let (app, s) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            s.value.revise(|v| *v = next);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn date_picker_open(value: bool) -> Result {
        DATE_PICKER.with(|slot| {
            let slot = slot.borrow();
            let (app, s) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            s.open.revise(|v| *v = value);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn date_picker_visible(visible: bool) -> Result {
        DATE_PICKER.with(|slot| {
            let slot = slot.borrow();
            let (app, s) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            s.visible.revise(|v| *v = visible);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn date_picker_is_retired() -> bool {
        DATE_PICKER.with(|slot| {
            slot.borrow()
                .as_ref()
                .is_none_or(|(_, s)| s.reference.get().is_none())
        })
    }
    #[wasm_bindgen]
    pub fn unmount_date_picker() -> Result {
        let old = DATE_PICKER.with(|slot| slot.borrow_mut().take());
        if let Some((app, _)) = old {
            app.app().clear().map_err(js)?;
        }
        Ok(())
    }
}
