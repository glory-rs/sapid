use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
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
}
