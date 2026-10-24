use glory::prelude::*;
use sapid::{
    Theme, button, table, table_body, table_cell, table_foot, table_head, table_header_cell,
    table_row, table_row_header,
};
pub fn table_form(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let label = scope.cage("Ada".to_owned());
        let text = scope.cage("Native editable cell".to_owned());
        let toggled = scope.cage(false);
        let t = theme.clone();
        let value=table("roster").theme(theme.clone()).striped(true).bordered(true).caption("Project team hours")
   .head(table_head().row(table_row("head").cells([table_header_cell("name").text("Person"),table_header_cell("hours").text("Hours"),table_header_cell("note").text("Note")])))
   .body(table_body().row(table_row("ada").cells([table_row_header("name").text(label),table_cell("hours").text("12"),table_cell("note").content(move||sapid::text_field().class("table-draft").theme(t.clone()).name("Ada note").value(text).on_value_change(move|v|text.revise(|old|*old=v)))]))
    .row(table_row("grace").cells([table_row_header("name").text("Grace"),table_cell("hours").text("20"),table_cell("note").text("A long review note which wraps without changing its column association")]))
    .row(table_row("linus").cells([table_row_header("name").text("Linus"),table_cell("hours").text("8"),table_cell("note").text("Ready")])) )
   .foot(table_foot().row(table_row("total").cells([table_row_header("name").text("Total"),table_cell("hours").text("40"),table_cell("note").text("Three active contributors")])));
        glory::ui::col()
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(glory::style::FlexDirection::Column)
                    .gap(12.),
            )
            .fill(value)
            .fill(span_table(theme.clone()))
            .fill(
                button()
                    .theme(theme.clone())
                    .label("Rename table row")
                    .on_press(move || {
                        toggled.revise(|v| *v = !*v);
                        label.revise(|v| {
                            *v = if toggled.get() {
                                "Ada Lovelace".into()
                            } else {
                                "Ada".into()
                            }
                        })
                    }),
            )
            .show_in(scope)
    })
}
pub fn span_table(theme: Theme) -> impl Widget {
    table("spans")
        .theme(theme)
        .class("table-span-example")
        .caption("Grouped totals")
        .bordered(true)
        .head(
            table_head()
                .row(
                    table_row("group").cell(
                        table_header_cell("all")
                            .column_span(2)
                            .text("Time allocation"),
                    ),
                )
                .row(table_row("columns").cells([
                    table_header_cell("project").text("Project"),
                    table_header_cell("hours").text("Hours"),
                ])),
        )
        .body(
            table_body()
                .row(
                    table_row("first")
                        .cell(table_row_header("project").row_span(2).text("Compiler"))
                        .cell(table_cell("hours").text("12")),
                )
                .row(table_row("second").cell(table_cell("hours").text("20"))),
        )
}
