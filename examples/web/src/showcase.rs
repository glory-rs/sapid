//! The documentation website is a Glory application, including its navigation.
use glory::prelude::*;
use glory::render::{Current, Role, SemanticId, Semantics, TextKind};
use sapid::{ColorScheme, Hue, Theme};

use crate::showcase_catalog::{ENTRIES, Entry, find};

pub fn route(id: &str, theme: &Theme) -> String {
    format!(
        "showcase.html?widget={id}&theme={}&scheme={}",
        if theme.hue == Hue::Green {
            "green"
        } else {
            "blue"
        },
        if theme.scheme == ColorScheme::Dark {
            "dark"
        } else {
            "light"
        },
    )
}

fn label(entry: &Entry) -> &'static str {
    entry.name.split(" / ").next().unwrap_or(entry.name)
}

fn link(label: impl Into<String>, href: impl Into<String>, class: &str) -> sapid::Anchor {
    sapid::anchor()
        .unstyled()
        .label(label.into())
        .href(href.into())
        .class(class.to_owned())
}

fn section_title(id: &str, value: &str) -> impl Widget {
    text()
        .kind(TextKind::Heading(2))
        .class("site-section-title")
        .content(value.to_owned())
        .semantics(Some(Box::new(Semantics {
            id: Some(SemanticId::new(id).expect("static section identifier")),
            ..Default::default()
        })))
}

fn matching(entry: &Entry, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty()
        || format!("{} {} {}", entry.id, entry.name, entry.group)
            .to_lowercase()
            .contains(&query)
}

fn sidebar(query: Cage<String>, selected: &'static str, theme: Theme) -> impl Widget {
    widget(move |scope| {
        let groups = [
            "Layout",
            "Inputs",
            "Combobox",
            "Buttons",
            "Navigation",
            "Feedback",
            "Overlays",
            "Data display",
            "Typography",
            "Charts",
            "Dates",
            "Extensions",
            "Chat",
        ];
        let mut children: Vec<Box<dyn Widget>> = Vec::new();
        children.push(Box::new(
            text().class("site-sidebar-heading").content("Widgets"),
        ));
        for category in groups {
            let shown = scope.bond(move || {
                ENTRIES
                    .iter()
                    .any(|entry| entry.group == category && matching(entry, &query.get()))
            });
            let theme = theme.clone();
            children.push(Box::new(when(shown, move || {
                let theme = theme.clone();
                widget(move |scope| {
                    let items = scope.bond(move || {
                        ENTRIES
                            .iter()
                            .filter(|entry| {
                                entry.group == category && matching(entry, &query.get())
                            })
                            .map(|entry| entry.id)
                            .collect::<Vec<_>>()
                    });
                    let theme = theme.clone();
                    view()
                        .class("site-nav-group")
                        .fill((
                            text().class("site-nav-title").content(category),
                            each(
                                items,
                                |id| *id,
                                move |id, _| {
                                    let entry =
                                        find(id.get()).expect("catalogued navigation entry");
                                    link(
                                        label(entry),
                                        route(entry.id, &theme),
                                        if entry.id == selected {
                                            "site-nav-link is-active"
                                        } else {
                                            "site-nav-link"
                                        },
                                    )
                                    .current(
                                        if entry.id == selected {
                                            Some(Current::Page)
                                        } else {
                                            None
                                        },
                                    )
                                },
                            ),
                        ))
                        .show_in(scope)
                })
            })));
        }
        let none =
            scope.bond(move || !ENTRIES.iter().any(|entry| matching(entry, &query.get())));
        children.push(Box::new(when(none, || {
            text()
                .class("site-nav-empty")
                .content("No widgets found. Try another search.")
        })));
        view()
            .role(Role::Navigation)
            .name("Widget navigation")
            .class("site-sidebar")
            .fill(children)
            .show_in(scope)
    })
}

fn search_results(query: Cage<String>, theme: Theme) -> impl Widget {
    widget(move |scope| {
        let theme = theme.clone();
        let items = scope.bond(move || {
            ENTRIES
                .iter()
                .filter(|entry| matching(entry, &query.get()))
                .take(8)
                .map(|entry| entry.id)
                .collect::<Vec<_>>()
        });
        let none =
            scope.bond(move || !ENTRIES.iter().any(|entry| matching(entry, &query.get())));
        view()
            .class("site-search-results")
            .name("Search results")
            .fill((
                each(
                    items,
                    |id| *id,
                    move |id, _| {
                        let entry = find(id.get()).expect("catalogued search result");
                        link(
                            format!("{} · {}", label(entry), entry.group),
                            route(entry.id, &theme),
                            "site-search-result",
                        )
                    },
                ),
                when(none, || {
                    text()
                        .class("site-nav-empty")
                        .content("No matching widgets")
                }),
            ))
            .show_in(scope)
    })
}

fn header(
    query: Cage<String>,
    menu: Cage<bool>,
    selected: &'static str,
    theme: Theme,
) -> impl Widget {
    let mut opposite = theme.clone();
    opposite.scheme = if theme.scheme == ColorScheme::Dark {
        ColorScheme::Light
    } else {
        ColorScheme::Dark
    };
    let theme_label = if theme.scheme == ColorScheme::Dark {
        "Switch to light theme"
    } else {
        "Switch to dark theme"
    };
    widget(move |scope| {
        let has_query = scope.bond(move || !query.get().trim().is_empty());
        let is_empty = scope.bond(move || query.get().is_empty());
        let menu_name = scope.bond(move || {
            if menu.get() {
                "Close navigation".to_owned()
            } else {
                "Open navigation".to_owned()
            }
        });
        let search_theme = theme.clone();
        view()
            .role(Role::Banner)
            .class("site-header")
            .fill(
                view().class("site-header-inner").fill((
                    view().class("site-brand-wrap").fill((
                        text().class("site-logo").content("s"),
                        link("Sapid", route("", &theme), "site-brand"),
                    )),
                    view()
                        .role(Role::Navigation)
                        .name("Primary navigation")
                        .class("site-topnav")
                        .fill((
                            link("Widgets", route("P0-01", &theme), "site-topnav-link"),
                            link(
                                "Get started",
                                format!("{}#get-started", route("", &theme)),
                                "site-topnav-link",
                            ),
                            link(
                                "GitHub",
                                "https://github.com/glory-rs/sapid",
                                "site-topnav-link",
                            ),
                        )),
                    view().class("site-header-tools").fill((
                        text().class("site-version").content("v0.1.0"),
                        view().class("site-search").fill((
                            when(is_empty, || {
                                text()
                                    .class("site-search-placeholder")
                                    .content("Search widgets…")
                            }),
                            sapid::text_field()
                                .theme(theme.clone())
                                .name("Search widgets")
                                .value(query)
                                .on_value_change(move |value| {
                                    query.revise(|current| {
                                        *current = value.chars().take(96).collect()
                                    })
                                }),
                            when(has_query, move || {
                                search_results(query, search_theme.clone())
                            }),
                        )),
                        link(
                            if theme.scheme == ColorScheme::Dark {
                                "☀"
                            } else {
                                "☾"
                            },
                            route(selected, &opposite),
                            "site-theme-toggle",
                        )
                        .name(theme_label),
                        sapid::button()
                            .unstyled()
                            .class("site-menu-toggle")
                            .label("☰")
                            .name(menu_name)
                            .on_press(move || menu.revise(|open| *open = !*open)),
                    )),
                )),
            )
            .show_in(scope)
    })
}

fn subnav(theme: &Theme) -> impl Widget {
    view()
        .class("site-subnav")
        .fill(view().class("site-subnav-inner").fill((
            link(
                "Get started",
                format!("{}#get-started", route("", theme)),
                "site-subnav-link",
            ),
            link(
                "Theming & styles",
                route("P0-12", theme),
                "site-subnav-link",
            ),
            link(
                "Widgets",
                route("P0-01", theme),
                "site-subnav-link is-active",
            ),
            link("Dates", route("P9-01", theme), "site-subnav-link"),
            link("Charts", route("P11-01", theme), "site-subnav-link"),
            link("Chat", route("P13-01", theme), "site-subnav-link"),
        )))
}

fn source(entry: &'static Entry, theme: Theme, full: bool) -> impl Widget {
    widget(move |scope| {
        let expanded = scope.cage(full);
        let class = scope.bond(move || {
            if expanded.get() {
                "site-code is-expanded".to_owned()
            } else {
                "site-code".to_owned()
            }
        });
        let toggle = scope.bond(move || {
            if expanded.get() {
                "Collapse code".to_owned()
            } else {
                "Expand code".to_owned()
            }
        });
        view()
            .class(class)
            .fill((
                view().class("site-code-header").fill((
                    text().content("Example.rs"),
                    sapid::copy_button()
                        .theme(theme.clone())
                        .size(sapid::Size::ExtraSmall)
                        .label("Copy code")
                        .source(entry.code),
                )),
                view().class("site-code-body").fill(
                    sapid::code_highlight()
                        .theme(theme.clone())
                        .source(entry.code)
                        .language(sapid::CodeLanguage::Rust),
                ),
                when(!full, move || {
                    sapid::button()
                        .unstyled()
                        .class("site-code-toggle")
                        .label(toggle)
                        .on_press(move || expanded.revise(|value| *value = !*value))
                }),
            ))
            .show_in(scope)
    })
}

fn support(entry: &'static Entry, theme: &Theme) -> impl Widget {
    view().class("site-support").fill((
        section_title("support", "Host support"),
        text().class("site-doc-note").content("Choose the host that fits your application. Availability is recorded for each widget and its supported variants."),
        view().class("site-support-row").fill((text().content("Web"), text().content(entry.web))),
        view().class("site-support-row").fill((text().content("Native"), text().content(entry.native))),
        text().class("site-doc-note").content(entry.native_note),
        link("Browse all widgets →", route("P0-01", theme), "site-inline-link"),
    ))
}

fn documentation(entry: &'static Entry, theme: Theme, tab: Cage<u8>) -> impl Widget {
    widget(move |scope| {
        let mut tabs = Vec::new();
        for (index, label) in [(0, "Documentation"), (1, "Source"), (2, "Capabilities")] {
            let class = scope.bond(move || {
                if tab.get() == index {
                    "site-tab is-active".to_owned()
                } else {
                    "site-tab".to_owned()
                }
            });
            tabs.push(
                sapid::button()
                    .unstyled()
                    .class(class)
                    .label(label)
                    .on_press(move || tab.revise(|value| *value = index)),
            );
        }
        let usage = scope.bond(move || tab.get() == 0);
        let code = scope.bond(move || tab.get() == 1);
        let capabilities = scope.bond(move || tab.get() == 2);
        let demo_theme = theme.clone();
        let code_theme = theme.clone();
        let caps_theme = theme.clone();
        view().role(Role::Main).class("site-doc-main").fill((
            text().class("site-breadcrumb").content(format!("Widgets  /  {}", entry.group)),
            text().kind(TextKind::Heading(1)).class("site-title").content(label(entry)),
            text().class("site-lead").content(entry.description),
            view().class("site-doc-meta").fill((
                text().content("Source"), sapid::button().unstyled().class("site-inline-link").label("View source code").on_press(move || tab.revise(|value| *value = 1)),
                text().content("Package"), text().content("sapid"),
                text().content("Built with"), text().content("Glory · handwritten Rust builders"),
            )),
            view().class("site-doc-tabs").name("Documentation views").fill(tabs),
            when(usage, move || {
                view().class("site-doc-content").fill((
                    section_title("usage", "Usage"),
                    view().class("site-preview").fill(view().class("site-demo").fill(crate::showcase_examples::demo(entry.id, demo_theme.clone()))),
                    view().semantics(Some(Box::new(Semantics { id: Some(SemanticId::new("example-source").expect("static id")), ..Default::default() }))).fill(source(entry, demo_theme.clone(), false)),
                    section_title("composition", "Compose your interface"),
                    text().class("site-doc-note").content("Use Cage, Bond and Lotus for reactive values. Compose these builders with other Sapid widgets and keep application data in your own model."),
                    view().class("site-demo-extra").fill(
                        sapid::code_highlight().theme(demo_theme.clone()).language(sapid::CodeLanguage::Rust).source("use glory::prelude::*;\nuse sapid::*;\n\n// Widgets are ordinary Rust builders.\n// Your application chooses Web, SSR or a native host."),
                    ),
                    support(entry, &demo_theme),
                ))
            }),
            when(code, move || view().class("site-doc-content").fill((section_title("source", "Rust source"), source(entry, code_theme.clone(), true)))),
            when(capabilities, move || view().class("site-doc-content").fill(support(entry, &caps_theme))),
            view().role(Role::ContentInfo).class("site-footer").fill((
                text().content("Sapid · Built with Glory"),
                link("Back to overview →", route("", &theme), "site-inline-link"),
            )),
        )).show_in(scope)
    })
}

pub fn site(theme: Theme, selection: String) -> impl Widget {
    widget(move |scope| {
        let query = scope.cage(String::new());
        let menu = scope.cage(false);
        let tab = scope.cage(0u8);
        let contents = scope.bond(move || tab.get() == 0);
        let root_scheme = if theme.scheme == ColorScheme::Dark {
            "site-dark"
        } else {
            "site-light"
        };
        let class = scope.bond(move || {
            format!(
                "sapid-site {root_scheme}{}",
                if menu.get() { " menu-open" } else { "" }
            )
        });
        let selected = find(&selection);
        let content: Box<dyn Widget> = if selection.is_empty() {
            Box::new(crate::showcase_home::home(theme.clone()))
        } else if let Some(entry) = selected {
            Box::new(view().class("site-doc-layout").fill((
                sidebar(query, entry.id, theme.clone()),
                documentation(entry, theme.clone(), tab),
                when(contents, || {
                    view()
                        .role(Role::Navigation)
                        .name("On this page")
                        .class("site-toc")
                        .fill((
                            text().class("site-toc-title").content("Table of contents"),
                            link("Usage", "#usage", "site-toc-link is-active"),
                            link("Example source", "#example-source", "site-toc-link"),
                            link("Composition", "#composition", "site-toc-link"),
                            link("Host support", "#support", "site-toc-link"),
                        ))
                }),
            )))
        } else {
            Box::new(
                view().role(Role::Main).class("site-not-found").fill((
                    text()
                        .kind(TextKind::Heading(1))
                        .content("Widget not found"),
                    text().content("Explore the widget catalogue to find what you need."),
                    link("Browse widgets", route("P0-01", &theme), "site-inline-link"),
                )),
            )
        };
        view()
            .class(class)
            .fill((
                header(
                    query,
                    menu,
                    selected.map_or("", |entry| entry.id),
                    theme.clone(),
                ),
                subnav(&theme),
                if selection.is_empty() {
                    let theme = theme.clone();
                    Box::new(when(menu, move || {
                        view()
                            .role(Role::Navigation)
                            .name("Mobile navigation")
                            .class("site-mobile-nav")
                            .fill((
                                link("Widgets", route("P0-01", &theme), "site-nav-link"),
                                link(
                                    "Get started",
                                    format!("{}#get-started", route("", &theme)),
                                    "site-nav-link",
                                ),
                                link("Charts", route("P11-01", &theme), "site-nav-link"),
                                link("Chat", route("P13-01", &theme), "site-nav-link"),
                            ))
                    })) as Box<dyn Widget>
                } else {
                    Box::new(group()) as Box<dyn Widget>
                },
                content,
            ))
            .show_in(scope)
    })
}
