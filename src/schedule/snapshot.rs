//! Current accepted data and derived geometry share one occurrence index space.
use super::*;
use crate::{ScheduleDayPlacement, ScheduleOccurrence, ScheduleTimeWindow};
use std::ops::Deref;

pub(super) struct Snapshot {
    model: ScheduleData,
    pub(super) occurrences: Vec<ScheduleOccurrence>,
    pub(super) day_layouts: Vec<Vec<ScheduleDayPlacement>>,
}

impl Deref for Snapshot {
    type Target = ScheduleData;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl Snapshot {
    fn new(model: &ScheduleData, view: ScheduleView, time: ScheduleTimeWindow) -> Result<Self> {
        let occurrences = model
            .expand(MAX_SCHEDULE_OCCURRENCES)
            .map_err(|_| invalid())?;
        let day_layouts = match view {
            ScheduleView::Day | ScheduleView::Week => timeline::validate(
                model,
                if view == ScheduleView::Day { 1 } else { 7 },
                time,
                &occurrences,
            )?,
            ScheduleView::Month | ScheduleView::Year | ScheduleView::MobileMonth => {
                calendar::validate(model, view)?;
                vec![]
            }
            ScheduleView::Agenda => vec![],
        };
        Ok(Self {
            model: model.clone(),
            occurrences,
            day_layouts,
        })
    }
}

pub(super) fn accept(
    scope: &Scope,
    source: &Vary<ScheduleData>,
    view: ScheduleView,
    time: ScheduleTimeWindow,
) -> Result<Cage<Rc<Snapshot>>> {
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
