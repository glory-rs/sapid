use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::data_table_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{DataColumn, DataRecord, DataRequest, DataTableQuery, Theme, data_table};
    pub fn data_table_form(theme: Theme) -> impl Widget {
        widget(move |scope| {
            let source = scope.cage(
                (0..1200)
                    .map(|index| {
                        DataRecord::new(
                            format!("r{index:04}"),
                            [
                                format!("Person {index:04}"),
                                ["Compiler", "Runtime", "Layout"][index % 3].into(),
                                format!("Draft for {index:04}"),
                            ],
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            let query = scope.cage(DataTableQuery {
                request: DataRequest {
                    page_size: 1200,
                    ..Default::default()
                },
                ..Default::default()
            });
            let selected = scope.cage(std::collections::BTreeSet::new());
            let refuse = scope.cage(false);
            let columns = vec![
                DataColumn {
                    width: 180,
                    ..DataColumn::new("name", "Name")
                },
                DataColumn {
                    width: 160,
                    ..DataColumn::new("team", "Team")
                },
                DataColumn {
                    width: 260,
                    sortable: false,
                    ..DataColumn::new("note", "Note")
                },
            ];
            let note_theme = theme.clone();
            glory::ui::col()
                .style(
                    Style::new()
                        .display(Display::Flex)
                        .flex_direction(glory::style::FlexDirection::Column)
                        .gap(12.),
                )
                .fill(
                    sapid::checkbox()
                        .theme(theme.clone())
                        .label("Refuse table changes")
                        .checked(refuse)
                        .on_checked_change(move |v| refuse.revise(|old| *old = v)),
                )
                .fill(
                    data_table("people")
                        .theme(theme.clone())
                        .name("Project people")
                        .columns(columns)
                        .local_rows(source)
                        .query(query)
                        .selected(selected)
                        .height(320.)
                        .row_height(64.)
                        .on_query_change(move |q| {
                            if !refuse.get() {
                                query.revise(|old| *old = q)
                            }
                        })
                        .on_selection_change(move |value| {
                            if !refuse.get() {
                                selected.revise(|old| *old = value)
                            }
                        })
                        .render_cell("note", move |_, key, value| {
                            sapid::text_field()
                                .theme(note_theme.clone())
                                .name(format!("Note for {key}"))
                                .default_value(value.get())
                        }),
                )
                .fill(
                    sapid::button()
                        .theme(theme.clone())
                        .label("Use small table pages")
                        .on_press(move || {
                            query.revise(|q| {
                                q.request.page_size = 25;
                                q.request.page = 1
                            })
                        }),
                )
                .fill(
                    sapid::button()
                        .theme(theme.clone())
                        .label("Reset table query")
                        .on_press(move || {
                            query.revise(|q| {
                                *q = DataTableQuery {
                                    request: DataRequest {
                                        page_size: 1200,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            })
                        }),
                )
                .fill(
                    sapid::button()
                        .theme(theme.clone())
                        .label("Reverse table source")
                        .on_press(move || source.revise(|rows| rows.reverse())),
                )
                .show_in(scope)
        })
    }
}
