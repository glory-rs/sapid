use glory::render::{EventKind, NodeId, Semantics};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};

use crate::appearance::{Common, common_methods, configure, observe};
use crate::composite::{self, ContentPolicy, Factory, Panel};
use crate::state::Value;

pub struct Collapsible {
    common: Common,
    key: String,
    label: Lotus<String>,
    open: Value<bool>,
    on_change: Option<Box<dyn FnMut(bool)>>,
    content: Option<Factory>,
    policy: ContentPolicy,
    trigger: Option<Cage<Option<NodeId>>>,
    panel: Option<Cage<Option<NodeId>>>,
}
pub fn collapsible(key: impl Into<String>) -> Collapsible {
    Collapsible {
        common: Common::default(),
        key: key.into(),
        label: "Details".into(),
        open: Value::default(),
        on_change: None,
        content: None,
        policy: ContentPolicy::Unmount,
        trigger: None,
        panel: None,
    }
}
impl Collapsible {
    common_methods!();
    pub fn label(mut self, label: impl Into<Lotus<String>>) -> Self {
        self.label = label.into();
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
    pub fn on_open_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }
    pub fn content<W: Widget>(mut self, mut factory: impl FnMut() -> W + 'static) -> Self {
        self.content = Some(Box::new(move || Box::new(factory())));
        self
    }
    pub fn content_policy(mut self, policy: ContentPolicy) -> Self {
        self.policy = policy;
        self
    }
    pub fn trigger_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
        self.trigger = Some(reference);
        self
    }
    pub fn panel_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
        self.panel = Some(reference);
        self
    }
}
impl Widget for Collapsible {
    fn build(&mut self, scope: &Scope) -> Result {
        let root_id = composite::id("c", &self.key, "root", "root")?;
        let trigger_id = composite::id("c", &self.key, "trigger", "root")?;
        let panel_id = composite::id("c", &self.key, "panel", "root")?;
        let control = std::mem::take(&mut self.open).mount(scope);
        let panel_open = control.source.clone();
        let trigger_ref = self.trigger.unwrap_or_else(|| scope.cage(None));
        let open = control.source.clone();
        let id = trigger_id.clone();
        let target = panel_id.clone();
        let mut trigger = glory::ui::button()
            .class("sapid-collapsible-trigger")
            .label(self.label.clone())
            .enabled(self.common.enabled.clone())
            .node_ref(trigger_ref)
            .semantics(scope.bond(move || {
                composite::metadata(Semantics {
                    id: Some(id.clone()),
                    controls: Some(target.clone()),
                    expanded: Some(open.get()),
                    ..Default::default()
                })
            }));
        if !self.common.unstyled {
            trigger = trigger.style(self.common.theme.control(self.common.size));
        }
        let mut on_change = self.on_change.take();
        let mut observers = self
            .common
            .handlers
            .remove(&EventKind::Activate)
            .unwrap_or_default();
        let weak = scope.downgrade();
        let owner = scope.id();
        trigger = trigger.on(EventKind::Activate, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let next = !control.source.try_get()?;
                control.accept(&next)?;
                if let Some(callback) = &mut on_change {
                    callback(next);
                }
                observe(&mut observers, owner, event);
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        let panel = Panel {
            id: panel_id,
            label: trigger_id,
            role: None,
            open: panel_open,
            trigger: trigger_ref,
            restore: None,
            reference: self.panel,
            factory: self.content.take(),
            policy: self.policy,
            class: "sapid-collapsible-panel",
            unstyled: self.common.unstyled,
        };
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-collapsible",
            composite::column(8.0)
        )
        .semantics(composite::metadata(Semantics {
            id: Some(root_id),
            ..Default::default()
        }))
        .fill((trigger, panel));
        for (kind, mut observers) in std::mem::take(&mut self.common.handlers) {
            root = root.on(kind, move |event| observe(&mut observers, owner, event));
        }
        root.show_in(scope)
    }
}
