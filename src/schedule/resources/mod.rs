//! Finite resource/date windows. Loading and accepting edit proposals belong to applications.
use super::*;
use crate::{
    ResourceScheduleData, ScheduleEdit, ScheduleEditKind, ScheduleResourceWindow,
    ScheduleTimeWindow,
};
use std::{cell::RefCell, collections::VecDeque};
mod drag;
mod timeline;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResourceAxis {
    #[default]
    Columns,
    Rows,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResourceScheduleView {
    #[default]
    Day,
    Week,
    Month,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceScheduleRequest {
    pub window: ScheduleWindow,
    pub resource_window: ScheduleResourceWindow,
    pub zone: ScheduleZone,
    pub revision: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct EditTarget {
    event: ScheduleKey,
    candidate: u32,
    interval: ScheduleInterval,
    resource: ScheduleKey,
}
struct ResourceController {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    live: Cell<bool>,
    accepted: Cage<ResourceScheduleData>,
    flags: Cage<Flags>,
    enabled: Lotus<bool>,
    readonly: Lotus<bool>,
    selection: Control<Option<ScheduleSelection>>,
    view: ResourceScheduleView,
    step: u16,
    page_capacity: usize,
    editable: bool,
    revision: Cell<u64>,
    cursor: Cell<Option<(ScheduleWindow, ScheduleResourceWindow)>>,
    dirty: Dirty,
    requested: Callback<ResourceScheduleRequest>,
    selected: Callback<ScheduleSelection>,
    edited: Callback<ScheduleEdit>,
    activated: Observers,
    grid_events: Observers,
    drag_events: Observers,
    edit_target: Cage<Option<EditTarget>>,
    edit_queue: RefCell<VecDeque<(ScheduleEditKind, i32, i32)>>,
    edit_cursor: RefCell<Option<(ScheduleEditKind, EditTarget, EditTarget)>>,
    edits_dirty: Dirty,
}
impl ResourceController {
    fn ensure(&self) -> Result {
        if self.live.get() && self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn report(&self, result: Result) {
        if let Err(error) = result
            && let Some(scope) = self.weak.upgrade()
        {
            scope.report_error(error)
        }
    }
    fn ready(&self) -> Result<bool> {
        self.ensure()?;
        Ok(self.enabled.try_get()? && !self.readonly.try_get()?)
    }
    fn request(&self, dates: i32, resources: i32) -> Result {
        self.ensure()?;
        if !self.enabled.try_get()? {
            return Ok(());
        }
        let (window, page, total, zone) = self.accepted.try_with(|m| {
            (
                m.data().window(),
                m.resource_window(),
                m.resources().items().len(),
                m.data().zone().clone(),
            )
        })?;
        let (window, page) = self.cursor.get().unwrap_or((window, page));
        let window = if dates == 0 {
            window
        } else if self.view == ResourceScheduleView::Month {
            let Ok(month) = window.start().date().year_month().checked_add(dates) else {
                return Ok(());
            };
            let Ok(next) = ScheduleWindow::month(month) else {
                return Ok(());
            };
            next
        } else {
            let Ok(next) = window.shifted(dates) else {
                return Ok(());
            };
            next
        };
        let page = if resources == 0 {
            page
        } else {
            let first = if resources < 0 {
                page.first().saturating_sub(self.page_capacity)
            } else {
                let Some(first) = page
                    .first()
                    .checked_add(self.page_capacity)
                    .filter(|first| *first < total)
                else {
                    return Ok(());
                };
                first
            };
            if first == page.first() {
                return Ok(());
            }
            ScheduleResourceWindow::new(first, self.page_capacity.min(total - first))
                .map_err(|_| invalid())?
        };
        let revision = self.revision.get().checked_add(1).ok_or_else(invalid)?;
        self.revision.set(revision);
        self.cursor.set(Some((window, page)));
        self.requested.emit(
            self.owner,
            ResourceScheduleRequest {
                window,
                resource_window: page,
                zone,
                revision,
            },
        );
        if self.live.get() && self.owner.is_alive() {
            self.dirty.touch()?;
        }
        Ok(())
    }
    fn select(&self, value: ScheduleSelection) -> Result {
        if !self.ready()? {
            return Ok(());
        }
        let allowed = self.accepted.try_with(|m| match &value {
            ScheduleSelection::Event {
                key,
                candidate,
                interval,
            } => m
                .data()
                .expand(MAX_SCHEDULE_OCCURRENCES)
                .is_ok_and(|items| {
                    items.iter().any(|occ| {
                        m.data().event(*occ).is_some_and(|event| {
                            event.key() == key
                                && event.edit_permissions().select
                                && occ.candidate_index() == *candidate
                                && occ.interval() == *interval
                                && m.visible_resources()
                                    .iter()
                                    .any(|r| Some(r.key()) == event.resource_key())
                        })
                    })
                }),
            ScheduleSelection::Slot {
                interval,
                resource: Some(resource),
            } => {
                m.visible_resources().iter().any(|r| r.key() == resource)
                    && m.data().window().interval().overlaps(*interval)
            }
            _ => false,
        })?;
        if allowed {
            self.selection.accept(&Some(value.clone()))?;
            self.selected.emit(self.owner, value);
        }
        Ok(())
    }
    fn event(&self, row: &AgendaRow) -> Result {
        self.select(ScheduleSelection::Event {
            key: row.key.clone(),
            candidate: row.candidate,
            interval: row.interval,
        })
    }
    fn open_edit(&self, target: EditTarget) -> Result {
        if !self.ready()? || !self.editable {
            return Ok(());
        }
        let allowed = self.accepted.try_with(|m| {
            m.data()
                .expand(MAX_SCHEDULE_OCCURRENCES)
                .is_ok_and(|items| {
                    items.iter().any(|occ| {
                        m.data().event(*occ).is_some_and(|event| {
                            event.key() == &target.event
                                && occ.candidate_index() == target.candidate
                                && occ.interval() == target.interval
                                && event.resource_key() == Some(&target.resource)
                                && (event.edit_permissions().move_event
                                    || event.edit_permissions().resize_event)
                        })
                    })
                })
        })?;
        if allowed {
            self.edit_queue.borrow_mut().clear();
            self.edit_cursor.borrow_mut().take();
            self.edit_target.try_revise(|v| *v = Some(target))?;
        }
        Ok(())
    }
    fn proposal(&self, edit: ScheduleEdit) -> Result {
        if !self.ready()? || !self.editable {
            return Ok(());
        }
        let allowed = self
            .accepted
            .try_with(|m| m.data().validate_edit(m.resources(), &edit).is_ok())?;
        if allowed {
            self.edited.emit(self.owner, edit);
        }
        Ok(())
    }
    fn keyboard_edit(&self, kind: ScheduleEditKind, steps: i32, resource_delta: i32) -> Result {
        if !self.ready()? || !self.editable {
            return Ok(());
        }
        // One host turn has bounded pending intents; process in FIFO order,
        // allowing structure to accept each preceding proposal between effects.
        if self.edit_queue.borrow().len() == 64 {
            return Ok(());
        }
        self.edit_queue
            .borrow_mut()
            .push_back((kind, steps, resource_delta));
        self.edits_dirty.touch()
    }
    fn drain_edit(&self) -> Result {
        let command = self.edit_queue.borrow_mut().pop_front();
        let Some((kind, steps, resource_delta)) = command else {
            return Ok(());
        };
        let result = (|| -> Result {
            if !self.ready()? {
                self.edit_queue.borrow_mut().clear();
                self.edit_cursor.borrow_mut().take();
                return Ok(());
            }
            let Some(chosen) = self.edit_target.try_get()? else {
                return Ok(());
            };
            let current = self.accepted.try_with(|m| {
                m.data()
                    .expand(MAX_SCHEDULE_OCCURRENCES)
                    .ok()?
                    .into_iter()
                    .find_map(|occ| {
                        let event = m.data().event(occ)?;
                        (event.key() == &chosen.event
                            && occ.candidate_index() == chosen.candidate)
                            .then(|| EditTarget {
                                event: chosen.event.clone(),
                                candidate: chosen.candidate,
                                interval: occ.interval(),
                                resource: event.resource_key().unwrap().clone(),
                            })
                    })
            })?;
            let Some(current) = current else {
                self.edit_queue.borrow_mut().clear();
                self.edit_cursor.borrow_mut().take();
                self.edit_target.try_revise(|v| *v = None)?;
                return Ok(());
            };
            let cursor = self.edit_cursor.borrow().clone();
            let (original, target) = match cursor {
                Some((previous, base, target)) if current == base && previous == kind => {
                    (base, target)
                }
                Some((previous, base, _)) if current == base && previous != kind => {
                    (current.clone(), current)
                }
                Some((_, _, target)) if current == target => (current.clone(), current),
                Some(_) => {
                    self.edit_queue.borrow_mut().clear();
                    return Ok(());
                }
                None => (current.clone(), current),
            };
            let all_day = self.accepted.try_with(|m| {
                m.data()
                    .events()
                    .iter()
                    .find(|e| e.key() == &target.event)
                    .is_some_and(|e| e.is_all_day())
            })?;
            let minutes = i64::from(steps) * if all_day { 1440 } else { i64::from(self.step) };
            let proposed = match kind {
                ScheduleEditKind::Move => target.interval.shifted_minutes(minutes),
                ScheduleEditKind::ResizeStart => {
                    target.interval.resize_start_by_minutes(minutes)
                }
                ScheduleEditKind::ResizeEnd => target.interval.resize_end_by_minutes(minutes),
            };
            let Ok(proposed) = proposed else {
                return Ok(());
            };
            let resource = if resource_delta == 0 {
                target.resource.clone()
            } else {
                let key = self.accepted.try_with(|m| {
                    let visible = m.visible_resources();
                    let at = visible.iter().position(|r| r.key() == &target.resource)?;
                    let next = at.checked_add_signed(resource_delta as isize)?;
                    visible.get(next).map(|r| r.key().clone())
                })?;
                let Some(key) = key else { return Ok(()) };
                key
            };
            let edit = ScheduleEdit {
                event: target.event.clone(),
                candidate: target.candidate,
                kind,
                original: original.interval,
                proposed,
                original_resource: Some(original.resource.clone()),
                proposed_resource: Some(resource.clone()),
            };
            let allowed = self
                .accepted
                .try_with(|m| m.data().validate_edit(m.resources(), &edit).is_ok())?;
            if allowed {
                *self.edit_cursor.borrow_mut() = Some((
                    kind,
                    original,
                    EditTarget {
                        interval: proposed,
                        resource,
                        ..target
                    },
                ));
                self.edited.emit(self.owner, edit);
            }
            Ok(())
        })();
        self.activated.emit(self.owner, Event::Activate);
        if !self.edit_queue.borrow().is_empty() && self.live.get() && self.owner.is_alive() {
            self.edits_dirty.touch()?;
        } else {
            self.edit_cursor.borrow_mut().take();
        }
        result
    }
    fn refresh_target(&self) -> Result {
        let Some(chosen) = self.edit_target.try_get()? else {
            self.accepted.try_with(|_| ())?;
            return Ok(());
        };
        let next = self.accepted.try_with(|m| {
            m.data()
                .expand(MAX_SCHEDULE_OCCURRENCES)
                .unwrap()
                .into_iter()
                .find_map(|occ| {
                    let e = m.data().event(occ).unwrap();
                    (e.key() == &chosen.event
                        && occ.candidate_index() == chosen.candidate
                        && m.visible_resources()
                            .iter()
                            .any(|r| Some(r.key()) == e.resource_key())
                        && (e.edit_permissions().move_event
                            || e.edit_permissions().resize_event))
                        .then(|| EditTarget {
                            event: e.key().clone(),
                            candidate: occ.candidate_index(),
                            interval: occ.interval(),
                            resource: e.resource_key().unwrap().clone(),
                        })
                })
        })?;
        if next.as_ref() != Some(&chosen) {
            self.edit_target.try_revise(|v| *v = next)?;
        }
        Ok(())
    }
    fn retire(&self) {
        self.live.set(false);
        self.cursor.set(None);
        self.edit_queue.borrow_mut().clear();
        self.edit_cursor.borrow_mut().take();
        self.requested.clear();
        self.selected.clear();
        self.edited.clear();
        self.activated.clear();
        self.grid_events.clear();
        self.drag_events.clear();
    }
}
pub struct ResourceSchedule {
    common: Common,
    key: String,
    data: Lotus<ResourceScheduleData>,
    selection: Value<Option<ScheduleSelection>>,
    readonly: Lotus<bool>,
    view: ResourceScheduleView,
    time: ScheduleTimeWindow,
    direction: glory::render::Direction,
    locale: DateLocale,
    column_width: f64,
    viewport_height: f64,
    axis: ResourceAxis,
    page_capacity: Option<usize>,
    grid_ref: Option<Cage<Option<NodeId>>>,
    viewport_ref: Option<Cage<Option<NodeId>>>,
    requested: Option<Box<dyn FnMut(ResourceScheduleRequest)>>,
    selected: Option<Box<dyn FnMut(ScheduleSelection)>>,
    edited: Option<Box<dyn FnMut(ScheduleEdit)>>,
}
pub fn resources_schedule(
    key: impl Into<String>,
    data: impl Into<Lotus<ResourceScheduleData>>,
) -> ResourceSchedule {
    ResourceSchedule {
        common: Common::default(),
        key: key.into(),
        data: data.into(),
        selection: Value::default(),
        readonly: false.into(),
        view: ResourceScheduleView::Day,
        time: ScheduleTimeWindow::default(),
        direction: glory::render::Direction::Ltr,
        locale: DateLocale::En,
        column_width: 160.,
        viewport_height: 520.,
        axis: ResourceAxis::Columns,
        page_capacity: None,
        grid_ref: None,
        viewport_ref: None,
        requested: None,
        selected: None,
        edited: None,
    }
}
pub type ResourcesDayView = ResourceSchedule;
pub type ResourcesWeekView = ResourceSchedule;
pub type ResourcesMonthView = ResourceSchedule;
pub fn resources_day_view(
    key: impl Into<String>,
    data: impl Into<Lotus<ResourceScheduleData>>,
) -> ResourcesDayView {
    resources_schedule(key, data)
}
pub fn resources_week_view(
    key: impl Into<String>,
    data: impl Into<Lotus<ResourceScheduleData>>,
) -> ResourcesWeekView {
    resources_schedule(key, data).view(ResourceScheduleView::Week)
}
pub fn resources_month_view(
    key: impl Into<String>,
    data: impl Into<Lotus<ResourceScheduleData>>,
) -> ResourcesMonthView {
    resources_schedule(key, data)
        .view(ResourceScheduleView::Month)
        .axis(ResourceAxis::Rows)
}
impl ResourceSchedule {
    common_methods!();
    pub fn axis(mut self, value: ResourceAxis) -> Self {
        self.axis = value;
        self
    }
    /// Defaults to the initial admitted window count. Supply this explicitly
    /// when mounting at a partially filled final resource page.
    pub fn resource_page_size(mut self, value: usize) -> Self {
        self.page_capacity = Some(value);
        self
    }
    pub fn view(mut self, value: ResourceScheduleView) -> Self {
        self.view = value;
        self
    }
    pub fn time_window(mut self, value: ScheduleTimeWindow) -> Self {
        self.time = value;
        self
    }
    pub fn direction(mut self, value: glory::render::Direction) -> Self {
        self.direction = value;
        self
    }
    pub fn locale(mut self, value: DateLocale) -> Self {
        self.locale = value;
        self
    }
    pub fn column_width(mut self, value: f64) -> Self {
        self.column_width = value;
        self
    }
    pub fn viewport_height(mut self, value: f64) -> Self {
        self.viewport_height = value;
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
    pub fn on_selection_change(
        mut self,
        callback: impl FnMut(ScheduleSelection) + 'static,
    ) -> Self {
        self.selected = Some(Box::new(callback));
        self
    }
    pub fn on_window_request(
        mut self,
        callback: impl FnMut(ResourceScheduleRequest) + 'static,
    ) -> Self {
        self.requested = Some(Box::new(callback));
        self
    }
    pub fn on_edit(mut self, callback: impl FnMut(ScheduleEdit) + 'static) -> Self {
        self.edited = Some(Box::new(callback));
        self
    }
}
fn validate(
    model: &ResourceScheduleData,
    view: ResourceScheduleView,
    time: ScheduleTimeWindow,
) -> Result {
    model
        .data()
        .expand(MAX_SCHEDULE_OCCURRENCES)
        .map_err(|_| invalid())?;
    match view {
        ResourceScheduleView::Day | ResourceScheduleView::Week => {
            model
                .layout_timeline(
                    time,
                    if view == ResourceScheduleView::Day {
                        1
                    } else {
                        7
                    },
                )
                .map_err(|_| invalid())?;
        }
        ResourceScheduleView::Month => {
            model.layout_month().map_err(|_| invalid())?;
        }
    }
    Ok(())
}
impl Widget for ResourceSchedule {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.key.is_empty()
            || self.key.len() > 40
            || !self
                .key
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
            || !self.column_width.is_finite()
            || !(96. ..=1024.).contains(&self.column_width)
            || !self.viewport_height.is_finite()
            || !(80. ..=2048.).contains(&self.viewport_height)
            || self.page_capacity.is_some_and(|v| !(1..=8).contains(&v))
        {
            return Err(invalid());
        }
        let view = self.view;
        let time = self.time;
        let accepted =
            composite::validated(scope, &self.data, move |m| validate(m, view, time))?;
        let enabled = self.common.enabled.clone();
        let readonly = self.readonly.clone();
        let enabled_flags = enabled.clone();
        let readonly_flags = readonly.clone();
        let flag_source = scope.bond(move || Flags {
            enabled: enabled_flags.get(),
            readonly: readonly_flags.get(),
        });
        let flags = composite::validated(scope, &flag_source.into(), |_: &Flags| Ok(()))?;
        let state = Rc::new(ResourceController {
            owner: scope.id(),
            weak: scope.downgrade(),
            live: Cell::new(true),
            accepted,
            flags,
            enabled,
            readonly,
            selection: std::mem::take(&mut self.selection).mount(scope),
            view,
            step: time.step_minutes(),
            page_capacity: self
                .page_capacity
                .unwrap_or_else(|| accepted.with(|m| m.resource_window().count())),
            editable: self.edited.is_some(),
            revision: Cell::new(0),
            cursor: Cell::new(None),
            dirty: Dirty::new(scope),
            edits_dirty: Dirty::new(scope),
            requested: Callback::new(self.requested.take()),
            selected: Callback::new(self.selected.take()),
            edited: Callback::new(self.edited.take()),
            activated: Observers::new(
                self.common
                    .handlers
                    .remove(&EventKind::Activate)
                    .unwrap_or_default(),
            ),
            grid_events: Observers::new(
                self.common
                    .handlers
                    .remove(&EventKind::GridNavigate)
                    .unwrap_or_default(),
            ),
            drag_events: Observers::new(
                self.common
                    .handlers
                    .remove(&EventKind::Drag)
                    .unwrap_or_default(),
            ),
            edit_target: scope.cage(None),
            edit_queue: RefCell::new(VecDeque::new()),
            edit_cursor: RefCell::new(None),
        });
        let weak = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(state) = weak.upgrade() {
                state.retire()
            }
        })?;
        let root = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let (class, appearance) = self.common.appearance(
            scope,
            "sapid-resource-schedule",
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
        let mut widget = glory::ui::view()
            .role(Role::Group)
            .class(class)
            .style(style)
            .name(
                self.common
                    .name
                    .take()
                    .unwrap_or_else(|| "Resource schedule".to_owned().into()),
            )
            .focus(self.common.focus)
            .node_ref(root);
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
            .class("sapid-resources-header")
            .style(Style {
                flex_wrap: Some(FlexWrap::Wrap),
                ..Style::new().display(Display::Flex).gap(8.)
            })
            .node_ref(header)
            .show_in(&content)?;
        let header_scope = child(scope, header)?;
        let step = match view {
            ResourceScheduleView::Day => 1,
            ResourceScheduleView::Week => 7,
            ResourceScheduleView::Month => 1,
        };
        for (dates, resources, label) in [
            (-step, 0, "Previous date window"),
            (step, 0, "Next date window"),
            (0, -1, "Previous resources"),
            (0, 1, "Next resources"),
        ] {
            let s = state.clone();
            let button_enabled = scope.bond(move || {
                flags.with(|f| f.enabled)
                    && accepted.with(|m| {
                        if resources < 0 {
                            m.resource_window().first() > 0
                        } else if resources > 0 {
                            m.resource_window().first() + m.resource_window().count()
                                < m.resources().items().len()
                        } else if view == ResourceScheduleView::Month {
                            m.data()
                                .window()
                                .start()
                                .date()
                                .year_month()
                                .checked_add(dates)
                                .ok()
                                .is_some_and(|month| ScheduleWindow::month(month).is_ok())
                        } else {
                            m.data().window().shifted(dates).is_ok()
                        }
                    })
            });
            let mut b = crate::button()
                .theme(self.common.theme.clone())
                .size(self.common.size)
                .label(label)
                .enabled(button_enabled)
                .on_press(move || {
                    s.report(s.request(dates, resources));
                    s.activated.emit(s.owner, Event::Activate);
                });
            if self.common.unstyled {
                b = b.unstyled()
            }
            b.show_in(&header_scope)?;
        }
        crate::text()
            .theme(self.common.theme.clone())
            .class("sapid-resources-window")
            .content(accepted.map(|m| {
                format!(
                    "{} — {} · resources {}–{} / {}",
                    m.data().window().start().date(),
                    m.data().window().end().date(),
                    if m.visible_resources().is_empty() {
                        0
                    } else {
                        m.resource_window().first() + 1
                    },
                    m.resource_window().first() + m.visible_resources().len(),
                    m.resources().items().len()
                )
            }))
            .show_in(&content)?;
        let config = FrameConfig {
            key: self.key.clone(),
            view,
            time,
            axis: self.axis,
            locale: self.locale,
            direction: self.direction,
            theme: self.common.theme.clone(),
            unstyled: self.common.unstyled,
            grid_ref: self.grid_ref,
            viewport_ref: self.viewport_ref,
            column_width: self.column_width,
            viewport_height: self.viewport_height,
        };
        let s = state.clone();
        glory::ui::each(
            accepted.map(move |m| {
                vec![(
                    m.visible_resources().len(),
                    if view == ResourceScheduleView::Month {
                        m.data().window().start().date().year_month().days()
                    } else {
                        0
                    },
                )]
            }),
            |count| *count,
            move |_, _| Frame {
                root: s.clone(),
                config: config.clone(),
            },
        )
        .show_in(&content)?;
        if view != ResourceScheduleView::Month {
            build_all_day(
                &content,
                state.clone(),
                &self.common.theme,
                self.common.unstyled,
            )?;
        }
        build_editor(
            &content,
            state.clone(),
            &self.common.theme,
            self.common.unstyled,
        )?;
        let weak = Rc::downgrade(&state);
        scope.effect(move || {
            if let Some(s) = weak.upgrade() {
                s.report(s.refresh_target());
                if s.dirty.take() {
                    s.cursor.set(None)
                }
                if s.edits_dirty.take() {
                    s.report(s.drain_edit())
                }
            }
        });
        scope.retain(header_scope)?;
        scope.retain(content)?;
        scope.retain(state)
    }
}
#[derive(Clone)]
struct FrameConfig {
    key: String,
    view: ResourceScheduleView,
    time: ScheduleTimeWindow,
    axis: ResourceAxis,
    locale: DateLocale,
    direction: glory::render::Direction,
    theme: crate::Theme,
    unstyled: bool,
    grid_ref: Option<Cage<Option<NodeId>>>,
    viewport_ref: Option<Cage<Option<NodeId>>>,
    column_width: f64,
    viewport_height: f64,
}
struct Frame {
    root: Rc<ResourceController>,
    config: FrameConfig,
}
impl Widget for Frame {
    fn build(&mut self, scope: &Scope) -> Result {
        if self
            .root
            .accepted
            .with(|m| m.visible_resources().is_empty())
        {
            return crate::text()
                .content("No resources in this window")
                .show_in(scope);
        }
        timeline::build(
            scope,
            self.root.clone(),
            timeline::Config {
                key: self.config.key.clone(),
                days: if self.config.view == ResourceScheduleView::Day {
                    1
                } else {
                    7
                },
                time: self.config.time,
                month: self.config.view == ResourceScheduleView::Month,
                axis: self.config.axis,
                theme: self.config.theme.clone(),
                unstyled: self.config.unstyled,
                direction: self.config.direction,
                locale: self.config.locale,
                grid_ref: self.config.grid_ref,
                viewport_ref: self.config.viewport_ref,
                column_width: self.config.column_width,
                viewport_height: self.config.viewport_height,
            },
        )
    }
}
fn build_editor(
    scope: &Scope,
    state: Rc<ResourceController>,
    theme: &crate::Theme,
    unstyled: bool,
) -> Result {
    if !state.editable {
        return Ok(());
    }
    let root = scope.cage(None);
    glory::ui::view()
        .role(Role::Group)
        .name("Resource event editor")
        .class("sapid-resources-editor")
        .style(
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .gap(6.),
        )
        .node_ref(root)
        .show_in(scope)?;
    let content = child(scope, root)?;
    let accepted = state.accepted;
    let rows = scope.bond(move || {
        accepted.with(|m| {
            m.data()
                .expand(MAX_SCHEDULE_OCCURRENCES)
                .unwrap()
                .into_iter()
                .filter_map(|occ| {
                    let event = m.data().event(occ).unwrap();
                    ((event.edit_permissions().move_event
                        || event.edit_permissions().resize_event)
                        && m.visible_resources()
                            .iter()
                            .any(|r| Some(r.key()) == event.resource_key()))
                    .then(|| {
                        (
                            EditTarget {
                                event: event.key().clone(),
                                candidate: occ.candidate_index(),
                                interval: occ.interval(),
                                resource: event.resource_key().unwrap().clone(),
                            },
                            event.title().to_owned(),
                        )
                    })
                })
                .collect::<Vec<_>>()
        })
    });
    let s = state.clone();
    let button_theme = theme.clone();
    glory::ui::each(
        rows,
        |row| (row.0.event.clone(), row.0.candidate),
        move |row, _| EditChoice {
            row,
            root: s.clone(),
            theme: button_theme.clone(),
            unstyled,
        },
    )
    .show_in(&content)?;
    let target = state.edit_target;
    crate::text()
        .theme(theme.clone())
        .class("sapid-resources-edit-target")
        .style(Style::new().width(Length::Percent(100.)))
        .content(target.map(|v| {
            v.as_ref().map_or_else(
                || "Choose an event to edit".into(),
                |v| {
                    format!(
                        "Editing {} · {} — {}",
                        v.event.as_str(),
                        crate::format_local_date_time(v.interval.start()),
                        crate::format_local_date_time(v.interval.end())
                    )
                },
            )
        }))
        .show_in(&content)?;
    for (kind, steps, resource, label) in [
        (ScheduleEditKind::Move, -1, 0, "Move earlier"),
        (ScheduleEditKind::Move, 1, 0, "Move later"),
        (ScheduleEditKind::Move, 0, -1, "Move to previous resource"),
        (ScheduleEditKind::Move, 0, 1, "Move to next resource"),
        (ScheduleEditKind::ResizeStart, -1, 0, "Start earlier"),
        (ScheduleEditKind::ResizeStart, 1, 0, "Start later"),
        (ScheduleEditKind::ResizeEnd, -1, 0, "End earlier"),
        (ScheduleEditKind::ResizeEnd, 1, 0, "End later"),
    ] {
        let s = state.clone();
        let flags = state.flags;
        let enabled = scope.bond(move || {
            flags.with(|f| f.enabled && !f.readonly)
                && target.with(|v| {
                    v.as_ref().is_some_and(|v| {
                        accepted.with(|m| {
                            m.data()
                                .events()
                                .iter()
                                .find(|e| e.key() == &v.event)
                                .is_some_and(|e| match kind {
                                    ScheduleEditKind::Move => e.edit_permissions().move_event,
                                    _ => e.edit_permissions().resize_event,
                                })
                        })
                    })
                })
        });
        let mut b = crate::button()
            .theme(theme.clone())
            .label(label)
            .class("sapid-resources-edit-command")
            .enabled(enabled)
            .on_press(move || s.report(s.keyboard_edit(kind, steps, resource)));
        if unstyled {
            b = b.unstyled()
        }
        b.show_in(&content)?;
    }
    scope.retain(content)
}
struct EditChoice {
    row: Cage<(EditTarget, String)>,
    root: Rc<ResourceController>,
    theme: crate::Theme,
    unstyled: bool,
}
fn build_all_day(
    scope: &Scope,
    state: Rc<ResourceController>,
    theme: &crate::Theme,
    unstyled: bool,
) -> Result {
    let data = state.accepted;
    let rows = scope.bond(move || {
        data.with(|m| {
            m.data()
                .expand(MAX_SCHEDULE_OCCURRENCES)
                .unwrap()
                .into_iter()
                .filter_map(|occ| {
                    let e = m.data().event(occ).unwrap();
                    (e.is_all_day()
                        && m.visible_resources()
                            .iter()
                            .any(|r| Some(r.key()) == e.resource_key()))
                    .then(|| AgendaRow {
                        key: e.key().clone(),
                        candidate: occ.candidate_index(),
                        interval: occ.interval(),
                        title: e.title().to_owned(),
                        resource: m
                            .resources()
                            .get(e.resource_key().unwrap())
                            .unwrap()
                            .title()
                            .to_owned(),
                        all_day: true,
                        selectable: e.edit_permissions().select,
                    })
                })
                .collect::<Vec<_>>()
        })
    });
    let parent = scope.cage(None);
    glory::ui::view()
        .document_part(DocumentPart::UnorderedList)
        .class("sapid-resources-all-day")
        .node_ref(parent)
        .show_in(scope)?;
    let content = child(scope, parent)?;
    let theme = theme.clone();
    glory::ui::each(
        rows,
        |row| (row.key.clone(), row.candidate),
        move |row, _| AllDayWidget {
            row,
            root: state.clone(),
            theme: theme.clone(),
            unstyled,
        },
    )
    .show_in(&content)?;
    scope.retain(content)
}
struct AllDayWidget {
    row: Cage<AgendaRow>,
    root: Rc<ResourceController>,
    theme: crate::Theme,
    unstyled: bool,
}
impl Widget for AllDayWidget {
    fn build(&mut self, scope: &Scope) -> Result {
        let parent = scope.cage(None);
        glory::ui::view()
            .document_part(DocumentPart::ListItem)
            .node_ref(parent)
            .show_in(scope)?;
        let content = child(scope, parent)?;
        let row = self.row;
        let root = self.root.clone();
        let flags = root.flags;
        let mut b = crate::button()
            .theme(self.theme.clone())
            .class("sapid-resources-all-day-event")
            .label(row.map(|r| {
                format!(
                    "{} · All day {} — {} · {}",
                    r.title,
                    r.interval.start().date(),
                    r.interval.end().date(),
                    r.resource
                )
            }))
            .enabled(scope.bond(move || {
                flags.with(|f| f.enabled && !f.readonly) && row.with(|r| r.selectable)
            }))
            .on_press(move || {
                root.report(root.event(&row.get()));
                root.activated.emit(root.owner, Event::Activate);
            });
        if self.unstyled {
            b = b.unstyled()
        }
        b.show_in(&content)?;
        scope.retain(content)
    }
}
impl Widget for EditChoice {
    fn build(&mut self, scope: &Scope) -> Result {
        let row = self.row;
        let root = self.root.clone();
        let flags = root.flags;
        let mut b = crate::button()
            .theme(self.theme.clone())
            .class("sapid-resources-edit-choice")
            .label(row.map(|v| {
                format!(
                    "Edit {} · {}",
                    v.1,
                    crate::format_local_date_time(v.0.interval.start())
                )
            }))
            .enabled(flags.map(|f| f.enabled && !f.readonly))
            .on_press(move || {
                root.report(root.open_edit(row.with(|v| v.0.clone())));
                root.activated.emit(root.owner, Event::Activate);
            });
        if self.unstyled {
            b = b.unstyled()
        }
        b.show_in(scope)
    }
}
