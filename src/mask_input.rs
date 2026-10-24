//! A bounded ASCII mask over one real editor. The public value contains slots only.
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback, Observers};
use crate::field::{FieldControl, control_metadata};
use crate::mask_model::{Direction, Draft, Intent, MaskPattern, MaskRejection, Selection};
use crate::state::{Control, Dirty, Value};
use glory::render::{
    Event, EventKind, InputEdit, InputEditCapture, InputEditIntent, InputEditRejection,
    SelectionDirection, Semantics, TextSelection,
};
use glory::style::Length;
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Lotus, Scope};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

fn invalid() -> glory::ui::Error {
    glory::render::Error::InvalidProps("invalid bounded ASCII masked input").into()
}
fn selection(value: TextSelection) -> Selection {
    Selection {
        start: value.start,
        end: value.end,
        direction: match value.direction {
            SelectionDirection::None => Direction::None,
            SelectionDirection::Forward => Direction::Forward,
            SelectionDirection::Backward => Direction::Backward,
        },
    }
}
fn host_selection(value: Selection) -> TextSelection {
    TextSelection {
        start: value.start,
        end: value.end,
        direction: match value.direction {
            Direction::None => SelectionDirection::None,
            Direction::Forward => SelectionDirection::Forward,
            Direction::Backward => SelectionDirection::Backward,
        },
    }
}
#[derive(Clone, PartialEq)]
struct Snapshot {
    value: String,
    enabled: bool,
    metadata: Semantics,
}
impl Snapshot {
    fn writable(&self) -> bool {
        self.enabled && !self.metadata.readonly
    }
}
struct Pending {
    physical: String,
    proposal: String,
    selection: Selection,
}
struct Controller {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    live: Cell<bool>,
    pattern: MaskPattern,
    control: Control<String>,
    accepted: Cage<Snapshot>,
    dirty: Dirty,
    changed: Callback<String>,
    committed: Callback<String>,
    rejected: Callback<MaskRejection>,
    draft: RefCell<Draft>,
    pending: RefCell<Option<Pending>>,
    composing: Cell<bool>,
    ended: Cell<bool>,
    echo: RefCell<Option<(u64, String)>>,
}
impl Controller {
    fn ensure(&self) -> Result {
        if self.live.get() && self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn current(&self) -> Result<String> {
        Ok(self
            .control
            .source
            .try_with(|v| self.pattern.validate_raw(v).ok().map(|()| v.clone()))?
            .unwrap_or(self.accepted.try_with(|v| v.value.clone())?))
    }
    fn reject(&self, reason: MaskRejection) -> Result {
        self.dirty.touch()?;
        self.rejected.emit(self.owner, reason);
        Ok(())
    }
    fn edit(&self, edit: &InputEdit) -> Result {
        self.ensure()?;
        if !self.accepted.try_with(Snapshot::writable)? {
            return Ok(());
        }
        let InputEdit::Value {
            value,
            revision,
            selection: after,
            before,
            intent,
            composing,
        } = edit
        else {
            return self.reject(match edit {
                InputEdit::Rejected {
                    reason: InputEditRejection::SelectionUnavailable,
                    ..
                } => MaskRejection::InvalidSelection,
                _ => MaskRejection::InputTooLong,
            });
        };
        if *composing {
            self.composing.set(true);
            // The host owns the preedit. No business value, display mapping or
            // selection action is published until its terminal value arrives.
            return Ok(());
        }
        let echo = self.echo.borrow_mut().take();
        if echo
            .as_ref()
            .is_some_and(|(r, v)| r == revision && v == value)
        {
            return Ok(());
        }
        let ended = self.ended.replace(false) || self.composing.replace(false);
        self.composing.set(false);
        let current = self.current()?;
        let pending_proposal = self.pending.borrow().as_ref().map(|p| p.proposal.clone());
        if pending_proposal.as_ref().is_some_and(|p| p != &current) {
            // A controlled refusal or an external write invalidates the current
            // physical mapping; interpret this event as a whole replacement.
            *self.draft.borrow_mut() =
                Draft::new(&self.pattern, &current).map_err(|_| invalid())?;
        }
        let before = if ended || pending_proposal.as_ref().is_some_and(|p| p != &current) {
            None
        } else {
            before.map(selection)
        };
        let result = self.draft.borrow_mut().apply(
            &self.pattern,
            value,
            selection(*after),
            before,
            match intent {
                InputEditIntent::Insert => Intent::Insert,
                InputEditIntent::DeleteBackward => Intent::Backward,
                InputEditIntent::DeleteForward => Intent::Forward,
                InputEditIntent::Replace => Intent::Replace,
            },
        );
        let proposal = match result {
            Ok(value) => value,
            Err(reason) => return self.reject(reason),
        };
        debug_assert!(
            proposal
                .formatted
                .raw_index(proposal.selection.end)
                .is_some()
        );
        let raw = proposal.raw;
        self.pending.borrow_mut().replace(Pending {
            physical: value.clone(),
            proposal: raw.clone(),
            selection: proposal.selection,
        });
        self.control.accept(&raw)?;
        self.dirty.touch()?;
        self.changed.emit(self.owner, raw);
        if ended && self.ensure().is_ok() {
            self.echo.borrow_mut().replace((*revision, value.clone()));
        }
        Ok(())
    }
    fn commit(&self, value: &str) -> Result {
        self.ensure()?;
        if self.composing.get() || !self.accepted.try_with(Snapshot::writable)? {
            return Ok(());
        }
        let pending = self
            .pending
            .borrow()
            .as_ref()
            .filter(|p| p.physical == value)
            .map(|_| self.draft.borrow().raw().to_owned());
        let raw = match pending
            .map(Ok)
            .unwrap_or_else(|| self.pattern.parse_display(value))
        {
            Ok(raw) => raw,
            Err(reason) => return self.reject(reason),
        };
        self.control.accept(&raw)?;
        self.dirty.touch()?;
        self.committed.emit(self.owner, raw);
        Ok(())
    }
    fn event(&self, event: &Event) -> Result {
        match event {
            Event::InputEdit(edit) => self.edit(edit),
            Event::Commit { value, .. } => self.commit(value),
            Event::Composition { active } => {
                self.composing.set(*active);
                self.ended.set(!active);
                if *active {
                    self.echo.borrow_mut().take();
                } else {
                    // A superseded terminal edit may be discarded by the
                    // revision gate. Still reconcile the accepted program value.
                    self.dirty.touch()?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    fn report(&self, result: Result) {
        if let Err(error) = result
            && let Some(scope) = self.weak.upgrade()
        {
            scope.report_error(error);
        }
    }
    fn retire(&self) {
        self.live.set(false);
        self.changed.clear();
        self.committed.clear();
        self.rejected.clear();
        self.pending.borrow_mut().take();
        self.echo.borrow_mut().take();
    }
}
pub struct MaskInput {
    common: Common,
    pattern: MaskPattern,
    value: Value<String>,
    field: Option<FieldControl>,
    required: Lotus<bool>,
    readonly: Lotus<bool>,
    invalid: Lotus<bool>,
    form_name: Option<Lotus<String>>,
    changed: Option<Box<dyn FnMut(String)>>,
    committed: Option<Box<dyn FnMut(String)>>,
    rejected: Option<Box<dyn FnMut(MaskRejection)>>,
}
/// The pattern is compiled and bounded explicitly; values contain raw slots.
pub fn mask_input(pattern: MaskPattern) -> MaskInput {
    MaskInput {
        common: Common::default(),
        pattern,
        value: Value::default(),
        field: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
        form_name: None,
        changed: None,
        committed: None,
        rejected: None,
    }
}
impl MaskInput {
    common_methods!();
    pub fn value(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.value.default = value.into();
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
    pub fn form_name(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.form_name = Some(value.into());
        self
    }
    pub fn field(mut self, value: FieldControl) -> Self {
        self.common.theme = value.theme.clone();
        self.common.size = value.size;
        self.common.unstyled = value.unstyled;
        self.field = Some(value);
        self
    }
    pub fn on_value_change(mut self, value: impl FnMut(String) + 'static) -> Self {
        self.changed = Some(Box::new(value));
        self
    }
    pub fn on_commit(mut self, value: impl FnMut(String) + 'static) -> Self {
        self.committed = Some(Box::new(value));
        self
    }
    pub fn on_reject(mut self, value: impl FnMut(MaskRejection) + 'static) -> Self {
        self.rejected = Some(Box::new(value));
        self
    }
}
impl Widget for MaskInput {
    fn build(&mut self, scope: &Scope) -> Result {
        let pattern = self.pattern.clone();
        if let Some(value) = &self.value.source {
            value
                .try_with(|v| pattern.validate_raw(v))?
                .map_err(|_| invalid())?;
        } else {
            pattern
                .validate_raw(&self.value.default)
                .map_err(|_| invalid())?;
        }
        let control = std::mem::take(&mut self.value).mount(scope);
        let p = pattern.clone();
        let value = composite::validated(scope, &control.source, move |v| {
            p.validate_raw(v).map_err(|_| invalid())
        })?;
        let enabled = self
            .field
            .as_ref()
            .map(|f| f.enabled(scope, self.common.enabled.clone()))
            .unwrap_or_else(|| self.common.enabled.clone());
        let metadata = control_metadata(
            scope,
            self.field.as_ref(),
            self.common.name.is_some(),
            self.required.clone(),
            self.readonly.clone(),
            self.invalid.clone(),
        );
        let form_name = composite::form_name(scope, &self.form_name)?;
        let source = scope.bond(move || {
            let mut metadata = metadata.get().map(|s| *s).unwrap_or_default();
            metadata.input_edit = Some(InputEditCapture {
                max_bytes: crate::mask_model::MAX_MASK_INPUT_BYTES,
            });
            if let Some(name) = form_name {
                metadata.form_control = Some(glory::render::FormControl {
                    name: name.get(),
                    value: None,
                });
            }
            Snapshot {
                value: value.get(),
                enabled: enabled.get(),
                metadata,
            }
        });
        let p = pattern.clone();
        let accepted = composite::validated(scope, &source.into(), move |v: &Snapshot| {
            p.validate_raw(&v.value).map_err(|_| invalid())
        })?;
        let initial = accepted.try_with(|v| v.value.clone())?;
        let display = pattern
            .format(&initial)
            .map_err(|_| invalid())?
            .display()
            .to_owned();
        let dirty = Dirty::new(scope);
        let state = Rc::new(Controller {
            owner: scope.id(),
            weak: scope.downgrade(),
            live: Cell::new(true),
            draft: RefCell::new(Draft::new(&pattern, &initial).map_err(|_| invalid())?),
            pattern,
            control,
            accepted,
            dirty: dirty.clone(),
            changed: Callback::new(self.changed.take()),
            committed: Callback::new(self.committed.take()),
            rejected: Callback::new(self.rejected.take()),
            pending: RefCell::new(None),
            composing: Cell::new(false),
            ended: Cell::new(false),
            echo: RefCell::new(None),
        });
        let cleanup = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(s) = cleanup.upgrade() {
                s.retire();
            }
        })?;
        let base = self
            .common
            .theme
            .control(self.common.size)
            .width(Length::Px(240.));
        let (class, appearance) = self.common.appearance(scope, "sapid-mask-input", base);
        // Unstyled removes paint, not the usable default editor dimensions.
        let style = scope.bond(move || {
            let mut s = appearance.get();
            s.width.get_or_insert(Length::Px(240.));
            s.min_height.get_or_insert(Length::Px(36.));
            s
        });
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let mut input = glory::ui::input()
            .class(class)
            .style(style)
            .value(display.clone())
            .focus(self.common.focus)
            .enabled(scope.bond(move || accepted.with(|v| v.enabled)))
            .semantics(
                scope.bond(move || composite::metadata(accepted.with(|v| v.metadata.clone()))),
            )
            .node_ref(reference);
        if let Some(name) = self.common.name.take() {
            input = input.name(name);
        }
        for kind in [
            EventKind::InputEdit,
            EventKind::Commit,
            EventKind::Composition,
        ] {
            let s = state.clone();
            let observers =
                Observers::new(self.common.handlers.remove(&kind).unwrap_or_default());
            let cleanup = observers.clone();
            scope.on_cleanup(move || cleanup.clear())?;
            input = input.on(kind, move |event| {
                if s.ensure().is_ok() {
                    s.report(s.event(&event));
                    observers.emit(s.owner, event);
                }
            });
        }
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            input = input.on(kind, move |event| {
                crate::appearance::observe(&mut handlers, owner, event)
            });
        }
        input.show_in(scope)?;
        let node = reference.try_get()?.ok_or(glory::ui::Error::Gone)?;
        let weak = Rc::downgrade(&state);
        let owner = scope.downgrade();
        let mut previous = display;
        scope.effect(move || {
            let (Some(s), Some(scope)) = (weak.upgrade(), owner.upgrade()) else {
                return;
            };
            let result = (|| -> Result {
                let edited = dirty.take();
                let raw = accepted.try_with(|v| v.value.clone())?;
                let formatted = s.pattern.format(&raw).map_err(|_| invalid())?;
                let text = formatted.display().to_owned();
                if s.composing.get() {
                    if text != previous {
                        // Publish the program revision while the host retains
                        // preedit; the old terminal cannot overwrite this value.
                        scope.set_input_value(node, text.clone())?;
                        previous = text;
                    }
                    return Ok(());
                }
                if edited || text != previous {
                    let pending = s.pending.borrow_mut().take();
                    let selection = pending.as_ref().map(|p| {
                        if p.proposal == raw {
                            p.selection
                        } else {
                            Selection {
                                start: p.selection.start.min(text.len()),
                                end: p.selection.end.min(text.len()),
                                direction: p.selection.direction,
                            }
                        }
                    });
                    let needs_selection = pending.as_ref().is_some_and(|p| p.physical != text);
                    scope.set_input_value(node, text.clone())?;
                    if needs_selection && let Some(selection) = selection {
                        scope.set_input_selection(node, host_selection(selection))?;
                    }
                    *s.draft.borrow_mut() =
                        Draft::new(&s.pattern, &raw).map_err(|_| invalid())?;
                    previous = text;
                }
                Ok(())
            })();
            s.report(result);
        });
        scope.retain(state)?;
        Ok(())
    }
}
