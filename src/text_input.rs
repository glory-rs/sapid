// Shared builder surface without exposing switches that break a specialized
// input's invariant (multiline for Textarea, purpose for PasswordInput).
pub(crate) fn placeholder(
    scope: &glory::Scope,
    source: Option<glory::Lotus<String>>,
    multiline: bool,
) -> glory::ui::Result<Option<glory::Cage<String>>> {
    source
        .as_ref()
        .map(|source| {
            crate::composite::validated(scope, source, move |value| {
                glory::render::validate_input_placeholder(value, multiline).map_err(Into::into)
            })
        })
        .transpose()
}

macro_rules! text_input_methods {
    () => {
        pub fn theme(mut self, value: crate::Theme) -> Self {
            self.input = self.input.theme(value);
            self
        }
        pub fn size(mut self, value: crate::Size) -> Self {
            self.input = self.input.size(value);
            self
        }
        pub fn unstyled(mut self) -> Self {
            self.input = self.input.unstyled();
            self
        }
        pub fn class(mut self, value: impl Into<glory::Lotus<String>>) -> Self {
            self.input = self.input.class(value);
            self
        }
        pub fn style(mut self, value: impl Into<glory::Lotus<glory::style::Style>>) -> Self {
            self.input = self.input.style(value);
            self
        }
        pub fn name(mut self, value: impl Into<glory::Lotus<String>>) -> Self {
            self.input = self.input.name(value);
            self
        }
        pub fn enabled(mut self, value: impl Into<glory::Lotus<bool>>) -> Self {
            self.input = self.input.enabled(value);
            self
        }
        pub fn focus(mut self, value: glory::render::FocusMode) -> Self {
            self.input = self.input.focus(value);
            self
        }
        pub fn node_ref(mut self, value: glory::Cage<Option<glory::render::NodeId>>) -> Self {
            self.input = self.input.node_ref(value);
            self
        }
        pub fn field(mut self, value: crate::FieldControl) -> Self {
            self.input = self.input.field(value);
            self
        }
        pub fn required(mut self, value: impl Into<glory::Lotus<bool>>) -> Self {
            self.input = self.input.required(value);
            self
        }
        pub fn readonly(mut self, value: impl Into<glory::Lotus<bool>>) -> Self {
            self.input = self.input.readonly(value);
            self
        }
        pub fn invalid(mut self, value: impl Into<glory::Lotus<bool>>) -> Self {
            self.input = self.input.invalid(value);
            self
        }
        pub fn value(mut self, value: impl Into<glory::Lotus<String>>) -> Self {
            self.input = self.input.value(value);
            self
        }
        pub fn placeholder(mut self, value: impl Into<glory::Lotus<String>>) -> Self {
            self.input = self.input.placeholder(value);
            self
        }
        pub fn default_value(mut self, value: impl Into<String>) -> Self {
            self.input = self.input.default_value(value);
            self
        }
        pub fn on_value_change(mut self, handler: impl FnMut(String) + 'static) -> Self {
            self.input = self.input.on_value_change(handler);
            self
        }
        pub fn on_commit(mut self, handler: impl FnMut(String) + 'static) -> Self {
            self.input = self.input.on_commit(handler);
            self
        }
        pub fn on(
            mut self,
            kind: glory::render::EventKind,
            handler: impl FnMut(glory::render::Event) + 'static,
        ) -> Self {
            self.input = self.input.on(kind, handler);
            self
        }
    };
}
pub(crate) use text_input_methods;
