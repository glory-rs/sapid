//! Finite mounted menu trees over intrinsic controls and host-owned top layers.
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::future::Future;
use std::rc::{Rc, Weak};

use glory::render::{
    Action, Change, CheckMode, Direction, Event, EventKind, FocusMode, Layer, LayerPlacement,
    MenuCommand, MenuItemKind, MenuItemSemantics, Navigation, NodeId, Orientation, Patch,
    PointerPhase, PopupKind, Role, SemanticId, Semantics,
};
use glory::style::{Edges, Length};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};

use crate::appearance::{Common, common_methods, configure, observe};
use crate::collection::{
    Collection, CollectionLimits, CollectionMember, RovingFocus, Typeahead, TypeaheadConfig,
};
use crate::composite::{self, Callback, Observers};
use crate::state::{Control, DelayFactory, Dirty, OwnerDelay, Value};

pub const MAX_MENU_ITEMS: usize = 1024;
pub const MAX_MENU_LEVEL_ITEMS: usize = 256;
pub const MAX_MENU_DEPTH: usize = 8;
type Delay = DelayFactory;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ItemKind {
    Action,
    Checkbox,
}
pub struct MenuItem {
    common: Common,
    key: String,
    label: Lotus<String>,
    kind: ItemKind,
    checked: Value<bool>,
    changed: Option<Box<dyn FnMut(bool)>>,
    activated: Option<Box<dyn FnMut()>>,
    close: bool,
    check_configured: bool,
}
pub fn menu_item(key: impl Into<String>) -> MenuItem {
    MenuItem {
        common: Common::default(),
        key: key.into(),
        label: "".into(),
        kind: ItemKind::Action,
        checked: Value::default(),
        changed: None,
        activated: None,
        close: true,
        check_configured: false,
    }
}
pub fn menu_checkbox(key: impl Into<String>) -> MenuItem {
    MenuItem {
        kind: ItemKind::Checkbox,
        close: false,
        ..menu_item(key)
    }
}
impl MenuItem {
    common_methods!();
    pub fn label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.label = value.into();
        self
    }
    pub fn on_activate(mut self, callback: impl FnMut() + 'static) -> Self {
        self.activated = Some(Box::new(callback));
        self
    }
    pub fn close_on_select(mut self, value: bool) -> Self {
        self.close = value;
        self
    }
    pub fn checked(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.checked.source = Some(value.into());
        self.check_configured = true;
        self
    }
    pub fn default_checked(mut self, value: bool) -> Self {
        self.checked.default = value;
        self.check_configured = true;
        self
    }
    pub fn on_checked_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.changed = Some(Box::new(callback));
        self.check_configured = true;
        self
    }
}
pub struct MenuRadioItem {
    common: Common,
    key: String,
    label: Lotus<String>,
}
pub fn menu_radio_item(
    key: impl Into<String>,
    label: impl Into<Lotus<String>>,
) -> MenuRadioItem {
    MenuRadioItem {
        common: Common::default(),
        key: key.into(),
        label: label.into(),
    }
}
impl MenuRadioItem {
    common_methods!();
}
pub struct MenuRadioGroup {
    key: String,
    label: Lotus<String>,
    value: Value<String>,
    changed: Option<Box<dyn FnMut(String)>>,
    items: Vec<MenuRadioItem>,
    close: bool,
}
pub fn menu_radio_group(key: impl Into<String>) -> MenuRadioGroup {
    MenuRadioGroup {
        key: key.into(),
        label: "".into(),
        value: Value::default(),
        changed: None,
        items: Vec::new(),
        close: false,
    }
}
impl MenuRadioGroup {
    pub fn label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.label = value.into();
        self
    }
    pub fn value(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.value.default = value.into();
        self
    }
    pub fn on_value_change(mut self, callback: impl FnMut(String) + 'static) -> Self {
        self.changed = Some(Box::new(callback));
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = MenuRadioItem>) -> Self {
        let remaining = (MAX_MENU_LEVEL_ITEMS + 1).saturating_sub(self.items.len());
        self.items.extend(items.into_iter().take(remaining));
        self
    }
    pub fn close_on_select(mut self, value: bool) -> Self {
        self.close = value;
        self
    }
}
pub struct MenuSubmenu {
    common: Common,
    key: String,
    label: Lotus<String>,
    open: Value<bool>,
    changed: Option<Box<dyn FnMut(bool)>>,
    entries: Vec<MenuEntry>,
    overflow: bool,
}
pub fn menu_submenu(key: impl Into<String>) -> MenuSubmenu {
    MenuSubmenu {
        common: Common::default(),
        key: key.into(),
        label: "".into(),
        open: Value::default(),
        changed: None,
        entries: Vec::new(),
        overflow: false,
    }
}
impl MenuSubmenu {
    common_methods!();
    pub fn label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.label = value.into();
        self
    }
    pub fn open(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.open.source = Some(value.into());
        self
    }
    pub fn default_open(mut self, value: bool) -> Self {
        self.open.default = value;
        self
    }
    pub fn on_open_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.changed = Some(Box::new(callback));
        self
    }
    pub fn item(mut self, item: impl Into<MenuEntry>) -> Self {
        push_entry(&mut self.entries, &mut self.overflow, item.into());
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        if self.overflow {
            return self;
        }
        for item in items.into_iter().take(MAX_MENU_LEVEL_ITEMS + 1) {
            push_entry(&mut self.entries, &mut self.overflow, MenuEntry::Item(item));
            if self.overflow {
                break;
            }
        }
        self
    }
    pub fn radio_group(self, group: MenuRadioGroup) -> Self {
        self.item(group)
    }
    pub fn submenu(self, submenu: MenuSubmenu) -> Self {
        self.item(submenu)
    }
    pub fn separator(mut self, key: impl Into<String>) -> Self {
        push_entry(
            &mut self.entries,
            &mut self.overflow,
            MenuEntry::Separator(key.into()),
        );
        self
    }
}
fn entry_size(entry: &MenuEntry) -> usize {
    match entry {
        MenuEntry::RadioGroup(group) => group.items.len(),
        MenuEntry::Submenu(menu) => 1 + menu.entries.iter().map(entry_size).sum::<usize>(),
        _ => 1,
    }
}
fn entry_width(entry: &MenuEntry) -> usize {
    match entry {
        MenuEntry::RadioGroup(group) => group.items.len(),
        _ => 1,
    }
}
fn entry_depth(entry: &MenuEntry) -> usize {
    match entry {
        MenuEntry::Submenu(menu) => 1 + menu.entries.iter().map(entry_depth).max().unwrap_or(0),
        _ => 0,
    }
}
fn push_entry(entries: &mut Vec<MenuEntry>, overflow: &mut bool, entry: MenuEntry) {
    if *overflow {
        return;
    }
    let total = entries.iter().map(entry_size).sum::<usize>() + entry_size(&entry);
    let width = entries.iter().map(entry_width).sum::<usize>() + entry_width(&entry);
    if total > MAX_MENU_ITEMS
        || width > MAX_MENU_LEVEL_ITEMS
        || entry_depth(&entry) >= MAX_MENU_DEPTH
        || matches!(&entry, MenuEntry::Submenu(menu) if menu.overflow)
    {
        *overflow = true;
        return;
    }
    entries.push(entry);
}
pub enum MenuEntry {
    Item(MenuItem),
    RadioGroup(MenuRadioGroup),
    Submenu(MenuSubmenu),
    Separator(String),
}
impl From<MenuItem> for MenuEntry {
    fn from(item: MenuItem) -> Self {
        Self::Item(item)
    }
}
impl From<MenuRadioGroup> for MenuEntry {
    fn from(item: MenuRadioGroup) -> Self {
        Self::RadioGroup(item)
    }
}
impl From<MenuSubmenu> for MenuEntry {
    fn from(item: MenuSubmenu) -> Self {
        Self::Submenu(item)
    }
}

pub struct Menu {
    common: Common,
    key: String,
    label: Lotus<String>,
    open: Value<bool>,
    changed: Option<Box<dyn FnMut(bool)>>,
    entries: Vec<MenuEntry>,
    overflow: bool,
    anchor: Option<SemanticId>,
    return_focus: Option<SemanticId>,
    direction: Direction,
    looping: bool,
    typeahead: TypeaheadConfig,
    delay: Option<Delay>,
    trigger: Option<Cage<Option<NodeId>>>,
    dropdown: bool,
}
pub fn menu(key: impl Into<String>) -> Menu {
    Menu {
        common: Common::default(),
        key: key.into(),
        label: "Menu".into(),
        open: Value::default(),
        changed: None,
        entries: Vec::new(),
        overflow: false,
        anchor: None,
        return_focus: None,
        direction: Direction::Ltr,
        looping: true,
        typeahead: TypeaheadConfig::default(),
        delay: None,
        trigger: None,
        dropdown: false,
    }
}
pub struct DropdownMenu {
    inner: Menu,
}
pub fn dropdown_menu(key: impl Into<String>) -> DropdownMenu {
    DropdownMenu {
        inner: Menu {
            dropdown: true,
            ..menu(key)
        },
    }
}
impl Menu {
    common_methods!();
    pub fn label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.label = value.into();
        self
    }
    pub fn open(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.open.source = Some(value.into());
        self
    }
    pub fn default_open(mut self, value: bool) -> Self {
        self.open.default = value;
        self
    }
    pub fn on_open_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.changed = Some(Box::new(callback));
        self
    }
    pub fn item(mut self, item: impl Into<MenuEntry>) -> Self {
        push_entry(&mut self.entries, &mut self.overflow, item.into());
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        if self.overflow {
            return self;
        }
        for item in items.into_iter().take(MAX_MENU_LEVEL_ITEMS + 1) {
            push_entry(&mut self.entries, &mut self.overflow, MenuEntry::Item(item));
            if self.overflow {
                break;
            }
        }
        self
    }
    pub fn radio_group(self, group: MenuRadioGroup) -> Self {
        self.item(group)
    }
    pub fn submenu(self, submenu: MenuSubmenu) -> Self {
        self.item(submenu)
    }
    pub fn separator(mut self, key: impl Into<String>) -> Self {
        push_entry(
            &mut self.entries,
            &mut self.overflow,
            MenuEntry::Separator(key.into()),
        );
        self
    }
    pub fn anchor(mut self, id: SemanticId) -> Self {
        self.anchor = Some(id);
        self
    }
    pub fn return_focus(mut self, id: SemanticId) -> Self {
        self.return_focus = Some(id);
        self
    }
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
    pub fn loop_focus(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }
    pub fn typeahead(mut self, config: TypeaheadConfig) -> Self {
        self.typeahead = config;
        self
    }
    /// Optional application/host delay. One pending task per level is cancelled
    /// on leave, sibling entry, close and Owner retirement. No global timer.
    pub fn hover_delay_with<F: Fn() -> Fut + 'static, Fut: Future<Output = ()> + 'static>(
        mut self,
        factory: F,
    ) -> Self {
        self.delay = Some(Rc::new(move || Box::pin(factory())));
        self
    }
    pub fn trigger_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
        self.trigger = Some(reference);
        self
    }
}
macro_rules! dropdown_methods {
    ($( $name:ident ( $arg:ident : $ty:ty ) );* $(;)?) => { $(pub fn $name(mut self, $arg: $ty) -> Self { self.inner = self.inner.$name($arg); self })* };
}
impl DropdownMenu {
    dropdown_methods! {
        label(value: impl Into<Lotus<String>>); open(value: impl Into<Lotus<bool>>); default_open(value: bool);
        on_open_change(callback: impl FnMut(bool) + 'static); item(item: impl Into<MenuEntry>); items(items: impl IntoIterator<Item = MenuItem>);
        radio_group(group: MenuRadioGroup); submenu(submenu: MenuSubmenu); separator(key: impl Into<String>);
        return_focus(id: SemanticId); direction(direction: Direction); loop_focus(looping: bool); typeahead(config: TypeaheadConfig);
        trigger_ref(reference: Cage<Option<NodeId>>); node_ref(reference: Cage<Option<NodeId>>); class(value: impl Into<Lotus<String>>);
        name(value: impl Into<Lotus<String>>); style(value: impl Into<Lotus<glory::style::Style>>); enabled(value: impl Into<Lotus<bool>>);
        theme(value: crate::Theme); size(value: crate::Size)
    }
    pub fn unstyled(mut self) -> Self {
        self.inner = self.inner.unstyled();
        self
    }
    pub fn hover_delay_with<F: Fn() -> Fut + 'static, Fut: Future<Output = ()> + 'static>(
        mut self,
        factory: F,
    ) -> Self {
        self.inner = self.inner.hover_delay_with(factory);
        self
    }
    pub fn on(mut self, kind: EventKind, callback: impl FnMut(Event) + 'static) -> Self {
        self.inner = self.inner.on(kind, callback);
        self
    }
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn id(namespace: &str, path: &[String], part: &str) -> Result<SemanticId> {
    let mut value = format!("sapid-menu-{}_{namespace}-{part}", namespace.len());
    for key in path {
        value.push_str(&format!("-{}_{key}", key.len()));
    }
    Ok(SemanticId::new(value)?)
}
fn key(value: &str) -> Result {
    SemanticId::new(value)?;
    if value.len() > 64 {
        return Err(invalid("menu keys exceed 64 bytes"));
    }
    Ok(())
}
fn validate(
    namespace: &str,
    entries: &[MenuEntry],
    path: &mut Vec<String>,
    total: &mut usize,
) -> Result {
    if path.len() >= MAX_MENU_DEPTH || entries.is_empty() {
        return Err(invalid(
            "menu requires nonempty levels within its depth budget",
        ));
    }
    let mut seen = HashSet::new();
    let mut count = 0;
    let mut open_submenu = false;
    for entry in entries {
        let (name, members) = match entry {
            MenuEntry::Item(item) => (&item.key, 1),
            MenuEntry::RadioGroup(group) => (&group.key, group.items.len()),
            MenuEntry::Submenu(menu) => (&menu.key, 1),
            MenuEntry::Separator(key) => (key, 1),
        };
        key(name)?;
        if !seen.insert(name) {
            return Err(invalid("duplicate menu level key"));
        }
        path.push(name.clone());
        id(namespace, path, "item")?;
        id(namespace, path, "panel")?;
        count += members;
        *total += members;
        if count > MAX_MENU_LEVEL_ITEMS || *total > MAX_MENU_ITEMS {
            return Err(invalid("menu mounted item budget exhausted"));
        }
        match entry {
            MenuEntry::Item(item)
                if item.common.focus != FocusMode::Auto
                    || item.kind == ItemKind::Action && item.check_configured =>
            {
                return Err(invalid(
                    "menu owns item focus and checkbox state requires a checkbox item",
                ));
            }
            MenuEntry::Submenu(menu) => {
                let open = match &menu.open.source {
                    Some(source) => source.try_get()?,
                    None => menu.open.default,
                };
                if open && open_submenu {
                    return Err(invalid(
                        "only one submenu at each menu level may initially be open",
                    ));
                }
                open_submenu |= open;
                if menu.overflow {
                    return Err(invalid("submenu configuration budget exhausted"));
                }
                if menu.common.focus != FocusMode::Auto {
                    return Err(invalid("menu owns submenu trigger focus"));
                }
                validate(namespace, &menu.entries, path, total)?;
            }
            MenuEntry::RadioGroup(group) => {
                if group.items.is_empty() {
                    return Err(invalid("menu radio group is empty"));
                }
                let mut radios = HashSet::new();
                for item in &group.items {
                    key(&item.key)?;
                    if !radios.insert(&item.key)
                        || !seen.insert(&item.key)
                        || item.common.focus != FocusMode::Auto
                    {
                        return Err(invalid(
                            "menu radio keys are duplicate or focus is overridden",
                        ));
                    }
                }
            }
            _ => {}
        }
        path.pop();
    }
    Ok(())
}

struct Child {
    trigger: NodeId,
    state: Rc<MenuState>,
}
struct MenuState {
    open: Control<bool>,
    changed: Callback<bool>,
    visible: Lotus<bool>,
    enabled: Lotus<bool>,
    collection: Collection,
    roving: RovingFocus,
    search: RefCell<Typeahead>,
    active_child: Cage<Option<NodeId>>,
    children: RefCell<Vec<Child>>,
    parent: Weak<MenuState>,
    root: Weak<MenuState>,
    panel: Cage<Option<NodeId>>,
    trigger: Cage<Option<NodeId>>,
    owner: glory::core::InstanceId,
    pending: Rc<OwnerDelay>,
    hovered: Cell<Option<NodeId>>,
    delay: Option<Delay>,
}
impl MenuState {
    fn cancel_pending(&self) {
        self.pending.cancel();
        self.hovered.set(None);
    }
    fn request(&self, open: bool) -> Result {
        if !self.owner.is_alive() {
            return Ok(());
        }
        if !open {
            self.cancel_pending();
            self.close_children(None)?;
        }
        if !self.owner.is_alive() {
            return Ok(());
        }
        if self.open.source.try_get()? != open {
            self.open.accept(&open)?;
            self.changed.emit(self.owner, open);
        }
        Ok(())
    }
    fn children(&self) -> Vec<(NodeId, Rc<MenuState>)> {
        self.children
            .borrow()
            .iter()
            .map(|child| (child.trigger, child.state.clone()))
            .collect()
    }
    fn close_children(&self, keep: Option<NodeId>) -> Result {
        for (trigger, child) in self.children() {
            if Some(trigger) != keep {
                child.request(false)?;
                if !self.owner.is_alive() {
                    break;
                }
            }
        }
        Ok(())
    }
    fn root_close(&self) -> Result {
        if let Some(root) = self.root.upgrade() {
            root.request(false)?;
        }
        Ok(())
    }
    fn focus_item(&self, scope: &Scope, item: crate::MountedItem) -> Result {
        self.roving.accept(&item)?;
        composite::focus_node(scope, Some(item.node))
    }
    fn boundary(&self, scope: &Scope, last: bool) -> Result {
        let items = self.collection.mounted()?;
        let item = if last {
            items.into_iter().rev().find(|item| item.enabled)
        } else {
            items.into_iter().find(|item| item.enabled)
        };
        if let Some(item) = item {
            self.focus_item(scope, item)?;
        }
        Ok(())
    }
    fn open_child(&self, scope: &Scope, from: NodeId, focus: bool) -> Result {
        self.close_children(Some(from))?;
        if !self.owner.is_alive() {
            return Ok(());
        }
        let children = self.children();
        if children
            .iter()
            .any(|(trigger, child)| *trigger != from && child.open.source.get())
        {
            return Ok(());
        }
        if let Some((_, child)) = children.into_iter().find(|(trigger, _)| *trigger == from) {
            child.request(true)?;
            if focus && child.owner.is_alive() {
                child.boundary(scope, false)?;
            }
        }
        Ok(())
    }
    fn navigate(&self, scope: &Scope, from: NodeId, command: Navigation) -> Result {
        if !self.visible.try_get()? {
            return Ok(());
        }
        self.cancel_pending();
        self.search.borrow_mut().clear();
        if let Some(item) = self.roving.destination(from, command)? {
            self.close_children(None)?;
            if self.owner.is_alive() {
                self.focus_item(scope, item)?;
            }
        }
        Ok(())
    }
    fn command(&self, scope: &Scope, from: NodeId, command: MenuCommand) -> Result {
        self.cancel_pending();
        match command {
            MenuCommand::First | MenuCommand::Last => {
                self.request(true)?;
                if self.owner.is_alive() {
                    self.boundary(scope, command == MenuCommand::Last)?;
                }
            }
            MenuCommand::Open if self.visible.try_get()? => {
                self.open_child(scope, from, true)?
            }
            MenuCommand::Close if self.visible.try_get()? => {
                self.request(false)?;
                if self.parent.upgrade().is_some() && self.owner.is_alive() {
                    composite::focus(scope, self.trigger)?;
                }
            }
            MenuCommand::Exit { backwards } => {
                if let Some(root) = self.root.upgrade() {
                    root.request(false)?;
                    if root.owner.is_alive()
                        && scope.capabilities()?.focus
                        && let Some(node) = root.panel.try_get()?
                    {
                        scope.action(Action::ExitMenu { node, backwards })?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn typeahead(&self, scope: &Scope, from: NodeId, text: &str, at_millis: u64) -> Result {
        if !self.visible.try_get()? {
            return Ok(());
        }
        self.cancel_pending();
        let item = self
            .search
            .borrow_mut()
            .search(&self.collection, from, text, at_millis)?;
        if let Some(item) = item {
            self.close_children(None)?;
            if self.owner.is_alive() {
                self.focus_item(scope, item)?;
            }
        }
        Ok(())
    }
    fn pointer(self: &Rc<Self>, scope: &Scope, from: NodeId, phase: PointerPhase) -> Result {
        if phase == PointerPhase::Enter {
            return Ok(());
        }
        if phase == PointerPhase::Move && self.hovered.get() == Some(from) {
            return Ok(());
        }
        self.cancel_pending();
        if phase != PointerPhase::Move || !self.visible.try_get()? {
            return Ok(());
        }
        self.hovered.set(Some(from));
        let Some(item) = self
            .collection
            .mounted()?
            .into_iter()
            .find(|item| item.node == from && item.enabled)
        else {
            return Ok(());
        };
        self.close_children(Some(from))?;
        if !self.owner.is_alive() {
            return Ok(());
        }
        self.focus_item(scope, item)?;
        if !self
            .children
            .borrow()
            .iter()
            .any(|child| child.trigger == from)
        {
            return Ok(());
        }
        let state = Rc::downgrade(self);
        let weak = scope.downgrade();
        self.pending.run(scope, self.delay.as_ref(), move || {
            if let (Some(state), Some(scope)) = (state.upgrade(), weak.upgrade())
                && state.visible.try_get()?
            {
                state.open_child(&scope, from, false)?;
            }
            Ok(())
        })
    }
}

struct Level {
    key: String,
    path: Vec<String>,
    common: Common,
    label: Lotus<String>,
    open: Value<bool>,
    changed: Option<Box<dyn FnMut(bool)>>,
    entries: Vec<MenuEntry>,
    anchor: SemanticId,
    return_focus: SemanticId,
    placement: LayerPlacement,
    parent: Weak<MenuState>,
    root: Weak<MenuState>,
    parent_visible: Lotus<bool>,
    root_enabled: Lotus<bool>,
    direction: Direction,
    looping: bool,
    typeahead: TypeaheadConfig,
    delay: Option<Delay>,
    trigger: Cage<Option<NodeId>>,
}
impl Level {
    fn build(mut self, scope: &Scope) -> Result<Rc<MenuState>> {
        let panel_id = id(&self.key, &self.path, "panel")?;
        let panel = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let control = self.open.mount(scope);
        let source = control.source.clone();
        let parent = self.parent_visible;
        let visible: Lotus<bool> = scope.bond(move || source.get() && parent.get()).into();
        let root_enabled = self.root_enabled;
        let own_enabled = self.common.enabled.clone();
        let enabled: Lotus<bool> = scope
            .bond(move || root_enabled.get() && own_enabled.get())
            .into();
        let collection = Collection::new(
            scope,
            CollectionLimits {
                items: MAX_MENU_LEVEL_ITEMS,
                ..Default::default()
            },
        )?;
        let roving = RovingFocus::new(scope, collection.clone(), self.looping)?;
        let search = Typeahead::new(self.typeahead)?;
        let pending = OwnerDelay::new(scope)?;
        let state = Rc::new_cyclic(|this| MenuState {
            open: control,
            changed: Callback::new(self.changed.take()),
            visible: visible.clone(),
            enabled: enabled.clone(),
            collection,
            roving,
            search: RefCell::new(search),
            active_child: scope.cage(None),
            children: RefCell::new(Vec::new()),
            parent: self.parent.clone(),
            root: if self.root.strong_count() == 0 {
                this.clone()
            } else {
                self.root.clone()
            },
            panel,
            trigger: self.trigger,
            owner: scope.id(),
            pending,
            hovered: Cell::new(None),
            delay: self.delay.clone(),
        });
        let cleanup = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(state) = cleanup.upgrade() {
                state.cancel_pending();
            }
        })?;
        let open = visible.clone();
        let direction = self.direction;
        let config = Layer {
            modal: false,
            escape: true,
            outside: true,
            focus: glory::render::LayerFocus::Manage,
            placement: self.placement,
            anchor: Some(self.anchor),
            initial_focus: None,
            return_focus: Some(self.return_focus),
        };
        let stable_id = panel_id.clone();
        let mut panel_widget = configure!(
            self.common,
            glory::ui::view().role(Role::Menu),
            scope,
            "sapid-menu sapid-layer",
            menu_style(&self.common)
        )
        .name(self.common.name.clone().unwrap_or(self.label.clone()))
        .node_ref(panel)
        .focus(FocusMode::Programmatic)
        .semantics(scope.bond(move || {
            composite::metadata(Semantics {
                id: Some(stable_id.clone()),
                layer: Some(config.clone()),
                hidden: !open.get(),
                navigation: Some(Orientation::Vertical),
                navigation_direction: direction,
                typeahead: true,
                ..Default::default()
            })
        }));
        for kind in [
            EventKind::Navigate,
            EventKind::Menu,
            EventKind::Typeahead,
            EventKind::Dismiss,
        ] {
            let live = state.clone();
            let weak = scope.downgrade();
            let owner = scope.id();
            let mut observers = self.common.handlers.remove(&kind).unwrap_or_default();
            panel_widget = panel_widget.on(kind, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                let result = match &event {
                    Event::Navigate { from, command } => live.navigate(&scope, *from, *command),
                    Event::Menu { from, command } => live.command(&scope, *from, *command),
                    Event::Typeahead {
                        from,
                        text,
                        at_millis,
                    } => live.typeahead(&scope, *from, text, *at_millis),
                    Event::Dismiss(_) => live.request(false),
                    _ => Ok(()),
                };
                if let Err(error) = result {
                    scope.report_error(error);
                    return;
                }
                observe(&mut observers, owner, event);
            });
        }
        for (kind, mut observers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            panel_widget =
                panel_widget.on(kind, move |event| observe(&mut observers, owner, event));
        }
        panel_widget.show_in(scope)?;
        let panel_node = panel.get().ok_or(glory::ui::Error::Gone)?;
        let content = scope.child_in(panel_node)?;
        for entry in self.entries {
            match entry {
                MenuEntry::Item(item) => build_item(
                    &content,
                    &state,
                    &self.common,
                    &self.key,
                    &self.path,
                    &panel_id,
                    item,
                )?,
                MenuEntry::Separator(key) => {
                    let mut path = self.path.clone();
                    path.push(key);
                    glory::ui::view()
                        .role(Role::Separator)
                        .class(if self.common.unstyled {
                            "sapid-menu-separator sapid-unstyled"
                        } else {
                            "sapid-menu-separator"
                        })
                        .semantics(composite::metadata(Semantics {
                            id: Some(id(&self.key, &path, "item")?),
                            ..Default::default()
                        }))
                        .show_in(&content)?;
                }
                MenuEntry::RadioGroup(group) => build_radios(
                    &content,
                    &state,
                    &self.common,
                    &self.key,
                    &self.path,
                    &panel_id,
                    group,
                )?,
                MenuEntry::Submenu(submenu) => {
                    let mut path = self.path.clone();
                    path.push(submenu.key.clone());
                    let trigger_id = id(&self.key, &path, "item")?;
                    let trigger = submenu
                        .common
                        .node_ref
                        .unwrap_or_else(|| content.cage(None));
                    let child_owner = content.child()?;
                    let child_label = submenu.label.clone();
                    let own_enabled = submenu.common.enabled.clone();
                    let parent_visible = visible.clone();
                    let active = state.active_child;
                    let child_visible: Lotus<bool> = content
                        .bond(move || {
                            parent_visible.get()
                                && trigger.get().is_some_and(|node| active.get() == Some(node))
                        })
                        .into();
                    let child = Level {
                        key: self.key.clone(),
                        path: path.clone(),
                        common: Common {
                            theme: self.common.theme.clone(),
                            size: self.common.size,
                            unstyled: self.common.unstyled || submenu.common.unstyled,
                            ..Default::default()
                        },
                        label: child_label.clone(),
                        open: submenu.open,
                        changed: submenu.changed,
                        entries: submenu.entries,
                        anchor: trigger_id.clone(),
                        return_focus: trigger_id.clone(),
                        placement: if direction == Direction::Rtl {
                            LayerPlacement::AnchorLeft
                        } else {
                            LayerPlacement::AnchorRight
                        },
                        parent: Rc::downgrade(&state),
                        root: state.root.clone(),
                        parent_visible: child_visible,
                        root_enabled: enabled.clone(),
                        direction,
                        looping: self.looping,
                        typeahead: self.typeahead,
                        delay: self.delay.clone(),
                        trigger,
                    }
                    .build(&child_owner)?;
                    content.retain(child_owner)?;
                    let child_panel = id(&self.key, &path, "panel")?;
                    let expanded = child.visible.clone();
                    let native = glory::ui::button()
                        .label(child_label.clone())
                        .node_ref(trigger);
                    let callback = child.clone();
                    let live = state.clone();
                    let weak = content.downgrade();
                    let mut common = submenu.common;
                    common.theme = self.common.theme.clone();
                    common.size = self.common.size;
                    common.unstyled |= self.common.unstyled;
                    let mut observers = common
                        .handlers
                        .remove(&EventKind::Activate)
                        .unwrap_or_default();
                    let instance = scope.id();
                    let native = configure!(
                        common,
                        native,
                        &content,
                        "sapid-menu-item sapid-menu-submenu-trigger",
                        menu_item_style(&self.common)
                    );
                    let item = mount_member(
                        &content,
                        &state,
                        native,
                        &self.common,
                        &mut common,
                        &submenu.key,
                        child_label,
                        trigger,
                        own_enabled,
                        Semantics {
                            id: Some(trigger_id),
                            menu_item: Some(MenuItemSemantics {
                                kind: MenuItemKind::Action,
                                owner: panel_id.clone(),
                            }),
                            popup: Some(PopupKind::Menu),
                            controls: Some(child_panel),
                            ..Default::default()
                        },
                        Some(expanded),
                        move |event| {
                            let Some(scope) = weak.upgrade() else { return };
                            if !live.visible.get() {
                                return;
                            }
                            let result = if callback.open.source.get() {
                                callback.request(false)
                            } else {
                                live.open_child(
                                    &scope,
                                    trigger.get().unwrap_or(panel_node),
                                    true,
                                )
                            };
                            if let Err(error) = result {
                                scope.report_error(error);
                                return;
                            }
                            observe(&mut observers, instance, event);
                        },
                    )?;
                    state.children.borrow_mut().push(Child {
                        trigger: item,
                        state: child,
                    });
                }
            }
        }
        scope.retain(content)?;
        state.roving.repair(None)?;
        let live = state.clone();
        let weak = scope.downgrade();
        scope.structural(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let mut accepted = None;
                for (trigger, child) in live.children() {
                    if child.open.source.try_get()? {
                        if accepted.is_some() {
                            return Err(invalid(
                                "only one submenu at each menu level may be open",
                            ));
                        }
                        accepted = Some(trigger);
                    }
                }
                if live.active_child.try_get()? != accepted {
                    live.active_child.try_revise(|active| *active = accepted)?;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        let live = state.clone();
        let weak = scope.downgrade();
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let visible = live.visible.try_get()?;
                live.roving.repair(None)?;
                if !visible {
                    live.cancel_pending();
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        Ok(state)
    }
}
fn menu_item_style(common: &Common) -> glory::style::Style {
    common
        .theme
        .control(common.size)
        .border_width(Edges::all(0.0))
        .min_height(Length::Px(32.0))
        .padding(Edges {
            top: 5.0,
            bottom: 5.0,
            left: 9.0,
            right: 9.0,
        })
}
fn menu_style(common: &Common) -> glory::style::Style {
    let mut style = composite::column(2.0)
        .width(Length::Px(220.0))
        .padding(Edges::all(5.0))
        .radius(common.theme.radius);
    style.background = Some(common.theme.surface);
    style.color = Some(common.theme.foreground);
    style
}

/// Construct the intrinsic control in its own mounted Owner, then register it.
/// Both Button and Check expose the same typed member contract.
#[allow(clippy::too_many_arguments)]
fn mount_member<W: MenuControl>(
    scope: &Scope,
    state: &Rc<MenuState>,
    control: W,
    parent: &Common,
    common: &mut Common,
    key: &str,
    label: Lotus<String>,
    reference: Cage<Option<NodeId>>,
    own_enabled: Lotus<bool>,
    mut semantics: Semantics,
    expanded: Option<Lotus<bool>>,
    activation: impl FnMut(Event) + 'static,
) -> Result<NodeId> {
    common.theme = parent.theme.clone();
    common.size = parent.size;
    common.unstyled |= parent.unstyled;
    let owner = scope.child()?;
    let token = owner.cage(None::<CollectionMember>);
    let roving = state.roving.clone();
    let focus = owner.bond(move || {
        if token
            .get()
            .is_some_and(|member| roving.is_tab_stop(&member).unwrap_or(false))
        {
            FocusMode::TabStop
        } else {
            FocusMode::Programmatic
        }
    });
    let root = state.enabled.clone();
    let flag = own_enabled;
    let enabled: Lotus<bool> = owner.bond(move || root.get() && flag.get()).into();
    semantics.navigation_item = true;
    let metadata = owner.bond(move || {
        let mut metadata = semantics.clone();
        if let Some(expanded) = &expanded {
            metadata.expanded = Some(expanded.get());
        }
        composite::metadata(metadata)
    });
    let control = control.member(focus.into(), enabled.clone(), metadata.into(), activation);
    let mounted = control.mount(&owner)?;
    let member = state
        .collection
        .register(mounted.scope(), key, reference, enabled, label)?;
    token.revise(|token| *token = Some(member.clone()));
    let node = reference.get().ok_or(glory::ui::Error::Gone)?;
    let live = state.clone();
    let weak = owner.downgrade();
    let mut pointer_observers = common
        .handlers
        .remove(&EventKind::Pointer)
        .unwrap_or_default();
    let instance = owner.id();
    owner.on(node, EventKind::Pointer, move |event| {
        let Some(scope) = weak.upgrade() else { return };
        if let Event::Pointer { from, phase, .. } = &event
            && let Err(error) = live.pointer(&scope, *from, *phase)
        {
            scope.report_error(error);
            return;
        }
        observe(&mut pointer_observers, instance, event);
    })?;
    let roving = state.roving.clone();
    let weak = owner.downgrade();
    let mut focus_observers = common
        .handlers
        .remove(&EventKind::FocusChanged)
        .unwrap_or_default();
    owner.on(node, EventKind::FocusChanged, move |event| {
        let Some(scope) = weak.upgrade() else { return };
        if event == Event::FocusChanged(true)
            && let Err(error) = roving.observe(&member)
        {
            scope.report_error(error);
            return;
        }
        observe(&mut focus_observers, instance, event);
    })?;
    for (kind, mut observers) in std::mem::take(&mut common.handlers) {
        let live = state.clone();
        let weak = owner.downgrade();
        owner.on(node, kind, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            let result = match &event {
                Event::Navigate { from, command } => live.navigate(&scope, *from, *command),
                Event::Menu { from, command } => live.command(&scope, *from, *command),
                Event::Typeahead {
                    from,
                    text,
                    at_millis,
                } => live.typeahead(&scope, *from, text, *at_millis),
                _ => Ok(()),
            };
            if let Err(error) = result {
                scope.report_error(error);
                return;
            }
            observe(&mut observers, instance, event);
        })?;
    }
    owner.retain(mounted)?;
    scope.retain(owner)?;
    Ok(node)
}
trait MenuControl: Widget + Sized {
    fn member(
        self,
        focus: Lotus<FocusMode>,
        enabled: Lotus<bool>,
        semantics: Lotus<Option<Box<Semantics>>>,
        activation: impl FnMut(Event) + 'static,
    ) -> Self;
}
impl MenuControl for glory::ui::Button {
    fn member(
        self,
        focus: Lotus<FocusMode>,
        enabled: Lotus<bool>,
        semantics: Lotus<Option<Box<Semantics>>>,
        activation: impl FnMut(Event) + 'static,
    ) -> Self {
        self.focus(focus)
            .enabled(enabled)
            .semantics(semantics)
            .on(EventKind::Activate, activation)
    }
}
impl MenuControl for glory::ui::Check {
    fn member(
        self,
        focus: Lotus<FocusMode>,
        enabled: Lotus<bool>,
        semantics: Lotus<Option<Box<Semantics>>>,
        activation: impl FnMut(Event) + 'static,
    ) -> Self {
        self.focus(focus)
            .enabled(enabled)
            .semantics(semantics)
            .on(EventKind::CheckedChanged, activation)
    }
}

fn build_item(
    scope: &Scope,
    state: &Rc<MenuState>,
    parent: &Common,
    namespace: &str,
    path: &[String],
    menu_id: &SemanticId,
    mut item: MenuItem,
) -> Result {
    let owner = scope.child()?;
    let reference = item.common.node_ref.unwrap_or_else(|| owner.cage(None));
    let enabled = item.common.enabled.clone();
    item.common.theme = parent.theme.clone();
    item.common.size = parent.size;
    item.common.unstyled |= parent.unstyled;
    let mut path = path.to_vec();
    path.push(item.key.clone());
    let semantics = Semantics {
        id: Some(id(namespace, &path, "item")?),
        menu_item: Some(MenuItemSemantics {
            kind: if item.kind == ItemKind::Action {
                MenuItemKind::Action
            } else {
                MenuItemKind::Checkbox
            },
            owner: menu_id.clone(),
        }),
        ..Default::default()
    };
    let callback = Callback::new(
        item.activated
            .take()
            .map(|mut callback| Box::new(move |()| callback()) as Box<dyn FnMut(())>),
    );
    let live = state.clone();
    let close = item.close;
    let instance = owner.id();
    let weak = owner.downgrade();
    if item.kind == ItemKind::Action {
        let native = configure!(
            item.common,
            glory::ui::button()
                .label(item.label.clone())
                .node_ref(reference),
            &owner,
            "sapid-menu-item",
            menu_item_style(parent)
        );
        let mut observers = item
            .common
            .handlers
            .remove(&EventKind::Activate)
            .unwrap_or_default();
        mount_member(
            &owner,
            state,
            native,
            parent,
            &mut item.common,
            &item.key,
            item.label,
            reference,
            enabled,
            semantics,
            None,
            move |event| {
                let Some(scope) = weak.upgrade() else { return };
                if !live.visible.get() {
                    return;
                }
                callback.emit(instance, ());
                if instance.is_alive()
                    && close
                    && let Err(error) = live.root_close()
                {
                    scope.report_error(error);
                    return;
                }
                observe(&mut observers, instance, event);
            },
        )?;
    } else {
        let control = item.checked.mount(&owner);
        let initial = control.source.try_get()?;
        let dirty = Dirty::new(&owner);
        let changed = Callback::new(item.changed.take());
        let accepted = control.clone();
        let pulse = dirty.clone();
        let native = configure!(
            item.common,
            glory::ui::check()
                .label(item.label.clone())
                .checked(initial)
                .node_ref(reference),
            &owner,
            "sapid-menu-item sapid-menu-check",
            menu_item_style(parent)
        );
        let mut observers = item
            .common
            .handlers
            .remove(&EventKind::CheckedChanged)
            .unwrap_or_default();
        let node = mount_member(
            &owner,
            state,
            native,
            parent,
            &mut item.common,
            &item.key,
            item.label,
            reference,
            enabled,
            semantics,
            None,
            move |event| {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    if !live.visible.try_get()? {
                        return Ok(());
                    }
                    if let Event::CheckedChanged(value) = &event {
                        accepted.accept(value)?;
                        pulse.touch()?;
                        changed.emit(instance, *value);
                        if instance.is_alive() {
                            callback.emit(instance, ());
                        }
                        if instance.is_alive() && close {
                            live.root_close()?;
                        }
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                    return;
                }
                observe(&mut observers, instance, event);
            },
        )?;
        let weak = owner.downgrade();
        let mut previous = initial;
        owner.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let value = control.source.try_get()?;
                if dirty.take() || value != previous {
                    scope.patch(node, Patch::Checked(Change::Set(value)))?;
                    previous = value;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
    }
    scope.retain(owner)?;
    Ok(())
}
fn build_radios(
    scope: &Scope,
    state: &Rc<MenuState>,
    parent: &Common,
    namespace: &str,
    path: &[String],
    menu_id: &SemanticId,
    group: MenuRadioGroup,
) -> Result {
    let owner = scope.child()?;
    let mut group_path = path.to_vec();
    group_path.push(group.key);
    let group_id = id(namespace, &group_path, "group")?;
    let control = group.value.mount(&owner);
    let keys = Rc::new(
        group
            .items
            .iter()
            .map(|item| item.key.clone())
            .collect::<Vec<_>>(),
    );
    let known = keys.clone();
    let selected = composite::validated(&owner, &control.source, move |value| {
        if value.is_empty() || known.contains(value) {
            Ok(())
        } else {
            Err(invalid("unknown menu radio value"))
        }
    })?;
    let mut group_widget = glory::ui::view()
        .role(Role::Group)
        .class(if parent.unstyled {
            "sapid-menu-radio-group sapid-unstyled"
        } else {
            "sapid-menu-radio-group"
        })
        .semantics(composite::metadata(Semantics {
            id: Some(group_id.clone()),
            ..Default::default()
        }));
    let label = group.label;
    if !label.get().is_empty() {
        group_widget = group_widget.name(label);
    }
    let reference = owner.cage(None);
    group_widget.node_ref(reference).show_in(&owner)?;
    let content = owner.child_in(reference.get().ok_or(glory::ui::Error::Gone)?)?;
    let dirty = Dirty::new(&owner);
    let callback = Callback::new(group.changed);
    let initial = selected.get();
    let mut references = Vec::new();
    for mut item in group.items {
        item.common.theme = parent.theme.clone();
        item.common.size = parent.size;
        item.common.unstyled |= parent.unstyled;
        let reference = item.common.node_ref.unwrap_or_else(|| content.cage(None));
        references.push(reference);
        let enabled = item.common.enabled.clone();
        let value = item.key.clone();
        let accepted = control.clone();
        let changed = callback.clone();
        let pulse = dirty.clone();
        let mut item_path = group_path.clone();
        item_path.push(item.key.clone());
        let semantics = Semantics {
            id: Some(id(namespace, &item_path, "item")?),
            menu_item: Some(MenuItemSemantics {
                kind: MenuItemKind::Radio,
                owner: menu_id.clone(),
            }),
            check_group: Some(group_id.clone()),
            ..Default::default()
        };
        let native = configure!(
            item.common,
            glory::ui::check()
                .mode(CheckMode::Radio)
                .label(item.label.clone())
                .checked(initial == item.key)
                .node_ref(reference),
            &content,
            "sapid-menu-item sapid-menu-radio",
            menu_item_style(parent)
        );
        let live = state.clone();
        let close = group.close;
        let weak = content.downgrade();
        let instance = owner.id();
        let mut observers = item
            .common
            .handlers
            .remove(&EventKind::CheckedChanged)
            .unwrap_or_default();
        mount_member(
            &content,
            state,
            native,
            parent,
            &mut item.common,
            &item.key,
            item.label,
            reference,
            enabled,
            semantics,
            None,
            move |event| {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    if !live.visible.try_get()? {
                        return Ok(());
                    }
                    if event == Event::CheckedChanged(true) {
                        accepted.accept(&value)?;
                        pulse.touch()?;
                        changed.emit(instance, value.clone());
                        if instance.is_alive() && close {
                            live.root_close()?;
                        }
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                    return;
                }
                observe(&mut observers, instance, event);
            },
        )?;
    }
    owner.retain(content)?;
    let weak = owner.downgrade();
    let mut previous = initial;
    owner.effect(move || {
        let Some(scope) = weak.upgrade() else { return };
        let result = (|| -> Result {
            let value = selected.try_get()?;
            if dirty.take() || value != previous {
                for (key, reference) in keys.iter().zip(references.iter()) {
                    if let Some(node) = reference.try_get()? {
                        scope.patch(node, Patch::Checked(Change::Set(key == &value)))?;
                    }
                }
                previous = value;
            }
            Ok(())
        })();
        if let Err(error) = result {
            scope.report_error(error);
        }
    });
    scope.retain(owner)?;
    Ok(())
}

impl Widget for Menu {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.overflow {
            return Err(invalid("menu configuration budget exhausted"));
        }
        key(&self.key)?;
        if self.common.focus != FocusMode::Auto {
            return Err(invalid("menu owns its focus scope"));
        }
        validate(&self.key, &self.entries, &mut Vec::new(), &mut 0)?;
        Typeahead::new(self.typeahead)?;
        let trigger_id = id(&self.key, &[], "trigger")?;
        let anchor = if self.dropdown {
            trigger_id.clone()
        } else {
            self.anchor
                .clone()
                .ok_or_else(|| invalid("standalone Menu requires an explicit anchor"))?
        };
        let return_focus = self.return_focus.clone().unwrap_or_else(|| anchor.clone());
        let trigger = self.trigger.unwrap_or_else(|| scope.cage(None));
        let enabled = self.common.enabled.clone();
        let label = self.label.clone();
        let mut common = std::mem::take(&mut self.common);
        let mut trigger_observers = if self.dropdown {
            common
                .handlers
                .remove(&EventKind::Activate)
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let menu_observers = Observers::new(if self.dropdown {
            common.handlers.remove(&EventKind::Menu).unwrap_or_default()
        } else {
            Vec::new()
        });
        let shared = menu_observers.clone();
        let instance = scope.id();
        if self.dropdown {
            common.handlers.insert(
                EventKind::Menu,
                vec![Box::new(move |event| shared.emit(instance, event))],
            );
        }
        let theme = common.theme.clone();
        let size = common.size;
        let unstyled = common.unstyled;
        let state = Level {
            key: self.key.clone(),
            path: Vec::new(),
            common,
            label: label.clone(),
            open: std::mem::take(&mut self.open),
            changed: self.changed.take(),
            entries: std::mem::take(&mut self.entries),
            anchor,
            return_focus,
            placement: LayerPlacement::AnchorStart,
            parent: Weak::new(),
            root: Weak::new(),
            parent_visible: true.into(),
            root_enabled: true.into(),
            direction: self.direction,
            looping: self.looping,
            typeahead: self.typeahead,
            delay: self.delay.clone(),
            trigger,
        }
        .build(scope)?;
        if self.dropdown {
            let panel_id = id(&self.key, &[], "panel")?;
            let open = state.visible.clone();
            let source = state.clone();
            let weak = scope.downgrade();
            let command = state.clone();
            let key_scope = scope.downgrade();
            let mut button = glory::ui::button()
                .class(format!(
                    "sapid-menu-trigger {}{}",
                    theme.class(),
                    if unstyled { " sapid-unstyled" } else { "" }
                ))
                .label(label)
                .enabled(enabled)
                .node_ref(trigger)
                .semantics(scope.bond(move || {
                    composite::metadata(Semantics {
                        id: Some(trigger_id.clone()),
                        popup: Some(PopupKind::Menu),
                        expanded: Some(open.get()),
                        controls: Some(panel_id.clone()),
                        ..Default::default()
                    })
                }))
                .on(EventKind::Activate, move |event| {
                    let Some(scope) = weak.upgrade() else { return };
                    if let Err(error) = source.request(!source.open.source.get()) {
                        scope.report_error(error);
                        return;
                    }
                    observe(&mut trigger_observers, instance, event);
                })
                .on(EventKind::Menu, move |event| {
                    let Some(scope) = key_scope.upgrade() else {
                        return;
                    };
                    if let Event::Menu { from, command: key } = &event
                        && let Err(error) = command.command(&scope, *from, *key)
                    {
                        scope.report_error(error);
                        return;
                    }
                    menu_observers.emit(instance, event);
                });
            if !unstyled {
                button = button.style(theme.control(size));
            }
            button.show_in(scope)?;
        }
        Ok(())
    }
}
impl Widget for DropdownMenu {
    fn build(&mut self, scope: &Scope) -> Result {
        self.inner.build(scope)
    }
}
