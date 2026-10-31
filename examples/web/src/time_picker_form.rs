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
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static TIME_PICKER:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(v: impl std::fmt::Display) -> JsValue {
        v.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_time_picker(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing segmented time root"))?;
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
        let old = TIME_PICKER.with(|s| s.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    fn update(f: impl FnOnce(&State)) -> Result {
        TIME_PICKER.with(|s| {
            let s = s.borrow();
            let (app, state) = s.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            f(state);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn time_picker_policy(accept: bool, enabled: bool, readonly: bool) -> Result {
        update(|s| {
            s.accept.revise(|v| *v = accept);
            s.enabled.revise(|v| *v = enabled);
            s.readonly.revise(|v| *v = readonly)
        })
    }
    #[wasm_bindgen]
    pub fn time_picker_value(value: String) -> Result {
        let v = if value.is_empty() {
            None
        } else {
            Some(LocalTime::parse_iso(&value).map_err(js)?)
        };
        update(|s| s.value.revise(|old| *old = v))
    }
    #[wasm_bindgen]
    pub fn time_picker_visible(value: bool) -> Result {
        update(|s| s.visible.revise(|v| *v = value))
    }
    #[wasm_bindgen]
    pub fn time_picker_is_retired() -> bool {
        TIME_PICKER.with(|s| {
            s.borrow()
                .as_ref()
                .is_none_or(|(_, s)| s.reference.get().is_none())
        })
    }
    #[wasm_bindgen]
    pub fn unmount_time_picker() -> Result {
        let old = TIME_PICKER.with(|s| s.borrow_mut().take());
        if let Some((app, _)) = old {
            app.app().clear().map_err(js)?
        }
        Ok(())
    }
}
