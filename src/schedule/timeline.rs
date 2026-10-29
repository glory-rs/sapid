use super::*;
use crate::{ScheduleDayPlacement, ScheduleTimeWindow};
use glory::render::{Action, Direction, EventKind, FocusMode, GridCommand, TableMetadata};
use glory::style::{Overflow, Position};

pub(super) fn validate(data: &ScheduleData, days: usize, time: ScheduleTimeWindow) -> Result {
    let window = data.window();
    if window.start().time() != crate::schedule_model::midnight()
        || window.end().time() != crate::schedule_model::midnight()
        || window.start().date().days_until(window.end().date()) != days as i32
    {
        return Err(invalid());
    }
    for day in 0..days {
        let date = window
            .start()
            .date()
            .checked_add_days(day as i32)
            .map_err(|_| invalid())?;
        if data
            .layout_day(date, time, None)
            .map_err(|_| invalid())?
            .iter()
            .any(|p| p.lanes() > 16)
        {
            return Err(invalid());
        }
    }
    Ok(())
}
#[derive(Clone)]
struct TimelineEvent {
    row: AgendaRow,
    day: usize,
    position: ScheduleDayPlacement,
}
#[derive(Clone)]
struct Prepared {
    events: Vec<TimelineEvent>,
    column_width: f64,
}
fn prepare(
    data: &ScheduleData,
    days: usize,
    time: ScheduleTimeWindow,
    base_width: f64,
) -> Prepared {
    let occurrences = data.expand(MAX_SCHEDULE_OCCURRENCES).unwrap();
    let mut events = vec![];
    let mut lanes = 1;
    for day in 0..days {
        let date = data
            .window()
            .start()
            .date()
            .checked_add_days(day as i32)
            .unwrap();
        for position in data.layout_day(date, time, None).unwrap() {
            let occurrence = occurrences[position.occurrence_index()];
            let event = data.event(occurrence).unwrap();
            lanes = lanes.max(position.lanes());
            events.push(TimelineEvent {
                row: AgendaRow {
                    key: event.key().clone(),
                    candidate: occurrence.candidate_index(),
                    interval: occurrence.interval(),
                    title: event.title().into(),
                    resource: event
                        .resource_key()
                        .map(|k| k.as_str().to_owned())
                        .unwrap_or_default(),
                    all_day: false,
                    selectable: event.edit_permissions().select,
                },
                day,
                position,
            });
        }
    }
    Prepared {
        events,
        column_width: base_width.max(lanes as f64 * 28.),
    }
}
struct TimelineState {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    live: Cell<bool>,
    root: Rc<Controller>,
    days: usize,
    time: ScheduleTimeWindow,
    refs: Vec<Cage<Option<NodeId>>>,
    focused: Cage<usize>,
    physical: Cage<Option<NodeId>>,
    cursor: Cell<Option<(NodeId, usize)>>,
    intent: Cell<Option<usize>>,
    dirty: Dirty,
}
impl TimelineState {
    fn interval(&self, at: usize) -> Result<ScheduleInterval> {
        if at >= self.refs.len() {
            return Err(invalid());
        }
        let date = self
            .root
            .accepted
            .try_with(|data| data.window().start().date())?
            .checked_add_days((at % self.days) as i32)
            .map_err(|_| invalid())?;
        self.time.at(date, at / self.days).map_err(|_| invalid())
    }
    fn ensure(&self) -> Result {
        if self.live.get() && self.owner.is_alive() {
            self.root.ensure()
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn navigate(&self, origin: NodeId, command: GridCommand) -> Result {
        self.ensure()?;
        if !self.root.flags.try_with(|f| f.enabled)? {
            return Ok(());
        }
        let Some(at) = self.refs.iter().position(|r| r.get() == Some(origin)) else {
            return Ok(());
        };
        let at = self
            .cursor
            .get()
            .filter(|(id, _)| *id == origin)
            .map_or(at, |(_, at)| at);
        let row = at / self.days;
        let col = at % self.days;
        let n = self.refs.len();
        let step = self.days;
        let next = match command {
            GridCommand::Left => {
                if col > 0 {
                    at - 1
                } else {
                    at
                }
            }
            GridCommand::Right => {
                if col + 1 < step {
                    at + 1
                } else {
                    at
                }
            }
            GridCommand::Up => {
                if row > 0 {
                    at - step
                } else {
                    at
                }
            }
            GridCommand::Down => {
                if row + 1 < self.time.len() {
                    at + step
                } else {
                    at
                }
            }
            GridCommand::FirstInRow => row * step,
            GridCommand::LastInRow => (row + 1) * step - 1,
            GridCommand::PreviousPage => row.saturating_sub(4) * step + col,
            GridCommand::NextPage => (row + 4).min(self.time.len() - 1) * step + col,
            GridCommand::PreviousLargePage => 0,
            GridCommand::NextLargePage => n - 1,
        };
        self.cursor.set(Some((origin, next)));
        self.intent.set(Some(next));
        self.focused.try_revise(|v| *v = next)?;
        self.dirty.touch()
    }
    fn select(&self, index: usize) -> Result {
        self.ensure()?;
        if !self.root.flags.try_with(|f| f.enabled && !f.readonly)? {
            return Ok(());
        }
        let index = self
            .cursor
            .get()
            .filter(|(id, _)| self.refs[index].get() == Some(*id))
            .map_or(index, |(_, at)| at);
        let value = ScheduleSelection::Slot {
            interval: self.interval(index)?,
            resource: None,
        };
        self.root.selection.accept(&Some(value.clone()))?;
        self.root.selected.emit(self.root.owner, value);
        Ok(())
    }
    fn report(&self, result: Result) {
        self.root.report(result)
    }
}
pub(super) struct Config {
    pub key: String,
    pub days: usize,
    pub time: ScheduleTimeWindow,
    pub theme: crate::Theme,
    pub unstyled: bool,
    pub direction: Direction,
    pub locale: DateLocale,
    pub grid_ref: Option<Cage<Option<NodeId>>>,
    pub viewport_ref: Option<Cage<Option<NodeId>>>,
    pub column_width: f64,
    pub viewport_height: f64,
}
pub(super) fn build(scope: &Scope, root: Rc<Controller>, config: Config) -> Result {
    let Config {
        key,
        days,
        time,
        theme,
        unstyled,
        direction,
        locale,
        grid_ref,
        viewport_ref,
        column_width,
        viewport_height,
    } = config;
    let accepted = root.accepted;
    let prepared =
        scope.bond(move || accepted.with(|data| prepare(data, days, time, column_width)));
    let state = Rc::new(TimelineState {
        owner: scope.id(),
        weak: scope.downgrade(),
        live: Cell::new(true),
        root: root.clone(),
        days,
        time,
        refs: (0..days * time.len()).map(|_| scope.cage(None)).collect(),
        focused: scope.cage(0),
        physical: scope.cage(None),
        cursor: Cell::new(None),
        intent: Cell::new(None),
        dirty: Dirty::new(scope),
    });
    let cleanup = Rc::downgrade(&state);
    scope.on_cleanup(move || {
        if let Some(s) = cleanup.upgrade() {
            s.live.set(false);
            s.cursor.set(None);
            s.intent.set(None);
        }
    })?;
    let viewport = viewport_ref.unwrap_or_else(|| scope.cage(None));
    glory::ui::view()
        .role(Role::Group)
        .name("Schedule timeline viewport")
        .class("sapid-schedule-viewport")
        .style(
            Style::new()
                .width(Length::Percent(100.))
                .height(Length::Px(viewport_height))
                .overflow_x(Overflow::Auto)
                .overflow_y(Overflow::Auto),
        )
        .node_ref(viewport)
        .show_in(scope)?;
    let viewport_scope = child(scope, viewport)?;
    let canvas = scope.cage(None);
    let height = time.len() as f64 * 32. + 32.;
    glory::ui::view()
        .class("sapid-schedule-timeline")
        .style(scope.bond(move || {
            Style::new()
                .position(Position::Relative)
                .width(Length::Px(
                    64. + days as f64 * prepared.with(|p| p.column_width),
                ))
                .height(Length::Px(height))
        }))
        .node_ref(canvas)
        .show_in(&viewport_scope)?;
    let canvas_scope = child(scope, canvas)?;
    let grid = grid_ref.unwrap_or_else(|| scope.cage(None));
    let id = composite::id("schedule", &key, "grid", "root")?;
    let s = state.clone();
    let flags = root.flags;
    glory::ui::view()
        .role(Role::Grid)
        .name("Choose schedule time slot")
        .class("sapid-schedule-slot-grid")
        .node_ref(grid)
        .semantics(scope.bond(move || {
            Some(Box::new(Semantics {
                id: Some(id.clone()),
                grid_navigation: Some(direction),
                readonly: flags.with(|f| f.readonly),
                table: Some(TableMetadata::Table {
                    rows: Some((time.len() + 1) as u32),
                    columns: Some((days + 1) as u16),
                }),
                ..Default::default()
            }))
        }))
        .style(
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column),
        )
        .on(EventKind::GridNavigate, move |event| {
            if let Event::GridNavigate { from, command } = &event {
                s.report(s.navigate(*from, *command));
            }
            s.root.grid_events.emit(s.root.owner, event);
        })
        .show_in(&canvas_scope)?;
    let grid_scope = child(scope, grid)?;
    for row in 0..=time.len() {
        let row_ref = scope.cage(None);
        glory::ui::view()
            .role(Role::Row)
            .class(if row == 0 {
                "sapid-schedule-day-headings"
            } else {
                "sapid-schedule-time-row"
            })
            .semantics(Some(Box::new(Semantics {
                table: Some(TableMetadata::Row { index: row as u32 }),
                ..Default::default()
            })))
            .style(scope.bond(move || {
                Style::new()
                    .position(Position::Relative)
                    .width(Length::Px(
                        64. + days as f64 * prepared.with(|p| p.column_width),
                    ))
                    .height(Length::Px(32.))
            }))
            .node_ref(row_ref)
            .show_in(&grid_scope)?;
        let row_scope = child(scope, row_ref)?;
        for col in 0..=days {
            let cell = scope.cage(None);
            let active = root.selection.source.clone();
            let state_for_cell = state.clone();
            glory::ui::view().role(if row==0{Role::ColumnHeader}else{Role::GridCell})
                .class(if col==0{"sapid-schedule-time-label"}else{"sapid-schedule-slot-cell"})
                .semantics(scope.bond(move||Some(Box::new(Semantics{selected:if row==0||col==0{None}else{Some(active.with(|v|matches!(v,Some(ScheduleSelection::Slot{interval,resource:None})if state_for_cell.interval((row-1)*days+col-1).ok()==Some(*interval))))},table:Some(TableMetadata::Cell{row:row as u32,column:col as u16,row_span:1,column_span:1,headers:vec![]}),..Default::default()}))))
                .style(scope.bond(move||{
                    let width=prepared.with(|p|p.column_width);let physical=if col==0{0}else if direction==Direction::Ltr{col}else{days+1-col};
                    Style::new().position(Position::Absolute).left(Length::Px(if physical==0{0.}else{64.+(physical-1)as f64*width})).top(Length::Px(0.)).width(Length::Px(if col==0{64.}else{width})).height(Length::Px(32.))
                })).node_ref(cell).show_in(&row_scope)?;
            let cell_scope = child(scope, cell)?;
            if row == 0 {
                let label = scope.bond(move || {
                    if col == 0 {
                        "Time".to_owned()
                    } else {
                        accepted.with(|data| {
                            let date = data
                                .window()
                                .start()
                                .date()
                                .checked_add_days((col - 1) as i32)
                                .unwrap();
                            format!(
                                "{} {:02}-{:02}",
                                locale.weekday_short(date.weekday()),
                                date.month(),
                                date.day()
                            )
                        })
                    }
                });
                glory::ui::text()
                    .content(label)
                    .style(Style {
                        font_size: Some(11.),
                        ..Style::new()
                    })
                    .show_in(&cell_scope)?;
            } else if col == 0 {
                let minutes = time.start_minutes() + (row as u16 - 1) * time.step_minutes();
                glory::ui::text()
                    .content(format!("{:02}:{:02}", minutes / 60, minutes % 60))
                    .style(Style {
                        font_size: Some(12.),
                        ..Style::new()
                    })
                    .show_in(&cell_scope)?;
            } else {
                let at = (row - 1) * days + col - 1;
                let reference = state.refs[at];
                let focus = state.focused;
                let physical = state.physical;
                let s = state.clone();
                let fs = state.clone();
                let labels = state.clone();
                let names = state.clone();
                let styles = state.clone();
                let selected = root.selection.source.clone();
                let button_theme = theme.clone();
                glory::ui::button().class("sapid-schedule-slot").node_ref(reference)
                    .label(scope.bond(move||labels.interval(at).map(|span|format!("{:02}:{:02}",span.start().time().hour(),span.start().time().minute())).unwrap_or_default()))
                    .name(scope.bond(move||names.interval(at).map(|span|format!("{} {} — {}",locale.date_label(span.start().date()),span.start().time(),span.end().time())).unwrap_or_default()))
                    .enabled(flags.map(|f|f.enabled)).focus(scope.bond(move||if focus.get()==at{FocusMode::TabStop}else{FocusMode::Programmatic}))
                    .style(scope.bond(move||{
                        let mut style=Style::new().width(Length::Percent(100.)).height(Length::Px(32.)).padding(Edges::all(0.)).overflow_x(Overflow::Hidden).overflow_y(Overflow::Hidden);
                        if !unstyled{let chosen=selected.with(|v|matches!(v,Some(ScheduleSelection::Slot{interval,resource:None})if styles.interval(at).ok()==Some(*interval)));
                            style.background=Some(if chosen{button_theme.accent_soft}else{button_theme.surface});style.color=Some(if chosen{button_theme.accent_text}else{button_theme.foreground});style.font_size=Some(10.);style=style.border_width(Edges::all(1.)).border_style(BorderStyle::Solid).border_color(if reference.get().is_some_and(|id|physical.get()==Some(id)){button_theme.accent}else{button_theme.border});}
                        style
                    }))
                    .on(EventKind::Activate,move|event|{s.report(s.select(at));s.root.activated.emit(s.root.owner,event);})
                    .on(EventKind::FocusChanged,move|event|{if let Event::FocusChanged(active)=event{let result=(||->Result{fs.ensure()?;if active{fs.cursor.set(None);focus.try_revise(|v|*v=at)?;physical.try_revise(|v|*v=reference.get())?;}else if reference.get().is_some_and(|id|physical.get()==Some(id)){physical.try_revise(|v|*v=None)?;}Ok(())})();fs.report(result);}})
                    .show_in(&cell_scope)?;
            }
            scope.retain(cell_scope)?;
        }
        scope.retain(row_scope)?;
    }
    let state_for_events = root.clone();
    let theme_for_events = theme.clone();
    glory::ui::each(scope.bond(move || prepared.with(|p| p.events.clone())))
        .key(|event| (event.row.key.clone(), event.row.interval.start(), event.day))
        .children(move |entry| {
            let event = entry.value;
            TimelineEventWidget {
                event,
                prepared,
                state: state_for_events.clone(),
                theme: theme_for_events.clone(),
                unstyled,
                direction,
                days,
                height: height - 32.,
            }
        })
        .show_in(&canvas_scope)?;
    let weak = Rc::downgrade(&state);
    scope.effect(move || {
        let Some(s) = weak.upgrade() else { return };
        let result = (|| -> Result {
            if s.dirty.take() {
                s.cursor.set(None);
            }
            if let Some(at) = s.intent.take()
                && let Some(scope) = s.weak.upgrade()
                && s.root.flags.try_with(|f| f.enabled)?
            {
                let target = s.refs[at].get();
                if let Some(target) = target
                    && scope.capabilities()?.scroll
                {
                    scope.action(Action::ScrollIntoView(target))?;
                }
                composite::focus_node(&scope, target)?;
            }
            Ok(())
        })();
        s.report(result);
    });
    scope.retain(grid_scope)?;
    scope.retain(canvas_scope)?;
    scope.retain(viewport_scope)?;
    scope.retain(state)
}
struct TimelineEventWidget {
    event: Cage<TimelineEvent>,
    prepared: glory::Bond<Prepared>,
    state: Rc<Controller>,
    theme: crate::Theme,
    unstyled: bool,
    direction: Direction,
    days: usize,
    height: f64,
}
impl Widget for TimelineEventWidget {
    fn build(&mut self, scope: &Scope) -> Result {
        let event = self.event;
        let prepared = self.prepared;
        let flags = self.state.flags;
        let direction = self.direction;
        let days = self.days;
        let height = self.height;
        let theme = self.theme.clone();
        let unstyled = self.unstyled;
        let selected = self.state.selection.source.clone();
        let style=scope.bond(move||{
            let width=prepared.with(|p|p.column_width);event.with(|e|{
                let day=if direction==Direction::Ltr{e.day}else{days-1-e.day};let fraction=width/e.position.lanes()as f64;
                let mut style=Style::new().position(Position::Absolute).left(Length::Px(64.+day as f64*width+e.position.lane()as f64*fraction)).top(Length::Px(32.+e.position.start_fraction()*height))
                    .width(Length::Px((fraction-2.).max(2.))).height(Length::Px(((e.position.end_fraction()-e.position.start_fraction())*height).max(18.))).padding(Edges::all(1.)).overflow_x(Overflow::Hidden).overflow_y(Overflow::Hidden);
                if !unstyled{let chosen=selected.with(|v|matches!(v,Some(ScheduleSelection::Event{key,candidate,interval})if *key==e.row.key&&*candidate==e.row.candidate&&*interval==e.row.interval));style.background=Some(if chosen{theme.accent}else{theme.accent_soft});style.color=Some(if chosen{theme.on_accent}else{theme.accent_text});style=style.border_width(Edges::all(1.)).border_style(BorderStyle::Solid).border_color(theme.accent);style.font_size=Some(11.);}
                style
            })
        });
        let state = self.state.clone();
        let enabled = scope.bond(move || {
            flags.with(|f| f.enabled && !f.readonly) && event.with(|e| e.row.selectable)
        });
        glory::ui::button()
            .class("sapid-schedule-timeline-event")
            .label(event.map(|e| e.row.title.clone()))
            .name(event.map(|e| {
                format!(
                    "{} · {} — {}",
                    e.row.title,
                    crate::format_local_date_time(e.row.interval.start()),
                    crate::format_local_date_time(e.row.interval.end())
                )
            }))
            .style(style)
            .enabled(enabled)
            .on(EventKind::Activate, move |e| {
                let row = event.with(|e| e.row.clone());
                state.report(state.event(&row));
                state.activated.emit(state.owner, e);
            })
            .show_in(scope)
    }
}
