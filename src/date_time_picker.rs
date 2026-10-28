//! Civil date/time drafts with explicit application-owned resolution.
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback};
use crate::state::{Control, Dirty, Value};
use crate::{
    DateSelection, DateTimeBounds, DateTimeDraft, DateTimeOutcome, DateTimeParts,
    DateTimePolicy, DateTimeStatus, DatesProvider, FieldControl, LocalDateTime, LocalTime,
    TimeFormat, TimeGridSpec, TimePickerDraft, TimePickerStatus,
};
use glory::render::{Direction, NodeId, Role};
use glory::style::{Display, FlexDirection};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Scope, Vary};
use std::{cell::Cell, rc::Rc};
fn invalid() -> glory::ui::Error {
    glory::render::Error::InvalidProps("invalid finite date/time picker configuration").into()
}
#[derive(Clone, Copy, PartialEq)]
struct Snapshot {
    value: Option<LocalDateTime>,
    provider: DatesProvider,
    bounds: DateTimeBounds,
    enabled: bool,
    readonly: bool,
    revision: u64,
}
#[derive(Clone, Copy)]
struct Pending {
    baseline: Option<LocalDateTime>,
    proposal: Option<Option<LocalDateTime>>,
}
struct Controller {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    live: Cell<bool>,
    epoch: Cell<u64>,
    control: Control<Option<LocalDateTime>>,
    accepted: Cage<Snapshot>,
    input: glory::Bond<Snapshot>,
    policy: DateTimePolicy,
    parts: Cage<DateTimeParts>,
    time_status: Cell<TimePickerStatus>,
    status: Cage<DateTimeStatus>,
    reset: Cage<u64>,
    last: Cell<Option<DateTimeDraft>>,
    pending: Cell<Option<Pending>>,
    dirty: Dirty,
    changed: Callback<Option<LocalDateTime>>,
    resolved: Callback<DateTimeOutcome>,
    drafted: Callback<DateTimeDraft>,
    committed: Callback<Option<DateTimeOutcome>>,
    rejected: Callback<DateTimeStatus>,
}
impl Controller {
    fn ensure(&self) -> Result {
        if self.live.get() && self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn next(&self) -> Result<u64> {
        self.ensure()?;
        let n = self.epoch.get().checked_add(1).ok_or_else(invalid)?;
        self.epoch.set(n);
        Ok(n)
    }
    fn writable(&self) -> Result<bool> {
        Ok(self.accepted.try_with(|s| s.enabled && !s.readonly)?)
    }
    fn reset_parts(&self, value: Option<LocalDateTime>) -> Result {
        self.next()?;
        self.parts
            .try_revise(|p| *p = DateTimeParts::from_local(value))?;
        self.time_status
            .set(value.map_or(TimePickerStatus::Empty, |v| {
                TimePickerStatus::Valid(v.time())
            }));
        let reset = self.reset.try_get()?.checked_add(1).ok_or_else(invalid)?;
        self.reset.try_revise(|n| *n = reset)?;
        self.last.set(None);
        Ok(())
    }
    fn stamp(&self) -> Result<(Option<LocalDateTime>, Option<u64>)> {
        let version = match &self.control.source {
            Vary::Cage(c) => Some(c.try_version()?),
            _ => None,
        };
        Ok((self.control.source.try_get()?, version))
    }
    fn evaluate(&self) -> Result<Option<(u64, DateTimeDraft)>> {
        let epoch = self.epoch.get();
        let s = self.accepted.try_get()?;
        let parts = self.parts.try_get()?;
        let stamp = self.stamp()?;
        let configuration = glory::core::untracked(|| self.input.try_get())?;
        // Application rules may reenter, write sources or retire this Owner.
        // No Cage or callback registry is borrowed across their invocation.
        let status = match self.time_status.get() {
            TimePickerStatus::Empty | TimePickerStatus::Valid(_) => {
                glory::core::untracked(|| parts.classify(s.bounds, &self.policy))
            }
            TimePickerStatus::Incomplete => DateTimeStatus::Incomplete,
            status => DateTimeStatus::InvalidTime(status),
        };
        if !self.live.get() || !self.owner.is_alive() {
            return Ok(None);
        }
        if self.epoch.get() != epoch
            || self.accepted.try_get()? != s
            || self.parts.try_get()? != parts
            || self.stamp()? != stamp
            || glory::core::untracked(|| self.input.try_get())? != configuration
        {
            self.dirty.touch()?;
            return Ok(None);
        }
        Ok(Some((epoch, DateTimeDraft { parts, status })))
    }
    fn publish(&self, epoch: u64, draft: DateTimeDraft, notify: bool) -> Result<bool> {
        if self.status.try_get()? != draft.status {
            self.status.try_revise(|s| *s = draft.status)?;
        }
        let different = self.last.replace(Some(draft)) != Some(draft);
        if notify && different {
            self.drafted.emit(self.owner, draft);
        }
        Ok(self.live.get() && self.owner.is_alive() && self.epoch.get() == epoch)
    }
    fn propose(&self) -> Result {
        let baseline = self.control.source.try_get()?;
        let stamp = self.stamp()?;
        let configuration = glory::core::untracked(|| self.input.try_get())?;
        let Some((epoch, draft)) = self.evaluate()? else {
            return Ok(());
        };
        let proposal = draft.status.proposal();
        self.pending.set(Some(Pending { baseline, proposal }));
        if !self.publish(epoch, draft, true)? {
            return Ok(());
        }
        if self.stamp()? != stamp
            || glory::core::untracked(|| self.input.try_get())? != configuration
        {
            self.pending.set(None);
            self.dirty.touch()?;
            return Ok(());
        }
        if let Some(value) = proposal {
            self.control.accept(&value)?;
            self.changed.emit(self.owner, value);
            if !self.live.get() || !self.owner.is_alive() {
                return Ok(());
            }
            if self.epoch.get() == epoch
                && self.control.source.try_get()? == value
                && let DateTimeStatus::Valid(outcome) = draft.status
            {
                self.resolved.emit(self.owner, outcome);
            }
        }
        if self.live.get() && self.owner.is_alive() {
            self.dirty.touch()?;
        }
        Ok(())
    }
    fn date(&self, value: DateSelection) -> Result {
        self.ensure()?;
        if !self.writable()? {
            return Ok(());
        }
        let DateSelection::Single(date) = value else {
            return Err(invalid());
        };
        self.next()?;
        self.parts.try_revise(|p| p.date = date)?;
        self.propose()
    }
    fn time_draft(&self, draft: TimePickerDraft) -> Result {
        self.ensure()?;
        if !self.writable()? {
            return Ok(());
        }
        self.next()?;
        self.time_status.set(draft.status);
        if !matches!(
            draft.status,
            TimePickerStatus::Valid(_) | TimePickerStatus::Empty
        ) {
            self.propose()?;
        }
        Ok(())
    }
    fn time(&self, value: Option<LocalTime>) -> Result {
        self.ensure()?;
        if !self.writable()? {
            return Ok(());
        }
        self.next()?;
        self.parts.try_revise(|p| p.time = value)?;
        self.time_status
            .set(value.map_or(TimePickerStatus::Empty, TimePickerStatus::Valid));
        self.propose()
    }
    fn clear(&self) -> Result {
        self.ensure()?;
        if !self.writable()? {
            return Ok(());
        }
        self.reset_parts(None)?;
        self.propose()
    }
    fn done(&self) -> Result {
        self.ensure()?;
        if !self.writable()? {
            return Ok(());
        }
        let configuration = glory::core::untracked(|| self.input.try_get())?;
        let Some((epoch, draft)) = self.evaluate()? else {
            return Ok(());
        };
        if !self.publish(epoch, draft, true)? {
            return Ok(());
        }
        if glory::core::untracked(|| self.input.try_get())? != configuration {
            self.dirty.touch()?;
            return Ok(());
        }
        let result = match draft.status {
            DateTimeStatus::Empty if self.control.source.try_get()?.is_none() => Some(None),
            DateTimeStatus::Valid(v) if self.control.source.try_get()? == Some(v.local) => {
                Some(Some(v))
            }
            _ => None,
        };
        if let Some(value) = result {
            self.committed.emit(self.owner, value);
        } else {
            self.rejected.emit(self.owner, draft.status);
        }
        Ok(())
    }
    fn report(&self, result: Result) {
        if let Err(e) = result
            && let Some(scope) = self.weak.upgrade()
        {
            scope.report_error(e);
        }
    }
    fn retire(&self) {
        self.live.set(false);
        self.pending.set(None);
        self.last.set(None);
        self.changed.clear();
        self.resolved.clear();
        self.drafted.clear();
        self.committed.clear();
        self.rejected.clear();
    }
}
/// Inline Calendar and real segmented spinbutton inputs. Policy is always explicit.
pub struct InlineDateTimePicker {
    pub(crate) common: Common,
    key: String,
    provider: Vary<DatesProvider>,
    policy: DateTimePolicy,
    value: Value<Option<LocalDateTime>>,
    bounds: Vary<DateTimeBounds>,
    revision: Vary<u64>,
    format: TimeFormat,
    readonly: Vary<bool>,
    field: Option<FieldControl>,
    direction: Direction,
    grid: Option<TimeGridSpec>,
    refs: [Option<Cage<Option<NodeId>>>; 3],
    calendar_ref: Option<Cage<Option<NodeId>>>,
    changed: Option<Box<dyn FnMut(Option<LocalDateTime>)>>,
    resolved: Option<Box<dyn FnMut(DateTimeOutcome)>>,
    drafted: Option<Box<dyn FnMut(DateTimeDraft)>>,
    committed: Option<Box<dyn FnMut(Option<DateTimeOutcome>)>>,
    rejected: Option<Box<dyn FnMut(DateTimeStatus)>>,
}
pub fn inline_date_time_picker(
    key: impl Into<String>,
    provider: impl Into<Vary<DatesProvider>>,
    policy: DateTimePolicy,
) -> InlineDateTimePicker {
    InlineDateTimePicker {
        common: Common::default(),
        key: key.into(),
        provider: provider.into(),
        policy,
        value: Value::default(),
        bounds: DateTimeBounds::default().into(),
        revision: 0.into(),
        format: TimeFormat::default(),
        readonly: false.into(),
        field: None,
        direction: Direction::Ltr,
        grid: None,
        refs: [None; 3],
        calendar_ref: None,
        changed: None,
        resolved: None,
        drafted: None,
        committed: None,
        rejected: None,
    }
}
impl InlineDateTimePicker {
    common_methods!();
    pub fn value(mut self, v: impl Into<Vary<Option<LocalDateTime>>>) -> Self {
        self.value.source = Some(v.into());
        self
    }
    pub fn default_value(mut self, v: Option<LocalDateTime>) -> Self {
        self.value.default = v;
        self
    }
    pub fn bounds(mut self, v: impl Into<Vary<DateTimeBounds>>) -> Self {
        self.bounds = v.into();
        self
    }
    /// An explicit application revision resets drafts, even for an equal civil value,
    /// and reevaluates resolver rules. Applications must not wrap this counter.
    pub fn reset_revision(mut self, v: impl Into<Vary<u64>>) -> Self {
        self.revision = v.into();
        self
    }
    pub fn format(mut self, v: TimeFormat) -> Self {
        self.format = v;
        self
    }
    pub fn readonly(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.readonly = v.into();
        self
    }
    pub fn field(mut self, v: FieldControl) -> Self {
        self.common.theme = v.theme.clone();
        self.common.size = v.size;
        self.common.unstyled = v.unstyled;
        self.field = Some(v);
        self
    }
    pub fn direction(mut self, v: Direction) -> Self {
        self.direction = v;
        self
    }
    pub fn grid(mut self, v: TimeGridSpec) -> Self {
        self.grid = Some(v);
        self
    }
    pub fn hour_ref(mut self, v: Cage<Option<NodeId>>) -> Self {
        self.refs[0] = Some(v);
        self
    }
    pub fn minute_ref(mut self, v: Cage<Option<NodeId>>) -> Self {
        self.refs[1] = Some(v);
        self
    }
    pub fn second_ref(mut self, v: Cage<Option<NodeId>>) -> Self {
        self.refs[2] = Some(v);
        self
    }
    pub fn calendar_ref(mut self, v: Cage<Option<NodeId>>) -> Self {
        self.calendar_ref = Some(v);
        self
    }
    pub fn on_value_change(mut self, v: impl FnMut(Option<LocalDateTime>) + 'static) -> Self {
        self.changed = Some(Box::new(v));
        self
    }
    pub fn on_resolved(mut self, v: impl FnMut(DateTimeOutcome) + 'static) -> Self {
        self.resolved = Some(Box::new(v));
        self
    }
    pub fn on_draft_change(mut self, v: impl FnMut(DateTimeDraft) + 'static) -> Self {
        self.drafted = Some(Box::new(v));
        self
    }
    pub fn on_commit(mut self, v: impl FnMut(Option<DateTimeOutcome>) + 'static) -> Self {
        self.committed = Some(Box::new(v));
        self
    }
    pub fn on_reject(mut self, v: impl FnMut(DateTimeStatus) + 'static) -> Self {
        self.rejected = Some(Box::new(v));
        self
    }
}
impl Widget for InlineDateTimePicker {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.key.is_empty()
            || self.key.len() > 48
            || !self
                .key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(invalid());
        }
        let enabled = self
            .field
            .as_ref()
            .map(|f| f.enabled(scope, self.common.enabled.clone()))
            .unwrap_or_else(|| self.common.enabled.clone());
        let readonly = self
            .field
            .as_ref()
            .map(|f| f.readonly_state(scope, self.readonly.clone()))
            .unwrap_or_else(|| self.readonly.clone());
        let control = std::mem::take(&mut self.value).mount(scope);
        let source = control.source.clone();
        let provider = self.provider.clone();
        let bounds = self.bounds.clone();
        let revision = self.revision.clone();
        let source = scope.bond(move || Snapshot {
            value: source.get(),
            provider: provider.get(),
            bounds: bounds.get(),
            enabled: enabled.get(),
            readonly: readonly.get(),
            revision: revision.get(),
        });
        let format = self.format;
        let accepted = composite::validated(scope, &source.into(), move |s: &Snapshot| {
            let provider = s.bounds.provider(s.provider).map_err(|_| invalid())?;
            if s.value.is_some_and(|v| {
                !s.bounds.contains(v)
                    || !provider.date_bounds().contains(v.date())
                    || format.validate(v.time()).is_err()
            }) {
                return Err(invalid());
            }
            Ok(())
        })?;
        let initial = accepted.get();
        let state = Rc::new(Controller {
            owner: scope.id(),
            weak: scope.downgrade(),
            live: Cell::new(true),
            epoch: Cell::new(0),
            control,
            accepted,
            input: source,
            policy: self.policy.clone(),
            parts: scope.cage(DateTimeParts::from_local(initial.value)),
            time_status: Cell::new(initial.value.map_or(TimePickerStatus::Empty, |v| {
                TimePickerStatus::Valid(v.time())
            })),
            status: scope.cage(DateTimeStatus::Empty),
            reset: scope.cage(0),
            last: Cell::new(None),
            pending: Cell::new(None),
            dirty: Dirty::new(scope),
            changed: Callback::new(self.changed.take()),
            resolved: Callback::new(self.resolved.take()),
            drafted: Callback::new(self.drafted.take()),
            committed: Callback::new(self.committed.take()),
            rejected: Callback::new(self.rejected.take()),
        });
        let cleanup = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(s) = cleanup.upgrade() {
                s.retire()
            }
        })?;
        let root = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let (class, appearance) = self.common.appearance(
            scope,
            "sapid-inline-date-time-picker",
            composite::column(8.),
        );
        let style = scope.bond(move || {
            let mut s = appearance.get();
            s.display.get_or_insert(Display::Flex);
            s.flex_direction.get_or_insert(FlexDirection::Column);
            s.gap.get_or_insert(8.);
            s
        });
        let mut widget = glory::ui::view()
            .role(Role::Group)
            .class(class)
            .style(style)
            .node_ref(root)
            .focus(self.common.focus);
        widget = widget.name(
            self.common
                .name
                .take()
                .unwrap_or_else(|| "Date and time".to_owned().into()),
        );
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            widget = widget.on(kind, move |e| {
                crate::appearance::observe(&mut handlers, owner, e)
            });
        }
        widget.show_in(scope)?;
        let content = scope.child_in(root.get().ok_or(glory::ui::Error::Gone)?)?;
        let parts = state.parts;
        let calendar_value = scope.bond(move || {
            let s = accepted.get();
            let bounds = s.bounds.provider(s.provider).unwrap().date_bounds();
            DateSelection::Single(parts.with(|p| p.date.filter(|d| bounds.contains(*d))))
        });
        let s = state.clone();
        let mut calendar = crate::date_picker(
            format!("{}-date", self.key),
            accepted.map(|s| s.bounds.provider(s.provider).unwrap()),
        )
        .value(calendar_value)
        .enabled(accepted.map(|s| s.enabled))
        .readonly(accepted.map(|s| s.readonly))
        .direction(self.direction)
        .theme(self.common.theme.clone())
        .size(self.common.size)
        .name("Date")
        .allow_deselect(true)
        .on_value_change(move |v| s.report(s.date(v)));
        if let Some(r) = self.calendar_ref {
            calendar = calendar.node_ref(r);
        }
        if self.common.unstyled {
            calendar = calendar.unstyled();
        }
        calendar.show_in(&content)?;
        let parts = state.parts;
        let ontime = state.clone();
        let drafted = state.clone();
        let mut time = crate::time_picker(format!("{}-time", self.key))
            .format(format)
            .value(parts.map(|p| p.time))
            .reset_epoch(state.reset)
            .enabled(accepted.map(|s| s.enabled))
            .readonly(accepted.map(|s| s.readonly))
            .theme(self.common.theme.clone())
            .size(self.common.size)
            .on_draft_change(move |v| drafted.report(drafted.time_draft(v)))
            .on_value_change(move |v| ontime.report(ontime.time(v)));
        if let Some(v) = self.grid {
            time = time.grid(v);
        }
        if let Some(v) = &self.field {
            time = time.field(v.clone());
        }
        if let Some(r) = self.refs[0] {
            time = time.hour_ref(r);
        }
        if let Some(r) = self.refs[1] {
            time = time.minute_ref(r);
        }
        if let Some(r) = self.refs[2] {
            time = time.second_ref(r);
        }
        if self.common.unstyled {
            time = time.unstyled();
        }
        time.show_in(&content)?;
        let status = state.status;
        glory::ui::text()
            .class("sapid-date-time-status")
            .content(scope.bond(move || {
                match status.get() {
                    DateTimeStatus::OutOfBounds(_) => {
                        "Date/time is outside the allowed interval"
                    }
                    DateTimeStatus::Ambiguous { .. } => {
                        "Date/time is ambiguous; choose an explicit application policy"
                    }
                    DateTimeStatus::Gap => {
                        "Date/time does not exist in the application timezone rules"
                    }
                    DateTimeStatus::InvalidResolver => {
                        "Application timezone resolver returned invalid candidates"
                    }
                    DateTimeStatus::InvalidTime(_) => "Complete a valid time",
                    _ => "",
                }
                .to_owned()
            }))
            .show_in(&content)?;
        for (clear, label) in [(true, "Clear date and time"), (false, "Done")] {
            let s = state.clone();
            let mut button = crate::button()
                .label(label)
                .theme(self.common.theme.clone())
                .size(self.common.size)
                .enabled(accepted.map(|v| v.enabled && !v.readonly))
                .on_press(move || s.report(if clear { s.clear() } else { s.done() }));
            if self.common.unstyled {
                button = button.unstyled();
            }
            button.show_in(&content)?;
        }
        let weak = Rc::downgrade(&state);
        let mut previous = initial;
        scope.effect(move || {
            let Some(s) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let dirty = s.dirty.take();
                let next = accepted.get();
                let external = next.value != previous.value
                    || next.provider != previous.provider
                    || next.bounds != previous.bounds
                    || next.revision != previous.revision;
                if dirty || external {
                    let pending = s.pending.take();
                    let keep = next.revision == previous.revision
                        && pending.is_some_and(|p| {
                            p.proposal == Some(next.value)
                                || p.proposal.is_none() && p.baseline == next.value
                        });
                    if !keep {
                        s.reset_parts(next.value)?;
                    }
                    previous = next;
                }
                if let Some((epoch, draft)) = s.evaluate()? {
                    s.publish(epoch, draft, false)?;
                }
                Ok(())
            })();
            s.report(result)
        });
        scope.retain(content)?;
        scope.retain(state)
    }
}
