use glory::render::{DocumentPart, Props, TableMetadata};
use glory::ui::{App, Widget};
use sapid::{
    table, table_body, table_cell, table_head, table_header_cell, table_row, table_row_header,
};
use std::{cell::Cell, rc::Rc};
fn fixture(theme: sapid::Theme) -> impl glory::Widget {
    table("people")
        .theme(theme)
        .caption("Project members")
        .striped(true)
        .bordered(true)
        .head(table_head().row(table_row("head").cells([
            table_header_cell("name").text("Name"),
            table_header_cell("hours").text("Hours"),
        ])))
        .body(
            table_body()
                .row(table_row("ada").cells([
                    table_row_header("name").text("Ada"),
                    table_cell("hours").text("12"),
                ]))
                .row(table_row("grace").cells([
                    table_row_header("name").text("Grace"),
                    table_cell("hours").text("20"),
                ])),
        )
}
#[test]
fn four_theme_ssr_uses_intrinsic_tags_caption_and_real_scoped_header_associations() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let snapshot = glory::ssr::snapshot(fixture(theme)).unwrap();
        let html = snapshot.html;
        assert!(
            html.contains("<table")
                && html.contains("<caption")
                && html.contains("<thead")
                && html.contains("<tbody")
        );
        assert_eq!(html.matches("<tr ").count(), 3);
        assert_eq!(html.matches("scope=\"col\"").count(), 2);
        assert_eq!(html.matches("scope=\"row\"").count(), 2);
        assert_eq!(html.matches("headers=\"").count(), 4);
        assert!(html.contains("aria-rowcount=\"3\"") && html.contains("aria-colcount=\"2\""));
    }
}
#[test]
fn live_text_updates_keep_editor_and_table_refs_and_release_every_short_owner() {
    let app = App::memory();
    let scope = app.scope();
    let name = scope.cage("Original".to_owned());
    let editor = scope.cage(None);
    let root = scope.cage(None);
    let row = scope.cage(None);
    let cleaned = Rc::new(Cell::new(0));
    let done = cleaned.clone();
    app.mount(
        table("live").node_ref(root).body(
            table_body().row(
                table_row("row")
                    .node_ref(row)
                    .cell(table_row_header("label").text(name))
                    .cell(table_cell("edit").content(move || {
                        let done = done.clone();
                        glory::ui::component(move |scope| {
                            let done = done.clone();
                            scope.on_cleanup(move || done.set(done.get() + 1))?;
                            glory::ui::input()
                                .value("draft")
                                .node_ref(editor)
                                .show_in(scope)
                        })
                    })),
            ),
        ),
    )
    .unwrap();
    let old = (
        root.get().unwrap(),
        row.get().unwrap(),
        editor.get().unwrap(),
    );
    name.revise(|value| *value = "Revised".into());
    app.flush().unwrap();
    assert_eq!(
        (
            root.get().unwrap(),
            row.get().unwrap(),
            editor.get().unwrap()
        ),
        old
    );
    assert_eq!(cleaned.get(), 0);
    assert!(matches!(
        app.tree().node(old.2).unwrap().descriptor.view().props,
        Props::Input(_)
    ));
    app.clear().unwrap();
    assert!(root.get().is_none() && row.get().is_none() && editor.get().is_none());
    assert_eq!(cleaned.get(), 1);
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(app.stats().queries, 0);
}
#[test]
fn bounded_constructors_stop_at_sentinels_and_invalid_grids_run_no_factories() {
    let consumed = Rc::new(Cell::new(0));
    let count = consumed.clone();
    let rows = (0..1_000_000).map(move |i| {
        count.set(count.get() + 1);
        table_row(format!("r{i}"))
    });
    let widget = table("limit").body(table_body().rows(rows));
    assert_eq!(consumed.get(), 257);
    assert!(App::memory().mount(widget).is_err());
    let calls = Rc::new(Cell::new(0));
    let counter = calls.clone();
    let widget = table("overlap").body(
        table_body()
            .row(table_row("first").cell(table_cell("app").content(move || {
                counter.set(counter.get() + 1);
                glory::ui::text().content("app")
            })))
            .row(table_row("bad").cell(table_cell("span").row_span(2))),
    );
    assert!(App::memory().mount(widget).is_err());
    assert_eq!(calls.get(), 0);
    let app = App::memory();
    assert!(
        app.mount(
            table("duplicate").body(table_body().rows([table_row("same"), table_row("same")]))
        )
        .is_err()
    );
}
#[test]
fn row_group_span_grid_maps_explicit_coordinates_and_headers_without_cycles() {
    let app = App::memory();
    app.mount(
        table("spans")
            .head(
                table_head()
                    .row(
                        table_row("h0")
                            .cell(table_header_cell("group").column_span(2).text("Group")),
                    )
                    .row(table_row("h1").cells([
                        table_header_cell("a").text("A"),
                        table_header_cell("b").text("B"),
                    ])),
            )
            .body(
                table_body()
                    .row(
                        table_row("r0")
                            .cell(table_row_header("row").row_span(2).text("Row"))
                            .cell(table_cell("value").text("First")),
                    )
                    .row(table_row("r1").cell(table_cell("value").text("Second"))),
            ),
    )
    .unwrap();
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut seen = 0;
    while let Some(id) = todo.pop() {
        let n = tree.node(id).unwrap();
        let d = n.descriptor.view();
        if let Some(s) = d.semantics.as_ref()
            && let Some(TableMetadata::Cell {
                row,
                column,
                headers,
                ..
            }) = &s.table
        {
            seen += 1;
            if s.document_part == Some(DocumentPart::ColumnHeader) && *row == 0 {
                assert!(headers.is_empty())
            }
            if *row == 3 {
                assert_eq!(*column, 1);
                assert_eq!(headers.len(), 3);
            }
        }
        todo.extend(&n.children);
    }
    assert_eq!(seen, 6);
}
