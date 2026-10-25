use glory::prelude::*;
use glory::render::Direction;
use glory::style::{Color, Length, Style};
use sapid::{Theme, alpha_slider, hue_slider};
pub struct State {
    pub hue: Cage<f64>,
    pub alpha: Cage<f64>,
    pub accept: Cage<bool>,
    pub commit_accept: Cage<bool>,
    pub readonly: Cage<bool>,
    pub enabled: Cage<bool>,
    pub color: Cage<Color>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let hue = scope.cage(120.123456789);
    let alpha = scope.cage(0.25);
    let accept = scope.cage(true);
    let commit_accept = scope.cage(false);
    let readonly = scope.cage(false);
    let enabled = scope.cage(true);
    let color = scope.cage(Color(100, 150, 200, 255));
    let changes = scope.cage(0usize);
    let commits = scope.cage(0usize);
    let requested = scope.cage(0.);
    let hue_control = sapid::field("hue")
        .theme(theme.clone())
        .label("Hue channel")
        .description("HSV hue in degrees")
        .form_name("hue")
        .required(true)
        .readonly(readonly)
        .enabled(enabled)
        .control(move |field| {
            hue_slider("main")
                .field(field)
                .value(hue)
                .step(0.25)
                .class("main-hue")
                .style(Style::new().width(Length::Px(320.)))
                .on_value_change(move |v| {
                    changes.revise(|old| *old += 1);
                    requested.revise(|old| *old = v);
                    if accept.get_untracked() {
                        hue.revise(|old| *old = v)
                    }
                })
                .on_value_commit(move |v| {
                    commits.revise(|old| *old += 1);
                    if commit_accept.get_untracked() {
                        hue.revise(|old| *old = v)
                    }
                })
        });
    let alpha_control = sapid::field("alpha")
        .theme(theme.clone())
        .label("Alpha channel")
        .form_name("alpha")
        .readonly(readonly)
        .enabled(enabled)
        .control(move |field| {
            alpha_slider("main")
                .field(field)
                .value(alpha)
                .color(color)
                .class("main-alpha")
                .style(Style::new().width(Length::Px(320.)))
                .on_value_change(move |v| {
                    changes.revise(|old| *old += 1);
                    requested.revise(|old| *old = v);
                    if accept.get_untracked() {
                        alpha.revise(|old| *old = v)
                    }
                })
                .on_value_commit(move |v| {
                    commits.revise(|old| *old += 1);
                    if commit_accept.get_untracked() {
                        alpha.revise(|old| *old = v)
                    }
                })
        });
    let content = view().class("color-sliders-demo").fill((
        sapid::button().theme(theme.clone()).label("Before colors"),
        hue_control,
        alpha_control,
        hue_slider("rtl")
            .theme(theme.clone())
            .name("RTL hue")
            .direction(Direction::Rtl)
            .default_value(180.)
            .class("rtl-hue"),
        alpha_slider("unstyled")
            .name("Unstyled alpha")
            .default_value(0.5)
            .color(Color(0, 0, 255, 10))
            .unstyled()
            .class("unstyled-alpha"),
        sapid::text_field()
            .theme(theme.clone())
            .name("Color notes")
            .default_value("Retain 😀 caret")
            .class("color-notes"),
        text()
            .class("color-sliders-status")
            .content(scope.bond(move || {
                format!(
                    "hue={};alpha={};request={};changes={};commits={}",
                    hue.get(),
                    alpha.get(),
                    requested.get(),
                    changes.get(),
                    commits.get()
                )
            })),
        sapid::button().theme(theme).label("After colors"),
    ));
    (
        content,
        State {
            hue,
            alpha,
            accept,
            commit_accept,
            readonly,
            enabled,
            color,
        },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static COLORS:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_color_sliders(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing color sliders root"))?;
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
        COLORS.with(|slot| *slot.borrow_mut() = Some((app, state.unwrap())));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn color_sliders_state(kind: String, value: String) -> Result {
        COLORS.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("colors unmounted"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "hue" => state
                    .hue
                    .revise(|old| *old = value.parse().unwrap_or(f64::NAN)),
                "alpha" => state
                    .alpha
                    .revise(|old| *old = value.parse().unwrap_or(f64::NAN)),
                "accept" => state.accept.revise(|old| *old = value == "true"),
                "commit_accept" => state.commit_accept.revise(|old| *old = value == "true"),
                "readonly" => state.readonly.revise(|old| *old = value == "true"),
                "enabled" => state.enabled.revise(|old| *old = value == "true"),
                "color" => state.color.revise(|old| {
                    *old = if value == "alternate" {
                        Color(200, 50, 100, 3)
                    } else {
                        Color(100, 150, 200, 255)
                    }
                }),
                _ => return Err(js("unknown colors state")),
            };
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_color_sliders() {
        drop(COLORS.with(|slot| slot.borrow_mut().take()));
    }
}
#[cfg(target_arch = "wasm32")]
pub use browser::*;
