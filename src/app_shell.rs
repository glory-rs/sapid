//! Explicit container-responsive application structure and one live sidebar tree.
use crate::appearance::{Common, configure, merge, observe};
use crate::composite::{self, Callback, Observers};
use crate::state::{Control, Value};
use glory::render::{
    Action, Change, Event, EventKind, FocusMode, Layer, LayerPlacement, NodeId, Patch,
    PopupKind, Role, SemanticId, Semantics,
};
use glory::style::{Display, Edges, FlexDirection, Length, Overflow, Style};
use glory::ui::{Result, WeakScope, Widget};
use glory::{Bond, Cage, Lotus, Scope};
use std::{cell::Cell, rc::Rc};

const MAX_SHELL_WIDTH: f64 = 1_000_000.0;
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn valid_width(value: f64) -> Result {
    if value.is_finite() && (0.0..=MAX_SHELL_WIDTH).contains(&value) {
        Ok(())
    } else {
        Err(invalid("shell widths must be finite and in 0..=1000000"))
    }
}

/// Local shared container measurement, not a global browser breakpoint service.
#[derive(Clone)]
pub struct ShellLayout {
    width: Cage<f64>,
    width_view: Bond<f64>,
    mobile: Bond<bool>,
    breakpoint: f64,
}
impl ShellLayout {
    pub fn new(scope: &Scope, initial_width: f64, breakpoint: f64) -> Result<Self> {
        valid_width(initial_width)?;
        valid_width(breakpoint)?;
        if breakpoint == 0.0 {
            return Err(invalid("shell breakpoint must be positive"));
        }
        let width = scope.cage(initial_width);
        Ok(Self {
            width,
            width_view: scope.bond(move || width.get()),
            mobile: scope.bond(move || width.get() < breakpoint),
            breakpoint,
        })
    }
    pub fn width(&self) -> Bond<f64> {
        self.width_view
    }
    pub fn mobile(&self) -> Bond<bool> {
        self.mobile
    }
    pub fn breakpoint(&self) -> f64 {
        self.breakpoint
    }
    fn validate_scope(&self, scope: &Scope) -> Result {
        if self.width.runtime() != scope.id().runtime() {
            return Err(glory::render::Error::ForeignRuntime.into());
        }
        self.width.try_version()?;
        Ok(())
    }
    /// Called from the normal queued Resize turn, or an explicit outer host driver.
    pub fn observe_width(&self, width: f64) -> Result<bool> {
        valid_width(width)?;
        if self.width.try_get()? != width {
            self.width.try_revise(|value| *value = width)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellMode {
    Desktop,
    Mobile,
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct Presentation {
    mode: ShellMode,
    collapsed: bool,
    open: bool,
}

struct Controller {
    weak: WeakScope,
    owner: glory::core::InstanceId,
    layout: ShellLayout,
    collapsed: Control<bool>,
    open: Control<bool>,
    accepted: Cage<Presentation>,
    collapsed_change: Callback<bool>,
    open_change: Callback<bool>,
    activate: Observers,
    dismiss: Observers,
    nav_id: SemanticId,
    dialog_id: SemanticId,
    trigger_id: SemanticId,
    close_id: SemanticId,
    nav: Cage<Option<NodeId>>,
    dock: Cage<Option<NodeId>>,
    dialog: Cage<Option<NodeId>>,
    trigger: Cage<Option<NodeId>>,
    nav_focus: Cell<Option<NodeId>>,
    live: Cell<bool>,
}
impl Controller {
    fn ensure(&self) -> Result {
        if self.live.get() && self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn next(&self) -> Result<Presentation> {
        Ok(Presentation {
            mode: if self.layout.mobile.try_get()? {
                ShellMode::Mobile
            } else {
                ShellMode::Desktop
            },
            collapsed: self.collapsed.source.try_get()?,
            open: self.open.source.try_get()?,
        })
    }
    fn metadata(&self, state: Presentation) -> Option<Box<Semantics>> {
        composite::metadata(Semantics {
            id: Some(self.dialog_id.clone()),
            hidden: state.mode == ShellMode::Desktop || !state.open,
            layer: (state.mode == ShellMode::Mobile).then(|| Layer {
                modal: true,
                escape: true,
                outside: true,
                placement: LayerPlacement::Left,
                initial_focus: Some(self.close_id.clone()),
                return_focus: Some(self.trigger_id.clone()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
    fn reconcile(&self) -> Result {
        self.ensure()?;
        let scope = self.weak.upgrade().ok_or(glory::ui::Error::Gone)?;
        let next = self.next()?;
        let previous = self.accepted.try_get()?;
        if next == previous {
            return Ok(());
        }
        let dialog = self.dialog.try_get()?.ok_or(glory::ui::Error::Gone)?;
        // Adapter admission precedes any parent move or accepted presentation change.
        scope.patch(dialog, Patch::Semantics(Change::Set(self.metadata(next))))?;
        if next.mode != previous.mode {
            let nav = self.nav.try_get()?.ok_or(glory::ui::Error::Gone)?;
            let parent = if next.mode == ShellMode::Mobile {
                Some(dialog)
            } else {
                self.dock.try_get()?
            };
            if let Err(error) = scope.place(nav, parent, None) {
                scope.patch(
                    dialog,
                    Patch::Semantics(Change::Set(self.metadata(previous))),
                )?;
                return Err(error);
            }
        }
        self.accepted.try_revise(|value| *value = next)?;
        if next.mode != previous.mode
            && let Some(origin) = self.nav_focus.take()
        {
            if next.mode == ShellMode::Desktop {
                scope.action(Action::Focus(origin))?;
            } else if !next.open
                && let Some(trigger) = self.trigger.try_get()?
            {
                scope.action(Action::Focus(trigger))?;
            }
        }
        Ok(())
    }
    fn retire(&self) {
        self.live.set(false);
        self.collapsed_change.clear();
        self.open_change.clear();
        self.activate.clear();
        self.dismiss.clear();
    }
    fn set_collapsed(&self, value: bool) -> Result {
        self.ensure()?;
        self.collapsed.accept(&value)?;
        self.collapsed_change.emit(self.owner, value);
        Ok(())
    }
    fn set_open(&self, value: bool) -> Result {
        self.ensure()?;
        self.open.accept(&value)?;
        self.open_change.emit(self.owner, value);
        Ok(())
    }
}
#[derive(Clone)]
pub struct SidebarHandle {
    inner: Rc<Controller>,
    collapsed: Bond<bool>,
    mobile: Bond<bool>,
    open: Bond<bool>,
}
impl SidebarHandle {
    pub fn collapsed(&self) -> Bond<bool> {
        self.collapsed
    }
    pub fn mobile(&self) -> Bond<bool> {
        self.mobile
    }
    pub fn open(&self) -> Bond<bool> {
        self.open
    }
    pub fn set_collapsed(&self, value: bool) -> Result {
        self.inner.set_collapsed(value)
    }
    pub fn set_open(&self, value: bool) -> Result {
        self.inner.set_open(value)
    }
    pub fn toggle(&self) -> Result {
        self.inner.ensure()?;
        if self.inner.accepted.try_get()?.mode == ShellMode::Mobile {
            self.set_open(!self.inner.open.source.try_get()?)
        } else {
            self.set_collapsed(!self.inner.collapsed.source.try_get()?)
        }
    }
}
#[derive(Clone)]
pub struct SidebarContext {
    pub sidebar: SidebarHandle,
    pub layout: ShellLayout,
    pub theme: crate::Theme,
    pub size: crate::Size,
    pub unstyled: bool,
}
#[derive(Clone)]
pub struct ShellContext {
    pub sidebar: Option<SidebarHandle>,
    pub layout: ShellLayout,
    pub theme: crate::Theme,
    pub size: crate::Size,
    pub unstyled: bool,
}
type Factory<C> = Box<dyn FnMut(C) -> Box<dyn Widget>>;

pub struct Sidebar {
    common: Common,
    theme_explicit: bool,
    size_explicit: bool,
    key: String,
    collapsed: Value<bool>,
    open: Value<bool>,
    layout: Option<ShellLayout>,
    content: Option<Factory<SidebarContext>>,
    width: f64,
    rail_width: f64,
    collapsed_change: Option<Box<dyn FnMut(bool)>>,
    open_change: Option<Box<dyn FnMut(bool)>>,
    nav_ref: Option<Cage<Option<NodeId>>>,
    drawer_ref: Option<Cage<Option<NodeId>>>,
}
pub fn sidebar(key: impl Into<String>) -> Sidebar {
    Sidebar {
        common: Common::default(),
        theme_explicit: false,
        size_explicit: false,
        key: key.into(),
        collapsed: Value::default(),
        open: Value::default(),
        layout: None,
        content: None,
        width: 256.0,
        rail_width: 72.0,
        collapsed_change: None,
        open_change: None,
        nav_ref: None,
        drawer_ref: None,
    }
}
macro_rules! shell_methods {
    ($($theme_explicit:ident,$size_explicit:ident)?) => {
        pub fn theme(mut self, value: crate::Theme) -> Self {
            self.common.theme = value;
            $(self.$theme_explicit=true;)?
            self
        }
        pub fn size(mut self, value: crate::Size) -> Self {
            self.common.size = value;
            $(self.$size_explicit=true;)?
            self
        }
        pub fn unstyled(mut self) -> Self {
            self.common.unstyled = true;
            self
        }
        pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
            self.common.class = value.into();
            self
        }
        pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
            self.common.style = value.into();
            self
        }
        pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
            self.common.node_ref = Some(value);
            self
        }
        pub fn on(mut self, kind: EventKind, callback: impl FnMut(Event) + 'static) -> Self {
            self.common
                .handlers
                .entry(kind)
                .or_default()
                .push(Box::new(callback));
            self
        }
    };
}
impl Sidebar {
    shell_methods!(theme_explicit, size_explicit);
    pub fn name(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn layout(mut self, value: ShellLayout) -> Self {
        self.layout = Some(value);
        self
    }
    pub fn collapsed(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.collapsed.source = Some(value.into());
        self
    }
    pub fn default_collapsed(mut self, value: bool) -> Self {
        self.collapsed.default = value;
        self
    }
    pub fn mobile_open(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.open.source = Some(value.into());
        self
    }
    pub fn default_mobile_open(mut self, value: bool) -> Self {
        self.open.default = value;
        self
    }
    pub fn width(mut self, value: f64) -> Self {
        self.width = value;
        self
    }
    pub fn rail_width(mut self, value: f64) -> Self {
        self.rail_width = value;
        self
    }
    pub fn on_collapsed_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.collapsed_change = Some(Box::new(callback));
        self
    }
    pub fn on_mobile_open_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.open_change = Some(Box::new(callback));
        self
    }
    pub fn content<W: Widget>(
        mut self,
        mut factory: impl FnMut(SidebarContext) -> W + 'static,
    ) -> Self {
        self.content = Some(Box::new(move |context| Box::new(factory(context))));
        self
    }
    pub fn nav_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.nav_ref = Some(value);
        self
    }
    pub fn drawer_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.drawer_ref = Some(value);
        self
    }
    fn controller(&mut self, scope: &Scope, layout: ShellLayout) -> Result<SidebarHandle> {
        layout.validate_scope(scope)?;
        valid_width(self.width)?;
        valid_width(self.rail_width)?;
        if self.width == 0.0 || self.rail_width == 0.0 || self.rail_width > self.width {
            return Err(invalid("sidebar needs positive widths and rail <= width"));
        }
        let nav_id = composite::id("sidebar", &self.key, "nav", "nav")?;
        let dialog_id = composite::id("sidebar", &self.key, "drawer", "drawer")?;
        let trigger_id = composite::id("sidebar", &self.key, "trigger", "trigger")?;
        let close_id = composite::id("sidebar", &self.key, "close", "close")?;
        let collapsed = std::mem::take(&mut self.collapsed).mount(scope);
        let open = std::mem::take(&mut self.open).mount(scope);
        let initial = Presentation {
            mode: if layout.mobile.try_get()? {
                ShellMode::Mobile
            } else {
                ShellMode::Desktop
            },
            collapsed: collapsed.source.try_get()?,
            open: open.source.try_get()?,
        };
        let accepted = scope.cage(initial);
        let inner = Rc::new(Controller {
            weak: scope.downgrade(),
            owner: scope.id(),
            layout: layout.clone(),
            collapsed,
            open,
            accepted,
            collapsed_change: Callback::new(self.collapsed_change.take()),
            open_change: Callback::new(self.open_change.take()),
            activate: Observers::new(
                self.common
                    .handlers
                    .remove(&EventKind::Activate)
                    .unwrap_or_default(),
            ),
            dismiss: Observers::new(
                self.common
                    .handlers
                    .remove(&EventKind::Dismiss)
                    .unwrap_or_default(),
            ),
            nav_id,
            dialog_id,
            trigger_id,
            close_id,
            nav: self.nav_ref.unwrap_or_else(|| scope.cage(None)),
            dock: scope.cage(None),
            dialog: self.drawer_ref.unwrap_or_else(|| scope.cage(None)),
            trigger: scope.cage(None),
            nav_focus: Cell::new(None),
            live: Cell::new(true),
        });
        let cleanup = inner.clone();
        scope.on_cleanup(move || cleanup.retire())?;
        Ok(SidebarHandle {
            inner,
            collapsed: scope.bond(move || accepted.get().collapsed),
            mobile: scope.bond(move || accepted.get().mode == ShellMode::Mobile),
            open: scope.bond(move || {
                let state = accepted.get();
                state.mode == ShellMode::Mobile && state.open
            }),
        })
    }
    fn build_body(&mut self, scope: &Scope, handle: SidebarHandle, embedded: bool) -> Result {
        let explicit_name = self.common.name.clone();
        let accepted = handle.inner.accepted;
        let width = self.width;
        let rail = self.rail_width;
        let common_style = self.common.style.clone();
        let theme = self.common.theme.clone();
        let unstyled = self.common.unstyled;
        let base = move || {
            let state = accepted.get();
            let mut style = if unstyled {
                Style::new()
            } else {
                Style {
                    background: Some(theme.surface),
                    color: Some(theme.foreground),
                    ..Style::new()
                }
            };
            style.width = Some(if embedded {
                Length::Px(if state.mode == ShellMode::Mobile {
                    0.0
                } else if state.collapsed {
                    rail
                } else {
                    width
                })
            } else {
                Length::Percent(100.0)
            });
            style.shrink = Some(0.0);
            merge(style, common_style.get())
        };
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-sidebar",
            Style::new()
        )
        .style(scope.bond(base));
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        let root_ref = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        root = root.node_ref(root_ref);
        root.show_in(scope)?;
        let parent = scope.child_in(root_ref.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        if !embedded {
            SidebarTrigger {
                handle: handle.clone(),
                theme: self.common.theme.clone(),
                size: self.common.size,
                unstyled: self.common.unstyled,
            }
            .show_in(&parent)?;
        }
        let mut dock = glory::ui::view()
            .class("sapid-sidebar-dock")
            .node_ref(handle.inner.dock)
            .semantics(scope.bond(move || {
                composite::metadata(Semantics {
                    hidden: accepted.get().mode == ShellMode::Mobile,
                    ..Default::default()
                })
            }));
        if !self.common.unstyled {
            dock = dock.style(scope.bond(move || {
                Style::new()
                    .width(Length::Px(if accepted.get().collapsed {
                        rail
                    } else {
                        width
                    }))
                    .overflow_x(Overflow::Hidden)
            }));
        }
        dock.show_in(&parent)?;
        let controller = handle.inner.clone();
        let open_change = controller.clone();
        let owner = controller.owner;
        let weak = scope.downgrade();
        let mut dialog = glory::ui::view()
            .role(Role::Dialog)
            .name(explicit_name.clone().unwrap_or_else(|| "Navigation".into()))
            .focus(FocusMode::Programmatic)
            .class("sapid-sidebar-drawer sapid-layer")
            .node_ref(controller.dialog)
            .semantics(controller.metadata(accepted.try_get()?))
            .on(EventKind::Dismiss, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                if let Event::Dismiss(reason) = event {
                    if let Err(error) = open_change.set_open(false) {
                        scope.report_error(error);
                        return;
                    }
                    if owner.is_alive() {
                        open_change.dismiss.emit(owner, Event::Dismiss(reason));
                    }
                }
            });
        if !self.common.unstyled {
            dialog = dialog.style(Style {
                background: Some(self.common.theme.surface),
                color: Some(self.common.theme.foreground),
                ..Style::new()
                    .width(Length::Px(width))
                    .padding(Edges::all(16.0))
            });
        }
        dialog.show_in(&parent)?;
        let dialog_scope =
            scope.child_in(controller.dialog.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        let close = controller.clone();
        let weak = scope.downgrade();
        let mut close_button = glory::ui::button()
            .label("Close navigation")
            .class("sapid-button sapid-sidebar-close")
            .semantics(composite::metadata(Semantics {
                id: Some(controller.close_id.clone()),
                ..Default::default()
            }))
            .on(EventKind::Activate, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                if let Err(error) = close.set_open(false) {
                    scope.report_error(error);
                    return;
                }
                if owner.is_alive() {
                    close.activate.emit(owner, event);
                }
            });
        if !self.common.unstyled {
            close_button = close_button.style(self.common.theme.control(self.common.size));
        }
        close_button.show_in(&dialog_scope)?;
        let nav_parent = if accepted.try_get()?.mode == ShellMode::Mobile {
            controller.dialog.try_get()?
        } else {
            controller.dock.try_get()?
        }
        .ok_or(glory::ui::Error::Gone)?;
        let nav_scope = scope.child_in(nav_parent)?;
        let focus = controller.clone();
        let mut nav = glory::ui::view()
            .role(Role::Navigation)
            .name(explicit_name.unwrap_or_else(|| "Primary navigation".into()))
            .class("sapid-sidebar-nav")
            .focus(FocusMode::Programmatic)
            .node_ref(controller.nav)
            .on(EventKind::FocusWithin, move |event| {
                if let Event::FocusWithin { node, focused, .. } = event {
                    focus.nav_focus.set(focused.then_some(node));
                }
            })
            .semantics(composite::metadata(Semantics {
                id: Some(controller.nav_id.clone()),
                ..Default::default()
            }));
        if let Some(mut factory) = self.content.take() {
            let context = SidebarContext {
                sidebar: handle.clone(),
                layout: controller.layout.clone(),
                theme: self.common.theme.clone(),
                size: self.common.size,
                unstyled: self.common.unstyled,
            };
            nav = nav.fill(glory::ui::widget(move |scope| {
                factory(context.clone()).show_in(scope)
            }));
        }
        nav.show_in(&nav_scope)?;
        scope.retain(parent)?;
        scope.retain(dialog_scope)?;
        scope.retain(nav_scope)?;
        let update = controller.clone();
        scope.effect(move || {
            if let Err(error) = update.reconcile()
                && let Some(scope) = update.weak.upgrade()
            {
                scope.report_error(error);
            }
        });
        Ok(())
    }
}
impl Widget for Sidebar {
    fn build(&mut self, scope: &Scope) -> Result {
        let layout = self.layout.take().ok_or_else(|| {
            invalid("standalone Sidebar requires an explicit ShellLayout driver")
        })?;
        let handle = self.controller(scope, layout)?;
        self.build_body(scope, handle, false)
    }
}

struct SidebarTrigger {
    handle: SidebarHandle,
    theme: crate::Theme,
    size: crate::Size,
    unstyled: bool,
}
impl Widget for SidebarTrigger {
    fn build(&mut self, scope: &Scope) -> Result {
        let controller = self.handle.inner.clone();
        let accepted = controller.accepted;
        let nav = controller.nav_id.clone();
        let dialog = controller.dialog_id.clone();
        let id = controller.trigger_id.clone();
        let label = scope.bond(move || {
            let state = accepted.get();
            if state.mode == ShellMode::Mobile {
                if state.open {
                    "Close navigation"
                } else {
                    "Open navigation"
                }
            } else if state.collapsed {
                "Expand navigation"
            } else {
                "Collapse navigation"
            }
            .to_owned()
        });
        let weak = scope.downgrade();
        let handle = self.handle.clone();
        let owner = controller.owner;
        let mut button = glory::ui::button()
            .label(label)
            .node_ref(controller.trigger)
            .class("sapid-button sapid-sidebar-trigger")
            .semantics(scope.bond(move || {
                let state = accepted.get();
                composite::metadata(Semantics {
                    id: Some(id.clone()),
                    controls: Some(if state.mode == ShellMode::Mobile {
                        dialog.clone()
                    } else {
                        nav.clone()
                    }),
                    expanded: Some(if state.mode == ShellMode::Mobile {
                        state.open
                    } else {
                        !state.collapsed
                    }),
                    popup: (state.mode == ShellMode::Mobile).then_some(PopupKind::Dialog),
                    ..Default::default()
                })
            }))
            .on(EventKind::Activate, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                if let Err(error) = handle.toggle() {
                    scope.report_error(error);
                    return;
                }
                if owner.is_alive() {
                    handle.inner.activate.emit(owner, event);
                }
            });
        if !self.unstyled {
            button = button.style(self.theme.control(self.size));
        }
        button.show_in(scope)
    }
}

pub struct AppShell {
    common: Common,
    key: String,
    initial_width: f64,
    breakpoint: f64,
    sidebar: Option<Sidebar>,
    header: Option<Factory<ShellContext>>,
    footer: Option<Factory<ShellContext>>,
    main: Option<Factory<ShellContext>>,
    layout: Option<ShellLayout>,
    skip_label: Lotus<String>,
    width_change: Option<Box<dyn FnMut(f64)>>,
    main_ref: Option<Cage<Option<NodeId>>>,
}
pub fn app_shell(key: impl Into<String>) -> AppShell {
    AppShell {
        common: Common::default(),
        key: key.into(),
        initial_width: 1024.0,
        breakpoint: 768.0,
        sidebar: None,
        header: None,
        footer: None,
        main: None,
        layout: None,
        skip_label: "Skip to main content".into(),
        width_change: None,
        main_ref: None,
    }
}
impl AppShell {
    shell_methods!();
    pub fn initial_width(mut self, value: f64) -> Self {
        self.initial_width = value;
        self
    }
    pub fn breakpoint(mut self, value: f64) -> Self {
        self.breakpoint = value;
        self
    }
    pub fn layout(mut self, value: ShellLayout) -> Self {
        self.layout = Some(value);
        self
    }
    pub fn sidebar(mut self, value: Sidebar) -> Self {
        self.sidebar = Some(value);
        self
    }
    pub fn header<W: Widget>(
        mut self,
        mut factory: impl FnMut(ShellContext) -> W + 'static,
    ) -> Self {
        self.header = Some(Box::new(move |context| Box::new(factory(context))));
        self
    }
    pub fn footer<W: Widget>(
        mut self,
        mut factory: impl FnMut(ShellContext) -> W + 'static,
    ) -> Self {
        self.footer = Some(Box::new(move |context| Box::new(factory(context))));
        self
    }
    pub fn main<W: Widget>(
        mut self,
        mut factory: impl FnMut(ShellContext) -> W + 'static,
    ) -> Self {
        self.main = Some(Box::new(move |context| Box::new(factory(context))));
        self
    }
    pub fn skip_label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.skip_label = value.into();
        self
    }
    pub fn main_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.main_ref = Some(value);
        self
    }
    pub fn on_width_change(mut self, callback: impl FnMut(f64) + 'static) -> Self {
        self.width_change = Some(Box::new(callback));
        self
    }
}
impl Widget for AppShell {
    fn build(&mut self, scope: &Scope) -> Result {
        let main_id = composite::id("shell", &self.key, "main", "main")?;
        let layout = if let Some(layout) = self.layout.take() {
            layout
        } else {
            ShellLayout::new(scope, self.initial_width, self.breakpoint)?
        };
        layout.validate_scope(scope)?;
        let mut sidebar = self.sidebar.take();
        if let Some(sidebar) = sidebar.as_mut() {
            if !sidebar.theme_explicit {
                sidebar.common.theme = self.common.theme.clone();
            }
            if !sidebar.size_explicit {
                sidebar.common.size = self.common.size;
            }
            sidebar.common.unstyled |= self.common.unstyled;
        }
        let handle = if let Some(sidebar) = sidebar.as_mut() {
            Some(sidebar.controller(scope, layout.clone())?)
        } else {
            None
        };
        let context = ShellContext {
            sidebar: handle.clone(),
            layout: layout.clone(),
            theme: self.common.theme.clone(),
            size: self.common.size,
            unstyled: self.common.unstyled,
        };
        let reference = self.main_ref.unwrap_or_else(|| scope.cage(None));
        let width_callback = Callback::new(self.width_change.take());
        let cleanup = width_callback.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let mut resize = self
            .common
            .handlers
            .remove(&EventKind::Resize)
            .unwrap_or_default();
        let owner = scope.id();
        let weak = scope.downgrade();
        let measured = layout.clone();
        let adapt = handle.clone();
        let mut base = Style::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width(Length::Percent(100.0));
        if !self.common.unstyled {
            base.background = Some(self.common.theme.surface);
            base.color = Some(self.common.theme.foreground);
        }
        let root_ref = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-app-shell",
            base
        )
        .node_ref(root_ref)
        .on(EventKind::Resize, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                if let Event::Resize { size, .. } = event {
                    let changed = measured.observe_width(size.width)?;
                    if let Some(handle) = &adapt {
                        handle.inner.reconcile()?;
                    }
                    if changed {
                        width_callback.emit(owner, size.width);
                    }
                }
                if owner.is_alive() {
                    observe(&mut resize, owner, event);
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)?;
        let parent = scope.child_in(root_ref.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        let weak = scope.downgrade();
        crate::anchor()
            .label(self.skip_label.clone())
            .href(format!("#{}", main_id.as_str()))
            .theme(self.common.theme.clone())
            .class("sapid-shell-skip")
            .on_navigate(move |_| {
                let Some(scope) = weak.upgrade() else { return };
                if let Some(main) = reference.get_untracked()
                    && let Err(error) = scope.action(Action::Focus(main))
                {
                    scope.report_error(error);
                }
            })
            .show_in(&parent)?;
        if self.header.is_some() || handle.is_some() {
            let mut header = glory::ui::view()
                .role(Role::Banner)
                .class("sapid-shell-header");
            if !self.common.unstyled {
                header = header.style(
                    Style::new()
                        .display(Display::Flex)
                        .gap(12.0)
                        .padding(Edges::all(12.0)),
                );
            }
            if let Some(handle) = handle.clone() {
                header = header.fill(SidebarTrigger {
                    handle,
                    theme: self.common.theme.clone(),
                    size: self.common.size,
                    unstyled: self.common.unstyled,
                });
            }
            if let Some(mut factory) = self.header.take() {
                let context = context.clone();
                header = header.fill(glory::ui::widget(move |scope| {
                    factory(context.clone()).show_in(scope)
                }));
            }
            header.show_in(&parent)?;
        }
        let body_ref = scope.cage(None);
        glory::ui::view()
            .class("sapid-shell-body")
            .style(Style {
                grow: Some(1.0),
                ..Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
            })
            .node_ref(body_ref)
            .show_in(&parent)?;
        let body = scope.child_in(body_ref.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        if let (Some(mut sidebar), Some(handle)) = (sidebar.take(), handle) {
            let sidebar_scope =
                scope.child_in(body_ref.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
            sidebar.build_body(&sidebar_scope, handle, true)?;
            scope.retain(sidebar_scope)?;
        }
        let mut main = glory::ui::view()
            .role(Role::Main)
            .focus(FocusMode::Programmatic)
            .node_ref(reference)
            .class("sapid-shell-main")
            .semantics(composite::metadata(Semantics {
                id: Some(main_id),
                ..Default::default()
            }))
            .style(Style {
                grow: Some(1.0),
                ..Style::new().min_width(Length::Px(0.0))
            });
        if let Some(mut factory) = self.main.take() {
            let context = context.clone();
            main = main.fill(glory::ui::widget(move |scope| {
                factory(context.clone()).show_in(scope)
            }));
        }
        main.show_in(&body)?;
        if let Some(mut factory) = self.footer.take() {
            let context = context.clone();
            let mut footer = glory::ui::view()
                .role(Role::ContentInfo)
                .class("sapid-shell-footer")
                .fill(glory::ui::widget(move |scope| {
                    factory(context.clone()).show_in(scope)
                }));
            if !self.common.unstyled {
                footer = footer.style(Style::new().padding(Edges::all(12.0)));
            }
            footer.show_in(&parent)?;
        }
        scope.retain(parent)?;
        scope.retain(body)?;
        Ok(())
    }
}
