//! Semantic links; the application owns navigation and external transport.
use crate::appearance::{Common, configure, observe};
use crate::composite::{self, Callback};
use glory::render::{
    Change, Current, EventKind, FocusMode, LinkOpen, NodeId, Patch, SemanticId, Semantics,
};
use glory::style::Style;
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnchorRequest {
    pub href: String,
    pub open: LinkOpen,
}
pub struct Anchor {
    common: Common,
    href: Vary<String>,
    label: Vary<String>,
    open: Vary<LinkOpen>,
    current: Vary<Option<Current>>,
    semantic_id: Option<SemanticId>,
    navigate: Option<Box<dyn FnMut(AnchorRequest)>>,
}
pub fn anchor() -> Anchor {
    Anchor {
        common: Common::default(),
        href: "".into(),
        label: "".into(),
        open: LinkOpen::Current.into(),
        current: None.into(),
        semantic_id: None,
        navigate: None,
    }
}
impl Anchor {
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn size(mut self, value: crate::Size) -> Self {
        self.common.size = value;
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
    pub fn focus(mut self, value: FocusMode) -> Self {
        self.common.focus = value;
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
    /// The semantic link destination (Glory calls this Link.target).
    pub fn href(mut self, value: impl Into<Vary<String>>) -> Self {
        self.href = value.into();
        self
    }
    pub fn label(mut self, value: impl Into<Vary<String>>) -> Self {
        self.label = value.into();
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
    pub(crate) fn semantic_id(mut self, value: SemanticId) -> Self {
        self.semantic_id = Some(value);
        self
    }
    /// A host activation request, emitted before generic Activate observers.
    pub fn on_navigate(mut self, callback: impl FnMut(AnchorRequest) + 'static) -> Self {
        self.navigate = Some(Box::new(callback));
        self
    }
    pub fn on(
        mut self,
        kind: EventKind,
        callback: impl FnMut(glory::render::Event) + 'static,
    ) -> Self {
        self.common
            .handlers
            .entry(kind)
            .or_default()
            .push(Box::new(callback));
        self
    }
}
impl Widget for Anchor {
    fn build(&mut self, scope: &Scope) -> Result {
        let mut style = self.common.theme.label(self.common.size);
        style.color = Some(self.common.theme.accent_text);
        style.font_weight = Some(400.0);
        let initial_open = self.open.try_get()?;
        let initial_current = self.current.try_get()?;
        let semantic_id = self.semantic_id.clone();
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let link_request = self.navigate.is_some();
        let mut link = configure!(self.common, glory::ui::link(), scope, "sapid-anchor", style)
            .target(self.href.clone())
            .label(self.label.clone())
            .node_ref(reference)
            .semantics(composite::metadata(Semantics {
                link_open: Some(initial_open),
                link_request,
                current: initial_current,
                id: semantic_id.clone(),
                ..Default::default()
            }));
        let callback = Callback::new(self.navigate.take());
        let mut observers = self
            .common
            .handlers
            .remove(&EventKind::Activate)
            .unwrap_or_default();
        let owner = scope.id();

        let weak = scope.downgrade();
        link = link.on(EventKind::Activate, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            let request = (|| -> Result<AnchorRequest> {
                let node = reference.try_get()?.ok_or(glory::ui::Error::Gone)?;
                let (href, open) = scope.link_request(node)?;
                Ok(AnchorRequest { href, open })
            })();
            match request {
                Ok(request) => callback.emit(owner, request),
                Err(error) => {
                    scope.report_error(error);
                    return;
                }
            }
            observe(&mut observers, owner, event);
        });
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            link = link.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        link.show_in(scope)?;
        if self.open.is_reactive() || self.current.is_reactive() {
            let open = self.open.clone();
            let current = self.current.clone();
            let weak = scope.downgrade();
            let mut previous = (initial_open, initial_current);
            scope.binding(move || {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    let next = (open.try_get()?, current.try_get()?);
                    if previous != next {
                        let node = reference.try_get()?.ok_or(glory::ui::Error::Gone)?;
                        scope.patch(
                            node,
                            Patch::Semantics(Change::Set(composite::metadata(Semantics {
                                link_open: Some(next.0),
                                link_request,
                                current: next.1,
                                id: semantic_id.clone(),
                                ..Default::default()
                            }))),
                        )?;
                        previous = next;
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                }
            });
        }
        Ok(())
    }
}
