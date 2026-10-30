//! Bounded Rust route state. Browser history and transport live in the host.
use crate::theme_studio::{EditorState, model::ThemeRecord};
use glory::prelude::*;
use glory::routing::{Location, RouteDef, RouteKey, RouteViews, RouterHandle, Routes, SlashPolicy};
use sapid::{ColorScheme, Theme};
pub const DOCUMENT: RouteKey<glory::routing::Params> = RouteKey::new("showcase.document");
pub const SCHEMA: &str = "sapid-showcase-routing-v1";
pub const SSR_PAGES: &[(&str, &str)] = &[
    ("", "home"),
    ("P0-01", "button"),
    ("P0-02", "textfield"),
    ("P2-08", "actions"),
    ("P3-10", "pills"),
    ("P4-08", "multi-select"),
    ("P4-09", "tags"),
    ("P9-01", "dates"),
    ("P13-01", "chat"),
];
pub fn routes() -> Routes {
    // The public application is query-routed, including its static SSR aliases.
    Routes::builder()
        .slash_policy(SlashPolicy::Ignore)
        .root(RouteDef::new(DOCUMENT, "*document"))
        .finish()
        .expect("showcase routes")
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    pub widget: String,
    pub palette: String,
    pub scheme: String,
    pub return_to: String,
    pub edit: Option<String>,
    pub fragment: String,
}
impl Route {
    pub fn from_location(location: &Location) -> Self {
        let mut route = Self::new(String::new(), &Theme::blue_light());
        let file = location.path().rsplit('/').next().unwrap_or_default();
        for &(widget, name) in SSR_PAGES {
            for palette in ["blue", "green"] {
                for scheme in ["light", "dark"] {
                    if file == format!("ssr-{name}-{palette}-{scheme}.html") {
                        route.widget = widget.into();
                        route.palette = palette.into();
                        route.scheme = scheme.into();
                    }
                }
            }
        }
        let query = location.query();
        if let Some(widget) = query.values("widget").next() {
            route.widget = widget.into();
        }
        if let Some(palette) = query.values("theme").next() {
            route.palette = palette.into();
        }
        if let Some(scheme) = query.values("scheme").next() {
            route.scheme = if scheme == "dark" { "dark" } else { "light" }.into();
        }
        route.return_to = query
            .values("return_to")
            .next()
            .filter(|id| crate::showcase_catalog::find(id).is_some())
            .unwrap_or("P0-01")
            .into();
        route.edit = query.values("edit").next().map(str::to_owned);
        route.fragment = location.fragment().unwrap_or_default().into();
        route
    }
    pub fn new(widget: String, theme: &Theme) -> Self {
        Self {
            widget,
            palette: theme.name.clone(),
            scheme: theme.scheme.name().into(),
            return_to: "P0-01".into(),
            edit: None,
            fragment: String::new(),
        }
    }
    pub fn same_view(&self, other: &Self) -> bool {
        self.widget == other.widget
            && self.palette == other.palette
            && self.scheme == other.scheme
            && self.return_to == other.return_to
            && self.edit == other.edit
    }
    pub fn mode(&self) -> ColorScheme {
        if self.scheme == "dark" {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        }
    }
    pub fn url(&self) -> String {
        self.location().to_string().trim_start_matches('/').into()
    }
    pub fn location(&self) -> Location {
        self.canonical_location(&Location::parse("/showcase.html").expect("showcase path"))
    }
    pub fn canonical_location(&self, requested: &Location) -> Location {
        let mut url = Location::parse(requested.path())
            .expect("validated showcase path")
            .with_query("widget", &self.widget)
            .with_query("theme", &self.palette)
            .with_query("scheme", self.mode().name());
        if self.return_to != "P0-01" {
            url = url.with_query("return_to", &self.return_to);
        }
        if let Some(edit) = &self.edit {
            url = url.with_query("edit", edit);
        }
        for (name, value) in &requested.query().0 {
            if !matches!(
                name.as_str(),
                "widget" | "theme" | "scheme" | "return_to" | "edit"
            ) {
                url = url.with_query(name, value);
            }
        }
        if !self.fragment.is_empty() || requested.fragment().is_some() {
            url = url.with_fragment(&self.fragment);
        }
        url
    }
}
#[derive(Clone)]
pub struct Page {
    pub route: Route,
    pub theme: Theme,
    pub initial: ThemeRecord,
    pub notice: String,
    pub revision: u64,
}
impl Page {
    pub fn builtin(theme: Theme, widget: String) -> Self {
        Self {
            route: Route::new(widget, &theme),
            initial: ThemeRecord::from_definition(if theme.name == "green" {
                &sapid::GREEN
            } else {
                &sapid::BLUE
            }),
            theme,
            notice: String::new(),
            revision: 0,
        }
    }
}
#[derive(Clone, Copy)]
pub struct State {
    pub page: Cage<Page>,
    pub catalog: Cage<Vec<ThemeRecord>>,
    pub notice: Cage<String>,
    pub editor: EditorState,
}
impl State {
    pub fn new(scope: &Scope, page: Page, catalog: Vec<ThemeRecord>, hydrate: bool) -> Self {
        let editor = EditorState::new(scope, page.initial.clone(), page.theme.scheme);
        Self {
            notice: scope.cage(if hydrate {
                String::new()
            } else {
                page.notice.clone()
            }),
            page: scope.cage(page),
            catalog: scope.cage(catalog),
            editor,
        }
    }
}
pub fn app(state: State) -> impl Widget {
    widget(move |scope| {
        let revisions = scope.bond(move || vec![state.page.with(|p| p.revision)]);
        scope.effect(move || {
            let page = state.page.get();
            if let Err(error) = crate::showcase_host::present(&page, state.page) {
                state.notice.revise(|v| *v = error);
            }
        });
        each(revisions)
            .key(|v| *v)
            .children(move |__each_row| {
                let page = state.page.get();
                if page.route.widget == "theme-studio" {
                    state.editor.prepare(
                        page.initial.clone(),
                        page.theme.scheme,
                        page.route.edit.as_deref(),
                    );
                }
                crate::showcase::site_with_state(
                    page.theme,
                    page.route.widget,
                    state.catalog,
                    page.route.return_to,
                    state.notice,
                    state.editor,
                )
            })
            .show_in(scope)
    })
}

pub fn routed_app(state: State, definitions: Routes, router: RouterHandle) -> impl Widget {
    RouteViews::new(definitions)
        .route(DOCUMENT, move |_| app(state))
        .expect("showcase view")
        .view(router)
        .expect("shared showcase declarations")
}
