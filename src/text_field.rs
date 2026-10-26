use glory::Scope;
use glory::core::Lotus;
use glory::render::{Event, EventKind, Purpose};
use glory::ui::{Result, Widget};

use crate::appearance::{Common, common_methods, configure, observe};
use crate::state::{Dirty, Value};

pub struct TextField {
    inner: Option<glory::ui::Input>,
    pub(crate) common: Common,
    pub(crate) widget_class: &'static str,
    value: Value<String>,
    on_change: Option<Box<dyn FnMut(String)>>,
    on_commit: Option<Box<dyn FnMut(String)>>,
    pub(crate) field: Option<crate::FieldControl>,
    required: Lotus<bool>,
    readonly: Lotus<bool>,
    invalid: Lotus<bool>,
    extra_error: Option<(glory::render::SemanticId, Lotus<String>)>,
    placeholder: Option<Lotus<String>>,
    multiline: bool,
}

pub fn text_field() -> TextField {
    TextField {
        inner: Some(glory::ui::input()),
        common: Common::default(),
        widget_class: "sapid-text-field",
        value: Value::default(),
        on_change: None,
        on_commit: None,
        field: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
        extra_error: None,
        placeholder: None,
        multiline: false,
    }
}

impl TextField {
    pub fn placeholder(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.placeholder = Some(value.into());
        self
    }
    pub(crate) fn additional_error(
        mut self,
        id: glory::render::SemanticId,
        message: Lotus<String>,
    ) -> Self {
        self.extra_error = Some((id, message));
        self
    }
    common_methods!();
    pub fn field(mut self, field: crate::FieldControl) -> Self {
        self.common.theme = field.theme.clone();
        self.common.size = field.size;
        self.common.unstyled = field.unstyled;
        self.field = Some(field);
        self
    }
    pub fn required(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.required = value.into();
        self
    }
    pub fn readonly(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn invalid(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.invalid = value.into();
        self
    }

    /// Controlled text. Native edits call on_value_change; the source decides
    /// which value is displayed after that host tick.
    pub fn value(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.value.source = Some(value.into());
        self
    }

    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.value.default = value.into();
        self
    }

    pub fn purpose(mut self, value: impl Into<Lotus<Purpose>>) -> Self {
        self.inner = self.inner.take().map(|inner| inner.purpose(value));
        self
    }

    pub fn multiline(mut self, value: bool) -> Self {
        self.multiline = value;
        self.inner = self.inner.take().map(|inner| inner.multiline(value));
        self
    }

    /// Includes composing edits. Observe EventKind::Composition separately when
    /// validation or submission should wait for composition to end.
    pub fn on_value_change(mut self, handler: impl FnMut(String) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Commit carries the host's final text. It updates local state and invokes
    /// this callback, without synthesizing an additional on_value_change call.
    pub fn on_commit(mut self, handler: impl FnMut(String) + 'static) -> Self {
        self.on_commit = Some(Box::new(handler));
        self
    }
}

impl Widget for TextField {
    fn build(&mut self, scope: &Scope) -> Result {
        let placeholder =
            crate::text_input::placeholder(scope, self.placeholder.take(), self.multiline)?;
        let control = std::mem::take(&mut self.value).mount(scope);
        let initial = control.source.try_get()?;
        let dirty = Dirty::new(scope);
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let base = self.common.theme.control(self.common.size);
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
        let metadata = if let Some((id, message)) = self.extra_error.take() {
            scope
                .bond(move || {
                    let mut state = metadata.get().unwrap_or_default();
                    let error = message.with(|value| !value.is_empty());
                    state.invalid |= error;
                    if error && state.error_message.is_none() {
                        state.error_message = Some(id.clone())
                    }
                    crate::composite::metadata(*state)
                })
                .into()
        } else {
            metadata
        };
        let metadata = if let Some(placeholder) = placeholder {
            scope
                .bond(move || {
                    let mut state = metadata.get().unwrap_or_default();
                    state.placeholder = Some(placeholder.get());
                    Some(state)
                })
                .into()
        } else {
            metadata
        };
        if metadata.is_reactive()
            || enabled.is_reactive()
            || !enabled.try_get()?
            || self.required.get()
            || self.readonly.get()
            || self.invalid.get()
        {
            let state = metadata.clone();
            let styles = self.common.style.clone();
            let theme = self.common.theme.clone();
            let unstyled = self.common.unstyled;
            let active = enabled.clone();
            self.common.style = scope
                .bond(move || {
                    let mut style = styles.get();
                    if !unstyled {
                        if !active.get() {
                            style.background.get_or_insert(theme.disabled);
                            style.color.get_or_insert(theme.disabled_text);
                        }
                        if let Some(state) = state.get() {
                            if state.invalid && style.border_color.is_none() {
                                style.border_color = Some(theme.danger);
                            }
                            if state.readonly && style.background.is_none() {
                                style.background = Some(theme.disabled);
                            }
                        }
                    }
                    style
                })
                .into();
        }
        let mut inner = configure!(
            self.common,
            self.inner.take().ok_or(glory::ui::Error::Gone)?,
            scope,
            self.widget_class,
            base
        )
        .value(initial.clone())
        .enabled(enabled)
        .semantics(metadata)
        .node_ref(reference);

        for (kind, mut callback) in [
            (EventKind::ValueChanged, self.on_change.take()),
            (EventKind::Commit, self.on_commit.take()),
        ] {
            let mut handlers = self.common.handlers.remove(&kind).unwrap_or_default();
            let accept = control.clone();
            let pulse = dirty.clone();
            let weak = scope.downgrade();
            let owner = scope.id();
            inner = inner.on(kind, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                if let Event::ValueChanged { value, .. } | Event::Commit { value, .. } = &event
                {
                    if let Err(error) = accept.accept(value).and_then(|()| pulse.touch()) {
                        scope.report_error(error);
                        return;
                    }
                    if let Some(handler) = &mut callback {
                        handler(value.clone());
                    }
                }
                observe(&mut handlers, owner, event);
            });
        }
        let owner = scope.id();
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
                    scope.set_input_value(node, value.clone())?;
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
