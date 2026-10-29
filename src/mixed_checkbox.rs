//! A three-state check control; native activation proposes a binary selection.
//! The pinned Blitz engine explicitly rejects the mixed state.

/// Complete semantic selection state, including an indeterminate group result.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckState {
    #[default]
    Unchecked,
    Checked,
    Mixed,
}
impl CheckState {
    pub(crate) fn mode(self) -> glory::render::CheckMode {
        match self {
            Self::Mixed => glory::render::CheckMode::Mixed,
            _ => glory::render::CheckMode::Checkbox,
        }
    }
    pub(crate) fn checked(self) -> bool {
        self == Self::Checked
    }
}
impl From<bool> for CheckState {
    fn from(value: bool) -> Self {
        if value {
            Self::Checked
        } else {
            Self::Unchecked
        }
    }
}

use glory::Scope;
use glory::core::Vary;
use glory::render::{Change, Event, EventKind, Patch};
use glory::ui::{Result, Widget};

use crate::appearance::{Common, common_methods, configure, observe};
use crate::state::{Dirty, Value};

pub struct MixedCheckbox {
    common: Common,
    value: Value<CheckState>,
    label: Vary<String>,
    on_change: Option<Box<dyn FnMut(CheckState)>>,
    field: Option<crate::FieldControl>,
    required: Vary<bool>,
    readonly: Vary<bool>,
    invalid: Vary<bool>,
}

pub fn mixed_checkbox() -> MixedCheckbox {
    MixedCheckbox {
        common: Common::default(),
        value: Value::default(),
        label: "".into(),
        on_change: None,
        field: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
    }
}

impl MixedCheckbox {
    common_methods!();
    pub fn field(mut self, field: crate::FieldControl) -> Self {
        self.common.theme = field.theme.clone();
        self.common.size = field.size;
        self.common.unstyled = field.unstyled;
        self.field = Some(field);
        self
    }
    pub fn required(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.required = value.into();
        self
    }
    pub fn readonly(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn invalid(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.invalid = value.into();
        self
    }

    pub fn label(mut self, value: impl Into<Vary<String>>) -> Self {
        self.label = value.into();
        self
    }

    /// A controlled source: events propose changes without writing this source.
    pub fn state(mut self, value: impl Into<Vary<CheckState>>) -> Self {
        self.value.source = Some(value.into());
        self
    }

    pub fn default_state(mut self, value: CheckState) -> Self {
        self.value.default = value;
        self
    }

    pub fn on_state_change(mut self, handler: impl FnMut(CheckState) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl Widget for MixedCheckbox {
    fn build(&mut self, scope: &Scope) -> Result {
        let control = std::mem::take(&mut self.value).mount(scope);
        let initial = control.source.try_get()?;
        let dirty = Dirty::new(scope);
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let base = self.common.theme.checkbox(self.common.size);
        let enabled = self
            .field
            .as_ref()
            .map(|field| field.enabled(scope, self.common.enabled.clone()))
            .unwrap_or_else(|| self.common.enabled.clone());
        let metadata = crate::field::control_metadata(
            scope,
            self.field.as_ref(),
            self.common.name.is_some(),
            self.required.clone(),
            self.readonly.clone(),
            self.invalid.clone(),
        );
        let mut inner = configure!(
            self.common,
            glory::ui::check(),
            scope,
            "sapid-checkbox sapid-mixed-checkbox",
            base
        )
        .label(self.label.clone())
        .mode(initial.mode())
        .checked(initial.checked())
        .enabled(enabled)
        .semantics(metadata)
        .node_ref(reference);

        let mut on_change = self.on_change.take();
        let mut handlers = self
            .common
            .handlers
            .remove(&EventKind::CheckedChanged)
            .unwrap_or_default();
        let accept = control.clone();
        let pulse = dirty.clone();
        let weak = scope.downgrade();
        let owner = scope.id();
        inner = inner.on(EventKind::CheckedChanged, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            if let Event::CheckedChanged(value) = &event {
                if let Err(error) = accept
                    .accept(&CheckState::from(*value))
                    .and_then(|()| pulse.touch())
                {
                    scope.report_error(error);
                    return;
                }
                if let Some(handler) = &mut on_change {
                    handler(CheckState::from(*value));
                }
            }
            observe(&mut handlers, owner, event);
        });
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            inner = inner.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        inner.show_in(scope)?;
        let node = reference.try_get()?.ok_or(glory::ui::Error::Gone)?;
        let weak = scope.downgrade();
        let mut previous = initial;
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let edited = dirty.take();
                let value = control.source.try_get()?;
                if edited || value != previous {
                    // An equal patch restores a rejected native toggle too.
                    scope.patch(node, Patch::CheckMode(Change::Set(value.mode())))?;
                    scope.patch(node, Patch::Checked(Change::Set(value.checked())))?;
                    previous = value;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        Ok(())
    }
}

impl Default for MixedCheckbox {
    fn default() -> Self {
        mixed_checkbox()
    }
}
