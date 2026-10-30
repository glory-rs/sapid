use super::*;
use glory::render::{Current, FocusMode, LinkOpen};

macro_rules! common {
    () => {
        pub fn context(mut self, value: NavigationMenuContext) -> Self {
            self.context = Some(value);
            self
        }
        pub fn label(mut self, value: impl Into<Vary<String>>) -> Self {
            self.label = value.into();
            self
        }
        pub fn theme(mut self, value: crate::Theme) -> Self {
            self.common.theme = value;
            self.inherit_theme = false;
            self
        }
        pub fn size(mut self, value: crate::Size) -> Self {
            self.common.size = value;
            self.inherit_size = false;
            self
        }
        pub fn unstyled(mut self) -> Self {
            self.common.unstyled = true;
            self
        }
        pub fn class(mut self, value: impl Into<Vary<String>>) -> Self {
            self.common.class = value.into();
            self
        }
        pub fn style(mut self, value: impl Into<Vary<Style>>) -> Self {
            self.common.style = value.into();
            self
        }
        pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
            self.common.name = Some(value.into());
            self
        }
        pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
            self.common.node_ref = Some(value);
            self
        }
        pub fn on(mut self, kind: EventKind, handler: impl FnMut(Event) + 'static) -> Self {
            self.common
                .handlers
                .entry(kind)
                .or_default()
                .push(Box::new(handler));
            self
        }
    };
}
pub struct NavigationMenuLink {
    key: String,
    context: Option<NavigationMenuContext>,
    common: Common,
    inherit_theme: bool,
    inherit_size: bool,
    label: Vary<String>,
    href: Vary<String>,
    open: Vary<LinkOpen>,
    current: Vary<Option<Current>>,
    navigate: Option<Box<dyn FnMut(crate::AnchorRequest)>>,
}
pub fn navigation_menu_link(key: impl Into<String>) -> NavigationMenuLink {
    NavigationMenuLink {
        key: key.into(),
        context: None,
        common: Common::default(),
        inherit_theme: true,
        inherit_size: true,
        label: "".into(),
        href: "".into(),
        open: LinkOpen::Current.into(),
        current: None.into(),
        navigate: None,
    }
}
impl NavigationMenuLink {
    common!();
    pub fn href(mut self, value: impl Into<Vary<String>>) -> Self {
        self.href = value.into();
        self
    }
    pub fn open_in(mut self, value: impl Into<Vary<LinkOpen>>) -> Self {
        self.open = value.into();
        self
    }
    pub fn current(mut self, value: impl Into<Vary<Option<Current>>>) -> Self {
        self.current = value.into();
        self
    }
    pub fn on_navigate(mut self, callback: impl FnMut(crate::AnchorRequest) + 'static) -> Self {
        self.navigate = Some(Box::new(callback));
        self
    }
}
pub struct NavigationMenuPanel {
    key: String,
    context: Option<NavigationMenuContext>,
    common: Common,
    inherit_theme: bool,
    inherit_size: bool,
    label: Vary<String>,
    content: Option<Factory>,
    panel_ref: Option<Cage<Option<NodeId>>>,
}
pub fn navigation_menu_panel(key: impl Into<String>) -> NavigationMenuPanel {
    NavigationMenuPanel {
        key: key.into(),
        context: None,
        common: Common::default(),
        inherit_theme: true,
        inherit_size: true,
        label: "".into(),
        content: None,
        panel_ref: None,
    }
}
impl NavigationMenuPanel {
    common!();
    pub fn enabled(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.common.enabled = value.into();
        self
    }
    pub fn panel_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.panel_ref = Some(value);
        self
    }
    pub fn content<W: glory::ui::IntoChildren>(
        mut self,
        mut factory: impl FnMut(NavigationMenuContext) -> W + 'static,
    ) -> Self {
        self.content = Some(Box::new(move |context, scope| {
            glory::ui::group().fill(factory(context)).show_in(scope)
        }));
        self
    }
}
fn inherit(common: &mut Common, context: &NavigationMenuContext, theme: bool, size: bool) {
    if theme {
        common.theme = context.theme.clone()
    }
    if size {
        common.size = context.size
    }
    common.unstyled |= context.unstyled;
}
fn entry(
    scope: &Scope,
    context: &NavigationMenuContext,
    key: &str,
    common: &Common,
    label: Cage<String>,
    panel: Option<Cage<Option<NodeId>>>,
) -> Result<(Rc<Root>, Entry, crate::collection::CollectionMember)> {
    let root = context.root(scope)?;
    valid_key(key)?;
    let reference = common.node_ref.unwrap_or_else(|| scope.cage(None));
    let member =
        context
            .level
            .collection
            .register(scope, key, reference, common.enabled.clone(), label)?;
    let registered = root.register(
        scope,
        Entry {
            token: 0,
            key: key.to_owned(),
            parent: context.level.parent,
            owner: scope.id(),
            level: context.level.clone(),
            reference,
            panel,
            opened: scope.cage(false),
            trigger_id: SemanticId::new("pending")?,
            panel_id: SemanticId::new("pending")?,
            enabled: common.enabled.clone(),
        },
    )?;
    Ok((root, registered, member))
}
fn wrapper(scope: &Scope) -> Result<Scope> {
    let reference = scope.cage(None);
    glory::ui::view()
        .role(Role::ListItem)
        .class("sapid-navigation-menu-item")
        .node_ref(reference)
        .show_in(scope)?;
    scope.child_in(reference.try_get()?.ok_or(glory::ui::Error::Gone)?)
}
fn observers(common: &mut Common, kind: EventKind) -> Observers {
    Observers::new(common.handlers.remove(&kind).unwrap_or_default())
}

impl Widget for NavigationMenuLink {
    fn build(&mut self, scope: &Scope) -> Result {
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| invalid("navigation items require a context"))?;
        inherit(
            &mut self.common,
            context,
            self.inherit_theme,
            self.inherit_size,
        );
        let label = super::label(scope, &self.label)?;
        let (root, entry, member) = entry(scope, context, &self.key, &self.common, label, None)?;
        let parent = wrapper(scope)?;
        let owner = scope.id();
        let request = self.navigate.is_some();
        let callback = Callback::new(self.navigate.take());
        let clear = callback.clone();
        scope.on_cleanup(move || clear.clear())?;
        let id = entry.trigger_id.clone();
        let current = self.current.clone();
        let open = self.open.clone();
        let metadata = scope.bond(move || {
            composite::metadata(Semantics {
                id: Some(id.clone()),
                navigation_item: true,
                current: current.get(),
                link_open: Some(open.get()),
                link_request: request,
                ..Default::default()
            })
        });
        let mut style = self.common.theme.label(self.common.size);
        style.color = Some(self.common.theme.accent_text);
        let mut link = configure!(
            self.common,
            glory::ui::link(),
            scope,
            "sapid-navigation-menu-link",
            style
        )
        .label(label)
        .target(self.href.clone())
        .node_ref(entry.reference)
        .semantics(metadata);
        let active = observers(&mut self.common, EventKind::Activate);
        let state = root.clone();
        let token = entry.token;
        let weak = scope.downgrade();
        let reference = entry.reference;
        link = link.on(EventKind::Activate, move |event| {
            if let Err(error) = state.activate(token) {
                state.report(error);
                return;
            }
            if request {
                let result = (|| {
                    let scope = weak.upgrade().ok_or(glory::ui::Error::Gone)?;
                    let (href, open) =
                        scope.link_request(reference.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
                    Ok::<_, glory::ui::Error>(crate::AnchorRequest { href, open })
                })();
                match result {
                    Ok(request) => callback.emit(owner, request),
                    Err(error) => {
                        state.report(error);
                        return;
                    }
                }
            }
            active.emit(owner, event)
        });
        let focus = observers(&mut self.common, EventKind::FocusChanged);
        let roving = context.level.roving.clone();
        let weak = scope.downgrade();
        link = link.on(EventKind::FocusChanged, move |event| {
            if matches!(event, Event::FocusChanged(true))
                && let Err(error) = roving.observe(&member)
            {
                if let Some(scope) = weak.upgrade() {
                    scope.report_error(error)
                }
                return;
            }
            focus.emit(owner, event)
        });
        if root.hover {
            let moves = observers(&mut self.common, EventKind::PointerWithin);
            let state = root.clone();
            link = link.on(EventKind::PointerWithin, move |event| {
                if matches!(
                    event,
                    Event::PointerWithin {
                        phase: PointerPhase::Move,
                        ..
                    }
                ) {
                    let result = state.path(token).and_then(|mut path| {
                        path.pop();
                        state.hover(path, false)
                    });
                    if let Err(error) = result {
                        state.report(error);
                        return;
                    }
                }
                if matches!(
                    event,
                    Event::PointerWithin {
                        phase: PointerPhase::Leave,
                        ..
                    }
                ) {
                    let cancel = state.hover_intent.borrow().as_ref().is_some_and(|pending| {
                        !pending.close && state.path(token).is_ok_and(|path| pending.path == path)
                    });
                    if cancel {
                        state.cancel_hover()
                    }
                }
                moves.emit(owner, event)
            });
        }
        if self.common.handlers.contains_key(&EventKind::Navigate) {
            let observers = observers(&mut self.common, EventKind::Navigate);
            let state = root.clone();
            let level = context.level.clone();
            link = link.on(EventKind::Navigate, move |event| {
                if let Event::Navigate { from, command } = event
                    && let Err(error) = state.navigate(&level, from, command)
                {
                    state.report(error);
                    return;
                }
                observers.emit(owner, event)
            });
        }
        if self
            .common
            .handlers
            .contains_key(&EventKind::NavigationPanel)
        {
            let observers = observers(&mut self.common, EventKind::NavigationPanel);
            let state = root.clone();
            let parent = entry.parent;
            link = link.on(EventKind::NavigationPanel, move |event| {
                if matches!(
                    event,
                    Event::NavigationPanel {
                        command: NavigationPanelCommand::Close,
                        ..
                    }
                ) && let Some(parent) = parent
                    && let Err(error) = state.close(parent, true)
                {
                    state.report(error);
                    return;
                }
                observers.emit(owner, event)
            });
        }
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            link = link.on(kind, move |event| observe(&mut handlers, owner, event))
        }
        link.show_in(&parent)?;
        scope.retain(parent)?;
        Ok(())
    }
}
impl Widget for NavigationMenuPanel {
    fn build(&mut self, scope: &Scope) -> Result {
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| invalid("navigation items require a context"))?;
        if context.level.depth >= MAX_NAVIGATION_MENU_DEPTH {
            return Err(invalid("navigation has at most eight panel levels"));
        }
        inherit(
            &mut self.common,
            context,
            self.inherit_theme,
            self.inherit_size,
        );
        let label = super::label(scope, &self.label)?;
        let panel_ref = self.panel_ref.unwrap_or_else(|| scope.cage(None));
        let (root, entry, member) = entry(
            scope,
            context,
            &self.key,
            &self.common,
            label,
            Some(panel_ref),
        )?;
        let parent = wrapper(scope)?;
        let owner = scope.id();
        let opened = entry.opened;
        let id = entry.trigger_id.clone();
        let controls = entry.panel_id.clone();
        let metadata = scope.bond(move || {
            composite::metadata(Semantics {
                id: Some(id.clone()),
                controls: Some(controls.clone()),
                expanded: Some(opened.get()),
                navigation_item: true,
                navigation_panel: Some(NavigationPanel::Trigger),
                ..Default::default()
            })
        });
        let mut trigger = configure!(
            self.common,
            glory::ui::button(),
            scope,
            "sapid-navigation-menu-trigger",
            self.common.theme.control(self.common.size)
        )
        .label(label)
        .enabled(self.common.enabled.clone())
        .node_ref(entry.reference)
        .semantics(metadata);
        let active = observers(&mut self.common, EventKind::Activate);
        let state = root.clone();
        let token = entry.token;
        trigger = trigger.on(EventKind::Activate, move |event| {
            if let Err(error) = state.activate(token) {
                state.report(error);
                return;
            }
            active.emit(owner, event)
        });
        let commands = observers(&mut self.common, EventKind::NavigationPanel);
        let state = root.clone();
        let current = entry.clone();
        trigger = trigger.on(EventKind::NavigationPanel, move |event| {
            if let Event::NavigationPanel { command, .. } = event
                && let Err(error) = state.panel_command(&current, command)
            {
                state.report(error);
                return;
            }
            commands.emit(owner, event)
        });
        let focus = observers(&mut self.common, EventKind::FocusChanged);
        let roving = context.level.roving.clone();
        let state = root.clone();
        trigger = trigger.on(EventKind::FocusChanged, move |event| {
            if matches!(event, Event::FocusChanged(true))
                && let Err(error) = roving.observe(&member)
            {
                state.report(error);
                return;
            }
            focus.emit(owner, event)
        });
        if root.hover {
            let moves = observers(&mut self.common, EventKind::PointerWithin);
            let state = root.clone();
            trigger = trigger.on(EventKind::PointerWithin, move |event| {
                if matches!(
                    event,
                    Event::PointerWithin {
                        phase: PointerPhase::Move,
                        ..
                    }
                ) {
                    let result = state.path(token).and_then(|path| state.hover(path, false));
                    if let Err(error) = result {
                        state.report(error);
                        return;
                    }
                }
                if matches!(
                    event,
                    Event::PointerWithin {
                        phase: PointerPhase::Leave,
                        ..
                    }
                ) {
                    let cancel = state.hover_intent.borrow().as_ref().is_some_and(|pending| {
                        !pending.close && state.path(token).is_ok_and(|path| pending.path == path)
                    });
                    if cancel {
                        state.cancel_hover()
                    }
                }
                moves.emit(owner, event)
            });
        }
        if self.common.handlers.contains_key(&EventKind::Navigate) {
            let observers = observers(&mut self.common, EventKind::Navigate);
            let state = root.clone();
            let level = context.level.clone();
            trigger = trigger.on(EventKind::Navigate, move |event| {
                if let Event::Navigate { from, command } = event
                    && let Err(error) = state.navigate(&level, from, command)
                {
                    state.report(error);
                    return;
                }
                observers.emit(owner, event)
            });
        }
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            trigger = trigger.on(kind, move |event| observe(&mut handlers, owner, event))
        }
        trigger.show_in(&parent)?;
        let focused = scope.cage(false);
        let state = root.clone();
        let close = root.clone();
        let navigate = root.clone();
        let focus_root = root.clone();
        let collection = Collection::new(
            scope,
            CollectionLimits {
                items: MAX_NAVIGATION_MENU_ITEMS,
                label_bytes: 4096,
            },
        )?;
        let level = Rc::new(Level {
            parent: Some(token),
            depth: context.level.depth + 1,
            container: panel_ref,
            roving: RovingFocus::new(scope, collection.clone(), true)?,
            collection,
        });
        let current = level.clone();
        glory::ui::view()
            .role(Role::Group)
            .focus(FocusMode::Auto)
            .name(label)
            .class("sapid-navigation-menu-panel")
            .node_ref(panel_ref)
            .semantics(root.metadata(&entry, false))
            .style(Style {
                background: Some(self.common.theme.surface),
                color: Some(self.common.theme.foreground),
                ..Style::new()
                    .width(Length::Px(280.0))
                    .padding(Edges::all(12.0))
            })
            .on(EventKind::FocusWithin, move |event| {
                if let Event::FocusWithin {
                    node,
                    focused: value,
                } = event
                {
                    let _ = focused.try_revise(|focused| *focused = value);
                    if let Err(error) = focus_root.observe_focus(node, value, Some(token)) {
                        focus_root.report(error)
                    }
                }
            })
            .on(EventKind::Dismiss, move |event| {
                if let Event::Dismiss(reason) = event
                    && let Err(error) = state.close(
                        token,
                        reason == DismissReason::Escape && focused.get_untracked(),
                    )
                {
                    state.report(error)
                }
            })
            .on(EventKind::NavigationPanel, move |event| {
                if matches!(
                    event,
                    Event::NavigationPanel {
                        command: NavigationPanelCommand::Close,
                        ..
                    }
                ) && let Err(error) = close.close(token, true)
                {
                    close.report(error)
                }
            })
            .on(EventKind::Navigate, move |event| {
                if let Event::Navigate { from, command } = event
                    && let Err(error) = navigate.navigate(&current, from, command)
                {
                    navigate.report(error)
                }
            })
            .show_in(&parent)?;
        let panel_scope = scope.child_in(panel_ref.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        let list_ref = scope.cage(None);
        glory::ui::view()
            .role(Role::List)
            .node_ref(list_ref)
            .style(composite::column(6.0))
            .show_in(&panel_scope)?;
        let content_scope = scope.child_in(list_ref.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        if let Some(factory) = self.content.as_mut() {
            factory(
                NavigationMenuContext {
                    root: Rc::downgrade(&root),
                    level,
                    theme: self.common.theme.clone(),
                    size: self.common.size,
                    unstyled: self.common.unstyled,
                    open: scope.bond(move || opened.get()),
                },
                &content_scope,
            )?
        }
        scope.retain(parent)?;
        scope.retain(panel_scope)?;
        scope.retain(content_scope)?;
        Ok(())
    }
}
