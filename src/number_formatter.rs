//! Explicit finite locale formatting, with no ambient host or Intl dependency.
use glory::render::{Change, Patch};
use glory::ui::{Result, Widget};
use glory::{Lotus, Scope};

use crate::appearance::{Common, configure};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NumberLocale {
    #[default]
    EnUs,
    DeDe,
    ZhCn,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Currency {
    Usd,
    Eur,
    Cny,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NumberFormat {
    #[default]
    Decimal,
    Percent,
    Currency(Currency),
}

/// Passive numeric text. Non-finite values and percent overflow are rejected;
/// a rejected reactive update preserves the last accepted rendered text.
pub struct NumberFormatter {
    common: Common,
    value: Lotus<f64>,
    locale: NumberLocale,
    format: NumberFormat,
    min_fraction: Option<u8>,
    max_fraction: Option<u8>,
    grouping: bool,
}
pub fn number_formatter() -> NumberFormatter {
    NumberFormatter {
        common: Common::default(),
        value: 0.0.into(),
        locale: NumberLocale::EnUs,
        format: NumberFormat::Decimal,
        min_fraction: None,
        max_fraction: None,
        grouping: true,
    }
}
impl NumberFormatter {
    pub fn theme(mut self, theme: crate::Theme) -> Self {
        self.common.theme = theme;
        self
    }
    pub fn size(mut self, size: crate::Size) -> Self {
        self.common.size = size;
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn class(mut self, class: impl Into<Lotus<String>>) -> Self {
        self.common.class = class.into();
        self
    }
    pub fn style(mut self, style: impl Into<Lotus<glory::style::Style>>) -> Self {
        self.common.style = style.into();
        self
    }
    pub fn node_ref(mut self, reference: glory::Cage<Option<glory::render::NodeId>>) -> Self {
        self.common.node_ref = Some(reference);
        self
    }
    pub fn value(mut self, value: impl Into<Lotus<f64>>) -> Self {
        self.value = value.into();
        self
    }
    pub fn locale(mut self, locale: NumberLocale) -> Self {
        self.locale = locale;
        self
    }
    pub fn format(mut self, format: NumberFormat) -> Self {
        self.format = format;
        self
    }
    /// Minimum fraction digits; all fraction budgets must be within 0–18.
    pub fn min_fraction_digits(mut self, digits: u8) -> Self {
        self.min_fraction = Some(digits);
        self
    }
    /// Maximum fraction digits; values use Rust's fixed decimal rounding.
    pub fn max_fraction_digits(mut self, digits: u8) -> Self {
        self.max_fraction = Some(digits);
        self
    }
    pub fn grouping(mut self, grouping: bool) -> Self {
        self.grouping = grouping;
        self
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Config {
    locale: NumberLocale,
    format: NumberFormat,
    min: u8,
    max: u8,
    grouping: bool,
}
pub(crate) fn config(
    locale: NumberLocale,
    format: NumberFormat,
    min_fraction: Option<u8>,
    max_fraction: Option<u8>,
    grouping: bool,
) -> std::result::Result<Config, glory::render::Error> {
    let default_min = if matches!(format, NumberFormat::Currency(_)) {
        2
    } else {
        0
    };
    let min = min_fraction.unwrap_or(default_min);
    let default_max = match format {
        NumberFormat::Decimal => 3,
        NumberFormat::Percent => 0,
        NumberFormat::Currency(_) => 2,
    };
    let max = max_fraction.unwrap_or(default_max.max(min));
    if min > max || max > 18 {
        return Err(glory::render::Error::InvalidProps(
            "number fractions require 0 <= min <= max <= 18",
        ));
    }
    Ok(Config {
        locale,
        format,
        min,
        max,
        grouping,
    })
}
impl Widget for NumberFormatter {
    fn build(&mut self, scope: &Scope) -> Result {
        let config = config(
            self.locale,
            self.format,
            self.min_fraction,
            self.max_fraction,
            self.grouping,
        )?;
        let initial = format_number(self.value.try_get()?, config)?;
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        configure!(
            self.common,
            glory::ui::text(),
            scope,
            "sapid-number-formatter",
            self.common.theme.label(self.common.size)
        )
        .node_ref(reference)
        .content(initial.clone())
        .show_in(scope)?;
        if self.value.is_reactive() {
            let node = reference.try_get()?.ok_or(glory::ui::Error::Gone)?;
            let value = self.value.clone();
            let weak = scope.downgrade();
            let mut previous = initial;
            scope.binding(move || {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    let formatted = format_number(value.try_get()?, config)?;
                    if previous != formatted {
                        scope.patch(node, Patch::Text(Change::Set(formatted.clone())))?;
                        previous = formatted;
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                }
            });
        }
        Ok(())
    }
}

pub(crate) fn format_number(
    value: f64,
    config: Config,
) -> std::result::Result<String, glory::render::Error> {
    let scaled = if config.format == NumberFormat::Percent {
        value * 100.0
    } else {
        value
    };
    if !value.is_finite() || !scaled.is_finite() {
        return Err(glory::render::Error::InvalidProps(
            "number formatter requires a finite value and scaled result",
        ));
    }
    // f64's finite exponent is bounded; fixed formatting uses at most 309 integer
    // digits plus 18 fraction digits, independent of any application collection.
    let mut plain = format!("{:.*}", usize::from(config.max), scaled.abs());
    if let Some(point) = plain.find('.') {
        while plain.len() - point - 1 > usize::from(config.min) && plain.ends_with('0') {
            plain.pop();
        }
        if plain.ends_with('.') {
            plain.pop();
        }
    }
    let (whole, fraction) = plain.split_once('.').unwrap_or((&plain, ""));
    let (group, decimal) = match config.locale {
        NumberLocale::DeDe => ('.', ','),
        _ => (',', '.'),
    };
    let mut number = String::with_capacity(plain.len() + whole.len() / 3);
    for (index, digit) in whole.chars().enumerate() {
        if config.grouping && index > 0 && (whole.len() - index).is_multiple_of(3) {
            number.push(group);
        }
        number.push(digit);
    }
    if !fraction.is_empty() {
        number.push(decimal);
        number.push_str(fraction);
    }
    let negative = if scaled.is_sign_negative() { "-" } else { "" };
    Ok(match config.format {
        NumberFormat::Decimal => format!("{negative}{number}"),
        NumberFormat::Percent if config.locale == NumberLocale::DeDe => {
            format!("{negative}{number}\u{a0}%")
        }
        NumberFormat::Percent => format!("{negative}{number}%"),
        NumberFormat::Currency(currency) => {
            let symbol = match (currency, config.locale) {
                (Currency::Usd, NumberLocale::EnUs) => "$",
                (Currency::Usd, _) => "US$",
                (Currency::Eur, _) => "€",
                (Currency::Cny, NumberLocale::ZhCn) => "¥",
                (Currency::Cny, _) => "CN¥",
            };
            if config.locale == NumberLocale::DeDe {
                format!("{negative}{number}\u{a0}{symbol}")
            } else {
                format!("{negative}{symbol}{number}")
            }
        }
    })
}
