//! Civil date selection on a real Grid. No routing, transport or timezone inference.
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback};
use crate::state::{Control, Dirty, Value};
use crate::{
    CivilDate, DatePickerMode, DateSelection, DateSelectionError, DatesProvider, Weekday,
    YearMonth,
};
use glory::render::{Direction, Error};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Scope, Vary};
use std::{cell::Cell, rc::Rc};
fn invalid() -> glory::ui::Error {
    Error::InvalidProps("invalid finite date picker selection").into()
}
#[derive(Clone, Copy, PartialEq)]
struct Snapshot {
    value: DateSelection,
    provider: DatesProvider,
    enabled: bool,
    readonly: bool,
}
struct Controller {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    live: Cell<bool>,
    control: Control<DateSelection>,
    accepted: Cage<Snapshot>,
    cursor: Cell<Option<DateSelection>>,
    dirty: Dirty,
    changed: Callback<DateSelection>,
    rejected: Callback<DateSelectionError>,
}
impl Controller {
    fn ensure(&self) -> Result {
        if self.live.get() && self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn select(&self, date: CivilDate, deselect: bool) -> Result {
        self.ensure()?;
        let s = self.accepted.try_get()?;
        if !s.enabled || s.readonly {
            return Ok(());
        }
        let current = self
            .cursor
            .get()
            .filter(|v| self.control.source.try_get().ok() == Some(*v))
            .unwrap_or(s.value);
        let next = match current.activate(date, deselect) {
            Ok(next) => next,
            Err(error) => {
                self.rejected.emit(self.owner, error);
                return Ok(());
            }
        };
        next.validate(s.value.mode(), s.provider.date_bounds())
            .map_err(|_| invalid())?;
        self.control.accept(&next)?;
        self.changed.emit(self.owner, next);
        self.ensure()?;
        self.cursor.set(Some(next));
        self.dirty.touch()
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
        self.cursor.set(None);
        self.changed.clear();
        self.rejected.clear();
    }
}
/// Inline civil date picker. Mode is fixed; changing a controlled selection must retain it.
pub struct DatePicker {
    pub(crate) common: Common,
    key: String,
    provider: Vary<DatesProvider>,
    mode: DatePickerMode,
    value: Value<DateSelection>,
    default_set: bool,
    month: Option<Vary<YearMonth>>,
    default_month: Option<YearMonth>,
    readonly: Vary<bool>,
    direction: Direction,
    excluded: Vec<CivilDate>,
    weekdays: u8,
    deselect: bool,
    changed: Option<Box<dyn FnMut(DateSelection)>>,
    rejected: Option<Box<dyn FnMut(DateSelectionError)>>,
    month_changed: Option<Box<dyn FnMut(YearMonth)>>,
}
pub fn date_picker(
    key: impl Into<String>,
    provider: impl Into<Vary<DatesProvider>>,
) -> DatePicker {
    DatePicker {
        common: Common::default(),
        key: key.into(),
        provider: provider.into(),
        mode: DatePickerMode::Single,
        value: Value::default(),
        default_set: false,
        month: None,
        default_month: None,
        readonly: false.into(),
        direction: Direction::Ltr,
        excluded: Vec::new(),
        weekdays: 0,
        deselect: false,
        changed: None,
        rejected: None,
        month_changed: None,
    }
}
impl DatePicker {
    common_methods!();
    pub fn mode(mut self, value: DatePickerMode) -> Self {
        self.mode = value;
        self
    }
    pub fn value(mut self, value: impl Into<Vary<DateSelection>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: DateSelection) -> Self {
        self.value.default = value;
        self.default_set = true;
        self
    }
    pub fn month(mut self, value: impl Into<Vary<YearMonth>>) -> Self {
        self.month = Some(value.into());
        self
    }
    pub fn default_month(mut self, value: YearMonth) -> Self {
        self.default_month = Some(value);
        self
    }
    pub fn readonly(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn direction(mut self, value: Direction) -> Self {
        self.direction = value;
        self
    }
    pub fn allow_deselect(mut self, value: bool) -> Self {
        self.deselect = value;
        self
    }
    pub fn exclude_dates(mut self, value: impl IntoIterator<Item = CivilDate>) -> Self {
        self.excluded = value
            .into_iter()
            .take(crate::MAX_CALENDAR_EXCLUDED_DATES + 1)
            .collect();
        self
    }
    pub fn exclude_date(mut self, value: CivilDate) -> Self {
        if self.excluded.len() <= crate::MAX_CALENDAR_EXCLUDED_DATES {
            self.excluded.push(value);
        }
        self
    }
    pub fn disable_weekday(mut self, value: Weekday) -> Self {
        self.weekdays |= 1 << value.index();
        self
    }
    pub fn on_value_change(mut self, callback: impl FnMut(DateSelection) + 'static) -> Self {
        self.changed = Some(Box::new(callback));
        self
    }
    pub fn on_reject(mut self, callback: impl FnMut(DateSelectionError) + 'static) -> Self {
        self.rejected = Some(Box::new(callback));
        self
    }
    pub fn on_month_change(mut self, callback: impl FnMut(YearMonth) + 'static) -> Self {
        self.month_changed = Some(Box::new(callback));
        self
    }
}
impl Widget for DatePicker {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.value.source.is_none() && !self.default_set {
            self.value.default = DateSelection::empty(self.mode);
        }
        let control = std::mem::take(&mut self.value).mount(scope);
        let provider = self.provider.clone();
        let value = control.source.clone();
        let enabled = self.common.enabled.clone();
        let readonly = self.readonly.clone();
        let source = scope.bond(move || Snapshot {
            value: value.get(),
            provider: provider.get(),
            enabled: enabled.get(),
            readonly: readonly.get(),
        });
        let mode = self.mode;
        let accepted = composite::validated(scope, &source.into(), move |s: &Snapshot| {
            s.value
                .validate(mode, s.provider.date_bounds())
                .map_err(|_| invalid())
        })?;
        let state = Rc::new(Controller {
            owner: scope.id(),
            weak: scope.downgrade(),
            live: Cell::new(true),
            control,
            accepted,
            cursor: Cell::new(None),
            dirty: Dirty::new(scope),
            changed: Callback::new(self.changed.take()),
            rejected: Callback::new(self.rejected.take()),
        });
        let cleanup = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(s) = cleanup.upgrade() {
                s.retire();
            }
        })?;
        let weak = Rc::downgrade(&state);
        let dirty = state.dirty.clone();
        scope.effect(move || {
            if dirty.take()
                && let Some(s) = weak.upgrade()
            {
                s.cursor.set(None);
            }
        });
        let s = state.clone();
        let deselect = self.deselect;
        let mut calendar = crate::calendar(
            std::mem::take(&mut self.key),
            scope.bond(move || accepted.with(|s| s.provider)),
        )
        .selection_projection(scope.bond(move || accepted.with(|s| s.value)))
        .readonly(scope.bond(move || accepted.with(|s| s.readonly)))
        .direction(self.direction)
        .exclude_dates(std::mem::take(&mut self.excluded))
        .on_value_change(move |date| {
            if let Some(date) = date {
                s.report(s.select(date, deselect));
            }
        });
        calendar.common = std::mem::take(&mut self.common);
        calendar.common.enabled = scope.bond(move || accepted.with(|s| s.enabled)).into();
        if let Some(month) = self.month.take() {
            calendar = calendar.month(month);
        }
        if let Some(month) = self.default_month {
            calendar = calendar.default_month(month);
        }
        if let Some(callback) = self.month_changed.take() {
            calendar = calendar.on_month_change(callback);
        }
        for day in Weekday::ALL {
            if self.weekdays & (1 << day.index()) != 0 {
                calendar = calendar.disable_weekday(day);
            }
        }
        calendar.show_in(scope)?;
        scope.retain(state)
    }
}
