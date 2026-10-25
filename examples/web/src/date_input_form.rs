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
            glory::ui::button()
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
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static DATE_INPUT:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(v: impl std::fmt::Display) -> JsValue {
        v.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_date_input(hydrate: bool, hue: String, scheme: String) -> Result {
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
        let old = DATE_INPUT.with(|slot| slot.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn date_input_policy(accept: bool, enabled: bool, readonly: bool) -> Result {
        DATE_INPUT.with(|slot| {
            let slot = slot.borrow();
            let (app, s) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            s.accept.revise(|v| *v = accept);
            s.enabled.revise(|v| *v = enabled);
            s.readonly.revise(|v| *v = readonly);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn date_input_value(spelling: String) -> Result {
        let next = if spelling.is_empty() {
            None
        } else {
            Some(CivilDate::parse_iso(&spelling).map_err(js)?)
        };
        DATE_INPUT.with(|slot| {
            let slot = slot.borrow();
            let (app, s) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            s.value.revise(|v| *v = next);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn date_input_visible(visible: bool) -> Result {
        DATE_INPUT.with(|slot| {
            let slot = slot.borrow();
            let (app, s) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            s.visible.revise(|v| *v = visible);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn date_input_is_retired() -> bool {
        DATE_INPUT.with(|slot| {
            slot.borrow()
                .as_ref()
                .is_none_or(|(_, s)| s.reference.get().is_none())
        })
    }
    #[wasm_bindgen]
    pub fn unmount_date_input() -> Result {
        let old = DATE_INPUT.with(|slot| slot.borrow_mut().take());
        if let Some((app, _)) = old {
            app.app().clear().map_err(js)?;
        }
        Ok(())
    }
}
