//! Pure finite live-heading hierarchy and viewport selection.
use glory::render::{Rect, SemanticId};
pub const MAX_TOC_HEADINGS: usize = 256;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TocEntry {
    pub id: SemanticId,
    pub label: String,
    pub level: u8,
    pub order: u32,
}
pub(super) fn validate_label(value: &str) -> glory::ui::Result {
    if value.trim().is_empty() || value.len() > 256 || value.contains(['\0', '\r', '\n']) {
        Err(glory::render::Error::InvalidProps(
            "heading labels require 1..=256 canonical single-line bytes",
        )
        .into())
    } else {
        Ok(())
    }
}
/// Parent indices into an already ordered live registry snapshot. Skipped heading
/// levels nest directly under the nearest preceding lower level, without filler.
pub(super) fn parents(entries: &[TocEntry]) -> Vec<Option<usize>> {
    let mut stack: Vec<(u8, usize)> = Vec::with_capacity(6);
    let mut out = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        while stack.last().is_some_and(|(level, _)| *level >= entry.level) {
            stack.pop();
        }
        out.push(stack.last().map(|(_, index)| *index));
        stack.push((entry.level, index));
    }
    out
}
/// Hidden/zero-height headings do not establish a current section. Ties retain
/// the registry's explicit order. Geometry has already passed Query validation.
pub(super) fn current(
    entries: &[(SemanticId, Rect)],
    viewport: Rect,
    offset: f64,
) -> Option<SemanticId> {
    let line = viewport.y + offset;
    let mut preceding: Option<(&SemanticId, f64)> = None;
    let mut next: Option<(&SemanticId, f64)> = None;
    for (id, rect) in entries {
        if rect.height <= 0. || rect.width <= 0. {
            continue;
        }
        if rect.y <= line {
            if preceding.is_none_or(|(_, top)| rect.y > top) {
                preceding = Some((id, rect.y));
            }
        } else if next.is_none_or(|(_, top)| rect.y < top) {
            next = Some((id, rect.y));
        }
    }
    preceding.or(next).map(|(id, _)| id.clone())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn entry(index: usize, level: u8) -> TocEntry {
        TocEntry {
            id: SemanticId::new(format!("h{index}")).unwrap(),
            label: format!("Heading {index}"),
            level,
            order: index as u32,
        }
    }
    #[test]
    fn semantic_hierarchy_handles_siblings_skipped_levels_and_returning_to_roots() {
        let values = [1, 2, 4, 3, 2, 1, 6]
            .into_iter()
            .enumerate()
            .map(|(i, l)| entry(i, l))
            .collect::<Vec<_>>();
        assert_eq!(
            parents(&values),
            vec![None, Some(0), Some(1), Some(1), Some(0), None, Some(5)]
        );
    }
    #[test]
    fn viewport_selection_is_physical_not_registration_or_hash_order() {
        let values = [
            (
                entry(0, 1).id,
                Rect {
                    x: 10.,
                    y: 300.,
                    width: 100.,
                    height: 20.,
                },
            ),
            (
                entry(1, 1).id,
                Rect {
                    x: 10.,
                    y: 50.,
                    width: 100.,
                    height: 20.,
                },
            ),
            (
                entry(2, 1).id,
                Rect {
                    x: 10.,
                    y: 180.,
                    width: 100.,
                    height: 20.,
                },
            ),
        ];
        let viewport = Rect {
            x: 0.,
            y: 100.,
            width: 200.,
            height: 200.,
        };
        assert_eq!(current(&values, viewport, 0.), Some(entry(1, 1).id));
        assert_eq!(current(&values, viewport, 100.), Some(entry(2, 1).id));
    }
    #[test]
    fn hidden_and_zero_width_headings_are_excluded_and_first_visible_is_fallback() {
        let values = [
            (
                entry(0, 1).id,
                Rect {
                    x: 0.,
                    y: -10.,
                    width: 100.,
                    height: 0.,
                },
            ),
            (
                entry(1, 1).id,
                Rect {
                    x: 0.,
                    y: 5.,
                    width: 0.,
                    height: 20.,
                },
            ),
            (
                entry(2, 1).id,
                Rect {
                    x: 0.,
                    y: 80.,
                    width: 100.,
                    height: 20.,
                },
            ),
        ];
        assert_eq!(
            current(
                &values,
                Rect {
                    x: 0.,
                    y: 0.,
                    width: 200.,
                    height: 200.
                },
                0.
            ),
            Some(entry(2, 1).id)
        );
        assert_eq!(
            current(
                &[],
                Rect {
                    x: 0.,
                    y: 0.,
                    width: 200.,
                    height: 200.
                },
                0.
            ),
            None
        );
    }
}
