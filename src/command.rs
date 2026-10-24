//! A command palette: Dialog + editable Input + finite live Listbox results.
use std::{
    cell::Cell,
    collections::HashSet,
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
};

use crate::appearance::{Common, common_methods, configure};
use crate::collection::{Collection, CollectionLimits, RovingFocus};
use crate::composite::{self, Callback};
use crate::state::{Control, Dirty, Value};
use glory::render::{
    ChoiceCommand, ComboboxKind, Event, EventKind, FocusMode, Layer, LayerPlacement,
    Navigation, NodeId, OptionSemantics, Orientation, PopupKind, Role, SemanticId, Semantics,
};
use glory::style::{Display, Edges, FlexDirection, Length, Limit, Overflow, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};

pub const MAX_COMMAND_ACTIONS: usize = 256;
pub const MAX_COMMAND_GROUPS: usize = 32;
pub const MAX_COMMAND_TEXT_BYTES: usize = 65_536;
const SEARCH_BYTES: usize = 4096;

/// Data only. Application actions are delivered through `on_action(key)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandAction {
    pub key: String,
    pub label: String,
    pub description: String,
    pub keywords: String,
    pub group: Option<String>,
    pub disabled: bool,
}
pub fn command_action(key: impl Into<String>, label: impl Into<String>) -> CommandAction {
    CommandAction {
        key: key.into(),
        label: label.into(),
        description: String::new(),
        keywords: String::new(),
        group: None,
        disabled: false,
    }
}
impl CommandAction {
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = value.into();
        self
    }
    pub fn keywords(mut self, value: impl Into<String>) -> Self {
        self.keywords = value.into();
        self
    }
    pub fn group(mut self, value: impl Into<String>) -> Self {
        self.group = Some(value.into());
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandGroup {
    pub key: String,
    pub label: String,
}
pub fn command_group(key: impl Into<String>, label: impl Into<String>) -> CommandGroup {
    CommandGroup {
        key: key.into(),
        label: label.into(),
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandResults {
    pub groups: Vec<CommandGroup>,
    pub actions: Vec<CommandAction>,
}
impl CommandResults {
    pub fn new(actions: Vec<CommandAction>) -> Self {
        Self {
            actions,
            groups: Vec::new(),
        }
    }
    fn validate(&self) -> Result {
        if self.actions.len() > MAX_COMMAND_ACTIONS || self.groups.len() > MAX_COMMAND_GROUPS {
            return Err(invalid(
                "command results exceed the finite action/group budget",
            ));
        }
        let mut groups = HashSet::with_capacity(self.groups.len());
        let mut actions = HashSet::with_capacity(self.actions.len());
        let mut bytes = 0usize;
        for group in &self.groups {
            key(&group.key)?;
            label(&group.label)?;
            if !groups.insert(group.key.as_str()) {
                return Err(invalid("duplicate command group key"));
            }
            bytes += group.key.len() + group.label.len();
        }
        for action in &self.actions {
            key(&action.key)?;
            label(&action.label)?;
            text(&action.description, 1024)?;
            text(&action.keywords, 1024)?;
            if !actions.insert(action.key.as_str()) {
                return Err(invalid("duplicate command action key"));
            }
            if let Some(group) = &action.group {
                key(group)?;
                if !groups.contains(group.as_str()) {
                    return Err(invalid("command action references an unknown group"));
                }
                bytes += group.len();
            }
            bytes += action.key.len()
                + action.label.len()
                + action.description.len()
                + action.keywords.len();
        }
        if bytes > MAX_COMMAND_TEXT_BYTES {
            return Err(invalid("command results exceed the text budget"));
        }
        Ok(())
    }
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn key(value: &str) -> Result {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(invalid("command keys must be bounded ASCII identifiers"));
    }
    Ok(())
}
fn text(value: &str, max: usize) -> Result {
    if value.len() > max || value.contains(['\0', '\r']) {
        return Err(invalid("command text exceeds its finite canonical budget"));
    }
    Ok(())
}
fn label(value: &str) -> Result {
    text(value, 256)?;
    if value.trim().is_empty() {
        return Err(invalid("command labels must be nonempty"));
    }
    Ok(())
}
fn search(value: &str) -> Result {
    text(value, SEARCH_BYTES)?;
    if value.contains('\n') {
        return Err(invalid("command search must be single-line"));
    }
    Ok(())
}
fn validate_action(action: &CommandAction) -> Result {
    key(&action.key)?;
    label(&action.label)?;
    text(&action.description, 1024)?;
    text(&action.keywords, 1024)?;
    if let Some(group) = &action.group {
        key(group)?
    }
    Ok(())
}

/// Controlled results explicitly identify the accepted query they answer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandResponse {
    pub query: String,
    pub results: CommandResults,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct LoadKey {
    query: String,
    active: bool,
    request: u64,
}
type Resource = glory::core::Resource<LoadKey, CommandResults, String>;
#[derive(Clone)]
enum Status {
    Ready,
    Pending,
    Failed(String),
    Cancelled,
}
#[derive(Clone)]
struct Snapshot {
    query: String,
    results: Rc<CommandResults>,
    status: Status,
    request: u64,
}

type LoadFuture = Pin<Box<dyn Future<Output = std::result::Result<CommandResults, String>>>>;
type Loader = Rc<dyn Fn(String) -> LoadFuture>;
pub struct Command {
    common: Common,
    key: String,
    label: Lotus<String>,
    open: Value<bool>,
    query: Value<String>,
    opened: Option<Box<dyn FnMut(bool)>>,
    searched: Option<Box<dyn FnMut(String)>>,
    action: Option<Box<dyn FnMut(String)>>,
    ready: Option<Box<dyn FnMut(CommandHandle)>>,
    results: Option<Lotus<CommandResponse>>,
    declarations: CommandResults,
    overflow: bool,
    declared_bytes: usize,
    loader: Option<Loader>,
}
pub fn command(key: impl Into<String>) -> Command {
    Command {
        common: Default::default(),
        key: key.into(),
        label: "Commands".into(),
        open: Default::default(),
        query: Default::default(),
        opened: None,
        searched: None,
        action: None,
        ready: None,
        results: None,
        declarations: Default::default(),
        overflow: false,
        declared_bytes: 0,
        loader: None,
    }
}
pub fn spotlight(key: impl Into<String>) -> Command {
    command(key)
}
impl Command {
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
    pub fn search(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.query.source = Some(value.into());
        self
    }
    pub fn default_search(mut self, value: impl Into<String>) -> Self {
        self.query.default = value.into();
        self
    }
    pub fn on_open_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.opened = Some(Box::new(callback));
        self
    }
    pub fn on_search_change(mut self, callback: impl FnMut(String) + 'static) -> Self {
        self.searched = Some(Box::new(callback));
        self
    }
    pub fn on_action(mut self, callback: impl FnMut(String) + 'static) -> Self {
        self.action = Some(Box::new(callback));
        self
    }
    pub fn on_ready(mut self, callback: impl FnMut(CommandHandle) + 'static) -> Self {
        self.ready = Some(Box::new(callback));
        self
    }
    pub fn action(mut self, action: CommandAction) -> Self {
        if !self.overflow {
            let bytes = action.key.len()
                + action.label.len()
                + action.description.len()
                + action.keywords.len()
                + action.group.as_ref().map_or(0, String::len);
            if self.declarations.actions.len() == MAX_COMMAND_ACTIONS
                || validate_action(&action).is_err()
                || bytes > MAX_COMMAND_TEXT_BYTES.saturating_sub(self.declared_bytes)
            {
                self.overflow = true;
            } else {
                self.declared_bytes += bytes;
                self.declarations.actions.push(action);
            }
        }
        self
    }
    pub fn actions(mut self, actions: impl IntoIterator<Item = CommandAction>) -> Self {
        if !self.overflow {
            for action in actions.into_iter().take(MAX_COMMAND_ACTIONS + 1) {
                self = self.action(action);
                if self.overflow {
                    break;
                }
            }
        }
        self
    }
    pub fn group(mut self, group: CommandGroup) -> Self {
        if !self.overflow {
            let bytes = group.key.len() + group.label.len();
            if self.declarations.groups.len() == MAX_COMMAND_GROUPS
                || key(&group.key).is_err()
                || label(&group.label).is_err()
                || bytes > MAX_COMMAND_TEXT_BYTES.saturating_sub(self.declared_bytes)
            {
                self.overflow = true
            } else {
                self.declared_bytes += bytes;
                self.declarations.groups.push(group)
            }
        }
        self
    }
    pub fn results(mut self, value: impl Into<Lotus<CommandResponse>>) -> Self {
        self.results = Some(value.into());
        self
    }
    pub fn load_with<
        F: Future<Output = std::result::Result<CommandResults, String>> + 'static,
    >(
        mut self,
        load: impl Fn(String) -> F + 'static,
    ) -> Self {
        self.loader = Some(Rc::new(move |query| Box::pin(load(query))));
        self
    }
}

#[derive(Clone)]
pub struct CommandHandle {
    state: Weak<Runtime>,
}
impl CommandHandle {
    pub fn open(&self) -> Result {
        self.state
            .upgrade()
            .ok_or(glory::ui::Error::Gone)?
            .request_open(true)
    }
    pub fn close(&self) -> Result {
        self.state
            .upgrade()
            .ok_or(glory::ui::Error::Gone)?
            .request_open(false)
    }
    pub fn toggle(&self) -> Result {
        let state = self.state.upgrade().ok_or(glory::ui::Error::Gone)?;
        state.request_open(!state.open.source.try_get()?)
    }
    pub fn cancel(&self) -> Result {
        let state = self.state.upgrade().ok_or(glory::ui::Error::Gone)?;
        if !state.owner.is_alive() {
            return Err(glory::ui::Error::Gone);
        }
        let resource = state
            .resource
            .ok_or_else(|| invalid("command cancellation requires a loader"))?;
        state.advance_request()?;
        resource.cancel()?;
        Ok(())
    }
    pub fn refresh(&self) -> Result {
        let state = self.state.upgrade().ok_or(glory::ui::Error::Gone)?;
        if !state.owner.is_alive() {
            return Err(glory::ui::Error::Gone);
        }
        if state.enabled.try_get()? && state.open.source.try_get()? {
            let resource = state
                .resource
                .ok_or_else(|| invalid("command refresh requires a loader"))?;
            state.advance_request()?;
            resource.refresh()?
        }
        Ok(())
    }
    /// Register one explicit physical ancestor. No document/global listener.
    pub fn register_shortcut(&self, target: NodeId) -> Result {
        let state = self.state.upgrade().ok_or(glory::ui::Error::Gone)?;
        let scope = state.scope.upgrade().ok_or(glory::ui::Error::Gone)?;
        if let Some(previous) = state.shortcut.get() {
            return if previous == target {
                Ok(())
            } else {
                Err(invalid("command allows one explicit shortcut scope"))
            };
        }
        let weak = Rc::downgrade(&state);
        scope.on(target, EventKind::Palette, move |event| {
            if matches!(event, Event::Palette { .. })
                && let Some(state) = weak.upgrade()
                && let Err(error) = state.request_open(!state.open.source.get())
                && let Some(scope) = state.scope.upgrade()
            {
                scope.report_error(error)
            }
        })?;
        state.shortcut.set(Some(target));
        Ok(())
    }
}

struct Runtime {
    scope: glory::ui::WeakScope,
    owner: glory::core::InstanceId,
    open: Control<bool>,
    query: Control<String>,
    opened: Callback<bool>,
    searched: Callback<String>,
    action: Callback<String>,
    enabled: Lotus<bool>,
    composing: Cage<bool>,
    shortcut: Cell<Option<NodeId>>,
    query_accepted: Cage<String>,
    snapshot: Lotus<Snapshot>,
    collection: Collection,
    cursor: RovingFocus,
    dirty: Dirty,
    resource: Option<Resource>,
    request: Cage<u64>,
    readiness: Lotus<bool>,
    filter: bool,
    key: String,
}
impl Runtime {
    fn advance_request(&self) -> Result {
        let next = self
            .request
            .try_get()?
            .checked_add(1)
            .ok_or_else(|| invalid("command request generation exhausted"))?;
        self.request.try_revise(|old| *old = next)?;
        Ok(())
    }
    fn request_open(&self, value: bool) -> Result {
        if !self.owner.is_alive() {
            return Err(glory::ui::Error::Gone);
        }
        if !self.enabled.try_get()? {
            return Ok(());
        }
        if self.open.source.try_get()? != value {
            self.open.accept(&value)?;
            self.opened.emit(self.owner, value)
        }
        Ok(())
    }
    fn ready(&self) -> Result<bool> {
        Ok(self.owner.is_alive() && self.readiness.try_get()?)
    }
    fn current(&self) -> Result<Option<crate::MountedItem>> {
        if !self.ready()? {
            return Ok(None);
        }
        let current = self.cursor.current()?;
        Ok(self
            .collection
            .mounted()?
            .into_iter()
            .find(|item| item.enabled && Some(&item.member) == current.as_ref()))
    }
    fn choose(&self, key: &str) -> Result {
        if !self.ready()? {
            return Ok(());
        }
        if !self
            .collection
            .mounted()?
            .iter()
            .any(|item| item.key == key && item.enabled)
        {
            return Ok(());
        }
        let snapshot = self.snapshot.try_get()?;
        if !snapshot
            .results
            .actions
            .iter()
            .any(|action| action.key == key && !action.disabled)
        {
            return Ok(());
        }
        self.action.emit(self.owner, key.to_owned());
        if self.owner.is_alive() {
            self.request_open(false)?
        }
        Ok(())
    }
    fn choice(&self, command: ChoiceCommand) -> Result {
        if !self.owner.is_alive() || !self.enabled.try_get()? || self.composing.try_get()? {
            return Ok(());
        }
        if command == ChoiceCommand::Close {
            return self.request_open(false);
        }
        if !self.ready()? {
            return Ok(());
        }
        if command == ChoiceCommand::Commit {
            if let Some(item) = self.current()? {
                self.choose(&item.key)?
            }
            return Ok(());
        }
        let items = self.collection.mounted()?;
        let next = match command {
            ChoiceCommand::First | ChoiceCommand::OpenFirst => {
                items.into_iter().find(|item| item.enabled)
            }
            ChoiceCommand::Last | ChoiceCommand::OpenLast => {
                items.into_iter().rev().find(|item| item.enabled)
            }
            ChoiceCommand::Next | ChoiceCommand::Previous => match self.current()? {
                Some(item) => self.cursor.destination(
                    item.node,
                    if command == ChoiceCommand::Next {
                        Navigation::Next
                    } else {
                        Navigation::Previous
                    },
                )?,
                None => items.into_iter().find(|item| item.enabled),
            },
            _ => None,
        };
        if let Some(item) = next {
            self.cursor.accept(&item)?
        }
        Ok(())
    }
    fn edit(&self, value: String, composing: bool) -> Result {
        if !self.owner.is_alive() || !self.enabled.try_get()? {
            return Ok(());
        }
        self.composing.try_revise(|old| *old = composing)?;
        if search(&value).is_err() {
            return self.dirty.touch();
        }
        let changed = self.query.source.try_with(|old| old != &value)?;
        if changed {
            self.query.accept(&value)?;
            self.searched.emit(self.owner, value)
        }
        if self.owner.is_alive() {
            self.dirty.touch()?
        }
        Ok(())
    }
}

fn snapshot(
    scope: &Scope,
    query: Cage<String>,
    active: Lotus<bool>,
    response: Option<Lotus<CommandResponse>>,
    loader: Option<Loader>,
    declarations: CommandResults,
    request: Cage<u64>,
) -> Result<(Lotus<Snapshot>, Option<Resource>)> {
    if let Some(loader) = loader {
        let load_active = active.clone();
        let resource: Resource = scope.owner().resource(
            move || LoadKey {
                query: query.get(),
                active: load_active.get(),
                request: request.get(),
            },
            move |key| {
                let loader = loader.clone();
                async move {
                    if !key.active {
                        return Ok(CommandResults::default());
                    }
                    let results = loader(key.query).await.map_err(|error| {
                        if error.len() <= 1024 {
                            error
                        } else {
                            "Command loader error exceeds its text budget".into()
                        }
                    })?;
                    results.validate().map_err(|error| error.to_string())?;
                    Ok(results)
                }
            },
        );
        let empty = Rc::new(CommandResults::default());
        let value = resource.map(move |state| {
            use glory::core::ResourceState;
            let current = query.get();
            if !active.get() {
                return Snapshot {
                    query: current,
                    results: empty.clone(),
                    status: Status::Ready,
                    request: request.get(),
                };
            }
            match state {
                ResourceState::Ready { key, value } => Snapshot {
                    query: key.query.clone(),
                    results: value.clone(),
                    status: Status::Ready,
                    request: key.request,
                },
                ResourceState::Failed { key, error } => Snapshot {
                    query: key.query.clone(),
                    results: empty.clone(),
                    status: Status::Failed((**error).clone()),
                    request: key.request,
                },
                ResourceState::Unavailable { .. } => Snapshot {
                    query: current,
                    results: empty.clone(),
                    status: Status::Pending,
                    request: request.get(),
                },
                ResourceState::Idle => Snapshot {
                    query: current,
                    results: empty.clone(),
                    status: Status::Cancelled,
                    request: request.get(),
                },
                ResourceState::Pending { key } => Snapshot {
                    query: key.query.clone(),
                    results: empty.clone(),
                    status: Status::Pending,
                    request: key.request,
                },
            }
        });
        Ok((value.into(), Some(resource)))
    } else if let Some(response) = response {
        let accepted = composite::validated(scope, &response, |response| {
            search(&response.query)?;
            response.results.validate()
        })?;
        Ok((
            scope
                .bond(move || {
                    accepted.with(|response| Snapshot {
                        query: response.query.clone(),
                        results: Rc::new(response.results.clone()),
                        status: Status::Ready,
                        request: 0,
                    })
                })
                .into(),
            None,
        ))
    } else {
        let results = Rc::new(declarations);
        Ok((
            scope
                .bond(move || Snapshot {
                    query: query.get(),
                    results: results.clone(),
                    status: Status::Ready,
                    request: 0,
                })
                .into(),
            None,
        ))
    }
}

#[derive(Clone)]
struct Section {
    key: String,
    label: String,
    offset: usize,
    actions: Vec<CommandAction>,
}
fn sections(state: &Runtime) -> Vec<Section> {
    if !state.ready().unwrap_or(false) {
        return Vec::new();
    }
    let snapshot = state.snapshot.get();
    let query = if state.filter {
        state.query_accepted.get().to_lowercase()
    } else {
        String::new()
    };
    let matches = |action: &&CommandAction| {
        query.is_empty()
            || action.label.to_lowercase().contains(&query)
            || action.description.to_lowercase().contains(&query)
            || action.keywords.to_lowercase().contains(&query)
    };
    let mut groups = Vec::new();
    let mut offset = 0;
    for (key, label) in std::iter::once(("", "")).chain(
        snapshot
            .results
            .groups
            .iter()
            .map(|group| (group.key.as_str(), group.label.as_str())),
    ) {
        let actions = snapshot
            .results
            .actions
            .iter()
            .filter(|action| action.group.as_deref().unwrap_or("") == key)
            .filter(matches)
            .cloned()
            .collect::<Vec<_>>();
        if !actions.is_empty() {
            let count = actions.len();
            groups.push(Section {
                key: key.to_owned(),
                label: label.to_owned(),
                offset,
                actions,
            });
            offset += count;
        }
    }
    groups
}

impl Widget for Command {
    fn build(&mut self, scope: &Scope) -> Result {
        key(&self.key)?;
        if self.overflow {
            return Err(invalid(
                "command declaration admission exceeded a finite budget",
            ));
        }
        self.declarations.validate()?;
        if (self.results.is_some() && self.loader.is_some())
            || ((self.results.is_some() || self.loader.is_some())
                && (!self.declarations.actions.is_empty()
                    || !self.declarations.groups.is_empty()))
        {
            return Err(invalid(
                "command static declarations, controlled results and loader are mutually exclusive",
            ));
        }
        if self.common.focus != FocusMode::Auto {
            return Err(invalid("command owns its trigger and editor focus"));
        }
        let label = composite::validated(scope, &self.label, |value| label(value))?;
        let query = std::mem::take(&mut self.query).mount(scope);
        let accepted = composite::validated(scope, &query.source, |value| search(value))?;
        let open = std::mem::take(&mut self.open).mount(scope);
        let open_source = open.source.clone();
        let enabled = self.common.enabled.clone();
        let composing = scope.cage(false);
        let request = scope.cage(0_u64);
        let active = scope.bond(move || open_source.get() && enabled.get() && !composing.get());
        let filter = self.loader.is_none() && self.results.is_none();
        let (snapshot, resource) = snapshot(
            scope,
            accepted,
            active.into(),
            self.results.take(),
            self.loader.take(),
            std::mem::take(&mut self.declarations),
            request,
        )?;
        let observed = snapshot.clone();
        let raw_query = query.source.clone();
        let has_loader = resource.is_some();
        let readiness = scope.bond(move || {
            active.get()
                && accepted.with(|value| {
                    raw_query.with(|raw| value == raw)
                        && observed.with(|snapshot| {
                            snapshot.query == *value
                                && matches!(snapshot.status, Status::Ready)
                                && (!has_loader || snapshot.request == request.get())
                        })
                })
        });
        let collection = Collection::new(
            scope,
            CollectionLimits {
                items: MAX_COMMAND_ACTIONS,
                ..Default::default()
            },
        )?;
        let cursor = RovingFocus::new(scope, collection.clone(), true)?;
        let state = Rc::new(Runtime {
            scope: scope.downgrade(),
            owner: scope.id(),
            open,
            query,
            query_accepted: accepted,
            opened: Callback::new(self.opened.take()),
            searched: Callback::new(self.searched.take()),
            action: Callback::new(self.action.take()),
            enabled: self.common.enabled.clone(),
            composing,
            shortcut: Cell::new(None),
            snapshot,
            collection,
            cursor,
            dirty: Dirty::new(scope),
            resource,
            request,
            readiness: readiness.into(),
            filter,
            key: self.key.clone(),
        });
        let panel_id = composite::id("command", &self.key, "dialog", "root")?;
        let input_id = composite::id("command", &self.key, "input", "root")?;
        let list_id = composite::id("command", &self.key, "listbox", "root")?;
        let trigger_id = composite::id("command", &self.key, "trigger", "root")?;
        let root_ref = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-command",
            Style::new()
        )
        .node_ref(root_ref);
        for (kind, handlers) in std::mem::take(&mut self.common.handlers) {
            let callbacks = crate::composite::Observers::new(handlers);
            let owner = scope.id();
            root = root.on(kind, move |event| callbacks.emit(owner, event));
        }
        root.show_in(scope)?;
        let content = scope.child_in(root_ref.get().ok_or(glory::ui::Error::Gone)?)?;
        let trigger_state = state.clone();
        let target = panel_id.clone();
        let id = trigger_id.clone();
        let enabled = self.common.enabled.clone();
        glory::ui::button()
            .label(scope.bond(move || format!("Open {}", label.get())))
            .class("sapid-command-trigger")
            .style(self.common.theme.control(self.common.size))
            .enabled(enabled)
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    id: Some(id.clone()),
                    controls: Some(target.clone()),
                    popup: Some(PopupKind::Dialog),
                    expanded: Some(trigger_state.open.source.get()),
                    ..Default::default()
                }))
            }))
            .on(EventKind::Activate, {
                let weak = Rc::downgrade(&state);
                move |_| {
                    if let Some(state) = weak.upgrade() {
                        report(&state, state.request_open(!state.open.source.get()))
                    }
                }
            })
            .show_in(&content)?;
        let panel_ref = scope.cage(None);
        let panel_state = state.clone();
        let id = panel_id.clone();
        let editor = input_id.clone();
        let theme = self.common.theme.clone();
        let plain = self.common.unstyled;
        let panel_style = if plain {
            Style::new()
        } else {
            theme
                .control(self.common.size)
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(12.0)
                .padding(Edges::all(18.0))
                .width(Length::Px(560.0))
                .max_width(Limit::Percent(95.0))
                .max_height(Limit::Percent(85.0))
                .overflow_y(Overflow::Auto)
        };
        glory::ui::view()
            .role(Role::Dialog)
            .name(label)
            .focus(FocusMode::Programmatic)
            .class("sapid-command-dialog")
            .style(panel_style)
            .node_ref(panel_ref)
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    id: Some(id.clone()),
                    hidden: !panel_state.open.source.get() || !panel_state.enabled.get(),
                    layer: Some(Layer {
                        modal: true,
                        placement: LayerPlacement::Center,
                        initial_focus: Some(editor.clone()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }))
            }))
            .on(EventKind::Dismiss, {
                let weak = Rc::downgrade(&state);
                move |_| {
                    if let Some(state) = weak.upgrade() {
                        report(&state, state.request_open(false))
                    }
                }
            })
            .show_in(&content)?;
        let body = scope.child_in(panel_ref.get().ok_or(glory::ui::Error::Gone)?)?;
        let input_ref = scope.cage(None);
        let input_state = state.clone();
        let id = input_id;
        let list = list_id.clone();
        let mut input = glory::ui::input()
            .name("Search commands")
            .value(accepted.get())
            .node_ref(input_ref)
            .class("sapid-command-search")
            .style(if plain {
                Style::new()
            } else {
                self.common.theme.control(self.common.size)
            })
            .enabled(self.common.enabled.clone())
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    id: Some(id.clone()),
                    combobox: Some(ComboboxKind::Editable),
                    controls: Some(list.clone()),
                    popup: Some(PopupKind::Listbox),
                    expanded: Some(input_state.open.source.get()),
                    active_descendant: input_state.current().ok().flatten().and_then(|item| {
                        composite::id("command", &input_state.key, "option", &item.key).ok()
                    }),
                    ..Default::default()
                }))
            }));
        for kind in [EventKind::ValueChanged, EventKind::Commit] {
            let weak = Rc::downgrade(&state);
            input = input.on(kind, move |event| {
                if let Some(state) = weak.upgrade() {
                    let result = match event {
                        Event::ValueChanged {
                            value, composing, ..
                        } => state.edit(value, composing),
                        Event::Commit { value, .. } => state.edit(value, false),
                        _ => Ok(()),
                    };
                    report(&state, result)
                }
            })
        }
        input = input
            .on(EventKind::Composition, {
                let weak = Rc::downgrade(&state);
                move |event| {
                    if let Some(state) = weak.upgrade()
                        && let Event::Composition { active: value } = event
                    {
                        report(
                            &state,
                            state
                                .composing
                                .try_revise(|old| *old = value)
                                .map_err(Into::into)
                                .and_then(|_| state.dirty.touch()),
                        )
                    }
                }
            })
            .on(EventKind::Choice, {
                let weak = Rc::downgrade(&state);
                move |event| {
                    if let Some(state) = weak.upgrade()
                        && let Event::Choice { command, .. } = event
                    {
                        report(&state, state.choice(command))
                    }
                }
            });
        input.show_in(&body)?;
        let input_node = input_ref.get().ok_or(glory::ui::Error::Gone)?;
        let list_ref = scope.cage(None);
        glory::ui::view()
            .role(Role::Listbox)
            .name("Command results")
            .class("sapid-command-results")
            .style(
                Style::new()
                    .max_height(Limit::Px(320.0))
                    .overflow_y(Overflow::Auto),
            )
            .node_ref(list_ref)
            .semantics(Some(Box::new(Semantics {
                id: Some(list_id.clone()),
                navigation: Some(Orientation::Vertical),
                ..Default::default()
            })))
            .show_in(&body)?;
        let results_scope = scope.child_in(list_ref.get().ok_or(glory::ui::Error::Gone)?)?;
        let live = state.clone();
        let data = scope.bond(move || sections(&live));
        let live = state.clone();
        let theme = self.common.theme.clone();
        let size = self.common.size;
        glory::ui::each(
            data,
            |section| section.key.clone(),
            move |section, _| {
                let live = live.clone();
                let owner = list_id.clone();
                let theme = theme.clone();
                glory::ui::component(move |scope| {
                    mount_section(
                        scope,
                        section,
                        &live,
                        owner.clone(),
                        theme.clone(),
                        size,
                        plain,
                    )
                })
            },
        )
        .show_in(&results_scope)?;
        let status_state = state.clone();
        glory::ui::view()
            .role(Role::Status)
            .fill(glory::ui::text().content(scope.bond(move || {
                let value = status_state.snapshot.get();
                if status_state.composing.get() {
                    return "Composing search".to_owned();
                }
                if value.query != status_state.query_accepted.get() {
                    return "Loading commands".to_owned();
                }
                match value.status {
                    Status::Pending => "Loading commands".into(),
                    Status::Cancelled => "Search cancelled".into(),
                    Status::Failed(error) => error,
                    Status::Ready => {
                        if sections(&status_state).is_empty() {
                            "No matching commands".into()
                        } else {
                            String::new()
                        }
                    }
                }
            })))
            .show_in(&body)?;
        glory::ui::button()
            .label("Close commands")
            .class("sapid-command-close")
            .style(if plain {
                Style::new()
            } else {
                self.common.theme.control(self.common.size)
            })
            .on(EventKind::Activate, {
                let weak = Rc::downgrade(&state);
                move |_| {
                    if let Some(state) = weak.upgrade() {
                        report(&state, state.request_open(false))
                    }
                }
            })
            .show_in(&body)?;
        let live = state.clone();
        let weak = scope.downgrade();
        let mut previous = accepted.get();
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let dirty = live.dirty.take();
                let query = live.query_accepted.try_get()?;
                if !live.composing.try_get()? && (dirty || query != previous) {
                    scope.set_input_value(input_node, query.clone())?;
                    previous = query;
                }
                live.cursor.repair(None)?;
                if let Some(item) = live.current()?
                    && scope.capabilities()?.scroll
                {
                    scope.action(glory::render::Action::RevealOption { node: item.node })?
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error)
            }
        });
        let cleanup = state.clone();
        scope.on_cleanup(move || {
            cleanup.opened.clear();
            cleanup.searched.clear();
            cleanup.action.clear();
        })?;
        body.retain(results_scope)?;
        content.retain(body)?;
        scope.retain(content)?;
        if let Some(mut callback) = self.ready.take() {
            callback(CommandHandle {
                state: Rc::downgrade(&state),
            })
        }
        Ok(())
    }
}
fn report(state: &Runtime, result: Result) {
    if let Err(error) = result
        && let Some(scope) = state.scope.upgrade()
    {
        scope.report_error(error)
    }
}

fn mount_section(
    scope: &Scope,
    section: Cage<Section>,
    state: &Rc<Runtime>,
    list_id: SemanticId,
    theme: crate::Theme,
    size: crate::Size,
    plain: bool,
) -> Result {
    let reference = scope.cage(None);
    let mut group = glory::ui::view()
        .node_ref(reference)
        .class("sapid-command-group")
        .style(
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(4.0),
        );
    if !section.with(|section| section.key.is_empty()) {
        group = group
            .role(Role::Group)
            .name(section.map(|section| section.label.clone()))
    }
    group.show_in(scope)?;
    let content = scope.child_in(reference.get().ok_or(glory::ui::Error::Gone)?)?;
    if !section.with(|section| section.key.is_empty()) {
        glory::ui::text()
            .content(section.map(|section| section.label.clone()))
            .show_in(&content)?;
    }
    let live = state.clone();
    glory::ui::each(
        section.map(|section| section.actions.clone()),
        |action| action.key.clone(),
        move |action, index| {
            let live = live.clone();
            let list_id = list_id.clone();
            let theme = theme.clone();
            glory::ui::component(move |scope| {
                let key = action.with(|action| action.key.clone());
                let id = composite::id("command", &live.key, "option", &key)?;
                let reference = scope.cage(None);
                let active_state = live.clone();
                let member = scope.cage(None::<crate::CollectionMember>);
                let enabled_state = live.clone();
                let enabled = scope.bond(move || {
                    enabled_state.ready().unwrap_or(false)
                        && !action.with(|action| action.disabled)
                });
                let list_id = list_id.clone();
                let meta = scope.bond(move || {
                    Some(Box::new(Semantics {
                        id: Some(id.clone()),
                        option: Some(OptionSemantics {
                            owner: list_id.clone(),
                        }),
                        selected: Some(member.with(|member| {
                            member.is_some()
                                && active_state.cursor.current().ok().flatten().as_ref()
                                    == member.as_ref()
                        })),
                        ..Default::default()
                    }))
                });
                let callback = Rc::downgrade(&live);
                let item_key = key.clone();
                glory::ui::button()
                    .label(action.map(|action| {
                        if action.description.is_empty() {
                            action.label.clone()
                        } else {
                            format!("{} — {}", action.label, action.description)
                        }
                    }))
                    .focus(FocusMode::Programmatic)
                    .class("sapid-command-option")
                    .style(if plain {
                        Style::new()
                    } else {
                        theme.control(size)
                    })
                    .enabled(enabled)
                    .semantics(meta)
                    .node_ref(reference)
                    .on(EventKind::Activate, move |_| {
                        if let Some(state) = callback.upgrade() {
                            report(&state, state.choose(&item_key))
                        }
                    })
                    .show_in(scope)?;
                let token = live.collection.register_ordered(
                    scope,
                    key,
                    reference,
                    enabled,
                    action.map(|action| action.label.clone()),
                    scope.bond(move || section.with(|section| section.offset) + index.get()),
                )?;
                member.revise(|value| *value = Some(token));
                Ok(())
            })
        },
    )
    .show_in(&content)?;
    scope.retain(content)?;
    Ok(())
}

#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;
