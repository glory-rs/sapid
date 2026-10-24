//! Real tree items with finite current data, controlled expansion/selection and
//! Owner-scoped keyed rows. Lazy loading remains an explicit application request.
use crate::{
    appearance::{Common, common_methods},
    composite::{self, Callback, Observers},
    state::{Control, Dirty, Value},
    tree_model::*,
};
use glory::render::{
    Direction, Event, EventKind, FocusMode, NodeId, Role, Semantics, TreeCommand,
    TreeNavigation,
};
use glory::style::{
    Align, BorderStyle, Display, Edges, FlexDirection, Length, Overflow, Style,
};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Lotus, Scope};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

fn error(value: TreeError) -> glory::ui::Error {
    glory::render::Error::InvalidProps(value.message()).into()
}
fn selection(value: &BTreeSet<String>, mode: TreeSelectionMode) -> Result {
    validate_tree_keys(value).map_err(error)?;
    if mode == TreeSelectionMode::None && !value.is_empty()
        || mode == TreeSelectionMode::Single && value.len() > 1
    {
        return Err(error(TreeError::State));
    }
    Ok(())
}
fn accept_model(scope: &Scope, source: Lotus<Vec<TreeNode>>) -> Result<Cage<Rc<TreeModel>>> {
    let read = || {
        source.try_with(|nodes| {
            TreeModel::validate(nodes).map_err(error)?;
            TreeModel::new(nodes.clone()).map(Rc::new).map_err(error)
        })?
    };
    let model = scope.cage(read()?);
    let weak = scope.downgrade();
    scope.structural(move || {
        let Some(scope) = weak.upgrade() else { return };
        let next = source.try_with(|nodes| {
            TreeModel::validate(nodes).map_err(error)?;
            if glory::core::untracked(|| model.with(|old| old.nodes() == nodes)) {
                return Ok(None);
            }
            TreeModel::new(nodes.clone())
                .map(|next| Some(Rc::new(next)))
                .map_err(error)
        });
        match next {
            Ok(Ok(Some(next))) => model.revise(|old| *old = next),
            Ok(Ok(None)) => {}
            Ok(Err(error)) => scope.report_error(error),
            Err(error) => scope.report_error(error),
        }
    });
    Ok(model)
}
struct SetControl {
    control: Control<BTreeSet<String>>,
    accepted: Cage<BTreeSet<String>>,
    pending: RefCell<Option<BTreeSet<String>>>,
    changed: Callback<BTreeSet<String>>,
}
impl SetControl {
    fn propose(
        &self,
        state: &State,
        edit: impl FnOnce(&mut BTreeSet<String>),
        selected: bool,
    ) -> Result {
        let mut next = self
            .pending
            .borrow()
            .clone()
            .unwrap_or_else(|| self.accepted.get_untracked());
        edit(&mut next);
        if selected {
            selection(&next, state.mode)?;
        } else {
            validate_tree_keys(&next).map_err(error)?;
        }
        self.pending.replace(Some(next.clone()));
        self.control.accept(&next)?;
        state.dirty.touch()?;
        self.changed.emit(state.owner, next);
        Ok(())
    }
}
#[derive(Default)]
struct Mounted {
    keys: BTreeMap<String, NodeId>,
    nodes: BTreeMap<NodeId, String>,
}
struct Search {
    text: String,
    at: u64,
}
struct State {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    model: Cage<Rc<TreeModel>>,
    expanded: SetControl,
    selected: SetControl,
    mode: TreeSelectionMode,
    enabled: Lotus<bool>,
    logical: Cage<Option<String>>,
    physical: Cage<Option<String>>,
    mounted: RefCell<Mounted>,
    cursor: RefCell<Option<(NodeId, String)>>,
    focus: RefCell<Option<String>>,
    search: RefCell<Option<Search>>,
    dirty: Dirty,
    activated: Callback<String>,
    load: Callback<String>,
    follows: bool,
    intent: Callback<String>,
    columns: bool,
}
impl State {
    fn ensure(&self) -> Result {
        if self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn report(&self, result: Result) {
        if let Err(error) = result
            && let Some(scope) = self.weak.upgrade()
        {
            scope.report_error(error);
        }
    }
    fn node(&self, key: &str) -> Option<TreeNode> {
        self.model
            .with(|model| model.index(key).map(|index| model.nodes()[index].clone()))
    }
    fn usable(&self, key: &str) -> bool {
        self.enabled.get() && self.node(key).is_some_and(|node| !node.disabled)
    }
    fn origin(&self, from: NodeId, rows: &[TreeRow]) -> Option<String> {
        let key = self.mounted.borrow().nodes.get(&from).cloned()?;
        if !self.usable(&key) {
            return None;
        }
        self.cursor
            .borrow()
            .as_ref()
            .filter(|(origin, key)| *origin == from && rows.iter().any(|row| &row.key == key))
            .map(|(_, key)| key.clone())
            .or(Some(key))
    }
    fn request_focus(&self, key: String, physical: bool) -> Result {
        if self.logical.get_untracked().as_ref() != Some(&key) {
            self.logical.try_revise(|old| *old = Some(key.clone()))?;
        }
        if physical {
            self.focus.replace(Some(key));
            self.dirty.touch()?;
        }
        Ok(())
    }
    fn select(&self, key: String, activate: bool) -> Result {
        self.ensure()?;
        if !self.usable(&key) {
            return Ok(());
        }
        if self.columns
            && self
                .node(&key)
                .is_some_and(|node| node.kind != TreeKind::Leaf)
        {
            return self.expand(key, true, true);
        }
        if self.mode != TreeSelectionMode::None {
            self.selected.propose(
                self,
                |set| {
                    if self.mode == TreeSelectionMode::Single {
                        set.clear();
                        set.insert(key.clone());
                    } else if !set.remove(&key) {
                        set.insert(key.clone());
                    }
                },
                true,
            )?;
            self.intent.emit(self.owner, key.clone());
        }
        if activate {
            self.activated.emit(self.owner, key);
        }
        Ok(())
    }
    fn expand(&self, key: String, open: bool, focus: bool) -> Result {
        self.ensure()?;
        let Some(node) = self
            .node(&key)
            .filter(|node| !node.disabled && node.kind != TreeKind::Leaf)
        else {
            return Ok(());
        };
        if !self.enabled.get() {
            return Ok(());
        }
        let mut chain = BTreeSet::new();
        if self.columns {
            self.model.with(|model| {
                let mut index = model.index(&key);
                if !open {
                    index = index.and_then(|index| model.parent(index));
                }
                while let Some(current) = index {
                    chain.insert(model.nodes()[current].key.clone());
                    index = model.parent(current);
                }
            });
        }
        self.expanded.propose(
            self,
            |set| {
                if self.columns {
                    *set = chain;
                    return;
                }
                if open {
                    set.insert(key.clone());
                } else {
                    set.remove(&key);
                }
            },
            false,
        )?;
        if open && node.kind == TreeKind::Lazy && self.owner.is_alive() {
            self.load.emit(self.owner, key.clone());
        }
        if self.owner.is_alive() {
            self.request_focus(key, focus)?;
        }
        Ok(())
    }
    fn navigate(&self, from: NodeId, command: TreeCommand) -> Result {
        self.ensure()?;
        // An accepted expansion may change earlier in this host tick, before
        // row mounting. Refused controlled expansion remains closed here.
        let rows = self.expanded.control.source.try_with(|expanded| {
            validate_tree_keys(expanded).map_err(error)?;
            if self.columns {
                self.model
                    .with(|model| model.validate_column_path(expanded))
                    .map_err(error)?;
            }
            self.model
                .with(|model| model.visible(expanded).map_err(error))
        })??;
        let Some(key) = self.origin(from, &rows) else {
            return Ok(());
        };
        if matches!(command, TreeCommand::Select | TreeCommand::Activate) {
            return self.select(key, matches!(command, TreeCommand::Activate));
        }
        let Some(position) = rows.iter().position(|row| row.key == key) else {
            return Ok(());
        };
        let node = self.node(&key).ok_or(glory::ui::Error::Gone)?;
        let sibling = |row: &&TreeRow| !self.columns || row.parent == rows[position].parent;
        let mut next = None;
        match command {
            TreeCommand::Previous => {
                next = rows[..position]
                    .iter()
                    .rev()
                    .filter(sibling)
                    .find(|row| self.usable(&row.key))
            }
            TreeCommand::Next => {
                next = rows[position + 1..]
                    .iter()
                    .filter(sibling)
                    .find(|row| self.usable(&row.key))
            }
            TreeCommand::First => {
                next = rows
                    .iter()
                    .filter(sibling)
                    .find(|row| self.usable(&row.key))
            }
            TreeCommand::Last => {
                next = rows
                    .iter()
                    .rev()
                    .filter(sibling)
                    .find(|row| self.usable(&row.key))
            }
            TreeCommand::Open => {
                if node.kind == TreeKind::Leaf {
                    return Ok(());
                }
                let open = self.expanded.control.source.with(|set| set.contains(&key));
                if !open {
                    return self.expand(key, true, false);
                }
                next = rows[position + 1..]
                    .iter()
                    .take_while(|row| row.level > rows[position].level)
                    .find(|row| {
                        row.parent == Some(rows[position].index) && self.usable(&row.key)
                    });
            }
            TreeCommand::Close => {
                if self.expanded.control.source.with(|set| set.contains(&key))
                    && node.kind != TreeKind::Leaf
                {
                    return self.expand(key, false, false);
                }
                if let Some(parent) = rows[position].parent {
                    next = rows
                        .iter()
                        .find(|row| row.index == parent && self.usable(&row.key));
                }
            }
            TreeCommand::Search { text, at_millis } => {
                let old = self.search.borrow_mut().take();
                let mut query = if let Some(old) =
                    old.filter(|old| at_millis >= old.at && at_millis - old.at <= 700)
                {
                    old.text
                } else {
                    String::new()
                };
                let lower = text.to_lowercase();
                if query.len().saturating_add(lower.len()) > 64 {
                    query.clear();
                }
                query.push_str(&lower);
                let repeated = query
                    .chars()
                    .next()
                    .is_some_and(|first| query.chars().all(|c| c == first));
                let needle = if repeated {
                    text.to_lowercase()
                } else {
                    query.clone()
                };
                self.search.replace(Some(Search {
                    text: query,
                    at: at_millis,
                }));
                next = (1..=rows.len())
                    .map(|step| &rows[(position + step) % rows.len()])
                    .find(|row| {
                        (!self.columns || row.parent == rows[position].parent)
                            && self.usable(&row.key)
                            && self.node(&row.key).is_some_and(|node| {
                                node.label.to_lowercase().starts_with(&needle)
                            })
                    });
            }
            TreeCommand::Select | TreeCommand::Activate => unreachable!(),
        }
        if let Some(next) = next {
            self.cursor.replace(Some((from, next.key.clone())));
            self.request_focus(next.key.clone(), true)?;
            if self.follows {
                self.select(next.key.clone(), false)?;
            }
        }
        Ok(())
    }
}
type LabelFactory = Rc<RefCell<Box<dyn FnMut(Lotus<TreeNode>) -> Box<dyn Widget>>>>;
pub struct Tree {
    common: Common,
    key: String,
    source: Option<Lotus<Vec<TreeNode>>>,
    expanded: Value<BTreeSet<String>>,
    selected: Value<BTreeSet<String>>,
    expand_changed: Option<Box<dyn FnMut(BTreeSet<String>)>>,
    selection_changed: Option<Box<dyn FnMut(BTreeSet<String>)>>,
    activated: Option<Box<dyn FnMut(String)>>,
    load: Option<Box<dyn FnMut(String)>>,
    mode: TreeSelectionMode,
    direction: Direction,
    typeahead: bool,
    follows: bool,
    height: Option<f64>,
    label: Option<LabelFactory>,
    file_icons: bool,
    checked: Option<Lotus<BTreeMap<String, glory::render::TreeCheckState>>>,
    intent: Option<Box<dyn FnMut(String)>>,
    popup: Option<TreePopup>,
    columns: bool,
}
pub(crate) struct TreePopup {
    pub id: glory::render::SemanticId,
    pub labelled_by: glory::render::SemanticId,
    pub hidden: Lotus<bool>,
    pub layer: glory::render::Layer,
}
pub fn tree(key: impl Into<String>) -> Tree {
    Tree {
        common: Common::default(),
        key: key.into(),
        source: None,
        expanded: Value::default(),
        selected: Value::default(),
        expand_changed: None,
        selection_changed: None,
        activated: None,
        load: None,
        mode: TreeSelectionMode::Single,
        direction: Direction::Ltr,
        typeahead: true,
        follows: false,
        height: None,
        label: None,
        file_icons: false,
        checked: None,
        intent: None,
        popup: None,
        columns: false,
    }
}
pub fn file_tree(
    key: impl Into<String>,
    entries: impl IntoIterator<Item = FileEntry>,
) -> Result<Tree> {
    Ok(tree(key)
        .nodes(file_tree_nodes(entries).map_err(error)?)
        .file_icons(true))
}
impl Tree {
    common_methods!();
    pub fn nodes(mut self, source: impl Into<Lotus<Vec<TreeNode>>>) -> Self {
        self.source = Some(source.into());
        self
    }
    pub fn expanded(mut self, source: impl Into<Lotus<BTreeSet<String>>>) -> Self {
        self.expanded.source = Some(source.into());
        self
    }
    pub fn default_expanded(mut self, value: BTreeSet<String>) -> Self {
        self.expanded.default = value;
        self
    }
    pub fn selected(mut self, source: impl Into<Lotus<BTreeSet<String>>>) -> Self {
        self.selected.source = Some(source.into());
        self
    }
    pub fn default_selected(mut self, value: BTreeSet<String>) -> Self {
        self.selected.default = value;
        self
    }
    pub fn selection_mode(mut self, mode: TreeSelectionMode) -> Self {
        self.mode = mode;
        self
    }
    /// Refine tree items to checked/half-checked states. A missing live key is
    /// unchecked; list selection metadata is omitted when this facet is used.
    pub fn checked(
        mut self,
        source: impl Into<Lotus<BTreeMap<String, glory::render::TreeCheckState>>>,
    ) -> Self {
        self.checked = Some(source.into());
        self
    }
    /// One current user toggle intent, before controlled reconciliation. This
    /// supports explicit parent/leaf policies without inferring a stale set diff.
    pub fn on_selection_intent(mut self, callback: impl FnMut(String) + 'static) -> Self {
        self.intent = Some(Box::new(callback));
        self
    }
    pub(crate) fn popup_root(mut self, popup: TreePopup) -> Self {
        self.popup = Some(popup);
        self.common.focus = FocusMode::Programmatic;
        self
    }
    pub(crate) fn column_layout(mut self) -> Self {
        self.columns = true;
        self
    }
    pub fn on_expanded_change(
        mut self,
        callback: impl FnMut(BTreeSet<String>) + 'static,
    ) -> Self {
        self.expand_changed = Some(Box::new(callback));
        self
    }
    pub fn on_selection_change(
        mut self,
        callback: impl FnMut(BTreeSet<String>) + 'static,
    ) -> Self {
        self.selection_changed = Some(Box::new(callback));
        self
    }
    pub fn on_activate(mut self, callback: impl FnMut(String) + 'static) -> Self {
        self.activated = Some(Box::new(callback));
        self
    }
    pub fn on_load_children(mut self, callback: impl FnMut(String) + 'static) -> Self {
        self.load = Some(Box::new(callback));
        self
    }
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
    pub fn typeahead(mut self, value: bool) -> Self {
        self.typeahead = value;
        self
    }
    pub fn selection_follows_focus(mut self, value: bool) -> Self {
        self.follows = value;
        self
    }
    pub fn height(mut self, value: f64) -> Self {
        self.height = Some(value);
        self
    }
    pub fn file_icons(mut self, value: bool) -> Self {
        self.file_icons = value;
        self
    }
    pub fn render_label<W: Widget>(
        mut self,
        mut render: impl FnMut(Lotus<TreeNode>) -> W + 'static,
    ) -> Self {
        self.label = Some(Rc::new(RefCell::new(Box::new(move |node| {
            Box::new(render(node))
        }))));
        self
    }
}
impl Widget for Tree {
    fn build(&mut self, scope: &Scope) -> Result {
        let id = if let Some(popup) = &self.popup {
            popup.id.clone()
        } else {
            composite::id("tree", &self.key, "root", "root")?
        };
        let checked = self
            .checked
            .take()
            .map(|source| {
                composite::validated(scope, &source, |values| {
                    if values.len() > MAX_TREE_NODES
                        || values.keys().any(|key| !valid_tree_key(key))
                    {
                        return Err(error(TreeError::State));
                    }
                    Ok(())
                })
            })
            .transpose()?;
        if self
            .height
            .is_some_and(|height| !height.is_finite() || height <= 0. || height > 16384.)
            || self.follows && self.mode != TreeSelectionMode::Single
        {
            return Err(error(TreeError::State));
        }
        let model = accept_model(
            scope,
            self.source.take().ok_or_else(|| error(TreeError::Nodes))?,
        )?;
        let expanded = std::mem::take(&mut self.expanded).mount(scope);
        let selected = std::mem::take(&mut self.selected).mount(scope);
        let columns = self.columns;
        let accepted_expanded = composite::validated(scope, &expanded.source, move |value| {
            validate_tree_keys(value).map_err(error)?;
            if columns {
                model
                    .with(|model| model.validate_column_path(value))
                    .map_err(error)?;
            }
            Ok(())
        })?;
        let mode = self.mode;
        let accepted_selected =
            composite::validated(scope, &selected.source, move |value| selection(value, mode))?;
        let rows = scope.cage(
            model
                .with(|model| model.visible(&accepted_expanded.get_untracked()))
                .map_err(error)?,
        );
        let first = rows.with(|rows| {
            rows.iter()
                .find(|row| model.with(|model| !model.nodes()[row.index].disabled))
                .map(|row| row.key.clone())
        });
        let dirty = Dirty::new(scope);
        let state = Rc::new(State {
            owner: scope.id(),
            weak: scope.downgrade(),
            model,
            expanded: SetControl {
                control: expanded,
                accepted: accepted_expanded,
                pending: RefCell::new(None),
                changed: Callback::new(self.expand_changed.take()),
            },
            selected: SetControl {
                control: selected,
                accepted: accepted_selected,
                pending: RefCell::new(None),
                changed: Callback::new(self.selection_changed.take()),
            },
            mode,
            enabled: self.common.enabled.clone(),
            logical: scope.cage(first),
            physical: scope.cage(None),
            mounted: RefCell::new(Mounted::default()),
            cursor: RefCell::new(None),
            focus: RefCell::new(None),
            search: RefCell::new(None),
            dirty: dirty.clone(),
            activated: Callback::new(self.activated.take()),
            load: Callback::new(self.load.take()),
            follows: self.follows,
            intent: Callback::new(self.intent.take()),
            columns: self.columns,
        });
        let weak = Rc::downgrade(&state);
        scope.structural(move || {
            let Some(state) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let next = model
                    .with(|model| model.visible(&accepted_expanded.get()))
                    .map_err(error)?;
                let previous = state.logical.get_untracked();
                let mut candidate = previous.clone();
                let enabled = |key: &str| {
                    model.with(|model| {
                        model
                            .index(key)
                            .is_some_and(|index| !model.nodes()[index].disabled)
                    })
                };
                while let Some(key) = candidate.as_ref() {
                    if next.iter().any(|row| &row.key == key) && enabled(key) {
                        break;
                    }
                    candidate = rows.with(|rows| {
                        rows.iter()
                            .find(|row| &row.key == key)
                            .and_then(|row| row.parent)
                            .and_then(|parent| {
                                rows.iter()
                                    .find(|row| row.index == parent)
                                    .map(|row| row.key.clone())
                            })
                    });
                }
                let candidate = candidate.or_else(|| {
                    next.iter()
                        .find(|row| enabled(&row.key))
                        .map(|row| row.key.clone())
                });
                if glory::core::untracked(|| rows.with(|old| old != &next)) {
                    rows.try_revise(|old| *old = next)?;
                }
                if candidate != previous {
                    let physical = state.physical.get_untracked().is_some();
                    state.logical.try_revise(|old| *old = candidate.clone())?;
                    if physical && let Some(key) = candidate {
                        state.focus.replace(Some(key));
                        state.dirty.touch()?;
                    }
                }
                Ok(())
            })();
            state.report(result);
        });
        let weak = Rc::downgrade(&state);
        scope.effect(move || {
            if !dirty.take() {
                return;
            }
            let Some(state) = weak.upgrade() else { return };
            state.expanded.pending.take();
            state.selected.pending.take();
            state.cursor.take();
            let focus = state.focus.borrow_mut().take();
            if let Some(key) = focus
                && let Some(scope) = state.weak.upgrade()
            {
                let node = state.mounted.borrow().keys.get(&key).copied();
                if state.usable(&key) {
                    state.report(composite::focus_node(&scope, node));
                    if let Some(node) = node
                        && scope
                            .capabilities()
                            .is_ok_and(|capabilities| capabilities.scroll)
                    {
                        match scope.action(glory::render::Action::ScrollIntoView(node)) {
                            Ok(())
                            | Err(glory::ui::Error::Render(
                                glory::render::Error::Unsupported(_),
                            )) => {}
                            Err(error) => state.report(Err(error)),
                        }
                    }
                }
            }
        });
        let weak = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(state) = weak.upgrade() {
                state.expanded.changed.clear();
                state.selected.changed.clear();
                state.activated.clear();
                state.load.clear();
                state.intent.clear();
                state.mounted.replace(Mounted::default());
                state.cursor.take();
                state.focus.take();
                state.search.take();
            }
        })?;
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let mut base = composite::column(2.)
            .padding(Edges::all(4.))
            .min_width(Length::Px(0.));
        if !self.common.unstyled {
            base = crate::appearance::merge(base, self.common.theme.label(self.common.size))
                .radius(self.common.theme.radius);
            base.background = Some(self.common.theme.surface);
        }
        if let Some(height) = self.height {
            base = base.height(Length::Px(height)).overflow_y(Overflow::Auto);
        }
        let columns = self.columns;
        let (classes, custom) = self.common.appearance(scope, "sapid-tree", base.clone());
        // Geometry remains usable in unstyled mode.
        let style = if self.common.unstyled {
            let custom = self.common.style.clone();
            scope
                .bond(move || crate::appearance::merge(base.clone(), custom.get()))
                .into()
        } else {
            custom
        };
        let style: Lotus<Style> = if columns {
            scope
                .bond(move || {
                    let count =
                        rows.with(|rows| rows.iter().map(|row| row.level).max().unwrap_or(1));
                    let row_count = rows.with(|rows| {
                        rows.iter().map(|row| row.position + 1).max().unwrap_or(1)
                    });
                    style
                        .get()
                        .display(Display::Grid)
                        .grid_columns(glory::style::GridTracks::Tracks(
                            vec![glory::style::GridTrack::Px(180.); usize::from(count)],
                        ))
                        .grid_rows(glory::style::GridTracks::Tracks(
                            vec![glory::style::GridTrack::Px(40.); usize::from(row_count)],
                        ))
                        .overflow_x(Overflow::Auto)
                })
                .into()
        } else {
            style
        };
        let s = state.clone();
        let enabled = self.common.enabled.clone();
        let direction = self.direction;
        let typeahead = self.typeahead;
        let popup = self.popup.take();
        let observers = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::TreeNavigate)
                .unwrap_or_default(),
        );
        let cleanup = observers.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let mut root = glory::ui::view()
            .role(Role::Tree)
            .class(classes)
            .style(style)
            .node_ref(reference)
            .focus(self.common.focus)
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    id: Some(id.clone()),
                    tree_navigation: Some(TreeNavigation {
                        direction,
                        typeahead,
                        multiple: mode == TreeSelectionMode::Multiple,
                    }),
                    disabled: !enabled.get(),
                    hidden: popup.as_ref().is_some_and(|popup| popup.hidden.get()),
                    labelled_by: popup.as_ref().map(|popup| popup.labelled_by.clone()),
                    layer: popup.as_ref().map(|popup| popup.layer.clone()),
                    ..Default::default()
                }))
            }))
            .on(EventKind::TreeNavigate, move |event| {
                if s.owner.is_alive() {
                    if let Event::TreeNavigate { from, command } = &event {
                        s.report(s.navigate(*from, command.clone()));
                    }
                    observers.emit(s.owner, event);
                }
            });
        if let Some(name) = self.common.name.take() {
            root = root.name(name);
        } else {
            root = root.name("Tree");
        }
        let s = state.clone();
        let focus_observers = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::FocusWithin)
                .unwrap_or_default(),
        );
        let cleanup = focus_observers.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        root = root.on(EventKind::FocusWithin, move |event| {
            if let Event::FocusWithin { node, focused } = &event {
                let key = s.mounted.borrow().nodes.get(node).cloned();
                if *focused && let Some(key) = key {
                    s.physical.revise(|old| *old = Some(key.clone()));
                    s.report(s.request_focus(key, false));
                } else if !*focused && key.as_ref() == s.physical.get_untracked().as_ref() {
                    s.physical.revise(|old| *old = None);
                }
            }
            focus_observers.emit(s.owner, event);
        });
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |event| {
                crate::appearance::observe(&mut handlers, owner, event)
            });
        }
        // Custom labels may contain genuine editors. A real Fieldset supplies
        // host disabling for those controls as well as the tree's own typed
        // navigation/activation gate; ARIA alone cannot disable an HTML input.
        let field_ref = scope.cage(None);
        let enabled = self.common.enabled.clone();
        glory::ui::view()
            .class("sapid-tree-fieldset")
            .node_ref(field_ref)
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    field_part: Some(glory::render::FieldPart::Fieldset),
                    disabled: !enabled.get(),
                    ..Default::default()
                }))
            }))
            .style(
                Style::new()
                    .min_width(Length::Px(0.))
                    .padding(Edges::all(0.))
                    .margin(Edges::all(0.))
                    .border_width(Edges::all(0.)),
            )
            .show_in(scope)?;
        let fields = scope.child_in(field_ref.get().ok_or(glory::ui::Error::Gone)?)?;
        root.show_in(&fields)?;
        let area = scope.child_in(reference.get().ok_or(glory::ui::Error::Gone)?)?;
        let theme = self.common.theme.clone();
        let unstyled = self.common.unstyled;
        let renderer = self.label.clone();
        let icons = self.file_icons;
        glory::ui::each(
            rows,
            |row| row.key.clone(),
            move |row, _index| {
                let state = state.clone();
                let theme = theme.clone();
                let renderer = renderer.clone();
                glory::ui::component(move |scope| {
                    let key = row.with(|row| row.key.clone());
                    let reference = scope.cage(None);
                    let model = state.model;
                    let source = scope.bond(move || {
                        model.with(|model| {
                            let current = row.get();
                            model
                                .nodes()
                                .get(current.index)
                                .filter(|node| node.key == current.key)
                                .cloned()
                                .unwrap_or_else(|| TreeNode::new(current.key, "Unavailable"))
                        })
                    });
                    let value = scope.cage(source.get());
                    scope.structural(move || {
                        let next = source.get();
                        if value.with_untracked(|old| old != &next) {
                            value.revise(|old| *old = next);
                        }
                    });
                    let selected = state.selected.accepted;
                    let expanded = state.expanded.accepted;
                    let logical = state.logical;
                    let physical = state.physical;
                    let enabled = state.enabled.clone();
                    let current_key = key.clone();
                    let focus_key = key.clone();
                    let paint_key = key.clone();
                    let semantics = scope.bond(move || {
                        let node = value.get();
                        let coords = row.get();
                        Some(Box::new(Semantics {
                            tree_item: Some(glory::render::TreeItem {
                                level: coords.level,
                                position: coords.position,
                                count: coords.count,
                            }),
                            expanded: (node.kind != TreeKind::Leaf)
                                .then(|| expanded.with(|set| set.contains(&current_key))),
                            selected: (mode != TreeSelectionMode::None && checked.is_none())
                                .then(|| selected.with(|set| set.contains(&current_key))),
                            tree_checked: checked.map(|source| {
                                source.with(|values| {
                                    values
                                        .get(&current_key)
                                        .copied()
                                        .unwrap_or(glory::render::TreeCheckState::Unchecked)
                                })
                            }),
                            disabled: node.disabled || !enabled.get(),
                            ..Default::default()
                        }))
                    });
                    let enabled = state.enabled.clone();
                    let focus = scope.bond(move || {
                        if logical.with(|key| key.as_ref() == Some(&focus_key))
                            && !value.with(|node| node.disabled)
                            && enabled.get()
                        {
                            FocusMode::TabStop
                        } else {
                            FocusMode::Programmatic
                        }
                    });
                    let paint = scope.bond(move || {
                        let level = row.with(|row| row.level);
                        let selected = selected.with(|set| set.contains(&paint_key));
                        let focused = physical.with(|key| key.as_ref() == Some(&paint_key));
                        let inset = if columns {
                            8.
                        } else {
                            8. + f64::from(level - 1) * 20.
                        };
                        let mut style = Style::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Row)
                            .gap(6.)
                            .min_height(Length::Px(32.))
                            .min_width(Length::Px(0.))
                            .padding(if direction == Direction::Rtl {
                                Edges {
                                    top: 4.,
                                    bottom: 4.,
                                    left: 8.,
                                    right: inset,
                                }
                            } else {
                                Edges {
                                    top: 4.,
                                    bottom: 4.,
                                    left: inset,
                                    right: 8.,
                                }
                            });
                        style.align = Some(Align::Center);
                        if columns {
                            let position = row.with(|row| row.position);
                            style = style
                                .grid_column(glory::style::GridPlacement::At {
                                    start: u16::from(level),
                                    span: 1,
                                })
                                .grid_row(glory::style::GridPlacement::At {
                                    start: position + 1,
                                    span: 1,
                                });
                        }
                        if !unstyled {
                            style.color = Some(if value.with(|node| node.disabled) {
                                theme.disabled_text
                            } else {
                                theme.foreground
                            });
                            style.background = Some(if selected {
                                theme.accent_soft
                            } else {
                                theme.surface
                            });
                            style = style
                                .radius(theme.radius)
                                .border_width(Edges::all(1.))
                                .border_style(BorderStyle::Solid)
                                .border_color(if focused {
                                    theme.accent
                                } else {
                                    theme.surface
                                });
                        }
                        style
                    });
                    let s = state.clone();
                    let click_key = key.clone();
                    glory::ui::view()
                        .role(Role::TreeItem)
                        .class("sapid-tree-item")
                        .name(value.map(|node| node.label.clone()))
                        .node_ref(reference)
                        .semantics(semantics)
                        .focus(focus)
                        .style(paint)
                        .on(EventKind::Activate, move |_| {
                            if !s.usable(&click_key) {
                                return;
                            }
                            s.report(s.request_focus(click_key.clone(), true));
                            s.report(s.select(click_key.clone(), false));
                        })
                        .show_in(scope)?;
                    let node = reference.get().ok_or(glory::ui::Error::Gone)?;
                    {
                        let mut mounted = state.mounted.borrow_mut();
                        mounted.keys.insert(key.clone(), node);
                        mounted.nodes.insert(node, key.clone());
                    }
                    let weak = Rc::downgrade(&state);
                    let remove = key.clone();
                    scope.on_cleanup(move || {
                        if let Some(state) = weak.upgrade() {
                            let mut mounted = state.mounted.borrow_mut();
                            mounted.keys.remove(&remove);
                            mounted.nodes.remove(&node);
                        }
                    })?;
                    let content = scope.child_in(node)?;
                    if let Some(checked) = checked {
                        let check_key = key.clone();
                        glory::ui::text()
                            .class("sapid-tree-check")
                            .content(checked.map(move |values| match values.get(&check_key) {
                                Some(glory::render::TreeCheckState::Checked) => "☑".into(),
                                Some(glory::render::TreeCheckState::Mixed) => "⊟".into(),
                                _ => "☐".into(),
                            }))
                            .show_in(&content)?;
                    }
                    let s = state.clone();
                    let toggle_key = key.clone();
                    let enabled = state.enabled.clone();
                    glory::ui::when(value.map(|node| node.kind != TreeKind::Leaf), move || {
                        let s = s.clone();
                        let key = toggle_key.clone();
                        let enabled = enabled.clone();
                        crate::button()
                            .size(crate::Size::ExtraSmall)
                            .unstyled()
                            .focus(FocusMode::Programmatic)
                            .class("sapid-tree-disclosure")
                            .label(expanded.map({
                                let key = key.clone();
                                move |set| {
                                    if set.contains(&key) {
                                        "−".into()
                                    } else {
                                        "+".into()
                                    }
                                }
                            }))
                            .name(value.map(|node| format!("Toggle {}", node.label)))
                            .enabled(value.map(move |node| enabled.get() && !node.disabled))
                            .on_press(move || {
                                let open = !s
                                    .expanded
                                    .pending
                                    .borrow()
                                    .as_ref()
                                    .unwrap_or(&s.expanded.accepted.get_untracked())
                                    .contains(&key);
                                s.report(s.expand(key.clone(), open, true));
                            })
                    })
                    .show_in(&content)?;
                    if icons {
                        glory::ui::text()
                            .content(value.map(|node| {
                                if node.kind == TreeKind::Leaf {
                                    String::from("•")
                                } else {
                                    String::from("▸")
                                }
                            }))
                            .class("sapid-file-tree-kind")
                            .show_in(&content)?;
                    }
                    if let Some(renderer) = &renderer {
                        let widget = (renderer.borrow_mut())(value.into());
                        widget.show_in(&content)?;
                    } else {
                        let mut style = Style::new().min_width(Length::Px(0.));
                        style.grow = Some(1.);
                        glory::ui::text()
                            .content(value.map(|node| node.label.clone()))
                            .class("sapid-tree-label")
                            .style(style)
                            .show_in(&content)?;
                    }
                    scope.retain(content)?;
                    Ok(())
                })
            },
        )
        .show_in(&area)?;
        scope.retain(area)?;
        scope.retain(fields)?;
        Ok(())
    }
}
