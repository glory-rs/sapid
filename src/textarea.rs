use glory::Scope;
use glory::style::{FieldSizing, Length, Limit, Overflow, Style};
use glory::ui::{Result, Widget};

use crate::appearance::merge;
use crate::text_input::text_input_methods;

/// A multiline control using TextField's revision/IME and Field contracts.
/// Row sizes use the effective typed line height, padding and border widths.
pub struct Textarea {
    input: crate::TextField,
    rows: u16,
    min_rows: Option<u16>,
    max_rows: Option<u16>,
    autosize: bool,
}
pub fn textarea() -> Textarea {
    let mut input = crate::text_field().multiline(true);
    input.widget_class = "sapid-text-field sapid-textarea";
    Textarea {
        input,
        rows: 3,
        min_rows: None,
        max_rows: None,
        autosize: false,
    }
}
impl Textarea {
    #[cfg(feature = "json")]
    pub(crate) fn additional_error(
        mut self,
        id: glory::render::SemanticId,
        message: glory::Vary<String>,
    ) -> Self {
        self.input = self.input.additional_error(id, message);
        self
    }
    text_input_methods!();
    pub fn rows(mut self, value: u16) -> Self {
        self.rows = value;
        self
    }
    pub fn min_rows(mut self, value: u16) -> Self {
        self.min_rows = Some(value);
        self
    }
    pub fn max_rows(mut self, value: u16) -> Self {
        self.max_rows = Some(value);
        self
    }
    /// HTML hosts use intrinsic content sizing, including soft wrapping and
    /// later font/width changes. Blitz currently rejects this declaration.
    pub fn autosize(mut self, value: bool) -> Self {
        self.autosize = value;
        self
    }
}
impl Widget for Textarea {
    fn build(&mut self, scope: &Scope) -> Result {
        let min = self.min_rows.unwrap_or(self.rows);
        let max = self.max_rows;
        if self.rows == 0 || min == 0 || max.is_some_and(|max| max == 0 || max < min) {
            return Err(glory::render::Error::InvalidProps(
                "textarea requires positive, ordered row limits",
            )
            .into());
        }
        let common = &mut self.input.common;
        let theme = common.theme.control(common.size);
        let default_line = theme.line_height.unwrap();
        let base = if common.unstyled { Style::new() } else { theme };
        let custom = common.style.clone();
        let rows = self.rows;
        let autosize = self.autosize;
        let sizing = move || {
            let overrides = custom.get();
            let mut style = merge(base.clone(), overrides.clone());
            let line = style.line_height.unwrap_or(default_line);
            let padding = style.padding.map_or(0.0, |v| v.top + v.bottom);
            let border = style.border_width.map_or(0.0, |v| v.top + v.bottom);
            let height = |rows: u16| Length::Px(f64::from(rows) * line + padding + border);
            if overrides.width.is_none() {
                style.width = Some(Length::Percent(100.0));
            }
            if overrides.height.is_none() {
                style.height = Some(if autosize { Length::Auto } else { height(rows) });
            }
            if overrides.min_height.is_none() {
                style.min_height = Some(height(min));
            }
            if overrides.max_height.is_none() {
                style.max_height = max.map(|rows| match height(rows) {
                    Length::Px(v) => Limit::Px(v),
                    _ => unreachable!(),
                });
            }
            if overrides.overflow_y.is_none() {
                style.overflow_y = Some(Overflow::Auto);
            }
            if autosize && overrides.field_sizing.is_none() {
                style.field_sizing = Some(FieldSizing::Content);
            }
            style
        };
        common.style = if common.style.is_reactive() {
            scope.bond(sizing).into()
        } else {
            sizing().into()
        };
        self.input.build(scope)
    }
}
