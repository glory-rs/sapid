use glory::prelude::*;
use glory::render::EventKind;
use sapid::{Theme, breadcrumb_item, breadcrumbs, nav_link, pagination};

pub struct State {
    pub page: Cage<u64>,
    pub total: Cage<Option<u64>>,
    pub accept: Cage<bool>,
    pub enabled: Cage<bool>,
    pub open: Cage<bool>,
    pub current: Cage<String>,
    pub requests: Cage<usize>,
    pub ordered: Cage<bool>,
    pub created: Cage<usize>,
    pub retired: Cage<usize>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let page = scope.cage(5);
    let total = scope.cage(Some(1_000_000_000_000));
    let accept = scope.cage(true);
    let enabled = scope.cage(true);
    let open = scope.cage(false);
    let current = scope.cage("article".to_owned());
    let requests = scope.cage(0usize);
    let ordered = scope.cage(true);
    let stage = scope.cage(false);
    let created = scope.cage(0usize);
    let retired = scope.cage(0usize);
    let status = scope.bond(move || {
        format!(
            "page={}; requests={}; ordered={}; created={}; retired={}",
            page.get(),
            requests.get(),
            ordered.get(),
            created.get(),
            retired.get()
        )
    });
    let widget = col()
        .class("navigation-demo")
        .style(
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(18.0),
        )
        .fill((
            sapid::title(1)
                .theme(theme.clone())
                .content("Navigate with context"),
            breadcrumbs("web-trail")
                .theme(theme.clone())
                .max_items(4)
                .current(current)
                .items([
                    breadcrumb_item("home", "Home").href("#home"),
                    breadcrumb_item("library", "Library").href("#library"),
                    breadcrumb_item("guides", "Guides").href("#guides"),
                    breadcrumb_item("rust", "Rust").href("#rust"),
                    breadcrumb_item("ui", "User interfaces").href("#ui"),
                    breadcrumb_item("article", "Current article"),
                ]),
            pagination("web-pages")
                .class("main-pages")
                .theme(theme.clone())
                .page(page)
                .total(total)
                .enabled(enabled)
                .on_page_change(move |proposal| {
                    requests.revise(|value| *value += 1);
                    stage.revise(|value| *value = true);
                    if accept.get_untracked() {
                        page.revise(|value| *value = proposal)
                    }
                })
                .on(EventKind::Activate, move |_| {
                    ordered.revise(|value| *value &= stage.get_untracked())
                }),
            pagination("web-unknown")
                .class("unknown-pages")
                .theme(theme.clone())
                .name("Unknown total pages")
                .default_page(3)
                .has_next(true),
            nav_link("web-docs")
                .class("docs-nav")
                .theme(theme.clone())
                .label("Documentation")
                .href("/documentation")
                .current(true)
                .highlighted(true)
                .expanded(open)
                .on_expanded_change(move |value| {
                    if accept.get_untracked() {
                        open.revise(|old| *old = value)
                    }
                })
                .on_navigate(move |_| requests.revise(|value| *value += 1))
                .children(move || {
                    glory::ui::widget(move |scope| {
                        created.revise(|value| *value += 1);
                        scope.on_cleanup(move || {
                            if retired.is_alive() {
                                retired.revise(|value| *value += 1)
                            }
                        })?;
                        glory::ui::input()
                            .class("nav-child-input")
                            .name("Nested note")
                            .value("Retired with its owner")
                            .show_in(scope)
                    })
                }),
            sapid::text()
                .theme(theme)
                .class("navigation-status")
                .content(status),
        ));
    (
        widget,
        State {
            page,
            total,
            accept,
            enabled,
            open,
            current,
            requests,
            ordered,
            created,
            retired,
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
    fn js(value: impl std::fmt::Display) -> JsValue {
        value.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_navigation(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing navigation root"))?;
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
        let old = NAVIGATION.with(|slot| slot.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn navigation_state(accept: bool, enabled: bool) -> Result {
        NAVIGATION.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted navigation"))?;
            app.app().pump().map_err(js)?;
            state.accept.revise(|value| *value = accept);
            state.enabled.revise(|value| *value = enabled);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn navigation_page(page: f64, total: f64) -> Result {
        if !page.is_finite()
            || page < 0.0
            || page.fract() != 0.0
            || !total.is_finite()
            || total < 0.0
            || total.fract() != 0.0
        {
            return Err(js("integer pages required"));
        }
        NAVIGATION.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted navigation"))?;
            app.app().pump().map_err(js)?;
            state.page.revise(|value| *value = page as u64);
            state.total.revise(|value| {
                *value = if total == 0.0 {
                    None
                } else {
                    Some(total as u64)
                }
            });
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn navigation_open(value: bool) -> Result {
        NAVIGATION.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted navigation"))?;
            app.app().pump().map_err(js)?;
            state.open.revise(|old| *old = value);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn navigation_current(value: String) -> Result {
        NAVIGATION.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted navigation"))?;
            app.app().pump().map_err(js)?;
            state.current.revise(|old| *old = value);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_navigation() {
        let old = NAVIGATION.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
