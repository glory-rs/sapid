//! Observe actual mounted DataTable page/column updates in release mode.
//! Run: cargo run --release --example data_table_derivation_bench
//! These local timings are not CI thresholds; derivation-count regressions live in unit tests.
use glory::{Cage, prelude::App};
use sapid::{DataColumn, DataRecord, DataRequest, DataSort, DataTableQuery, data_table};
use std::{hint::black_box, time::Instant};

const RUNS: usize = 20;
fn fixture() -> (App, Cage<DataTableQuery>) {
    let columns = (0..8)
        .map(|i| DataColumn::new(format!("c{i}"), format!("Column {i}")))
        .collect::<Vec<_>>();
    let records = (0..4096)
        .map(|row| {
            DataRecord::new(
                format!("r{row}"),
                (0..8).map(|column| {
                    if column == 7 {
                        format!("{}needle", "x".repeat(250))
                    } else {
                        format!("{row:04}-{}", "x".repeat(251))
                    }
                }),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        records
            .iter()
            .flat_map(|r| &r.cells)
            .map(String::len)
            .sum::<usize>(),
        8 * 1024 * 1024
    );
    let app = App::memory();
    let query = app.scope().cage(DataTableQuery {
        request: DataRequest {
            page_size: 2,
            filter: "needle".into(),
            sort: Some(DataSort {
                column: "c0".into(),
                descending: true,
            }),
            ..Default::default()
        },
        ..Default::default()
    });
    app.mount(
        data_table("bench")
            .columns(columns)
            .local_rows(records)
            .query(query)
            .height(88.),
    )
    .unwrap();
    (app, query)
}
fn measure(label: &str, app: &App, mut change: impl FnMut()) {
    for _ in 0..2 {
        change();
        app.flush().unwrap();
    }
    let start = Instant::now();
    for _ in 0..RUNS {
        change();
        app.flush().unwrap();
        black_box(app.stats().nodes);
    }
    let elapsed = start.elapsed();
    println!(
        "{label}: runs={RUNS}, total_ms={:.3}, mean_ms={:.3}, live_nodes={}",
        elapsed.as_secs_f64() * 1000.,
        elapsed.as_secs_f64() * 1000. / RUNS as f64,
        app.stats().nodes
    );
}
fn main() {
    let (app, query) = fixture();
    println!(
        "fixture: rows=4096, columns=8, text_bytes=8388608, page_size=2, matching=4096, warmup=2, runs={RUNS}"
    );
    measure("page_flip", &app, || {
        query.revise(|q| q.request.page = if q.request.page == 1 { 2 } else { 1 })
    });
    measure("column_visibility", &app, || {
        query.revise(|q| {
            if q.hidden.is_empty() {
                q.hidden.insert("c7".into());
            } else {
                q.hidden.clear();
            }
        })
    });
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
}
