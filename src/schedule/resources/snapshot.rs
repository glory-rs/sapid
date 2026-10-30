//! One accepted resource window and its complete occurrence/index space.
use super::*;
use crate::{ResourceScheduleTimeline, ScheduleOccurrence};
use std::ops::Deref;

pub(super) struct Snapshot {
    model: ResourceScheduleData,
    pub(super) occurrences: Vec<ScheduleOccurrence>,
    pub(super) layout: ResourceScheduleTimeline,
}

impl Deref for Snapshot {
    type Target = ResourceScheduleData;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl Snapshot {
    fn new(
        model: &ResourceScheduleData,
        view: ResourceScheduleView,
        time: ScheduleTimeWindow,
    ) -> Result<Self> {
        let occurrences = model
            .data()
            .expand(MAX_SCHEDULE_OCCURRENCES)
            .map_err(|_| invalid())?;
        let layout = match view {
            ResourceScheduleView::Day | ResourceScheduleView::Week => model
                .layout_timeline_expanded(
                    time,
                    if view == ResourceScheduleView::Day {
                        1
                    } else {
                        7
                    },
                    &occurrences,
                ),
            ResourceScheduleView::Month => model.layout_month_expanded(&occurrences),
        }
        .map_err(|_| invalid())?;
        Ok(Self {
            model: model.clone(),
            occurrences,
            layout,
        })
    }
}

/// Publish the data, expansion and geometry atomically. Invalid source updates
/// preserve the complete last accepted snapshot; there is no visited-window cache.
pub(super) fn accept(
    scope: &Scope,
    source: &Vary<ResourceScheduleData>,
    view: ResourceScheduleView,
    time: ScheduleTimeWindow,
) -> Result<AcceptedResources> {
    let initial = glory::core::untracked(|| {
        source.try_with(|model| Snapshot::new(model, view, time).map(Rc::new))
    })??;
    let current = scope.cage(initial);
    let source = source.clone();
    let weak = scope.downgrade();
    scope.structural(move || {
        let Some(scope) = weak.upgrade() else { return };
        let result = source.try_with(|model| {
            if glory::core::untracked(|| current.with(|old| &old.model == model)) {
                return Ok(None);
            }
            Snapshot::new(model, view, time).map(|next| Some(Rc::new(next)))
        });
        match result {
            Ok(Ok(Some(next))) => current.revise(|old| *old = next),
            Ok(Ok(None)) => {}
            Ok(Err(error)) => scope.report_error(error),
            Err(error) => scope.report_error(error),
        }
    });
    Ok(current)
}
