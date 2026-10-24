use glory::prelude::*;
use glory::render::{Event, EventKind};
use sapid::{RatingPrecision, Theme, rating};
pub struct State {
    pub value: Cage<f64>,
    pub accept: Cage<bool>,
    pub enabled: Cage<bool>,
    pub readonly: Cage<bool>,
    pub error: Cage<String>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let value = scope.cage(2.5);
    let accept = scope.cage(true);
    let enabled = scope.cage(true);
    let readonly = scope.cage(false);
    let error = scope.cage(String::new());
    let request = scope.cage(2.5);
    let changes = scope.cage(0usize);
    let stage = scope.cage(false);
    let ordered = scope.cage(true);
    let status = scope.bond(move || {
        format!(
            "value={};request={};changes={};ordered={}",
            value.get(),
            request.get(),
            changes.get(),
            ordered.get()
        )
    });
    let control = sapid::field("quality")
        .theme(theme.clone())
        .label("Quality")
        .description("Choose zero to five stars in half-star increments")
        .enabled(enabled)
        .readonly(readonly)
        .required(true)
        .error(error)
        .control(move |context| {
            rating("quality")
                .field(context)
                .class("primary-rating")
                .precision(RatingPrecision::Half)
                .clearable(true)
                .form_name("quality")
                .value(value)
                .on_value_change(move |next| {
                    request.revise(|v| *v = next);
                    changes.revise(|v| *v += 1);
                    stage.revise(|v| *v = true);
                    if accept.get_untracked() {
                        value.revise(|v| *v = next)
                    }
                })
                .on(EventKind::CheckedChanged, move |event| {
                    if event == Event::CheckedChanged(true) {
                        ordered.revise(|v| *v &= stage.get_untracked());
                        stage.revise(|v| *v = false)
                    }
                })
        });
    (
        glory::ui::view().class("rating-demo").fill((
            sapid::title(1).theme(theme.clone()).content("Rating"),
            control,
            sapid::text()
                .theme(theme.clone())
                .class("rating-status")
                .content(status),
            rating("whole")
                .theme(theme.clone())
                .name("Whole stars")
                .default_value(3.)
                .class("whole-rating"),
            rating("readonly")
                .theme(theme.clone())
                .name("Readonly stars")
                .precision(RatingPrecision::Half)
                .default_value(3.5)
                .readonly(true)
                .class("readonly-rating"),
            rating("plain")
                .theme(theme.clone())
                .name("Unstyled stars")
                .precision(RatingPrecision::Half)
                .default_value(2.5)
                .unstyled()
                .class("plain-rating"),
            rating("display")
                .theme(theme.clone())
                .name("Current score")
                .precision(RatingPrecision::Half)
                .display_only()
                .value(value)
                .class("display-rating"),
            glory::ui::input()
                .name("Rating draft")
                .class("rating-draft")
                .value("Preserve this rating draft"),
            glory::ui::button().label("After ratings"),
        )),
        State {
            value,
            accept,
            enabled,
            readonly,
            error,
        },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static RATING:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_rating(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing rating root"))?;
        let theme = Theme::new(
            if hue == "green" {
                sapid::Hue::Green
            } else {
                sapid::Hue::Blue
            },
            if scheme == "dark" {
                sapid::ColorScheme::Dark
            } else {
                sapid::ColorScheme::Light
            },
        );
        let mut source = None;
        let build = |scope: &Scope| {
            let (widget, state) = form(scope, theme);
            source = Some(state);
            widget
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        let old = RATING.with(|slot| slot.borrow_mut().replace((app, source.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn rating_policy(accept: bool, enabled: bool, readonly: bool) -> Result {
        RATING.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.accept.revise(|v| *v = accept);
            state.enabled.revise(|v| *v = enabled);
            state.readonly.revise(|v| *v = readonly);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn rating_value(value: f64) -> Result {
        RATING.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.value.revise(|v| *v = value);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn rating_error(value: String) -> Result {
        RATING.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.error.revise(|v| *v = value);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_rating() {
        let old = RATING.with(|slot| slot.borrow_mut().take());
        drop(old)
    }
}
