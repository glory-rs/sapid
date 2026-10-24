//! A nonmodal focus policy around ordinary content. The host owns actual
//! traversal and focus restoration; this widget creates no overlay.
use glory::render::{FocusMode, NodeId, SemanticId, Semantics};
pub use glory::render::{FocusScopeConfig, MAX_FOCUS_SCOPE_NODES, MAX_FOCUS_SCOPES};
use glory::style::Style;
use glory::ui::{Children, IntoChildren, Result, Widget};
use glory::{Cage, Lotus, Scope};

pub struct FocusScope {
    configuration: Lotus<FocusScopeConfig>,
    active: Option<Lotus<bool>>,
    loop_tab: Option<bool>,
    contain_focus: Option<bool>,
    auto_focus: Option<bool>,
    restore_focus: Option<bool>,
    initial_focus: Option<SemanticId>,
    return_focus: Option<SemanticId>,
    id: Option<SemanticId>,
    class: Lotus<String>,
    style: Lotus<Style>,
    node_ref: Option<Cage<Option<NodeId>>>,
    content: Option<Children>,
}
pub fn focus_scope() -> FocusScope {
    FocusScope {
        configuration: FocusScopeConfig::default().into(),
        active: None,
        loop_tab: None,
        contain_focus: None,
        auto_focus: None,
        restore_focus: None,
        initial_focus: None,
        return_focus: None,
        id: None,
        class: "".into(),
        style: Style::new().into(),
        node_ref: None,
        content: None,
    }
}
impl FocusScope {
    pub fn configuration(mut self, configuration: impl Into<Lotus<FocusScopeConfig>>) -> Self {
        self.configuration = configuration.into();
        self
    }
    pub fn active(mut self, active: impl Into<Lotus<bool>>) -> Self {
        self.active = Some(active.into());
        self
    }
    pub fn loop_tab(mut self, enabled: bool) -> Self {
        self.loop_tab = Some(enabled);
        self
    }
    pub fn contain_focus(mut self, enabled: bool) -> Self {
        self.contain_focus = Some(enabled);
        self
    }
    pub fn auto_focus(mut self, enabled: bool) -> Self {
        self.auto_focus = Some(enabled);
        self
    }
    pub fn restore_focus(mut self, enabled: bool) -> Self {
        self.restore_focus = Some(enabled);
        self
    }
    pub fn initial_focus(mut self, id: SemanticId) -> Self {
        self.initial_focus = Some(id);
        self
    }
    pub fn return_focus(mut self, id: SemanticId) -> Self {
        self.return_focus = Some(id);
        self
    }
    pub fn semantic_id(mut self, id: SemanticId) -> Self {
        self.id = Some(id);
        self
    }
    pub fn class(mut self, class: impl Into<Lotus<String>>) -> Self {
        self.class = class.into();
        self
    }
    pub fn style(mut self, style: impl Into<Lotus<Style>>) -> Self {
        self.style = style.into();
        self
    }
    pub fn node_ref(mut self, node_ref: Cage<Option<NodeId>>) -> Self {
        self.node_ref = Some(node_ref);
        self
    }
    pub fn fill(mut self, content: impl IntoChildren) -> Self {
        self.content = Some(content.into_children());
        self
    }
}
impl Widget for FocusScope {
    fn build(&mut self, scope: &Scope) -> Result {
        let configuration = self.configuration.clone();
        let active = self.active.clone();
        let id = self.id.clone();
        let flags = (
            self.loop_tab,
            self.contain_focus,
            self.auto_focus,
            self.restore_focus,
        );
        let initial_focus = self.initial_focus.clone();
        let return_focus = self.return_focus.clone();
        let semantics = scope.bond(move || {
            let mut configuration = configuration.get();
            if let Some(active) = &active {
                configuration.active = active.get();
            }
            if let Some(value) = flags.0 {
                configuration.loop_tab = value;
            }
            if let Some(value) = flags.1 {
                configuration.contain_focus = value;
            }
            if let Some(value) = flags.2 {
                configuration.auto_focus = value;
            }
            if let Some(value) = flags.3 {
                configuration.restore_focus = value;
            }
            if let Some(value) = &initial_focus {
                configuration.initial_focus = Some(value.clone());
            }
            if let Some(value) = &return_focus {
                configuration.return_focus = Some(value.clone());
            }
            Some(Box::new(Semantics {
                id: id.clone(),
                focus_scope: Some(configuration),
                ..Default::default()
            }))
        });
        let class = self.class.clone();
        let class = scope.bond(move || format!("sapid-focus-scope {}", class.get()));
        let node_ref = self.node_ref.unwrap_or_else(|| scope.cage(None));
        glory::ui::view()
            .focus(FocusMode::Programmatic)
            .semantics(semantics)
            .class(class)
            .style(self.style.clone())
            .node_ref(node_ref)
            .fill(self.content.take().unwrap_or_else(|| ().into_children()))
            .show_in(scope)
    }
}
