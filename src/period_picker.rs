//! Finite month/year grids using the shared physical Grid/Button protocol.
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback, Observers};
use crate::state::{Control, Dirty, Value};
use crate::{
    CivilYear, DatePeriod, DatePickerMode, DateSelectionError, DatesProvider, PeriodBounds,
    PeriodSelection, YearMonth,
};
use glory::render::{
    Current, Direction, Event, EventKind, FocusMode, GridCommand, NodeId, Role, SemanticId,
    Semantics, TableMetadata,
};
use glory::style::{
    Align, BorderStyle, Display, Edges, FlexDirection, Justify, Length, Position, Style,
};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Scope, Vary};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
pub const MAX_EXCLUDED_PERIODS: usize = 128;
pub const MAX_PERIOD_SEARCH_STEPS: usize = MAX_EXCLUDED_PERIODS + 1;
fn invalid() -> glory::ui::Error {
    glory::render::Error::InvalidProps("invalid finite month/year picker").into()
}
fn count<T: DatePeriod>() -> usize {
    if T::MONTH { 12 } else { 10 }
}
fn columns<T: DatePeriod>() -> usize {
    if T::MONTH { 3 } else { 5 }
}
fn year<T: DatePeriod>(value: T) -> CivilYear {
    CivilYear::new(
        (if T::MONTH {
            value.ordinal() / 12 + 1
        } else {
            value.ordinal() + 1
        }) as u16,
    )
    .unwrap()
}
fn same_page<T: DatePeriod>(a: CivilYear, b: CivilYear) -> bool {
    if T::MONTH {
        a == b
    } else {
        a.value() / 10 == b.value() / 10
    }
}
fn cells<T: DatePeriod>(page: CivilYear) -> [Option<T>; 12] {
    std::array::from_fn(|index| {
        if index >= count::<T>() {
            None
        } else if T::MONTH {
            T::from_ordinal((u32::from(page.value()) - 1) * 12 + index as u32).ok()
        } else {
            let y = u32::from(page.value() / 10) * 10 + index as u32;
            y.checked_sub(1).and_then(|v| T::from_ordinal(v).ok())
        }
    })
}
fn index<T: DatePeriod>(value: T) -> usize {
    if T::MONTH {
        (value.ordinal() % 12) as usize
    } else {
        ((value.ordinal() + 1) % 10) as usize
    }
}
fn label<T: DatePeriod>(value: T, provider: DatesProvider, full: bool) -> String {
    if T::MONTH {
        let month = YearMonth::from_ordinal(value.ordinal()).unwrap();
        if full {
            provider.locale_value().month_label(month)
        } else {
            provider.locale_value().month_name(month).into()
        }
    } else {
        value.to_string()
    }
}
fn page_label<T: DatePeriod>(page: CivilYear) -> String {
    if T::MONTH {
        page.to_string()
    } else {
        let start = page.value() / 10 * 10;
        format!("{:04}–{:04}", start.max(1), start + 9)
    }
}
#[derive(Clone, Copy, PartialEq)]
struct Snapshot<T: DatePeriod> {
    provider: DatesProvider,
    page: CivilYear,
    value: PeriodSelection<T>,
    cells: [Option<T>; 12],
    enabled: bool,
    readonly: bool,
}
impl<T: DatePeriod> Snapshot<T> {
    fn bounds(self) -> PeriodBounds<T> {
        PeriodBounds::from_dates(self.provider.date_bounds())
    }
}
#[derive(Clone, Copy)]
struct Cursor<T: DatePeriod> {
    from: NodeId,
    value: T,
    page: CivilYear,
    provider: DatesProvider,
}
struct Controller<T: DatePeriod> {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    live: Cell<bool>,
    accepted: Cage<Snapshot<T>>,
    provider: Vary<DatesProvider>,
    value: Control<PeriodSelection<T>>,
    page: Control<CivilYear>,
    excluded: Vec<T>,
    deselect: bool,
    focused: Cage<Option<T>>,
    physical: Cage<Option<NodeId>>,
    references: [Cage<Option<NodeId>>; 12],
    cursor: RefCell<Option<Cursor<T>>>,
    selection_cursor: Cell<Option<PeriodSelection<T>>>,
    intent: Cell<Option<T>>,
    dirty: Dirty,
    changed: Callback<PeriodSelection<T>>,
    page_changed: Callback<CivilYear>,
    rejected: Callback<DateSelectionError>,
}
impl<T: DatePeriod> Controller<T> {
    fn ensure(&self) -> Result {
        if self.live.get() && self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn allowed(&self, s: Snapshot<T>, value: T) -> bool {
        s.enabled && s.bounds().contains(value) && self.excluded.binary_search(&value).is_err()
    }
    fn report(&self, result: Result) {
        if let Err(e) = result
            && let Some(scope) = self.weak.upgrade()
        {
            scope.report_error(e);
        }
    }
    fn base(&self, from: NodeId, fallback: T) -> Result<(Snapshot<T>, T)> {
        let s = self.accepted.try_get()?;
        let cursor = *self.cursor.borrow();
        if let Some(c) = cursor
            && c.from == from
            && c.provider == s.provider
            && self.provider.try_get()? == c.provider
            && self.page.source.try_get()? == c.page
        {
            Ok((
                Snapshot {
                    page: c.page,
                    cells: cells(c.page),
                    ..s
                },
                c.value,
            ))
        } else {
            Ok((s, fallback))
        }
    }
    fn nearest(&self, s: Snapshot<T>, preferred: usize) -> Option<T> {
        for distance in 0..12i32 {
            for sign in [1, -1] {
                if distance == 0 && sign < 0 {
                    continue;
                }
                let index = preferred as i32 + distance * sign;
                if index >= 0
                    && let Some(value) = s.cells.get(index as usize).copied().flatten()
                    && self.allowed(s, value)
                {
                    return Some(value);
                }
            }
        }
        None
    }
    fn move_to(&self, from: NodeId, s: Snapshot<T>, value: T, focus: bool) -> Result {
        self.ensure()?;
        if !self.allowed(s, value) {
            return Ok(());
        }
        let mut page = s.page;
        if !same_page::<T>(page, year(value)) {
            page = year(value);
            self.page.accept(&page)?;
            self.page_changed.emit(self.owner, page);
            self.ensure()?;
            if self.page.source.try_get()? != page {
                self.cursor.borrow_mut().take();
                return Ok(());
            }
        }
        self.focused.try_revise(|v| *v = Some(value))?;
        self.cursor.borrow_mut().replace(Cursor {
            from,
            value,
            page,
            provider: s.provider,
        });
        if focus {
            self.intent.set(Some(value));
        }
        self.dirty.touch()
    }
    fn navigate(&self, from: NodeId, command: GridCommand) -> Result {
        self.ensure()?;
        let s = self.accepted.try_get()?;
        let Some(at) = self
            .references
            .iter()
            .position(|r| r.try_get().ok().flatten() == Some(from))
        else {
            return Ok(());
        };
        let Some(origin) = s.cells[at] else {
            return Ok(());
        };
        let (s, current) = self.base(from, origin)?;
        if !self.allowed(s, current) {
            return Ok(());
        }
        let cols = columns::<T>() as i32;
        let delta = match command {
            GridCommand::Left => -1,
            GridCommand::Right => 1,
            GridCommand::Up => -cols,
            GridCommand::Down => cols,
            _ => 0,
        };
        if delta != 0 {
            let mut target = current;
            for _ in 0..MAX_PERIOD_SEARCH_STEPS {
                let Ok(next) = target.checked_shift(delta) else {
                    return Ok(());
                };
                target = next;
                if !s.bounds().contains(target) {
                    return Ok(());
                }
                if self.allowed(s, target) {
                    return self.move_to(from, s, target, true);
                }
            }
            return Ok(());
        }
        match command {
            GridCommand::FirstInRow | GridCommand::LastInRow => {
                let col = (index(current) % columns::<T>()) as i32;
                let (delta, step) = if command == GridCommand::FirstInRow {
                    (-col, 1)
                } else {
                    (cols - 1 - col, -1)
                };
                for n in 0..cols {
                    if let Ok(value) = current.checked_shift(delta + n * step)
                        && self.allowed(s, value)
                    {
                        return self.move_to(from, s, value, true);
                    }
                }
                Ok(())
            }
            GridCommand::PreviousPage
            | GridCommand::NextPage
            | GridCommand::PreviousLargePage
            | GridCommand::NextLargePage => {
                let amount = if T::MONTH { 12 } else { 10 };
                let delta = match command {
                    GridCommand::PreviousPage => -amount,
                    GridCommand::NextPage => amount,
                    GridCommand::PreviousLargePage => -amount * 10,
                    _ => amount * 10,
                };
                let Ok(target) = current.checked_shift(delta) else {
                    return Ok(());
                };
                let page = year(target);
                let next = Snapshot {
                    page,
                    cells: cells(page),
                    ..s
                };
                if let Some(value) = self.nearest(next, index(target)) {
                    self.move_to(from, s, value, true)
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
    fn shift(&self, from: NodeId, years: i32) -> Result {
        self.ensure()?;
        let s = self.accepted.try_get()?;
        if !s.enabled {
            return Ok(());
        }
        let Ok(page) = s.page.checked_add(years) else {
            return Ok(());
        };
        let next = Snapshot {
            page,
            cells: cells(page),
            ..s
        };
        if !next
            .cells
            .iter()
            .flatten()
            .any(|v| next.bounds().contains(*v))
        {
            return Ok(());
        }
        self.page.accept(&page)?;
        self.page_changed.emit(self.owner, page);
        self.ensure()?;
        if self.page.source.try_get()? == page {
            let preferred = self.focused.try_get()?.map(index).unwrap_or(0);
            let focused = self.nearest(next, preferred);
            self.focused.try_revise(|v| *v = focused)?;
            if let Some(value) = focused {
                self.cursor.borrow_mut().replace(Cursor {
                    from,
                    value,
                    page,
                    provider: s.provider,
                });
            }
            self.dirty.touch()?;
        }
        Ok(())
    }
    fn select(&self, at: usize) -> Result {
        self.ensure()?;
        let s = self.accepted.try_get()?;
        let Some(origin) = s.cells[at] else {
            return Ok(());
        };
        let Some(from) = self.references[at].try_get()? else {
            return Ok(());
        };
        let (s, value) = self.base(from, origin)?;
        if s.readonly || !self.allowed(s, value) {
            return Ok(());
        }
        let current = self
            .selection_cursor
            .get()
            .filter(|v| self.value.source.try_get().ok() == Some(*v))
            .unwrap_or(s.value);
        let next = match current.activate(value, self.deselect) {
            Ok(v) => v,
            Err(e) => {
                self.rejected.emit(self.owner, e);
                return Ok(());
            }
        };
        self.focused.try_revise(|v| *v = Some(value))?;
        self.value.accept(&next)?;
        self.changed.emit(self.owner, next);
        self.ensure()?;
        self.selection_cursor.set(Some(next));
        self.dirty.touch()
    }
    fn retire(&self) {
        self.live.set(false);
        self.cursor.borrow_mut().take();
        self.selection_cursor.set(None);
        self.intent.set(None);
        self.changed.clear();
        self.page_changed.clear();
        self.rejected.clear();
    }
}
/// The domain is sealed to YearMonth/CivilYear; it cannot carry arbitrary business data.
pub struct PeriodPicker<T: DatePeriod> {
    pub(crate) common: Common,
    key: String,
    provider: Vary<DatesProvider>,
    mode: DatePickerMode,
    value: Value<PeriodSelection<T>>,
    default_set: bool,
    page: Option<Vary<CivilYear>>,
    default_page: Option<CivilYear>,
    readonly: Vary<bool>,
    direction: Direction,
    excluded: Vec<T>,
    deselect: bool,
    changed: Option<Box<dyn FnMut(PeriodSelection<T>)>>,
    page_changed: Option<Box<dyn FnMut(CivilYear)>>,
    rejected: Option<Box<dyn FnMut(DateSelectionError)>>,
}
pub type MonthPicker = PeriodPicker<YearMonth>;
pub type YearPicker = PeriodPicker<CivilYear>;
pub(crate) fn period_picker<T: DatePeriod>(
    key: impl Into<String>,
    provider: impl Into<Vary<DatesProvider>>,
) -> PeriodPicker<T> {
    PeriodPicker {
        common: Common::default(),
        key: key.into(),
        provider: provider.into(),
        mode: DatePickerMode::Single,
        value: Value::default(),
        default_set: false,
        page: None,
        default_page: None,
        readonly: false.into(),
        direction: Direction::Ltr,
        excluded: Vec::new(),
        deselect: false,
        changed: None,
        page_changed: None,
        rejected: None,
    }
}
pub fn month_picker(
    key: impl Into<String>,
    provider: impl Into<Vary<DatesProvider>>,
) -> MonthPicker {
    period_picker(key, provider)
}
pub fn year_picker(
    key: impl Into<String>,
    provider: impl Into<Vary<DatesProvider>>,
) -> YearPicker {
    period_picker(key, provider)
}
impl<T: DatePeriod> PeriodPicker<T> {
    common_methods!();
    pub fn mode(mut self, v: DatePickerMode) -> Self {
        self.mode = v;
        self
    }
    pub fn value(mut self, v: impl Into<Vary<PeriodSelection<T>>>) -> Self {
        self.value.source = Some(v.into());
        self
    }
    pub fn default_value(mut self, v: PeriodSelection<T>) -> Self {
        self.value.default = v;
        self.default_set = true;
        self
    }
    /// Month view's year, or any year inside the visible ten-year view.
    pub fn year(mut self, v: impl Into<Vary<CivilYear>>) -> Self {
        self.page = Some(v.into());
        self
    }
    pub fn default_year(mut self, v: CivilYear) -> Self {
        self.default_page = Some(v);
        self
    }
    pub fn readonly(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.readonly = v.into();
        self
    }
    pub fn direction(mut self, v: Direction) -> Self {
        self.direction = v;
        self
    }
    pub fn allow_deselect(mut self, v: bool) -> Self {
        self.deselect = v;
        self
    }
    pub fn exclude_values(mut self, v: impl IntoIterator<Item = T>) -> Self {
        self.excluded = v.into_iter().take(MAX_EXCLUDED_PERIODS + 1).collect();
        self
    }
    pub fn exclude_value(mut self, v: T) -> Self {
        if self.excluded.len() <= MAX_EXCLUDED_PERIODS {
            self.excluded.push(v);
        }
        self
    }
    pub fn on_value_change(mut self, v: impl FnMut(PeriodSelection<T>) + 'static) -> Self {
        self.changed = Some(Box::new(v));
        self
    }
    pub fn on_year_change(mut self, v: impl FnMut(CivilYear) + 'static) -> Self {
        self.page_changed = Some(Box::new(v));
        self
    }
    pub fn on_reject(mut self, v: impl FnMut(DateSelectionError) + 'static) -> Self {
        self.rejected = Some(Box::new(v));
        self
    }
}
fn place(scope: &Scope, r: Cage<Option<NodeId>>) -> Result<Scope> {
    scope.child_in(r.get().ok_or(glory::ui::Error::Gone)?)
}
impl<T: DatePeriod> Widget for PeriodPicker<T> {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.key.is_empty()
            || self.key.len() > 64
            || !self
                .key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || self.excluded.len() > MAX_EXCLUDED_PERIODS
        {
            return Err(invalid());
        }
        let id = SemanticId::new(format!("{}-grid", self.key))?;
        let mut excluded = std::mem::take(&mut self.excluded);
        excluded.sort_unstable();
        excluded.dedup();
        if self.value.source.is_none() && !self.default_set {
            self.value.default = PeriodSelection::empty(self.mode);
        }
        let value = std::mem::take(&mut self.value).mount(scope);
        let initial = self.provider.try_get()?;
        let default_page = self.default_page.unwrap_or_else(|| {
            CivilYear::new(initial.date_bounds().clamp(initial.reference_date()).year())
                .unwrap()
        });
        let page = Value {
            source: self.page.take(),
            default: default_page,
        }
        .mount(scope);
        let provider = self.provider.clone();
        let p = provider.clone();
        let source = value.source.clone();
        let year = page.source.clone();
        let enabled = self.common.enabled.clone();
        let readonly = self.readonly.clone();
        let mode = self.mode;
        let source = scope.bond(move || {
            let provider = p.get();
            let page = year.get();
            Snapshot {
                provider,
                page,
                value: source.get(),
                cells: cells(page),
                enabled: enabled.get(),
                readonly: readonly.get(),
            }
        });
        let accepted = composite::validated(scope, &source.into(), move |s: &Snapshot<T>| {
            s.value.validate(mode, s.bounds()).map_err(|_| invalid())?;
            if !s.cells.iter().flatten().any(|v| s.bounds().contains(*v)) {
                Err(invalid())
            } else {
                Ok(())
            }
        })?;
        let s = accepted.get();
        let focused = scope.cage(
            s.value
                .anchor()
                .filter(|v| s.cells.contains(&Some(*v)))
                .or_else(|| {
                    s.cells
                        .iter()
                        .flatten()
                        .find(|v| {
                            s.bounds().contains(**v) && excluded.binary_search(v).is_err()
                        })
                        .copied()
                }),
        );
        let state = Rc::new(Controller {
            owner: scope.id(),
            weak: scope.downgrade(),
            live: Cell::new(true),
            accepted,
            provider,
            value,
            page,
            excluded,
            deselect: self.deselect,
            focused,
            physical: scope.cage(None),
            references: std::array::from_fn(|_| scope.cage(None)),
            cursor: RefCell::new(None),
            selection_cursor: Cell::new(None),
            intent: Cell::new(None),
            dirty: Dirty::new(scope),
            changed: Callback::new(self.changed.take()),
            page_changed: Callback::new(self.page_changed.take()),
            rejected: Callback::new(self.rejected.take()),
        });
        let cleanup = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(s) = cleanup.upgrade() {
                s.retire();
            }
        })?;
        let theme = self.common.theme.clone();
        let themed = !self.common.unstyled;
        let height = match theme.control(self.common.size).min_height {
            Some(Length::Px(v)) => v,
            _ => 36.,
        };
        let width = if T::MONTH {
            height.max(30.) * 2.5
        } else {
            height.max(24.) * 1.5
        };
        let gap = 2.;
        let cols = columns::<T>();
        let rows = count::<T>() / cols;
        let total = cols as f64 * (width + gap) - gap;
        let (class, appearance) = self.common.appearance(
            scope,
            if T::MONTH {
                "sapid-month-picker"
            } else {
                "sapid-year-picker"
            },
            Style {
                background: Some(theme.surface),
                color: Some(theme.foreground),
                ..Style::new()
            }
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(8.),
        );
        let outer = scope.cage(None);
        glory::ui::view()
            .class(class)
            .style(scope.bond(move || {
                let mut style = appearance.get();
                style.width.get_or_insert(Length::Px(total));
                style.display.get_or_insert(Display::Flex);
                style.flex_direction.get_or_insert(FlexDirection::Column);
                style.gap.get_or_insert(8.);
                style
            }))
            .node_ref(outer)
            .show_in(scope)?;
        let content = place(scope, outer)?;
        let header = scope.cage(None);
        glory::ui::view()
            .class("sapid-period-header")
            .style(
                Style {
                    align: Some(Align::Center),
                    justify: Some(Justify::Between),
                    ..Style::new()
                }
                .display(Display::Flex)
                .width(Length::Px(total))
                .height(Length::Px(height)),
            )
            .node_ref(header)
            .show_in(&content)?;
        let hs = place(scope, header)?;
        let activate = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Activate)
                .unwrap_or_default(),
        );
        let cleanup = activate.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let stride = if T::MONTH { 1 } else { 10 };
        for delta in [-stride, stride] {
            let s = state.clone();
            let r = scope.cage(None);
            let observers = activate.clone();
            let mut style = Style::new()
                .width(Length::Px(height))
                .height(Length::Px(height))
                .padding(Edges::all(0.));
            if themed {
                style.background = Some(theme.surface);
                style.color = Some(theme.foreground);
                style = style
                    .border_width(Edges::all(1.))
                    .border_style(BorderStyle::Solid)
                    .border_color(theme.border);
            }
            glory::ui::button()
                .label(if delta < 0 { "‹" } else { "›" })
                .name(if T::MONTH {
                    if delta < 0 {
                        "Previous year"
                    } else {
                        "Next year"
                    }
                } else if delta < 0 {
                    "Previous 10 years"
                } else {
                    "Next 10 years"
                })
                .class("sapid-period-nav")
                .style(style)
                .node_ref(r)
                .enabled(scope.bond(move || {
                    let s = accepted.get();
                    s.enabled
                        && s.page.checked_add(delta).is_ok_and(|page| {
                            cells::<T>(page)
                                .iter()
                                .flatten()
                                .any(|v| s.bounds().contains(*v))
                        })
                }))
                .on(EventKind::Activate, move |event| {
                    if s.ensure().is_ok() {
                        if let Some(id) = r.get() {
                            s.report(s.shift(id, delta));
                        }
                        observers.emit(s.owner, event);
                    }
                })
                .show_in(&hs)?;
            if delta < 0 {
                glory::ui::text()
                    .content(scope.bond(move || page_label::<T>(accepted.with(|s| s.page))))
                    .show_in(&hs)?;
            }
        }
        let grid_ref = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let direction = self.direction;
        let s = state.clone();
        let observers = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::GridNavigate)
                .unwrap_or_default(),
        );
        let cleanup = observers.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let mut grid = glory::ui::view()
            .role(Role::Grid)
            .class("sapid-period-grid")
            .node_ref(grid_ref)
            .focus(self.common.focus)
            .name(self.common.name.take().unwrap_or_else(|| {
                scope
                    .bond(move || page_label::<T>(accepted.with(|s| s.page)))
                    .into()
            }))
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    id: Some(id.clone()),
                    grid_navigation: Some(direction),
                    grid_multiple: mode != DatePickerMode::Single,
                    readonly: accepted.with(|s| s.readonly),
                    table: Some(TableMetadata::Table {
                        rows: Some(rows as u32),
                        columns: Some(cols as u16),
                    }),
                    ..Default::default()
                }))
            }))
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(gap),
            )
            .on(EventKind::GridNavigate, move |e| {
                if s.ensure().is_ok() {
                    if let Event::GridNavigate { from, command } = &e {
                        s.report(s.navigate(*from, *command));
                    }
                    observers.emit(s.owner, e);
                }
            });
        for (kind, mut callbacks) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            grid = grid.on(kind, move |e| {
                crate::appearance::observe(&mut callbacks, owner, e)
            });
        }
        grid.show_in(&content)?;
        let gs = place(scope, grid_ref)?;
        for row in 0..rows {
            let reference = scope.cage(None);
            glory::ui::view()
                .role(Role::Row)
                .style(
                    Style::new()
                        .position(Position::Relative)
                        .width(Length::Px(total))
                        .height(Length::Px(height)),
                )
                .node_ref(reference)
                .show_in(&gs)?;
            let row_scope = place(scope, reference)?;
            for col in 0..cols {
                let at = row * cols + col;
                let reference = scope.cage(None);
                let physical = if direction == Direction::Ltr {
                    col
                } else {
                    cols - 1 - col
                };
                glory::ui::view()
                    .role(Role::GridCell)
                    .class("sapid-period-cell")
                    .style(
                        Style::new()
                            .position(Position::Absolute)
                            .left(Length::Px(physical as f64 * (width + gap)))
                            .top(Length::Px(0.))
                            .width(Length::Px(width))
                            .height(Length::Px(height)),
                    )
                    .semantics(scope.bond(move || {
                        Some(Box::new(Semantics {
                            selected: Some(
                                accepted
                                    .with(|s| s.cells[at].is_some_and(|v| s.value.contains(v))),
                            ),
                            table: Some(TableMetadata::Cell {
                                row: row as u32,
                                column: col as u16,
                                row_span: 1,
                                column_span: 1,
                                headers: vec![],
                            }),
                            ..Default::default()
                        }))
                    }))
                    .node_ref(reference)
                    .show_in(&row_scope)?;
                let cell_scope = place(scope, reference)?;
                let s = state.clone();
                let theme = theme.clone();
                let observers = activate.clone();
                glory::ui::when(
                    scope.bond(move || accepted.with(|s| s.cells[at].is_some())),
                    move || PeriodButton {
                        state: s.clone(),
                        index: at,
                        width,
                        height,
                        theme: theme.clone(),
                        themed,
                        observers: observers.clone(),
                    },
                )
                .show_in(&cell_scope)?;
                scope.retain(cell_scope)?;
            }
            scope.retain(row_scope)?;
        }
        let weak = Rc::downgrade(&state);
        let owner = scope.downgrade();
        let dirty = state.dirty.clone();
        scope.effect(move || {
            let (Some(s), Some(scope)) = (weak.upgrade(), owner.upgrade()) else {
                return;
            };
            let result = (|| -> Result {
                let edited = dirty.take();
                let snap = accepted.get();
                let current = focused.get();
                if current.is_none_or(|v| !snap.cells.contains(&Some(v)) || !s.allowed(snap, v))
                {
                    let next = s.nearest(snap, current.map(index).unwrap_or(0));
                    if next != current {
                        focused.try_revise(|v| *v = next)?;
                    }
                }
                if edited {
                    s.cursor.borrow_mut().take();
                    s.selection_cursor.set(None);
                }
                if let Some(target) = s.intent.take()
                    && let Some(at) = snap.cells.iter().position(|v| *v == Some(target))
                    && let Some(id) = s.references[at].get()
                {
                    composite::focus_node(&scope, Some(id))?;
                }
                Ok(())
            })();
            s.report(result);
        });
        scope.retain(hs)?;
        scope.retain(gs)?;
        scope.retain(content)?;
        scope.retain(state)
    }
}
struct PeriodButton<T: DatePeriod> {
    state: Rc<Controller<T>>,
    index: usize,
    width: f64,
    height: f64,
    theme: crate::Theme,
    themed: bool,
    observers: Observers,
}
impl<T: DatePeriod> Widget for PeriodButton<T> {
    fn build(&mut self, scope: &Scope) -> Result {
        let at = self.index;
        let s = self.state.clone();
        let accepted = s.accepted;
        let focused = s.focused;
        let physical = s.physical;
        let reference = s.references[at];
        let enabled = s.clone();
        let style_state = s.clone();
        let selected = s.clone();
        let focus_state = s.clone();
        let observers = self.observers.clone();
        let theme = self.theme.clone();
        let themed = self.themed;
        let width = self.width;
        let height = self.height;
        glory::ui::button()
            .class("sapid-period-button")
            .node_ref(reference)
            .label(scope.bond(move || {
                accepted.with(|s| {
                    s.cells[at]
                        .map(|v| label(v, s.provider, false))
                        .unwrap_or_default()
                })
            }))
            .name(scope.bond(move || {
                accepted.with(|s| {
                    s.cells[at]
                        .map(|v| label(v, s.provider, true))
                        .unwrap_or_default()
                })
            }))
            .enabled(scope.bond(move || {
                accepted.with(|s| s.cells[at].is_some_and(|v| enabled.allowed(*s, v)))
            }))
            .focus(scope.bond(move || {
                if accepted.with(|s| s.cells[at]) == focused.get() {
                    FocusMode::TabStop
                } else {
                    FocusMode::Programmatic
                }
            }))
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    current: accepted.with(|s| {
                        s.cells[at]
                            .filter(|v| s.provider.today_date().map(T::from_date) == Some(*v))
                            .map(|_| Current::Date)
                    }),
                    ..Default::default()
                }))
            }))
            .style(scope.bond(move || {
                let snap = accepted.get();
                let value = snap.cells[at];
                let chosen = value.is_some_and(|v| snap.value.contains(v));
                let enabled = value.is_some_and(|v| style_state.allowed(snap, v));
                let actual = reference.get().is_some_and(|id| physical.get() == Some(id));
                let mut style = Style::new()
                    .width(Length::Px(width))
                    .height(Length::Px(height))
                    .padding(Edges::all(0.))
                    .min_width(Length::Px(0.));
                if themed {
                    style.font_size = Some(if height < 30. { 11. } else { 14. });
                    style.background = Some(if chosen { theme.accent } else { theme.surface });
                    style.color = Some(if !enabled {
                        theme.disabled_text
                    } else if chosen {
                        theme.on_accent
                    } else {
                        theme.foreground
                    });
                    style = style
                        .radius(theme.radius)
                        .border_width(Edges::all(1.))
                        .border_style(BorderStyle::Solid)
                        .border_color(if actual { theme.accent } else { theme.border });
                }
                style
            }))
            .on(EventKind::Activate, move |e| {
                if selected.ensure().is_ok() {
                    selected.report(selected.select(at));
                    observers.emit(selected.owner, e);
                }
            })
            .on(EventKind::FocusChanged, move |e| {
                if focus_state.ensure().is_ok()
                    && let Event::FocusChanged(active) = e
                {
                    let result = (|| -> Result {
                        if active {
                            focus_state.cursor.borrow_mut().take();
                            focused.try_revise(|v| *v = accepted.with(|s| s.cells[at]))?;
                            physical.try_revise(|v| *v = reference.get())?;
                        } else if physical.get() == reference.get() {
                            physical.try_revise(|v| *v = None)?;
                        }
                        Ok(())
                    })();
                    focus_state.report(result);
                }
            })
            .show_in(scope)
    }
}
