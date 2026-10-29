//! Numeric reels with one stable accessible formatted value.
use crate::appearance::{Common, configure, observe};
use crate::composite;
use glory::render::{
    Event, EventKind, NodeId, Semantics, TranslationKind, TranslationMotion, TranslationOffset,
};
use glory::style::{Display, Family, Length, Overflow, PointerEvents, Style, WhiteSpace};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};
pub const MAX_ROLLING_NUMBER_DIGITS: usize = 24;
pub const MAX_ROLLING_NUMBER_CHARACTERS: usize = 64;
pub struct RollingNumber {
    common: Common,
    value: Vary<f64>,
    locale: crate::NumberLocale,
    format: crate::NumberFormat,
    min: Option<u8>,
    max: Option<u8>,
    grouping: bool,
    duration: Vary<u16>,
    paused: Vary<bool>,
    animated: Vary<bool>,
}
pub fn rolling_number() -> RollingNumber {
    RollingNumber {
        common: Common::default(),
        value: 0f64.into(),
        locale: crate::NumberLocale::EnUs,
        format: crate::NumberFormat::Decimal,
        min: None,
        max: None,
        grouping: true,
        duration: 500.into(),
        paused: false.into(),
        animated: true.into(),
    }
}
impl RollingNumber {
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn size(mut self, value: crate::Size) -> Self {
        self.common.size = value;
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn class(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Vary<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
    pub fn value(mut self, value: impl Into<Vary<f64>>) -> Self {
        self.value = value.into();
        self
    }
    pub fn locale(mut self, value: crate::NumberLocale) -> Self {
        self.locale = value;
        self
    }
    pub fn format(mut self, value: crate::NumberFormat) -> Self {
        self.format = value;
        self
    }
    pub fn min_fraction_digits(mut self, value: u8) -> Self {
        self.min = Some(value);
        self
    }
    pub fn max_fraction_digits(mut self, value: u8) -> Self {
        self.max = Some(value);
        self
    }
    pub fn grouping(mut self, value: bool) -> Self {
        self.grouping = value;
        self
    }
    pub fn duration_millis(mut self, value: impl Into<Vary<u16>>) -> Self {
        self.duration = value.into();
        self
    }
    pub fn paused(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.paused = value.into();
        self
    }
    pub fn animated(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.animated = value.into();
        self
    }
    pub fn on(mut self, kind: EventKind, callback: impl FnMut(Event) + 'static) -> Self {
        self.common
            .handlers
            .entry(kind)
            .or_default()
            .push(Box::new(callback));
        self
    }
}
fn format(value: f64, config: crate::number_formatter::Config) -> Result<String> {
    let value = crate::number_formatter::format_number(value, config)?;
    if value.chars().count() > MAX_ROLLING_NUMBER_CHARACTERS
        || value.bytes().filter(u8::is_ascii_digit).count() > MAX_ROLLING_NUMBER_DIGITS
    {
        return Err(glory::render::Error::InvalidProps(
            "rolling number exceeds 64 characters or 24 numeric digits",
        )
        .into());
    }
    Ok(value)
}
impl Widget for RollingNumber {
    fn build(&mut self, scope: &Scope) -> Result {
        let config = crate::number_formatter::config(
            self.locale,
            self.format,
            self.min,
            self.max,
            self.grouping,
        )?;
        let initial = format(self.value.try_get()?, config)?;
        let formatted = scope.cage(initial);
        let value = self.value.clone();
        let weak = scope.downgrade();
        scope.structural(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let next = format(value.try_get()?, config)?;
                if glory::core::untracked(|| formatted.try_with(|old| old != &next))? {
                    formatted.try_revise(|old| *old = next)?;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        let duration = composite::validated(scope, &self.duration, |value| {
            if *value <= 2000 {
                Ok(())
            } else {
                Err(glory::render::Error::InvalidProps(
                    "rolling duration must be in 0..=2000 milliseconds",
                )
                .into())
            }
        })?;
        let paused = self.paused.clone();
        let animated = self.animated.clone();
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let owner = scope.id();
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-rolling-number",
            self.common.theme.label(self.common.size)
        )
        .node_ref(reference);
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)?;
        let area = scope.child_in(reference.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
        crate::visually_hidden().content(formatted).show_in(&area)?;
        let line = 24.;
        let keys = scope.bond(move || {
            formatted.with(|value| (0..value.chars().count()).collect::<Vec<_>>())
        });
        glory::ui::view()
            .class("sapid-rolling-number-visual")
            .semantics(Some(Box::new(Semantics {
                inert: true,
                ..Default::default()
            })))
            .style(Style {
                display: Some(Display::InlineFlex),
                pointer_events: Some(PointerEvents::None),
                font_family: Some(vec![Family::Mono]),
                white_space: Some(WhiteSpace::NoWrap),
                ..Style::new()
            })
            .fill(
                glory::ui::each(keys)
                    .key(move |index| *index)
                    .children(move |entry| {
                        let index = entry.value;
                        let index = index.get();
                        let paused = paused.clone();
                        let animated = animated.clone();
                        glory::ui::widget(move |scope| {
                            let character = scope.bond(move || {
                                formatted.with(|value| value.chars().nth(index).unwrap_or(' '))
                            });
                            let digit = scope.bond(move || character.get().is_ascii_digit());
                            let paused = paused.clone();
                            let animated = animated.clone();
                            glory::ui::when(digit, move || {
                                let paused = paused.clone();
                                let animated = animated.clone();
                                glory::ui::widget(move |scope| {
                                    let paused = paused.clone();
                                    let animated = animated.clone();
                                    let metadata = scope.bond(move || {
                                        Some(Box::new(Semantics {
                                            inert: true,
                                            translation: Some(TranslationMotion {
                                                kind: TranslationKind::To {
                                                    target: TranslationOffset::new(
                                                        0.,
                                                        -line
                                                            * f64::from(
                                                                character
                                                                    .get()
                                                                    .to_digit(10)
                                                                    .unwrap_or(0),
                                                            ),
                                                    ),
                                                    duration_millis: if animated.get() {
                                                        duration.get()
                                                    } else {
                                                        0
                                                    },
                                                },
                                                paused: paused.get(),
                                            }),
                                            ..Default::default()
                                        }))
                                    });
                                    glory::ui::view()
                                        .class("sapid-rolling-number-digit")
                                        .style(
                                            Style::new()
                                                .width(Length::Ch(1.))
                                                .height(Length::Px(line))
                                                .overflow_x(Overflow::Hidden)
                                                .overflow_y(Overflow::Hidden),
                                        )
                                        .fill(
                                            glory::ui::view()
                                                .class("sapid-rolling-number-reel")
                                                .semantics(metadata)
                                                .style(Style::new().width(Length::Ch(1.)))
                                                .fill(
                                                    (0..10)
                                                        .map(move |number| {
                                                            glory::ui::text()
                                                                .content(number.to_string())
                                                                .style(Style {
                                                                    display: Some(
                                                                        Display::Block,
                                                                    ),
                                                                    line_height: Some(line),
                                                                    ..Style::new()
                                                                        .width(Length::Ch(1.))
                                                                        .height(Length::Px(
                                                                            line,
                                                                        ))
                                                                })
                                                        })
                                                        .collect::<Vec<_>>(),
                                                ),
                                        )
                                        .show_in(scope)
                                })
                            })
                            .otherwise(move || {
                                glory::ui::widget(move |scope| {
                                    glory::ui::text()
                                        .class("sapid-rolling-number-literal")
                                        .content(
                                            scope.bond(move || character.get().to_string()),
                                        )
                                        .style(Style {
                                            line_height: Some(line),
                                            ..Style::new().height(Length::Px(line))
                                        })
                                        .show_in(scope)
                                })
                            })
                            .show_in(scope)
                        })
                    }),
            )
            .show_in(&area)?;
        scope.retain(area)?;
        Ok(())
    }
}

impl Default for RollingNumber {
    fn default() -> Self {
        rolling_number()
    }
}
