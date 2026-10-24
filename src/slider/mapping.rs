use glory::render::{Direction, Error, Orientation, RangeCommand, RangeValue, Rect};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Mapping {
    #[default]
    Linear,
    Angle,
}
impl Mapping {
    pub fn validate(self, range: RangeValue, step: f64, page: f64) -> Result<(), Error> {
        if self == Self::Angle
            && (range.min != 0. || range.max != 360. || step >= 360. || page >= 360.)
        {
            return Err(Error::InvalidProps(
                "angle slider requires 0..=360 degrees and steps below a full turn",
            ));
        }
        Ok(())
    }
    pub fn point(
        self,
        rect: Rect,
        x: f64,
        y: f64,
        axis: Orientation,
        direction: Direction,
        range: RangeValue,
    ) -> Result<Option<f64>, Error> {
        if [rect.x, rect.y, rect.width, rect.height, x, y]
            .iter()
            .any(|v| !v.is_finite())
        {
            return Err(Error::InvalidProps("slider geometry must be finite"));
        }
        if self == Self::Angle {
            if rect.width <= 0. || rect.height <= 0. {
                return Err(Error::InvalidProps(
                    "angle slider needs positive measurable bounds",
                ));
            }
            let center = (rect.x + rect.width / 2., rect.y + rect.height / 2.);
            if center.0.is_finite() && center.1.is_finite() && x == center.0 && y == center.1 {
                return Ok(None);
            }
            let scale = [
                rect.x.abs(),
                rect.y.abs(),
                rect.width,
                rect.height,
                x.abs(),
                y.abs(),
                1.,
            ]
            .into_iter()
            .fold(1., f64::max);
            let dx = x / scale - rect.x / scale - rect.width / scale / 2.;
            let dy = y / scale - rect.y / scale - rect.height / scale / 2.;
            if dx == 0. && dy == 0. {
                return Ok(None);
            }
            let angle = dx.atan2(-dy).to_degrees().rem_euclid(360.);
            return Ok(Some(if angle == 0. { 0. } else { angle }));
        }
        let (point, start, extent) = if axis == Orientation::Horizontal {
            (x, rect.x, rect.width)
        } else {
            (y, rect.y, rect.height)
        };
        if extent <= 0. {
            return Err(Error::InvalidProps(
                "slider needs positive measurable bounds",
            ));
        }
        let mut fraction = ((point - start) / extent).clamp(0., 1.);
        if axis == Orientation::Vertical || direction == Direction::Rtl {
            fraction = 1. - fraction
        }
        Ok(Some(range.min + (range.max - range.min) * fraction))
    }
    pub fn key(
        self,
        current: f64,
        command: RangeCommand,
        range: RangeValue,
        step: f64,
        page: f64,
    ) -> f64 {
        let raw = match command {
            RangeCommand::Decrease => current - step,
            RangeCommand::Increase => current + step,
            RangeCommand::PageDecrease => current - page,
            RangeCommand::PageIncrease => current + page,
            RangeCommand::Minimum => range.min,
            RangeCommand::Maximum => range.max,
            RangeCommand::Set(value) => value,
        };
        if self == Self::Angle && (raw < range.min || raw > range.max) {
            let value = raw.rem_euclid(360.);
            if value == 0. { 0. } else { value }
        } else {
            raw
        }
    }
    pub fn retains_increment(self, command: RangeCommand) -> bool {
        self == Self::Angle
            && matches!(
                command,
                RangeCommand::Increase
                    | RangeCommand::Decrease
                    | RangeCommand::PageIncrease
                    | RangeCommand::PageDecrease
            )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn range() -> RangeValue {
        RangeValue {
            min: 0.,
            max: 360.,
            value: 0.,
        }
    }
    #[test]
    fn atan2_uses_both_axes_top_zero_clockwise_and_ignores_the_undefined_center() {
        let rect = Rect {
            x: 100.,
            y: 200.,
            width: 200.,
            height: 100.,
        };
        for ((x, y), angle) in [
            ((200., 200.), 0.),
            ((300., 250.), 90.),
            ((200., 300.), 180.),
            ((100., 250.), 270.),
        ] {
            let actual = Mapping::Angle
                .point(rect, x, y, Orientation::Horizontal, Direction::Rtl, range())
                .unwrap()
                .unwrap();
            assert!((actual - angle).abs() < 1e-10, "{actual}/{angle}");
        }
        assert!(
            Mapping::Angle
                .point(
                    rect,
                    200.,
                    250.,
                    Orientation::Horizontal,
                    Direction::Ltr,
                    range()
                )
                .unwrap()
                .is_none()
        );
    }
    #[test]
    fn angular_keys_preserve_endpoints_then_wrap_and_keep_nondivisor_step_progress() {
        let r = range();
        let map = Mapping::Angle;
        assert_eq!(map.key(359., RangeCommand::Increase, r, 1., 15.), 360.);
        assert_eq!(map.key(360., RangeCommand::Increase, r, 1., 15.), 1.);
        assert_eq!(map.key(0., RangeCommand::Decrease, r, 1., 15.), 359.);
        assert_eq!(map.key(359., RangeCommand::PageIncrease, r, 1., 15.), 14.);
        assert_eq!(map.key(270., RangeCommand::Increase, r, 270., 15.), 180.);
        assert_eq!(map.key(120., RangeCommand::Maximum, r, 1., 15.), 360.);
        assert!(map.validate(r, 360., 15.).is_err());
        assert!(map.retains_increment(RangeCommand::Increase));
        assert!(!map.retains_increment(RangeCommand::Maximum));
    }
    #[test]
    fn linear_mapping_keeps_axis_rtl_and_unclamped_key_proposals_for_the_shared_snapper() {
        let rect = Rect {
            x: 10.,
            y: 20.,
            width: 100.,
            height: 200.,
        };
        let r = RangeValue {
            min: 0.,
            max: 100.,
            value: 50.,
        };
        assert_eq!(
            Mapping::Linear
                .point(rect, 35., 70., Orientation::Horizontal, Direction::Ltr, r)
                .unwrap(),
            Some(25.)
        );
        assert_eq!(
            Mapping::Linear
                .point(rect, 35., 70., Orientation::Horizontal, Direction::Rtl, r)
                .unwrap(),
            Some(75.)
        );
        assert_eq!(
            Mapping::Linear
                .point(rect, 35., 70., Orientation::Vertical, Direction::Ltr, r)
                .unwrap(),
            Some(75.)
        );
        assert_eq!(
            Mapping::Linear.key(100., RangeCommand::Increase, r, 1., 10.),
            101.
        );
    }
}
