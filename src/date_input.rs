//! Editable civil date drafts over one bounded actual Input.
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback, Observers};
use crate::date_draft::*;
use crate::field::control_metadata;
use crate::state::{Control, Dirty, Value};
use crate::{CivilDate, DateSelection, DatesProvider, FieldControl};
use glory::render::{
    Event, EventKind, InputEdit, InputEditCapture, InputEditRejection, SemanticId, Semantics,
};
use glory::style::{Display, FlexDirection, Length, Style};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Scope, Vary};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
type Parser = Rc<dyn Fn(&str) -> DateParse>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateInputDraft {
    pub text: String,
    pub status: DateDraftStatus,
}
#[derive(Clone, PartialEq)]
struct Snapshot {
    value: Option<CivilDate>,
    provider: DatesProvider,
    enabled: bool,
    metadata: Semantics,
}
struct Pending {
    text: String,
    status: DateDraftStatus,
    proposal: Option<Option<CivilDate>>,
    baseline: Option<CivilDate>,
    commit: bool,
}
struct Controller {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    live: Cell<bool>,
    control: Control<Option<CivilDate>>,
    accepted: Cage<Snapshot>,
    draft: Cage<DateInputDraft>,
    dirty: Dirty,
    parser: Parser,
    pending: RefCell<Option<Pending>>,
    composing: Cell<bool>,
    composition_base: Cell<Option<Option<CivilDate>>>,
    ended: Cell<bool>,
    echo: RefCell<Option<(u64, String)>>,
    changed: Callback<Option<CivilDate>>,
    committed: Callback<Option<CivilDate>>,
    draft_changed: Callback<DateInputDraft>,
    rejected: Callback<DateDraftStatus>,
}
fn invalid() -> glory::ui::Error {
    glory::render::Error::InvalidProps("invalid bounded civil date input").into()
}
fn text(value: Option<CivilDate>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}
fn status(value: Option<CivilDate>) -> DateDraftStatus {
    value.map_or(DateDraftStatus::Empty, DateDraftStatus::Valid)
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
    fn current(&self) -> Result<Option<CivilDate>> {
        let snapshot = self.accepted.try_get()?;
        let value = self.control.source.try_get()?;
        Ok(
            if value.is_none_or(|d| snapshot.provider.date_bounds().contains(d)) {
                value
            } else {
                snapshot.value
            },
        )
    }
    fn classify(&self, value: &str) -> Result<DateDraftStatus> {
        let bounds = self.accepted.try_with(|s| s.provider.date_bounds())?;
        let parsed = classify_date_draft(value, bounds, |v| (self.parser)(v));
        self.ensure()?;
        Ok(parsed)
    }
    fn reject(&self, reason: DateDraftStatus) -> Result {
        self.rejected.emit(self.owner, reason);
        self.ensure()?;
        let value = self.current()?;
        let draft = DateInputDraft {
            text: text(value),
            status: reason,
        };
        self.draft.try_revise(|d| *d = draft.clone())?;
        self.draft_changed.emit(self.owner, draft);
        self.ensure()?;
        self.pending.borrow_mut().replace(Pending {
            text: text(value),
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
                } => DateDraftStatus::TooLong,
                _ => DateDraftStatus::Invalid,
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
        let draft = DateInputDraft {
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
        if value.len() > MAX_DATE_DRAFT_BYTES {
            return self.reject(DateDraftStatus::TooLong);
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
    fn choose(&self, next: Option<CivilDate>) -> Result<bool> {
        self.ensure()?;
        if !self.writable()? {
            return Ok(false);
        }
        let baseline = self.current()?;
        let draft = DateInputDraft {
            text: text(next),
            status: status(next),
        };
        self.draft.try_revise(|old| *old = draft.clone())?;
        self.pending.borrow_mut().replace(Pending {
            text: draft.text.clone(),
            status: draft.status,
            proposal: Some(next),
            baseline,
            commit: true,
        });
        self.draft_changed.emit(self.owner, draft);
        self.ensure()?;
        self.control.accept(&next)?;
        self.changed.emit(self.owner, next);
        self.ensure()?;
        self.dirty.touch()?;
        Ok(self.control.source.try_get()? == next)
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
/// Editable exact ISO input; a custom parser may additionally accept bounded spellings.
pub struct DateInput {
    common: Common,
    key: String,
    provider: Vary<DatesProvider>,
    value: Value<Option<CivilDate>>,
    field: Option<FieldControl>,
    form_name: Option<Vary<String>>,
    required: Vary<bool>,
    readonly: Vary<bool>,
    invalid: Vary<bool>,
    parser: Parser,
    picker: bool,
    picker_label: Vary<String>,
    picker_title: Vary<String>,
    picker_close: Vary<String>,
    changed: Option<Box<dyn FnMut(Option<CivilDate>)>>,
    committed: Option<Box<dyn FnMut(Option<CivilDate>)>>,
    draft_changed: Option<Box<dyn FnMut(DateInputDraft)>>,
    rejected: Option<Box<dyn FnMut(DateDraftStatus)>>,
}
pub fn date_input(
    key: impl Into<String>,
    provider: impl Into<Vary<DatesProvider>>,
) -> DateInput {
    DateInput {
        common: Common::default(),
        key: key.into(),
        provider: provider.into(),
        value: Value::default(),
        field: None,
        form_name: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
        parser: Rc::new(parse_iso_date_draft),
        picker: true,
        picker_label: "Choose date".into(),
        picker_title: "Choose date".into(),
        picker_close: "Close date input".into(),
        changed: None,
        committed: None,
        draft_changed: None,
        rejected: None,
    }
}
impl DateInput {
    common_methods!();
    pub fn value(mut self, v: impl Into<Vary<Option<CivilDate>>>) -> Self {
        self.value.source = Some(v.into());
        self
    }
    pub fn default_value(mut self, v: Option<CivilDate>) -> Self {
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
    pub fn form_name(mut self, v: impl Into<Vary<String>>) -> Self {
        self.form_name = Some(v.into());
        self
    }
    pub fn required(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.required = v.into();
        self
    }
    pub fn readonly(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.readonly = v.into();
        self
    }
    pub fn invalid(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.invalid = v.into();
        self
    }
    pub fn parser(mut self, v: impl Fn(&str) -> DateParse + 'static) -> Self {
        self.parser = Rc::new(v);
        self
    }
    pub fn picker_label(mut self, v: impl Into<Vary<String>>) -> Self {
        self.picker_label = v.into();
        self
    }
    pub fn picker_title(mut self, v: impl Into<Vary<String>>) -> Self {
        self.picker_title = v.into();
        self
    }
    pub fn picker_close_label(mut self, v: impl Into<Vary<String>>) -> Self {
        self.picker_close = v.into();
        self
    }
    pub fn with_picker(mut self, v: bool) -> Self {
        self.picker = v;
        self
    }
    pub fn on_value_change(mut self, v: impl FnMut(Option<CivilDate>) + 'static) -> Self {
        self.changed = Some(Box::new(v));
        self
    }
    pub fn on_commit(mut self, v: impl FnMut(Option<CivilDate>) + 'static) -> Self {
        self.committed = Some(Box::new(v));
        self
    }
    pub fn on_draft_change(mut self, v: impl FnMut(DateInputDraft) + 'static) -> Self {
        self.draft_changed = Some(Box::new(v));
        self
    }
    pub fn on_reject(mut self, v: impl FnMut(DateDraftStatus) + 'static) -> Self {
        self.rejected = Some(Box::new(v));
        self
    }
}
impl Widget for DateInput {
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
        let error_id = SemanticId::new(format!("{}-date-error", self.key))?;
        let control = std::mem::take(&mut self.value).mount(scope);
        let value = control.source.clone();
        let provider = self.provider.clone();
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
                max_bytes: MAX_DATE_DRAFT_BYTES,
            });
            if let Some(name) = form_name {
                metadata.form_control = Some(glory::render::FormControl {
                    name: name.get(),
                    value: None,
                });
            }
            Snapshot {
                value: value.get(),
                provider: provider.get(),
                enabled: enabled.get(),
                metadata,
            }
        });
        let accepted = composite::validated(scope, &source.into(), |s: &Snapshot| {
            if s.value.is_none_or(|d| s.provider.date_bounds().contains(d)) {
                Ok(())
            } else {
                Err(invalid())
            }
        })?;
        let initial = accepted.try_with(|s| s.value)?;
        let initial_text = text(initial);
        let draft = scope.cage(DateInputDraft {
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
            parser: self.parser.clone(),
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
            .class("sapid-date-input-group")
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
            "sapid-date-input",
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
                        scope.set_input_value(node, text(value))?;
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
                            DateInputDraft {
                                text: p.text,
                                status: p.status,
                            }
                        }
                        _ => DateInputDraft {
                            text: text(value),
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
            .class("sapid-date-input-error")
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
                        DateDraftStatus::Invalid => "Enter a valid date",
                        DateDraftStatus::OutOfBounds(_) => "Date is outside the allowed bounds",
                        DateDraftStatus::TooLong => "Date input exceeds 64 bytes",
                        _ => "",
                    })
                    .to_owned()
            })))
            .show_in(&content)?;
        if self.picker {
            let open = scope.cage(false);
            let s = state.clone();
            let theme = self.common.theme.clone();
            let size = self.common.size;
            let picker_value = state.control.source.clone();
            let picker_selection =
                scope.bond(move || DateSelection::Single(picker_value.get()));
            let calendar_key = format!("{}-days", self.key);
            let dialog_key = format!("{}-picker", self.key);
            let close = composite::id("dialog", &dialog_key, "close", "root")?;
            crate::dialog(dialog_key)
                .initial_focus(close)
                .label(self.picker_label.clone())
                .title(self.picker_title.clone())
                .close_label(self.picker_close.clone())
                .open(open)
                .on_open_change(move |v| open.revise(|old| *old = v))
                .theme(theme.clone())
                .size(size)
                .enabled(scope.bond(move || accepted.with(|s| s.enabled)))
                .content_policy(crate::ContentPolicy::KeepMounted)
                .content(move || {
                    let s = s.clone();
                    crate::date_picker(calendar_key.clone(), accepted.map(|s| s.provider))
                        .value(picker_selection)
                        .enabled(accepted.map(|s| s.enabled))
                        .readonly(accepted.map(|s| s.metadata.readonly))
                        .theme(theme.clone())
                        .size(size)
                        .on_value_change(move |v| {
                            if let DateSelection::Single(next) = v
                                && s.ensure().is_ok()
                            {
                                let result = (|| -> Result {
                                    if s.choose(next)? {
                                        open.revise(|v| *v = false);
                                    }
                                    Ok(())
                                })();
                                s.report(result);
                            }
                        })
                })
                .show_in(&content)?;
        }
        scope.retain(state)?;
        scope.retain(content)
    }
}
