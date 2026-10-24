//! Current-window schedule composition. Routing and loading belong to the app.
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback, Observers};
use crate::state::{Control, Dirty, Value};
use crate::{
    DateLocale, MAX_SCHEDULE_OCCURRENCES, ScheduleData, ScheduleInterval, ScheduleKey,
    ScheduleWindow, ScheduleZone,
};
use glory::render::{DocumentPart, Event, EventKind, NodeId, Role, Semantics};
use glory::style::{BorderStyle, Display, Edges, FlexDirection, FlexWrap, Length, Style};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Lotus, Scope};
use std::{cell::Cell, rc::Rc};
mod calendar;
mod resources;
mod timeline;
pub use resources::*;
fn invalid() -> glory::ui::Error {
    glory::render::Error::InvalidProps("invalid finite schedule window").into()
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScheduleSelection {
    Event {
        key: ScheduleKey,
        candidate: u32,
        interval: ScheduleInterval,
    },
    Slot {
        interval: ScheduleInterval,
        resource: Option<ScheduleKey>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleRequest {
    pub window: ScheduleWindow,
    pub zone: ScheduleZone,
    pub revision: u64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScheduleView {
    #[default]
    Agenda,
    Day,
    Week,
    Month,
    Year,
    MobileMonth,
}
#[derive(Clone, Copy, PartialEq)]
struct Flags {
    enabled: bool,
    readonly: bool,
}
pub(crate) struct Controller {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    live: Cell<bool>,
    accepted: Cage<ScheduleData>,
    flags: Cage<Flags>,
    selection: Control<Option<ScheduleSelection>>,
    request_cursor: Cell<Option<ScheduleWindow>>,
    revision: Cell<u64>,
    dirty: Dirty,
    requested: Callback<ScheduleRequest>,
    selected: Callback<ScheduleSelection>,
    activated: Observers,
    grid_events: Observers,
    view: ScheduleView,
}
impl Controller {
    fn ensure(&self) -> Result {
        if self.live.get() && self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn report(&self, result: Result) {
        if let Err(e) = result
            && let Some(scope) = self.weak.upgrade()
        {
            scope.report_error(e);
        }
    }
    fn request(&self, delta: i32) -> Result {
        self.ensure()?;
        if !self.flags.try_with(|s| s.enabled)? {
            return Ok(());
        }
        let window = self
            .request_cursor
            .get()
            .unwrap_or(self.accepted.try_with(|data| data.window())?);
        let Ok(next) = shift_window(window, self.view, delta) else {
            return Ok(());
        };
        let revision = self.revision.get().checked_add(1).ok_or_else(invalid)?;
        self.revision.set(revision);
        self.request_cursor.set(Some(next));
        let zone = self.accepted.try_with(|data| data.zone().clone())?;
        self.requested.emit(
            self.owner,
            ScheduleRequest {
                window: next,
                zone,
                revision,
            },
        );
        if self.live.get() && self.owner.is_alive() {
            self.dirty.touch()?;
        }
        Ok(())
    }
    fn event(&self, row: &AgendaRow) -> Result {
        self.ensure()?;
        if !self.flags.try_with(|s| s.enabled && !s.readonly)? {
            return Ok(());
        }
        let allowed = self.accepted.try_with(|data| -> Result<bool> {
            Ok(data
                .expand(MAX_SCHEDULE_OCCURRENCES)
                .map_err(|_| invalid())?
                .iter()
                .any(|occurrence| {
                    data.event(*occurrence).is_some_and(|event| {
                        event.edit_permissions().select
                            && event.key() == &row.key
                            && occurrence.candidate_index() == row.candidate
                            && occurrence.interval() == row.interval
                    })
                }))
        })??;
        if !allowed {
            return Ok(());
        }
        let value = ScheduleSelection::Event {
            key: row.key.clone(),
            candidate: row.candidate,
            interval: row.interval,
        };
        self.selection.accept(&Some(value.clone()))?;
        self.selected.emit(self.owner, value);
        Ok(())
    }
    fn retire(&self) {
        self.live.set(false);
        self.request_cursor.set(None);
        self.requested.clear();
        self.selected.clear();
        self.activated.clear();
        self.grid_events.clear();
    }
}
/// A finite current-window schedule. The application accepts page requests by
/// supplying new ScheduleData; no network, clock or route is installed.
pub struct Schedule {
    common: Common,
    key: String,
    data: Lotus<ScheduleData>,
    selection: Value<Option<ScheduleSelection>>,
    readonly: Lotus<bool>,
    locale: DateLocale,
    requested: Option<Box<dyn FnMut(ScheduleRequest)>>,
    selected: Option<Box<dyn FnMut(ScheduleSelection)>>,
    view: ScheduleView,
    time_window: crate::ScheduleTimeWindow,
    direction: glory::render::Direction,
    grid_ref: Option<Cage<Option<NodeId>>>,
    viewport_ref: Option<Cage<Option<NodeId>>>,
    column_width: Option<f64>,
    viewport_height: f64,
    week_start: crate::Weekday,
}
pub fn schedule(key: impl Into<String>, data: impl Into<Lotus<ScheduleData>>) -> Schedule {
    Schedule {
        common: Common::default(),
        key: key.into(),
        data: data.into(),
        selection: Value::default(),
        readonly: false.into(),
        locale: DateLocale::En,
        requested: None,
        selected: None,
        view: ScheduleView::Agenda,
        time_window: crate::ScheduleTimeWindow::default(),
        direction: glory::render::Direction::Ltr,
        grid_ref: None,
        viewport_ref: None,
        column_width: None,
        viewport_height: 520.,
        week_start: crate::Weekday::Monday,
    }
}
pub type AgendaView = Schedule;
pub type DayView = Schedule;
pub type WeekView = Schedule;
pub type MonthView = Schedule;
pub type YearView = Schedule;
pub type MobileMonthView = Schedule;
pub fn month_view(key: impl Into<String>, data: impl Into<Lotus<ScheduleData>>) -> MonthView {
    schedule(key, data).view(ScheduleView::Month)
}
pub fn year_view(key: impl Into<String>, data: impl Into<Lotus<ScheduleData>>) -> YearView {
    schedule(key, data).view(ScheduleView::Year)
}
pub fn mobile_month_view(
    key: impl Into<String>,
    data: impl Into<Lotus<ScheduleData>>,
) -> MobileMonthView {
    schedule(key, data).view(ScheduleView::MobileMonth)
}
pub fn day_view(key: impl Into<String>, data: impl Into<Lotus<ScheduleData>>) -> DayView {
    schedule(key, data).view(ScheduleView::Day)
}
pub fn week_view(key: impl Into<String>, data: impl Into<Lotus<ScheduleData>>) -> WeekView {
    schedule(key, data).view(ScheduleView::Week)
}
pub fn agenda_view(key: impl Into<String>, data: impl Into<Lotus<ScheduleData>>) -> AgendaView {
    schedule(key, data)
}
impl Schedule {
    common_methods!();
    pub fn week_start(mut self, value: crate::Weekday) -> Self {
        self.week_start = value;
        self
    }
    pub fn view(mut self, value: ScheduleView) -> Self {
        self.view = value;
        self
    }
    pub fn time_window(mut self, value: crate::ScheduleTimeWindow) -> Self {
        self.time_window = value;
        self
    }
    pub fn direction(mut self, value: glory::render::Direction) -> Self {
        self.direction = value;
        self
    }
    pub fn grid_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.grid_ref = Some(value);
        self
    }
    pub fn viewport_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.viewport_ref = Some(value);
        self
    }
    /// Minimum timed day-column width in px (96..=1024). Overlap lanes may widen it.
    pub fn column_width(mut self, value: f64) -> Self {
        self.column_width = Some(value);
        self
    }
    /// Scroll viewport height in px (80..=2048), independent of timeline extent.
    pub fn viewport_height(mut self, value: f64) -> Self {
        self.viewport_height = value;
        self
    }
    pub fn selection(mut self, value: impl Into<Lotus<Option<ScheduleSelection>>>) -> Self {
        self.selection.source = Some(value.into());
        self
    }
    pub fn default_selection(mut self, value: Option<ScheduleSelection>) -> Self {
        self.selection.default = value;
        self
    }
    pub fn readonly(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn locale(mut self, value: DateLocale) -> Self {
        self.locale = value;
        self
    }
    pub fn on_window_request(
        mut self,
        callback: impl FnMut(ScheduleRequest) + 'static,
    ) -> Self {
        self.requested = Some(Box::new(callback));
        self
    }
    pub fn on_selection_change(
        mut self,
        callback: impl FnMut(ScheduleSelection) + 'static,
    ) -> Self {
        self.selected = Some(Box::new(callback));
        self
    }
}
fn child(scope: &Scope, id: Cage<Option<NodeId>>) -> Result<Scope> {
    scope.child_in(id.get().ok_or(glory::ui::Error::Gone)?)
}
fn shift_window(
    window: ScheduleWindow,
    view: ScheduleView,
    delta: i32,
) -> std::result::Result<ScheduleWindow, crate::ScheduleError> {
    match view {
        ScheduleView::Month | ScheduleView::Year | ScheduleView::MobileMonth => {
            calendar::window(view, window, delta)
        }
        _ => window.shifted(delta),
    }
}
impl Widget for Schedule {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.key.is_empty()
            || self.key.len() > 48
            || !self
                .key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || self
                .column_width
                .is_some_and(|v| !v.is_finite() || !(96. ..=1024.).contains(&v))
            || !self.viewport_height.is_finite()
            || !(80. ..=2048.).contains(&self.viewport_height)
        {
            return Err(invalid());
        }
        let view = self.view;
        let time_window = self.time_window;
        let accepted = composite::validated(scope, &self.data, move |data| {
            data.expand(MAX_SCHEDULE_OCCURRENCES)
                .map_err(|_| invalid())?;
            match view {
                ScheduleView::Agenda => Ok(()),
                ScheduleView::Day => timeline::validate(data, 1, time_window),
                ScheduleView::Week => timeline::validate(data, 7, time_window),
                ScheduleView::Month | ScheduleView::Year | ScheduleView::MobileMonth => {
                    calendar::validate(data, view)
                }
            }
        })?;
        let selection = std::mem::take(&mut self.selection).mount(scope);
        let enabled = self.common.enabled.clone();
        let readonly = self.readonly.clone();
        let flags = scope.bond(move || Flags {
            enabled: enabled.get(),
            readonly: readonly.get(),
        });
        let flags = composite::validated(scope, &flags.into(), |_: &Flags| Ok(()))?;
        let state = Rc::new(Controller {
            owner: scope.id(),
            weak: scope.downgrade(),
            live: Cell::new(true),
            accepted,
            flags,
            view,
            selection,
            request_cursor: Cell::new(None),
            revision: Cell::new(0),
            dirty: Dirty::new(scope),
            requested: Callback::new(self.requested.take()),
            selected: Callback::new(self.selected.take()),
            grid_events: Observers::new(if view == ScheduleView::Agenda {
                vec![]
            } else {
                self.common
                    .handlers
                    .remove(&EventKind::GridNavigate)
                    .unwrap_or_default()
            }),
            activated: Observers::new(
                self.common
                    .handlers
                    .remove(&EventKind::Activate)
                    .unwrap_or_default(),
            ),
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
            match view {
                ScheduleView::Agenda => "sapid-schedule sapid-agenda-view",
                ScheduleView::Day => "sapid-schedule sapid-day-view",
                ScheduleView::Week => "sapid-schedule sapid-week-view",
                ScheduleView::Month => "sapid-schedule sapid-month-view",
                ScheduleView::Year => "sapid-schedule sapid-year-view",
                ScheduleView::MobileMonth => "sapid-schedule sapid-mobile-month-view",
            },
            Style {
                color: Some(self.common.theme.foreground),
                background: Some(self.common.theme.surface),
                ..composite::column(8.)
            },
        );
        let style = scope.bond(move || {
            let mut s = appearance.get();
            s.display.get_or_insert(Display::Flex);
            s.flex_direction.get_or_insert(FlexDirection::Column);
            s.gap.get_or_insert(8.);
            s
        });
        let id = composite::id("schedule", &self.key, "root", "root")?;
        let mut widget = glory::ui::view()
            .role(Role::Group)
            .class(class)
            .style(style)
            .node_ref(root)
            .focus(self.common.focus)
            .name(
                self.common
                    .name
                    .take()
                    .unwrap_or_else(|| "Schedule".to_owned().into()),
            )
            .semantics(Some(Box::new(Semantics {
                id: Some(id),
                ..Default::default()
            })));
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            widget = widget.on(kind, move |e| {
                crate::appearance::observe(&mut handlers, owner, e)
            });
        }
        widget.show_in(scope)?;
        let content = child(scope, root)?;
        let header = scope.cage(None);
        glory::ui::view()
            .class("sapid-schedule-header")
            .style(Style {
                flex_wrap: Some(FlexWrap::Wrap),
                ..Style::new().display(Display::Flex).gap(8.)
            })
            .node_ref(header)
            .show_in(&content)?;
        let header_scope = child(scope, header)?;
        for (delta, label) in [
            (-1, "Previous schedule window"),
            (1, "Next schedule window"),
        ] {
            let s = state.clone();
            let mut button = crate::button()
                .label(label)
                .variant(crate::ButtonVariant::Secondary)
                .theme(self.common.theme.clone())
                .size(self.common.size)
                .enabled(scope.bond(move || {
                    flags.with(|f| f.enabled)
                        && accepted
                            .with(|data| shift_window(data.window(), view, delta).is_ok())
                }))
                .on_press(move || {
                    s.report(s.request(delta));
                    s.activated.emit(s.owner, Event::Activate);
                });
            if self.common.unstyled {
                button = button.unstyled();
            }
            button.show_in(&header_scope)?;
        }
        let locale = self.locale;
        let title = accepted.map(move |data| {
            format!(
                "{} — {}",
                locale.date_label(data.window().start().date()),
                data.window().end().date()
            )
        });
        crate::text()
            .theme(self.common.theme.clone())
            .class("sapid-schedule-window")
            .content(title)
            .show_in(&content)?;
        let groups = scope.bond(move || {
            accepted.with(|data| {
                if matches!(
                    view,
                    ScheduleView::Month | ScheduleView::Year | ScheduleView::MobileMonth
                ) {
                    return vec![];
                }
                let mut groups = agenda_groups(data, locale);
                if matches!(view, ScheduleView::Day | ScheduleView::Week) {
                    for group in &mut groups {
                        group.rows.retain(|row| row.all_day);
                    }
                    groups.retain(|group| !group.rows.is_empty());
                }
                groups
            })
        });
        let theme = self.common.theme.clone();
        let size = self.common.size;
        let unstyled = self.common.unstyled;
        let s = state.clone();
        glory::ui::each(
            groups,
            |day| day.date,
            move |day, _| AgendaDayWidget {
                day,
                state: s.clone(),
                theme: theme.clone(),
                size,
                unstyled,
            },
        )
        .show_in(&content)?;
        if matches!(view, ScheduleView::Day | ScheduleView::Week) {
            timeline::build(
                &content,
                state.clone(),
                timeline::Config {
                    key: self.key.clone(),
                    days: if view == ScheduleView::Day { 1 } else { 7 },
                    time: time_window,
                    theme: self.common.theme.clone(),
                    unstyled: self.common.unstyled,
                    direction: self.direction,
                    locale,
                    grid_ref: self.grid_ref,
                    viewport_ref: self.viewport_ref,
                    column_width: self.column_width.unwrap_or(if view == ScheduleView::Day {
                        480.
                    } else {
                        120.
                    }),
                    viewport_height: self.viewport_height,
                },
            )?;
        } else if view != ScheduleView::Agenda {
            calendar::build(
                &content,
                state.clone(),
                calendar::Config {
                    key: self.key.clone(),
                    view,
                    locale,
                    week_start: self.week_start,
                    direction: self.direction,
                    theme: self.common.theme.clone(),
                    unstyled: self.common.unstyled,
                    size: self.common.size,
                    grid_ref: self.grid_ref,
                },
            )?;
        }
        let empty = accepted.map(|data| {
            data.expand(MAX_SCHEDULE_OCCURRENCES)
                .is_ok_and(|items| items.is_empty())
        });
        let theme = self.common.theme.clone();
        glory::ui::when(empty, move || {
            crate::text()
                .theme(theme.clone())
                .class("sapid-schedule-empty")
                .content("No events in this window")
        })
        .show_in(&content)?;
        let weak = Rc::downgrade(&state);
        let dirty = state.dirty.clone();
        scope.effect(move || {
            if dirty.take()
                && let Some(s) = weak.upgrade()
            {
                s.request_cursor.set(None);
            }
        });
        scope.retain(header_scope)?;
        scope.retain(content)?;
        scope.retain(state)
    }
}
#[derive(Clone)]
struct AgendaRow {
    key: ScheduleKey,
    candidate: u32,
    interval: ScheduleInterval,
    title: String,
    resource: String,
    all_day: bool,
    selectable: bool,
}
#[derive(Clone)]
struct AgendaDay {
    date: crate::CivilDate,
    label: String,
    rows: Vec<AgendaRow>,
}
fn agenda_groups(data: &ScheduleData, locale: DateLocale) -> Vec<AgendaDay> {
    let mut groups: Vec<AgendaDay> = vec![];
    let Ok(occurrences) = data.expand(MAX_SCHEDULE_OCCURRENCES) else {
        return groups;
    };
    for occurrence in occurrences {
        let Some(event) = data.event(occurrence) else {
            continue;
        };
        let date = occurrence
            .interval()
            .start()
            .max(data.window().start())
            .date();
        if groups.last().is_none_or(|day| day.date != date) {
            groups.push(AgendaDay {
                date,
                label: locale.date_label(date),
                rows: vec![],
            });
        }
        groups.last_mut().unwrap().rows.push(AgendaRow {
            key: event.key().clone(),
            candidate: occurrence.candidate_index(),
            interval: occurrence.interval(),
            title: event.title().into(),
            resource: event
                .resource_key()
                .map(|key| key.as_str().to_owned())
                .unwrap_or_default(),
            all_day: event.is_all_day(),
            selectable: event.edit_permissions().select,
        });
    }
    groups
}
struct AgendaDayWidget {
    day: Cage<AgendaDay>,
    state: Rc<Controller>,
    theme: crate::Theme,
    size: crate::Size,
    unstyled: bool,
}
impl Widget for AgendaDayWidget {
    fn build(&mut self, scope: &Scope) -> Result {
        let day = self.day;
        let root = scope.cage(None);
        glory::ui::view()
            .class("sapid-agenda-day")
            .style(composite::column(4.))
            .node_ref(root)
            .show_in(scope)?;
        let content = child(scope, root)?;
        crate::title(2)
            .theme(self.theme.clone())
            .content(day.map(|day| day.label.clone()))
            .show_in(&content)?;
        let list = scope.cage(None);
        glory::ui::view()
            .document_part(DocumentPart::UnorderedList)
            .semantics(Some(Box::new(Semantics {
                document_part: Some(DocumentPart::UnorderedList),
                ..Default::default()
            })))
            .class("sapid-agenda-events")
            .style(
                Style::new()
                    .padding(Edges {
                        left: 20.,
                        ..Edges::all(0.)
                    })
                    .margin(Edges::all(0.)),
            )
            .node_ref(list)
            .show_in(&content)?;
        let list_scope = child(scope, list)?;
        let state = self.state.clone();
        let theme = self.theme.clone();
        let size = self.size;
        let unstyled = self.unstyled;
        glory::ui::each(
            day.map(|day| day.rows.clone()),
            |row| (row.key.clone(), row.interval.start()),
            move |row, _| AgendaEventWidget {
                row,
                state: state.clone(),
                theme: theme.clone(),
                size,
                unstyled,
            },
        )
        .show_in(&list_scope)?;
        scope.retain(list_scope)?;
        scope.retain(content)
    }
}
struct AgendaEventWidget {
    row: Cage<AgendaRow>,
    state: Rc<Controller>,
    theme: crate::Theme,
    size: crate::Size,
    unstyled: bool,
}
impl Widget for AgendaEventWidget {
    fn build(&mut self, scope: &Scope) -> Result {
        let row = self.row;
        let flags = self.state.flags;
        let root = scope.cage(None);
        glory::ui::view()
            .document_part(DocumentPart::ListItem)
            .semantics(Some(Box::new(Semantics {
                document_part: Some(DocumentPart::ListItem),
                ..Default::default()
            })))
            .class("sapid-agenda-event")
            .style(Style::new().margin(Edges::all(0.)))
            .node_ref(root)
            .show_in(scope)?;
        let content = child(scope, root)?;
        let active = self.state.selection.source.clone();
        let chosen=scope.bond(move||active.with(|v|matches!(v,Some(ScheduleSelection::Event{key,candidate,interval})if row.with(|r|*key==r.key&&*candidate==r.candidate&&*interval==r.interval))));
        let label = scope.bond(move || {
            let selected = chosen.get();
            row.with(|row| {
                let range = if row.all_day {
                    format!(
                        "All day {} — {}",
                        row.interval.start().date(),
                        row.interval.end().date().checked_add_days(-1).unwrap()
                    )
                } else if row.interval.start().date() == row.interval.end().date() {
                    format!(
                        "{} — {}",
                        row.interval.start().time(),
                        row.interval.end().time()
                    )
                } else {
                    format!(
                        "{} — {}",
                        crate::format_local_date_time(row.interval.start()),
                        crate::format_local_date_time(row.interval.end())
                    )
                };
                format!(
                    "{} · {}{}{}",
                    row.title,
                    range,
                    if row.resource.is_empty() {
                        String::new()
                    } else {
                        format!(" · {}", row.resource)
                    },
                    if selected { " [selected]" } else { "" },
                )
            })
        });
        let enabled = scope.bond(move || {
            flags.with(|f| f.enabled && !f.readonly) && row.with(|r| r.selectable)
        });
        let theme = self.theme.clone();
        let unstyled = self.unstyled;
        let style = scope.bond(move || {
            let selected = chosen.get();
            let style = Style::new()
                .width(Length::Percent(100.))
                .min_height(Length::Px(32.));
            if unstyled {
                style
            } else {
                style
                    .border_width(Edges::all(if selected { 2. } else { 1. }))
                    .border_style(BorderStyle::Solid)
                    .border_color(if selected { theme.accent } else { theme.border })
            }
        });
        let state = self.state.clone();
        let mut button = crate::button()
            .label(label)
            .variant(crate::ButtonVariant::Secondary)
            .theme(self.theme.clone())
            .size(self.size)
            .enabled(enabled)
            .style(style)
            .on_press(move || {
                let value = row.get();
                state.report(state.event(&value));
                state.activated.emit(state.owner, Event::Activate);
            });
        if self.unstyled {
            button = button.unstyled();
        }
        button.show_in(&content)?;
        scope.retain(content)
    }
}
