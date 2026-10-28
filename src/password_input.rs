use glory::render::Purpose;
use glory::ui::{Result, Widget};
use glory::{Scope, Vary};

use crate::state::Value;
use crate::text_input::text_input_methods;

/// A single input identity with an explicitly named visibility action.
/// Revealing does not copy its value or write a new input revision.
pub struct PasswordInput {
    input: crate::TextField,
    visible: Value<bool>,
    on_visibility: Option<Box<dyn FnMut(bool)>>,
    show_label: String,
    hide_label: String,
}
pub fn password_input() -> PasswordInput {
    let mut input = crate::text_field();
    input.widget_class = "sapid-text-field sapid-password-input";
    PasswordInput {
        input,
        visible: Value::default(),
        on_visibility: None,
        show_label: "Show password".into(),
        hide_label: "Hide password".into(),
    }
}
impl PasswordInput {
    text_input_methods!();
    pub fn visible(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.visible.source = Some(value.into());
        self
    }
    pub fn default_visible(mut self, value: bool) -> Self {
        self.visible.default = value;
        self
    }
    pub fn on_visibility_change(mut self, handler: impl FnMut(bool) + 'static) -> Self {
        self.on_visibility = Some(Box::new(handler));
        self
    }
    /// Localized action names must both be nonempty.
    pub fn visibility_labels(
        mut self,
        show: impl Into<String>,
        hide: impl Into<String>,
    ) -> Self {
        self.show_label = show.into();
        self.hide_label = hide.into();
        self
    }
}
impl Widget for PasswordInput {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.show_label.trim().is_empty() || self.hide_label.trim().is_empty() {
            return Err(glory::render::Error::InvalidProps(
                "password visibility actions require names",
            )
            .into());
        }
        let control = std::mem::take(&mut self.visible).mount(scope);
        let source = control.source.clone();
        let purpose = scope.bond(move || {
            if source.get() {
                Purpose::Text
            } else {
                Purpose::Password
            }
        });
        let source = control.source.clone();
        let show = self.show_label.clone();
        let hide = self.hide_label.clone();
        let label = scope.bond(move || {
            if source.get() {
                hide.clone()
            } else {
                show.clone()
            }
        });
        let common = &self.input.common;
        let enabled = self
            .input
            .field
            .as_ref()
            .map(|field| field.enabled(scope, common.enabled.clone()))
            .unwrap_or_else(|| common.enabled.clone());
        let theme = common.theme.clone();
        let size = common.size;
        let unstyled = common.unstyled;
        let weak = scope.downgrade();
        let mut callback = self.on_visibility.take();
        let mut toggle = crate::button()
            .theme(theme.clone())
            .size(size)
            .variant(crate::ButtonVariant::Secondary)
            .class("sapid-password-toggle")
            .label(label)
            .enabled(enabled)
            .on_press(move || {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    let proposal = !control.source.try_get()?;
                    control.accept(&proposal)?;
                    if let Some(callback) = &mut callback {
                        callback(proposal);
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                }
            });
        if unstyled {
            toggle = toggle.unstyled();
        }
        let input = std::mem::replace(&mut self.input, crate::text_field()).purpose(purpose);
        let mut group = crate::input_group()
            .theme(theme)
            .size(size)
            .class("sapid-password-group")
            .input(input)
            .end(toggle);
        if unstyled {
            group = group.unstyled();
        }
        group.build(scope)
    }
}
