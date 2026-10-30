use super::*;
use crate::showcase_navigation::{self, Route, State};
use crate::theme_studio::{
    host,
    model::{ThemeRecord, ThemeStore},
};
use glory::routing::{Bootstrap, Location, RouterHandle};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::prelude::*;

struct Running {
    app: glory::web::routing::RoutingApp,
    state: State,
    seen: Rc<Cell<u64>>,
    palette_scroll: Cell<Option<i32>>,
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
        .unwrap_or_else(|| "Browser operation failed".into())
}
pub(super) fn report(message: String) {
    let state = APP.with(|slot| slot.borrow().as_ref().map(|r| r.state));
    if let Some(state) = state {
        state.notice.revise(|v| *v = message);
    } else {
        web_sys::console::error_1(&message.into());
    }
}
fn router_error(error: glory::routing::Error) {
    report(error.to_string());
}
fn location(url: &web_sys::Url) -> Result<Location, String> {
    Location::parse(&format!("{}{}{}", url.pathname(), url.search(), url.hash()))
        .map_err(|error| error.to_string())
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
            notice =
                "The saved theme to edit is unavailable. Your current workspace was kept.".into();
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
        },
        store,
    ))
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
pub(super) fn palette_navigate(href: &str) -> Result<(), String> {
    let scroll = window()?
        .document()
        .and_then(|document| document.query_selector(".site-palettes").ok().flatten())
        .map(|palettes| palettes.scroll_left());
    let revision = APP.with(|slot| {
        slot.borrow()
            .as_ref()
            .map(|running| running.state.page.with(|page| page.revision))
    });
    navigate(href)?;
    APP.with(|slot| {
        if let Some(running) = slot.borrow().as_ref() {
            let changed = Some(running.state.page.with(|page| page.revision)) != revision;
            running
                .palette_scroll
                .set(if changed { scroll } else { None });
        }
    });
    Ok(())
}
pub(super) fn navigate(href: &str) -> Result<(), String> {
    let window = window()?;
    let url = web_sys::Url::new_with_base(href, &window.location().href().map_err(error)?)
        .map_err(error)?;
    let local_document = url.pathname().ends_with("/showcase.html")
        || (href.starts_with('#')
            && url.pathname() == window.location().pathname().map_err(error)?);
    if url.origin() != window.location().origin().map_err(error)? || !local_document {
        return Err("Only showcase routes use local navigation".into());
    }
    let (state, router, seen) = APP
        .with(|slot| {
            slot.borrow()
                .as_ref()
                .map(|r| (r.state, r.app.router(), r.seen.clone()))
        })
        .ok_or("Showcase is not mounted")?;
    let requested = location(&url)?;
    let (page, store) = resolve(Route::from_location(&requested))?;
    let target = if href.starts_with('#') {
        requested
    } else {
        page.route.canonical_location(&requested)
    };
    router.push(target).map_err(|error| error.to_string())?;
    seen.set(
        router
            .current()
            .map_err(|error| error.to_string())?
            .revision,
    );
    let previous = state.page.get();
    if page.route == previous.route
        && page.theme == previous.theme
        && page.initial == previous.initial
        && page.notice == previous.notice
        && state.catalog.with(|list| list == &store.themes)
    {
        return Ok(());
    }
    // Component selection is a query change, so its top-of-page policy belongs
    // to this application. BrowserHistory already captured the outgoing entry.
    if page.route.widget != previous.route.widget && page.route.fragment.is_empty() {
        window.scroll_to_with_x_and_y(0., 0.);
    }
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
    let palette_scroll = APP.with(|slot| {
        slot.borrow()
            .as_ref()
            .and_then(|running| running.palette_scroll.take())
    });
    // This horizontal palette strip is application state; window/fragment
    // restoration is owned entirely by Glory's browser routing driver.
    if let Some(x) = palette_scroll {
        let revision = page.revision;
        let callback_window = window.clone();
        let callback = Closure::once_into_js(move || {
            if source.try_get().is_ok_and(|p| p.revision == revision)
                && let Some(palettes) = callback_window
                    .document()
                    .and_then(|d| d.query_selector(".site-palettes").ok().flatten())
            {
                palettes.set_scroll_left(x);
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
    let route = Route::from_location(&location(&url).map_err(js)?);
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
    drop(APP.with(|slot| slot.borrow_mut().take()));
    let definitions = showcase_navigation::routes();
    let seed = if hydrate {
        let text = document
            .get_element_by_id("sapid-route-seed")
            .ok_or_else(|| js("Missing showcase route seed"))?
            .text_content()
            .unwrap_or_default();
        Some(Bootstrap::from_payload(&text).map_err(js)?)
    } else {
        None
    };
    let initial = if let Some(seed) = &seed {
        seed.validate(&definitions, showcase_navigation::SCHEMA)
            .map_err(js)?
    } else {
        location(&web_sys::Url::new(&window.location().href()?)?).map_err(js)?
    };
    let mut route = Route::from_location(&initial);
    route.widget = selection;
    route.palette = hue;
    route.scheme = scheme;
    let (page, store) = resolve(route).map_err(js)?;
    host::install_catalog(&store).map_err(js)?;
    host::install_page(&page.theme).map_err(js)?;
    let captured = Rc::new(RefCell::new(None));
    let capture = captured.clone();
    let seen = Rc::new(Cell::new(0));
    let observed_revision = seen.clone();
    let views = definitions.clone();
    let catalog = store.themes.clone();
    let build = move |scope: &glory::Scope, router: RouterHandle| {
        let state = State::new(
            scope,
            page,
            if hydrate { Vec::new() } else { catalog },
            hydrate,
        );
        *capture.borrow_mut() = Some(state);
        let observed = router.clone();
        scope.effect(move || {
            if let Ok(snapshot) = observed.snapshot()
                && observed_revision.replace(snapshot.revision) != snapshot.revision
            {
                match resolve(Route::from_location(snapshot.location())) {
                    Ok((page, store)) => {
                        if let Err(error) = publish(state, page, store) {
                            report(error);
                        }
                    }
                    Err(error) => report(error),
                }
            }
        });
        showcase_navigation::routed_app(state, views, router)
    };
    let app = if let Some(seed) = &seed {
        glory::web::routing::hydrate_with(
            root,
            definitions,
            seed,
            showcase_navigation::SCHEMA,
            build,
            router_error,
        )
    } else {
        glory::web::routing::launch_with(root, definitions, build, router_error)
    }
    .map_err(js)?;
    let state = captured
        .borrow_mut()
        .take()
        .expect("mounted showcase state");
    if hydrate {
        state.catalog.revise(|v| *v = store.themes);
        state
            .notice
            .revise(|v| *v = state.page.with(|p| p.notice.clone()));
        app.app().app().flush().map_err(js)?;
    }
    APP.with(|slot| {
        *slot.borrow_mut() = Some(Running {
            app,
            state,
            seen,
            palette_scroll: Cell::new(None),
        })
    });
    Ok(())
}
