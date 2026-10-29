use super::*;
use crate::{CivilDate, CivilYear, MonthGrid, Weekday};
use glory::render::{Direction, FocusMode, GridCommand, TableMetadata};
use glory::style::{Align, Limit, Overflow};
pub(super) fn window(
    view: ScheduleView,
    base: ScheduleWindow,
    delta: i32,
) -> std::result::Result<ScheduleWindow, crate::ScheduleError> {
    let date = base.start().date();
    if view == ScheduleView::Year {
        ScheduleWindow::year(
            CivilYear::new(date.year())
                .map_err(|_| crate::ScheduleError::InvalidInterval)?
                .checked_add(delta)
                .map_err(|_| crate::ScheduleError::InvalidInterval)?,
        )
    } else {
        ScheduleWindow::month(
            date.year_month()
                .checked_add(delta)
                .map_err(|_| crate::ScheduleError::InvalidInterval)?,
        )
    }
}
pub(super) fn validate(data: &ScheduleData, view: ScheduleView) -> Result {
    if window(view, data.window(), 0).map_err(|_| invalid())? != data.window() {
        return Err(invalid());
    }
    Ok(())
}
#[derive(Clone)]
struct PeriodCell {
    date: Option<CivilDate>,
    span: Option<ScheduleInterval>,
    indices: Vec<usize>,
}
struct Prepared {
    cells: Vec<PeriodCell>,
    events: Vec<AgendaRow>,
}
fn prepare(data: &ScheduleData, view: ScheduleView, week_start: Weekday) -> Prepared {
    let first = data.window().start().date();
    let dates: Vec<Option<CivilDate>> = if view == ScheduleView::Year {
        (1..=12)
            .map(|m| CivilDate::new(first.year(), m, 1).ok())
            .collect()
    } else {
        MonthGrid::new(first.year_month(), week_start)
            .cells()
            .to_vec()
    };
    let mut cells: Vec<_> = dates
        .into_iter()
        .map(|date| {
            let span = date
                .filter(|d| view == ScheduleView::Year || d.year_month() == first.year_month())
                .and_then(|d| {
                    if view == ScheduleView::Year {
                        ScheduleWindow::month(d.year_month())
                            .ok()
                            .map(|v| v.interval())
                    } else {
                        ScheduleInterval::all_day(d, d.checked_add_days(1).ok()?).ok()
                    }
                });
            PeriodCell {
                date,
                span,
                indices: vec![],
            }
        })
        .collect();
    let mut events = vec![];
    for occurrence in data.expand(MAX_SCHEDULE_OCCURRENCES).unwrap() {
        let event = data.event(occurrence).unwrap();
        let index = events.len();
        events.push(AgendaRow {
            key: event.key().clone(),
            candidate: occurrence.candidate_index(),
            interval: occurrence.interval(),
            title: event.title().into(),
            resource: event
                .resource_key()
                .map(|v| v.as_str().to_owned())
                .unwrap_or_default(),
            all_day: event.is_all_day(),
            selectable: event.edit_permissions().select,
        });
        for cell in &mut cells {
            if cell
                .span
                .is_some_and(|span| span.overlaps(occurrence.interval()))
            {
                cell.indices.push(index);
            }
        }
    }
    Prepared { cells, events }
}
struct State {
    root: Rc<Controller>,
    weak: WeakScope,
    live: Cell<bool>,
    prepared: glory::Bond<Prepared>,
    refs: Vec<Cage<Option<NodeId>>>,
    focused: Cage<usize>,
    physical: Cage<Option<NodeId>>,
    expanded: Cage<Option<usize>>,
    cursor: Cell<Option<(NodeId, usize)>>,
    intent: Cell<Option<usize>>,
    dirty: Dirty,
    columns: usize,
}
impl State {
    fn ensure(&self) -> Result {
        if self.live.get() {
            self.root.ensure()
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn report(&self, result: Result) {
        self.root.report(result)
    }
    fn navigate(&self, from: NodeId, command: GridCommand) -> Result {
        self.ensure()?;
        if !self.root.flags.try_with(|f| f.enabled)? {
            return Ok(());
        }
        let Some(at) = self.refs.iter().position(|r| r.get() == Some(from)) else {
            return Ok(());
        };
        let at = self
            .cursor
            .get()
            .filter(|(origin, _)| *origin == from)
            .map_or(at, |(_, i)| i);
        let cols = self.columns;
        let n = self.refs.len();
        let row = at / cols;
        let col = at % cols;
        let next = match command {
            GridCommand::Left => {
                if col > 0 {
                    at - 1
                } else {
                    at
                }
            }
            GridCommand::Right => {
                if col + 1 < cols {
                    at + 1
                } else {
                    at
                }
            }
            GridCommand::Up => {
                if row > 0 {
                    at - cols
                } else {
                    at
                }
            }
            GridCommand::Down => {
                if row + 1 < n / cols {
                    at + cols
                } else {
                    at
                }
            }
            GridCommand::FirstInRow => row * cols,
            GridCommand::LastInRow => ((row + 1) * cols - 1).min(n - 1),
            GridCommand::PreviousPage => at.saturating_sub(cols * 2),
            GridCommand::NextPage => (at + cols * 2).min(n - 1),
            GridCommand::PreviousLargePage => 0,
            GridCommand::NextLargePage => n - 1,
        };
        let next = self.prepared.try_with(|p| {
            if matches!(
                command,
                GridCommand::Left | GridCommand::Right | GridCommand::Up | GridCommand::Down
            ) && p.cells[next].span.is_none()
            {
                return at;
            }
            let first = p.cells.iter().position(|c| c.span.is_some()).unwrap();
            let last = p.cells.iter().rposition(|c| c.span.is_some()).unwrap();
            next.clamp(first, last)
        })?;
        self.cursor.set(Some((from, next)));
        self.intent.set(Some(next));
        self.focused.try_revise(|v| *v = next)?;
        self.expanded.try_revise(|v| *v = None)?;
        self.dirty.touch()
    }
    fn activate(&self, index: usize) -> Result {
        self.ensure()?;
        if !self.root.flags.try_with(|f| f.enabled && !f.readonly)? {
            return Ok(());
        }
        let index = self
            .cursor
            .get()
            .filter(|(id, _)| self.refs[index].get() == Some(*id))
            .map_or(index, |(_, at)| at);
        let span = self.prepared.try_with(|p| p.cells[index].span)?;
        if let Some(interval) = span {
            let v = ScheduleSelection::Slot {
                interval,
                resource: None,
            };
            self.root.selection.accept(&Some(v.clone()))?;
            self.root.selected.emit(self.root.owner, v);
        }
        Ok(())
    }
    fn focus(&self, index: usize, active: bool) -> Result {
        self.ensure()?;
        if active {
            self.cursor.set(None);
            self.focused.try_revise(|v| *v = index)?;
            self.physical.try_revise(|v| *v = self.refs[index].get())?;
        } else if self.refs[index]
            .get()
            .is_some_and(|id| self.physical.get() == Some(id))
        {
            self.physical.try_revise(|v| *v = None)?;
        }
        Ok(())
    }
    fn rows(&self, index: usize, all: bool) -> Vec<AgendaRow> {
        self.prepared.with(|p| {
            let cell = &p.cells[index];
            cell.indices
                .iter()
                .take(if all { MAX_SCHEDULE_OCCURRENCES } else { 3 })
                .map(|i| p.events[*i].clone())
                .collect()
        })
    }
}
pub(super) struct Config {
    pub key: String,
    pub view: ScheduleView,
    pub locale: DateLocale,
    pub week_start: Weekday,
    pub direction: Direction,
    pub theme: crate::Theme,
    pub unstyled: bool,
    pub size: crate::Size,
    pub grid_ref: Option<Cage<Option<NodeId>>>,
}
pub(super) fn build(scope: &Scope, root: Rc<Controller>, config: Config) -> Result {
    let Config {
        key,
        view,
        locale,
        week_start,
        direction,
        theme,
        unstyled,
        size,
        grid_ref,
    } = config;
    let data = root.accepted;
    let prepared = scope.bond(move || data.with(|d| prepare(d, view, week_start)));
    let columns = if view == ScheduleView::Year { 3 } else { 7 };
    let count = if view == ScheduleView::Year { 12 } else { 42 };
    let header_rows = usize::from(view != ScheduleView::Year);
    let initial = prepared.with(|p| p.cells.iter().position(|c| c.span.is_some()).unwrap());
    let state = Rc::new(State {
        root: root.clone(),
        weak: scope.downgrade(),
        live: Cell::new(true),
        prepared,
        refs: (0..count).map(|_| scope.cage(None)).collect(),
        focused: scope.cage(initial),
        physical: scope.cage(None),
        expanded: scope.cage(None),
        cursor: Cell::new(None),
        intent: Cell::new(None),
        dirty: Dirty::new(scope),
        columns,
    });
    let cleanup = Rc::downgrade(&state);
    scope.on_cleanup(move || {
        if let Some(s) = cleanup.upgrade() {
            s.live.set(false);
            s.cursor.set(None);
            s.intent.set(None);
        }
    })?;
    let grid = grid_ref.unwrap_or_else(|| scope.cage(None));
    let id = composite::id("schedule", &key, "calendar-grid", "root")?;
    let flags = root.flags;
    let s = state.clone();
    glory::ui::view()
        .role(Role::Grid)
        .class("sapid-schedule-calendar-grid")
        .name(if view == ScheduleView::Year {
            "Choose schedule month"
        } else {
            "Choose schedule date"
        })
        .node_ref(grid)
        .semantics(scope.bond(move || {
            Some(Box::new(Semantics {
                id: Some(id.clone()),
                grid_navigation: Some(direction),
                readonly: flags.with(|f| f.readonly),
                table: Some(TableMetadata::Table {
                    rows: Some((count / columns + header_rows) as u32),
                    columns: Some(columns as u16),
                }),
                ..Default::default()
            }))
        }))
        .style(composite::column(0.))
        .on(EventKind::GridNavigate, move |e| {
            if let Event::GridNavigate { from, command } = e {
                s.report(s.navigate(from, command));
            }
            s.root.grid_events.emit(s.root.owner, e);
        })
        .show_in(scope)?;
    let grid_scope = child(scope, grid)?;
    if header_rows != 0 {
        let header = scope.cage(None);
        glory::ui::view()
            .role(Role::Row)
            .class("sapid-schedule-weekday-row")
            .semantics(Some(Box::new(Semantics {
                table: Some(TableMetadata::Row { index: 0 }),
                ..Default::default()
            })))
            .node_ref(header)
            .style(Style::new().display(Display::Flex).height(Length::Px(28.)))
            .show_in(&grid_scope)?;
        let header_scope = child(scope, header)?;
        for physical in 0..7 {
            let col = if direction == Direction::Rtl {
                6 - physical
            } else {
                physical
            };
            let weekday = Weekday::ALL[(week_start.index() as usize + col) % 7];
            let cell = scope.cage(None);
            glory::ui::view()
                .role(Role::ColumnHeader)
                .name(locale.weekday_long(weekday))
                .node_ref(cell)
                .semantics(Some(Box::new(Semantics {
                    table: Some(TableMetadata::Cell {
                        row: 0,
                        column: col as u16,
                        row_span: 1,
                        column_span: 1,
                        headers: vec![],
                    }),
                    ..Default::default()
                })))
                .style(
                    Style::new()
                        .width(Length::Percent(100. / 7.))
                        .height(Length::Px(28.)),
                )
                .show_in(&header_scope)?;
            let cell_scope = child(scope, cell)?;
            glory::ui::text()
                .content(locale.weekday_short(weekday))
                .show_in(&cell_scope)?;
            scope.retain(cell_scope)?;
        }
        scope.retain(header_scope)?;
    }
    for row in 0..count / columns {
        let r = scope.cage(None);
        glory::ui::view()
            .role(Role::Row)
            .semantics(Some(Box::new(Semantics {
                table: Some(TableMetadata::Row {
                    index: (row + header_rows) as u32,
                }),
                ..Default::default()
            })))
            .node_ref(r)
            .style(Style {
                align: Some(Align::Stretch),
                ..Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
            })
            .show_in(&grid_scope)?;
        let row_scope = child(scope, r)?;
        for physical in 0..columns {
            let col = if direction == Direction::Rtl {
                columns - 1 - physical
            } else {
                physical
            };
            let index = row * columns + col;
            CalendarCell {
                state: state.clone(),
                index,
                view,
                locale,
                theme: theme.clone(),
                unstyled,
            }
            .show_in(&row_scope)?;
        }
        scope.retain(row_scope)?;
    }
    if view == ScheduleView::MobileMonth {
        let s = state.clone();
        let day = scope.bond(move || {
            let index = s.focused.get();
            s.prepared.with(|p| {
                let c = &p.cells[index];
                AgendaDay {
                    date: c.date.unwrap(),
                    label: locale.date_label(c.date.unwrap()),
                    rows: s.rows(index, true),
                }
            })
        });
        let root = state.root.clone();
        glory::ui::each(scope.bond(move || vec![day.get()]))
            .key(|d| d.date)
            .children(move |entry| {
                let day = entry.value;
                AgendaDayWidget {
                    day,
                    state: root.clone(),
                    theme: theme.clone(),
                    size,
                    unstyled,
                }
            })
            .show_in(scope)?;
    }
    let weak = Rc::downgrade(&state);
    scope.effect(move || {
        let Some(s) = weak.upgrade() else { return };
        let result = (|| -> Result {
            if s.dirty.take() {
                s.cursor.set(None);
            }
            if let Some(index) = s.intent.take()
                && let Some(scope) = s.weak.upgrade()
                && s.root.flags.try_with(|f| f.enabled)?
            {
                let target = s.refs[index].get();
                if let Some(target) = target
                    && scope.capabilities()?.scroll
                {
                    scope.action(glory::render::Action::ScrollIntoView(target))?;
                }
                composite::focus_node(&scope, target)?;
            }
            Ok(())
        })();
        s.report(result);
    });
    // Current data changes clear only the current disclosure and bound the roving index.
    let weak = Rc::downgrade(&state);
    scope.effect(move || {
        let first = prepared.with(|p| p.cells.iter().position(|c| c.span.is_some()).unwrap());
        if let Some(s) = weak.upgrade() {
            let result = (|| -> Result {
                s.expanded.try_revise(|v| *v = None)?;
                let at = s.focused.get_untracked();
                if !prepared.with(|p| p.cells[at].span.is_some()) {
                    s.focused.try_revise(|v| *v = first)?;
                }
                Ok(())
            })();
            s.report(result);
        }
    });
    scope.retain(grid_scope)?;
    scope.retain(state)
}
struct CalendarCell {
    state: Rc<State>,
    index: usize,
    view: ScheduleView,
    locale: DateLocale,
    theme: crate::Theme,
    unstyled: bool,
}
impl Widget for CalendarCell {
    fn build(&mut self, scope: &Scope) -> Result {
        let index = self.index;
        let p = self.state.prepared;
        let view = self.view;
        let locale = self.locale;
        let theme = self.theme.clone();
        let unstyled = self.unstyled;
        let state = self.state.clone();
        let flags = state.root.flags;
        let selection = state.root.selection.source.clone();
        let focused = state.focused;
        let physical = state.physical;
        let reference = state.refs[index];
        let cell = scope.cage(None);
        let mut cell_style = Style {
            min_width: Some(Length::Px(0.)),
            ..Style::new()
                .width(Length::Percent(if view == ScheduleView::Year {
                    100. / 3.
                } else {
                    100. / 7.
                }))
                .min_height(Length::Px(if view == ScheduleView::MobileMonth {
                    48.
                } else {
                    140.
                }))
                .overflow_x(Overflow::Hidden)
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
        };
        if !unstyled {
            cell_style = cell_style
                .border_width(Edges::all(1.))
                .border_style(BorderStyle::Solid)
                .border_color(theme.border);
        }
        glory::ui::view().role(Role::GridCell).class("sapid-schedule-calendar-cell").node_ref(cell)
            .semantics(scope.bond(move||Some(Box::new(Semantics{selected:Some(selection.with(|s|matches!(s,Some(ScheduleSelection::Slot{interval,resource:None})if p.with(|p|p.cells[index].span==Some(*interval))))),table:Some(TableMetadata::Cell{row:((index/if view==ScheduleView::Year{3}else{7})+usize::from(view!=ScheduleView::Year))as u32,column:(index%if view==ScheduleView::Year{3}else{7})as u16,row_span:1,column_span:1,headers:vec![]}),..Default::default()}))))
            .style(cell_style).show_in(scope)?;
        let content = child(scope, cell)?;
        let labels = scope.bond(move || {
            p.with(|p| {
                p.cells[index]
                    .date
                    .map(|d| {
                        if view == ScheduleView::Year {
                            format!("{} {}", locale.month_name(d.year_month()), d.year())
                        } else {
                            d.day().to_string()
                        }
                    })
                    .unwrap_or_default()
            })
        });
        let names = scope.bond(move || {
            p.with(|p| {
                p.cells[index]
                    .date
                    .map(|d| {
                        if view == ScheduleView::Year {
                            format!(
                                "{} {}, {} events",
                                locale.month_name(d.year_month()),
                                d.year(),
                                p.cells[index].indices.len()
                            )
                        } else {
                            format!(
                                "{}, {} events",
                                locale.date_label(d),
                                p.cells[index].indices.len()
                            )
                        }
                    })
                    .unwrap_or_else(|| "Outside civil date domain".into())
            })
        });
        let chosen = state.root.selection.source.clone();
        let t = theme.clone();
        let s = state.clone();
        let f = state.clone();
        glory::ui::button().class("sapid-schedule-period-slot").label(labels).name(names).node_ref(reference)
            .enabled(scope.bond(move||flags.with(|f|f.enabled)&&p.with(|p|p.cells[index].span.is_some())))
            .focus(scope.bond(move||if focused.get()==index{FocusMode::TabStop}else{FocusMode::Programmatic}))
            .style(scope.bond(move||{let mut style=Style::new().width(Length::Percent(100.)).height(Length::Px(if view==ScheduleView::MobileMonth{48.}else{36.})).padding(Edges::all(2.));if !unstyled{let active=chosen.with(|v|matches!(v,Some(ScheduleSelection::Slot{interval,resource:None})if p.with(|p|p.cells[index].span==Some(*interval))));style.background=Some(if active{t.accent_soft}else{t.surface});style.color=Some(if !p.with(|p|p.cells[index].span.is_some()) || !flags.with(|f|f.enabled) {t.disabled_text}else if active{t.accent_text}else{t.foreground});style=style.border_width(Edges::all(1.)).border_style(BorderStyle::Solid).border_color(if reference.get().is_some_and(|id|physical.get()==Some(id)){t.accent}else{t.border});}style}))
            .on(EventKind::Activate,move|e|{s.report(s.activate(index));s.root.activated.emit(s.root.owner,e);})
            .on(EventKind::FocusChanged,move|e|{if let Event::FocusChanged(active)=e{f.report(f.focus(index,active));}}).show_in(&content)?;
        if view != ScheduleView::Month {
            glory::ui::text()
                .content(scope.bond(move || {
                    p.with(|p| {
                        let count = p.cells[index].indices.len();
                        if count == 0 {
                            String::new()
                        } else if view == ScheduleView::MobileMonth {
                            count.to_string()
                        } else {
                            format!("{count} events")
                        }
                    })
                }))
                .class("sapid-schedule-period-count")
                .style(Style {
                    font_size: Some(11.),
                    ..Style::new()
                })
                .show_in(&content)?;
        }
        if view != ScheduleView::MobileMonth {
            let group = scope.cage(None);
            glory::ui::view()
                .role(Role::Group)
                .class("sapid-schedule-period-events")
                .name("Events in this period")
                .node_ref(group)
                .style(composite::column(2.))
                .show_in(&content)?;
            let events_scope = child(scope, group)?;
            let s = state.clone();
            let expanded = state.expanded;
            let rows = scope.bond(move || {
                let all = expanded.get() == Some(index);
                s.rows(index, all)
            });
            let root = state.root.clone();
            let event_theme = theme.clone();
            let list = scope.cage(None);
            glory::ui::view()
                .document_part(DocumentPart::UnorderedList)
                .node_ref(list)
                .class("sapid-schedule-period-list")
                .style(Style::new().margin(Edges::all(0.)).padding(Edges {
                    left: 16.,
                    ..Edges::all(0.)
                }))
                .show_in(&events_scope)?;
            let list_scope = child(scope, list)?;
            glory::ui::each(rows)
                .key(|r| (r.key.clone(), r.interval.start()))
                .children(move |entry| {
                    let row = entry.value;
                    PeriodEventWidget {
                        row,
                        state: root.clone(),
                        theme: event_theme.clone(),
                        unstyled,
                    }
                })
                .show_in(&list_scope)?;
            let s = state.clone();
            let more = state.expanded;
            let expanded = state.expanded;
            let more_semantics = scope.bond(move || {
                Some(Box::new(Semantics {
                    expanded: Some(more.get() == Some(index)),
                    ..Default::default()
                }))
            });
            let has_more = scope.bond(move || p.with(|p| p.cells[index].indices.len() > 3));
            let more_name = scope.bond(move || {
                p.with(|p| format!("Show events for {}", p.cells[index].date.unwrap()))
            });
            let mut more_style = Style::new()
                .min_height(Length::Px(24.))
                .max_width(Limit::Percent(100.))
                .padding(Edges::all(2.));
            if !unstyled {
                more_style = more_style
                    .border_width(Edges::all(1.))
                    .border_style(BorderStyle::Solid)
                    .border_color(theme.border);
                more_style.color = Some(theme.foreground);
                more_style.background = Some(theme.input);
            }
            glory::ui::when(has_more, move || {
                let state = s.clone();
                glory::ui::button()
                    .class("sapid-schedule-more")
                    .label(expanded.map(move |v| {
                        if *v == Some(index) {
                            "Show fewer events".to_owned()
                        } else {
                            "Show all events".to_owned()
                        }
                    }))
                    .name(more_name)
                    .enabled(flags.map(|f| f.enabled))
                    .semantics(more_semantics)
                    .style(more_style.clone())
                    .on(EventKind::Activate, move |e| {
                        let result = (|| -> Result {
                            state.ensure()?;
                            if state.root.flags.try_with(|f| f.enabled)? {
                                state.expanded.try_revise(|v| {
                                    *v = if *v == Some(index) { None } else { Some(index) }
                                })?;
                            }
                            Ok(())
                        })();
                        state.report(result);
                        state.root.activated.emit(state.root.owner, e);
                    })
            })
            .show_in(&content)?;
            scope.retain(list_scope)?;
            scope.retain(events_scope)?;
        }
        scope.retain(content)
    }
}
struct PeriodEventWidget {
    row: Cage<AgendaRow>,
    state: Rc<Controller>,
    theme: crate::Theme,
    unstyled: bool,
}
impl Widget for PeriodEventWidget {
    fn build(&mut self, scope: &Scope) -> Result {
        let row = self.row;
        let flags = self.state.flags;
        let selected = self.state.selection.source.clone();
        let theme = self.theme.clone();
        let unstyled = self.unstyled;
        let wrapper = scope.cage(None);
        glory::ui::view()
            .document_part(DocumentPart::ListItem)
            .class("sapid-agenda-event sapid-schedule-period-event")
            .node_ref(wrapper)
            .style(Style::new().height(Length::Px(28.)).margin(Edges::all(0.)))
            .show_in(scope)?;
        let content = child(scope, wrapper)?;
        let state = self.state.clone();
        glory::ui::button().class("sapid-schedule-period-event-button").label(row.map(|r|r.title.clone()))
            .name(row.map(|r|format!("{} · {} — {}",r.title,crate::format_local_date_time(r.interval.start()),crate::format_local_date_time(r.interval.end()))))
            .enabled(scope.bond(move||flags.with(|f|f.enabled&&!f.readonly)&&row.with(|r|r.selectable)))
            .style(scope.bond(move||{let mut style=Style::new().width(Length::Percent(100.)).height(Length::Px(28.)).padding(Edges::all(1.)).overflow_x(Overflow::Hidden).overflow_y(Overflow::Hidden);if !unstyled{let chosen=selected.with(|v|matches!(v,Some(ScheduleSelection::Event{key,candidate,interval})if row.with(|r|*key==r.key&&*candidate==r.candidate&&*interval==r.interval)));style.background=Some(if chosen{theme.accent}else{theme.accent_soft});style.color=Some(if chosen{theme.on_accent}else{theme.accent_text});style.font_size=Some(11.);style=style.border_width(Edges::all(1.)).border_style(BorderStyle::Solid).border_color(theme.accent);}style}))
            .on(EventKind::Activate,move|e|{let r=row.with(|r|r.clone());state.report(state.event(&r));state.activated.emit(state.owner,e);}).show_in(&content)?;
        scope.retain(content)
    }
}
