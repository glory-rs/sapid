//! Public mask examples. The status reports lengths and counters, not user data.
use glory::prelude::*;
use glory::render::EventKind;
use sapid::{MaskPattern, Theme, mask_input};
pub struct State {
    pub value: Cage<String>,
    pub accept: Cage<bool>,
    pub enabled: Cage<bool>,
    pub readonly: Cage<bool>,
    pub error: Cage<String>,
    pub visible: Cage<bool>,
    pub reference: Cage<Option<glory::render::NodeId>>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let value = scope.cage(String::new());
    let accept = scope.cage(true);
    let enabled = scope.cage(true);
    let readonly = scope.cage(false);
    let error = scope.cage(String::new());
    let visible = scope.cage(true);
    let reference = scope.cage(None);
    let changes = scope.cage(0usize);
    let commits = scope.cage(0usize);
    let rejected = scope.cage(0usize);
    let submits = scope.cage(0usize);
    let staged = scope.cage(false);
    let ordered = scope.cage(true);
    let legacy = scope.cage(0usize);
    let status = scope.bond(move || {
        format!(
            "length={};changes={};commits={};rejected={};submits={};ordered={};legacy={}",
            value.with(String::len),
            changes.get(),
            commits.get(),
            rejected.get(),
            submits.get(),
            ordered.get(),
            legacy.get()
        )
    });
    let primary_theme = theme.clone();
    let primary = glory::ui::when(visible, move || {
        sapid::field("mask-reference")
            .theme(primary_theme.clone())
            .label("Public reference")
            .description("ASCII slots: ##-##; business value omits the separator")
            .required(true)
            .form_name("reference")
            .enabled(enabled)
            .readonly(readonly)
            .error(error)
            .control(move |field| {
                mask_input(MaskPattern::parse("##-##").unwrap())
                    .field(field)
                    .class("primary-mask")
                    .value(value)
                    .node_ref(reference)
                    .on_value_change(move |next| {
                        changes.revise(|n| *n += 1);
                        staged.revise(|v| *v = true);
                        if accept.get_untracked() {
                            value.revise(|v| *v = next);
                        }
                    })
                    .on_commit(move |next| {
                        commits.revise(|n| *n += 1);
                        if accept.get_untracked() {
                            value.revise(|v| *v = next);
                        }
                    })
                    .on_reject(move |_| {
                        rejected.revise(|n| *n += 1);
                        staged.revise(|v| *v = true);
                    })
                    .on(EventKind::InputEdit, move |event| {
                        if !matches!(
                            event,
                            Event::InputEdit(glory::render::InputEdit::Value {
                                composing: true,
                                ..
                            })
                        ) {
                            ordered.revise(|v| *v &= staged.get_untracked());
                            staged.revise(|v| *v = false);
                        }
                    })
                    .on(EventKind::ValueChanged, move |_| legacy.revise(|v| *v += 1))
            })
    });
    (
        sapid::form("mask-demo")
            .theme(theme.clone())
            .on_submit(move |_| submits.revise(|n| *n += 1))
            .fill(
                glory::ui::view().class("mask-input-demo").fill((
                    sapid::title(1).theme(theme.clone()).content("MaskInput"),
                    primary,
                    sapid::text()
                        .theme(theme.clone())
                        .class("mask-input-status")
                        .content(status),
                    mask_input(MaskPattern::parse("+1 (###) ###").unwrap())
                        .theme(theme.clone())
                        .name("Public phone pattern")
                        .class("phone-mask")
                        .default_value("123456"),
                    mask_input(MaskPattern::parse("AA-##-**").unwrap())
                        .theme(theme.clone())
                        .name("Letters and digits")
                        .class("letters-mask")
                        .default_value("aZ09x1"),
                    mask_input(MaskPattern::parse("##-##").unwrap())
                        .theme(theme.clone())
                        .name("Readonly masked reference")
                        .default_value("0012")
                        .readonly(true),
                    mask_input(MaskPattern::parse("##-##").unwrap())
                        .theme(theme.clone())
                        .name("Disabled masked reference")
                        .default_value("0012")
                        .enabled(false),
                    mask_input(MaskPattern::parse("##-##").unwrap())
                        .theme(theme.clone())
                        .name("Plain masked reference")
                        .class("plain-mask")
                        .default_value("0012")
                        .unstyled(),
                    glory::ui::input()
                        .name("Unrelated draft")
                        .class("mask-draft")
                        .value("Stable draft"),
                )),
            ),
        State {
            value,
            accept,
            enabled,
            readonly,
            error,
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
    thread_local! {static MASK:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(v: impl std::fmt::Display) -> JsValue {
        v.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_mask_input(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing mask root"))?;
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
            let (widget, value) = form(scope, theme);
            state = Some(value);
            widget
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        let old = MASK.with(|slot| slot.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn mask_input_policy(accept: bool, enabled: bool, readonly: bool) -> Result {
        MASK.with(|slot| {
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
    pub fn mask_input_text(text: String) -> Result {
        MASK.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.value.revise(|v| *v = text);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn mask_input_error(text: String) -> Result {
        MASK.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            state.error.revise(|v| *v = text);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn mask_input_visible(visible: bool) -> Result {
        MASK.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            state.visible.revise(|v| *v = visible);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn mask_input_is_retired() -> bool {
        MASK.with(|slot| {
            slot.borrow()
                .as_ref()
                .is_none_or(|(_, s)| s.reference.get().is_none())
        })
    }
    #[wasm_bindgen]
    pub fn unmount_mask_input() -> Result {
        let old = MASK.with(|slot| slot.borrow_mut().take());
        if let Some((app, _)) = old {
            app.app().clear().map_err(js)?;
        }
        Ok(())
    }
}
