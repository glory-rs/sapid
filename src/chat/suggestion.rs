//! Explicit application actions. Choosing never sends a prompt or starts a service.
use crate::appearance::{Common, common_methods, configure};
use crate::composite::{self, Callback};
use glory::Scope;
use glory::render::{Direction, FocusMode, Orientation};
use glory::ui::{Result, Widget};
pub const MAX_SUGGESTIONS: usize = 128;
pub const MAX_SUGGESTION_LIST_BYTES: usize = 128 * 1024;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuggestionChoice {
    pub key: String,
    pub label: String,
    pub value: String,
    pub source: Option<String>,
}
pub struct Suggestion {
    common: Common,
    choice: SuggestionChoice,
    choose: Option<Box<dyn FnMut(SuggestionChoice)>>,
}
pub fn suggestion(key: impl Into<String>) -> Suggestion {
    let key = key.into();
    Suggestion {
        common: Common::default(),
        choice: SuggestionChoice {
            label: key.clone(),
            value: key.clone(),
            key,
            source: None,
        },
        choose: None,
    }
}
impl Suggestion {
    common_methods!();
    pub fn label(mut self, value: impl Into<String>) -> Self {
        self.choice.label = value.into();
        self
    }
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.choice.value = value.into();
        self
    }
    pub fn source(mut self, value: impl Into<String>) -> Self {
        self.choice.source = Some(value.into());
        self
    }
    pub fn on_choose(mut self, value: impl FnMut(SuggestionChoice) + 'static) -> Self {
        self.choose = Some(Box::new(value));
        self
    }
    fn validate(&self) -> Result {
        glory::render::SemanticId::new(&self.choice.key)?;
        if self.choice.key.len() > 64
            || !text(&self.choice.label, 256)
            || self.choice.value.len() > 8192
            || self
                .choice
                .source
                .as_ref()
                .is_some_and(|source| !text(source, 128))
        {
            return Err(invalid("suggestion exceeds its finite payload budget"));
        }
        Ok(())
    }
    fn button(
        &mut self,
        scope: &Scope,
        group: Option<Callback<SuggestionChoice>>,
    ) -> Result<crate::Button> {
        self.validate()?;
        let choice = self.choice.clone();
        let displayed = match &choice.source {
            Some(source) => format!("{} · {}", choice.label, source),
            None => choice.label.clone(),
        };
        let choose = Callback::new(self.choose.take());
        let callback = choose.clone();
        let owner = scope.id();
        let mut button = configure!(
            self.common,
            crate::button(),
            scope,
            "sapid-suggestion",
            glory::style::Style::new()
        )
        .theme(self.common.theme.clone())
        .size(self.common.size)
        .enabled(self.common.enabled.clone())
        .label(displayed)
        .on_press(move || {
            callback.emit(owner, choice.clone());
            if let Some(group) = &group {
                group.emit(owner, choice.clone())
            }
        });
        if self.common.unstyled {
            button = button.unstyled()
        }
        for (kind, handlers) in std::mem::take(&mut self.common.handlers) {
            for handler in handlers {
                button = button.on(kind, handler)
            }
        }
        scope.on_cleanup(move || choose.clear())?;
        Ok(button)
    }
}
impl Widget for Suggestion {
    fn build(&mut self, scope: &Scope) -> Result {
        self.button(scope, None)?.show_in(scope)
    }
}
fn text(value: &str, limit: usize) -> bool {
    !value.trim().is_empty() && value.len() <= limit && !value.contains(['\0', '\r', '\n'])
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
pub struct SuggestionGroup {
    common: Common,
    key: String,
    items: Vec<Suggestion>,
    overflow: bool,
    direction: Direction,
    orientation: Orientation,
    looping: bool,
    choose: Option<Box<dyn FnMut(SuggestionChoice)>>,
}
pub fn suggestion_group(key: impl Into<String>) -> SuggestionGroup {
    SuggestionGroup {
        common: Common::default(),
        key: key.into(),
        items: Vec::new(),
        overflow: false,
        direction: Direction::Ltr,
        orientation: Orientation::Horizontal,
        looping: true,
        choose: None,
    }
}
impl SuggestionGroup {
    common_methods!();
    pub fn suggestion(mut self, value: Suggestion) -> Self {
        if self.items.len() == MAX_SUGGESTIONS {
            self.overflow = true
        } else {
            self.items.push(value)
        }
        self
    }
    pub fn suggestions(mut self, values: impl IntoIterator<Item = Suggestion>) -> Self {
        for value in values
            .into_iter()
            .take(MAX_SUGGESTIONS - self.items.len() + 1)
        {
            self = self.suggestion(value);
            if self.overflow {
                break;
            }
        }
        self
    }
    pub fn direction(mut self, value: Direction) -> Self {
        self.direction = value;
        self
    }
    pub fn orientation(mut self, value: Orientation) -> Self {
        self.orientation = value;
        self
    }
    pub fn loop_focus(mut self, value: bool) -> Self {
        self.looping = value;
        self
    }
    pub fn on_choose(mut self, value: impl FnMut(SuggestionChoice) + 'static) -> Self {
        self.choose = Some(Box::new(value));
        self
    }
}
impl Widget for SuggestionGroup {
    fn build(&mut self, scope: &Scope) -> Result {
        composite::id("suggestions", &self.key, "root", "root")?;
        if self.items.is_empty()
            || self.overflow
            || self.common.focus != FocusMode::Auto
            || self
                .items
                .iter()
                .any(|item| item.common.focus != FocusMode::Auto)
        {
            return Err(invalid(
                "suggestion group requires 1..128 actions and owns its roving entry",
            ));
        }
        let mut keys = std::collections::BTreeSet::new();
        let mut bytes = 0usize;
        for item in &self.items {
            item.validate()?;
            if !keys.insert(item.choice.key.as_str()) {
                return Err(invalid("duplicate suggestion key"));
            }
            bytes += item.choice.key.len()
                + item.choice.label.len()
                + item.choice.value.len()
                + item.choice.source.as_ref().map_or(0, String::len);
            if bytes > MAX_SUGGESTION_LIST_BYTES {
                return Err(invalid(
                    "suggestion list exceeds its 128KiB declaration budget",
                ));
            }
        }
        let callback = Callback::new(self.choose.take());
        let mut group = crate::toolbar_group("suggestions");
        for mut item in self.items.drain(..) {
            // Toolbar owns group appearance; apply it before producing nested classes too.
            item.common.theme = self.common.theme.clone();
            item.common.size = self.common.size;
            item.common.unstyled |= self.common.unstyled;
            group = group.button(item.button(scope, Some(callback.clone()))?);
        }
        let (class, style) =
            self.common
                .appearance(scope, "sapid-suggestions", glory::style::Style::new());
        let mut toolbar = crate::toolbar(self.key.clone())
            .theme(self.common.theme.clone())
            .size(self.common.size)
            .class(class)
            .style(style)
            .name(
                self.common
                    .name
                    .take()
                    .unwrap_or_else(|| "Suggestions".into()),
            )
            .enabled(self.common.enabled.clone())
            .direction(self.direction)
            .orientation(self.orientation)
            .loop_focus(self.looping)
            .group(group);
        if self.common.unstyled {
            toolbar = toolbar.unstyled()
        }
        if let Some(reference) = self.common.node_ref {
            toolbar = toolbar.node_ref(reference)
        }
        for (kind, handlers) in std::mem::take(&mut self.common.handlers) {
            for handler in handlers {
                toolbar = toolbar.on(kind, handler)
            }
        }
        toolbar.show_in(scope)?;
        scope.on_cleanup(move || callback.clear())?;
        Ok(())
    }
}
