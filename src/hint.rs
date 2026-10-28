//! Passive descriptions and interactive hover cards over shared host layers.
use std::cell::Cell;
use std::future::Future;
use std::rc::{Rc, Weak};

use glory::render::{
    DismissReason, Event, EventKind, FocusMode, Layer, LayerFocus, LayerPlacement, NodeId,
    PointerPhase, PopupKind, Role, SemanticId, Semantics,
};
use glory::style::{BoxShadow, Edges, Length, Shadow, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};

use crate::appearance::{Common, configure, observe};
use crate::composite::{self, Callback, ContentPolicy, Factory};
use crate::state::{Control, DelayFactory, OwnerDelay, Value};

pub const MAX_HINT_TEXT_BYTES: usize = 16_384;
const TARGET_HOVER: u8 = 1;
const TARGET_FOCUS: u8 = 2;
const PANEL_HOVER: u8 = 4;
const PANEL_FOCUS: u8 = 8;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Tooltip,
    HoverCard,
}
type TargetFactory = Box<dyn FnMut(FloatingTarget) -> Box<dyn Widget>>;

/// A finite adapter for an actual trigger control. Adapters preserve the
/// supplied builder's content/value/focus and install only this hint's relations
/// and boundary interests. PointerWithin and FocusWithin belong to the hint.
#[derive(Clone)]
pub struct FloatingTarget {
    id: SemanticId,
    panel: SemanticId,
    reference: Cage<Option<NodeId>>,
    scope: glory::ui::WeakScope,
    state: Weak<State>,
    conflict: Rc<Cell<bool>>,
    bound: Rc<Cell<bool>>,
}
impl FloatingTarget {
    pub fn id(&self) -> &SemanticId {
        &self.id
    }
    pub fn node_ref(&self) -> Cage<Option<NodeId>> {
        self.reference
    }
    fn metadata(&self, mut base: Semantics, button: bool) -> Vary<Option<Box<Semantics>>> {
        let (Some(scope), Some(state)) = (self.scope.upgrade(), self.state.upgrade()) else {
            self.conflict.set(true);
            return None.into();
        };
        self.bound.set(true);
        if base.id.as_ref().is_some_and(|id| id != &self.id)
            || (state.kind == Kind::Tooltip
                && base
                    .described_by
                    .as_ref()
                    .is_some_and(|id| id != &self.panel))
            || base.popup.is_some()
            || base.expanded.is_some()
            || base.controls.is_some()
        {
            self.conflict.set(true);
        }
        base.id = Some(self.id.clone());
        let id = self.panel.clone();
        let open = state.visible.clone();
        let enabled = state.enabled.clone();
        let tooltip = state.kind == Kind::Tooltip;
        scope
            .bond(move || {
                let mut value = base.clone();
                if tooltip {
                    value.described_by = enabled.get().then(|| id.clone());
                    value.popup = None;
                    value.expanded = None;
                } else {
                    value.controls = Some(id.clone());
                    if button {
                        value.popup = Some(PopupKind::Dialog);
                        value.expanded = Some(open.get());
                    }
                }
                Some(Box::new(value))
            })
            .into()
    }
    fn pointer(&self) -> impl FnMut(Event) + 'static {
        let state = self.state.clone();
        move |event| {
            if let (Some(state), Event::PointerWithin { phase, .. }) = (state.upgrade(), event)
            {
                state.activity(TARGET_HOVER, phase != PointerPhase::Leave);
            }
        }
    }
    fn focus(&self) -> impl FnMut(Event) + 'static {
        let state = self.state.clone();
        move |event| {
            if let (Some(state), Event::FocusWithin { focused, .. }) = (state.upgrade(), event)
            {
                state.activity(TARGET_FOCUS, focused);
            }
        }
    }
}
macro_rules! target_adapter {
    ($method:ident, $with:ident, $builder:ident, $button:literal) => {
        impl FloatingTarget {
            pub fn $method(&self, builder: glory::ui::$builder) -> glory::ui::$builder {
                self.$with(builder, Semantics::default())
            }
            /// Preserve existing finite metadata; conflicting IDs/relations
            /// are rejected by the enclosing hint before it is retained.
            pub fn $with(
                &self,
                builder: glory::ui::$builder,
                base: Semantics,
            ) -> glory::ui::$builder {
                builder
                    .semantics(self.metadata(base, $button))
                    .node_ref(self.reference)
                    .on(EventKind::PointerWithin, self.pointer())
                    .on(EventKind::FocusWithin, self.focus())
            }
        }
    };
}
target_adapter!(button, button_with, Button, true);
target_adapter!(link, link_with, Link, false);
target_adapter!(input, input_with, Input, false);
target_adapter!(check, check_with, Check, false);

struct Hint {
    key: String,
    kind: Kind,
    common: Common,
    label: Vary<String>,
    text: Vary<String>,
    open: Value<bool>,
    change: Option<Box<dyn FnMut(bool)>>,
    dismiss: Option<Box<dyn FnMut(DismissReason)>>,
    target: Option<TargetFactory>,
    content: Option<Factory>,
    policy: ContentPolicy,
    placement: LayerPlacement,
    open_delay: Option<DelayFactory>,
    close_delay: Option<DelayFactory>,
    target_ref: Option<Cage<Option<NodeId>>>,
}
impl Hint {
    fn new(key: impl Into<String>, kind: Kind) -> Self {
        Self {
            key: key.into(),
            kind,
            common: Common::default(),
            label: "More information".into(),
            text: "".into(),
            open: Value::default(),
            change: None,
            dismiss: None,
            target: None,
            content: None,
            policy: ContentPolicy::Unmount,
            placement: LayerPlacement::Anchor(glory::render::AnchorPlacement::new(
                glory::render::AnchorSide::Bottom,
                glory::render::AnchorAlign::Start,
            )),
            open_delay: None,
            close_delay: None,
            target_ref: None,
        }
    }
}
pub struct Tooltip {
    inner: Hint,
}
pub fn tooltip(key: impl Into<String>) -> Tooltip {
    Tooltip {
        inner: Hint::new(key, Kind::Tooltip),
    }
}
pub struct HoverCard {
    inner: Hint,
}
pub fn hover_card(key: impl Into<String>) -> HoverCard {
    HoverCard {
        inner: Hint::new(key, Kind::HoverCard),
    }
}
macro_rules! hint_api {
    ($name:ident) => {
        impl $name {
            pub fn label(mut self, value: impl Into<Vary<String>>) -> Self {
                self.inner.label = value.into();
                self
            }
            pub fn theme(mut self, value: crate::Theme) -> Self {
                self.inner.common.theme = value;
                self
            }
            pub fn size(mut self, value: crate::Size) -> Self {
                self.inner.common.size = value;
                self
            }
            pub fn class(mut self, value: impl Into<Vary<String>>) -> Self {
                self.inner.common.class = value.into();
                self
            }
            pub fn style(mut self, value: impl Into<Vary<Style>>) -> Self {
                self.inner.common.style = value.into();
                self
            }
            pub fn enabled(mut self, value: impl Into<Vary<bool>>) -> Self {
                self.inner.common.enabled = value.into();
                self
            }
            pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
                self.inner.common.name = Some(value.into());
                self
            }
            pub fn unstyled(mut self) -> Self {
                self.inner.common.unstyled = true;
                self
            }
            pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
                self.inner.common.node_ref = Some(value);
                self
            }
            pub fn trigger_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
                self.inner.target_ref = Some(value);
                self
            }
            pub fn open(mut self, value: impl Into<Vary<bool>>) -> Self {
                self.inner.open.source = Some(value.into());
                self
            }
            pub fn default_open(mut self, value: bool) -> Self {
                self.inner.open.default = value;
                self
            }
            pub fn on_open_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
                self.inner.change = Some(Box::new(callback));
                self
            }
            pub fn on_dismiss(mut self, callback: impl FnMut(DismissReason) + 'static) -> Self {
                self.inner.dismiss = Some(Box::new(callback));
                self
            }
            pub fn on(
                mut self,
                kind: EventKind,
                callback: impl FnMut(Event) + 'static,
            ) -> Self {
                self.inner
                    .common
                    .handlers
                    .entry(kind)
                    .or_default()
                    .push(Box::new(callback));
                self
            }
            pub fn target<W: Widget + 'static>(
                mut self,
                mut factory: impl FnMut(FloatingTarget) -> W + 'static,
            ) -> Self {
                self.inner.target = Some(Box::new(move |target| Box::new(factory(target))));
                self
            }
            pub fn placement(mut self, value: LayerPlacement) -> Self {
                self.inner.placement = value;
                self
            }
            pub fn open_delay_with<
                F: Fn() -> Fut + 'static,
                Fut: Future<Output = ()> + 'static,
            >(
                mut self,
                factory: F,
            ) -> Self {
                self.inner.open_delay = Some(Rc::new(move || Box::pin(factory())));
                self
            }
            pub fn close_delay_with<
                F: Fn() -> Fut + 'static,
                Fut: Future<Output = ()> + 'static,
            >(
                mut self,
                factory: F,
            ) -> Self {
                self.inner.close_delay = Some(Rc::new(move || Box::pin(factory())));
                self
            }
        }
        impl Widget for $name {
            fn build(&mut self, scope: &Scope) -> Result {
                self.inner.build(scope)
            }
        }
    };
}
hint_api!(Tooltip);
hint_api!(HoverCard);
impl Tooltip {
    pub fn text(mut self, value: impl Into<Vary<String>>) -> Self {
        self.inner.text = value.into();
        self
    }
}
impl HoverCard {
    pub fn content<W: Widget + 'static>(
        mut self,
        mut factory: impl FnMut() -> W + 'static,
    ) -> Self {
        self.inner.content = Some(Box::new(move || Box::new(factory())));
        self
    }
    pub fn content_policy(mut self, value: ContentPolicy) -> Self {
        self.inner.policy = value;
        self
    }
}

struct State {
    scope: glory::ui::WeakScope,
    owner: glory::core::InstanceId,
    kind: Kind,
    open: Control<bool>,
    changed: Callback<bool>,
    dismissed: Callback<DismissReason>,
    visible: Vary<bool>,
    enabled: Vary<bool>,
    activity: Cage<(u8, bool, bool)>,
    pending: Rc<OwnerDelay>,
    open_delay: Option<DelayFactory>,
    close_delay: Option<DelayFactory>,
}
impl State {
    fn activity(&self, bit: u8, entered: bool) {
        if !self.owner.is_alive() {
            return;
        }
        match self
            .activity
            .try_with(|(bits, _, _)| (*bits & bit != 0) == entered)
        {
            Ok(true) => return,
            Err(error) => {
                if let Some(scope) = self.scope.upgrade() {
                    scope.report_error(error);
                }
                return;
            }
            Ok(false) => {}
        }
        let result = self.activity.try_revise(|(bits, dismissed, touched)| {
            if entered {
                *bits |= bit;
                *dismissed = false;
            } else {
                *bits &= !bit;
            }
            *touched = true;
        });
        if let Err(error) = result
            && let Some(scope) = self.scope.upgrade()
        {
            scope.report_error(error);
        }
    }
    fn request(&self, value: bool) -> Result {
        if self.owner.is_alive()
            && glory::core::untracked(|| self.open.source.try_get())? != value
        {
            self.open.accept(&value)?;
            glory::core::untracked(|| self.changed.emit(self.owner, value));
        }
        Ok(())
    }
    fn dismiss(&self, reason: DismissReason) -> Result {
        self.pending.cancel();
        self.activity.try_revise(|(_, dismissed, touched)| {
            *dismissed = true;
            *touched = true;
        })?;
        self.request(false)?;
        if self.owner.is_alive() {
            self.dismissed.emit(self.owner, reason);
        }
        Ok(())
    }
}

impl Hint {
    fn build(&mut self, scope: &Scope) -> Result {
        if !matches!(self.placement, LayerPlacement::Anchor(_)) {
            return Err(glory::render::Error::InvalidProps(
                "hints require an anchor placement",
            )
            .into());
        }
        if self.kind == Kind::HoverCard && self.content.is_none() {
            return Err(glory::render::Error::InvalidProps(
                "hover cards require explicit content",
            )
            .into());
        }
        let target_id = composite::id("hint", &self.key, "target", "root")?;
        let panel_id = composite::id("hint", &self.key, "panel", "root")?;
        let owner = scope.child()?;
        let control = std::mem::take(&mut self.open).mount(&owner);
        let source = control.source.clone();
        let enabled = self.common.enabled.clone();
        let active_enabled = enabled.clone();
        let visible: Vary<bool> = owner
            .bond(move || source.get() && active_enabled.get())
            .into();
        let state = Rc::new(State {
            scope: owner.downgrade(),
            owner: owner.id(),
            kind: self.kind,
            open: control,
            changed: Callback::new(self.change.take()),
            dismissed: Callback::new(self.dismiss.take()),
            visible: visible.clone(),
            enabled,
            activity: owner.cage((0, false, false)),
            pending: OwnerDelay::new(&owner)?,
            open_delay: self.open_delay.clone(),
            close_delay: self.close_delay.clone(),
        });
        let target = FloatingTarget {
            id: target_id.clone(),
            panel: panel_id.clone(),
            reference: self.target_ref.unwrap_or_else(|| owner.cage(None)),
            scope: owner.downgrade(),
            state: Rc::downgrade(&state),
            conflict: Rc::new(Cell::new(false)),
            bound: Rc::new(Cell::new(false)),
        };
        if let Some(factory) = &mut self.target {
            factory(target.clone()).show_in(&owner)?;
        } else {
            let mut common = Common {
                theme: self.common.theme.clone(),
                size: self.common.size,
                unstyled: self.common.unstyled,
                ..Default::default()
            };
            let builder = configure!(
                common,
                glory::ui::button()
                    .label(self.label.clone())
                    .enabled(self.common.enabled.clone()),
                &owner,
                "sapid-hint-trigger",
                common.theme.control(common.size)
            );
            target.button(builder).show_in(&owner)?;
        }
        if !target.bound.get() || target.reference.try_get()?.is_none() || target.conflict.get()
        {
            return Err(glory::render::Error::InvalidProps(
                "hint target requires its typed control adapter and nonconflicting relations",
            )
            .into());
        }
        drop(target);
        let role = if self.kind == Kind::Tooltip {
            Role::Tooltip
        } else {
            Role::Dialog
        };
        let config = Layer {
            sheet: None,
            modal: false,
            escape: true,
            outside: self.kind == Kind::HoverCard,
            focus: LayerFocus::Preserve,
            placement: self.placement,
            anchor: Some(target_id),
            initial_focus: None,
            return_focus: None,
        };
        let stable = panel_id;
        let panel_name = self.common.name.clone();
        let mut panel = configure!(
            self.common,
            glory::ui::view().role(role),
            &owner,
            if self.kind == Kind::Tooltip {
                "sapid-tooltip sapid-layer"
            } else {
                "sapid-hover-card sapid-layer"
            },
            hint_style(&self.common, self.kind)
        )
        .focus(if self.kind == Kind::Tooltip {
            FocusMode::Auto
        } else {
            FocusMode::Programmatic
        })
        .semantics(owner.bond(move || {
            composite::metadata(Semantics {
                id: Some(stable.clone()),
                hidden: !visible.get(),
                layer: Some(config.clone()),
                ..Default::default()
            })
        }));
        if self.kind == Kind::HoverCard {
            panel = panel.name(panel_name.unwrap_or(self.label.clone()));
        }
        let instance = owner.id();
        let weak_state = Rc::downgrade(&state);
        let mut pointer_observers = self
            .common
            .handlers
            .remove(&EventKind::PointerWithin)
            .unwrap_or_default();
        panel = panel.on(EventKind::PointerWithin, move |event| {
            if let (Some(state), Event::PointerWithin { phase, .. }) =
                (weak_state.upgrade(), &event)
            {
                state.activity(PANEL_HOVER, *phase != PointerPhase::Leave);
            }
            observe(&mut pointer_observers, instance, event);
        });
        let weak_state = Rc::downgrade(&state);
        let mut focus_observers = self
            .common
            .handlers
            .remove(&EventKind::FocusWithin)
            .unwrap_or_default();
        panel = panel.on(EventKind::FocusWithin, move |event| {
            if let (Some(state), Event::FocusWithin { focused, .. }) =
                (weak_state.upgrade(), &event)
            {
                state.activity(PANEL_FOCUS, *focused);
            }
            observe(&mut focus_observers, instance, event);
        });
        let weak_state = Rc::downgrade(&state);
        let weak_owner = owner.downgrade();
        let mut dismiss_observers = self
            .common
            .handlers
            .remove(&EventKind::Dismiss)
            .unwrap_or_default();
        panel = panel.on(EventKind::Dismiss, move |event| {
            if let (Some(state), Event::Dismiss(reason)) = (weak_state.upgrade(), &event)
                && let Err(error) = state.dismiss(*reason)
            {
                if let Some(scope) = weak_owner.upgrade() {
                    scope.report_error(error);
                }
                return;
            }
            observe(&mut dismiss_observers, instance, event);
        });
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            panel = panel.on(kind, move |event| observe(&mut handlers, instance, event));
        }
        if self.kind == Kind::Tooltip {
            let text = composite::validated(&owner, &self.text, |value| validate_text(value))?;
            panel
                .fill(glory::ui::text().content(text))
                .show_in(&owner)?;
        } else {
            let mut content = self.content.take().expect("validated content");
            match self.policy {
                ContentPolicy::KeepMounted => panel.fill(content()).show_in(&owner)?,
                ContentPolicy::Unmount => panel
                    .fill(glory::ui::when(state.visible.clone(), content))
                    .show_in(&owner)?,
            }
        }
        let weak = Rc::downgrade(&state);
        let weak_owner = owner.downgrade();
        owner.structural(move || {
            let (Some(state), Some(scope)) = (weak.upgrade(), weak_owner.upgrade()) else {
                return;
            };
            let result = (|| -> Result {
                let (bits, dismissed, touched) = state.activity.try_get()?;
                let enabled = state.enabled.try_get()?;
                if dismissed {
                    state.pending.cancel();
                    return Ok(());
                }
                if !touched && enabled {
                    return Ok(());
                }
                let desired = enabled && bits != 0 && !dismissed;
                if glory::core::untracked(|| state.open.source.try_get())? == desired {
                    state.pending.cancel();
                    return Ok(());
                }
                let weak = Rc::downgrade(&state);
                state.pending.run(
                    &scope,
                    if desired {
                        state.open_delay.as_ref()
                    } else {
                        state.close_delay.as_ref()
                    },
                    move || {
                        if let Some(state) = weak.upgrade() {
                            state.request(desired)?;
                        }
                        Ok(())
                    },
                )
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        // Keep current controller state with its mounting Owner, while handlers
        // and effects retain only weak references.
        owner.on_cleanup(move || drop(state))?;
        scope.retain(owner)?;
        Ok(())
    }
}
fn validate_text(value: &str) -> Result {
    if value.is_empty() || value.len() > MAX_HINT_TEXT_BYTES {
        Err(glory::render::Error::InvalidProps("tooltip text requires 1..=16384 bytes").into())
    } else {
        Ok(())
    }
}
fn hint_style(common: &Common, kind: Kind) -> Style {
    let mut style = Style::new()
        .width(Length::Px(if kind == Kind::Tooltip {
            240.0
        } else {
            320.0
        }))
        .padding(Edges::all(if kind == Kind::Tooltip { 8.0 } else { 16.0 }))
        .border_width(Edges::all(1.0))
        .border_style(glory::style::BorderStyle::Solid)
        .radius(6.0)
        .box_shadow(BoxShadow::Outset(Shadow {
            x: 0.0,
            y: 4.0,
            blur: 12.0,
            spread: 0.0,
            color: common.theme.shadow,
        }));
    style.background = Some(common.theme.surface);
    style.color = Some(common.theme.foreground);
    style.border_color = Some(common.theme.border);
    style
}
