//! Finite civil calendar. The host owns physical focus, never the date model.
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback, Observers};
use crate::dates::*;
use crate::state::{Control, Dirty, Value};
use crate::{DatePickerMode, DateSelection};
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
pub const MAX_CALENDAR_EXCLUDED_DATES: usize = 366;
pub const MAX_CALENDAR_SEARCH_DAYS: usize = 366;
fn invalid() -> glory::ui::Error {
    glory::render::Error::InvalidProps("invalid finite calendar configuration").into()
}
#[derive(Clone, Copy, PartialEq)]
struct Snapshot {
    provider: DatesProvider,
    month: YearMonth,
    value: Option<CivilDate>,
    selection: DateSelection,
    grid: MonthGrid,
    enabled: bool,
    readonly: bool,
}
#[derive(Clone, Copy)]
struct Cursor {
    from: NodeId,
    date: CivilDate,
    month: YearMonth,
    provider: DatesProvider,
}
struct Controller {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    live: Cell<bool>,
    accepted: Cage<Snapshot>,
    value: Control<Option<CivilDate>>,
    month: Control<YearMonth>,
    provider: Vary<DatesProvider>,
    excluded: Vec<CivilDate>,
    weekdays: u8,
    allow_deselect: bool,
    selection_surface: bool,
    focused: Cage<CivilDate>,
    physical: Cage<Option<NodeId>>,
    references: [Cage<Option<NodeId>>; 42],
    cursor: RefCell<Option<Cursor>>,
    selection_cursor: Cell<Option<(NodeId, Option<CivilDate>)>>,
    intent: Cell<Option<CivilDate>>,
    dirty: Dirty,
    changed: Callback<Option<CivilDate>>,
    month_changed: Callback<YearMonth>,
}
impl Controller {
    fn ensure(&self) -> Result {
        if self.live.get() && self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn allowed(&self, s: Snapshot, date: CivilDate) -> bool {
        s.enabled
            && s.provider.date_bounds().contains(date)
            && self.weekdays & (1 << date.weekday().index()) == 0
            && self.excluded.binary_search(&date).is_err()
    }
    fn report(&self, result: Result) {
        if let Err(error) = result
            && let Some(scope) = self.weak.upgrade()
        {
            scope.report_error(error);
        }
    }
    fn date(&self, index: usize) -> Result<Option<CivilDate>> {
        Ok(self.accepted.try_with(|s| s.grid.cells()[index])?)
    }
    fn base(&self, from: NodeId, fallback: CivilDate) -> Result<(Snapshot, CivilDate)> {
        let s = self.accepted.try_get()?;
        let cursor = *self.cursor.borrow();
        if let Some(c) = cursor
            && c.from == from
            && c.provider == s.provider
            && self.provider.try_get()? == c.provider
            && self.month.source.try_get()? == c.month
        {
            Ok((
                Snapshot {
                    month: c.month,
                    ..s
                },
                c.date,
            ))
        } else {
            Ok((s, fallback))
        }
    }
    fn move_to(&self, from: NodeId, s: Snapshot, date: CivilDate, focus: bool) -> Result {
        self.ensure()?;
        if !self.allowed(s, date) {
            return Ok(());
        }
        let month = date.year_month();
        if month != s.month {
            self.month.accept(&month)?;
            self.month_changed.emit(self.owner, month);
            self.ensure()?;
            if self.month.source.try_get()? != month {
                self.cursor.borrow_mut().take();
                return Ok(());
            }
        }
        self.focused.try_revise(|v| *v = date)?;
        self.cursor.borrow_mut().replace(Cursor {
            from,
            date,
            month,
            provider: s.provider,
        });
        if focus {
            self.intent.set(Some(date));
        }
        self.dirty.touch()
    }
    fn first_allowed(&self, s: Snapshot, month: YearMonth, preferred: u8) -> Option<CivilDate> {
        let first = month.first();
        let day = preferred.min(month.days());
        // A single month's at most 31 days, with stable nearest-day tie order.
        for distance in 0..31i32 {
            for sign in [1, -1] {
                if distance == 0 && sign < 0 {
                    continue;
                }
                let d = i32::from(day) + distance * sign;
                if (1..=i32::from(month.days())).contains(&d) {
                    let date = first.checked_add_days(d - 1).ok()?;
                    if self.allowed(s, date) {
                        return Some(date);
                    }
                }
            }
        }
        None
    }
    fn navigate(&self, from: NodeId, command: GridCommand) -> Result {
        self.ensure()?;
        let s = self.accepted.try_get()?;
        let Some(index) = self
            .references
            .iter()
            .position(|r| r.try_get().ok().flatten() == Some(from))
        else {
            return Ok(());
        };
        let Some(origin) = s.grid.cells()[index] else {
            return Ok(());
        };
        let (s, current) = self.base(from, origin)?;
        if !self.allowed(s, current) {
            return Ok(());
        }
        let (delta, count) = match command {
            GridCommand::Left => (-1, MAX_CALENDAR_SEARCH_DAYS),
            GridCommand::Right => (1, MAX_CALENDAR_SEARCH_DAYS),
            GridCommand::Up => (-7, MAX_CALENDAR_SEARCH_DAYS / 7),
            GridCommand::Down => (7, MAX_CALENDAR_SEARCH_DAYS / 7),
            _ => (0, 0),
        };
        if delta != 0 {
            let mut target = current;
            for _ in 0..count {
                let Ok(next) = target.checked_add_days(delta) else {
                    return Ok(());
                };
                target = next;
                if !s.provider.date_bounds().contains(target) {
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
                let offset = i32::from(current.weekday().index_from(s.provider.first_weekday()));
                let (delta, step) = if command == GridCommand::FirstInRow {
                    (-offset, 1)
                } else {
                    (6 - offset, -1)
                };
                for i in 0..7 {
                    if let Ok(date) = current.checked_add_days(delta + i * step)
                        && self.allowed(s, date)
                    {
                        return self.move_to(from, s, date, true);
                    }
                }
                Ok(())
            }
            GridCommand::PreviousPage
            | GridCommand::NextPage
            | GridCommand::PreviousLargePage
            | GridCommand::NextLargePage => {
                let delta = match command {
                    GridCommand::PreviousPage => -1,
                    GridCommand::NextPage => 1,
                    GridCommand::PreviousLargePage => -12,
                    _ => 12,
                };
                let Ok(month) = current.year_month().checked_add(delta) else {
                    return Ok(());
                };
                if let Some(date) = self.first_allowed(s, month, current.day()) {
                    self.move_to(from, s, date, true)
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
    fn shift(&self, from: NodeId, delta: i32) -> Result {
        let fallback = self.focused.try_get()?;
        let (s, current) = self.base(from, fallback)?;
        let Ok(month) = s.month.checked_add(delta) else {
            return Ok(());
        };
        if !s.enabled || !s.provider.date_bounds().month_allowed(month) {
            return Ok(());
        }
        // Browsing a completely excluded month is permitted; its navigation
        // controls remain available even when there is no selectable day.
        let date = self
            .first_allowed(s, month, current.day())
            .unwrap_or(month.first());
        self.month.accept(&month)?;
        self.month_changed.emit(self.owner, month);
        self.ensure()?;
        if self.month.source.try_get()? == month {
            self.focused.try_revise(|v| *v = date)?;
            self.cursor.borrow_mut().replace(Cursor {
                from,
                date,
                month,
                provider: s.provider,
            });
            self.dirty.touch()?;
        }
        Ok(())
    }
    fn select(&self, index: usize) -> Result {
        self.ensure()?;
        let s = self.accepted.try_get()?;
        let Some(origin) = s.grid.cells()[index] else {
            return Ok(());
        };
        let Some(from) = self.references[index].try_get()? else {
            return Ok(());
        };
        let (s, date) = self.base(from, origin)?;
        if s.readonly || !self.allowed(s, date) {
            return Ok(());
        }
        if self.selection_surface {
            self.focused.try_revise(|v| *v = date)?;
            self.changed.emit(self.owner, Some(date));
            self.ensure()?;
            return self.dirty.touch();
        }
        let current = match self.selection_cursor.get() {
            Some((node, value)) if node == from && self.value.source.try_get()? == value => value,
            _ => s.value,
        };
        let next = if self.allow_deselect && current == Some(date) {
            None
        } else {
            Some(date)
        };
        self.focused.try_revise(|v| *v = date)?;
        self.value.accept(&next)?;
        self.changed.emit(self.owner, next);
        self.ensure()?;
        self.selection_cursor.set(Some((from, next)));
        self.dirty.touch()
    }
    fn retire(&self) {
        self.live.set(false);
        self.changed.clear();
        self.month_changed.clear();
        self.cursor.borrow_mut().take();
        self.selection_cursor.set(None);
        self.intent.set(None);
    }
}
pub struct Calendar {
    pub(crate) common: Common,
    key: String,
    provider: Vary<DatesProvider>,
    value: Value<Option<CivilDate>>,
    projection: Option<Vary<DateSelection>>,
    month: Option<Vary<YearMonth>>,
    default_month: Option<YearMonth>,
    readonly: Vary<bool>,
    direction: Direction,
    excluded: Vec<CivilDate>,
    weekdays: u8,
    allow_deselect: bool,
    mini: bool,
    changed: Option<Box<dyn FnMut(Option<CivilDate>)>>,
    month_changed: Option<Box<dyn FnMut(YearMonth)>>,
}
pub fn calendar(key: impl Into<String>, provider: impl Into<Vary<DatesProvider>>) -> Calendar {
    Calendar {
        common: Common::default(),
        key: key.into(),
        provider: provider.into(),
        value: Value::default(),
        projection: None,
        month: None,
        default_month: None,
        readonly: false.into(),
        direction: Direction::Ltr,
        excluded: Vec::new(),
        weekdays: 0,
        allow_deselect: false,
        mini: false,
        changed: None,
        month_changed: None,
    }
}
pub fn mini_calendar(key: impl Into<String>, provider: impl Into<Vary<DatesProvider>>) -> Calendar {
    let mut c = calendar(key, provider);
    c.common.size = crate::Size::ExtraSmall;
    c.mini = true;
    c
}
impl Calendar {
    common_methods!();
    pub(crate) fn selection_projection(mut self, value: impl Into<Vary<DateSelection>>) -> Self {
        self.projection = Some(value.into());
        self
    }
    pub fn value(mut self, value: impl Into<Vary<Option<CivilDate>>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: Option<CivilDate>) -> Self {
        self.value.default = value;
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
        self.allow_deselect = value;
        self
    }
    pub fn exclude_dates(mut self, value: impl IntoIterator<Item = CivilDate>) -> Self {
        self.excluded = value
            .into_iter()
            .take(MAX_CALENDAR_EXCLUDED_DATES + 1)
            .collect();
        self
    }
    pub fn exclude_date(mut self, value: CivilDate) -> Self {
        if self.excluded.len() <= MAX_CALENDAR_EXCLUDED_DATES {
            self.excluded.push(value);
        }
        self
    }
    pub fn disable_weekday(mut self, value: Weekday) -> Self {
        self.weekdays |= 1 << value.index();
        self
    }
    pub fn on_value_change(mut self, value: impl FnMut(Option<CivilDate>) + 'static) -> Self {
        self.changed = Some(Box::new(value));
        self
    }
    pub fn on_month_change(mut self, value: impl FnMut(YearMonth) + 'static) -> Self {
        self.month_changed = Some(Box::new(value));
        self
    }
}
fn row_style(width: f64, height: f64) -> Style {
    Style::new()
        .position(Position::Relative)
        .width(Length::Px(width))
        .height(Length::Px(height))
}
fn place(scope: &Scope, reference: Cage<Option<NodeId>>) -> Result<Scope> {
    scope.child_in(reference.try_get()?.ok_or(glory::ui::Error::Gone)?)
}
impl Widget for Calendar {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.key.is_empty()
            || self.key.len() > 251
            || !self
                .key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(invalid());
        }
        let id = SemanticId::new(format!("{}-grid", self.key))?;
        if self.excluded.len() > MAX_CALENDAR_EXCLUDED_DATES {
            return Err(invalid());
        }
        let mut excluded = std::mem::take(&mut self.excluded);
        excluded.sort_unstable();
        excluded.dedup();
        let provider = self.provider.clone();
        let initial_provider = provider.try_get()?;
        let default_month = self.default_month.unwrap_or_else(|| {
            initial_provider
                .date_bounds()
                .clamp(initial_provider.reference_date())
                .year_month()
        });
        let month = Value {
            source: self.month.take(),
            default: default_month,
        }
        .mount(scope);
        let value = std::mem::take(&mut self.value).mount(scope);
        let month_source = month.source.clone();
        let value_source = value.source.clone();
        let enabled = self.common.enabled.clone();
        let readonly = self.readonly.clone();
        let p = provider.clone();
        let projection = self.projection.take();
        let selection_surface = projection.is_some();
        let source = scope.bond(move || {
            let provider = p.get();
            let month = month_source.get();
            let selection = projection
                .as_ref()
                .map_or_else(|| DateSelection::Single(value_source.get()), Vary::get);
            Snapshot {
                provider,
                month,
                value: selection.anchor(),
                selection,
                grid: MonthGrid::new(month, provider.first_weekday()),
                enabled: enabled.get(),
                readonly: readonly.get(),
            }
        });
        let accepted = composite::validated(scope, &source.into(), |s: &Snapshot| {
            if s.selection
                .validate(s.selection.mode(), s.provider.date_bounds())
                .is_err()
                || !s.provider.date_bounds().month_allowed(s.month)
                || s.value
                    .is_some_and(|d| !s.provider.date_bounds().contains(d))
            {
                Err(invalid())
            } else {
                Ok(())
            }
        })?;
        let initial = accepted.try_get()?;
        let start = initial
            .value
            .filter(|d| d.year_month() == initial.month)
            .unwrap_or_else(|| {
                initial
                    .provider
                    .date_bounds()
                    .clamp(initial.provider.reference_date())
            });
        let focused = scope.cage(start);
        let physical = scope.cage(None);
        let references = std::array::from_fn(|_| scope.cage(None));
        let dirty = Dirty::new(scope);
        let state = Rc::new(Controller {
            owner: scope.id(),
            weak: scope.downgrade(),
            live: Cell::new(true),
            accepted,
            value,
            month,
            provider,
            excluded,
            weekdays: self.weekdays,
            allow_deselect: self.allow_deselect,
            selection_surface,
            focused,
            physical,
            references,
            cursor: RefCell::new(None),
            selection_cursor: Cell::new(None),
            intent: Cell::new(None),
            dirty: dirty.clone(),
            changed: Callback::new(self.changed.take()),
            month_changed: Callback::new(self.month_changed.take()),
        });
        let weak = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(s) = weak.upgrade() {
                s.retire();
            }
        })?;
        let size = match self.common.theme.control(self.common.size).min_height {
            Some(Length::Px(n)) => n,
            _ => 36.,
        };
        let gap = 2.;
        let pitch = size + gap;
        let width = 7. * pitch - gap;
        let themed = !self.common.unstyled;
        let theme = self.common.theme.clone();
        let direction = self.direction;
        let (classes, appearance) = self.common.appearance(
            scope,
            if self.mini {
                "sapid-calendar sapid-mini-calendar"
            } else {
                "sapid-calendar"
            },
            Style {
                color: Some(theme.foreground),
                background: Some(theme.surface),
                ..Style::new()
            }
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(8.),
        );
        let outer = scope.cage(None);
        glory::ui::view()
            .class(classes)
            .style(scope.bond(move || {
                let mut style = appearance.get();
                style.width.get_or_insert(Length::Px(width));
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
            .class("sapid-calendar-header")
            .style(
                Style {
                    justify: Some(Justify::Between),
                    align: Some(Align::Center),
                    ..Style::new()
                }
                .display(Display::Flex)
                .width(Length::Px(width))
                .height(Length::Px(size)),
            )
            .node_ref(header)
            .show_in(&content)?;
        let header_scope = place(scope, header)?;
        let activate = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Activate)
                .unwrap_or_default(),
        );
        let cleanup = activate.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        for (delta, label) in [
            (-12, "Previous year"),
            (-1, "Previous month"),
            (1, "Next month"),
            (12, "Next year"),
        ] {
            if delta == 1 {
                let mini = self.mini;
                glory::ui::text()
                    .class("sapid-calendar-month")
                    .content(scope.bond(move || {
                        let s = accepted.get();
                        if mini && s.provider.locale_value() == DateLocale::En {
                            format!(
                                "{} {}",
                                &s.provider.locale_value().month_name(s.month)[..3],
                                s.month.year()
                            )
                        } else {
                            s.provider.locale_value().month_label(s.month)
                        }
                    }))
                    .style(Style {
                        font_size: Some(if self.mini { 11. } else { 14. }),
                        ..Style::new()
                    })
                    .show_in(&header_scope)?;
            }
            let r = scope.cage(None);
            let s = state.clone();
            let observers = activate.clone();
            let border = theme.border;
            let foreground = theme.foreground;
            let mut style = Style::new()
                .width(Length::Px(size * 0.75))
                .height(Length::Px(size))
                .padding(Edges::all(0.));
            if themed {
                style.color = Some(foreground);
                style.background = Some(theme.surface);
                style = style
                    .border_width(Edges::all(1.))
                    .border_style(BorderStyle::Solid)
                    .border_color(border);
            }
            glory::ui::button()
                .label(match delta {
                    -12 => "«",
                    -1 => "‹",
                    1 => "›",
                    _ => "»",
                })
                .name(label)
                .class("sapid-calendar-nav")
                .style(style)
                .node_ref(r)
                .enabled(scope.bond(move || {
                    let s = accepted.get();
                    s.enabled
                        && s.month
                            .checked_add(delta)
                            .is_ok_and(|m| s.provider.date_bounds().month_allowed(m))
                }))
                .on(EventKind::Activate, move |event| {
                    if s.ensure().is_ok() {
                        if let Some(id) = r.get() {
                            s.report(s.shift(id, delta));
                        }
                        observers.emit(s.owner, event);
                    }
                })
                .show_in(&header_scope)?;
        }
        let grid_ref = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
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
            .class("sapid-calendar-grid")
            .node_ref(grid_ref)
            .focus(self.common.focus)
            .name(self.common.name.take().unwrap_or_else(|| {
                scope
                    .bond(move || accepted.with(|s| s.provider.locale_value().month_label(s.month)))
                    .into()
            }))
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    id: Some(id.clone()),
                    grid_navigation: Some(direction),
                    grid_multiple: accepted.with(|s| s.selection.mode() != DatePickerMode::Single),
                    readonly: accepted.with(|s| s.readonly),
                    table: Some(TableMetadata::Table {
                        rows: Some(7),
                        columns: Some(7),
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
            .on(EventKind::GridNavigate, move |event| {
                if s.ensure().is_ok() {
                    if let Event::GridNavigate { from, command } = &event {
                        s.report(s.navigate(*from, *command));
                    }
                    observers.emit(s.owner, event);
                }
            });
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            grid = grid.on(kind, move |event| {
                crate::appearance::observe(&mut handlers, owner, event)
            });
        }
        grid.show_in(&content)?;
        let grid_scope = place(scope, grid_ref)?;
        for row in 0..7usize {
            let r = scope.cage(None);
            let height = if row == 0 { size * 0.75 } else { size };
            glory::ui::view()
                .role(Role::Row)
                .class(if row == 0 {
                    "sapid-calendar-weekdays"
                } else {
                    "sapid-calendar-week"
                })
                .semantics(Some(Box::new(Semantics {
                    table: Some(TableMetadata::Row { index: row as u32 }),
                    ..Default::default()
                })))
                .style(row_style(width, height))
                .node_ref(r)
                .show_in(&grid_scope)?;
            let row_scope = place(scope, r)?;
            for column in 0..7usize {
                let left = (if direction == Direction::Rtl {
                    6 - column
                } else {
                    column
                }) as f64
                    * pitch;
                let cell_style = Style::new()
                    .position(Position::Absolute)
                    .left(Length::Px(left))
                    .top(Length::Px(0.))
                    .width(Length::Px(size))
                    .height(Length::Px(height));
                if row == 0 {
                    glory::ui::view()
                        .role(Role::ColumnHeader)
                        .style(Style {
                            display: Some(Display::Flex),
                            align: Some(Align::Center),
                            justify: Some(Justify::Center),
                            ..cell_style
                        })
                        .name(scope.bond(move || {
                            let s = accepted.get();
                            let day = Weekday::ALL
                                [(usize::from(s.provider.first_weekday().index()) + column) % 7];
                            s.provider.locale_value().weekday_long(day).to_owned()
                        }))
                        .semantics(Some(Box::new(Semantics {
                            table: Some(TableMetadata::Cell {
                                row: 0,
                                column: column as u16,
                                row_span: 1,
                                column_span: 1,
                                headers: vec![],
                            }),
                            ..Default::default()
                        })))
                        .fill(
                            glory::ui::text()
                                .content(scope.bond(move || {
                                    let s = accepted.get();
                                    let day = Weekday::ALL[(usize::from(
                                        s.provider.first_weekday().index(),
                                    ) + column)
                                        % 7];
                                    s.provider.locale_value().weekday_short(day).to_owned()
                                }))
                                .style(Style {
                                    font_size: Some(if self.mini { 10. } else { 12. }),
                                    ..Style::new()
                                }),
                        )
                        .show_in(&row_scope)?;
                } else {
                    let index = (row - 1) * 7 + column;
                    let r = scope.cage(None);
                    glory::ui::view()
                        .role(Role::GridCell)
                        .class("sapid-calendar-cell")
                        .style(cell_style)
                        .node_ref(r)
                        .semantics(scope.bond(move || {
                            Some(Box::new(Semantics {
                                selected: Some(accepted.with(|s| {
                                    s.grid.cells()[index].is_some_and(|d| s.selection.contains(d))
                                })),
                                table: Some(TableMetadata::Cell {
                                    row: row as u32,
                                    column: column as u16,
                                    row_span: 1,
                                    column_span: 1,
                                    headers: vec![],
                                }),
                                ..Default::default()
                            }))
                        }))
                        .show_in(&row_scope)?;
                    let cell_scope = place(scope, r)?;
                    let s = state.clone();
                    let theme = theme.clone();
                    let observers = activate.clone();
                    glory::ui::when(
                        scope.bond(move || accepted.with(|s| s.grid.cells()[index].is_some())),
                        move || DayButton {
                            state: s.clone(),
                            index,
                            size,
                            themed,
                            theme: theme.clone(),
                            observers: observers.clone(),
                        },
                    )
                    .show_in(&cell_scope)?;
                    scope.retain(cell_scope)?;
                }
            }
            scope.retain(row_scope)?;
        }
        scope.retain(header_scope)?;
        scope.retain(grid_scope)?;
        scope.retain(content)?;
        let weak = Rc::downgrade(&state);
        let owner = scope.downgrade();
        let mut previous_month = initial.month;
        scope.effect(move || {
            let (Some(s), Some(scope)) = (weak.upgrade(), owner.upgrade()) else {
                return;
            };
            let result = (|| -> Result {
                dirty.take();
                let current = s.accepted.try_get()?;
                let month_changed = current.month != previous_month;
                previous_month = current.month;
                s.cursor.borrow_mut().take();
                s.selection_cursor.set(None);
                let focus = s.focused.try_with_untracked(|v| *v)?;
                let desired = if s.allowed(current, focus)
                    && current.grid.index_of(focus).is_some()
                    && (!month_changed || focus.year_month() == current.month)
                {
                    Some(focus)
                } else {
                    current
                        .value
                        .filter(|d| {
                            s.allowed(current, *d)
                                && current.grid.index_of(*d).is_some()
                                && (!month_changed || d.year_month() == current.month)
                        })
                        .or_else(|| s.first_allowed(current, current.month, focus.day()))
                };
                if let Some(date) = desired
                    && date != focus
                {
                    s.focused.try_revise(|v| *v = date)?;
                }
                if let Some(date) = s.intent.get() {
                    if let Some(index) = current.grid.index_of(date)
                        && s.allowed(current, date)
                    {
                        if let Some(node) = s.references[index].get() {
                            s.intent.set(None);
                            composite::focus_node(&scope, Some(node))?;
                        }
                    } else {
                        s.intent.set(None);
                    }
                }
                Ok(())
            })();
            s.report(result);
        });
        scope.retain(state)?;
        Ok(())
    }
}
struct DayButton {
    state: Rc<Controller>,
    index: usize,
    size: f64,
    themed: bool,
    theme: crate::Theme,
    observers: Observers,
}
impl Widget for DayButton {
    fn build(&mut self, scope: &Scope) -> Result {
        let index = self.index;
        let s = self.state.clone();
        let accepted = s.accepted;
        let focus = s.focused;
        let physical = s.physical;
        let reference = s.references[index];
        let size = self.size;
        let themed = self.themed;
        let theme = self.theme.clone();
        let observers = self.observers.clone();
        let enabled_state = s.clone();
        let style_state = s.clone();
        let activate = s.clone();
        let focus_state = s.clone();
        let button = glory::ui::button()
            .class("sapid-calendar-day")
            .node_ref(reference)
            .label(scope.bond(move || {
                accepted.with(|s| {
                    s.grid.cells()[index]
                        .map(|d| d.day().to_string())
                        .unwrap_or_default()
                })
            }))
            .name(scope.bond(move || {
                accepted.with(|s| {
                    s.grid.cells()[index]
                        .map(|d| s.provider.locale_value().date_label(d))
                        .unwrap_or_default()
                })
            }))
            .enabled(scope.bond(move || {
                let s = accepted.get();
                s.grid.cells()[index].is_some_and(|d| enabled_state.allowed(s, d))
            }))
            .focus(scope.bond(move || {
                let date = accepted.with(|s| s.grid.cells()[index]);
                if date == Some(focus.get()) {
                    FocusMode::TabStop
                } else {
                    FocusMode::Programmatic
                }
            }))
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    current: accepted.with(|s| {
                        s.grid.cells()[index]
                            .filter(|d| Some(*d) == s.provider.today_date())
                            .map(|_| Current::Date)
                    }),
                    ..Default::default()
                }))
            }))
            .style(scope.bond(move || {
                let s = accepted.get();
                let date = s.grid.cells()[index];
                let chosen = date.is_some_and(|d| s.selection.contains(d));
                let enabled = date.is_some_and(|d| style_state.allowed(s, d));
                let focused = reference.get().is_some_and(|id| physical.get() == Some(id));
                let mut style = Style::new()
                    .width(Length::Px(size))
                    .height(Length::Px(size))
                    .padding(Edges::all(0.))
                    .min_width(Length::Px(0.));
                if themed {
                    style.font_size = Some(if size < 30. { 11. } else { 14. });
                    style.background = Some(if chosen { theme.accent } else { theme.surface });
                    style.color = Some(if !enabled {
                        theme.disabled_text
                    } else if chosen {
                        theme.on_accent
                    } else if date.is_some_and(|d| d.year_month() != s.month) {
                        theme.muted
                    } else {
                        theme.foreground
                    });
                    style = style
                        .radius(theme.radius)
                        .border_width(Edges::all(1.))
                        .border_style(BorderStyle::Solid)
                        .border_color(if focused { theme.accent } else { theme.border });
                }
                style
            }))
            .on(EventKind::Activate, move |event| {
                if activate.ensure().is_ok() {
                    activate.report(activate.select(index));
                    observers.emit(activate.owner, event);
                }
            })
            .on(EventKind::FocusChanged, move |event| {
                if focus_state.ensure().is_ok()
                    && let Event::FocusChanged(focused) = event
                {
                    let result = (|| -> Result {
                        if focused {
                            focus_state.cursor.borrow_mut().take();
                            if let Some(date) = focus_state.date(index)? {
                                focus_state.focused.try_revise(|v| *v = date)?;
                            }
                            focus_state.physical.try_revise(|v| *v = reference.get())?;
                        } else if focus_state.physical.try_get()? == reference.get() {
                            focus_state.physical.try_revise(|v| *v = None)?;
                        }
                        Ok(())
                    })();
                    focus_state.report(result);
                }
            });
        // The date is always Some while this scoped branch is mounted. Its
        // reactive source can disappear only through this branch's retirement.
        button.show_in(scope)
    }
}
