use glory::{
    prelude::*,
    render::{Event, NodeId, Props, Role, TableMetadata},
};
use sapid::{DataColumn, DataRecord, DataRequest, DataSort, DataTableQuery, data_table};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn columns() -> Vec<DataColumn> {
    vec![
        DataColumn::new("name", "Name"),
        DataColumn::new("team", "Team"),
    ]
}
fn records() -> Vec<DataRecord> {
    vec![
        DataRecord::new("c", ["Charlie", "Runtime"]),
        DataRecord::new("a", ["Ada", "Compiler"]),
        DataRecord::new("b", ["Ben", "Compiler"]),
    ]
}
fn named(app: &App, name: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let n = tree.node(id).unwrap();
        let d = n.descriptor.view();
        if d.name.as_deref() == Some(name) || matches!(d.props,Props::Button(p) if p.label==name) {
            return id;
        }
        pending.extend(&n.children);
    }
    panic!("missing {name}")
}
fn rows(app: &App) -> Vec<(u32, NodeId)> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut rows = Vec::new();
    while let Some(id) = pending.pop() {
        let n = tree.node(id).unwrap();
        let d = n.descriptor.view();
        if d.role == Some(Role::Row)
            && let Some(TableMetadata::Row { index }) =
                d.semantics.as_ref().and_then(|s| s.table.as_ref())
            && *index > 0
        {
            rows.push((*index, id))
        }
        pending.extend(&n.children);
    }
    rows.sort_by_key(|r| r.0);
    rows
}
#[test]
fn four_theme_ssr_exports_windowed_table_header_coordinates_and_bounded_rows() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let records = (0..4096)
            .map(|i| DataRecord::new(format!("r{i}"), [format!("Person {i}"), "Compiler".into()]))
            .collect::<Vec<_>>();
        let q = DataTableQuery {
            request: DataRequest {
                page_size: 4096,
                ..Default::default()
            },
            ..Default::default()
        };
        let html = glory::ssr::snapshot(
            data_table("roster")
                .name("People")
                .theme(theme)
                .columns(columns())
                .local_rows(records)
                .default_query(q)
                .height(132.)
                .row_height(44.),
        )
        .unwrap()
        .html;
        assert!(
            html.contains("role=\"table\"")
                && html.contains("aria-rowcount=\"4097\"")
                && html.contains("aria-sort=\"none\"")
        );
        assert!(!html.contains("role=\"list\"") && !html.contains("aria-posinset"));
        assert!(html.matches("sapid-data-table-row").count() <= 6);
        assert!(!html.contains("Person 4095"));
    }
}
#[test]
fn controlled_query_proposes_before_callback_and_external_updates_are_silent() {
    let app = App::memory();
    let query = app.scope().cage(DataTableQuery::default());
    let proposals = Rc::new(RefCell::new(Vec::new()));
    let out = proposals.clone();
    app.mount(
        data_table("controlled")
            .columns(columns())
            .local_rows(records())
            .query(query)
            .on_query_change(move |q| out.borrow_mut().push(q)),
    )
    .unwrap();
    app.flush().unwrap();
    let sort = named(&app, "Sort by Name");
    app.scope()
        .batch(|| {
            app.dispatch(sort, Event::Activate).unwrap();
            app.dispatch(sort, Event::Activate).unwrap();
        })
        .unwrap();
    app.flush().unwrap();
    assert!(query.with(|q| q.request.sort.is_none()));
    assert_eq!(proposals.borrow().len(), 2);
    assert!(
        proposals.borrow()[1]
            .request
            .sort
            .as_ref()
            .unwrap()
            .descending
    );
    query.revise(|q| q.request.filter = "Compiler".into());
    app.flush().unwrap();
    assert_eq!(proposals.borrow().len(), 2);
    assert_eq!(rows(&app).len(), 2);
}
#[test]
fn local_sort_keeps_live_cell_owners_and_releases_them_on_page_retirement() {
    let app = App::memory();
    let calls = Rc::new(Cell::new(0));
    let closed = Rc::new(Cell::new(0));
    let created = calls.clone();
    let retired = closed.clone();
    let query = app.scope().cage(DataTableQuery::default());
    app.mount(
        data_table("identity")
            .columns(columns())
            .local_rows(records())
            .query(query)
            .on_query_change(move |q| query.revise(|old| *old = q))
            .render_cell("name", move |_, key, value| {
                created.set(created.get() + 1);
                let retired = retired.clone();
                widget(move |scope| {
                    let retired = retired.clone();
                    scope.on_cleanup(move || retired.set(retired.get() + 1))?;
                    glory::ui::input()
                        .name(format!("Draft {key}"))
                        .value(value.get())
                        .show_in(scope)
                })
            }),
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(calls.get(), 3);
    let editor = named(&app, "Draft a");
    app.dispatch(named(&app, "Sort by Name"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert_eq!(named(&app, "Draft a"), editor);
    assert_eq!(calls.get(), 3);
    assert_eq!(closed.get(), 0);
    app.clear().unwrap();
    assert_eq!(closed.get(), 3);
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(app.stats().queries, 0);
}
#[test]
fn invalid_source_and_grid_are_rejected_before_custom_cell_factories() {
    let app = App::memory();
    let called = Rc::new(Cell::new(0));
    let calls = called.clone();
    let mut invalid = records();
    invalid[1].key = invalid[0].key.clone();
    assert!(
        app.mount(
            data_table("bad")
                .columns(columns())
                .local_rows(invalid)
                .render_cell("name", move |_, _, _| {
                    calls.set(calls.get() + 1);
                    glory::ui::text().content("never")
                })
        )
        .is_err()
    );
    assert_eq!(called.get(), 0);
    assert!(app.tree().is_empty());
}
#[test]
fn server_pages_wait_for_matching_requests_and_ignore_late_different_pages() {
    let app = App::memory();
    let query = app.scope().cage(DataTableQuery {
        request: DataRequest {
            page_size: 2,
            ..Default::default()
        },
        ..Default::default()
    });
    let first = sapid::DataPage {
        request: query.get().request,
        rows: records()[..2].to_vec(),
        total: Some(3),
        has_next: true,
    };
    let source = app.scope().cage(first.clone());
    app.mount(
        data_table("server")
            .columns(columns())
            .server_page(source)
            .query(query)
            .on_query_change(move |q| query.revise(|old| *old = q)),
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(rows(&app).len(), 2);
    app.dispatch(named(&app, "Next table page"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert!(rows(&app).is_empty());
    source.revise(|page| {
        *page = sapid::DataPage {
            request: query.get().request,
            rows: vec![records().pop().unwrap()],
            total: Some(3),
            has_next: false,
        }
    });
    app.flush().unwrap();
    let current = rows(&app);
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].0, 3);
    source.revise(|page| *page = first);
    app.flush().unwrap();
    assert_eq!(rows(&app), current);
}
#[test]
fn explicit_selection_survives_filtering_and_controlled_refusal_recovers_the_check() {
    let app = App::memory();
    let query = app.scope().cage(DataTableQuery::default());
    let selected = app.scope().cage(std::collections::BTreeSet::new());
    let proposals = Rc::new(RefCell::new(Vec::new()));
    let out = proposals.clone();
    app.mount(
        data_table("selection")
            .columns(columns())
            .local_rows(records())
            .query(query)
            .selected(selected)
            .on_selection_change(move |set| out.borrow_mut().push(set)),
    )
    .unwrap();
    app.flush().unwrap();
    let check = named(&app, "Select row a");
    app.dispatch(check, Event::CheckedChanged(true)).unwrap();
    assert!(selected.with(|set| set.is_empty()));
    assert!(proposals.borrow()[0].contains("a"));
    let tree = app.tree();
    assert!(
        matches!(tree.node(check).unwrap().descriptor.view().props,Props::Check(p) if !p.checked)
    );
    drop(tree);
    selected.revise(|set| {
        set.insert("a".into());
    });
    query.revise(|q| q.request.filter = "Runtime".into());
    app.flush().unwrap();
    assert_eq!(rows(&app).len(), 1);
    assert!(selected.with(|set| set.contains("a")));
    query.revise(|q| q.request.filter.clear());
    app.flush().unwrap();
    assert!(
        matches!(app.tree().node(named(&app,"Select row a")).unwrap().descriptor.view().props,Props::Check(p) if p.checked)
    );
}

#[test]
fn page_and_visibility_changes_keep_intersecting_row_drafts_and_focus_state() {
    let app = App::memory();
    let query = app.scope().cage(DataTableQuery::default());
    app.mount(
        data_table("draft-pages")
            .columns(columns())
            .local_rows(records())
            .query(query)
            .render_cell("name", move |_, key, value| {
                widget(move |scope| {
                    let draft = scope.cage(value.get());
                    let focused = scope.cage(false);
                    sapid::text_field()
                        .name(format!("Draft {key}"))
                        .value(draft)
                        .on_value_change(move |value| draft.revise(|old| *old = value))
                        .class(scope.bond(move || {
                            if focused.get() {
                                "editing-focused".into()
                            } else {
                                "editing".into()
                            }
                        }))
                        .on(glory::render::EventKind::FocusChanged, move |event| {
                            if let Event::FocusChanged(value) = event {
                                focused.revise(|old| *old = value);
                            }
                        })
                        .show_in(scope)
                })
            }),
    )
    .unwrap();
    let editor = named(&app, "Draft a");
    app.dispatch(editor, Event::FocusChanged(true)).unwrap();
    let revision = match app.tree().node(editor).unwrap().descriptor.view().props {
        Props::Input(props) => props.revision,
        _ => panic!("expected draft input"),
    };
    app.dispatch(
        editor,
        Event::ValueChanged {
            value: "Unsubmitted draft".into(),
            revision,
            composing: false,
        },
    )
    .unwrap();
    app.flush().unwrap();
    query.revise(|q| {
        q.hidden.insert("team".into());
        q.request.page_size = 2;
    });
    app.flush().unwrap();
    query.revise(|q| {
        q.request.page_size = 1;
        q.request.page = 2;
    });
    app.flush().unwrap();
    assert_eq!(named(&app, "Draft a"), editor);
    assert!(
        matches!(app.tree().node(editor).unwrap().descriptor.view().props, Props::Input(p) if p.value == "Unsubmitted draft")
    );
    assert!(
        app.tree()
            .node(editor)
            .unwrap()
            .descriptor
            .view()
            .classes
            .iter()
            .any(|c| c == "editing-focused")
    );
    // The controlled page proposal is refused; its pending intent cannot replace
    // the accepted page or discard the live row's local editing state.
    app.dispatch(named(&app, "Next table page"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert_eq!(query.with(|q| q.request.page), 2);
    assert_eq!(named(&app, "Draft a"), editor);
    assert!(
        matches!(app.tree().node(editor).unwrap().descriptor.view().props, Props::Input(p) if p.value == "Unsubmitted draft")
    );
    query.revise(|q| {
        q.request.page = 1;
        q.hidden.clear();
        q.request.page_size = 3;
    });
    app.flush().unwrap();
    assert_eq!(named(&app, "Draft a"), editor);
}

#[test]
fn replacing_data_and_changing_filter_sort_publish_the_current_page_without_stale_indices() {
    let app = App::memory();
    let source = app.scope().cage(records());
    let query = app.scope().cage(DataTableQuery {
        request: DataRequest {
            page_size: 1,
            filter: "Compiler".into(),
            sort: Some(DataSort {
                column: "name".into(),
                descending: false,
            }),
            ..Default::default()
        },
        ..Default::default()
    });
    app.mount(
        data_table("replace-matches")
            .columns(columns())
            .local_rows(source)
            .query(query),
    )
    .unwrap();
    assert!(app.tree().node(named(&app, "Select row a")).is_ok());
    query.revise(|q| q.request.page = 2);
    app.flush().unwrap();
    assert!(app.tree().node(named(&app, "Select row b")).is_ok());
    source.revise(|rows| {
        *rows = vec![
            DataRecord::new("d", ["Dean", "Compiler"]),
            DataRecord::new("a", ["Ada updated", "Compiler"]),
            DataRecord::new("b", ["Ben", "Runtime"]),
        ]
    });
    app.flush().unwrap();
    assert!(app.tree().node(named(&app, "Select row d")).is_ok());
    query.revise(|q| {
        q.request.page = 1;
        q.request.sort.as_mut().unwrap().descending = true;
    });
    app.flush().unwrap();
    assert!(app.tree().node(named(&app, "Select row d")).is_ok());
    query.revise(|q| {
        q.request.filter = "Runtime".into();
        q.hidden.insert("team".into());
    });
    app.flush().unwrap();
    assert!(
        app.tree().node(named(&app, "Select row b")).is_ok(),
        "hidden cells still participate in filtering"
    );
    source.revise(|rows| rows[2].cells[1] = "Compiler".into());
    app.flush().unwrap();
    assert!(rows(&app).is_empty());
    source.revise(|rows| rows[0].key = "a".into());
    assert!(app.flush().is_err());
    assert!(
        rows(&app).is_empty(),
        "rejected data cannot publish a partial match set"
    );
    source.revise(|rows| rows[0].key = "d".into());
    query.revise(|q| q.request.filter = "Compiler".into());
    app.flush().unwrap();
    assert!(app.tree().node(named(&app, "Select row d")).is_ok());
}
