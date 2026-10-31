use glory::prelude::*;
use glory::render::{Direction, Event, EventKind};
use glory::style::{Length, Style};
use sapid::{Theme, angle_slider};
pub struct State {
    pub value: Cage<f64>,
    pub accept: Cage<bool>,
    pub commit_accept: Cage<bool>,
    pub enabled: Cage<bool>,
    pub readonly: Cage<bool>,
    pub error: Cage<String>,
    pub wide: Cage<bool>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let value = scope.cage(25.);
    let accept = scope.cage(true);
    let commit_accept = scope.cage(false);
    let enabled = scope.cage(true);
    let readonly = scope.cage(false);
    let error = scope.cage(String::new());
    let wide = scope.cage(true);
    let dimensions = scope.bond(move || {
        Style::new()
            .width(Length::Px(if wide.get() { 240. } else { 160. }))
            .height(Length::Px(if wide.get() { 160. } else { 240. }))
    });
    let changes = scope.cage(0usize);
    let commits = scope.cage(0usize);
    let request = scope.cage(25.);
    let phase = scope.cage(String::from("idle"));
    let stage = scope.cage(false);
    let ordered = scope.cage(true);
    let status = scope.bond(move || {
        format!(
            "value={:.2};request={:.2};changes={};commits={};phase={};ordered={}",
            value.get(),
            request.get(),
            changes.get(),
            commits.get(),
            phase.get(),
            ordered.get()
        )
    });
    let value_text = scope.bond(move || Some(format!("{} degrees", value.get())));
    let horizontal = sapid::field("heading")
        .theme(theme.clone())
        .label("Heading")
        .description("Degrees clockwise from the top")
        .form_name("heading")
        .required(true)
        .error(error)
        .enabled(enabled)
        .readonly(readonly)
        .control(move |field| {
            angle_slider("heading")
                .field(field)
                .class("primary-angle-slider")
                .value(value)
                .step(1.)
                .page_step(10.)
                .value_text(value_text)
                .style(dimensions)
                .on_value_change(move |next| {
                    changes.revise(|v| *v += 1);
                    request.revise(|v| *v = next);
                    stage.revise(|v| *v = true);
                    if accept.get_untracked() {
                        value.revise(|v| *v = next)
                    }
                })
                .on_value_commit(move |next| {
                    commits.revise(|v| *v += 1);
                    if commit_accept.get_untracked() {
                        value.revise(|v| *v = next)
                    }
                })
                .on(EventKind::Range, move |_| {
                    ordered.revise(|v| *v &= stage.get_untracked());
                    stage.revise(|v| *v = false)
                })
                .on(EventKind::Drag, move |event| {
                    if let Event::Drag { phase: current, .. } = event {
                        phase.revise(|v| *v = format!("{current:?}"))
                    }
                })
        });
    let nondivisor = angle_slider("nondivisor")
        .theme(theme.clone())
        .name("Large angular step")
        .step(270.)
        .class("nondivisor-angle-slider");
    let rtl = angle_slider("rtl")
        .theme(theme.clone())
        .name("RTL angle")
        .direction(Direction::Rtl)
        .default_value(50.)
        .class("rtl-angle-slider")
        .style(
            Style::new()
                .width(Length::Px(120.))
                .height(Length::Px(120.)),
        );
    (
        glory::ui::view().class("angle-slider-demo").fill((
            sapid::title(1).theme(theme.clone()).content("AngleSlider"),
            horizontal,
            sapid::text()
                .theme(theme.clone())
                .class("angle-slider-status")
                .content(status),
            nondivisor,
            rtl,
            glory::ui::input()
                .name("Angle draft")
                .class("angle-slider-draft")
                .value("Preserve this angular draft"),
            sapid::button().theme(theme.clone()).label("After angles"),
        )),
        State {
            value,
            accept,
            commit_accept,
            enabled,
            readonly,
            error,
            wide,
        },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static SLIDER:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_angle_slider(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing angle_slider root"))?;
        let theme = crate::themes::select(
            if hue == "green" { "green" } else { "blue" },
            if scheme == "dark" {
                sapid::ColorScheme::Dark
            } else {
                sapid::ColorScheme::Light
            },
        );
        let mut state = None;
        let build = |scope: &Scope| {
            let (widget, source) = form(scope, theme);
            state = Some(source);
            widget
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        let old = SLIDER.with(|slot| slot.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn angle_slider_policy(accept: bool, enabled: bool, readonly: bool) -> Result {
        SLIDER.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.accept.revise(|v| *v = accept);
            state.commit_accept.revise(|v| *v = false);
            state.enabled.revise(|v| *v = enabled);
            state.readonly.revise(|v| *v = readonly);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn angle_slider_commit_policy(enabled: bool) -> Result {
        SLIDER.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.accept.revise(|v| *v = !enabled);
            state.commit_accept.revise(|v| *v = enabled);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn angle_slider_value(value: f64) -> Result {
        SLIDER.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.value.revise(|v| *v = value);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn angle_slider_error(value: String) -> Result {
        SLIDER.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.error.revise(|v| *v = value);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn angle_slider_resize(wide: bool) -> Result {
        SLIDER.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.wide.revise(|value| *value = wide);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_angle_slider() {
        let old = SLIDER.with(|slot| slot.borrow_mut().take());
        drop(old)
    }
}
