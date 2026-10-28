//! Segmented real spinbutton editors over one typed wall-clock value.
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback};
use crate::state::{Control, Dirty, Value};
use crate::{
    Decimal, FieldControl, HourCycle, LocalTime, MAX_DISABLED_TIMES, NumberValue, TimeBounds,
    TimeFormat, TimeGridSpec, TimeParts, TimePickerDraft, TimePickerStatus, TimePrecision,
};
use glory::render::{NodeId, Role};
use glory::style::{Align, Display, FlexDirection, Length, Style};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Scope, Vary};
use std::{cell::Cell, rc::Rc};
fn invalid() -> glory::ui::Error {
    glory::render::Error::InvalidProps("invalid finite segmented time picker").into()
}
fn decimal(v: u8) -> Decimal {
    Decimal::new(i128::from(v), 0).unwrap()
}
fn number(v: Option<u8>) -> NumberValue {
    v.map_or(NumberValue::Empty, |v| NumberValue::Text(format!("{v:02}")))
}
fn part(v: &NumberValue) -> std::result::Result<Option<u8>, TimePickerStatus> {
    match v {
        NumberValue::Empty => Ok(None),
        NumberValue::Decimal(v) if v.scale() == 0 && (0..=99).contains(&v.coefficient()) => {
            Ok(Some(v.coefficient() as u8))
        }
        NumberValue::Text(v) if v.is_empty() => Ok(None),
        NumberValue::Text(v) if v.len() > 2 => Err(TimePickerStatus::TooLong),
        NumberValue::Text(v) if v.bytes().all(|b| b.is_ascii_digit()) => {
            Ok(Some(v.parse().map_err(|_| TimePickerStatus::Invalid)?))
        }
        _ => Err(TimePickerStatus::Invalid),
    }
}
#[derive(Clone, Copy, PartialEq)]
struct Snapshot {
    value: Option<LocalTime>,
    bounds: TimeBounds,
    enabled: bool,
    readonly: bool,
    reset_epoch: u64,
}
#[derive(Clone, Copy)]
struct Pending {
    baseline: Option<LocalTime>,
    proposal: Option<Option<LocalTime>>,
}
struct Controller {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    live: Cell<bool>,
    control: Control<Option<LocalTime>>,
    accepted: Cage<Snapshot>,
    format: TimeFormat,
    parts: [Cage<NumberValue>; 3],
    afternoon: Cage<bool>,
    status: Cage<TimePickerStatus>,
    last: Cell<Option<TimePickerDraft>>,
    disabled: Rc<Vec<LocalTime>>,
    pending: Cell<Option<Pending>>,
    dirty: Dirty,
    changed: Callback<Option<LocalTime>>,
    committed: Callback<Option<LocalTime>>,
    draft_changed: Callback<TimePickerDraft>,
    rejected: Callback<TimePickerStatus>,
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
        self.accepted
            .try_with(|s| s.enabled && !s.readonly)
            .map_err(Into::into)
    }
    fn current(&self) -> Result<Option<LocalTime>> {
        let s = self.accepted.try_get()?;
        let value = self.control.source.try_get()?;
        Ok(
            if value.is_none_or(|v| s.bounds.contains(v) && self.format.validate(v).is_ok()) {
                value
            } else {
                s.value
            },
        )
    }
    fn parts(&self) -> Result<TimeParts> {
        Ok(TimeParts {
            hour: self.parts[0].try_with(part)?.map_err(|_| invalid())?,
            minute: self.parts[1].try_with(part)?.map_err(|_| invalid())?,
            second: if self.format.precision == TimePrecision::Second {
                self.parts[2].try_with(part)?.map_err(|_| invalid())?
            } else {
                None
            },
            afternoon: self.afternoon.get(),
        })
    }
    fn classify(&self, parts: TimeParts) -> Result<TimePickerStatus> {
        let status = parts.classify(self.format, self.accepted.try_with(|s| s.bounds)?);
        Ok(match status {
            TimePickerStatus::Valid(v) if self.disabled.binary_search(&v).is_ok() => {
                TimePickerStatus::Unavailable(v)
            }
            _ => status,
        })
    }
    fn publish_draft(&self, draft: TimePickerDraft) -> Result {
        if self.status.try_get()? != draft.status {
            self.status.try_revise(|v| *v = draft.status)?;
        }
        if self.last.replace(Some(draft)) != Some(draft) {
            self.draft_changed.emit(self.owner, draft);
            self.ensure()?;
        }
        Ok(())
    }
    fn reject(&self, status: TimePickerStatus) -> Result {
        self.ensure()?;
        self.publish_draft(TimePickerDraft {
            parts: self.parts()?,
            status,
        })?;
        self.rejected.emit(self.owner, status);
        self.ensure()?;
        self.pending.set(None);
        self.dirty.touch()
    }
    fn propose(&self, committing: bool) -> Result {
        let baseline = self.current()?;
        let parts = self.parts()?;
        let status = self.classify(parts)?;
        let proposal = status.proposal();
        self.pending.set(Some(Pending { baseline, proposal }));
        self.publish_draft(TimePickerDraft { parts, status })?;
        if let Some(next) = proposal {
            self.control.accept(&next)?;
            if !committing || next != baseline {
                self.changed.emit(self.owner, next);
                self.ensure()?;
            }
            if committing {
                self.committed.emit(self.owner, next);
                self.ensure()?;
            }
        } else if committing {
            self.rejected.emit(self.owner, status);
            self.ensure()?;
        }
        self.dirty.touch()
    }
    fn segment(&self, index: usize, value: NumberValue, committing: bool) -> Result {
        self.ensure()?;
        if !self.writable()? {
            return Ok(());
        }
        let parsed = match part(&value) {
            Ok(v) => v,
            Err(e) => return self.reject(e),
        };
        let value = if committing || matches!(value, NumberValue::Decimal(_)) {
            number(parsed)
        } else {
            value
        };
        if self.parts[index].try_with(|old| *old != value)? {
            self.parts[index].try_revise(|v| *v = value)?;
        }
        self.propose(committing)
    }
    fn period(&self, afternoon: bool) -> Result {
        self.ensure()?;
        if !self.writable()? {
            return Ok(());
        }
        self.afternoon.try_revise(|v| *v = afternoon)?;
        self.propose(false)
    }
    fn set_parts(&self, value: Option<LocalTime>) -> Result {
        let parts = TimeParts::from_time(value, self.format).map_err(|_| invalid())?;
        for (index, part) in [parts.hour, parts.minute, parts.second]
            .into_iter()
            .enumerate()
        {
            let next = number(part);
            if self.parts[index].try_with(|old| *old != next)? {
                self.parts[index].try_revise(|v| *v = next)?;
            }
        }
        if self.afternoon.get() != parts.afternoon {
            self.afternoon.try_revise(|v| *v = parts.afternoon)?;
        }
        let status = parts.classify(self.format, self.accepted.try_with(|s| s.bounds)?);
        if self.status.get() != status {
            self.status.try_revise(|v| *v = status)?;
        }
        self.last.set(Some(TimePickerDraft { parts, status }));
        Ok(())
    }
    fn choose(&self, value: Option<LocalTime>) -> Result {
        self.ensure()?;
        if !self.writable()? {
            return Ok(());
        }
        let s = self.accepted.try_get()?;
        if value.is_some_and(|v| {
            !s.bounds.contains(v)
                || self.format.validate(v).is_err()
                || self.disabled.binary_search(&v).is_ok()
        }) {
            return Ok(());
        }
        let baseline = self.current()?;
        self.set_parts(value)?;
        let draft = TimePickerDraft {
            parts: self.parts()?,
            status: self.status.get(),
        };
        self.last.set(None);
        self.publish_draft(draft)?;
        self.control.accept(&value)?;
        self.changed.emit(self.owner, value);
        self.ensure()?;
        self.pending.set(Some(Pending {
            baseline,
            proposal: Some(value),
        }));
        self.dirty.touch()
    }
    fn report(&self, result: Result) {
        if let Err(e) = result
            && let Some(scope) = self.weak.upgrade()
        {
            scope.report_error(e)
        }
    }
    fn retire(&self) {
        self.live.set(false);
        self.pending.set(None);
        self.last.set(None);
        self.changed.clear();
        self.committed.clear();
        self.draft_changed.clear();
        self.rejected.clear();
    }
}
pub struct TimePicker {
    common: Common,
    key: String,
    format: TimeFormat,
    bounds: Vary<TimeBounds>,
    value: Value<Option<LocalTime>>,
    readonly: Vary<bool>,
    reset_epoch: Vary<u64>,
    required: Vary<bool>,
    invalid: Vary<bool>,
    field: Option<FieldControl>,
    form_name: Option<Vary<String>>,
    grid: Option<TimeGridSpec>,
    disabled: Vec<LocalTime>,
    references: [Option<Cage<Option<NodeId>>>; 3],
    steps: [u8; 3],
    changed: Option<Box<dyn FnMut(Option<LocalTime>)>>,
    committed: Option<Box<dyn FnMut(Option<LocalTime>)>>,
    draft_changed: Option<Box<dyn FnMut(TimePickerDraft)>>,
    rejected: Option<Box<dyn FnMut(TimePickerStatus)>>,
}
pub fn time_picker(key: impl Into<String>) -> TimePicker {
    TimePicker {
        common: Common::default(),
        key: key.into(),
        format: TimeFormat::default(),
        bounds: TimeBounds::default().into(),
        value: Value::default(),
        readonly: false.into(),
        reset_epoch: 0.into(),
        required: false.into(),
        invalid: false.into(),
        field: None,
        form_name: None,
        grid: None,
        disabled: vec![],
        references: [None; 3],
        steps: [1; 3],
        changed: None,
        committed: None,
        draft_changed: None,
        rejected: None,
    }
}
impl TimePicker {
    common_methods!();
    pub(crate) fn reset_epoch(mut self, v: impl Into<Vary<u64>>) -> Self {
        self.reset_epoch = v.into();
        self
    }
    pub fn format(mut self, v: TimeFormat) -> Self {
        self.format = v;
        self
    }
    pub fn bounds(mut self, v: impl Into<Vary<TimeBounds>>) -> Self {
        self.bounds = v.into();
        self
    }
    pub fn value(mut self, v: impl Into<Vary<Option<LocalTime>>>) -> Self {
        self.value.source = Some(v.into());
        self
    }
    pub fn default_value(mut self, v: Option<LocalTime>) -> Self {
        self.value.default = v;
        self
    }
    pub fn readonly(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.readonly = v.into();
        self
    }
    pub fn required(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.required = v.into();
        self
    }
    pub fn invalid(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.invalid = v.into();
        self
    }
    pub fn field(mut self, v: FieldControl) -> Self {
        self.common.theme = v.theme.clone();
        self.common.size = v.size;
        self.common.unstyled = v.unstyled;
        self.field = Some(v);
        self
    }
    /// Produces explicit `_hour`, `_minute` and optional `_second` successful names.
    pub fn form_name(mut self, v: impl Into<Vary<String>>) -> Self {
        self.form_name = Some(v.into());
        self
    }
    pub fn grid(mut self, v: TimeGridSpec) -> Self {
        self.grid = Some(v);
        self
    }
    pub fn disable_time(mut self, v: LocalTime) -> Self {
        if self.disabled.len() <= MAX_DISABLED_TIMES {
            self.disabled.push(v)
        }
        self
    }
    pub fn disable_times(mut self, v: impl IntoIterator<Item = LocalTime>) -> Self {
        self.disabled = v.into_iter().take(MAX_DISABLED_TIMES + 1).collect();
        self
    }
    pub fn hour_step(mut self, value: u8) -> Self {
        self.steps[0] = value;
        self
    }
    pub fn minute_step(mut self, value: u8) -> Self {
        self.steps[1] = value;
        self
    }
    pub fn second_step(mut self, value: u8) -> Self {
        self.steps[2] = value;
        self
    }
    pub fn hour_ref(mut self, v: Cage<Option<NodeId>>) -> Self {
        self.references[0] = Some(v);
        self
    }
    pub fn minute_ref(mut self, v: Cage<Option<NodeId>>) -> Self {
        self.references[1] = Some(v);
        self
    }
    pub fn second_ref(mut self, v: Cage<Option<NodeId>>) -> Self {
        self.references[2] = Some(v);
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
    pub fn on_draft_change(mut self, v: impl FnMut(TimePickerDraft) + 'static) -> Self {
        self.draft_changed = Some(Box::new(v));
        self
    }
    pub fn on_reject(mut self, v: impl FnMut(TimePickerStatus) + 'static) -> Self {
        self.rejected = Some(Box::new(v));
        self
    }
}
impl Widget for TimePicker {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.key.is_empty()
            || self.key.len() > 56
            || !self
                .key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || self.disabled.len() > MAX_DISABLED_TIMES
            || self
                .steps
                .iter()
                .enumerate()
                .any(|(at, step)| *step == 0 || *step > if at == 0 { 24 } else { 59 })
        {
            return Err(invalid());
        }
        let format = self.format;
        if let Some(spec) = self.grid {
            for at in 0..spec.len() {
                format
                    .validate(spec.at(at).unwrap())
                    .map_err(|_| invalid())?;
            }
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
        let value = control.source.clone();
        let bounds = self.bounds.clone();
        let reset_epoch = self.reset_epoch.clone();
        let source = scope.bond(move || Snapshot {
            value: value.get(),
            bounds: bounds.get(),
            enabled: enabled.get(),
            readonly: readonly.get(),
            reset_epoch: reset_epoch.get(),
        });
        let accepted = composite::validated(scope, &source.into(), move |s: &Snapshot| {
            if s.value
                .is_none_or(|v| s.bounds.contains(v) && format.validate(v).is_ok())
            {
                Ok(())
            } else {
                Err(invalid())
            }
        })?;
        let form_name = if let Some(name) = &self.form_name {
            Some(composite::validated(scope, name, |s| {
                if s.len() <= 240 && !s.contains(['\0', '\r', '\n']) {
                    Ok(())
                } else {
                    Err(invalid())
                }
            })?)
        } else {
            None
        };
        let initial = accepted.with(|s| s.value);
        let initial_parts = TimeParts::from_time(initial, format).map_err(|_| invalid())?;
        let initial_status = initial_parts.classify(format, accepted.with(|s| s.bounds));
        let mut disabled = std::mem::take(&mut self.disabled);
        disabled.sort_unstable();
        disabled.dedup();
        let disabled = Rc::new(disabled);
        let state = Rc::new(Controller {
            owner: scope.id(),
            weak: scope.downgrade(),
            live: Cell::new(true),
            control,
            accepted,
            format,
            parts: [
                scope.cage(number(initial_parts.hour)),
                scope.cage(number(initial_parts.minute)),
                scope.cage(number(initial_parts.second)),
            ],
            afternoon: scope.cage(initial_parts.afternoon),
            status: scope.cage(initial_status),
            last: Cell::new(Some(TimePickerDraft {
                parts: initial_parts,
                status: initial_status,
            })),
            disabled: disabled.clone(),
            pending: Cell::new(None),
            dirty: Dirty::new(scope),
            changed: Callback::new(self.changed.take()),
            committed: Callback::new(self.committed.take()),
            draft_changed: Callback::new(self.draft_changed.take()),
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
            "sapid-time-picker",
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(8.),
        );
        let style = scope.bond(move || {
            let mut style = appearance.get();
            style.display.get_or_insert(Display::Flex);
            style.flex_direction.get_or_insert(FlexDirection::Column);
            style.gap.get_or_insert(8.);
            style
        });
        let metadata = crate::field::control_metadata(
            scope,
            self.field.as_ref(),
            self.common.name.is_some(),
            self.required.clone(),
            self.readonly.clone(),
            self.invalid.clone(),
        );
        let mut widget = glory::ui::view()
            .role(Role::Group)
            .class(class)
            .style(style)
            .node_ref(root)
            .focus(self.common.focus);
        if let Some(name) = self.common.name.take() {
            widget = widget.name(name);
        } else if self.field.is_some() {
            let group_metadata = metadata.clone();
            widget = widget.semantics(scope.bond(move || {
                group_metadata.with(|m| {
                    m.as_ref().map(|m| {
                        Box::new(glory::render::Semantics {
                            labelled_by: m.labelled_by.clone(),
                            described_by: m.described_by.clone(),
                            ..Default::default()
                        })
                    })
                })
            }));
        } else {
            widget = widget.name("Time");
        }
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            widget = widget.on(kind, move |e| {
                crate::appearance::observe(&mut handlers, owner, e)
            });
        }
        widget.show_in(scope)?;
        let content = scope.child_in(root.get().ok_or(glory::ui::Error::Gone)?)?;
        let row = scope.cage(None);
        glory::ui::view()
            .class("sapid-time-segments")
            .style(Style {
                align: Some(Align::Center),
                ..Style::new().display(Display::Flex).gap(4.)
            })
            .node_ref(row)
            .show_in(&content)?;
        let editors = scope.child_in(row.get().ok_or(glory::ui::Error::Gone)?)?;
        let status = state.status;
        for index in 0..if format.precision == TimePrecision::Second {
            3
        } else {
            2
        } {
            if index > 0 {
                glory::ui::text().content(":").show_in(&editors)?;
            }
            let labels = if format.locale == crate::DateLocale::ZhCn {
                ["小时", "分钟", "秒"]
            } else {
                ["Hour", "Minute", "Second"]
            };
            let min = if index == 0 && format.cycle == HourCycle::H12 {
                1
            } else {
                0
            };
            let max = if index == 0 {
                if format.cycle == HourCycle::H12 {
                    12
                } else {
                    23
                }
            } else {
                59
            };
            let changed = state.clone();
            let committed = state.clone();
            let rejected = state.clone();
            let meta = metadata.clone();
            let errors = metadata.clone();
            let mut input = crate::number_input()
                .theme(self.common.theme.clone())
                .size(self.common.size)
                .name(labels[index])
                .class(match index {
                    0 => "sapid-time-hour",
                    1 => "sapid-time-minute",
                    _ => "sapid-time-second",
                })
                .style(Style::new().width(Length::Px(72.)))
                .value(state.parts[index])
                .min(Some(decimal(min)))
                .max(Some(decimal(max)))
                .step(decimal(self.steps[index]))
                .page_step(Decimal::new(i128::from(self.steps[index]) * 10, 0).unwrap())
                .controls(false)
                .enabled(accepted.map(|s| s.enabled))
                .readonly(accepted.map(|s| s.readonly))
                .required(scope.bond(move || meta.get().is_some_and(|s| s.required)))
                .invalid(scope.bond(move || {
                    status.get().is_error() || errors.get().is_some_and(|s| s.invalid)
                }))
                .on_value_change(move |v| changed.report(changed.segment(index, v, false)))
                .on_commit(move |v| committed.report(committed.segment(index, v, true)))
                .on_reject(move |reason| {
                    rejected.report(rejected.reject(
                        if reason == crate::NumberError::InputTooLong {
                            TimePickerStatus::TooLong
                        } else {
                            TimePickerStatus::Invalid
                        },
                    ))
                });
            if index == 0
                && let Some(field) = &self.field
            {
                input = input.field(field.clone());
            }
            if self.common.unstyled {
                input = input.unstyled();
            }
            if let Some(reference) = self.references[index] {
                input = input.node_ref(reference);
            }
            if let Some(name) = form_name {
                let suffix = ["hour", "minute", "second"][index];
                input = input.form_name(scope.bond(move || format!("{}_{suffix}", name.get())));
            }
            input.show_in(&editors)?;
        }
        if format.cycle == HourCycle::H12 {
            let s = state.clone();
            let mut toggle = crate::toggle()
                .theme(self.common.theme.clone())
                .size(self.common.size)
                .label(if format.locale == crate::DateLocale::ZhCn {
                    "下午"
                } else {
                    "Afternoon (PM)"
                })
                .pressed(state.afternoon)
                .enabled(accepted.map(|s| s.enabled && !s.readonly))
                .on_pressed_change(move |v| s.report(s.period(v)));
            if self.common.unstyled {
                toggle = toggle.unstyled();
            }
            toggle.show_in(&editors)?;
        }
        if let Some(spec) = self.grid {
            let s = state.clone();
            let value = accepted.map(|s| s.value);
            let mut grid = crate::time_grid(format!("{}-slots", self.key))
                .spec(spec)
                .format(format)
                .bounds(accepted.map(|s| s.bounds))
                .value(value)
                .enabled(accepted.map(|s| s.enabled))
                .readonly(accepted.map(|s| s.readonly))
                .theme(self.common.theme.clone())
                .size(self.common.size)
                .disable_times(disabled.iter().copied())
                .on_value_change(move |v| s.report(s.choose(v)));
            if self.common.unstyled {
                grid = grid.unstyled();
            }
            grid.show_in(&content)?;
        }
        let weak = Rc::downgrade(&state);
        let mut previous = initial;
        let mut previous_reset = accepted.with(|s| s.reset_epoch);
        scope.effect(move || {
            let Some(s) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let dirty = s.dirty.take();
                let value = accepted.with(|s| s.value);
                let reset = accepted.with(|s| s.reset_epoch);
                if dirty || value != previous || reset != previous_reset {
                    let pending = s.pending.take();
                    let keep = reset == previous_reset
                        && pending.is_some_and(|p| {
                            p.proposal == Some(value)
                                || p.proposal.is_none() && p.baseline == value
                        });
                    if !keep {
                        s.set_parts(value)?;
                    }
                    previous = value;
                    previous_reset = reset;
                }
                Ok(())
            })();
            s.report(result)
        });
        scope.retain(editors)?;
        scope.retain(content)?;
        scope.retain(state)
    }
}
