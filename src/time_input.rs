//! Editable wall-clock drafts over one bounded actual Input.
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback, Observers};
use crate::field::control_metadata;
use crate::state::{Control, Dirty, Value};
use crate::time_model::*;
use crate::{FieldControl, LocalTime};
use glory::render::{
    Event, EventKind, InputEdit, InputEditCapture, InputEditRejection, SemanticId, Semantics,
};
use glory::style::{Display, FlexDirection, Length, Style};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Lotus, Scope};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeInputDraft {
    pub text: String,
    pub status: TimeDraftStatus,
}
#[derive(Clone, PartialEq)]
struct Snapshot {
    value: Option<LocalTime>,
    bounds: TimeBounds,
    enabled: bool,
    metadata: Semantics,
}
struct Pending {
    text: String,
    status: TimeDraftStatus,
    proposal: Option<Option<LocalTime>>,
    baseline: Option<LocalTime>,
    commit: bool,
}
struct Controller {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    live: Cell<bool>,
    control: Control<Option<LocalTime>>,
    accepted: Cage<Snapshot>,
    draft: Cage<TimeInputDraft>,
    dirty: Dirty,
    format: TimeFormat,
    pending: RefCell<Option<Pending>>,
    composing: Cell<bool>,
    composition_base: Cell<Option<Option<LocalTime>>>,
    ended: Cell<bool>,
    echo: RefCell<Option<(u64, String)>>,
    changed: Callback<Option<LocalTime>>,
    committed: Callback<Option<LocalTime>>,
    draft_changed: Callback<TimeInputDraft>,
    rejected: Callback<TimeDraftStatus>,
}
fn invalid() -> glory::ui::Error {
    glory::render::Error::InvalidProps("invalid bounded wall-clock time input").into()
}
fn text(value: Option<LocalTime>, format: TimeFormat) -> String {
    value
        .map(|v| format.format(v).expect("validated time precision"))
        .unwrap_or_default()
}
fn status(value: Option<LocalTime>) -> TimeDraftStatus {
    value.map_or(TimeDraftStatus::Empty, TimeDraftStatus::Valid)
}
impl Controller {
    fn ensure(&self) -> Result {
        if self.live.get() && self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn writable(&self) -> Result<bool> {
        Ok(self
            .accepted
            .try_with(|s| s.enabled && !s.metadata.readonly)?)
    }
    fn current(&self) -> Result<Option<LocalTime>> {
        let snapshot = self.accepted.try_get()?;
        let value = self.control.source.try_get()?;
        Ok(
            if value
                .is_none_or(|d| snapshot.bounds.contains(d) && self.format.validate(d).is_ok())
            {
                value
            } else {
                snapshot.value
            },
        )
    }
    fn classify(&self, value: &str) -> Result<TimeDraftStatus> {
        let bounds = self.accepted.try_with(|s| s.bounds)?;
        let parsed = classify_time_draft(value, self.format, bounds);
        self.ensure()?;
        Ok(parsed)
    }
    fn reject(&self, reason: TimeDraftStatus) -> Result {
        self.rejected.emit(self.owner, reason);
        self.ensure()?;
        let value = self.current()?;
        let draft = TimeInputDraft {
            text: text(value, self.format),
            status: reason,
        };
        self.draft.try_revise(|d| *d = draft.clone())?;
        self.draft_changed.emit(self.owner, draft);
        self.ensure()?;
        self.pending.borrow_mut().replace(Pending {
            text: text(value, self.format),
            status: reason,
            proposal: Some(value),
            baseline: value,
            commit: false,
        });
        self.dirty.touch()
    }
    fn edit(&self, edit: &InputEdit) -> Result {
        self.ensure()?;
        if !self.writable()? {
            return Ok(());
        }
        let InputEdit::Value {
            value,
            revision,
            composing,
            ..
        } = edit
        else {
            return self.reject(match edit {
                InputEdit::Rejected {
                    reason: InputEditRejection::TooLong,
                    ..
                } => TimeDraftStatus::TooLong,
                _ => TimeDraftStatus::Invalid,
            });
        };
        if *composing {
            if self.composition_base.get().is_none() {
                self.composition_base.set(Some(self.current()?));
            }
            self.composing.set(true);
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
        let baseline = self.current()?;
        let parsed = self.classify(value)?;
        let proposal = parsed.proposal();
        let draft = TimeInputDraft {
            text: value.clone(),
            status: parsed,
        };
        self.draft.try_revise(|d| *d = draft.clone())?;
        self.pending.borrow_mut().replace(Pending {
            text: value.clone(),
            status: parsed,
            proposal,
            baseline,
            commit: false,
        });
        self.draft_changed.emit(self.owner, draft);
        self.ensure()?;
        if let Some(next) = proposal {
            self.control.accept(&next)?;
            self.changed.emit(self.owner, next);
            self.ensure()?;
        }
        self.dirty.touch()?;
        if ended {
            self.echo.borrow_mut().replace((*revision, value.clone()));
        }
        Ok(())
    }
    fn commit(&self, value: &str) -> Result {
        self.ensure()?;
        if self.composing.get() || !self.writable()? {
            return Ok(());
        }
        if value.len() > MAX_TIME_DRAFT_BYTES {
            return self.reject(TimeDraftStatus::TooLong);
        }
        let draft = self.draft.try_get()?;
        let parsed = if draft.text == value {
            draft.status
        } else {
            self.classify(value)?
        };
        let Some(next) = parsed.proposal() else {
            self.rejected.emit(self.owner, parsed);
            self.ensure()?;
            self.pending.borrow_mut().replace(Pending {
                text: value.to_owned(),
                status: parsed,
                proposal: None,
                baseline: self.current()?,
                commit: false,
            });
            return self.dirty.touch();
        };
        self.control.accept(&next)?;
        self.pending.borrow_mut().replace(Pending {
            text: value.to_owned(),
            status: parsed,
            proposal: Some(next),
            baseline: self.current()?,
            commit: true,
        });
        self.committed.emit(self.owner, next);
        self.ensure()?;
        self.dirty.touch()
    }
    fn event(&self, event: &Event) -> Result {
        match event {
            Event::InputEdit(edit) => self.edit(edit),
            Event::Commit { value, .. } => self.commit(value),
            Event::Composition { active } => {
                self.composing.set(*active);
                self.ended.set(!active);
                if *active {
                    self.composition_base.set(Some(self.current()?));
                } else {
                    self.composition_base.set(None);
                }
                if *active {
                    self.echo.borrow_mut().take();
                } else {
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
        self.composition_base.set(None);
        self.pending.borrow_mut().take();
        self.echo.borrow_mut().take();
        self.changed.clear();
        self.committed.clear();
        self.draft_changed.clear();
        self.rejected.clear();
    }
}
/// One actual editable Input with exact finite wall-clock display formatting.
pub struct TimeInput {
    common: Common,
    key: String,
    bounds: Lotus<TimeBounds>,
    value: Value<Option<LocalTime>>,
    field: Option<FieldControl>,
    form_name: Option<Lotus<String>>,
    required: Lotus<bool>,
    readonly: Lotus<bool>,
    invalid: Lotus<bool>,
    format: TimeFormat,
    changed: Option<Box<dyn FnMut(Option<LocalTime>)>>,
    committed: Option<Box<dyn FnMut(Option<LocalTime>)>>,
    draft_changed: Option<Box<dyn FnMut(TimeInputDraft)>>,
    rejected: Option<Box<dyn FnMut(TimeDraftStatus)>>,
}
pub fn time_input(key: impl Into<String>) -> TimeInput {
    TimeInput {
        common: Common::default(),
        key: key.into(),
        bounds: TimeBounds::default().into(),
        value: Value::default(),
        field: None,
        form_name: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
        format: TimeFormat::default(),
        changed: None,
        committed: None,
        draft_changed: None,
        rejected: None,
    }
}
impl TimeInput {
    common_methods!();
    pub fn value(mut self, v: impl Into<Lotus<Option<LocalTime>>>) -> Self {
        self.value.source = Some(v.into());
        self
    }
    pub fn default_value(mut self, v: Option<LocalTime>) -> Self {
        self.value.default = v;
        self
    }
    pub fn field(mut self, v: FieldControl) -> Self {
        self.common.theme = v.theme.clone();
        self.common.size = v.size;
        self.common.unstyled = v.unstyled;
        self.field = Some(v);
        self
    }
    pub fn form_name(mut self, v: impl Into<Lotus<String>>) -> Self {
        self.form_name = Some(v.into());
        self
    }
    pub fn required(mut self, v: impl Into<Lotus<bool>>) -> Self {
        self.required = v.into();
        self
    }
    pub fn readonly(mut self, v: impl Into<Lotus<bool>>) -> Self {
        self.readonly = v.into();
        self
    }
    pub fn invalid(mut self, v: impl Into<Lotus<bool>>) -> Self {
        self.invalid = v.into();
        self
    }
    pub fn format(mut self, format: TimeFormat) -> Self {
        self.format = format;
        self
    }
    pub fn bounds(mut self, bounds: impl Into<Lotus<TimeBounds>>) -> Self {
        self.bounds = bounds.into();
        self
    }
    pub fn on_value_change(mut self, v: impl FnMut(Option<LocalTime>) + 'static) -> Self {
        self.changed = Some(Box::new(v));
        self
    }
    pub fn on_commit(mut self, v: impl FnMut(Option<LocalTime>) + 'static) -> Self {
        self.committed = Some(Box::new(v));
        self
    }
    pub fn on_draft_change(mut self, v: impl FnMut(TimeInputDraft) + 'static) -> Self {
        self.draft_changed = Some(Box::new(v));
        self
    }
    pub fn on_reject(mut self, v: impl FnMut(TimeDraftStatus) + 'static) -> Self {
        self.rejected = Some(Box::new(v));
        self
    }
}
impl Widget for TimeInput {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.key.is_empty()
            || self.key.len() > 57
            || !self
                .key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(invalid());
        }
        let error_id = SemanticId::new(format!("{}-time-error", self.key))?;
        let control = std::mem::take(&mut self.value).mount(scope);
        let value = control.source.clone();
        let bounds = self.bounds.clone();
        let format = self.format;
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
                max_bytes: MAX_TIME_DRAFT_BYTES,
            });
            if let Some(name) = form_name {
                metadata.form_control = Some(glory::render::FormControl {
                    name: name.get(),
                    value: None,
                });
            }
            Snapshot {
                value: value.get(),
                bounds: bounds.get(),
                enabled: enabled.get(),
                metadata,
            }
        });
        let accepted = composite::validated(scope, &source.into(), move |s: &Snapshot| {
            if s.value
                .is_none_or(|d| s.bounds.contains(d) && format.validate(d).is_ok())
            {
                Ok(())
            } else {
                Err(invalid())
            }
        })?;
        let initial = accepted.try_with(|s| s.value)?;
        let initial_text = text(initial, format);
        let draft = scope.cage(TimeInputDraft {
            text: initial_text.clone(),
            status: status(initial),
        });
        let dirty = Dirty::new(scope);
        let state = Rc::new(Controller {
            owner: scope.id(),
            weak: scope.downgrade(),
            live: Cell::new(true),
            control,
            accepted,
            draft,
            dirty: dirty.clone(),
            format,
            pending: RefCell::new(None),
            composing: Cell::new(false),
            composition_base: Cell::new(None),
            ended: Cell::new(false),
            echo: RefCell::new(None),
            changed: Callback::new(self.changed.take()),
            committed: Callback::new(self.committed.take()),
            draft_changed: Callback::new(self.draft_changed.take()),
            rejected: Callback::new(self.rejected.take()),
        });
        let cleanup = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(s) = cleanup.upgrade() {
                s.retire();
            }
        })?;
        let wrapper = scope.cage(None);
        glory::ui::view()
            .class("sapid-time-input-group")
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(6.),
            )
            .node_ref(wrapper)
            .show_in(scope)?;
        let content = scope.child_in(wrapper.get().ok_or(glory::ui::Error::Gone)?)?;
        let default_height = self
            .common
            .theme
            .control(self.common.size)
            .min_height
            .unwrap_or(Length::Px(36.));
        let (class, appearance) = self.common.appearance(
            scope,
            "sapid-time-input",
            self.common
                .theme
                .control(self.common.size)
                .width(Length::Px(240.)),
        );
        let style = scope.bond(move || {
            let mut style = appearance.get();
            style.width.get_or_insert(Length::Px(240.));
            style.min_height.get_or_insert(default_height);
            style
        });
        let id = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let error = error_id.clone();
        let mut input = glory::ui::input()
            .class(class)
            .style(style)
            .value(initial_text.clone())
            .node_ref(id)
            .focus(self.common.focus)
            .enabled(scope.bond(move || accepted.with(|s| s.enabled)))
            .semantics(scope.bond(move || {
                let mut meta = accepted.with(|s| s.metadata.clone());
                if draft.with(|d| d.status.is_error()) {
                    meta.invalid = true;
                    if meta.error_message.is_none() {
                        meta.error_message = Some(error.clone());
                    }
                }
                Some(Box::new(meta))
            }));
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
            input = input.on(kind, move |e| {
                if s.ensure().is_ok() {
                    s.report(s.event(&e));
                    observers.emit(s.owner, e);
                }
            });
        }
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            input = input.on(kind, move |e| {
                crate::appearance::observe(&mut handlers, owner, e)
            });
        }
        input.show_in(&content)?;
        let node = id.get().ok_or(glory::ui::Error::Gone)?;
        let owner = scope.downgrade();
        let weak = Rc::downgrade(&state);
        let mut previous = initial;
        scope.effect(move || {
            let (Some(s), Some(scope)) = (weak.upgrade(), owner.upgrade()) else {
                return;
            };
            let result = (|| -> Result {
                let edited = dirty.take();
                let value = accepted.with(|s| s.value);
                let external = value != previous;
                if s.composing.get() {
                    if external && s.composition_base.get() != Some(value) {
                        scope.set_input_value(node, text(value, format))?;
                        previous = value;
                        s.pending.borrow_mut().take();
                    }
                    return Ok(());
                }
                if edited || external {
                    let pending = s.pending.borrow_mut().take();
                    let desired = match pending {
                        Some(p)
                            if !p.commit
                                && (p.proposal == Some(value)
                                    || p.proposal.is_none() && p.baseline == value) =>
                        {
                            TimeInputDraft {
                                text: p.text,
                                status: p.status,
                            }
                        }
                        _ => TimeInputDraft {
                            text: text(value, format),
                            status: status(value),
                        },
                    };
                    scope.set_input_value(node, desired.text.clone())?;
                    if draft.try_with(|old| *old != desired)? {
                        draft.try_revise(|old| *old = desired)?;
                    }
                    previous = value;
                }
                Ok(())
            })();
            s.report(result);
        });
        glory::ui::view()
            .class("sapid-time-input-error")
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    id: Some(error_id.clone()),
                    hidden: !draft.with(|d| d.status.is_error()),
                    ..Default::default()
                }))
            }))
            .fill(glory::ui::text().content(scope.bond(move || {
                draft
                    .with(|d| match d.status {
                        TimeDraftStatus::Invalid => "Enter a valid time",
                        TimeDraftStatus::OutOfBounds(_) => "Time is outside the allowed bounds",
                        TimeDraftStatus::TooLong => "Time input exceeds 32 bytes",
                        _ => "",
                    })
                    .to_owned()
            })))
            .show_in(&content)?;
        scope.retain(state)?;
        scope.retain(content)
    }
}
