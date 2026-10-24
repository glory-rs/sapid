//! Finite viewport-window geometry, independent of a physical host.
use glory::ui::Result;

pub const MAX_WINDOW_COORDINATE: f64 = 1_000_000.;
pub const MAX_WINDOW_SIZE: f64 = 65_536.;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
impl Default for WindowBounds {
    fn default() -> Self {
        Self {
            x: 48.,
            y: 48.,
            width: 400.,
            height: 280.,
        }
    }
}
impl WindowBounds {
    pub fn validate(self) -> Result {
        if ![self.x, self.y]
            .iter()
            .all(|v| v.is_finite() && v.abs() <= MAX_WINDOW_COORDINATE)
            || ![self.width, self.height]
                .iter()
                .all(|v| v.is_finite() && (1. ..=MAX_WINDOW_SIZE).contains(v))
        {
            return Err(glory::render::Error::InvalidProps(
                "window bounds exceed finite viewport coordinates or dimensions",
            )
            .into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowLimits {
    pub min_width: f64,
    pub min_height: f64,
    pub max_width: f64,
    pub max_height: f64,
}
impl Default for WindowLimits {
    fn default() -> Self {
        Self {
            min_width: 180.,
            min_height: 100.,
            max_width: 4096.,
            max_height: 4096.,
        }
    }
}
impl WindowLimits {
    pub fn validate(self) -> Result {
        if ![
            self.min_width,
            self.min_height,
            self.max_width,
            self.max_height,
        ]
        .iter()
        .all(|v| v.is_finite() && (1. ..=MAX_WINDOW_SIZE).contains(v))
            || self.min_width > self.max_width
            || self.min_height > self.max_height
        {
            return Err(glory::render::Error::InvalidProps(
                "window limits must be finite ordered positive dimensions",
            )
            .into());
        }
        Ok(())
    }
    pub fn validate_bounds(self, bounds: WindowBounds) -> Result {
        self.validate()?;
        bounds.validate()?;
        if !(self.min_width..=self.max_width).contains(&bounds.width)
            || !(self.min_height..=self.max_height).contains(&bounds.height)
        {
            return Err(glory::render::Error::InvalidProps(
                "window bounds disagree with declared limits",
            )
            .into());
        }
        Ok(())
    }
    pub(crate) fn change(
        self,
        start: WindowBounds,
        dx: f64,
        dy: f64,
        resize: bool,
    ) -> Result<WindowBounds> {
        self.validate_bounds(start)?;
        if !dx.is_finite() || !dy.is_finite() {
            return Err(
                glory::render::Error::InvalidProps("window delta must be finite").into(),
            );
        }
        let next = if resize {
            WindowBounds {
                width: (start.width + dx).clamp(self.min_width, self.max_width),
                height: (start.height + dy).clamp(self.min_height, self.max_height),
                ..start
            }
        } else {
            WindowBounds {
                x: (start.x + dx).clamp(-MAX_WINDOW_COORDINATE, MAX_WINDOW_COORDINATE),
                y: (start.y + dy).clamp(-MAX_WINDOW_COORDINATE, MAX_WINDOW_COORDINATE),
                ..start
            }
        };
        self.validate_bounds(next)?;
        Ok(next)
    }
}

/// Real Button actions provide keyboard equivalents for the pointer caption and
/// resize grip. Movement and sizing are physical viewport directions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowCommand {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Narrower,
    Wider,
    Shorter,
    Taller,
}
impl WindowCommand {
    pub(crate) fn delta(self, step: f64) -> (f64, f64, bool) {
        match self {
            Self::MoveLeft => (-step, 0., false),
            Self::MoveRight => (step, 0., false),
            Self::MoveUp => (0., -step, false),
            Self::MoveDown => (0., step, false),
            Self::Narrower => (-step, 0., true),
            Self::Wider => (step, 0., true),
            Self::Shorter => (0., -step, true),
            Self::Taller => (0., step, true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finite_window_geometry_resizes_with_limits_and_moves_without_changing_size() {
        let limits = WindowLimits::default();
        let initial = WindowBounds::default();
        let moved = limits.change(initial, -30., 45., false).unwrap();
        assert_eq!(
            (moved.x, moved.y, moved.width, moved.height),
            (18., 93., 400., 280.)
        );
        let resized = limits.change(moved, -1000., 10000., true).unwrap();
        assert_eq!(
            (resized.x, resized.y, resized.width, resized.height),
            (18., 93., 180., 4096.)
        );
        assert!(limits.change(initial, f64::NAN, 0., false).is_err());
        assert!(
            WindowBounds {
                x: f64::INFINITY,
                ..initial
            }
            .validate()
            .is_err()
        );
        assert!(
            WindowLimits {
                min_width: 4097.,
                ..limits
            }
            .validate()
            .is_err()
        );
        assert!(
            limits
                .validate_bounds(WindowBounds {
                    width: 1.,
                    ..initial
                })
                .is_err()
        );
        for command in [
            WindowCommand::MoveLeft,
            WindowCommand::MoveRight,
            WindowCommand::MoveUp,
            WindowCommand::MoveDown,
            WindowCommand::Narrower,
            WindowCommand::Wider,
            WindowCommand::Shorter,
            WindowCommand::Taller,
        ] {
            let (dx, dy, resize) = command.delta(10.);
            limits
                .validate_bounds(limits.change(initial, dx, dy, resize).unwrap())
                .unwrap();
        }
    }
}
