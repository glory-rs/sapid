//! Readonly finite wall-clock display.
use crate::appearance::{Common, common_methods, configure, observe};
use crate::composite;
use crate::{LocalTime, TimeFormat};
use glory::style::Style;
use glory::ui::{Result, Widget};
use glory::{Scope, Vary};
pub struct TimeValue {
    common: Common,
    value: Vary<Option<LocalTime>>,
    format: TimeFormat,
    empty: Vary<String>,
}
pub fn time_value(value: impl Into<Vary<Option<LocalTime>>>) -> TimeValue {
    TimeValue {
        common: Common::default(),
        value: value.into(),
        format: TimeFormat::default(),
        empty: "—".into(),
    }
}
impl TimeValue {
    common_methods!();
    pub fn format(mut self, format: TimeFormat) -> Self {
        self.format = format;
        self
    }
    pub fn empty_label(mut self, label: impl Into<Vary<String>>) -> Self {
        self.empty = label.into();
        self
    }
}
impl Widget for TimeValue {
    fn build(&mut self, scope: &Scope) -> Result {
        let format = self.format;
        let value = composite::validated(scope, &self.value, move |v| {
            if v.is_none_or(|v| format.validate(v).is_ok()) {
                Ok(())
            } else {
                Err(glory::render::Error::InvalidProps("time value loses precision").into())
            }
        })?;
        let empty = composite::validated(scope, &self.empty, |s| {
            if s.len() <= 64 && !s.contains(['\0', '\r', '\n']) {
                Ok(())
            } else {
                Err(
                    glory::render::Error::InvalidProps("invalid finite empty time label")
                        .into(),
                )
            }
        })?;
        let base = Style {
            font_size: self.common.theme.label(self.common.size).font_size,
            color: Some(self.common.theme.foreground),
            ..Style::new()
        };
        let mut widget = configure!(
            self.common,
            glory::ui::text(),
            scope,
            "sapid-time-value",
            base
        )
        .content(scope.bond(move || {
            value.with(|v| {
                v.map(|v| format.format(v).unwrap())
                    .unwrap_or_else(|| empty.get())
            })
        }));
        let owner = scope.id();
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            widget = widget.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        widget.show_in(scope)
    }
}
