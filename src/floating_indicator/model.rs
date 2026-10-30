use glory::render::{GraphicPoint, PathCommand, Rect};
use glory::ui::Result;
pub const MAX_FLOATING_TARGETS: usize = 128;
pub const MAX_FLOATING_EXTENT: f64 = 1_000_000.;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    pub width: f64,
    pub height: f64,
    pub target: Rect,
}
pub fn geometry(area: Rect, target: Rect) -> Result<Option<Geometry>> {
    for rect in [area, target] {
        if ![rect.x, rect.y, rect.width, rect.height]
            .into_iter()
            .all(f64::is_finite)
            || rect.width < 0.
            || rect.height < 0.
        {
            return Err(
                glory::render::Error::InvalidResponse("invalid floating indicator bounds").into(),
            );
        }
    }
    if area.width == 0. || area.height == 0. || target.width == 0. || target.height == 0. {
        return Ok(None);
    }
    let target = Rect {
        x: target.x - area.x,
        y: target.y - area.y,
        ..target
    };
    if area.width > MAX_FLOATING_EXTENT
        || area.height > MAX_FLOATING_EXTENT
        || [
            target.x,
            target.y,
            target.x + target.width,
            target.y + target.height,
        ]
        .into_iter()
        .any(|value| !value.is_finite() || value.abs() > MAX_FLOATING_EXTENT)
    {
        return Err(glory::render::Error::InvalidProps(
            "floating indicator local geometry exceeds +/-1000000 pixels",
        )
        .into());
    }
    Ok(Some(Geometry {
        width: area.width,
        height: area.height,
        target,
    }))
}
pub fn rectangle(rect: Rect) -> Vec<PathCommand> {
    vec![
        PathCommand::Move(GraphicPoint::new(rect.x, rect.y)),
        PathCommand::Line(GraphicPoint::new(rect.x + rect.width, rect.y)),
        PathCommand::Line(GraphicPoint::new(rect.x + rect.width, rect.y + rect.height)),
        PathCommand::Line(GraphicPoint::new(rect.x, rect.y + rect.height)),
        PathCommand::Close,
    ]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn viewport_origins_cancel_while_scrolled_targets_keep_their_real_local_offsets() {
        let area = Rect {
            x: 1_500_000.,
            y: -200.,
            width: 400.,
            height: 120.,
        };
        let target = Rect {
            x: 1_500_100.,
            y: -180.,
            width: 80.,
            height: 40.,
        };
        let result = geometry(area, target).unwrap().unwrap();
        assert_eq!(
            result.target,
            Rect {
                x: 100.,
                y: 20.,
                width: 80.,
                height: 40.
            }
        );
        let shifted = geometry(
            area,
            Rect {
                x: area.x - 30.,
                ..target
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(shifted.target.x, -30.);
        assert_eq!(rectangle(result.target).len(), 5);
    }
    #[test]
    fn zero_boxes_hide_and_every_nonfinite_or_unrepresentable_local_box_is_rejected() {
        let area = Rect {
            x: 0.,
            y: 0.,
            width: 100.,
            height: 100.,
        };
        assert!(
            geometry(Rect { width: 0., ..area }, area)
                .unwrap()
                .is_none()
        );
        for value in [f64::NAN, f64::INFINITY, -1.] {
            assert!(
                geometry(
                    area,
                    Rect {
                        width: value,
                        ..area
                    }
                )
                .is_err()
            );
        }
        assert!(
            geometry(
                area,
                Rect {
                    x: MAX_FLOATING_EXTENT,
                    width: 1.,
                    ..area
                }
            )
            .is_err()
        );
        assert!(
            geometry(
                Rect {
                    width: MAX_FLOATING_EXTENT + 1.,
                    ..area
                },
                area
            )
            .is_err()
        );
    }
}
