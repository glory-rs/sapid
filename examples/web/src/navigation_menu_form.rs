use glory::prelude::*;
use glory::render::EventKind;
use sapid::{Theme, navigation_menu, navigation_menu_link, navigation_menu_panel};
pub struct State {
    pub path: Cage<Vec<String>>,
    pub accept: Cage<bool>,
    pub enabled: Cage<bool>,
    pub editor: Cage<bool>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let path = scope.cage(Vec::<String>::new());
    let accept = scope.cage(true);
    let enabled = scope.cage(true);
    let status = scope.cage("none".to_owned());
    let draft = scope.cage("Kept navigation draft".to_owned());
    let editor = scope.cage(true);
    let draft_ref = scope.cage(None);
    let menu = navigation_menu("fixture-nav")
        .theme(theme.clone())
        .name("Primary navigation")
        .hover(true)
        .class("navigation-menu-demo")
        .open_path(path)
        .on_open_path_change(move |next| {
            if accept.get() {
                path.revise(|path| *path = next)
            }
        })
        .content(move |ctx| {
            (
                navigation_menu_link("home")
                    .context(ctx.clone())
                    .label("Home")
                    .href("/home")
                    .on_navigate(move |request| status.revise(|value| *value = request.href)),
                navigation_menu_panel("products")
                    .context(ctx.clone())
                    .label("Products")
                    .enabled(enabled)
                    .class("products-trigger")
                    .content(move |ctx| {
                        (
                            navigation_menu_link("editor")
                                .context(ctx.clone())
                                .label("Editor")
                                .href("/editor")
                                .class("editor-link")
                                .on_navigate(move |request| {
                                    status.revise(|value| *value = request.href)
                                }),
                            navigation_menu_panel("advanced")
                                .context(ctx)
                                .label("Advanced")
                                .class("advanced-trigger")
                                .content(move |ctx| {
                                    (
                                        navigation_menu_link("api")
                                            .context(ctx.clone())
                                            .label("API reference")
                                            .href("/api")
                                            .class("api-link")
                                            .on_navigate(move |request| {
                                                status.revise(|value| *value = request.href)
                                            }),
                                        glory::ui::when(editor, move || {
                                            ctx.track_focusable(
                                                draft_ref,
                                                glory::ui::view().role(Role::ListItem).fill(
                                                    glory::ui::input()
                                                        .node_ref(draft_ref)
                                                        .name("Navigation draft")
                                                        .class("navigation-draft")
                                                        .value(draft)
                                                        .on(
                                                            EventKind::ValueChanged,
                                                            move |event| {
                                                                if let Event::ValueChanged {
                                                                    value,
                                                                    ..
                                                                } = event
                                                                {
                                                                    draft.revise(|draft| {
                                                                        *draft = value
                                                                    })
                                                                }
                                                            },
                                                        ),
                                                ),
                                            )
                                        }),
                                    )
                                }),
                        )
                    }),
                navigation_menu_panel("support")
                    .context(ctx)
                    .label("Support")
                    .class("support-trigger")
                    .content(move |ctx| {
                        navigation_menu_link("help")
                            .context(ctx)
                            .label("Help center")
                            .href("/help")
                            .class("help-link")
                            .on_navigate(move |request| {
                                status.revise(|value| *value = request.href)
                            })
                    }),
            )
        });
    (
        glory::ui::col()
            .class("navigation-menu-fixture")
            .style(glory::style::Style::new().gap(16.0))
            .fill((
                glory::ui::text()
                    .content("Navigation menu · actual links and separate disclosures"),
                menu,
                sapid::button()
                    .theme(theme.clone())
                    .label("Outside action")
                    .class("navigation-outside"),
                glory::ui::text()
                    .class("navigation-request")
                    .content(status),
                glory::ui::text()
                    .class("navigation-path")
                    .content(scope.bond(move || format!("{:?}", path.get()))),
            )),
        State {
            path,
            accept,
            enabled,
            editor,
        },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static NAVIGATION:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_navigation_menu(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("app"))
            .ok_or_else(|| js("missing navigation root"))?;
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
            let (widget, sources) = form(scope, theme);
            state = Some(sources);
            widget
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        let old = NAVIGATION.with(|slot| slot.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn navigation_menu_control(accept: bool, enabled: bool) -> Result {
        NAVIGATION.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            state.accept.revise(|value| *value = accept);
            state.enabled.revise(|value| *value = enabled);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn navigation_menu_path(path: String) -> Result {
        NAVIGATION.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            state.path.revise(|value| {
                *value = if path.is_empty() {
                    Vec::new()
                } else {
                    path.split('/').map(str::to_owned).take(9).collect()
                }
            });
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_navigation_menu() {
        let old = NAVIGATION.with(|slot| slot.borrow_mut().take());
        drop(old)
    }
    #[wasm_bindgen]
    pub fn navigation_menu_editor(shown: bool) -> Result {
        NAVIGATION.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            state.editor.revise(|value| *value = shown);
            app.app().flush().map_err(js)
        })
    }
}
