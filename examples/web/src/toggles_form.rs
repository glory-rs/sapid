use glory::prelude::*;
use sapid::{Theme, ToggleMode, toggle, toggle_group, toggle_item};

pub struct State {
    pub pressed: Cage<bool>,
    pub single: Cage<Vec<String>>,
    pub multiple: Cage<Vec<String>>,
    pub reject: Cage<bool>,
    pub disabled: Cage<bool>,
    pub changes: Cage<usize>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let pressed = scope.cage(false);
    let single = scope.cage(vec!["left".to_owned()]);
    let multiple = scope.cage(vec!["locked".to_owned()]);
    let reject = scope.cage(false);
    let disabled = scope.cage(false);
    let changes = scope.cage(0usize);
    let enabled = scope.bond(move || !disabled.get());
    let widgets = view()
        .style(
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(24.0),
        )
        .fill((
            toggle()
                .theme(theme.clone())
                .class("toggle-bold")
                .label("Bold text")
                .pressed(pressed)
                .enabled(enabled)
                .on_pressed_change(move |value| {
                    changes.revise(|count| *count += 1);
                    if !reject.get() {
                        pressed.revise(|old| *old = value)
                    }
                }),
            toggle_group("text-alignment")
                .theme(theme.clone())
                .class("toggle-alignment")
                .name("Text alignment")
                .value(single)
                .enabled(enabled)
                .items([
                    toggle_item("left").label("Align left").class("toggle-left"),
                    toggle_item("locked")
                        .label("Unavailable alignment")
                        .class("toggle-unavailable")
                        .enabled(false),
                    toggle_item("center")
                        .label("Align center")
                        .class("toggle-center"),
                    toggle_item("right")
                        .label("Align right")
                        .class("toggle-right"),
                ])
                .on_value_change(move |value| {
                    changes.revise(|count| *count += 1);
                    if !reject.get() {
                        single.revise(|old| *old = value)
                    }
                }),
            toggle_group("text-styles")
                .theme(theme)
                .class("toggle-formatting")
                .name("Text styles")
                .orientation(glory::render::Orientation::Vertical)
                .mode(ToggleMode::Multiple)
                .value(multiple)
                .enabled(enabled)
                .items([
                    toggle_item("locked")
                        .label("Inherited style")
                        .class("toggle-inherited")
                        .enabled(false),
                    toggle_item("italic")
                        .label("Italic text")
                        .class("toggle-italic"),
                    toggle_item("underline")
                        .label("Underline text")
                        .class("toggle-underline"),
                ])
                .on_value_change(move |value| {
                    changes.revise(|count| *count += 1);
                    if !reject.get() {
                        multiple.revise(|old| *old = value)
                    }
                }),
            text().class("toggle-status").content(scope.bond(move || {
                format!(
                    "pressed={}; single={:?}; multiple={:?}; changes={}",
                    pressed.get(),
                    single.get(),
                    multiple.get(),
                    changes.get()
                )
            })),
        ));
    (
        widgets,
        State {
            pressed,
            single,
            multiple,
            reject,
            disabled,
            changes,
        },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static TOGGLES:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_toggles(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing toggle root"))?;
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
        let old = TOGGLES.with(|slot| slot.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn toggle_state(kind: String, value: bool) -> Result {
        TOGGLES.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted toggles"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "pressed" => state.pressed.revise(|v| *v = value),
                "reject" => state.reject.revise(|v| *v = value),
                "disabled" => state.disabled.revise(|v| *v = value),
                _ => return Err(js("unknown toggle state")),
            };
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn toggle_value(which: String, value: String) -> Result {
        TOGGLES.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted toggles"))?;
            app.app().pump().map_err(js)?;
            let value = if value.is_empty() {
                vec![]
            } else {
                value.split(',').map(str::to_owned).collect()
            };
            match which.as_str() {
                "single" => state.single.revise(|v| *v = value),
                "multiple" => state.multiple.revise(|v| *v = value),
                _ => return Err(js("unknown toggle group")),
            };
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_toggles() {
        let old = TOGGLES.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
