//! Optional finite rich editing. The document is data, never HTML markup.
use crate::appearance::{Common, common_methods, merge, observe};
use crate::composite::Callback;
use glory::render::{
    self, Action, Event, EventKind, NodeId, Query, QueryToken, QueryValue, RichCommand,
    RichDocument, RichFormat, RichMarks, RichSelection, RichSnapshot, Semantics,
};
use glory::style::{Display, FlexDirection, Length, Overflow, Style, WhiteSpace};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

pub const MAX_EDITOR_HISTORY: usize = 32;
pub const MAX_EDITOR_HISTORY_BYTES: usize = 2 * 1024 * 1024;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditorHistory {
    pub undo: usize,
    pub redo: usize,
    pub charged_bytes: usize,
}
#[derive(Clone)]
struct Entry {
    document: Rc<RichDocument>,
    selection: RichSelection,
}
#[derive(Default)]
struct History {
    undo: VecDeque<Entry>,
    redo: VecDeque<Entry>,
    bytes: usize,
}
impl History {
    fn summary(&self) -> EditorHistory {
        EditorHistory {
            undo: self.undo.len(),
            redo: self.redo.len(),
            charged_bytes: self.bytes,
        }
    }
    fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.bytes = 0;
    }
    fn charge(entry: &Entry) -> usize {
        entry.document.charged_bytes() + std::mem::size_of::<Entry>()
    }
    fn push(&mut self, entry: Entry, redo: bool) {
        self.bytes += Self::charge(&entry);
        if redo {
            self.redo.push_back(entry)
        } else {
            self.undo.push_back(entry)
        }
        while self.undo.len() + self.redo.len() > MAX_EDITOR_HISTORY
            || self.bytes > MAX_EDITOR_HISTORY_BYTES
        {
            let old = if !self.undo.is_empty() {
                self.undo.pop_front()
            } else {
                self.redo.pop_front()
            };
            let Some(old) = old else { break };
            self.bytes -= Self::charge(&old);
        }
    }
    fn accept(&mut self, previous: Entry, kind: ChangeKind) {
        match kind {
            ChangeKind::Edit => {
                for entry in self.redo.drain(..) {
                    self.bytes -= Self::charge(&entry);
                }
                self.push(previous, false);
            }
            ChangeKind::Undo => {
                if let Some(entry) = self.undo.pop_back() {
                    self.bytes -= Self::charge(&entry);
                }
                self.push(previous, true);
            }
            ChangeKind::Redo => {
                if let Some(entry) = self.redo.pop_back() {
                    self.bytes -= Self::charge(&entry);
                }
                self.push(previous, false);
            }
        }
    }
}
#[derive(Clone, Copy)]
enum ChangeKind {
    Edit,
    Undo,
    Redo,
}
struct State {
    scope: glory::ui::WeakScope,
    node: Cage<Option<NodeId>>,
    owner: glory::core::InstanceId,
    source: Option<Vary<Rc<RichDocument>>>,
    accepted: Cage<Rc<RichDocument>>,
    selection: Cage<RichSelection>,
    accepted_selection: Cage<RichSelection>,
    composing: Cage<bool>,
    history: Cage<EditorHistory>,
    entries: RefCell<History>,
    pending: RefCell<Option<QueryToken>>,
    changed: Callback<Rc<RichDocument>>,
}
impl State {
    fn current(&self) -> Result<(Scope, NodeId)> {
        let scope = self.scope.upgrade().ok_or(glory::ui::Error::Gone)?;
        let node = self.node.try_get()?.ok_or(glory::ui::Error::Gone)?;
        Ok((scope, node))
    }
    fn publish_history(&self) -> Result {
        let summary = self.entries.borrow().summary();
        self.history.try_revise(|value| *value = summary)?;
        Ok(())
    }
    fn cancel_query(&self) -> Result {
        let token = self.pending.borrow_mut().take();
        if let Some(token) = token
            && let Some(scope) = self.scope.upgrade()
        {
            scope.cancel_request(token)?;
        }
        Ok(())
    }
    fn replace_program(&self, document: Rc<RichDocument>) -> Result {
        document.validate()?;
        self.cancel_query()?;
        let (scope, node) = self.current()?;
        scope.set_rich_document(node, document.clone())?;
        self.accepted.try_revise(|value| *value = document)?;
        self.accepted_selection
            .try_revise(|value| *value = RichSelection::default())?;
        self.entries.borrow_mut().clear();
        self.publish_history()
    }
    fn propose(&self, next: Entry, kind: ChangeKind) -> Result<bool> {
        next.document.validate()?;
        next.document.validate_selection(next.selection)?;
        let (scope, node) = self.current()?;
        if !scope.rich_editor_editable(node)? || self.composing.try_get()? {
            return Ok(false);
        }
        let current = self.accepted.try_get()?;
        let previous = Entry {
            document: current.clone(),
            selection: self.accepted_selection.try_get()?,
        };
        // No graph/history/registry borrow spans an application callback.
        let changes_document = current != next.document;
        if self.source.is_some() && changes_document {
            self.changed.emit(self.owner, next.document.clone());
            if !self.owner.is_alive() {
                return Ok(false);
            }
        }
        let accepted = if let Some(source) = &self.source {
            source.try_get()?
        } else {
            next.document.clone()
        };
        accepted.validate()?;
        let accepted_proposal = accepted == next.document;
        // Refusal still allocates a new accepted revision. The host repairs its
        // physical draft even when the controlled source published no mutation.
        scope.set_rich_document(node, accepted.clone())?;
        if accepted_proposal {
            if current != accepted {
                self.entries.borrow_mut().accept(previous, kind);
            }
            self.accepted.try_revise(|value| *value = accepted)?;
            self.selection.try_revise(|value| *value = next.selection)?;
            self.accepted_selection
                .try_revise(|value| *value = next.selection)?;
            self.publish_history()?;
            if scope.capabilities()?.input {
                scope.action(Action::Focus(node))?;
                scope.set_rich_selection(node, next.selection)?;
            }
            if self.source.is_none() && changes_document {
                self.changed.emit(self.owner, next.document);
            }
        } else if accepted != current {
            self.accepted.try_revise(|value| *value = accepted)?;
            self.entries.borrow_mut().clear();
            self.publish_history()?;
        }
        Ok(accepted_proposal)
    }
    fn history_move(&self, redo: bool) -> Result<bool> {
        self.current()?;
        let next = {
            let entries = self.entries.borrow();
            if redo {
                entries.redo.back().cloned()
            } else {
                entries.undo.back().cloned()
            }
        };
        if let Some(next) = next {
            self.propose(
                next,
                if redo {
                    ChangeKind::Redo
                } else {
                    ChangeKind::Undo
                },
            )
        } else {
            Ok(false)
        }
    }
}
/// A weak, Owner-scoped controller. Formatting reads the actual host selection;
/// explicit `_at` methods accept a validated finite program range.
#[derive(Clone)]
pub struct EditorController(Rc<State>);
impl EditorController {
    pub fn document(&self) -> Vary<Rc<RichDocument>> {
        self.0.accepted.into()
    }
    pub fn selection(&self) -> Vary<RichSelection> {
        self.0.selection.into()
    }
    pub fn composing(&self) -> Vary<bool> {
        self.0.composing.into()
    }
    pub fn history(&self) -> Vary<EditorHistory> {
        self.0.history.into()
    }
    pub fn format_at(&self, selection: RichSelection, format: RichFormat) -> Result<bool> {
        let document = self.0.accepted.try_get()?;
        let document = Rc::new(document.format(selection, format)?);
        self.0.propose(
            Entry {
                document,
                selection,
            },
            ChangeKind::Edit,
        )
    }
    pub fn insert_at(
        &self,
        selection: RichSelection,
        text: &str,
        marks: RichMarks,
    ) -> Result<bool> {
        let current = self.0.accepted.try_get()?;
        let (document, selection) = current.replace(selection, text, marks)?;
        self.0.propose(
            Entry {
                document: Rc::new(document),
                selection,
            },
            ChangeKind::Edit,
        )
    }
    pub fn undo(&self) -> Result<bool> {
        self.0.history_move(false)
    }
    pub fn redo(&self) -> Result<bool> {
        self.0.history_move(true)
    }
    /// Silent program replacement; it clears history and cancels pending commands.
    /// In controlled mode, replace the source instead of bypassing it.
    pub fn reset(&self, document: Rc<RichDocument>) -> Result {
        if self.0.source.is_some() {
            return Err(render::Error::Unsupported(
                "controlled editor reset requires updating its source",
            )
            .into());
        }
        self.0.replace_program(document)
    }
    pub fn format(&self, format: RichFormat) -> Result<QueryToken> {
        format.validate()?;
        self.query(move |snapshot| {
            Ok((
                Rc::new(snapshot.document.format(snapshot.selection, format)?),
                snapshot.selection,
            ))
        })
    }
    pub fn insert(&self, text: impl Into<String>, marks: RichMarks) -> Result<QueryToken> {
        let text = text.into();
        if text.len() > render::MAX_RICH_TEXT_BYTES {
            return Err(
                render::Error::InvalidProps("rich insertion exceeds text budget").into(),
            );
        }
        marks.validate()?;
        self.query(move |snapshot| {
            let (document, selection) =
                snapshot
                    .document
                    .replace(snapshot.selection, &text, marks)?;
            Ok((Rc::new(document), selection))
        })
    }
    fn query(
        &self,
        operation: impl FnOnce(RichSnapshot) -> Result<(Rc<RichDocument>, RichSelection)> + 'static,
    ) -> Result<QueryToken> {
        self.0.cancel_query()?;
        let (scope, node) = self.0.current()?;
        if !scope.rich_editor_editable(node)? || self.0.composing.try_get()? {
            return Err(render::Error::Unsupported("editor is not editable").into());
        }
        let state = self.0.clone();
        let token = scope.request(Query::RichSelection(node), move |reply| {
            state.pending.borrow_mut().take();
            let result = (|| -> Result {
                let QueryValue::RichSelection(snapshot) = reply? else {
                    return Err(
                        render::Error::InvalidResponse("rich selection response kind").into(),
                    );
                };
                let (scope, current) = state.current()?;
                if current != node
                    || snapshot.composing
                    || scope.rich_editor_value(node)?.1 != snapshot.revision
                {
                    return Ok(());
                }
                let (document, selection) = operation(*snapshot)?;
                state.propose(
                    Entry {
                        document,
                        selection,
                    },
                    ChangeKind::Edit,
                )?;
                Ok(())
            })();
            if let Err(error) = result
                && let Some(scope) = state.scope.upgrade()
            {
                scope.report_error(error);
            }
        })?;
        *self.0.pending.borrow_mut() = Some(token);
        Ok(token)
    }
    fn toggle(&self, command: RichCommand) -> Result {
        match command {
            RichCommand::Undo => {
                self.undo()?;
            }
            RichCommand::Redo => {
                self.redo()?;
            }
            _ => {
                self.query(move |snapshot| {
                    let (start, end) = snapshot.selection.ordered();
                    let mut marked = true;
                    let mut found = false;
                    for (index, block) in snapshot
                        .document
                        .blocks
                        .iter()
                        .enumerate()
                        .take(end.block + 1)
                        .skip(start.block)
                    {
                        let mut position = 0;
                        for run in &block.runs {
                            let length = run.text.encode_utf16().count();
                            let first = if index == start.block {
                                start.offset
                            } else {
                                0
                            };
                            let last = if index == end.block {
                                end.offset
                            } else {
                                block.utf16_len()
                            };
                            if position < last && position + length > first {
                                found = true;
                                marked &= match command {
                                    RichCommand::Bold => run.marks.bold,
                                    RichCommand::Italic => run.marks.italic,
                                    _ => run.marks.code,
                                };
                            }
                            position += length;
                        }
                    }
                    let value = !(found && marked);
                    let format = match command {
                        RichCommand::Bold => RichFormat::Bold(value),
                        RichCommand::Italic => RichFormat::Italic(value),
                        _ => RichFormat::Code(value),
                    };
                    Ok((
                        Rc::new(snapshot.document.format(snapshot.selection, format)?),
                        snapshot.selection,
                    ))
                })?;
            }
        }
        Ok(())
    }
}
pub struct RichTextEditor {
    common: Common,
    source: Option<Vary<Rc<RichDocument>>>,
    initial: Rc<RichDocument>,
    readonly: Vary<bool>,
    toolbar: bool,
    controller_ref: Option<Cage<Option<EditorController>>>,
    changed: Option<Box<dyn FnMut(Rc<RichDocument>)>>,
}
pub fn rich_text_editor() -> RichTextEditor {
    RichTextEditor {
        common: Common {
            name: Some("Rich text editor".into()),
            focus: render::FocusMode::TabStop,
            ..Default::default()
        },
        source: None,
        initial: Rc::new(RichDocument::default()),
        readonly: false.into(),
        toolbar: true,
        controller_ref: None,
        changed: None,
    }
}
impl RichTextEditor {
    common_methods!();
    pub fn document(mut self, source: impl Into<Vary<Rc<RichDocument>>>) -> Self {
        self.source = Some(source.into());
        self
    }
    pub fn default_document(mut self, value: Rc<RichDocument>) -> Self {
        self.initial = value;
        self
    }
    pub fn readonly(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn toolbar(mut self, value: bool) -> Self {
        self.toolbar = value;
        self
    }
    pub fn controller_ref(mut self, value: Cage<Option<EditorController>>) -> Self {
        self.controller_ref = Some(value);
        self
    }
    pub fn on_document_change(
        mut self,
        callback: impl FnMut(Rc<RichDocument>) + 'static,
    ) -> Self {
        self.changed = Some(Box::new(callback));
        self
    }
}
impl Widget for RichTextEditor {
    fn build(&mut self, scope: &Scope) -> Result {
        let initial = if let Some(source) = &self.source {
            source.try_get()?
        } else {
            self.initial.clone()
        };
        initial.validate()?;
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let state = Rc::new(State {
            scope: scope.downgrade(),
            node: reference,
            owner: scope.id(),
            source: self.source.clone(),
            accepted: scope.cage(initial.clone()),
            selection: scope.cage(RichSelection::default()),
            accepted_selection: scope.cage(RichSelection::default()),
            composing: scope.cage(false),
            history: scope.cage(EditorHistory::default()),
            entries: RefCell::new(History::default()),
            pending: RefCell::new(None),
            changed: Callback::new(self.changed.take()),
        });
        let controller = EditorController(state.clone());
        let output = self.controller_ref;
        let cleaned = state.clone();
        scope.on_cleanup(move || {
            cleaned.changed.clear();
            cleaned.pending.borrow_mut().take();
            cleaned.entries.borrow_mut().clear();
            if let Some(output) = output {
                let _ = output.try_revise(|value| *value = None);
            }
        })?;
        let parent = scope.cage(None);
        glory::ui::view()
            .class("sapid-rich-editor-shell")
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(8.),
            )
            .node_ref(parent)
            .show_in(scope)?;
        let parent = parent.try_get()?.ok_or(glory::ui::Error::Gone)?;
        let content = scope.child_in(parent)?;
        if self.toolbar {
            let history = state.history;
            let composing = state.composing;
            let enabled = self.common.enabled.clone();
            let readonly = self.readonly.clone();
            let available =
                scope.bond(move || enabled.get() && !readonly.get() && !composing.get());
            let mut toolbar = crate::button_group()
                .toolbar(true)
                .name("Rich text formatting")
                .theme(self.common.theme.clone());
            if self.common.unstyled {
                toolbar = toolbar.unstyled();
            }
            for (label, command) in [
                ("Bold", RichCommand::Bold),
                ("Italic", RichCommand::Italic),
                ("Code", RichCommand::Code),
                ("Undo", RichCommand::Undo),
                ("Redo", RichCommand::Redo),
            ] {
                let current = controller.clone();
                let weak = scope.downgrade();
                let active = scope.bond(move || {
                    available.get()
                        && match command {
                            RichCommand::Undo => history.get().undo > 0,
                            RichCommand::Redo => history.get().redo > 0,
                            _ => true,
                        }
                });
                let mut button = crate::button()
                    .label(label)
                    .theme(self.common.theme.clone())
                    .enabled(active)
                    .on_press(move || {
                        if let Err(error) = current.toggle(command)
                            && let Some(scope) = weak.upgrade()
                        {
                            scope.report_error(error);
                        }
                    });
                if self.common.unstyled {
                    button = button.unstyled();
                }
                toolbar = toolbar.button(button);
            }
            for (label, kind) in [
                ("Paragraph", render::RichBlockKind::Paragraph),
                ("Heading", render::RichBlockKind::Heading(2)),
                ("Quote", render::RichBlockKind::Quote),
            ] {
                let current = controller.clone();
                let weak = scope.downgrade();
                let mut button = crate::button()
                    .label(label)
                    .theme(self.common.theme.clone())
                    .enabled(available)
                    .on_press(move || {
                        if let Err(error) = current.format(RichFormat::Block(kind))
                            && let Some(scope) = weak.upgrade()
                        {
                            scope.report_error(error);
                        }
                    });
                if self.common.unstyled {
                    button = button.unstyled();
                }
                toolbar = toolbar.button(button);
            }
            toolbar.show_in(&content)?;
        }
        let mut base = self.common.theme.control(self.common.size);
        base.height = Some(Length::Auto);
        let (classes, custom) = self
            .common
            .appearance(scope, "sapid-rich-text-editor", base);
        let geometry = Style::new()
            .width(Length::Percent(100.))
            .min_width(Length::Px(0.))
            .min_height(Length::Px(120.))
            .overflow_x(Overflow::Auto)
            .overflow_y(Overflow::Auto)
            .white_space(WhiteSpace::PreWrap);
        let style = scope.bond(move || merge(geometry.clone(), custom.get()));
        let readonly = self.readonly.clone();
        let semantics = scope.bond(move || {
            Some(Box::new(Semantics {
                readonly: readonly.get(),
                ..Default::default()
            }))
        });
        let mut editor = glory::ui::rich_editor()
            .document(initial)
            .class(classes)
            .style(style)
            .semantics(semantics)
            .enabled(self.common.enabled.clone())
            .focus(self.common.focus)
            .node_ref(reference);
        if let Some(name) = self.common.name.take() {
            editor = editor.name(name);
        }
        let mut generic = self
            .common
            .handlers
            .remove(&EventKind::RichEdit)
            .unwrap_or_default();
        let changed = state.clone();
        editor = editor.on(EventKind::RichEdit, move |event| {
            let result = (|| -> Result {
                if let Event::RichEdit(snapshot) = &event {
                    changed
                        .composing
                        .try_revise(|value| *value = snapshot.composing)?;
                    changed
                        .selection
                        .try_revise(|value| *value = snapshot.selection)?;
                    if !snapshot.composing {
                        changed.propose(
                            Entry {
                                document: snapshot.document.clone(),
                                selection: snapshot.selection,
                            },
                            ChangeKind::Edit,
                        )?;
                    }
                }
                Ok(())
            })();
            if let Err(error) = result {
                if let Some(scope) = changed.scope.upgrade() {
                    scope.report_error(error);
                }
                return;
            }
            observe(&mut generic, changed.owner, event);
        });
        let mut generic = self
            .common
            .handlers
            .remove(&EventKind::RichSelection)
            .unwrap_or_default();
        let selected = state.clone();
        editor = editor.on(EventKind::RichSelection, move |event| {
            if let Event::RichSelection(snapshot) = &event {
                let result = (|| -> Result {
                    selected
                        .selection
                        .try_revise(|value| *value = snapshot.selection)?;
                    if snapshot.document == selected.accepted.try_get()? && !snapshot.composing
                    {
                        selected
                            .accepted_selection
                            .try_revise(|value| *value = snapshot.selection)?;
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    if let Some(scope) = selected.scope.upgrade() {
                        scope.report_error(error);
                    }
                    return;
                }
            }
            observe(&mut generic, selected.owner, event);
        });
        let mut generic = self
            .common
            .handlers
            .remove(&EventKind::Composition)
            .unwrap_or_default();
        let composing = state.clone();
        editor = editor.on(EventKind::Composition, move |event| {
            if let Event::Composition { active } = event
                && let Err(error) = composing.composing.try_revise(|value| *value = active)
            {
                if let Some(scope) = composing.scope.upgrade() {
                    scope.report_error(error);
                }
                return;
            }
            observe(&mut generic, composing.owner, event);
        });
        let mut generic = self
            .common
            .handlers
            .remove(&EventKind::RichCommand)
            .unwrap_or_default();
        let command = controller.clone();
        editor = editor.on(EventKind::RichCommand, move |event| {
            if let Event::RichCommand { command: kind, .. } = event
                && let Err(error) = command.toggle(kind)
            {
                if let Some(scope) = command.0.scope.upgrade() {
                    scope.report_error(error);
                }
                return;
            }
            observe(&mut generic, command.0.owner, event);
        });
        for (kind, mut generic) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            editor = editor.on(kind, move |event| observe(&mut generic, owner, event));
        }
        editor.show_in(&content)?;
        if let Some(source) = self.source.clone() {
            let current = state.clone();
            scope.structural(move || {
                let result = (|| -> Result {
                    let next = source.try_get()?;
                    next.validate()?;
                    if next != glory::core::untracked(|| current.accepted.try_get())? {
                        current.replace_program(next)?;
                    }
                    Ok(())
                })();
                if let Err(error) = result
                    && let Some(scope) = current.scope.upgrade()
                {
                    scope.report_error(error);
                }
            });
        }
        if let Some(output) = output {
            output.try_revise(|value| *value = Some(controller))?;
        }
        scope.retain(content)?;
        scope.retain(state)?;
        Ok(())
    }
}

impl Default for RichTextEditor {
    fn default() -> Self {
        rich_text_editor()
    }
}
