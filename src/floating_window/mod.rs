//! A real nonmodal viewport Dialog with controlled geometry and host top layers.
use crate::appearance::{Common, common_methods, observe};
use crate::composite::{self, Callback, ContentPolicy, Factory};
use crate::state::{Control, Value};
use glory::render::{
    Action, DismissReason, DragCapture, DragPhase, Event, EventKind, FocusMode, Layer,
    LayerPlacement, NodeId, Query, QueryToken, QueryValue, Role, Semantics, TextKind,
    ViewportPlacement,
};
use glory::style::{Edges, Length, Limit, Overflow, Style};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Cage, Lotus, Scope};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

mod model;
pub use model::{
    MAX_WINDOW_COORDINATE, MAX_WINDOW_SIZE, WindowBounds, WindowCommand, WindowLimits,
};

fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn text(value: &str) -> Result {
    if value.trim().is_empty() || value.len() > 256 || value.contains(['\0', '\r', '\n']) {
        return Err(invalid(
            "window labels require 1..=256 canonical single-line bytes",
        ));
    }
    Ok(())
}
fn decoration(style: &Style) -> Result {
    if style.position.is_some()
        || style.top.is_some()
        || style.left.is_some()
        || style.bottom.is_some()
        || style.right.is_some()
        || style.width.is_some()
        || style.height.is_some()
        || style.min_width.is_some()
        || style.min_height.is_some()
        || style.max_width.is_some()
        || style.max_height.is_some()
        || style.aspect_ratio.is_some()
    {
        return Err(invalid(
            "window bounds and limits own all positioning and dimensions",
        ));
    }
    Ok(())
}

struct State {
    owner: glory::core::InstanceId,
    weak: WeakScope,
    bounds: Control<WindowBounds>,
    input_bounds: Lotus<WindowBounds>,
    open: Control<bool>,
    enabled: Lotus<bool>,
    limits: WindowLimits,
    step: f64,
    reference: Cage<Option<NodeId>>,
    changed: Callback<WindowBounds>,
    open_changed: Callback<bool>,
    dismissed: Callback<DismissReason>,
}
impl State {
    fn current(&self) -> Result<WindowBounds> {
        let value = self.input_bounds.try_get()?;
        self.limits.validate_bounds(value)?;
        Ok(value)
    }
    fn ready(&self) -> Result<bool> {
        Ok(self.owner.is_alive() && self.open.source.try_get()? && self.enabled.try_get()?)
    }
    fn propose(&self, next: WindowBounds) -> Result {
        if !self.ready()? {
            return Ok(());
        }
        self.limits.validate_bounds(next)?;
        if self.current()? != next {
            self.bounds.accept(&next)?;
            self.changed.emit(self.owner, next);
        }
        Ok(())
    }
    fn command(&self, command: WindowCommand) -> Result {
        if !self.ready()? {
            return Ok(());
        }
        let (dx, dy, resize) = command.delta(self.step);
        self.propose(self.limits.change(self.current()?, dx, dy, resize)?)
    }
    fn close(&self, reason: Option<DismissReason>) -> Result {
        if !self.ready()? {
            return Ok(());
        }
        self.open.accept(&false)?;
        if let Some(reason) = reason {
            self.dismissed.emit(self.owner, reason)
        }
        if self.owner.is_alive() {
            self.open_changed.emit(self.owner, false)
        }
        Ok(())
    }
    fn raise(&self) -> Result {
        if !self.ready()? {
            return Ok(());
        }
        let scope = self
            .weak
            .upgrade()
            .ok_or_else(|| invalid("window Owner retired"))?;
        let node = self
            .reference
            .try_get()?
            .ok_or_else(|| invalid("window is not mounted"))?;
        scope.action(Action::RaiseLayer(node))
    }
    fn report(&self, result: Result) {
        if let Err(error) = result
            && let Some(scope) = self.weak.upgrade()
        {
            scope.report_error(error)
        }
    }
}

/// A current Owner-bound command handle. Controlled values are proposals; the
/// handle never writes an application's source and cannot reopen a retired Owner.
#[derive(Clone)]
pub struct FloatingWindowHandle(Weak<State>);
impl FloatingWindowHandle {
    fn state(&self) -> Result<Rc<State>> {
        self.0
            .upgrade()
            .filter(|s| s.owner.is_alive())
            .ok_or_else(|| invalid("window Owner retired"))
    }
    pub fn command(&self, command: WindowCommand) -> Result {
        self.state()?.command(command)
    }
    pub fn set_bounds(&self, bounds: WindowBounds) -> Result {
        self.state()?.propose(bounds)
    }
    pub fn raise(&self) -> Result {
        self.state()?.raise()
    }
    pub fn close(&self) -> Result {
        self.state()?.close(None)
    }
}

#[derive(Clone, Copy)]
struct Work {
    ticket: u64,
    from: NodeId,
    resize: bool,
    start: (f64, f64),
    latest: (f64, f64),
    phase: DragPhase,
    accepted: WindowBounds,
    baseline: Option<WindowBounds>,
    proposed: Option<WindowBounds>,
}
/// One current drag and at most one initial Bounds query. No pointer history.
struct Pad {
    state: Weak<State>,
    weak: WeakScope,
    work: RefCell<Option<Work>>,
    pending: Cell<Option<QueryToken>>,
    next: Cell<u64>,
}
impl Pad {
    fn new(scope: &Scope, state: &Rc<State>) -> Result<Rc<Self>> {
        let pad = Rc::new(Self {
            state: Rc::downgrade(state),
            weak: scope.downgrade(),
            work: RefCell::new(None),
            pending: Cell::new(None),
            next: Cell::new(0),
        });
        let weak = Rc::downgrade(&pad);
        scope.on_cleanup(move || {
            if let Some(pad) = weak.upgrade() {
                pad.cancel()
            }
        })?;
        Ok(pad)
    }
    fn cancel(&self) {
        self.work.borrow_mut().take();
        if let Some(token) = self.pending.take()
            && let Some(scope) = self.weak.upgrade()
            && let Err(error) = scope.cancel_request(token)
        {
            scope.report_error(error)
        }
    }
    fn apply(&self) -> Result {
        let Some(state) = self.state.upgrade() else {
            self.cancel();
            return Ok(());
        };
        if !state.ready()? {
            self.cancel();
            return Ok(());
        }
        let Some(work) = *self.work.borrow() else {
            return Ok(());
        };
        let Some(baseline) = work.baseline else {
            return Ok(());
        };
        let current = match state.current() {
            Ok(value) => value,
            Err(error) => {
                self.cancel();
                return Err(error);
            }
        };
        if current != work.accepted && Some(current) != work.proposed {
            self.cancel();
            return Ok(());
        }
        let dx = work.latest.0 - work.start.0;
        let dy = work.latest.1 - work.start.1;
        if dx != 0. || dy != 0. {
            let next = state.limits.change(baseline, dx, dy, work.resize)?;
            if let Some(work) = self.work.borrow_mut().as_mut() {
                work.proposed = Some(next)
            }
            if let Err(error) = state.propose(next) {
                self.cancel();
                return Err(error);
            }
            if !state.owner.is_alive() {
                self.cancel();
                return Ok(());
            }
            let current = state.current()?;
            if current != work.accepted && current != next {
                self.cancel();
                return Ok(());
            }
            if current == next
                && let Some(work) = self.work.borrow_mut().as_mut()
            {
                work.accepted = current
            }
        }
        if work.phase == DragPhase::Drop {
            self.work.borrow_mut().take();
        }
        Ok(())
    }
    fn event(self: &Rc<Self>, resize: bool, event: Event) -> Result {
        let Event::Drag { from, phase, x, y } = event else {
            return Ok(());
        };
        let Some(state) = self.state.upgrade() else {
            self.cancel();
            return Ok(());
        };
        if phase == DragPhase::Cancel {
            if self.work.borrow().is_some_and(|work| work.from == from) {
                self.cancel()
            }
            return Ok(());
        }
        if !state.ready()? {
            self.cancel();
            return Ok(());
        }
        if phase == DragPhase::Start {
            self.cancel();
            state.raise()?;
            let ticket = self
                .next
                .get()
                .checked_add(1)
                .ok_or_else(|| invalid("window drag ticket exhausted"))?;
            self.next.set(ticket);
            *self.work.borrow_mut() = Some(Work {
                ticket,
                from,
                resize,
                start: (x, y),
                latest: (x, y),
                phase,
                accepted: state.current()?,
                baseline: None,
                proposed: None,
            });
            let node = state
                .reference
                .try_get()?
                .ok_or_else(|| invalid("window panel is not mounted"))?;
            let weak = Rc::downgrade(self);
            let token = self
                .weak
                .upgrade()
                .ok_or_else(|| invalid("window Owner retired"))?
                .request(Query::Bounds(node), move |result| {
                    let Some(pad) = weak.upgrade() else { return };
                    pad.pending.set(None);
                    if !pad.work.borrow().is_some_and(|work| work.ticket == ticket) {
                        return;
                    }
                    let Ok(QueryValue::Bounds(rect)) = result else {
                        pad.cancel();
                        return;
                    };
                    let baseline = WindowBounds {
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: rect.height,
                    };
                    let Some(state) = pad.state.upgrade() else {
                        pad.cancel();
                        return;
                    };
                    if let Err(error) = state.limits.validate_bounds(baseline) {
                        state.report(Err(error));
                        pad.cancel();
                        return;
                    }
                    if let Some(work) = pad.work.borrow_mut().as_mut() {
                        work.baseline = Some(baseline)
                    }
                    state.report(pad.apply());
                })?;
            self.pending.set(Some(token));
        } else {
            let mut work = self.work.borrow_mut();
            let Some(work) = work.as_mut() else {
                return Ok(());
            };
            if work.from != from {
                return Ok(());
            }
            work.latest = (x, y);
            work.phase = phase;
        }
        self.apply()
    }
}

pub struct FloatingWindow {
    common: Common,
    key: String,
    title: Lotus<String>,
    bounds: Value<WindowBounds>,
    open: Value<bool>,
    limits: WindowLimits,
    step: f64,
    padding: f64,
    constrain: bool,
    changed: Option<Box<dyn FnMut(WindowBounds)>>,
    open_changed: Option<Box<dyn FnMut(bool)>>,
    dismissed: Option<Box<dyn FnMut(DismissReason)>>,
    ready: Option<Box<dyn FnMut(FloatingWindowHandle)>>,
    content: Option<Factory>,
    policy: ContentPolicy,
}
pub fn floating_window(key: impl Into<String>) -> FloatingWindow {
    FloatingWindow {
        common: Common::default(),
        key: key.into(),
        title: "Window".into(),
        bounds: Value::default(),
        open: Value {
            source: None,
            default: true,
        },
        limits: WindowLimits::default(),
        step: 10.,
        padding: 8.,
        constrain: true,
        changed: None,
        open_changed: None,
        dismissed: None,
        ready: None,
        content: None,
        policy: ContentPolicy::KeepMounted,
    }
}
impl FloatingWindow {
    common_methods!();
    pub fn title(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.title = value.into();
        self
    }
    pub fn bounds(mut self, value: impl Into<Lotus<WindowBounds>>) -> Self {
        self.bounds.source = Some(value.into());
        self
    }
    pub fn default_bounds(mut self, value: WindowBounds) -> Self {
        self.bounds.default = value;
        self
    }
    pub fn open(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.open.source = Some(value.into());
        self
    }
    pub fn default_open(mut self, value: bool) -> Self {
        self.open.default = value;
        self
    }
    pub fn limits(mut self, value: WindowLimits) -> Self {
        self.limits = value;
        self
    }
    pub fn keyboard_step(mut self, value: f64) -> Self {
        self.step = value;
        self
    }
    pub fn viewport_padding(mut self, value: f64) -> Self {
        self.padding = value;
        self
    }
    pub fn constrain_to_viewport(mut self, value: bool) -> Self {
        self.constrain = value;
        self
    }
    pub fn content_policy(mut self, value: ContentPolicy) -> Self {
        self.policy = value;
        self
    }
    pub fn content<W: Widget>(mut self, mut factory: impl FnMut() -> W + 'static) -> Self {
        self.content = Some(Box::new(move || Box::new(factory())));
        self
    }
    pub fn on_bounds_change(mut self, callback: impl FnMut(WindowBounds) + 'static) -> Self {
        self.changed = Some(Box::new(callback));
        self
    }
    pub fn on_open_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.open_changed = Some(Box::new(callback));
        self
    }
    pub fn on_dismiss(mut self, callback: impl FnMut(DismissReason) + 'static) -> Self {
        self.dismissed = Some(Box::new(callback));
        self
    }
    pub fn on_ready(mut self, callback: impl FnMut(FloatingWindowHandle) + 'static) -> Self {
        self.ready = Some(Box::new(callback));
        self
    }
}
impl Widget for FloatingWindow {
    fn build(&mut self, scope: &Scope) -> Result {
        self.limits.validate()?;
        if !self.step.is_finite() || !(1. ..=4096.).contains(&self.step) {
            return Err(invalid(
                "window keyboard step must be within 1..=4096 CSS pixels",
            ));
        }
        ViewportPlacement {
            x: 0.,
            y: 0.,
            viewport_padding: self.padding,
            constrain: self.constrain,
        }
        .validate()?;
        let id = composite::id("window", &self.key, "panel", "root")?;
        let title_id = composite::id("window", &self.key, "title", "root")?;
        let title = composite::validated(scope, &self.title, |value| text(value))?;
        if let Some(name) = &self.common.name {
            self.common.name =
                Some(composite::validated(scope, name, |value| text(value))?.into())
        }
        self.common.style = composite::validated(scope, &self.common.style, decoration)?.into();
        let mut bounds = std::mem::take(&mut self.bounds).mount(scope);
        let input_bounds = bounds.source.clone();
        let limits = self.limits;
        bounds.source = composite::validated(scope, &bounds.source, move |value| {
            limits.validate_bounds(*value)
        })?
        .into();
        let open = std::mem::take(&mut self.open).mount(scope);
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let state = Rc::new(State {
            owner: scope.id(),
            weak: scope.downgrade(),
            bounds,
            input_bounds,
            open,
            enabled: self.common.enabled.clone(),
            limits,
            step: self.step,
            reference,
            changed: Callback::new(self.changed.take()),
            open_changed: Callback::new(self.open_changed.take()),
            dismissed: Callback::new(self.dismissed.take()),
        });
        let moving = Pad::new(scope, &state)?;
        let sizing = moving.clone();
        let source = state.bounds.source.clone();
        let accepted_open = state.open.source.clone();
        let padding = self.padding;
        let constrain = self.constrain;
        let labelled = self.common.name.is_none().then(|| title_id.clone());
        let mut panel = glory::ui::view()
            .role(Role::Dialog)
            .focus(if self.common.focus == FocusMode::Auto {
                FocusMode::Programmatic
            } else {
                self.common.focus
            })
            .node_ref(reference)
            .semantics(scope.bond(move || {
                let bounds = source.get();
                composite::metadata(Semantics {
                    id: Some(id.clone()),
                    labelled_by: labelled.clone(),
                    hidden: !accepted_open.get(),
                    layer: Some(Layer {
                        modal: false,
                        outside: false,
                        placement: LayerPlacement::Viewport(ViewportPlacement {
                            x: bounds.x,
                            y: bounds.y,
                            viewport_padding: padding,
                            constrain,
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            }));
        let mut base = Style::new()
            .padding(Edges::all(12.))
            .radius(self.common.theme.radius)
            .overflow_y(Overflow::Auto);
        base.background = Some(self.common.theme.surface);
        base.color = Some(self.common.theme.foreground);
        let (classes, decorations) =
            self.common
                .appearance(scope, "sapid-floating-window sapid-layer", base);
        let source = state.bounds.source.clone();
        panel = panel.class(classes).style(scope.bond(move || {
            let bounds = source.get();
            let mut style = decorations.get();
            style.width = Some(Length::Px(bounds.width));
            style.height = Some(Length::Px(bounds.height));
            style.min_width = Some(Length::Px(limits.min_width));
            style.min_height = Some(Length::Px(limits.min_height));
            style.max_width = Some(Limit::Px(limits.max_width));
            style.max_height = Some(Limit::Px(limits.max_height));
            style
        }));
        if let Some(name) = self.common.name.clone() {
            panel = panel.name(name)
        }
        let s = state.clone();
        panel = panel.on(EventKind::Dismiss, move |event| {
            if let Event::Dismiss(reason) = event {
                s.report(s.close(Some(reason)))
            }
        });
        let s = state.clone();
        panel = panel.on(EventKind::FocusWithin, move |event| {
            if matches!(event, Event::FocusWithin { focused: true, .. }) {
                s.report(s.raise())
            }
        });
        let owner = scope.id();
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            panel = panel.on(kind, move |event| observe(&mut handlers, owner, event))
        }
        let theme = self.common.theme.clone();
        let size = self.common.size;
        let unstyled = self.common.unstyled;
        let policy = self.policy;
        let mut content = self.content.take();
        let content_open = state.open.source.clone();
        let body_state = state.clone();
        let enabled = state.enabled.clone();
        panel = panel.fill(
            glory::ui::view()
                .field_part(glory::render::FieldPart::Fieldset)
                .class("sapid-window-controls")
                .style(
                    Style::new()
                        .padding(Edges::all(0.))
                        .margin(Edges::all(0.))
                        .border_width(Edges::all(0.))
                        .min_width(Length::Px(0.)),
                )
                .semantics(scope.bond(move || {
                    composite::metadata(Semantics {
                        field_part: Some(glory::render::FieldPart::Fieldset),
                        disabled: !enabled.get(),
                        ..Default::default()
                    })
                }))
                .fill(glory::ui::widget(move |scope| {
                    let pad = moving.clone();
                    let s = body_state.clone();
                    glory::ui::view()
                        .class("sapid-window-caption")
                        .semantics(composite::metadata(Semantics {
                            drag_capture: Some(DragCapture::Press),
                            ..Default::default()
                        }))
                        .on(EventKind::Drag, move |event| {
                            s.report(pad.event(false, event))
                        })
                        .fill(
                            glory::ui::text()
                                .kind(TextKind::Heading(2))
                                .content(title)
                                .semantics(composite::metadata(Semantics {
                                    id: Some(title_id.clone()),
                                    ..Default::default()
                                })),
                        )
                        .show_in(scope)?;
                    let s = body_state.clone();
                    let mut close = glory::ui::button()
                        .label("Close window")
                        .class("sapid-window-close")
                        .enabled(s.enabled.clone())
                        .on(EventKind::Activate, move |_| s.report(s.close(None)));
                    if !unstyled {
                        close = close.style(theme.control(size));
                    }
                    close.show_in(scope)?;
                    let controls = body_state.clone();
                    let theme = theme.clone();
                    glory::ui::view()
                        .role(Role::Group)
                        .name("Window movement and size")
                        .fill(glory::ui::widget(move |scope| {
                            for (command, label) in [
                                (WindowCommand::MoveLeft, "Move window left"),
                                (WindowCommand::MoveRight, "Move window right"),
                                (WindowCommand::MoveUp, "Move window up"),
                                (WindowCommand::MoveDown, "Move window down"),
                                (WindowCommand::Narrower, "Make window narrower"),
                                (WindowCommand::Wider, "Make window wider"),
                                (WindowCommand::Shorter, "Make window shorter"),
                                (WindowCommand::Taller, "Make window taller"),
                            ] {
                                let s = controls.clone();
                                let mut button = glory::ui::button()
                                    .label(label)
                                    .enabled(s.enabled.clone())
                                    .on(EventKind::Activate, move |_| {
                                        s.report(s.command(command))
                                    });
                                if !unstyled {
                                    button = button.style(theme.control(size));
                                }
                                button.show_in(scope)?;
                            }
                            Ok(())
                        }))
                        .show_in(scope)?;
                    if let Some(mut factory) = content.take() {
                        match policy {
                            ContentPolicy::KeepMounted => factory().show_in(scope)?,
                            ContentPolicy::Unmount => {
                                glory::ui::when(content_open.clone(), factory).show_in(scope)?
                            }
                        }
                    }
                    let pad = sizing.clone();
                    let s = body_state.clone();
                    glory::ui::view()
                        .class("sapid-window-resize-grip")
                        .name("Pointer resize area")
                        .style(
                            Style::new()
                                .position(glory::style::Position::Absolute)
                                .right(Length::Px(8.))
                                .bottom(Length::Px(8.))
                                .width(Length::Px(24.))
                                .height(Length::Px(24.)),
                        )
                        .semantics(composite::metadata(Semantics {
                            drag_capture: Some(DragCapture::Press),
                            ..Default::default()
                        }))
                        .on(EventKind::Drag, move |event| {
                            s.report(pad.event(true, event))
                        })
                        .fill(glory::ui::text().content("↘"))
                        .show_in(scope)?;
                    Ok(())
                })),
        );
        panel.show_in(scope)?;
        if let Some(callback) = self.ready.take() {
            Callback::new(Some(callback))
                .emit(scope.id(), FloatingWindowHandle(Rc::downgrade(&state)))
        }
        scope.retain(state)?;
        Ok(())
    }
}
