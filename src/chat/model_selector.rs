//! Application-declared model catalog, with no bundled provider or price list.
use crate::appearance::{Common, common_methods, configure};
use crate::composite::{self, Callback};
use crate::state::Value;
use glory::ui::{Result, Widget};
use glory::{Scope, Vary};
use std::rc::Rc;
pub const MAX_MODEL_CHOICES: usize = 1024;
pub const MAX_MODEL_PROVIDERS: usize = 32;
pub const MAX_MODEL_CATALOG_BYTES: usize = 128 * 1024;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRecord {
    pub key: String,
    pub provider: String,
    pub name: String,
    pub value: String,
    pub available: bool,
}
impl ModelRecord {
    pub fn new(
        key: impl Into<String>,
        provider: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        let key = key.into();
        Self {
            value: key.clone(),
            key,
            provider: provider.into(),
            name: name.into(),
            available: true,
        }
    }
}
pub struct ModelSelector {
    common: Common,
    key: String,
    models: Vec<ModelRecord>,
    overflow: bool,
    value: Value<Option<String>>,
    search: Value<String>,
    explicit_search: bool,
    open: Value<bool>,
    field: Option<crate::FieldControl>,
    form_name: Option<Vary<String>>,
    required: Vary<bool>,
    readonly: Vary<bool>,
    invalid: Vary<bool>,
    changed: Option<Box<dyn FnMut(Option<ModelRecord>)>>,
    searched: Option<Box<dyn FnMut(String)>>,
    opened: Option<Box<dyn FnMut(bool)>>,
}
pub fn model_selector(key: impl Into<String>) -> ModelSelector {
    ModelSelector {
        common: Common::default(),
        key: key.into(),
        models: Vec::new(),
        overflow: false,
        value: Value::default(),
        search: Value::default(),
        explicit_search: false,
        open: Value::default(),
        field: None,
        form_name: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
        changed: None,
        searched: None,
        opened: None,
    }
}
impl ModelSelector {
    common_methods!();
    pub fn model(mut self, value: ModelRecord) -> Self {
        if self.models.len() == MAX_MODEL_CHOICES {
            self.overflow = true
        } else {
            self.models.push(value)
        }
        self
    }
    pub fn models(mut self, values: impl IntoIterator<Item = ModelRecord>) -> Self {
        for value in values
            .into_iter()
            .take(MAX_MODEL_CHOICES - self.models.len() + 1)
        {
            self = self.model(value);
            if self.overflow {
                break;
            }
        }
        self
    }
    pub fn value(mut self, value: impl Into<Vary<Option<String>>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: Option<String>) -> Self {
        self.value.default = value;
        self
    }
    pub fn search(mut self, value: impl Into<Vary<String>>) -> Self {
        self.search.source = Some(value.into());
        self.explicit_search = true;
        self
    }
    pub fn default_search(mut self, value: impl Into<String>) -> Self {
        self.search.default = value.into();
        self.explicit_search = true;
        self
    }
    pub fn open(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.open.source = Some(value.into());
        self
    }
    pub fn default_open(mut self, value: bool) -> Self {
        self.open.default = value;
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
    pub fn form_name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.form_name = Some(value.into());
        self
    }
    pub fn field(mut self, value: crate::FieldControl) -> Self {
        self.common.theme = value.theme.clone();
        self.common.size = value.size;
        self.common.unstyled = value.unstyled;
        self.field = Some(value);
        self
    }
    /// Selection keys and business values remain distinct. Controlled keys are proposals.
    pub fn on_change(mut self, value: impl FnMut(Option<ModelRecord>) + 'static) -> Self {
        self.changed = Some(Box::new(value));
        self
    }
    pub fn on_search_change(mut self, value: impl FnMut(String) + 'static) -> Self {
        self.searched = Some(Box::new(value));
        self
    }
    pub fn on_open_change(mut self, value: impl FnMut(bool) + 'static) -> Self {
        self.opened = Some(Box::new(value));
        self
    }
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
impl Widget for ModelSelector {
    fn build(&mut self, scope: &Scope) -> Result {
        composite::id("model-selector", &self.key, "root", "root")?;
        if self.overflow {
            return Err(invalid("model selector exceeds 1024 declared choices"));
        }
        let mut keys = std::collections::BTreeSet::new();
        let mut providers = Vec::<String>::new();
        let mut bytes = 0usize;
        for model in &self.models {
            if model.key.len() > 64 || !keys.insert(model.key.as_str()) {
                return Err(invalid("invalid or duplicate model key"));
            }
            glory::render::SemanticId::new(&model.key)?;
            for text in [&model.provider, &model.name, &model.value] {
                if text.is_empty() || text.len() > 256 || text.contains(['\0', '\r', '\n']) {
                    return Err(invalid("invalid bounded model metadata"));
                }
            }
            bytes +=
                model.key.len() + model.provider.len() + model.name.len() + model.value.len();
            if bytes > MAX_MODEL_CATALOG_BYTES {
                return Err(invalid("model catalog exceeds its 128KiB byte budget"));
            }
            if !providers.iter().any(|provider| provider == &model.provider) {
                if providers.len() == MAX_MODEL_PROVIDERS {
                    return Err(invalid("model selector exceeds 32 provider groups"));
                }
                providers.push(model.provider.clone());
            }
        }
        if self.common.name.is_none() && self.field.is_none() {
            self.common.name = Some("Choose model".into())
        }
        let mut selector = configure!(
            self.common,
            crate::combobox(self.key.clone()),
            scope,
            "sapid-model-selector",
            glory::style::Style::new()
        )
        .theme(self.common.theme.clone())
        .size(self.common.size)
        .enabled(self.common.enabled.clone())
        .required(self.required.clone())
        .readonly(self.readonly.clone())
        .invalid(self.invalid.clone())
        .empty_message("No matching models");
        if self.common.unstyled {
            selector = selector.unstyled()
        }
        if let Some(field) = self.field.take() {
            selector = selector.field(field)
        }
        if let Some(name) = self.form_name.take() {
            selector = selector.form_name(name)
        }
        let models = Rc::new(std::mem::take(&mut self.models));
        let control = std::mem::take(&mut self.value).mount(scope);
        let valid = models.clone();
        let accepted = composite::validated(scope, &control.source, move |key| {
            if key.as_ref().is_some_and(|key| {
                key.len() > 64 || !valid.iter().any(|model| model.key == *key)
            }) {
                Err(invalid("model selection must be a declared key"))
            } else {
                Ok(())
            }
        })?;
        // Pass the actual accepted source to the inner controller: an extra
        // binding mirror would still contain the previous key while its same-tick
        // selection finalizer checks an application-accepted proposal.
        selector = selector.value(control.source.clone());
        if let Some(search) = self.search.source.take() {
            selector = selector.search(search)
        } else if self.explicit_search {
            selector = selector.default_search(std::mem::take(&mut self.search.default))
        }
        selector = match self.open.source.take() {
            Some(value) => selector.open(value),
            None => selector.default_open(self.open.default),
        };
        for provider in providers {
            let mut group = crate::select_group(provider.clone());
            for model in models.iter().filter(|model| model.provider == provider) {
                group = group.option(
                    crate::select_option(model.key.clone(), model.name.clone())
                        .value(model.value.clone())
                        .disabled(!model.available),
                );
            }
            selector = selector.group(group);
        }
        let changed = Callback::new(self.changed.take());
        let callback = changed.clone();
        let owner = scope.id();
        let described = models.clone();
        let current = scope.bond(move || {
            accepted.with(|key| {
                match key
                    .as_ref()
                    .and_then(|key| described.iter().find(|model| model.key == *key))
                {
                    Some(model) => {
                        format!("Selected model: {} ({})", model.name, model.provider)
                    }
                    None => "No model selected".into(),
                }
            })
        });
        let weak = scope.downgrade();
        selector = selector.on_value_change(move |key| {
            if let Err(error) = control.accept(&key) {
                if let Some(scope) = weak.upgrade() {
                    scope.report_error(error)
                }
                return;
            }
            callback.emit(
                owner,
                key.and_then(|key| models.iter().find(|model| model.key == key).cloned()),
            )
        });
        if let Some(searched) = self.searched.take() {
            selector = selector.on_search_change(searched)
        }
        if let Some(opened) = self.opened.take() {
            selector = selector.on_open_change(opened)
        }
        for (kind, handlers) in std::mem::take(&mut self.common.handlers) {
            for handler in handlers {
                selector = selector.on(kind, handler)
            }
        }
        let mut description = crate::text()
            .theme(self.common.theme.clone())
            .size(self.common.size)
            .class("sapid-model-selector-current")
            .content(current);
        if self.common.unstyled {
            description = description.unstyled()
        }
        glory::ui::view()
            .class("sapid-model-selector-root")
            .style(composite::column(6.))
            .fill((selector, description))
            .show_in(scope)?;
        scope.on_cleanup(move || changed.clear())?;
        Ok(())
    }
}
