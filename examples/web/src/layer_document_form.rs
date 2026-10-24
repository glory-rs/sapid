use glory::prelude::*;
use glory::render::EventKind;
use sapid::{Theme, dialog, popover};

pub struct State {
    pub open: Cage<bool>,
    pub reject: Cage<bool>,
}
pub fn form(scope: &Scope, key: &'static str, theme: Theme) -> (impl Widget + use<>, State) {
    let open = scope.cage(false);
    let reject = scope.cage(false);
    let clicks = scope.cage(0);
    let widget = view()
        .class(format!("document-layer-app document-app-{key}"))
        .style(
            Style::new()
                .width(Length::Px(320.))
                .padding(Edges::all(12.)),
        )
        .fill((
            button()
                .label(format!("Outside {key}"))
                .on(EventKind::Activate, move |_| clicks.revise(|n| *n += 1)),
            input()
                .name(format!("Sibling {key}"))
                .value("sibling draft"),
            dialog(format!("document-{key}"))
                .theme(theme.clone())
                .label(format!("Open {key}"))
                .title(format!("Modal {key}"))
                .close_label(format!("Close {key}"))
                .open(open)
                .on_open_change(move |value| {
                    if !reject.get() {
                        open.revise(|old| *old = value)
                    }
                })
                .style(Style {
                    width: Some(Length::Px(360.)),
                    max_height: Some(Limit::Px(280.)),
                    ..Default::default()
                })
                .content(move || {
                    view().fill((
                        input().name(format!("First {key}")).value("first draft"),
                        input().name(format!("Second {key}")).value("second draft"),
                        popover(format!("document-nested-{key}"))
                            .theme(theme.clone())
                            .label(format!("Nested {key}"))
                            .content(move || input().name(format!("Nested editor {key}"))),
                        view()
                            .class("document-modal-tall")
                            .style(Style {
                                height: Some(Length::Px(900.)),
                                shrink: Some(0.),
                                ..Default::default()
                            })
                            .fill(text().content("Scrollable modal content")),
                        button().label(format!("Last {key}")),
                    ))
                }),
            text()
                .class(format!("document-clicks-{key}"))
                .content(clicks.map(|n| format!("clicks:{n}"))),
        ));
    (widget, State { open, reject })
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static APPS:RefCell<[Option<(WebApp,State)>;2]>=const{RefCell::new([None,None])};}
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    fn index(key: &str) -> std::result::Result<usize, JsValue> {
        match key {
            "a" => Ok(0),
            "b" => Ok(1),
            _ => Err(js("invalid layer App")),
        }
    }
    #[wasm_bindgen]
    pub fn mount_layer_document(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> std::result::Result<(), JsValue> {
        let old = APPS.with(|slot| slot.replace([None, None]));
        drop(old);
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| js("missing document"))?;
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
        for (i, key) in ["a", "b"].into_iter().enumerate() {
            let root = document
                .get_element_by_id(&format!("app-{key}"))
                .ok_or_else(|| js("missing layer root"))?;
            let mut state = None;
            let theme = theme.clone();
            let build = |scope: &Scope| {
                let (widget, value) = form(scope, key, theme);
                state = Some(value);
                widget
            };
            let app = if hydrate {
                glory::web::hydrate_with(root, build).map_err(js)?
            } else {
                glory::web::launch_with(root, build).map_err(js)?
            };
            APPS.with(|slot| slot.borrow_mut()[i] = Some((app, state.unwrap())));
        }
        Ok(())
    }
    #[wasm_bindgen]
    pub fn layer_document_state(
        key: String,
        kind: String,
        value: bool,
    ) -> std::result::Result<(), JsValue> {
        let i = index(&key)?;
        APPS.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot[i].as_ref().ok_or_else(|| js("layer App retired"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "open" => state.open.revise(|old| *old = value),
                "reject" => state.reject.revise(|old| *old = value),
                _ => return Err(js("invalid layer state")),
            }
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_layer_document_app(key: String) -> std::result::Result<(), JsValue> {
        let i = index(&key)?;
        let old = APPS.with(|slot| slot.borrow_mut()[i].take());
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn layer_document_budget_probe() -> std::result::Result<bool, JsValue> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| js("missing document"))?;
        let body = document.body().ok_or_else(|| js("missing body"))?;
        let mut apps = Vec::new();
        let mut roots = Vec::new();
        let mut rejected = false;
        for _ in 0..65 {
            let root = document.create_element("div")?;
            body.append_child(&root)?;
            let before = root.outer_html();
            match glory::web::launch(root.clone(), view()) {
                Ok(app) => apps.push(app),
                Err(_) => {
                    rejected = root.outer_html() == before;
                    roots.push(root);
                    break;
                }
            }
            roots.push(root);
        }
        drop(apps);
        for root in roots {
            root.remove()
        }
        Ok(rejected)
    }
}
