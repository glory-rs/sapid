//! Bounded Rust route state. Browser history and transport live in the host.
use crate::theme_studio::{EditorState, model::ThemeRecord};
use glory::prelude::*;
use sapid::{ColorScheme, Theme};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub widget: String,
    pub palette: String,
    pub scheme: String,
    pub return_to: String,
    pub edit: Option<String>,
    pub fragment: String,
}
impl Route {
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
        let mut url = format!(
            "showcase.html?widget={}&theme={}&scheme={}",
            encode(&self.widget),
            encode(&self.palette),
            self.mode().name()
        );
        if self.return_to != "P0-01" {
            url.push_str(&format!("&return_to={}", encode(&self.return_to)));
        }
        if let Some(edit) = &self.edit {
            url.push_str(&format!("&edit={}", encode(edit)));
        }
        if !self.fragment.is_empty() {
            url.push('#');
            url.push_str(&encode(&self.fragment));
        }
        url
    }
}
fn encode(value: &str) -> String {
    let mut out = String::new();
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(b as char)
        } else {
            out.push_str(&format!("%{b:02X}"))
        }
    }
    out
}
#[derive(Clone)]
pub struct Page {
    pub route: Route,
    pub theme: Theme,
    pub initial: ThemeRecord,
    pub notice: String,
    pub revision: u64,
    pub scroll: Option<(f64, f64)>,
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
            scroll: None,
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
        each(
            revisions,
            |v| *v,
            move |_, _| {
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
            },
        )
        .show_in(scope)
    })
}
