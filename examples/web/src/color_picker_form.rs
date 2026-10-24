use glory::prelude::*;
use glory::render::Direction;
use sapid::{Hsva, Theme, color_input, color_picker};
pub struct State {
    pub color: Cage<Hsva>,
    pub draft: Cage<String>,
    pub accept: Cage<bool>,
    pub readonly: Cage<bool>,
    pub enabled: Cage<bool>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let color = scope.cage(Hsva {
        hue: 120.,
        saturation: 0.5,
        value: 0.75,
        alpha: 0.4,
    });
    let draft = scope.cage("#6496c880".to_owned());
    let accept = scope.cage(true);
    let readonly = scope.cage(false);
    let enabled = scope.cage(true);
    let changes = scope.cage(0u32);
    let commits = scope.cage(0u32);
    let field = sapid::field("accent")
        .label("Accent hex")
        .theme(theme.clone())
        .required(true)
        .form_name("accent")
        .readonly(readonly)
        .enabled(enabled)
        .control(move |field| {
            color_input("accent")
                .field(field)
                .value(draft)
                .on_value_change(move |value| {
                    if accept.get_untracked() {
                        draft.revise(|old| *old = value);
                    }
                })
        });
    let widget = view().class("color-picker-demo").fill((
        sapid::text_field()
            .theme(theme.clone())
            .name("Picker notes")
            .default_value("Retain 😀 caret")
            .class("picker-notes"),
        color_picker("main")
            .theme(theme.clone())
            .name("Main color")
            .value(color)
            .readonly(readonly)
            .enabled(enabled)
            .class("main-picker")
            .on_value_change(move |value| {
                changes.revise(|count| *count += 1);
                if accept.get_untracked() {
                    color.revise(|old| *old = value);
                }
            })
            .on_value_commit(move |_| commits.revise(|count| *count += 1)),
        field,
        color_picker("rtl")
            .theme(theme.clone())
            .name("RTL color")
            .direction(Direction::Rtl)
            .plane_size(128.)
            .show_alpha(false)
            .class("rtl-picker"),
        color_picker("bare")
            .theme(theme.clone())
            .name("Unstyled color")
            .plane_size(128.)
            .show_alpha(false)
            .unstyled()
            .class("bare-picker"),
        sapid::text_field()
            .theme(theme)
            .name("After picker")
            .default_value("after"),
        text().class("picker-status").content(scope.bond(move || {
            let c = color.get();
            format!(
                "hue={} saturation={} value={} alpha={} changes={} commits={}",
                c.hue,
                c.saturation,
                c.value,
                c.alpha,
                changes.get(),
                commits.get()
            )
        })),
    ));
    (
        widget,
        State {
            color,
            draft,
            accept,
            readonly,
            enabled,
        },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static PICKER:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_color_picker(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing root"))?;
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
        PICKER.with(|slot| *slot.borrow_mut() = Some((app, state.unwrap())));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn color_picker_state(kind: String, value: String) -> Result {
        PICKER.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "accept" => state.accept.revise(|old| *old = value == "true"),
                "readonly" => state.readonly.revise(|old| *old = value == "true"),
                "enabled" => state.enabled.revise(|old| *old = value == "true"),
                "draft" => state.draft.revise(|old| *old = value),
                "hue" => state
                    .color
                    .revise(|old| old.hue = value.parse().unwrap_or(f64::NAN)),
                _ => return Err(js("unknown picker command")),
            };
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_color_picker() {
        drop(PICKER.with(|slot| slot.borrow_mut().take()));
    }
}
#[cfg(target_arch = "wasm32")]
pub use browser::*;
