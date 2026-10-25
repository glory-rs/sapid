use glory::prelude::*;
use sapid::{Theme, hover_card, tooltip};

pub struct State {
    pub tip: Cage<bool>,
    pub card: Cage<bool>,
    pub disabled: Cage<bool>,
    pub reject: Cage<bool>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let tip = scope.cage(false);
    let card = scope.cage(false);
    let disabled = scope.cage(false);
    let reject = scope.cage(false);
    let dismissed = scope.cage(0usize);
    let followed = scope.cage(0usize);
    let enabled = scope.bond(move || !disabled.get());
    let widgets = view()
        .style(
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(18.0),
        )
        .fill((
            tooltip("save-tip")
                .label("Save draft")
                .text("Your draft is saved privately until you publish it.")
                .theme(theme.clone())
                .open(tip)
                .enabled(enabled)
                .open_delay_with(|| crate::menu_form::local_delay(80))
                .close_delay_with(|| crate::menu_form::local_delay(120))
                .on_open_change(move |value| {
                    if !reject.get() {
                        tip.revise(|old| *old = value);
                    }
                })
                .on_dismiss(move |_| dismissed.revise(|value| *value += 1)),
            hover_card("person-card")
                .label("About Ada")
                .name("Ada profile")
                .theme(theme.clone())
                .open(card)
                .open_delay_with(|| crate::menu_form::local_delay(80))
                .close_delay_with(|| crate::menu_form::local_delay(160))
                .on_open_change(move |value| {
                    if !reject.get() {
                        card.revise(|old| *old = value);
                    }
                })
                .content(move || {
                    view()
                        .style(
                            Style::new()
                                .display(Display::Flex)
                                .flex_direction(FlexDirection::Column)
                                .gap(12.0),
                        )
                        .fill((
                            text().content("Ada builds thoughtful developer tools."),
                            link().label("Read Ada profile").target("#ada"),
                            button()
                                .label("Follow Ada")
                                .on(activate, move |_| followed.revise(|value| *value += 1)),
                        ))
                }),
            tooltip("search-tip")
                .text("Search by title or author.")
                .theme(theme)
                .target(|target| target.input(input().name("Search").value("initial query"))),
            text().content(scope.bond(move || format!("Dismissed tips: {}", dismissed.get()))),
            text().content(scope.bond(move || format!("Followed: {}", followed.get()))),
            button().label("Outside hints"),
        ));
    (
        widgets,
        State {
            tip,
            card,
            disabled,
            reject,
        },
    )
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static HINTS:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_hints(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("app"))
            .ok_or_else(|| js("missing hints root"))?;
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
        HINTS.with(|slot| *slot.borrow_mut() = Some((app, state.unwrap())));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn hints_state(kind: String, value: bool) -> Result {
        HINTS.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("hints unmounted"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "tip" => state.tip.revise(|old| *old = value),
                "card" => state.card.revise(|old| *old = value),
                "disabled" => state.disabled.revise(|old| *old = value),
                "reject" => state.reject.revise(|old| *old = value),
                _ => return Err(js("unknown hint state")),
            };
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_hints() {
        let old = HINTS.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
