//! One admitted row snapshot and one current filter/sort result, never page history.
use super::*;

pub(super) fn accept(
    scope: &Scope,
    source: &Vary<Vec<DataRecord>>,
    columns: Rc<Vec<DataColumn>>,
) -> Result<Cage<Rc<Vec<DataRecord>>>> {
    let initial = glory::core::untracked(|| {
        source.try_with(|rows| {
            validate_data_records(&columns, rows).map_err(error)?;
            Ok::<_, glory::ui::Error>(Rc::new(rows.clone()))
        })
    })??;
    let current = scope.cage(initial);
    let source = source.clone();
    let weak = scope.downgrade();
    scope.structural(move || {
        let Some(scope) = weak.upgrade() else { return };
        let result = source.try_with(|rows| {
            if glory::core::untracked(|| current.with(|old| old.as_ref() == rows)) {
                return Ok(None);
            }
            validate_data_records(&columns, rows).map_err(error)?;
            Ok::<_, glory::ui::Error>(Some(Rc::new(rows.clone())))
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

pub(super) struct Matches {
    rows: Rc<Vec<DataRecord>>,
    filter: String,
    sort: Option<DataSort>,
    indices: Vec<usize>,
}

pub(super) fn plan(
    columns: &[DataColumn],
    rows: &Rc<Vec<DataRecord>>,
    query: &DataTableQuery,
    current: &mut Option<Matches>,
) -> DataPlan {
    let filter = query.request.filter.to_lowercase();
    if current.as_ref().is_none_or(|old| {
        !Rc::ptr_eq(&old.rows, rows) || old.filter != filter || old.sort != query.request.sort
    }) {
        *current = Some(Matches {
            indices: model::local_match_indices(
                columns,
                rows,
                &filter,
                query.request.sort.as_ref(),
            ),
            rows: rows.clone(),
            filter,
            sort: query.request.sort.clone(),
        });
    }
    model::local_page_plan(
        &current.as_ref().expect("current admitted query").indices,
        &query.request,
    )
}
