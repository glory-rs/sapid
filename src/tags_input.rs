//! Free tag values and editor text form one atomic controlled proposal.
use crate::appearance::{Common, common_methods, configure};
use crate::composite::{self, Callback, Observers};
use crate::state::{Control, Dirty, Value};
use glory::render::{
    Descriptor, Event, EventKind, FormControl, Props, SelectEntry, SelectOption, SelectProps,
    SemanticId, Semantics, TagSeparators, TagsCommand,
};
use glory::style::{Display, Edges, FlexDirection, FlexWrap, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};
use std::rc::Rc;

pub const MAX_TAGS_INPUT_VALUES: usize = 256;
pub const MAX_TAGS_INPUT_TAG_BYTES: usize = 256;
pub const MAX_TAGS_INPUT_TOTAL_BYTES: usize = 64 * 1024;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TagsInputState {
    pub values: Vec<String>,
    pub search: String,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TagDuplicatePolicy {
    #[default]
    Reject,
    Ignore,
    Allow,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagRejection {
    Duplicate,
    TagTooLong,
    TooMany,
    TotalBytes,
    SearchTooLong,
    NonCanonical,
}

#[derive(Clone, Copy)]
struct Settings {
    max_tags: usize,
    duplicates: TagDuplicatePolicy,
    separators: TagSeparators,
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn validate_text(value: &str) -> std::result::Result<(), TagRejection> {
    if value.len() > glory::render::MAX_TAG_INPUT_BYTES {
        return Err(TagRejection::SearchTooLong);
    }
    if value.contains(['\0', '\r', '\n']) {
        return Err(TagRejection::NonCanonical);
    }
    Ok(())
}
impl Settings {
    fn validate(self, state: &TagsInputState) -> Result {
        if self.max_tags == 0
            || self.max_tags > MAX_TAGS_INPUT_VALUES
            || state.values.len() > self.max_tags
            || validate_text(&state.search).is_err()
        {
            return Err(invalid("invalid bounded tags input state"));
        }
        let mut bytes = 0usize;
        for (index, value) in state.values.iter().enumerate() {
            if value.trim().is_empty()
                || value.len() > MAX_TAGS_INPUT_TAG_BYTES
                || value.contains(['\0', '\r', '\n'])
                || self.duplicates != TagDuplicatePolicy::Allow
                    && state.values[..index].contains(value)
            {
                return Err(invalid("invalid bounded tag value or duplicate"));
            }
            bytes += value.len();
            if bytes > MAX_TAGS_INPUT_TOTAL_BYTES {
                return Err(invalid("tags input total value budget"));
            }
        }
        Ok(())
    }
    fn insert(
        self,
        state: &mut TagsInputState,
        text: &str,
    ) -> std::result::Result<Vec<String>, TagRejection> {
        let mut added = Vec::new();
        let mut bytes: usize = state.values.iter().map(String::len).sum();
        for value in text.split(|character| self.separators.contains(character)) {
            let value = value.trim();
            if value.is_empty() {
                continue;
            }
            if value.len() > MAX_TAGS_INPUT_TAG_BYTES {
                return Err(TagRejection::TagTooLong);
            }
            if value.contains(['\0', '\r', '\n']) {
                return Err(TagRejection::NonCanonical);
            }
            if state.values.iter().any(|existing| existing == value) {
                match self.duplicates {
                    TagDuplicatePolicy::Reject => return Err(TagRejection::Duplicate),
                    TagDuplicatePolicy::Ignore => continue,
                    TagDuplicatePolicy::Allow => {}
                }
            }
            if state.values.len() == self.max_tags {
                return Err(TagRejection::TooMany);
            }
            bytes += value.len();
            if bytes > MAX_TAGS_INPUT_TOTAL_BYTES {
                return Err(TagRejection::TotalBytes);
            }
            state.values.push(value.to_owned());
            added.push(value.to_owned());
        }
        state.search.clear();
        Ok(added)
    }
}
pub struct TagsInput {
    common: Common,
    key: String,
    state: Value<TagsInputState>,
    settings: Settings,
    pills: bool,
    changed: Option<Box<dyn FnMut(TagsInputState)>>,
    committed: Option<Box<dyn FnMut(Vec<String>)>>,
    removed: Option<Box<dyn FnMut(String)>>,
    rejected: Option<Box<dyn FnMut(TagRejection)>>,
    field: Option<crate::FieldControl>,
    form_name: Option<Lotus<String>>,
    required: Lotus<bool>,
    readonly: Lotus<bool>,
    invalid: Lotus<bool>,
}
pub fn tags_input(key: impl Into<String>) -> TagsInput {
    TagsInput {
        common: Common::default(),
        key: key.into(),
        state: Value::default(),
        settings: Settings {
            max_tags: MAX_TAGS_INPUT_VALUES,
            duplicates: TagDuplicatePolicy::Reject,
            separators: TagSeparators::default(),
        },
        pills: false,
        changed: None,
        committed: None,
        removed: None,
        rejected: None,
        field: None,
        form_name: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
    }
}
impl TagsInput {
    common_methods!();
    pub(crate) fn pills(mut self) -> Self {
        self.pills = true;
        self
    }
    pub fn label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn state(mut self, value: impl Into<Lotus<TagsInputState>>) -> Self {
        self.state.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: Vec<String>) -> Self {
        self.state.default.values = value;
        self
    }
    pub fn default_search(mut self, value: impl Into<String>) -> Self {
        self.state.default.search = value.into();
        self
    }
    pub fn duplicate_policy(mut self, value: TagDuplicatePolicy) -> Self {
        self.settings.duplicates = value;
        self
    }
    pub fn separators(mut self, value: TagSeparators) -> Self {
        self.settings.separators = value;
        self
    }
    pub fn max_tags(mut self, value: usize) -> Self {
        self.settings.max_tags = value;
        self
    }
    pub fn on_change(mut self, callback: impl FnMut(TagsInputState) + 'static) -> Self {
        self.changed = Some(Box::new(callback));
        self
    }
    pub fn on_commit(mut self, callback: impl FnMut(Vec<String>) + 'static) -> Self {
        self.committed = Some(Box::new(callback));
        self
    }
    pub fn on_remove(mut self, callback: impl FnMut(String) + 'static) -> Self {
        self.removed = Some(Box::new(callback));
        self
    }
    pub fn on_reject(mut self, callback: impl FnMut(TagRejection) + 'static) -> Self {
        self.rejected = Some(Box::new(callback));
        self
    }
    pub fn field(mut self, value: crate::FieldControl) -> Self {
        self.common.theme = value.theme.clone();
        self.common.size = value.size;
        self.common.unstyled = value.unstyled;
        self.field = Some(value);
        self
    }
    pub fn form_name(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.form_name = Some(value.into());
        self
    }
    pub fn required(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.required = value.into();
        self
    }
    pub fn readonly(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn invalid(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.invalid = value.into();
        self
    }
}
struct Controller {
    owner: glory::core::InstanceId,
    state: Control<TagsInputState>,
    accepted: Cage<TagsInputState>,
    settings: Settings,
    composing: Cage<bool>,
    editable: Lotus<bool>,
    dirty: Dirty,
    changed: Callback<TagsInputState>,
    committed: Callback<Vec<String>>,
    removed: Callback<String>,
    rejected: Callback<TagRejection>,
}
impl Controller {
    fn current(&self) -> Result<TagsInputState> {
        self.state.source.try_with(|value| {
            self.settings.validate(value)?;
            Ok::<_, glory::ui::Error>(value.clone())
        })?
    }
    fn propose(&self, value: TagsInputState) -> Result<bool> {
        self.settings.validate(&value)?;
        self.state.accept(&value)?;
        self.changed.emit(self.owner, value.clone());
        if !self.owner.is_alive() {
            return Ok(false);
        }
        self.state.source.try_with(|current| {
            self.settings.validate(current)?;
            Ok::<_, glory::ui::Error>(current == &value)
        })?
    }
    fn edit(&self, value: String, composing: bool) -> Result {
        if !self.editable.try_get()? {
            return Ok(());
        }
        self.dirty.touch()?;
        if let Err(reason) = validate_text(&value) {
            self.rejected.emit(self.owner, reason);
            return Ok(());
        }
        self.composing.try_revise(|current| *current = composing)?;
        let mut next = self.current()?;
        next.search = value;
        self.propose(next)?;
        Ok(())
    }
    fn remove(&self, value: &str, occurrence: usize) -> Result {
        if !self.owner.is_alive() || !self.editable.try_get()? || self.composing.try_get()? {
            return Ok(());
        }
        let mut next = self.current()?;
        let Some(index) = next
            .values
            .iter()
            .enumerate()
            .filter(|(_, item)| item.as_str() == value)
            .nth(occurrence)
            .map(|(index, _)| index)
        else {
            return Ok(());
        };
        let removed = next.values.remove(index);
        self.dirty.touch()?;
        if self.propose(next)? {
            self.removed.emit(self.owner, removed)
        }
        Ok(())
    }
    fn command(&self, command: &TagsCommand) -> Result {
        if !self.owner.is_alive() || !self.editable.try_get()? || self.composing.try_get()? {
            return Ok(());
        }
        command.validate()?;
        self.dirty.touch()?;
        let mut next = self.current()?;
        let text = match command {
            TagsCommand::RemoveLast => {
                if !next.search.is_empty() {
                    return Ok(());
                }
                let Some(value) = next.values.pop() else {
                    return Ok(());
                };
                if self.propose(next)? {
                    self.removed.emit(self.owner, value)
                }
                return Ok(());
            }
            TagsCommand::Commit { value } => {
                if next.search != *value {
                    return Ok(());
                }
                value.clone()
            }
            TagsCommand::Paste {
                value,
                text,
                start,
                end,
            } => {
                if next.search != *value {
                    return Ok(());
                }
                let byte = |offset: u32| {
                    let mut units = 0u32;
                    for (index, character) in value.char_indices() {
                        if units == offset {
                            return index;
                        }
                        units += character.len_utf16() as u32;
                    }
                    value.len()
                };
                let start = byte(*start);
                let end = byte(*end);
                let mut combined = String::with_capacity(value.len() + text.len());
                combined.push_str(&value[..start]);
                combined.push_str(text);
                combined.push_str(&value[end..]);
                combined
            }
        };
        // Work on a bounded private proposal. A later invalid token cannot
        // publish an earlier part of the paste or clear the accepted search.
        let added = match self.settings.insert(&mut next, &text) {
            Ok(added) => added,
            Err(reason) => {
                self.rejected.emit(self.owner, reason);
                return Ok(());
            }
        };
        if self.propose(next)? && !added.is_empty() {
            self.committed.emit(self.owner, added)
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Row {
    value: String,
    occurrence: usize,
}
impl Widget for TagsInput {
    fn build(&mut self, scope: &Scope) -> Result {
        let settings = self.settings;
        if settings.max_tags == 0 || settings.max_tags > MAX_TAGS_INPUT_VALUES {
            return Err(invalid("tags input max_tags must be 1..=256"));
        }
        let input_id = self
            .field
            .as_ref()
            .map(|field| Ok(field.id.clone()))
            .unwrap_or_else(|| composite::id("tags-input", &self.key, "input", "root"))?;
        let form_name = composite::form_name(scope, &self.form_name)?;
        let state = std::mem::take(&mut self.state).mount(scope);
        let accepted =
            composite::validated(scope, &state.source, move |value| settings.validate(value))?;
        let enabled = self
            .field
            .as_ref()
            .map(|field| field.enabled(scope, self.common.enabled.clone()))
            .unwrap_or_else(|| self.common.enabled.clone());
        let metadata = crate::field::control_metadata(
            scope,
            self.field.as_ref(),
            self.common.name.is_some(),
            self.required.clone(),
            self.readonly.clone(),
            self.invalid.clone(),
        );
        let editable_enabled = enabled.clone();
        let editable_metadata = metadata.clone();
        let editable: Lotus<bool> = scope
            .bond(move || {
                editable_enabled.get()
                    && !editable_metadata.get().is_some_and(|value| value.readonly)
            })
            .into();
        let controller = Rc::new(Controller {
            owner: scope.id(),
            state,
            accepted,
            settings,
            composing: scope.cage(false),
            editable,
            dirty: Dirty::new(scope),
            changed: Callback::new(self.changed.take()),
            committed: Callback::new(self.committed.take()),
            removed: Callback::new(self.removed.take()),
            rejected: Callback::new(self.rejected.take()),
        });
        let cleanup = controller.clone();
        scope.on_cleanup(move || {
            cleanup.changed.clear();
            cleanup.committed.clear();
            cleanup.removed.clear();
            cleanup.rejected.clear();
        })?;
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let root = scope.create(Descriptor::new(Props::Group))?;
        let content = scope.child_in(root)?;
        let rows = scope.bond(move || {
            accepted.with(|state| {
                state
                    .values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| Row {
                        value: value.clone(),
                        occurrence: state.values[..index]
                            .iter()
                            .filter(|item| *item == value)
                            .count(),
                    })
                    .collect::<Vec<_>>()
            })
        });
        let tags = controller.clone();
        let theme = self.common.theme.clone();
        let plain = self.common.unstyled;
        let pills = self.pills;
        glory::ui::each(
            rows,
            |row| row.clone(),
            move |row, _| {
                let tags = tags.clone();
                let theme = theme.clone();
                glory::ui::component(move |scope| {
                    let row = row.get();
                    let command = tags.clone();
                    let weak = scope.downgrade();
                    let mut style = Style::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Row)
                        .flex_wrap(FlexWrap::Wrap);
                    if !plain {
                        style.background = Some(theme.accent_soft);
                        style.color = Some(theme.accent_text);
                        style = style.padding(Edges::all(4.0)).radius(if pills {
                            999.0
                        } else {
                            4.0
                        });
                    }
                    glory::ui::view()
                        .class(if pills {
                            "sapid-pill-input-tag"
                        } else {
                            "sapid-tags-input-tag"
                        })
                        .style(style)
                        .fill(glory::ui::text().content(row.value.clone()))
                        .fill(
                            glory::ui::button()
                                .label("×")
                                .name(format!("Remove {} ({})", row.value, row.occurrence + 1))
                                .enabled(tags.editable.clone())
                                .on(EventKind::Activate, move |_| {
                                    let Some(scope) = weak.upgrade() else { return };
                                    let result = command
                                        .remove(&row.value, row.occurrence)
                                        .and_then(|()| {
                                            if scope.capabilities()?.focus
                                                && let Some(node) = reference.try_get()?
                                            {
                                                scope.action(glory::render::Action::Focus(
                                                    node,
                                                ))?;
                                            }
                                            Ok(())
                                        });
                                    if let Err(error) = result {
                                        scope.report_error(error)
                                    }
                                }),
                        )
                        .show_in(scope)
                })
            },
        )
        .show_in(&content)?;
        let input_metadata = metadata.clone();
        let metadata = scope.bond(move || {
            let mut value = input_metadata.get().map(|value| *value).unwrap_or_default();
            value.id = Some(input_id.clone());
            value.form_control = None;
            value.tags_input = Some(settings.separators);
            Some(Box::new(value))
        });
        let base = self.common.theme.control(self.common.size);
        let mut input = configure!(
            self.common,
            glory::ui::input(),
            scope,
            if pills {
                "sapid-pills-input"
            } else {
                "sapid-tags-input"
            },
            base
        )
        .value(accepted.with(|state| state.search.clone()))
        .enabled(enabled.clone())
        .semantics(metadata)
        .node_ref(reference);
        for kind in [
            EventKind::ValueChanged,
            EventKind::Commit,
            EventKind::Composition,
            EventKind::Tags,
        ] {
            let command = controller.clone();
            let weak = scope.downgrade();
            let observers =
                Observers::new(self.common.handlers.remove(&kind).unwrap_or_default());
            input = input.on(kind, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                let result = match &event {
                    Event::ValueChanged {
                        value, composing, ..
                    } => command.edit(value.clone(), *composing),
                    Event::Commit { value, .. } => command.edit(value.clone(), false),
                    Event::Composition { active } => command
                        .composing
                        .try_revise(|current| *current = *active)
                        .map_err(Into::into)
                        .and_then(|()| command.dirty.touch()),
                    Event::Tags {
                        command: action, ..
                    } => command.command(action),
                    _ => Ok(()),
                };
                if let Err(error) = result {
                    scope.report_error(error);
                    return;
                }
                observers.emit(scope.id(), event);
            });
        }
        for (kind, handlers) in std::mem::take(&mut self.common.handlers) {
            let observers = Observers::new(handlers);
            let owner = scope.id();
            input = input.on(kind, move |event| observers.emit(owner, event));
        }
        input.show_in(&content)?;
        let input_node = reference.get().ok_or(glory::ui::Error::Gone)?;
        let weak = scope.downgrade();
        let observed = controller.clone();
        let mut previous = accepted.with(|state| state.search.clone());
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let dirty = observed.dirty.take();
                let search = observed.accepted.try_with(|state| state.search.clone())?;
                if !observed.composing.try_get()? && (dirty || search != previous) {
                    scope.set_input_value(input_node, search.clone())?;
                    previous = search;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error)
            }
        });
        let hidden = scope.cage(None);
        glory::ui::view()
            .semantics(Some(Box::new(Semantics {
                hidden: true,
                ..Default::default()
            })))
            .node_ref(hidden)
            .show_in(&content)?;
        let proxy_scope = scope.child_in(hidden.get().ok_or(glory::ui::Error::Gone)?)?;
        let proxies = scope.bond(move || vec![accepted.with(|state| state.values.clone())]);
        let form_meta = crate::field::control_metadata(
            scope,
            self.field.as_ref(),
            false,
            false.into(),
            false.into(),
            false.into(),
        );
        glory::ui::each(
            proxies,
            |values| values.clone(),
            move |values, _| {
                let enabled = enabled.clone();
                let form_meta = form_meta.clone();
                glory::ui::component(move |scope| {
                    let values = values.get();
                    let options = values
                        .into_iter()
                        .enumerate()
                        .map(|(index, value)| {
                            SelectEntry::Option(SelectOption {
                                key: SemanticId::new(format!("tag-{index}"))
                                    .expect("bounded index key"),
                                label: String::new(),
                                value,
                                disabled: false,
                            })
                        })
                        .collect::<Vec<_>>();
                    let selection = options
                        .iter()
                        .map(|entry| match entry {
                            SelectEntry::Option(option) => option.key.clone(),
                            _ => unreachable!(),
                        })
                        .collect::<Vec<_>>();
                    let name = form_name;
                    let meta = form_meta.clone();
                    glory::ui::select()
                        .configuration(SelectProps {
                            multiple: true,
                            entries: Rc::new(options),
                            ..Default::default()
                        })
                        .selected(selection)
                        .enabled(enabled.clone())
                        .semantics(scope.bond(move || {
                            Some(Box::new(Semantics {
                                form_control: name
                                    .map(|name| name.get())
                                    .or_else(|| {
                                        meta.get().and_then(|meta| {
                                            meta.form_control.map(|value| value.name)
                                        })
                                    })
                                    .map(|name| FormControl { name, value: None }),
                                ..Default::default()
                            }))
                        }))
                        .show_in(scope)
                })
            },
        )
        .show_in(&proxy_scope)?;
        content.retain(proxy_scope)?;
        scope.retain(content)?;
        scope.retain(controller)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glory::render::{InputProps, NodeId};
    use glory::ui::App;
    fn nodes(app: &App) -> Vec<NodeId> {
        let tree = app.tree();
        let mut pending = tree.roots().to_vec();
        let mut nodes = Vec::new();
        while let Some(id) = pending.pop() {
            nodes.push(id);
            pending.extend(tree.node(id).unwrap().children.iter().copied());
        }
        nodes
    }
    fn editor(app: &App) -> NodeId {
        nodes(app)
            .into_iter()
            .find(|id| {
                matches!(
                    app.tree().node(*id).unwrap().descriptor.view().props,
                    Props::Input(_)
                )
            })
            .unwrap()
    }
    fn input(app: &App) -> InputProps {
        let tree = app.tree();
        let Props::Input(value) = tree.node(editor(app)).unwrap().descriptor.view().props
        else {
            panic!()
        };
        value.clone()
    }
    fn proxy(app: &App) -> NodeId {
        nodes(app)
            .into_iter()
            .find(|id| {
                matches!(
                    app.tree().node(*id).unwrap().descriptor.view().props,
                    Props::Select(_)
                )
            })
            .unwrap()
    }
    fn values(app: &App) -> Vec<String> {
        let tree = app.tree();
        let Props::Select(value) = tree.node(proxy(app)).unwrap().descriptor.view().props
        else {
            panic!()
        };
        assert!(value.multiple);
        value
            .options()
            .filter(|(option, _)| value.selection.keys.contains(&option.key))
            .map(|(option, _)| option.value.clone())
            .collect()
    }
    fn edit(app: &App, value: &str, composing: bool) {
        app.dispatch(
            editor(app),
            Event::ValueChanged {
                value: value.into(),
                composing,
                revision: input(app).revision,
            },
        )
        .unwrap();
    }
    fn command(app: &App, command: TagsCommand) {
        let id = editor(app);
        app.dispatch(
            id,
            Event::Tags {
                from: id,
                revision: input(app).revision,
                command,
            },
        )
        .unwrap();
    }
    fn commit(app: &App) {
        command(
            app,
            TagsCommand::Commit {
                value: input(app).value,
            },
        );
    }
    #[test]
    fn free_tags_commit_and_remove_have_no_known_option_requirement_or_combo_roles() {
        let app = App::memory();
        app.mount(tags_input("free").default_value(vec!["initial".into()]))
            .unwrap();
        let editor_id = editor(&app);
        let proxy_id = proxy(&app);
        edit(&app, "new free tag", false);
        assert_eq!(proxy(&app), proxy_id);
        commit(&app);
        assert_eq!(values(&app), ["initial", "new free tag"]);
        assert_eq!(input(&app).value, "");
        assert_eq!(editor(&app), editor_id);
        command(&app, TagsCommand::RemoveLast);
        assert_eq!(values(&app), ["initial"]);
        let tree = app.tree();
        let descriptor = tree.node(editor_id).unwrap().descriptor.view();
        assert!(descriptor.semantics.as_ref().unwrap().combobox.is_none());
        assert!(descriptor.role.is_none());
    }
    #[test]
    fn controlled_atomic_refusal_preserves_tags_search_and_accepted_observer_order() {
        let app = App::memory();
        let state = app.scope().cage(TagsInputState {
            values: vec!["A".into()],
            search: "B".into(),
        });
        let reject = app.scope().cage(true);
        let order = Rc::new(std::cell::RefCell::new(Vec::new()));
        let proposals = order.clone();
        let commits = order.clone();
        let removes = order.clone();
        app.mount(
            tags_input("controlled")
                .state(state)
                .on_change(move |value| {
                    proposals.borrow_mut().push("proposal");
                    if !reject.get() {
                        state.revise(|current| *current = value);
                    }
                })
                .on_commit(move |_| commits.borrow_mut().push("commit"))
                .on_remove(move |_| removes.borrow_mut().push("remove")),
        )
        .unwrap();
        commit(&app);
        assert_eq!(values(&app), ["A"]);
        assert_eq!(input(&app).value, "B");
        assert_eq!(*order.borrow(), ["proposal"]);
        reject.revise(|value| *value = false);
        commit(&app);
        assert_eq!(values(&app), ["A", "B"]);
        assert_eq!(input(&app).value, "");
        assert_eq!(*order.borrow(), ["proposal", "proposal", "commit"]);
        reject.revise(|value| *value = true);
        command(&app, TagsCommand::RemoveLast);
        assert_eq!(values(&app), ["A", "B"]);
        assert_eq!(
            *order.borrow(),
            ["proposal", "proposal", "commit", "proposal"]
        );
    }
    #[test]
    fn paste_replaces_utf16_selection_and_commits_all_free_values_atomically() {
        let app = App::memory();
        app.mount(tags_input("paste").default_search("A😀B"))
            .unwrap();
        command(
            &app,
            TagsCommand::Paste {
                value: "A😀B".into(),
                text: "c,d\r\ne".into(),
                start: 1,
                end: 3,
            },
        );
        assert_eq!(values(&app), ["Ac", "d", "eB"]);
        assert_eq!(input(&app).value, "");
        let id = editor(&app);
        assert!(
            app.dispatch(
                id,
                Event::Tags {
                    from: id,
                    revision: input(&app).revision,
                    command: TagsCommand::Paste {
                        value: "😀".into(),
                        text: "x,y".into(),
                        start: 1,
                        end: 2
                    }
                }
            )
            .is_err()
        );
    }
    #[test]
    fn duplicate_policy_and_invalid_later_paste_token_never_partially_commit() {
        for policy in [
            TagDuplicatePolicy::Reject,
            TagDuplicatePolicy::Ignore,
            TagDuplicatePolicy::Allow,
        ] {
            let app = App::memory();
            let rejects = app.scope().cage(Vec::<TagRejection>::new());
            app.mount(
                tags_input("duplicates")
                    .duplicate_policy(policy)
                    .default_value(vec!["A".into()])
                    .on_reject(move |reason| rejects.revise(|values| values.push(reason))),
            )
            .unwrap();
            edit(&app, "A", false);
            commit(&app);
            match policy {
                TagDuplicatePolicy::Reject => {
                    assert_eq!(values(&app), ["A"]);
                    assert_eq!(input(&app).value, "A");
                    assert_eq!(rejects.get(), [TagRejection::Duplicate]);
                }
                TagDuplicatePolicy::Ignore => {
                    assert_eq!(values(&app), ["A"]);
                    assert_eq!(input(&app).value, "");
                }
                TagDuplicatePolicy::Allow => assert_eq!(values(&app), ["A", "A"]),
            }
        }
        let app = App::memory();
        let rejects = app.scope().cage(Vec::<TagRejection>::new());
        app.mount(
            tags_input("late-invalid")
                .default_search("draft")
                .on_reject(move |reason| rejects.revise(|values| values.push(reason))),
        )
        .unwrap();
        command(
            &app,
            TagsCommand::Paste {
                value: "draft".into(),
                text: format!("valid,{}", "x".repeat(257)),
                start: 0,
                end: 5,
            },
        );
        assert!(values(&app).is_empty());
        assert_eq!(input(&app).value, "draft");
        assert_eq!(rejects.get(), [TagRejection::TagTooLong]);
    }
    #[test]
    fn max_tags_is_a_whole_batch_boundary_and_search_is_rechecked_on_stale_commands() {
        let app = App::memory();
        let rejects = app.scope().cage(Vec::<TagRejection>::new());
        app.mount(
            tags_input("max")
                .max_tags(1)
                .default_value(vec!["A".into()])
                .default_search("B")
                .on_reject(move |reason| rejects.revise(|values| values.push(reason))),
        )
        .unwrap();
        commit(&app);
        assert_eq!(values(&app), ["A"]);
        assert_eq!(input(&app).value, "B");
        assert_eq!(rejects.get(), [TagRejection::TooMany]);
        command(&app, TagsCommand::RemoveLast);
        assert_eq!(values(&app), ["A"]);
        command(
            &app,
            TagsCommand::Commit {
                value: "old host text".into(),
            },
        );
        assert_eq!(values(&app), ["A"]);
    }
    #[test]
    fn composition_preserves_preedit_and_disabled_readonly_commands_have_no_effect() {
        let app = App::memory();
        let readonly = app.scope().cage(false);
        let enabled = app.scope().cage(true);
        app.mount(
            tags_input("ime")
                .default_value(vec!["A".into()])
                .readonly(readonly)
                .enabled(enabled),
        )
        .unwrap();
        let id = editor(&app);
        let before = input(&app);
        app.dispatch(id, Event::Composition { active: true })
            .unwrap();
        edit(&app, "Fra", true);
        command(
            &app,
            TagsCommand::Commit {
                value: "Fra".into(),
            },
        );
        assert_eq!(values(&app), ["A"]);
        assert_eq!(input(&app), before);
        app.dispatch(id, Event::Composition { active: false })
            .unwrap();
        assert_eq!(input(&app).value, "Fra");
        readonly.revise(|value| *value = true);
        app.flush().unwrap();
        commit(&app);
        assert_eq!(values(&app), ["A"]);
        readonly.revise(|value| *value = false);
        enabled.revise(|value| *value = false);
        app.flush().unwrap();
        commit(&app);
        assert_eq!(values(&app), ["A"]);
    }
    #[test]
    fn huge_sources_validate_before_clone_and_maximum_live_tags_keep_bounded_owners() {
        let app = App::memory();
        assert!(
            app.mount(tags_input("huge").state(TagsInputState {
                values: vec!["A".into(); 1_000_000],
                search: String::new()
            }))
            .is_err()
        );
        assert_eq!(app.stats().nodes, 0);
        assert!(
            app.mount(tags_input("huge-search").default_search("x".repeat(1_000_000)))
                .is_err()
        );
        assert_eq!(app.stats().nodes, 0);
        let state = app.scope().cage(TagsInputState {
            values: vec!["x".repeat(256); 256],
            search: String::new(),
        });
        app.mount(
            tags_input("max-live")
                .duplicate_policy(TagDuplicatePolicy::Allow)
                .state(state),
        )
        .unwrap();
        assert!(app.stats().nodes < 1100);
        assert_eq!(values(&app).len(), 256);
        state.revise(|value| value.values = vec!["A".into(); 1_000_000]);
        assert!(app.flush().is_err());
        assert!(app.stats().nodes < 1100);
        state.revise(|value| value.values = vec!["A".into()]);
        app.flush().unwrap();
        assert!(app.stats().nodes < 20);
        assert_eq!(values(&app), ["A"]);
    }
    #[test]
    fn field_required_search_has_no_successful_name_or_false_native_required_proxy_is_single_multiple()
     {
        let html = glory::ssr::snapshot(
            crate::field("tags-field")
                .label("Skills")
                .form_name("skills")
                .required(true)
                .control(|field| {
                    tags_input("skills")
                        .field(field)
                        .default_value(vec!["Rust".into(), "C++".into()])
                }),
        )
        .unwrap()
        .html;
        assert_eq!(html.matches("<select").count(), 1);
        assert!(html.contains("multiple=\"\"") && html.contains("name=\"skills\""));
        assert!(
            html.contains("value=\"Rust\" selected") && html.contains("value=\"C++\" selected")
        );
        assert!(
            !html.contains(" required=\"\"")
                && !html.contains("role=\"combobox\"")
                && !html.contains("aria-haspopup")
                && !html.contains("aria-expanded")
        );
        assert!(html.contains("aria-required=\"true\""));
    }
}
