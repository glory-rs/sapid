//! Finite rating domain; kept independent of host geometry and radio controls.
pub const MAX_RATING_COUNT: u8 = 10;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RatingPrecision {
    #[default]
    Whole,
    Half,
}
impl RatingPrecision {
    pub(crate) fn denominator(self) -> u8 {
        match self {
            Self::Whole => 1,
            Self::Half => 2,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Domain {
    pub count: u8,
    pub precision: RatingPrecision,
}
impl Domain {
    pub fn validate(self) -> Result<(), &'static str> {
        if self.count == 0 || self.count > MAX_RATING_COUNT {
            Err("rating count must be 1..=10")
        } else {
            Ok(())
        }
    }
    pub fn units(self, value: f64) -> Result<u8, &'static str> {
        self.validate()?;
        let units = value * f64::from(self.precision.denominator());
        if !value.is_finite()
            || value < 0.
            || value > f64::from(self.count)
            || units.fract() != 0.
        {
            Err("rating must be an exact whole/half value inside 0..=count")
        } else {
            Ok(units as u8)
        }
    }
    pub fn value(self, units: u8) -> Option<f64> {
        if self.validate().is_err() || units > self.count * self.precision.denominator() {
            None
        } else {
            Some(f64::from(units) / f64::from(self.precision.denominator()))
        }
    }
    pub fn option_count(self, clearable: bool) -> Result<usize, &'static str> {
        self.validate()?;
        Ok(usize::from(self.count * self.precision.denominator()) + usize::from(clearable))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_whole_and_half_values_round_trip_without_quantizing_business_sources() {
        for precision in [RatingPrecision::Whole, RatingPrecision::Half] {
            let domain = Domain {
                count: 5,
                precision,
            };
            for units in 0..=5 * precision.denominator() {
                let value = domain.value(units).unwrap();
                assert_eq!(domain.units(value), Ok(units));
            }
            for value in [f64::NAN, f64::INFINITY, -0.5, 5.5, 0.25] {
                assert!(domain.units(value).is_err());
            }
        }
        assert!(
            Domain {
                count: 5,
                precision: RatingPrecision::Whole
            }
            .units(0.5)
            .is_err()
        );
    }
    #[test]
    fn count_is_validated_before_allocating_or_iterating_options() {
        for count in [0, 11, 255] {
            let domain = Domain {
                count,
                precision: RatingPrecision::Half,
            };
            assert!(domain.option_count(true).is_err());
            assert!(domain.value(0).is_none());
        }
        assert_eq!(
            Domain {
                count: 10,
                precision: RatingPrecision::Half
            }
            .option_count(true),
            Ok(21)
        );
    }
}
