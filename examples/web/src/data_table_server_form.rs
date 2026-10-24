use glory::prelude::*;
use sapid::{DataColumn, DataPage, DataRecord, DataRequest, DataTableQuery, Theme, data_table};
fn page_for(request: DataRequest, unknown: bool) -> DataPage {
    let needle = request.filter.to_lowercase();
    let mut all = (0..48)
        .map(|index| {
            DataRecord::new(
                format!("server-{index:02}"),
                [
                    format!("Server row {index:02}"),
                    if index % 2 == 0 {
                        "Even".into()
                    } else {
                        "Odd".into()
                    },
                ],
            )
        })
        .filter(|record| {
            needle.is_empty()
                || record
                    .cells
                    .iter()
                    .any(|cell| cell.to_lowercase().contains(&needle))
        })
        .collect::<Vec<_>>();
    if let Some(sort) = &request.sort {
        let column = usize::from(sort.column == "kind");
        all.sort_by(|a, b| {
            let order = a.cells[column].cmp(&b.cells[column]);
            if sort.descending {
                order.reverse()
            } else {
                order
            }
        })
    }
    let total = all.len();
    let start = (request.page - 1) * request.page_size;
    let rows = all
        .into_iter()
        .skip(start)
        .take(request.page_size)
        .collect::<Vec<_>>();
    let has_next = start.saturating_add(rows.len()) < total;
    DataPage {
        request,
        rows,
        total: (!unknown).then_some(total),
        has_next,
    }
}
pub fn data_table_server_form(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let query = scope.cage(DataTableQuery {
            request: DataRequest {
                page_size: 4,
                ..Default::default()
            },
            ..Default::default()
        });
        let first = page_for(query.get().request, false);
        let previous = scope.cage(first.clone());
        let page = scope.cage(first);
        let unknown = scope.cage(false);
        glory::ui::col()
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(glory::style::FlexDirection::Column)
                    .gap(12.),
            )
            .fill(
                data_table("remote")
                    .theme(theme.clone())
                    .name("Server records")
                    .columns([
                        DataColumn::new("name", "Name"),
                        DataColumn::new("kind", "Kind"),
                    ])
                    .server_page(page)
                    .query(query)
                    .on_query_change(move |value| query.revise(|old| *old = value))
                    .height(224.)
                    .row_height(44.),
            )
            .fill(
                sapid::checkbox()
                    .theme(theme.clone())
                    .label("Unknown server total")
                    .checked(unknown)
                    .on_checked_change(move |value| unknown.revise(|old| *old = value)),
            )
            .fill(
                sapid::button()
                    .theme(theme.clone())
                    .label("Publish server reply")
                    .on_press(move || {
                        let old = page.get();
                        let next = page_for(query.get().request, unknown.get());
                        previous.revise(|value| *value = old);
                        page.revise(|value| *value = next)
                    }),
            )
            .fill(
                sapid::button()
                    .theme(theme.clone())
                    .label("Replay previous server reply")
                    .on_press(move || {
                        let old = previous.get();
                        page.revise(|value| *value = old)
                    }),
            )
            .show_in(scope)
    })
}
