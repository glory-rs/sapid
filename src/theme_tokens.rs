//! Immutable bounded design tokens. Cloning a Theme shares this numeric table.
use crate::Size;
use glory::render::Error;
use std::sync::{Arc, OnceLock};

pub(crate) const SIZES: [Size; 5] = [
    Size::ExtraSmall,
    Size::Small,
    Size::Medium,
    Size::Large,
    Size::ExtraLarge,
];
fn index(size: Size) -> usize {
    match size {
        Size::ExtraSmall => 0,
        Size::Small => 1,
        Size::Medium => 2,
        Size::Large => 3,
        Size::ExtraLarge => 4,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TypographyRole {
    Control,
    Label,
    Description,
    Body,
    Small,
    Caption,
    Lead,
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
}
impl TypographyRole {
    pub const fn heading(level: u8) -> Option<Self> {
        match level {
            1 => Some(Self::Heading1),
            2 => Some(Self::Heading2),
            3 => Some(Self::Heading3),
            4 => Some(Self::Heading4),
            5 => Some(Self::Heading5),
            6 => Some(Self::Heading6),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypographyMetrics {
    pub font_size: f64,
    pub line_height: f64,
    pub weight: Option<f64>,
}
impl TypographyMetrics {
    fn validate(self) -> Result<(), Error> {
        if !self.font_size.is_finite()
            || !(0.0..=65_536.).contains(&self.font_size)
            || !self.line_height.is_finite()
            || !(0.0..=65_536.).contains(&self.line_height)
            || self
                .weight
                .is_some_and(|value| !value.is_finite() || !(1.0..=1000.).contains(&value))
        {
            return Err(Error::InvalidProps("invalid typography token"));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlMetrics {
    pub min_height: f64,
    pub padding_x: f64,
    pub padding_y: f64,
    /// Multiplier of Theme.radius, independent for each size.
    pub radius_scale: f64,
}
impl ControlMetrics {
    fn validate(self) -> Result<(), Error> {
        if [
            self.min_height,
            self.padding_x,
            self.padding_y,
            self.radius_scale,
        ]
        .into_iter()
        .any(|value| !value.is_finite() || !(0.0..=65_536.).contains(&value))
        {
            return Err(Error::InvalidProps("invalid control token"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq)]
struct Data {
    spacing: [f64; 5],
    controls: [ControlMetrics; 5],
    typography: [[TypographyMetrics; 5]; 13],
}
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeTokens(Arc<Data>);
impl Default for ThemeTokens {
    fn default() -> Self {
        static DEFAULT: OnceLock<Arc<Data>> = OnceLock::new();
        Self(
            DEFAULT
                .get_or_init(|| {
                    let controls = std::array::from_fn(|i| ControlMetrics {
                        min_height: [24., 30., 36., 42., 50.][i],
                        padding_x: [8., 10., 12., 14., 20.][i],
                        padding_y: [2., 4., 6., 8., 8.][i],
                        radius_scale: 1.,
                    });
                    let typography = std::array::from_fn(|role| {
                        std::array::from_fn(|i| {
                            let control_size = [11., 13., 14., 16., 20.][i];
                            let (size, factor, weight) = match role {
                                0 => (control_size, 1.55, None),
                                1 => (control_size, 1.55, Some(500.)),
                                2 => (control_size - 1., 1.5, None),
                                3 => (control_size + 2., 1.55, Some(400.)),
                                4 => (control_size, 1.55, Some(400.)),
                                5 => (control_size - 2., 1.4, Some(400.)),
                                6 => (control_size + 6., 1.55, Some(400.)),
                                heading => (
                                    [32., 28., 24., 20., 18., 16.][heading - 7] * control_size
                                        / 14.,
                                    1.3,
                                    Some(700.),
                                ),
                            };
                            TypographyMetrics {
                                font_size: size,
                                line_height: size * factor,
                                weight,
                            }
                        })
                    });
                    Arc::new(Data {
                        spacing: [4., 8., 12., 16., 24.],
                        controls,
                        typography,
                    })
                })
                .clone(),
        )
    }
}
impl ThemeTokens {
    pub fn spacing(&self, size: Size) -> f64 {
        self.0.spacing[index(size)]
    }
    pub fn control(&self, size: Size) -> ControlMetrics {
        self.0.controls[index(size)]
    }
    pub fn typography(&self, role: TypographyRole, size: Size) -> TypographyMetrics {
        self.0.typography[role as usize][index(size)]
    }
    pub fn with_spacing(mut self, size: Size, value: f64) -> Result<Self, Error> {
        if !value.is_finite() || !(0.0..=65_536.).contains(&value) {
            return Err(Error::InvalidProps("invalid spacing token"));
        }
        Arc::make_mut(&mut self.0).spacing[index(size)] = value;
        Ok(self)
    }
    pub fn with_control(mut self, size: Size, metrics: ControlMetrics) -> Result<Self, Error> {
        metrics.validate()?;
        Arc::make_mut(&mut self.0).controls[index(size)] = metrics;
        Ok(self)
    }
    pub fn with_typography(
        mut self,
        role: TypographyRole,
        size: Size,
        metrics: TypographyMetrics,
    ) -> Result<Self, Error> {
        metrics.validate()?;
        Arc::make_mut(&mut self.0).typography[role as usize][index(size)] = metrics;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_numeric_tokens_are_immutable_and_reject_invalid_values_before_installation()
    {
        let defaults = ThemeTokens::default();
        let customized = defaults
            .clone()
            .with_spacing(Size::Medium, 27.)
            .unwrap()
            .with_control(
                Size::Small,
                ControlMetrics {
                    min_height: 41.,
                    padding_x: 21.,
                    padding_y: 6.,
                    radius_scale: 2.,
                },
            )
            .unwrap()
            .with_typography(
                TypographyRole::Label,
                Size::Medium,
                TypographyMetrics {
                    font_size: 18.,
                    line_height: 25.,
                    weight: Some(550.),
                },
            )
            .unwrap();
        assert_eq!(defaults.spacing(Size::Medium), 12.);
        assert_eq!(customized.spacing(Size::Medium), 27.);
        assert_eq!(customized.control(Size::Small).min_height, 41.);
        assert_eq!(
            customized
                .typography(TypographyRole::Label, Size::Medium)
                .font_size,
            18.
        );
        for value in [f64::NAN, f64::INFINITY, -1., 65_537.] {
            assert!(defaults.clone().with_spacing(Size::Medium, value).is_err());
            assert!(
                defaults
                    .clone()
                    .with_control(
                        Size::Medium,
                        ControlMetrics {
                            padding_x: value,
                            ..defaults.control(Size::Medium)
                        }
                    )
                    .is_err()
            );
            assert!(
                defaults
                    .clone()
                    .with_typography(
                        TypographyRole::Body,
                        Size::Medium,
                        TypographyMetrics {
                            line_height: value,
                            ..defaults.typography(TypographyRole::Body, Size::Medium)
                        }
                    )
                    .is_err()
            );
        }
        assert!(
            defaults
                .clone()
                .with_typography(
                    TypographyRole::Body,
                    Size::Medium,
                    TypographyMetrics {
                        weight: Some(1001.),
                        ..defaults.typography(TypographyRole::Body, Size::Medium)
                    }
                )
                .is_err()
        );
    }
}
