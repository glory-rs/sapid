use super::ResourceController as Controller;
use super::*;
use crate::{ScheduleDayPlacement, ScheduleTimeWindow};
use glory::render::{Action, Direction, EventKind, FocusMode, GridCommand, TableMetadata};
use glory::style::{Overflow, Position};

#[derive(Clone)]
struct TimelineEvent {
    row: AgendaRow,
    column: usize,
    date: crate::CivilDate,
    resource: ScheduleKey,
    position: ScheduleDayPlacement,
}
#[derive(Clone)]
struct Prepared {
    events: Vec<TimelineEvent>,
    column_width: f64,
}
fn prepare(
    model: &ResourceScheduleData,
    days: usize,
    time: ScheduleTimeWindow,
    base_width: f64,
    month: bool,
) -> Prepared {
    let layout = if month {
        model.layout_month().unwrap()
    } else {
        model.layout_timeline(time, days).unwrap()
    };
    let data = model.data();
    let occurrences = data.expand(MAX_SCHEDULE_OCCURRENCES).unwrap();
    let mut events = vec![];
    let mut lanes = 1;
    for item in layout.placements() {
        let position = item.position();
        let column = layout.columns()[item.column()];
        let occurrence = occurrences[position.occurrence_index()];
        let event = data.event(occurrence).unwrap();
        lanes = lanes.max(position.lanes());
        events.push(TimelineEvent {
            row: AgendaRow {
                key: event.key().clone(),
                candidate: occurrence.candidate_index(),
                interval: occurrence.interval(),
                title: event.title().into(),
                resource: event.resource_key().unwrap().as_str().to_owned(),
                all_day: event.is_all_day(),
                selectable: event.edit_permissions().select,
            },
            column: item.column(),
            date: column.date(),
            resource: model.resources().items()[column.resource_index()]
                .key()
                .clone(),
            position,
        });
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
    date_days: usize,
    time: ScheduleTimeWindow,
    axis: ResourceAxis,
    period: Option<usize>,
    refs: Vec<Cage<Option<NodeId>>>,
    focused: Cage<usize>,
    physical: Cage<Option<NodeId>>,
    cursor: Cell<Option<(NodeId, usize)>>,
    intent: Cell<Option<usize>>,
    dirty: Dirty,
}
impl TimelineState {
    fn steps(&self) -> usize {
        self.period.unwrap_or(self.time.len())
    }
    fn interval(&self, at: usize) -> Result<ScheduleInterval> {
        if at >= self.refs.len() {
            return Err(invalid());
        }
        let date = self
            .root
            .accepted
            .try_with(|data| data.data().window().start().date())?
            .checked_add_days(if self.period.is_some() {
                (at / self.days) as i32
            } else {
                (at % self.days % self.date_days) as i32
            })
            .map_err(|_| invalid())?;
        if self.period.is_some() {
            return crate::ScheduleInterval::all_day(
                date,
                date.checked_add_days(1).map_err(|_| invalid())?,
            )
            .map_err(|_| invalid());
        }
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
        if !self.root.enabled.try_get()? {
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
        let (row, col, rows, cols) = if self.axis == ResourceAxis::Columns {
            (at / self.days, at % self.days, self.steps(), self.days)
        } else {
            (at % self.days, at / self.days, self.days, self.steps())
        };
        let n = self.refs.len();
        let step = cols;
        let physical = row * cols + col;
        let next = match command {
            GridCommand::Left => {
                if col > 0 {
                    physical - 1
                } else {
                    physical
                }
            }
            GridCommand::Right => {
                if col + 1 < step {
                    physical + 1
                } else {
                    physical
                }
            }
            GridCommand::Up => {
                if row > 0 {
                    physical - step
                } else {
                    physical
                }
            }
            GridCommand::Down => {
                if row + 1 < rows {
                    physical + step
                } else {
                    physical
                }
            }
            GridCommand::FirstInRow => row * step,
            GridCommand::LastInRow => (row + 1) * step - 1,
            GridCommand::PreviousPage => row.saturating_sub(4) * step + col,
            GridCommand::NextPage => (row + 4).min(rows - 1) * step + col,
            GridCommand::PreviousLargePage => 0,
            GridCommand::NextLargePage => n - 1,
        };
        let next = if self.axis == ResourceAxis::Columns {
            next
        } else {
            (next % cols) * self.days + next / cols
        };
        self.cursor.set(Some((origin, next)));
        self.intent.set(Some(next));
        self.focused.try_revise(|v| *v = next)?;
        self.dirty.touch()
    }
    fn select(&self, index: usize) -> Result {
        self.ensure()?;
        if !self.root.ready()? {
            return Ok(());
        }
        let index = self
            .cursor
            .get()
            .filter(|(id, _)| self.refs[index].get() == Some(*id))
            .map_or(index, |(_, at)| at);
        let value = ScheduleSelection::Slot {
            interval: self.interval(index)?,
            resource: Some(self.root.accepted.try_with(|m| {
                m.visible_resources()[index % self.days / self.date_days]
                    .key()
                    .clone()
            })?),
        };
        self.root.select(value)?;
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
    pub axis: ResourceAxis,
    pub month: bool,
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
        axis,
        month,
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
    let date_days = if month { 1 } else { days };
    let period = month.then(|| {
        accepted.with(|m| m.data().window().start().date().year_month().days() as usize)
    });
    let steps = period.unwrap_or(time.len());
    let days = accepted.with(|m| m.visible_resources().len()) * date_days;
    let prepared = scope.bond(move || {
        accepted.with(|data| prepare(data, date_days, time, column_width, month))
    });
    let state = Rc::new(TimelineState {
        owner: scope.id(),
        weak: scope.downgrade(),
        live: Cell::new(true),
        root: root.clone(),
        days,
        date_days,
        time,
        axis,
        period,
        refs: (0..days * steps).map(|_| scope.cage(None)).collect(),
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
        .class("sapid-resources-viewport")
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
    let height = if axis == ResourceAxis::Columns {
        steps as f64 * 32. + 32.
    } else {
        days as f64 * column_width + 32.
    };
    let columns = if axis == ResourceAxis::Columns {
        days
    } else {
        steps
    };
    let rows = if axis == ResourceAxis::Columns {
        steps
    } else {
        days
    };
    glory::ui::view()
        .class("sapid-resources-timeline")
        .style(scope.bond(move || {
            Style::new()
                .position(Position::Relative)
                .width(Length::Px(if axis == ResourceAxis::Columns {
                    64. + days as f64 * prepared.with(|p| p.column_width)
                } else {
                    160. + steps as f64 * 64.
                }))
                .height(Length::Px(if axis == ResourceAxis::Columns {
                    height
                } else {
                    32. + days as f64 * prepared.with(|p| p.column_width)
                }))
        }))
        .node_ref(canvas)
        .show_in(&viewport_scope)?;
    let canvas_scope = child(scope, canvas)?;
    let drag = drag::Drag::new(
        scope,
        root.clone(),
        canvas,
        drag::Config {
            axis,
            direction,
            date_days,
            step: if month { 1440 } else { time.step_minutes() },
            column_width: scope.bond(move || prepared.with(|p| p.column_width)),
        },
    )?;
    let cancel = drag.clone();
    scope.on(
        canvas.get().ok_or(glory::ui::Error::Gone)?,
        EventKind::Resize,
        move |_| cancel.cancel(),
    )?;
    let cancel = drag.clone();
    scope.on(
        viewport.get().ok_or(glory::ui::Error::Gone)?,
        EventKind::Scroll,
        move |_| cancel.cancel(),
    )?;
    let grid = grid_ref.unwrap_or_else(|| scope.cage(None));
    let id = composite::id("resource-schedule", &key, "grid", "root")?;
    let s = state.clone();
    let flags = root.flags;
    glory::ui::view()
        .role(Role::Grid)
        .name("Choose schedule time slot")
        .class("sapid-resources-slot-grid")
        .node_ref(grid)
        .semantics(scope.bond(move || {
            Some(Box::new(Semantics {
                id: Some(id.clone()),
                grid_navigation: Some(direction),
                readonly: flags.with(|f| f.readonly),
                table: Some(TableMetadata::Table {
                    rows: Some((rows + 1) as u32),
                    columns: Some((columns + 1) as u16),
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
    for row in 0..=rows {
        let row_ref = scope.cage(None);
        glory::ui::view()
            .role(Role::Row)
            .class(if row == 0 {
                "sapid-resources-day-headings"
            } else {
                "sapid-resources-time-row"
            })
            .semantics(Some(Box::new(Semantics {
                table: Some(TableMetadata::Row { index: row as u32 }),
                ..Default::default()
            })))
            .style(scope.bond(move || {
                Style::new()
                    .position(Position::Relative)
                    .width(Length::Px(if axis == ResourceAxis::Columns {
                        64. + days as f64 * prepared.with(|p| p.column_width)
                    } else {
                        160. + steps as f64 * 64.
                    }))
                    .height(Length::Px(if row == 0 || axis == ResourceAxis::Columns {
                        32.
                    } else {
                        prepared.with(|p| p.column_width)
                    }))
            }))
            .node_ref(row_ref)
            .show_in(&grid_scope)?;
        let row_scope = child(scope, row_ref)?;
        for col in 0..=columns {
            let cell = scope.cage(None);
            let at = if row == 0 || col == 0 {
                0
            } else if axis == ResourceAxis::Columns {
                (row - 1) * days + col - 1
            } else {
                (col - 1) * days + row - 1
            };
            let resource_col = if axis == ResourceAxis::Columns {
                col.saturating_sub(1)
            } else {
                row.saturating_sub(1)
            };
            let active = root.selection.source.clone();
            let state_for_cell = state.clone();
            glory::ui::view().role(if row==0{Role::ColumnHeader}else{Role::GridCell})
                .class(if col==0{"sapid-resources-time-label"}else{"sapid-resources-slot-cell"})
                .semantics(scope.bond(move||Some(Box::new(Semantics{selected:if row==0||col==0{None}else{Some(active.with(|v|matches!(v,Some(ScheduleSelection::Slot{interval,resource:Some(key)})if state_for_cell.interval(at).ok()==Some(*interval)&&accepted.with(|m|m.visible_resources()[resource_col/date_days].key()==key))))},table:Some(TableMetadata::Cell{row:row as u32,column:col as u16,row_span:1,column_span:1,headers:vec![]}),..Default::default()}))))
                .style(scope.bond(move||{
                    let width=if axis==ResourceAxis::Columns {prepared.with(|p|p.column_width)} else {64.};
                    let gutter=if axis==ResourceAxis::Columns {64.} else {160.};
                    let physical=if col==0 {0} else if direction==Direction::Ltr {col} else {columns+1-col};
                    Style::new().position(Position::Absolute).left(Length::Px(if physical==0 {0.} else {gutter+(physical-1)as f64*width})).top(Length::Px(0.)).width(Length::Px(if col==0 {gutter} else {width})).height(Length::Px(if row==0||axis==ResourceAxis::Columns {32.} else {prepared.with(|p|p.column_width)}))
                })).node_ref(cell).show_in(&row_scope)?;
            let cell_scope = child(scope, cell)?;
            if row == 0 || col == 0 {
                let label = scope.bond(move || {
                    if row == 0 && col == 0 {
                        return "Resource / time".to_owned();
                    }
                    let resource_header = if axis == ResourceAxis::Columns {
                        row == 0
                    } else {
                        col == 0
                    };
                    if resource_header {
                        accepted.with(|m| {
                            if month {
                                return m.visible_resources()[resource_col].title().to_owned();
                            }
                            let date = m
                                .data()
                                .window()
                                .start()
                                .date()
                                .checked_add_days((resource_col % date_days) as i32)
                                .unwrap();
                            format!(
                                "{} · {} {:02}-{:02}",
                                m.visible_resources()[resource_col / date_days].title(),
                                locale.weekday_short(date.weekday()),
                                date.month(),
                                date.day()
                            )
                        })
                    } else {
                        let i = if axis == ResourceAxis::Columns {
                            row - 1
                        } else {
                            col - 1
                        };
                        if month {
                            let date = accepted.with(|m| {
                                m.data()
                                    .window()
                                    .start()
                                    .date()
                                    .checked_add_days(i as i32)
                                    .unwrap()
                            });
                            return format!(
                                "{} {:02}-{:02}",
                                locale.weekday_short(date.weekday()),
                                date.month(),
                                date.day()
                            );
                        }
                        let minute = time.start_minutes() + i as u16 * time.step_minutes();
                        format!("{:02}:{:02}", minute / 60, minute % 60)
                    }
                });
                glory::ui::text()
                    .content(label)
                    .style(Style {
                        font_size: Some(11.),
                        ..Style::new()
                    })
                    .show_in(&cell_scope)?;
            } else {
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
                glory::ui::button().class("sapid-resources-slot").node_ref(reference)
                    .label(scope.bond(move||labels.interval(at).map(|span|if month {format!("{:02}-{:02}",span.start().date().month(),span.start().date().day())}else{format!("{:02}:{:02}",span.start().time().hour(),span.start().time().minute())}).unwrap_or_default()))
                    .name(scope.bond(move||names.interval(at).map(|span|if month {format!("{} · {} — {}",accepted.with(|m|m.visible_resources()[at%days].title().to_owned()),locale.date_label(span.start().date()),span.end().date())}else{format!("{} · {} {} — {}",accepted.with(|m|m.visible_resources()[at%days/date_days].title().to_owned()),locale.date_label(span.start().date()),span.start().time(),span.end().time())}).unwrap_or_default()))
                    .enabled(flags.map(|f|f.enabled)).focus(scope.bond(move||if focus.get()==at{FocusMode::TabStop}else{FocusMode::Programmatic}))
                    .style(scope.bond(move||{
                        let mut style=Style::new().width(Length::Percent(100.)).height(Length::Px(if axis==ResourceAxis::Columns {32.} else {prepared.with(|p|p.column_width)})).padding(Edges::all(0.)).overflow_x(Overflow::Hidden).overflow_y(Overflow::Hidden);
                        if !unstyled{let chosen=selected.with(|v|matches!(v,Some(ScheduleSelection::Slot{interval,resource:Some(key)})if styles.interval(at).ok()==Some(*interval)&&accepted.with(|m|m.visible_resources()[at%days/date_days].key()==key)));
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
    let drag_for_events = drag.clone();
    glory::ui::each(
        scope.bond(move || prepared.with(|p| p.events.clone())),
        |event| {
            (
                event.row.key.clone(),
                event.row.interval.start(),
                event.resource.clone(),
                event.date,
            )
        },
        move |event, _| TimelineEventWidget {
            event,
            prepared,
            state: state_for_events.clone(),
            theme: theme_for_events.clone(),
            unstyled,
            direction,
            days,
            height: if axis == ResourceAxis::Columns {
                height - 32.
            } else {
                steps as f64 * 64.
            },
            axis,
            drag: drag_for_events.clone(),
        },
    )
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
                && s.root.enabled.try_get()?
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
    scope.retain(drag)?;
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
    axis: ResourceAxis,
    drag: Rc<drag::Drag>,
}
impl Widget for TimelineEventWidget {
    fn build(&mut self, scope: &Scope) -> Result {
        let event = self.event;
        let prepared = self.prepared;
        let flags = self.state.flags;
        let direction = self.direction;
        let days = self.days;
        let height = self.height;
        let axis = self.axis;
        let theme = self.theme.clone();
        let unstyled = self.unstyled;
        let selected = self.state.selection.source.clone();
        let style=scope.bond(move||{
            let width=prepared.with(|p|p.column_width);event.with(|e|{
                let day=if direction==Direction::Ltr{e.column}else{days-1-e.column};let fraction=width/e.position.lanes()as f64;
                let start=e.position.start_fraction()*height;
                let duration=((e.position.end_fraction()-e.position.start_fraction())*height).max(32.);
                let (left,top,w,h)=if axis==ResourceAxis::Columns {(64.+day as f64*width+e.position.lane()as f64*fraction,32.+start,(fraction-2.).max(2.),duration)} else {
                    let start=if direction==Direction::Ltr {start} else {height-e.position.end_fraction()*height};
                    (160.+start,32.+e.column as f64*width+e.position.lane()as f64*fraction,duration,(fraction-2.).max(32.))
                };
                let mut style=Style::new().position(Position::Absolute).left(Length::Px(left)).top(Length::Px(top))
                    .width(Length::Px(w)).height(Length::Px(h)).padding(Edges::all(1.)).overflow_x(Overflow::Hidden).overflow_y(Overflow::Hidden);
                if !unstyled{let chosen=selected.with(|v|matches!(v,Some(ScheduleSelection::Event{key,candidate,interval})if *key==e.row.key&&*candidate==e.row.candidate&&*interval==e.row.interval));style.background=Some(if chosen{theme.accent}else{theme.accent_soft});style.color=Some(if chosen{theme.on_accent}else{theme.accent_text});style=style.border_width(Edges::all(1.)).border_style(BorderStyle::Solid).border_color(theme.accent);style.font_size=Some(11.);}
                style
            })
        });
        let state = self.state.clone();
        let enabled = scope.bond(move || {
            flags.with(|f| f.enabled && !f.readonly) && event.with(|e| e.row.selectable)
        });
        let card = scope.cage(None);
        glory::ui::view()
            .class("sapid-resources-event-card")
            .style(style)
            .node_ref(card)
            .show_in(scope)?;
        let content = child(scope, card)?;
        let accepted = self.state.accepted;
        let editable = self.state.editable;
        let button_theme = self.theme.clone();
        let button_selected = self.state.selection.source.clone();
        glory::ui::button()
            .class("sapid-resources-timeline-event")
            .label(event.map(|e| e.row.title.clone()))
            .name(event.map(move |e| {
                format!(
                    "{} · {} — {} · {}{}",
                    e.row.title,
                    crate::format_local_date_time(e.row.interval.start()),
                    crate::format_local_date_time(e.row.interval.end()),
                    accepted.with(|m| m
                        .resources()
                        .get(&e.resource)
                        .map(|r| r.title().to_owned())
                        .unwrap_or_default()),
                    if e.row.all_day { " · All day" } else { "" }
                )
            }))
            .style(scope.bond(move||{
                let mut style=Style::new().width(Length::Percent(100.)).height(Length::Percent(if editable {65.}else {100.})).padding(Edges::all(0.));
                if !unstyled {
                    let chosen=button_selected.with(|v|matches!(v,Some(ScheduleSelection::Event{key,candidate,interval})if event.with(|e|e.row.key==*key&&e.row.candidate==*candidate&&e.row.interval==*interval)));
                    style.background=Some(if chosen {button_theme.accent}else{button_theme.accent_soft});
                    style.color=Some(if chosen {button_theme.on_accent}else{button_theme.accent_text});
                    style.font_size=Some(11.);style.border_width=Some(Edges::all(0.));
                }
                style
            }))
            .enabled(enabled)
            .on(EventKind::Activate, move |e| {
                let row = event.with(|e| e.row.clone());
                state.report(state.event(&row));
                state.activated.emit(state.owner, e);
            })
            .show_in(&content)?;
        if self.state.editable {
            let handles = scope.cage(None);
            glory::ui::view()
                .class("sapid-resources-drag-handles")
                .style(
                    Style::new()
                        .display(Display::Flex)
                        .width(Length::Percent(100.))
                        .height(Length::Percent(35.)),
                )
                .node_ref(handles)
                .show_in(&content)?;
            let handle_scope = child(scope, handles)?;
            for (kind, label) in [
                (crate::ScheduleEditKind::Move, "Drag to move event"),
                (crate::ScheduleEditKind::ResizeStart, "Drag to resize start"),
                (crate::ScheduleEditKind::ResizeEnd, "Drag to resize end"),
            ] {
                let handle = scope.cage(None);
                let drag = self.drag.clone();
                let root = self.state.clone();
                let enabled = scope.bond(move || {
                    flags.with(|f| f.enabled && !f.readonly)
                        && event.with(|e| accepted_permission(root.accepted, &e.row.key, kind))
                });

                let color = self.theme.accent;
                glory::ui::view()
                    .role(Role::Group)
                    .class(match kind {
                        crate::ScheduleEditKind::Move => "sapid-resources-drag-move",
                        crate::ScheduleEditKind::ResizeStart => "sapid-resources-drag-start",
                        crate::ScheduleEditKind::ResizeEnd => "sapid-resources-drag-end",
                    })
                    .name(label)
                    .node_ref(handle)
                    .semantics(scope.bond(move || {
                        Some(Box::new(Semantics {
                            drag_capture: Some(glory::render::DragCapture::Press),
                            disabled: !enabled.get(),
                            ..Default::default()
                        }))
                    }))
                    .style(Style {
                        background: (!unstyled).then_some(color),
                        ..Style::new()
                            .width(Length::Percent(33.333333))
                            .height(Length::Percent(100.))
                    })
                    .on(EventKind::Drag, move |e| {
                        let target = event.with(|v| EditTarget {
                            event: v.row.key.clone(),
                            candidate: v.row.candidate,
                            interval: v.row.interval,
                            resource: v.resource.clone(),
                        });
                        let date = event.with(|v| v.date);
                        drag.root_report(drag.event(target, date, kind, e));
                    })
                    .show_in(&handle_scope)?;
                let label_scope = child(scope, handle)?;
                glory::ui::text()
                    .content(match kind {
                        crate::ScheduleEditKind::Move => "Move",
                        crate::ScheduleEditKind::ResizeStart => "Start",
                        crate::ScheduleEditKind::ResizeEnd => "End",
                    })
                    .style(Style {
                        font_size: Some(9.),
                        color: (!unstyled).then_some(self.theme.on_accent),
                        ..Style::new()
                    })
                    .show_in(&label_scope)?;
                scope.retain(label_scope)?;
            }
            scope.retain(handle_scope)?;
        }
        scope.retain(content)
    }
}
fn accepted_permission(
    data: Cage<ResourceScheduleData>,
    key: &ScheduleKey,
    kind: crate::ScheduleEditKind,
) -> bool {
    data.with(|m| {
        m.data()
            .events()
            .iter()
            .find(|e| e.key() == key)
            .is_some_and(|e| match kind {
                crate::ScheduleEditKind::Move => e.edit_permissions().move_event,
                _ => e.edit_permissions().resize_event,
            })
    })
}
