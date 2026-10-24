//! Included as a child of `menu` once the shared Menubar protocol is installed.
use super::*;
use glory::render::Role;

pub const MAX_MENUBAR_ROOTS: usize = 32;

/// One intrinsic menuitem trigger and its ordinary Menu tree.
pub struct MenubarMenu {
    inner: Menu,
}
pub fn menubar_menu(key: impl Into<String>) -> MenubarMenu {
    MenubarMenu {
        inner: Menu {
            dropdown: true,
            ..menu(key)
        },
    }
}
impl MenubarMenu {
    dropdown_methods! {
        label(value: impl Into<Lotus<String>>); item(item: impl Into<MenuEntry>);
        items(items: impl IntoIterator<Item=MenuItem>); radio_group(group:MenuRadioGroup);
        submenu(submenu:MenuSubmenu); separator(key:impl Into<String>);
        enabled(value:impl Into<Lotus<bool>>);
    }
}

pub struct Menubar {
    common: Common,
    key: String,
    value: Value<Option<String>>,
    changed: Option<Box<dyn FnMut(Option<String>)>>,
    menus: Vec<MenubarMenu>,
    overflow: bool,
    direction: Direction,
    looping: bool,
    typeahead: TypeaheadConfig,
    delay: Option<Delay>,
}
pub fn menubar(key: impl Into<String>) -> Menubar {
    Menubar {
        common: Default::default(),
        key: key.into(),
        value: Default::default(),
        changed: None,
        menus: Vec::new(),
        overflow: false,
        direction: Direction::Ltr,
        looping: true,
        typeahead: Default::default(),
        delay: None,
    }
}
impl Menubar {
    common_methods!();
    pub fn value(mut self, value: impl Into<Lotus<Option<String>>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: Option<String>) -> Self {
        self.value.default = value;
        self
    }
    pub fn on_value_change(mut self, callback: impl FnMut(Option<String>) + 'static) -> Self {
        self.changed = Some(Box::new(callback));
        self
    }
    pub fn menu(mut self, menu: MenubarMenu) -> Self {
        if self.overflow {
            return self;
        }
        let total = self
            .menus
            .iter()
            .map(|menu| 1 + menu.inner.entries.iter().map(entry_size).sum::<usize>())
            .sum::<usize>()
            + 1
            + menu.inner.entries.iter().map(entry_size).sum::<usize>();
        if self.menus.len() >= MAX_MENUBAR_ROOTS
            || total > MAX_MENU_ITEMS
            || menu.inner.overflow
        {
            self.overflow = true;
        } else {
            self.menus.push(menu);
        }
        self
    }
    pub fn menus(mut self, menus: impl IntoIterator<Item = MenubarMenu>) -> Self {
        if self.overflow {
            return self;
        }
        for menu in menus.into_iter().take(MAX_MENUBAR_ROOTS + 1) {
            self = self.menu(menu);
            if self.overflow {
                break;
            }
        }
        self
    }
    pub fn direction(mut self, value: Direction) -> Self {
        self.direction = value;
        self
    }
    pub fn loop_focus(mut self, value: bool) -> Self {
        self.looping = value;
        self
    }
    pub fn typeahead(mut self, value: TypeaheadConfig) -> Self {
        self.typeahead = value;
        self
    }
    pub fn hover_delay_with<F: Fn() -> Fut + 'static, Fut: Future<Output = ()> + 'static>(
        mut self,
        factory: F,
    ) -> Self {
        self.delay = Some(Rc::new(move || Box::pin(factory())));
        self
    }
}

struct BarRoot {
    key: String,
    trigger: NodeId,
    state: Rc<MenuState>,
    member: CollectionMember,
}
pub(super) struct BarState {
    control: Control<Option<String>>,
    accepted: Cage<Option<String>>,
    keys: Rc<Vec<String>>,
    changed: Callback<Option<String>>,
    collection: Collection,
    roving: RovingFocus,
    search: RefCell<Typeahead>,
    roots: RefCell<Vec<BarRoot>>,
    owner: glory::core::InstanceId,
    enabled: Lotus<bool>,
    focus: Cage<Option<BarFocus>>,
}
#[derive(Clone, PartialEq, Eq)]
struct BarFocus {
    key: String,
    boundary: Option<bool>,
}
impl BarState {
    pub(super) fn register(
        self: &Rc<Self>,
        scope: &Scope,
        key: &str,
        reference: Cage<Option<NodeId>>,
        enabled: Lotus<bool>,
        label: Lotus<String>,
        state: Rc<MenuState>,
    ) -> Result {
        let member = self
            .collection
            .register(scope, key, reference, enabled, label)?;
        let trigger = reference.try_get()?.ok_or(glory::ui::Error::Gone)?;
        let roving = self.roving.clone();
        let current = member.clone();
        let weak = scope.downgrade();
        scope.on(trigger, EventKind::FocusChanged, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            if event == Event::FocusChanged(true)
                && let Err(error) = roving.observe(&current)
            {
                scope.report_error(error);
            }
        })?;
        self.roots.borrow_mut().push(BarRoot {
            key: key.to_owned(),
            trigger,
            state,
            member,
        });
        Ok(())
    }
    fn read(&self) -> Result<Option<String>> {
        self.control.source.try_with(|value| {
            validate_value(&self.keys, value)?;
            Ok(value.clone())
        })?
    }
    fn request(&self, value: Option<String>) -> Result<bool> {
        if !self.owner.is_alive() || !self.enabled.try_get()? {
            return Ok(false);
        }
        validate_value(&self.keys, &value)?;
        if let Some(key) = &value {
            let enabled = self
                .collection
                .mounted()?
                .into_iter()
                .any(|item| item.key == *key && item.enabled);
            if !enabled {
                return Ok(false);
            }
        }
        let previous = self.read()?;
        if previous != value {
            self.control.accept(&value)?;
            self.changed.emit(self.owner, value.clone());
        }
        if !self.owner.is_alive() {
            return Ok(false);
        }
        if self.read()? != value {
            return Ok(false);
        }
        if previous != value {
            let inactive = {
                self.roots
                    .borrow()
                    .iter()
                    .filter(|root| Some(root.key.as_str()) != value.as_deref())
                    .map(|root| root.state.clone())
                    .collect::<Vec<_>>()
            };
            for root in inactive {
                root.cancel_pending();
                root.close_children(None)?;
                if !self.owner.is_alive() {
                    return Ok(false);
                }
            }
        }
        Ok(self.read()? == value)
    }
    pub(super) fn toggle(&self, key: &str) -> Result<bool> {
        self.request(if self.read()?.as_deref() == Some(key) {
            None
        } else {
            Some(key.to_owned())
        })
    }
    pub(super) fn set_root(&self, key: &str, open: bool) -> Result {
        if open {
            self.request(Some(key.to_owned()))?;
        } else if self.read()?.as_deref() == Some(key) {
            self.request(None)?;
        }
        Ok(())
    }
    fn focus_destination(
        &self,
        _scope: &Scope,
        item: crate::MountedItem,
        open: bool,
        last: bool,
    ) -> Result {
        if !self.owner.is_alive() {
            return Ok(());
        }
        if open && !self.request(Some(item.key.clone()))? {
            return Ok(());
        }
        if !self.owner.is_alive() || !self.enabled.try_get()? {
            return Ok(());
        }
        let Some(item) = self.collection.mounted()?.into_iter().find(|current| {
            current.node == item.node && current.member == item.member && current.enabled
        }) else {
            return Ok(());
        };
        self.roving.accept(&item)?;
        self.focus.try_revise(|intent| {
            *intent = Some(BarFocus {
                key: item.key,
                boundary: open.then_some(last),
            })
        })?;
        Ok(())
    }
    pub(super) fn move_from_menu(&self, scope: &Scope, key: &str, next: bool) -> Result {
        let node = {
            self.roots
                .borrow()
                .iter()
                .find(|root| root.key == key)
                .map(|root| root.trigger)
        };
        if let Some(node) = node
            && let Some(item) = self.roving.destination(
                node,
                if next {
                    Navigation::Next
                } else {
                    Navigation::Previous
                },
            )?
        {
            self.focus_destination(scope, item, true, false)?;
        }
        Ok(())
    }
    fn navigate(&self, scope: &Scope, from: NodeId, command: Navigation) -> Result {
        if !self.enabled.try_get()? {
            return Ok(());
        }
        self.search.borrow_mut().clear();
        if let Some(item) = self.roving.destination(from, command)? {
            self.focus_destination(scope, item, self.read()?.is_some(), false)?;
        }
        Ok(())
    }
    fn typeahead(&self, scope: &Scope, from: NodeId, text: &str, at_millis: u64) -> Result {
        if !self.enabled.try_get()? {
            return Ok(());
        }
        let destination =
            self.search
                .borrow_mut()
                .search(&self.collection, from, text, at_millis)?;
        if let Some(item) = destination {
            self.focus_destination(scope, item, self.read()?.is_some(), false)?;
        }
        Ok(())
    }
    pub(super) fn open_from_trigger(&self, scope: &Scope, key: &str, last: bool) -> Result {
        if !self.request(Some(key.to_owned()))? {
            return Ok(());
        }
        let destination = self
            .collection
            .mounted()?
            .into_iter()
            .find(|item| item.key == key && item.enabled);
        if let Some(item) = destination {
            self.focus_destination(scope, item, true, last)?;
        }
        Ok(())
    }
}
fn validate_value(keys: &[String], value: &Option<String>) -> Result {
    if let Some(value) = value
        && (value.len() > 64 || !keys.contains(value))
    {
        return Err(invalid("menubar value must be one declared root key"));
    }
    Ok(())
}

/// Internal per-trigger metadata; never substitutes an arbitrary View's role.
pub(super) struct BarItem {
    pub owner: SemanticId,
    pub key: String,
    pub state: Weak<BarState>,
    pub focus: Lotus<FocusMode>,
}

impl Widget for Menubar {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.overflow || self.menus.is_empty() {
            return Err(invalid("menubar requires bounded nonempty roots"));
        }
        key(&self.key)?;
        if self.common.focus != FocusMode::Auto {
            return Err(invalid("menubar owns its roving tab entry"));
        }
        let owner_id = SemanticId::new(format!("sapid-menubar-{}", self.key))?;
        let mut keys = Vec::with_capacity(self.menus.len());
        let mut names = HashSet::new();
        for root in &mut self.menus {
            key(&root.inner.key)?;
            if !names.insert(root.inner.key.clone()) {
                return Err(invalid("duplicate menubar root key"));
            }
            // Validate all declarations before constructing any live root.
            let namespace = format!("{}-{}-{}", self.key.len(), self.key, root.inner.key);
            key(&namespace)?;
            validate(&namespace, &root.inner.entries, &mut Vec::new(), &mut 0)?;
            keys.push(root.inner.key.clone());
            let limit = CollectionLimits::default().label_bytes;
            root.inner.label = composite::validated(scope, &root.inner.label, move |label| {
                if label.len() <= limit {
                    Ok(())
                } else {
                    Err(invalid("menubar root label exceeds collection budget"))
                }
            })?
            .into();
        }
        Typeahead::new(self.typeahead)?;
        let keys = Rc::new(keys);
        let control = std::mem::take(&mut self.value).mount(scope);
        let checked = keys.clone();
        let accepted = composite::validated(scope, &control.source, move |value| {
            validate_value(&checked, value)
        })?;
        let collection = Collection::new(
            scope,
            CollectionLimits {
                items: MAX_MENUBAR_ROOTS,
                ..Default::default()
            },
        )?;
        let roving = RovingFocus::new(scope, collection.clone(), self.looping)?;
        let state = Rc::new(BarState {
            control,
            accepted,
            keys,
            changed: Callback::new(self.changed.take()),
            collection,
            roving,
            search: RefCell::new(Typeahead::new(self.typeahead)?),
            roots: RefCell::new(Vec::new()),
            owner: scope.id(),
            enabled: self.common.enabled.clone(),
            focus: scope.cage(None),
        });
        let navigation_observers = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Navigate)
                .unwrap_or_default(),
        );
        let typeahead_observers = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Typeahead)
                .unwrap_or_default(),
        );
        let instance = scope.id();
        let navigation = state.clone();
        let weak = scope.downgrade();
        let typeahead = state.clone();
        let search_scope = scope.downgrade();
        let id = owner_id.clone();
        let direction = self.direction;
        let enabled = self.common.enabled.clone();
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let name = self.common.name.take().unwrap_or_else(|| "Menu bar".into());
        let bar = configure!(
            self.common,
            glory::ui::view().role(Role::Menubar),
            scope,
            "sapid-menubar",
            glory::style::Style::new()
                .display(glory::style::Display::Flex)
                .flex_direction(glory::style::FlexDirection::Row)
                .gap(2.0)
        )
        .name(name)
        .node_ref(reference)
        .semantics(scope.bond(move || {
            Some(Box::new(Semantics {
                id: Some(id.clone()),
                disabled: !enabled.get(),
                navigation: Some(Orientation::Horizontal),
                navigation_direction: direction,
                typeahead: true,
                ..Default::default()
            }))
        }))
        .on(EventKind::Navigate, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            if let Event::Navigate { from, command } = &event
                && let Err(error) = navigation.navigate(&scope, *from, *command)
            {
                scope.report_error(error);
                return;
            }
            navigation_observers.emit(instance, event);
        })
        .on(EventKind::Typeahead, move |event| {
            let Some(scope) = search_scope.upgrade() else {
                return;
            };
            if let Event::Typeahead {
                from,
                text,
                at_millis,
            } = &event
                && let Err(error) = typeahead.typeahead(&scope, *from, text, *at_millis)
            {
                scope.report_error(error);
                return;
            }
            typeahead_observers.emit(instance, event);
        })
        .mount(scope)?;
        let contents = bar
            .scope()
            .child_in(reference.get().ok_or(glory::ui::Error::Gone)?)?;
        for mut root in std::mem::take(&mut self.menus) {
            let key = root.inner.key.clone();
            let namespace = format!("{}-{}-{key}", self.key.len(), self.key);
            root.inner.key = namespace;
            let token = scope.cage(None::<CollectionMember>);
            let current = state.roving.clone();
            let focus = scope.bond(move || {
                if token
                    .get()
                    .is_some_and(|member| current.is_tab_stop(&member).unwrap_or(false))
                {
                    FocusMode::TabStop
                } else {
                    FocusMode::Programmatic
                }
            });
            let selected = state.accepted;
            let selected_key = key.clone();
            let bar_enabled = state.enabled.clone();
            let own_enabled = root.inner.common.enabled.clone();
            let enabled: Lotus<bool> = scope
                .bond(move || bar_enabled.get() && own_enabled.get())
                .into();
            let visible = enabled.clone();
            root.inner.open.source = Some(
                scope
                    .bond(move || {
                        visible.get() && selected.get().as_deref() == Some(&selected_key)
                    })
                    .into(),
            );
            let handle = Rc::downgrade(&state);
            let callback_key = key.clone();
            let callback_scope = scope.downgrade();
            root.inner.changed = Some(Box::new(move |open| {
                if let Some(state) = handle.upgrade()
                    && let Err(error) = state.set_root(&callback_key, open)
                    && let Some(scope) = callback_scope.upgrade()
                {
                    scope.report_error(error);
                }
            }));
            root.inner.common.enabled = enabled;
            root.inner.common.theme = self.common.theme.clone();
            root.inner.common.size = self.common.size;
            root.inner.common.unstyled |= self.common.unstyled;
            root.inner.direction = self.direction;
            root.inner.looping = self.looping;
            root.inner.typeahead = self.typeahead;
            root.inner.delay = self.delay.clone();
            root.inner.bar = Some(BarItem {
                owner: owner_id.clone(),
                key: key.clone(),
                state: Rc::downgrade(&state),
                focus: focus.into(),
            });
            let trigger = scope.cage(None);
            root.inner.trigger = Some(trigger);
            root.inner.show_in(&contents)?;
            // Menu::build registers the complete root after mounting its trigger.
            let member = state
                .roots
                .borrow()
                .last()
                .filter(|root| root.key == key)
                .map(|root| root.member.clone())
                .ok_or(glory::ui::Error::Gone)?;
            token.revise(|value| *value = Some(member));
        }
        state.roving.repair(state.accepted.try_get()?.as_deref())?;
        let live = state.clone();
        let weak = scope.downgrade();
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let selected = live.accepted.try_get()?;
                live.roving.repair(selected.as_deref())?;
                let Some(intent) = live.focus.try_get()? else {
                    return Ok(());
                };
                live.focus.try_revise(|value| *value = None)?;
                if !live.enabled.try_get()?
                    || (intent.boundary.is_some()
                        && selected.as_deref() != Some(intent.key.as_str()))
                    || (intent.boundary.is_none() && selected.is_some())
                {
                    return Ok(());
                }
                let item = live
                    .collection
                    .mounted()?
                    .into_iter()
                    .find(|item| item.key == intent.key && item.enabled);
                let Some(item) = item else { return Ok(()) };
                let root = {
                    live.roots
                        .borrow()
                        .iter()
                        .find(|root| root.trigger == item.node)
                        .map(|root| root.state.clone())
                };
                if let Some(last) = intent.boundary {
                    if let Some(root) = root {
                        root.boundary(&scope, last)?;
                    }
                } else {
                    composite::focus_node(&scope, Some(item.node))?;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        let cleanup = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(state) = cleanup.upgrade() {
                let roots = std::mem::take(&mut *state.roots.borrow_mut());
                drop(roots);
            }
        })?;
        bar.scope().retain(contents)?;
        scope.retain(bar)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests;
