//! Standalone Tool/Subagent records reuse the same accepted trace renderer.
use super::trace_model::*;
use super::traces::{Trace, TraceAction, TraceExpansion, trace};
use crate::appearance::{Common, common_methods};
use crate::composite::{self, ContentPolicy};
use crate::state::Value;
use glory::ui::{Result, Widget};
use glory::{Scope, Vary};

struct Entry {
    key: String,
    record: Vary<TraceRecord>,
    kind: TraceKind,
    open: Value<bool>,
    changed: Option<Box<dyn FnMut(bool)>>,
    action: Option<Box<dyn FnMut(TraceAction)>>,
    action_label: Vary<String>,
    policy: ContentPolicy,
    payload_height: f64,
}
impl Entry {
    fn new(key: String, record: Vary<TraceRecord>, kind: TraceKind) -> Self {
        Self {
            key,
            record,
            kind,
            open: Value::default(),
            changed: None,
            action: None,
            action_label: "Inspect record".to_owned().into(),
            policy: ContentPolicy::Unmount,
            payload_height: 240.,
        }
    }
    fn build(&mut self, scope: &Scope, mut common: Common) -> Result {
        // Reject a large constant namespace before duplicating it for the
        // shared renderer. Move the label into that renderer's admission path
        // as well, rather than cloning an unvalidated Vary constant.
        super::traces::validate_presentation(&self.key, self.payload_height)?;
        let kind = self.kind;
        let accepted = composite::validated(scope, &self.record, move |r| {
            if r.kind() != kind || r.parent_key().is_some() {
                Err(glory::render::Error::InvalidProps(
                    "standalone trace record needs matching kind and no parent",
                )
                .into())
            } else {
                Ok(())
            }
        })?;
        let data = scope.bond(move || {
            // The admitted record is sealed and has no parent; one record is
            // always within the graph's aggregate budget.
            TraceData::new([accepted.get()]).expect("admitted standalone trace")
        });
        let open = std::mem::take(&mut self.open).mount(scope);
        let source = open.source.clone();
        let expanded = scope.bond(move || {
            TraceExpansion::new(if source.get() {
                vec![accepted.with(|r| r.key().clone())]
            } else {
                vec![]
            })
            .expect("at most one sealed trace key")
        });
        let mut changed = self.changed.take();
        let weak = scope.downgrade();
        let mut widget: Trace = trace(self.key.clone(), data)
            .expanded(expanded)
            .action_label(std::mem::replace(
                &mut self.action_label,
                String::new().into(),
            ))
            .content_policy(self.policy)
            .payload_height(self.payload_height)
            .on_expand_change(move |proposal| {
                if let Err(error) = open.accept(&proposal.open) {
                    if let Some(scope) = weak.upgrade() {
                        scope.report_error(error);
                    }
                    return;
                }
                if let Some(callback) = changed.as_mut() {
                    callback(proposal.open);
                }
            });
        if common.name.is_none() {
            common.name = Some(kind.label().to_owned().into());
        }
        if let Some(action) = self.action.take() {
            widget = widget.on_action(action);
        }
        widget.replace_common(common);
        widget.build(scope)
    }
}
macro_rules! entry_widget {
    ($name:ident, $constructor:ident, $kind:ident) => {
        pub struct $name {
            common: Common,
            entry: Entry,
        }
        pub fn $constructor(key: impl Into<String>, record: impl Into<Vary<TraceRecord>>) -> $name {
            $name {
                common: Common::default(),
                entry: Entry::new(key.into(), record.into(), TraceKind::$kind),
            }
        }
        impl $name {
            common_methods!();
            pub fn open(mut self, value: impl Into<Vary<bool>>) -> Self {
                self.entry.open.source = Some(value.into());
                self
            }
            pub fn default_open(mut self, value: bool) -> Self {
                self.entry.open.default = value;
                self
            }
            pub fn on_open_change(mut self, value: impl FnMut(bool) + 'static) -> Self {
                self.entry.changed = Some(Box::new(value));
                self
            }
            pub fn on_action(mut self, value: impl FnMut(TraceAction) + 'static) -> Self {
                self.entry.action = Some(Box::new(value));
                self
            }
            pub fn action_label(mut self, value: impl Into<Vary<String>>) -> Self {
                self.entry.action_label = value.into();
                self
            }
            pub fn content_policy(mut self, value: ContentPolicy) -> Self {
                self.entry.policy = value;
                self
            }
            pub fn payload_height(mut self, value: f64) -> Self {
                self.entry.payload_height = value;
                self
            }
        }
        impl Widget for $name {
            fn build(&mut self, scope: &Scope) -> Result {
                self.entry.build(scope, std::mem::take(&mut self.common))
            }
        }
    };
}
entry_widget!(Tool, tool, Tool);
entry_widget!(Subagent, subagent, Subagent);
