//! Bounded application navigation; no router or transport is installed.
use crate::appearance::{Common, configure};
use crate::composite::{self, Callback, Observers};
use crate::state::Value;
use crate::{AnchorRequest, ContentPolicy};
use glory::render::{Current, Event, EventKind, Role, Semantics};
use glory::style::{Display, FlexDirection, Style};
use glory::ui::{IntoChildren, Result, Widget};
use glory::{Lotus, Scope};

pub const MAX_BREADCRUMB_ITEMS: usize = 128;
pub const MAX_PAGINATION_WINDOW_ITEMS: usize = 35;

macro_rules! navigation_methods {
    () => {
        pub fn theme(mut self, value: crate::Theme) -> Self {
            self.common.theme = value;
            self
        }
        pub fn size(mut self, value: crate::Size) -> Self {
            self.common.size = value;
            self
        }
        pub fn unstyled(mut self) -> Self {
            self.common.unstyled = true;
            self
        }
        pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
            self.common.class = value.into();
            self
        }
        pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
            self.common.style = value.into();
            self
        }
        pub fn name(mut self, value: impl Into<Lotus<String>>) -> Self {
            self.common.name = Some(value.into());
            self
        }
        pub fn node_ref(mut self, value: glory::Cage<Option<glory::render::NodeId>>) -> Self {
            self.common.node_ref = Some(value);
            self
        }
        pub fn on(mut self, kind: EventKind, callback: impl FnMut(Event) + 'static) -> Self {
            self.common
                .handlers
                .entry(kind)
                .or_default()
                .push(Box::new(callback));
            self
        }
    };
}
fn row() -> Style {
    Style::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .gap(8.0)
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn observers(common: &mut Common) -> Observers {
    Observers::new(
        common
            .handlers
            .remove(&EventKind::Activate)
            .unwrap_or_default(),
    )
}
fn attach(mut root: glory::ui::View, common: &mut Common, scope: &Scope) -> glory::ui::View {
    let owner = scope.id();
    for (kind, mut handlers) in std::mem::take(&mut common.handlers) {
        root = root.on(kind, move |event| {
            crate::appearance::observe(&mut handlers, owner, event)
        });
    }
    root
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageItem {
    Page(u64),
    Gap,
}

/// At most 35 entries, independent of total. Unknown totals expose only current.
pub fn pagination_window(
    page: u64,
    total: Option<u64>,
    siblings: u8,
    boundaries: u8,
) -> Result<Vec<PageItem>> {
    if page == 0
        || total.is_some_and(|total| total == 0 || page > total)
        || siblings > 8
        || boundaries > 8
    {
        return Err(invalid(
            "pagination needs positive bounded pages and windows <= 8",
        ));
    }
    let Some(total) = total else {
        return Ok(vec![PageItem::Page(page)]);
    };
    let mut result = Vec::with_capacity(MAX_PAGINATION_WINDOW_ITEMS);
    let mut last = 0_u64;
    let mut push = |first: u64, end: u64| {
        for value in first..=end {
            if value <= last {
                continue;
            }
            if last != 0 && value - last > 1 {
                result.push(PageItem::Gap);
            }
            result.push(PageItem::Page(value));
            last = value;
        }
    };
    push(1, u64::from(boundaries).min(total));
    push(
        page.saturating_sub(u64::from(siblings)).max(1),
        page.saturating_add(u64::from(siblings)).min(total),
    );
    if boundaries != 0 {
        push(
            total.saturating_sub(u64::from(boundaries) - 1).max(1),
            total,
        );
    }
    Ok(result)
}

#[derive(Clone, PartialEq, Eq)]
struct PageState {
    page: u64,
    total: Option<u64>,
    has_next: bool,
}
pub struct Pagination {
    common: Common,
    key: String,
    page: Value<u64>,
    total: Lotus<Option<u64>>,
    has_next: Lotus<bool>,
    siblings: u8,
    boundaries: u8,
    change: Option<Box<dyn FnMut(u64)>>,
}
pub fn pagination(key: impl Into<String>) -> Pagination {
    Pagination {
        common: Common::default(),
        key: key.into(),
        page: Value {
            source: None,
            default: 1,
        },
        total: None.into(),
        has_next: false.into(),
        siblings: 1,
        boundaries: 1,
        change: None,
    }
}
impl Pagination {
    navigation_methods!();
    pub fn enabled(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.common.enabled = value.into();
        self
    }
    pub fn page(mut self, value: impl Into<Lotus<u64>>) -> Self {
        self.page.source = Some(value.into());
        self
    }
    pub fn default_page(mut self, value: u64) -> Self {
        self.page.default = value;
        self
    }
    pub fn total(mut self, value: impl Into<Lotus<Option<u64>>>) -> Self {
        self.total = value.into();
        self
    }
    pub fn has_next(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.has_next = value.into();
        self
    }
    pub fn siblings(mut self, value: u8) -> Self {
        self.siblings = value;
        self
    }
    pub fn boundaries(mut self, value: u8) -> Self {
        self.boundaries = value;
        self
    }
    pub fn on_page_change(mut self, callback: impl FnMut(u64) + 'static) -> Self {
        self.change = Some(Box::new(callback));
        self
    }
}
impl Widget for Pagination {
    fn build(&mut self, scope: &Scope) -> Result {
        composite::id("pagination", &self.key, "root", "root")?;
        let control = std::mem::replace(
            &mut self.page,
            Value {
                source: None,
                default: 1,
            },
        )
        .mount(scope);
        let source = control.source.clone();
        let total = self.total.clone();
        let next = self.has_next.clone();
        let siblings = self.siblings;
        let boundaries = self.boundaries;
        let accepted = composite::validated(
            scope,
            &scope
                .bond(move || PageState {
                    page: source.get(),
                    total: total.get(),
                    has_next: next.get(),
                })
                .into(),
            move |state: &PageState| {
                pagination_window(state.page, state.total, siblings, boundaries).map(|_| ())
            },
        )?;
        let window = scope.bond(move || {
            let state = accepted.get();
            pagination_window(state.page, state.total, siblings, boundaries)
                .expect("accepted pagination")
        });
        let callback = Callback::new(self.change.take());
        let observers = observers(&mut self.common);
        if self.common.name.is_none() {
            self.common.name = Some("Pagination".into());
        }
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-pagination",
            crate::appearance::merge(self.common.theme.label(self.common.size), row())
        )
        .role(Role::Navigation);
        let owner = scope.id();
        // Four directional controls and a fixed number of live numeric slots.
        for index in 0..(MAX_PAGINATION_WINDOW_ITEMS + 4) {
            let slot = index.checked_sub(4);
            let label = scope.bond(move || match index {
                0 => "First page".to_owned(),
                1 => "Previous page".to_owned(),
                2 => "Next page".to_owned(),
                3 => "Last page".to_owned(),
                _ => match window.get().get(slot.unwrap()) {
                    Some(PageItem::Page(page)) => format!("Page {page}"),
                    _ => String::new(),
                },
            });
            let enabled = self.common.enabled.clone();
            let enabled = scope.bond(move || {
                let state = accepted.get();
                enabled.get()
                    && match index {
                        0 | 1 => state.page > 1,
                        2 => {
                            state.page < u64::MAX
                                && state
                                    .total
                                    .map_or(state.has_next, |total| state.page < total)
                        }
                        3 => state.total.is_some_and(|total| state.page < total),
                        _ => matches!(window.get().get(slot.unwrap()), Some(PageItem::Page(_))),
                    }
            });
            let metadata = scope.bond(move || {
                composite::metadata(Semantics {
                    current: match slot.and_then(|slot| window.get().get(slot).copied()) {
                        Some(PageItem::Page(page)) if page == accepted.get().page => {
                            Some(Current::Page)
                        }
                        _ => None,
                    },
                    ..Default::default()
                })
            });
            let callback = callback.clone();
            let observers = observers.clone();
            let control = control.clone();
            let weak = scope.downgrade();
            let mut button = glory::ui::button()
                .label(label)
                .enabled(enabled)
                .semantics(metadata)
                .class("sapid-button sapid-page-button");
            if !self.common.unstyled {
                let theme = self.common.theme.clone();
                let size = self.common.size;
                button = button.style(scope.bond(move || {
                    let mut style = theme.control(size);
                    if slot.and_then(|slot| window.get().get(slot).copied())
                        == Some(PageItem::Page(accepted.get().page))
                    {
                        style.background = Some(theme.accent);
                        style.color = Some(theme.on_accent);
                    }
                    style
                }));
            }
            button = button.on(EventKind::Activate, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    if !enabled.try_get()? {
                        return Ok(());
                    }
                    let state = accepted.try_get()?;
                    let proposal = match index {
                        0 => Some(1),
                        1 => state.page.checked_sub(1),
                        2 => state.page.checked_add(1),
                        3 => state.total,
                        _ => match window.try_get()?.get(slot.unwrap()) {
                            Some(PageItem::Page(page)) => Some(*page),
                            _ => None,
                        },
                    };
                    if let Some(page) = proposal
                        && page != state.page
                    {
                        control.accept(&page)?;
                        callback.emit(owner, page);
                    }
                    if owner.is_alive() {
                        observers.emit(owner, event);
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                }
            });
            if let Some(slot) = slot {
                let visible = scope
                    .bond(move || matches!(window.get().get(slot), Some(PageItem::Page(_))));
                let gap =
                    scope.bond(move || matches!(window.get().get(slot), Some(PageItem::Gap)));
                root = root.fill(
                    glory::ui::view()
                        .class("sapid-page-slot")
                        .fill((
                            glory::ui::view()
                                .semantics(scope.bond(move || {
                                    composite::metadata(Semantics {
                                        hidden: !visible.get(),
                                        ..Default::default()
                                    })
                                }))
                                .fill(button),
                            glory::ui::view()
                                .semantics(scope.bond(move || {
                                    composite::metadata(Semantics {
                                        hidden: !gap.get(),
                                        ..Default::default()
                                    })
                                }))
                                .fill(glory::ui::text().content("…")),
                        ))
                        .semantics(scope.bond(move || {
                            composite::metadata(Semantics {
                                hidden: window.get().get(slot).is_none(),
                                ..Default::default()
                            })
                        })),
                );
            } else {
                root = root.fill(button);
            }
        }
        attach(root, &mut self.common, scope).show_in(scope)
    }
}

pub struct BreadcrumbItem {
    key: String,
    label: Lotus<String>,
    href: Option<Lotus<String>>,
}
pub fn breadcrumb_item(
    key: impl Into<String>,
    label: impl Into<Lotus<String>>,
) -> BreadcrumbItem {
    BreadcrumbItem {
        key: key.into(),
        label: label.into(),
        href: None,
    }
}
impl BreadcrumbItem {
    pub fn href(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.href = Some(value.into());
        self
    }
}
pub struct Breadcrumbs {
    common: Common,
    key: String,
    items: Vec<BreadcrumbItem>,
    current: Option<Lotus<String>>,
    max_items: usize,
    expanded: Value<bool>,
    change: Option<Box<dyn FnMut(bool)>>,
}
pub fn breadcrumbs(key: impl Into<String>) -> Breadcrumbs {
    Breadcrumbs {
        common: Common::default(),
        key: key.into(),
        items: Vec::new(),
        current: None,
        max_items: 5,
        expanded: Value::default(),
        change: None,
    }
}
impl Breadcrumbs {
    navigation_methods!();
    pub fn items(mut self, items: impl IntoIterator<Item = BreadcrumbItem>) -> Self {
        self.items = items.into_iter().take(MAX_BREADCRUMB_ITEMS + 1).collect();
        self
    }
    pub fn current(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.current = Some(value.into());
        self
    }
    pub fn max_items(mut self, value: usize) -> Self {
        self.max_items = value;
        self
    }
    pub fn expanded(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.expanded.source = Some(value.into());
        self
    }
    pub fn default_expanded(mut self, value: bool) -> Self {
        self.expanded.default = value;
        self
    }
    pub fn on_expanded_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.change = Some(Box::new(callback));
        self
    }
}
impl Widget for Breadcrumbs {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.items.len() > MAX_BREADCRUMB_ITEMS || self.max_items < 2 {
            return Err(invalid(
                "breadcrumbs require <= 128 items and max_items >= 2",
            ));
        }
        let keys: Vec<_> = self.items.iter().map(|item| item.key.clone()).collect();
        composite::validate_keys(&keys)?;
        composite::id("breadcrumbs", &self.key, "root", "root")?;
        let focus_keys = keys.clone();
        let default = keys.last().cloned().unwrap_or_default();
        let current = composite::validated(
            scope,
            &self.current.take().unwrap_or_else(|| default.into()),
            move |value| {
                if value.is_empty() || keys.contains(value) {
                    Ok(())
                } else {
                    Err(invalid("unknown current breadcrumb"))
                }
            },
        )?;
        let control = std::mem::take(&mut self.expanded).mount(scope);
        let expanded = control.source.clone();
        let collapse = self.items.len() > self.max_items;
        let first_middle = 1;
        let last_middle = self.items.len().saturating_sub(self.max_items - 2);
        let trigger = scope.cage(None);
        let focused = scope.cage(None::<usize>);
        let callback = Callback::new(self.change.take());
        let observers = observers(&mut self.common);
        let owner = scope.id();
        if self.common.name.is_none() {
            self.common.name = Some("Breadcrumbs".into());
        }
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-breadcrumbs",
            self.common.theme.label(self.common.size)
        )
        .role(Role::Navigation);
        let mut list = glory::ui::view().role(Role::List).style(row());
        for (index, item) in std::mem::take(&mut self.items).into_iter().enumerate() {
            if collapse && index == first_middle {
                let open = expanded.clone();
                let label = scope.bond(move || {
                    if open.get() {
                        "Collapse breadcrumb path"
                    } else {
                        "Expand breadcrumb path"
                    }
                    .to_owned()
                });
                let open = expanded.clone();
                let mut button = glory::ui::button()
                    .label(label)
                    .node_ref(trigger)
                    .class("sapid-button sapid-breadcrumb-expand")
                    .semantics(scope.bond(move || {
                        composite::metadata(Semantics {
                            expanded: Some(open.get()),
                            ..Default::default()
                        })
                    }));
                if !self.common.unstyled {
                    button = button.style(self.common.theme.control(self.common.size));
                }
                let control = control.clone();
                let callback = callback.clone();
                let observers = observers.clone();
                let weak = scope.downgrade();
                button = button.on(EventKind::Activate, move |event| {
                    let Some(scope) = weak.upgrade() else { return };
                    let result = (|| -> Result {
                        let next = !control.source.try_get()?;
                        control.accept(&next)?;
                        callback.emit(owner, next);
                        if owner.is_alive() {
                            observers.emit(owner, event);
                        }
                        Ok(())
                    })();
                    if let Err(error) = result {
                        scope.report_error(error);
                    }
                });
                list = list.fill(glory::ui::view().role(Role::ListItem).fill(button));
            }
            let middle = collapse && (first_middle..last_middle).contains(&index);
            let key = item.key;
            let key_for_current = key.clone();
            let state =
                scope.bond(move || (current.get() == key_for_current).then_some(Current::Page));
            let open = expanded.clone();
            let visible = scope.bond(move || !middle || open.get() || current.get() == key);
            let mut shell = glory::ui::view()
                .role(Role::ListItem)
                .semantics(scope.bond(move || {
                    composite::metadata(Semantics {
                        hidden: !visible.get(),
                        ..Default::default()
                    })
                }))
                .on(EventKind::FocusWithin, move |event| {
                    if let Event::FocusWithin { focused: value, .. } = event {
                        focused.revise(|old| {
                            if value {
                                *old = Some(index)
                            } else if *old == Some(index) {
                                *old = None
                            }
                        });
                    }
                });
            shell = if let Some(href) = item.href {
                let mut link = crate::anchor()
                    .href(href)
                    .label(item.label)
                    .current(state)
                    .theme(self.common.theme.clone())
                    .size(self.common.size);
                if self.common.unstyled {
                    link = link.unstyled();
                }
                shell.fill(link)
            } else {
                shell.fill(glory::ui::text().content(item.label).semantics(scope.bond(
                    move || {
                        composite::metadata(Semantics {
                            current: state.get(),
                            ..Default::default()
                        })
                    },
                )))
            };
            list = list.fill(shell);
        }
        let open = expanded;
        let weak = scope.downgrade();
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            if !open.get()
                && focused.get().is_some_and(|index| {
                    (first_middle..last_middle).contains(&index)
                        && focus_keys[index] != current.get()
                })
            {
                focused.revise(|value| *value = None);
                if let Err(error) = composite::focus(&scope, trigger) {
                    scope.report_error(error);
                }
            }
        });
        root = root.fill(list);
        attach(root, &mut self.common, scope).show_in(scope)
    }
}

pub struct NavLink {
    common: Common,
    key: String,
    href: Lotus<String>,
    label: Lotus<String>,
    current: Lotus<bool>,
    highlighted: Lotus<bool>,
    expanded: Value<bool>,
    change: Option<Box<dyn FnMut(bool)>>,
    navigate: Option<Box<dyn FnMut(AnchorRequest)>>,
    children: Option<composite::Factory>,
    policy: ContentPolicy,
}
pub fn nav_link(key: impl Into<String>) -> NavLink {
    NavLink {
        common: Common::default(),
        key: key.into(),
        href: "".into(),
        label: "".into(),
        current: false.into(),
        highlighted: false.into(),
        expanded: Value::default(),
        change: None,
        navigate: None,
        children: None,
        policy: ContentPolicy::Unmount,
    }
}
impl NavLink {
    navigation_methods!();
    pub fn href(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.href = value.into();
        self
    }
    pub fn label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.label = value.into();
        self
    }
    pub fn current(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.current = value.into();
        self
    }
    /// Presentation only. This does not claim selection or browser visit state.
    pub fn highlighted(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.highlighted = value.into();
        self
    }
    pub fn expanded(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.expanded.source = Some(value.into());
        self
    }
    pub fn default_expanded(mut self, value: bool) -> Self {
        self.expanded.default = value;
        self
    }
    pub fn on_expanded_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.change = Some(Box::new(callback));
        self
    }
    pub fn on_navigate(mut self, callback: impl FnMut(AnchorRequest) + 'static) -> Self {
        self.navigate = Some(Box::new(callback));
        self
    }
    pub fn children<W: IntoChildren + 'static>(
        mut self,
        mut factory: impl FnMut() -> W + 'static,
    ) -> Self {
        self.children = Some(Box::new(move || {
            Box::new(glory::ui::group().fill(factory()))
        }));
        self
    }
    pub fn content_policy(mut self, value: ContentPolicy) -> Self {
        self.policy = value;
        self
    }
}
impl Widget for NavLink {
    fn build(&mut self, scope: &Scope) -> Result {
        let link_id = composite::id("nav", &self.key, "link", "link")?;
        let panel_id = composite::id("nav", &self.key, "panel", "panel")?;
        let trigger = scope.cage(None);
        let control = std::mem::take(&mut self.expanded).mount(scope);
        let open = control.source.clone();
        let current = self.current.clone();
        let observers = observers(&mut self.common);
        let owner = scope.id();
        let link_observers = observers.clone();
        let mut link = crate::anchor()
            .href(self.href.clone())
            .label(self.label.clone())
            .semantic_id(link_id.clone())
            .current(scope.bond(move || current.get().then_some(Current::Page)))
            .theme(self.common.theme.clone())
            .size(self.common.size);
        link = link.on(EventKind::Activate, move |event| {
            link_observers.emit(owner, event)
        });
        if self.common.unstyled {
            link = link.unstyled();
        }
        if let Some(callback) = self.navigate.take() {
            link = link.on_navigate(callback);
        }
        let highlight = self.highlighted.clone();
        let theme = self.common.theme.clone();
        let unstyled = self.common.unstyled;
        let style = scope.bond(move || {
            if highlight.get() && !unstyled {
                Style {
                    background: Some(theme.accent),
                    color: Some(theme.on_accent),
                    ..Style::new()
                }
            } else {
                Style::new()
            }
        });
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-nav-link",
            Style::new()
        )
        .fill(link.style(style));
        if let Some(factory) = self.children.take() {
            let callback = Callback::new(self.change.take());
            let expanded = open.clone();
            let panel = panel_id.clone();
            let label = self.label.clone();
            let label_open = open.clone();
            let mut button = glory::ui::button()
                .node_ref(trigger)
                .class("sapid-button sapid-nav-expand")
                .label(scope.bond(move || {
                    format!(
                        "{} {}",
                        if label_open.get() {
                            "Collapse"
                        } else {
                            "Expand"
                        },
                        label.get()
                    )
                }))
                .semantics(scope.bond(move || {
                    composite::metadata(Semantics {
                        controls: Some(panel.clone()),
                        expanded: Some(expanded.get()),
                        ..Default::default()
                    })
                }));
            if !self.common.unstyled {
                button = button.style(self.common.theme.control(self.common.size));
            }
            let weak = scope.downgrade();
            button = button.on(EventKind::Activate, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    let next = !control.source.try_get()?;
                    control.accept(&next)?;
                    callback.emit(owner, next);
                    if owner.is_alive() {
                        observers.emit(owner, event);
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                }
            });
            root = root.fill((
                button,
                composite::Panel {
                    id: panel_id,
                    label: link_id,
                    role: None,
                    open,
                    trigger,
                    restore: None,
                    reference: None,
                    factory: Some(factory),
                    policy: self.policy,
                    class: "sapid-nav-children",
                    unstyled: self.common.unstyled,
                },
            ));
        }
        attach(root, &mut self.common, scope).show_in(scope)
    }
}
