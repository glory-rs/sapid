use super::*;
use crate::showcase_navigation::State;
use crate::{
    showcase_navigation::Route,
    theme_studio::{
        host,
        model::{ThemeRecord, ThemeStore},
    },
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
struct Running {
    _app: glory::web::WebApp,
    state: State,
    window: web_sys::Window,
    pop: Closure<dyn FnMut(web_sys::PopStateEvent)>,
}
impl Drop for Running {
    fn drop(&mut self) {
        let _ = self
            .window
            .remove_event_listener_with_callback("popstate", self.pop.as_ref().unchecked_ref());
    }
}
thread_local! {static APP:RefCell<Option<Running>>=const{RefCell::new(None)};}
fn js(error: impl std::fmt::Display) -> JsValue {
    error.to_string().into()
}
fn window() -> Result<web_sys::Window, String> {
    web_sys::window().ok_or_else(|| "Missing window".into())
}
fn error(value: JsValue) -> String {
    value
        .as_string()
        .unwrap_or_else(|| "Browser history operation failed".into())
}
pub(super) fn report(message: String) {
    let state = APP.with(|slot| slot.borrow().as_ref().map(|r| r.state));
    if let Some(state) = state {
        state.notice.revise(|v| *v = message);
    }
}
fn parse(url: &web_sys::Url) -> Route {
    let query = url.search_params();
    Route {
        widget: query.get("widget").unwrap_or_default(),
        palette: query.get("theme").unwrap_or_else(|| "blue".into()),
        scheme: if query.get("scheme").as_deref() == Some("dark") {
            "dark"
        } else {
            "light"
        }
        .into(),
        return_to: query
            .get("return_to")
            .filter(|id| crate::showcase_catalog::find(id).is_some())
            .unwrap_or_else(|| "P0-01".into()),
        edit: query.get("edit"),
        fragment: url.hash().trim_start_matches('#').to_owned(),
    }
}
fn resolve(mut route: Route) -> Result<(Page, ThemeStore), String> {
    let (store, mut notice) = match host::load() {
        Ok(store) => (store, String::new()),
        Err(e) => (
            ThemeStore::default(),
            format!("Local themes could not be loaded: {e}. Stored data was kept."),
        ),
    };
    let selected = store.themes.iter().find(|r| r.id == route.palette);
    let theme = if let Some(record) = selected {
        let mode = if record.supports(route.mode()) {
            route.mode()
        } else {
            notice = format!(
                "{} does not support {}. Showing its default {} mode.",
                record.title,
                route.scheme,
                record.default_scheme.name()
            );
            record.default_scheme
        };
        record.theme(mode)?
    } else {
        if !matches!(route.palette.as_str(), "blue" | "green") {
            notice="That local preview is unavailable in this browser. Choose a saved palette or create one in Theme studio.".into();
        }
        crate::themes::select(&route.palette, route.mode())
    };
    let mut initial = selected.cloned().unwrap_or_else(|| {
        ThemeRecord::from_definition(if theme.name == "green" {
            &sapid::GREEN
        } else {
            &sapid::BLUE
        })
    });
    if let Some(edit) = &route.edit {
        if let Some(record) = store.themes.iter().find(|r| &r.id == edit) {
            initial = record.clone()
        } else {
            notice = "The saved theme to edit is unavailable. Your current workspace was kept."
                .into();
            route.edit = None;
        }
    }
    route.scheme = theme.scheme.name().into();
    Ok((
        Page {
            route,
            theme,
            initial,
            notice,
            revision: 0,
            scroll: None,
        },
        store,
    ))
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    version: u8,
    route: Route,
    x: f64,
    y: f64,
}
fn entry(route: Route, window: &web_sys::Window) -> Entry {
    Entry {
        version: 1,
        route,
        x: window.scroll_x().unwrap_or(0.),
        y: window.scroll_y().unwrap_or(0.),
    }
}
fn value(entry: &Entry) -> Result<JsValue, String> {
    serde_json::to_string(entry)
        .map(|s| JsValue::from_str(&s))
        .map_err(|e| e.to_string())
}
fn read_entry(value: JsValue) -> Option<Entry> {
    let text = value.as_string()?;
    if text.len() > 8192 {
        return None;
    }
    let entry: Entry = serde_json::from_str(&text).ok()?;
    (entry.version == 1 && entry.x.is_finite() && entry.y.is_finite()).then_some(entry)
}
fn publish(state: State, mut page: Page, store: ThemeStore) -> Result<(), String> {
    page.revision = state
        .page
        .with(|old| {
            if old.route.same_view(&page.route)
                && old.theme == page.theme
                && old.initial == page.initial
            {
                Some(old.revision)
            } else {
                old.revision.checked_add(1)
            }
        })
        .ok_or("Navigation revision exhausted")?;
    host::install_catalog(&store)?;
    state.catalog.revise(|v| *v = store.themes);
    state.notice.revise(|v| *v = page.notice.clone());
    state.page.revise(|v| *v = page);
    Ok(())
}
pub(super) fn navigate(href: &str) -> Result<(), String> {
    let window = window()?;
    let url = web_sys::Url::new_with_base(href, &window.location().href().map_err(error)?)
        .map_err(error)?;
    if url.origin() != window.location().origin().map_err(error)?
        || !url.pathname().ends_with("/showcase.html")
    {
        return Err("Only showcase routes use local navigation".into());
    }
    let state = APP
        .with(|slot| slot.borrow().as_ref().map(|r| r.state))
        .ok_or("Showcase is not mounted")?;
    let (mut page, store) = resolve(parse(&url))?;
    let previous = state.page.get();
    if page.route == previous.route
        && page.theme == previous.theme
        && page.initial == previous.initial
    {
        return Ok(());
    }
    let history = window.history().map_err(error)?;
    history
        .replace_state_with_url(&value(&entry(previous.route.clone(), &window))?, "", None)
        .map_err(error)?;
    let keep = page.route.widget == previous.route.widget;
    page.scroll = Some(if keep {
        (
            window.scroll_x().unwrap_or(0.),
            window.scroll_y().unwrap_or(0.),
        )
    } else {
        (0., 0.)
    });
    let mut next = entry(page.route.clone(), &window);
    let (x, y) = page.scroll.unwrap();
    next.x = x;
    next.y = y;
    history
        .push_state_with_url(&value(&next)?, "", Some(&page.route.url()))
        .map_err(error)?;
    publish(state, page, store)
}
fn pop(state: State, event: web_sys::PopStateEvent) -> Result<(), String> {
    let window = window()?;
    let current =
        web_sys::Url::new(&window.location().href().map_err(error)?).map_err(error)?;
    let history = read_entry(event.state());
    let mut route = history
        .as_ref()
        .map(|e| e.route.clone())
        .unwrap_or_else(|| parse(&current));
    route.fragment = current.hash().trim_start_matches('#').to_owned();
    let (mut page, store) = resolve(route)?;
    page.scroll = history.map(|e| (e.x, e.y));
    publish(state, page, store)
}
pub(super) fn present(page: &Page, source: Cage<Page>) -> Result<(), String> {
    host::install_page(&page.theme)?;
    let window = window()?;
    let doc = window.document().ok_or("Missing document")?;
    let html = doc.document_element().ok_or("Missing document root")?;
    html.set_attribute("data-sapid-widget", &page.route.widget)
        .map_err(error)?;
    html.set_attribute("data-sapid-palette", &page.theme.name)
        .map_err(error)?;
    html.set_attribute("data-sapid-scheme", page.theme.scheme.name())
        .map_err(error)?;
    if page.scroll.is_some() || !page.route.fragment.is_empty() {
        let revision = page.revision;
        let fragment = page.route.fragment.clone();
        let position = page.scroll;
        let callback_window = window.clone();
        let callback = Closure::once_into_js(move || {
            if source.try_get().is_ok_and(|p| p.revision == revision) {
                if !fragment.is_empty()
                    && let Some(element) = callback_window
                        .document()
                        .and_then(|d| d.get_element_by_id(&fragment))
                {
                    element.scroll_into_view();
                } else if let Some((x, y)) = position {
                    callback_window.scroll_to_with_x_and_y(x, y);
                }
            }
        });
        window
            .request_animation_frame(callback.unchecked_ref())
            .map_err(error)?;
    }
    Ok(())
}
#[wasm_bindgen]
pub fn start_showcase() -> Result<bool, JsValue> {
    let window = window().map_err(js)?;
    let document = window.document().ok_or_else(|| js("Missing document"))?;
    let url = web_sys::Url::new(&window.location().href()?)?;
    if url.search_params().has("manual") {
        return Ok(false);
    }
    let meta = |key: &str| -> Result<String, JsValue> {
        document
            .query_selector(&format!("meta[name=\"sapid-showcase-{key}\"]"))?
            .and_then(|e| e.get_attribute("content"))
            .ok_or_else(|| js("Missing showcase metadata"))
    };
    let hydrate = meta("render")? == "ssr";
    let route = parse(&url);
    mount_showcase(
        hydrate,
        if hydrate {
            meta("selection")?
        } else {
            route.widget
        },
        if hydrate { meta("hue")? } else { route.palette },
        if hydrate {
            meta("scheme")?
        } else {
            route.scheme
        },
    )?;
    Ok(true)
}
#[wasm_bindgen]
pub fn mount_showcase(
    hydrate: bool,
    selection: String,
    hue: String,
    scheme: String,
) -> Result<(), JsValue> {
    let window = window().map_err(js)?;
    let document = window.document().ok_or_else(|| js("Missing document"))?;
    let root = document
        .get_element_by_id("app")
        .ok_or_else(|| js("Missing showcase root"))?;
    let url = web_sys::Url::new(&window.location().href()?)?;
    let mut route = parse(&url);
    route.widget = selection;
    route.palette = hue;
    route.scheme = scheme;
    let (page, store) = resolve(route).map_err(js)?;
    host::install_catalog(&store).map_err(js)?;
    host::install_page(&page.theme).map_err(js)?;
    let history = window.history()?;
    history.replace_state_with_url(
        &value(&entry(page.route.clone(), &window)).map_err(js)?,
        "",
        None,
    )?;
    let old = APP.with(|slot| slot.borrow_mut().take());
    drop(old);
    let mut handles = None;
    let build = |scope: &glory::Scope| {
        let state = State::new(
            scope,
            page,
            if hydrate {
                Vec::new()
            } else {
                store.themes.clone()
            },
            hydrate,
        );
        handles = Some(state);
        crate::showcase_navigation::app(state)
    };
    let app = if hydrate {
        glory::web::hydrate_with(root, build).map_err(js)?
    } else {
        glory::web::launch_with(root, build).map_err(js)?
    };
    let state = handles.expect("mounted route state");
    if hydrate {
        state.catalog.revise(|v| *v = store.themes);
        state
            .notice
            .revise(|v| *v = state.page.with(|p| p.notice.clone()));
        app.app().flush().map_err(js)?;
    }
    let pop = Closure::wrap(Box::new(move |event| {
        if let Err(e) = pop(state, event) {
            report(e);
        }
    }) as Box<dyn FnMut(web_sys::PopStateEvent)>);
    window.add_event_listener_with_callback("popstate", pop.as_ref().unchecked_ref())?;
    APP.with(|slot| {
        *slot.borrow_mut() = Some(Running {
            _app: app,
            state,
            window,
            pop,
        })
    });
    Ok(())
}
