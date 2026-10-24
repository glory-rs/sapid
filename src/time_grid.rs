//! Bounded physical time-slot Grid/Button composition.
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback, Observers};
use crate::state::{Control, Dirty, Value};
use crate::{LocalTime, MAX_DISABLED_TIMES, TimeBounds, TimeFormat, TimeGridSpec};
use glory::render::{
    Direction, Event, EventKind, FocusMode, GridCommand, NodeId, Role, SemanticId, Semantics,
    TableMetadata,
};
use glory::style::{BorderStyle, Display, Edges, FlexDirection, Length, Position, Style};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Lotus, Scope};
use std::{cell::Cell, rc::Rc};
fn invalid() -> glory::ui::Error {
    glory::render::Error::InvalidProps("invalid finite time grid").into()
}
#[derive(Clone, Copy, PartialEq)]
struct Snapshot {
    value: Option<LocalTime>,
    bounds: TimeBounds,
    enabled: bool,
    readonly: bool,
}
struct Controller {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    live: Cell<bool>,
    accepted: Cage<Snapshot>,
    control: Control<Option<LocalTime>>,
    spec: TimeGridSpec,
    disabled: Vec<LocalTime>,
    references: Vec<Cage<Option<NodeId>>>,
    focused: Cage<Option<usize>>,
    physical: Cage<Option<NodeId>>,
    cursor: Cell<Option<(NodeId, usize)>>,
    intent: Cell<Option<usize>>,
    dirty: Dirty,
    changed: Callback<Option<LocalTime>>,
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
            && let Some(s) = self.weak.upgrade()
        {
            s.report_error(e)
        }
    }
    fn allowed(&self, s: Snapshot, at: usize) -> bool {
        s.enabled
            && self.spec.at(at).is_some_and(|v| {
                s.bounds.contains(v) && self.disabled.binary_search(&v).is_err()
            })
    }
    fn nearest(&self, s: Snapshot, at: usize) -> Option<usize> {
        for distance in 0..self.spec.len() {
            for positive in [true, false] {
                if distance == 0 && !positive {
                    continue;
                }
                let target = if positive {
                    at.checked_add(distance)
                } else {
                    at.checked_sub(distance)
                };
                if let Some(target) = target
                    && self.allowed(s, target)
                {
                    return Some(target);
                }
            }
        }
        None
    }
    fn navigate(&self, from: NodeId, command: GridCommand) -> Result {
        self.ensure()?;
        let s = self.accepted.try_get()?;
        let Some(origin) = self.references.iter().position(|r| r.get() == Some(from)) else {
            return Ok(());
        };
        if !self.allowed(s, origin) {
            return Ok(());
        }
        let at = self
            .cursor
            .get()
            .filter(|(id, at)| *id == from && self.allowed(s, *at))
            .map_or(origin, |(_, at)| at);
        let cols = self.spec.columns();
        let len = self.spec.len();
        let row = at / cols;
        let (start, delta, steps, lo, hi) = match command {
            GridCommand::Left => (at as i32 - 1, -1, len, 0, len - 1),
            GridCommand::Right => (at as i32 + 1, 1, len, 0, len - 1),
            GridCommand::Up => (
                at as i32 - cols as i32,
                -(cols as i32),
                self.spec.rows(),
                0,
                len - 1,
            ),
            GridCommand::Down => (
                at as i32 + cols as i32,
                cols as i32,
                self.spec.rows(),
                0,
                len - 1,
            ),
            GridCommand::FirstInRow => (
                (row * cols) as i32,
                1,
                cols,
                row * cols,
                ((row + 1) * cols - 1).min(len - 1),
            ),
            GridCommand::LastInRow => (
                ((row + 1) * cols - 1).min(len - 1) as i32,
                -1,
                cols,
                row * cols,
                ((row + 1) * cols - 1).min(len - 1),
            ),
            GridCommand::PreviousPage => {
                (at.saturating_sub(cols * 4) as i32, -1, len, 0, len - 1)
            }
            GridCommand::NextPage => ((at + cols * 4).min(len - 1) as i32, 1, len, 0, len - 1),
            GridCommand::PreviousLargePage => (0, 1, len, 0, len - 1),
            GridCommand::NextLargePage => ((len - 1) as i32, -1, len, 0, len - 1),
        };
        for n in 0..steps {
            let candidate = start + n as i32 * delta;
            if candidate < lo as i32 || candidate > hi as i32 {
                break;
            }
            let candidate = candidate as usize;
            if self.allowed(s, candidate) {
                self.cursor.set(Some((from, candidate)));
                self.intent.set(Some(candidate));
                self.focused.try_revise(|v| *v = Some(candidate))?;
                return self.dirty.touch();
            }
        }
        Ok(())
    }
    fn select(&self, index: usize) -> Result {
        self.ensure()?;
        let s = self.accepted.try_get()?;
        if !self.allowed(s, index) || s.readonly {
            return Ok(());
        }
        let index = self
            .cursor
            .get()
            .filter(|(origin, target)| {
                self.references[index].get() == Some(*origin) && self.allowed(s, *target)
            })
            .map_or(index, |(_, target)| target);
        let value = self.spec.at(index);
        self.control.accept(&value)?;
        self.changed.emit(self.owner, value);
        self.ensure()?;
        self.focused.try_revise(|v| *v = Some(index))?;
        self.dirty.touch()
    }
    fn retire(&self) {
        self.live.set(false);
        self.cursor.set(None);
        self.intent.set(None);
        self.changed.clear();
    }
}
pub struct TimeGrid {
    common: Common,
    key: String,
    spec: TimeGridSpec,
    format: TimeFormat,
    bounds: Lotus<TimeBounds>,
    value: Value<Option<LocalTime>>,
    readonly: Lotus<bool>,
    direction: Direction,
    disabled: Vec<LocalTime>,
    changed: Option<Box<dyn FnMut(Option<LocalTime>)>>,
}
pub fn time_grid(key: impl Into<String>) -> TimeGrid {
    TimeGrid {
        common: Common::default(),
        key: key.into(),
        spec: TimeGridSpec::default(),
        format: TimeFormat::default(),
        bounds: TimeBounds::default().into(),
        value: Value::default(),
        readonly: false.into(),
        direction: Direction::Ltr,
        disabled: vec![],
        changed: None,
    }
}
impl TimeGrid {
    common_methods!();
    pub fn spec(mut self, v: TimeGridSpec) -> Self {
        self.spec = v;
        self
    }
    pub fn format(mut self, v: TimeFormat) -> Self {
        self.format = v;
        self
    }
    pub fn bounds(mut self, v: impl Into<Lotus<TimeBounds>>) -> Self {
        self.bounds = v.into();
        self
    }
    pub fn value(mut self, v: impl Into<Lotus<Option<LocalTime>>>) -> Self {
        self.value.source = Some(v.into());
        self
    }
    pub fn default_value(mut self, v: Option<LocalTime>) -> Self {
        self.value.default = v;
        self
    }
    pub fn readonly(mut self, v: impl Into<Lotus<bool>>) -> Self {
        self.readonly = v.into();
        self
    }
    pub fn direction(mut self, v: Direction) -> Self {
        self.direction = v;
        self
    }
    pub fn disable_time(mut self, v: LocalTime) -> Self {
        if self.disabled.len() <= MAX_DISABLED_TIMES {
            self.disabled.push(v);
        }
        self
    }
    pub fn disable_times(mut self, v: impl IntoIterator<Item = LocalTime>) -> Self {
        self.disabled = v.into_iter().take(MAX_DISABLED_TIMES + 1).collect();
        self
    }
    pub fn on_value_change(mut self, v: impl FnMut(Option<LocalTime>) + 'static) -> Self {
        self.changed = Some(Box::new(v));
        self
    }
}
fn child(scope: &Scope, id: Cage<Option<NodeId>>) -> Result<Scope> {
    scope.child_in(id.get().ok_or(glory::ui::Error::Gone)?)
}
impl Widget for TimeGrid {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.key.is_empty()
            || self.key.len() > 64
            || !self
                .key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || self.disabled.len() > MAX_DISABLED_TIMES
        {
            return Err(invalid());
        }
        let spec = self.spec;
        let format = self.format;
        for at in 0..spec.len() {
            format
                .validate(spec.at(at).unwrap())
                .map_err(|_| invalid())?;
        }
        let control = std::mem::take(&mut self.value).mount(scope);
        let source = control.source.clone();
        let bounds = self.bounds.clone();
        let enabled = self.common.enabled.clone();
        let readonly = self.readonly.clone();
        let snapshot = scope.bond(move || Snapshot {
            value: source.get(),
            bounds: bounds.get(),
            enabled: enabled.get(),
            readonly: readonly.get(),
        });
        let accepted = composite::validated(scope, &snapshot.into(), move |s: &Snapshot| {
            if s.value
                .is_none_or(|v| s.bounds.contains(v) && format.validate(v).is_ok())
            {
                Ok(())
            } else {
                Err(invalid())
            }
        })?;
        let mut disabled = std::mem::take(&mut self.disabled);
        disabled.sort_unstable();
        disabled.dedup();
        let focused = scope.cage(None);
        let state = Rc::new(Controller {
            owner: scope.id(),
            weak: scope.downgrade(),
            live: Cell::new(true),
            accepted,
            control,
            spec,
            disabled,
            references: (0..spec.len()).map(|_| scope.cage(None)).collect(),
            focused,
            physical: scope.cage(None),
            cursor: Cell::new(None),
            intent: Cell::new(None),
            dirty: Dirty::new(scope),
            changed: Callback::new(self.changed.take()),
        });
        let initial = accepted.get();
        let initial_index = initial.value.map(|v| spec.nearest_index(v)).unwrap_or(0);
        focused.revise(|v| *v = state.nearest(initial, initial_index));
        let cleanup = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(s) = cleanup.upgrade() {
                s.retire()
            }
        })?;
        let cols = spec.columns();
        let height = match self.common.theme.control(self.common.size).min_height {
            Some(Length::Px(v)) => v,
            _ => 36.,
        };
        let width = if format.cycle == crate::HourCycle::H12 {
            height.max(32.) * 3.5
        } else {
            height.max(28.) * 2.5
        };
        let gap = 2.;
        let total = cols as f64 * (width + gap) - gap;
        let base = Style::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(gap)
            .width(Length::Px(total));
        let (classes, appearance) = self.common.appearance(scope, "sapid-time-grid", base);
        let style = scope.bond(move || {
            let mut s = appearance.get();
            s.display.get_or_insert(Display::Flex);
            s.flex_direction.get_or_insert(FlexDirection::Column);
            s.gap.get_or_insert(gap);
            s.width.get_or_insert(Length::Px(total));
            s
        });
        let root = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let id = SemanticId::new(format!("{}-grid", self.key))?;
        let direction = self.direction;
        let grid_events = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::GridNavigate)
                .unwrap_or_default(),
        );
        let activate = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Activate)
                .unwrap_or_default(),
        );
        let cleanup_events = (grid_events.clone(), activate.clone());
        scope.on_cleanup(move || {
            cleanup_events.0.clear();
            cleanup_events.1.clear();
        })?;
        let s = state.clone();
        let mut root_widget = glory::ui::view()
            .role(Role::Grid)
            .class(classes)
            .style(style)
            .node_ref(root)
            .focus(self.common.focus)
            .name(
                self.common
                    .name
                    .take()
                    .unwrap_or_else(|| "Choose a time".into()),
            )
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    id: Some(id.clone()),
                    grid_navigation: Some(direction),
                    readonly: accepted.with(|s| s.readonly),
                    table: Some(TableMetadata::Table {
                        rows: Some(spec.rows() as u32),
                        columns: Some(cols as u16),
                    }),
                    ..Default::default()
                }))
            }))
            .on(EventKind::GridNavigate, move |e| {
                if s.ensure().is_ok() {
                    if let Event::GridNavigate { from, command } = &e {
                        s.report(s.navigate(*from, *command))
                    }
                    grid_events.emit(s.owner, e)
                }
            });
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root_widget = root_widget.on(kind, move |e| {
                crate::appearance::observe(&mut handlers, owner, e)
            });
        }
        root_widget.show_in(scope)?;
        let content = child(scope, root)?;
        for row in 0..spec.rows() {
            let rr = scope.cage(None);
            glory::ui::view()
                .role(Role::Row)
                .style(
                    Style::new()
                        .position(Position::Relative)
                        .width(Length::Px(total))
                        .height(Length::Px(height)),
                )
                .node_ref(rr)
                .show_in(&content)?;
            let rs = child(scope, rr)?;
            for col in 0..cols {
                let at = row * cols + col;
                let value = spec.at(at);
                let cr = scope.cage(None);
                let physical = if direction == Direction::Ltr {
                    col
                } else {
                    cols - 1 - col
                };
                glory::ui::view()
                    .role(Role::GridCell)
                    .class("sapid-time-cell")
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
                                value.is_some() && accepted.with(|s| s.value == value),
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
                    .node_ref(cr)
                    .show_in(&rs)?;
                let cs = child(scope, cr)?;
                if let Some(value) = value {
                    let enabled = state.clone();
                    let styles = state.clone();
                    let selected = state.clone();
                    let fs = state.clone();
                    let theme = self.common.theme.clone();
                    let themed = !self.common.unstyled;
                    let reference = state.references[at];
                    let physical = state.physical;
                    let observers = activate.clone();
                    glory::ui::button()
                        .class("sapid-time-slot")
                        .label(format.format(value).unwrap())
                        .name(format.format(value).unwrap())
                        .node_ref(reference)
                        .enabled(scope.bond(move || accepted.with(|s| enabled.allowed(*s, at))))
                        .focus(scope.bond(move || {
                            if focused.get() == Some(at) {
                                FocusMode::TabStop
                            } else {
                                FocusMode::Programmatic
                            }
                        }))
                        .style(scope.bond(move || {
                            let snap = accepted.get();
                            let mut style = Style::new()
                                .width(Length::Px(width))
                                .height(Length::Px(height))
                                .padding(Edges::all(0.));
                            if themed {
                                let chosen = snap.value == Some(value);
                                style.background =
                                    Some(if chosen { theme.accent } else { theme.surface });
                                style.color = Some(if !styles.allowed(snap, at) {
                                    theme.disabled_text
                                } else if chosen {
                                    theme.on_accent
                                } else {
                                    theme.foreground
                                });
                                style.font_size = Some(14.);
                                style = style
                                    .border_width(Edges::all(1.))
                                    .border_style(BorderStyle::Solid)
                                    .border_color(
                                        if reference
                                            .get()
                                            .is_some_and(|id| physical.get() == Some(id))
                                        {
                                            theme.accent
                                        } else {
                                            theme.border
                                        },
                                    )
                                    .radius(theme.radius);
                            }
                            style
                        }))
                        .on(EventKind::Activate, move |e| {
                            if selected.ensure().is_ok() {
                                selected.report(selected.select(at));
                                observers.emit(selected.owner, e)
                            }
                        })
                        .on(EventKind::FocusChanged, move |e| {
                            if fs.ensure().is_ok()
                                && let Event::FocusChanged(active) = e
                            {
                                let result = (|| -> Result {
                                    if active {
                                        fs.cursor.set(None);
                                        focused.try_revise(|v| *v = Some(at))?;
                                        physical.try_revise(|v| *v = reference.get())?;
                                    } else if reference
                                        .get()
                                        .is_some_and(|id| physical.get() == Some(id))
                                    {
                                        physical.try_revise(|v| *v = None)?;
                                    }
                                    Ok(())
                                })();
                                fs.report(result)
                            }
                        })
                        .show_in(&cs)?;
                }
                scope.retain(cs)?;
            }
            scope.retain(rs)?;
        }
        let weak = Rc::downgrade(&state);
        scope.effect(move || {
            let Some(s) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let edited = s.dirty.take();
                let snap = accepted.get();
                if focused.get().is_none_or(|at| !s.allowed(snap, at)) {
                    let next =
                        s.nearest(snap, snap.value.map(|v| spec.nearest_index(v)).unwrap_or(0));
                    if next != focused.get() {
                        focused.try_revise(|v| *v = next)?;
                    }
                }
                if edited {
                    s.cursor.set(None);
                }
                if let Some(at) = s.intent.take()
                    && let Some(scope) = s.weak.upgrade()
                {
                    composite::focus_node(&scope, s.references[at].get())?;
                }
                Ok(())
            })();
            s.report(result)
        });
        scope.retain(content)?;
        scope.retain(state)
    }
}
