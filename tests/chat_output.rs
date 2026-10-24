#![cfg(feature = "chat")]
use glory::{
    prelude::*,
    render::{NodeId, Props},
};
use sapid::chat::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[path = "../examples/web/src/output_form.rs"]
mod fixture;
#[test]
fn output_fixture_contains_both_windows_and_explicit_test_controls() {
    let app = App::memory();
    app.mount(fixture::output_form(sapid::Theme::blue_light()))
        .unwrap();
    assert_eq!(class(&app, "terminal-primary").len(), 1);
    assert_eq!(class(&app, "test-results-primary").len(), 1);
    assert_eq!(class(&app, "sapid-test-case").len(), 4);
    let _ = button(&app, "Failed · renderer applies accepted prefix · 18 ms");
    app.dispatch(
        button(&app, "Failed · renderer applies accepted prefix · 18 ms"),
        Event::Activate,
    )
    .unwrap();
    app.dispatch(button(&app, "Clear test results"), Event::Activate)
        .unwrap();
    assert!(class(&app, "sapid-test-case").is_empty());
}
fn all(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = pending.pop() {
        pending.extend(&tree.node(id).unwrap().children);
        out.push(id)
    }
    out
}
fn class(app: &App, name: &str) -> Vec<NodeId> {
    all(app)
        .into_iter()
        .filter(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|c| c == name)
        })
        .collect()
}
fn text(app: &App) -> String {
    all(app)
        .into_iter()
        .filter_map(
            |id| match app.tree().node(id).unwrap().descriptor.view().props {
                Props::Text(p) => Some(p.content.clone()),
                _ => None,
            },
        )
        .collect::<Vec<_>>()
        .join("|")
}
fn button(app: &App, name: &str) -> NodeId {
    all(app).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(p)if p.label==name)).unwrap()
}
#[test]
fn literal_control_projection_redaction_and_counts_have_explicit_budgets() {
    assert_eq!(
        LogPayload::Text("a\r\nb\rc\u{1b}[31m\0\t<script>".into())
            .display_text()
            .unwrap(),
        "a\nb\nc\\u{1b}[31m\\u{0}\t<script>"
    );
    assert_eq!(LogPayload::Redacted.display_text().unwrap(), "[redacted]");
    assert!(LogPayload::Text("x".repeat(8193)).display_text().is_err());
    assert!(
        TestCounts {
            passed: u64::MAX,
            failed: 1,
            ..Default::default()
        }
        .total()
        .is_err()
    );
    assert!(
        TestCounts {
            passed: 1_000_000_001,
            ..Default::default()
        }
        .total()
        .is_err()
    );
    assert_eq!(
        TestCounts {
            passed: 1,
            failed: 2,
            skipped: 3,
            running: 4
        }
        .total()
        .unwrap(),
        10
    );
}
#[test]
fn billion_records_only_sample_bounded_live_rows_and_seek_without_retaining_source() {
    let app = App::memory();
    let reads = Rc::new(Cell::new(0));
    let calls = reads.clone();
    let handle = Rc::new(RefCell::new(None));
    let out = handle.clone();
    let stats = Rc::new(RefCell::new(None));
    let report = stats.clone();
    let sentinel = Rc::new(());
    let weak = Rc::downgrade(&sentinel);
    app.mount(
        terminal(
            "build",
            1_000_000_000usize,
            |i| i as u64,
            |k| usize::try_from(k).ok().filter(|k| *k < 1_000_000_000),
            move |i| {
                let _ = &sentinel;
                calls.set(calls.get() + 1);
                LogRecord::new(i as u64, format!("line {i}"))
            },
        )
        .height(200.)
        .estimate(40.)
        .row_limit(16)
        .measure_limit(32)
        .on_ready(move |h| *out.borrow_mut() = Some(h))
        .inspect(move |v| *report.borrow_mut() = Some(v)),
    )
    .unwrap();
    assert!(reads.get() < 100, "{}", reads.get());
    assert!(class(&app, "sapid-log-record").len() <= 16);
    assert!(stats.borrow().as_ref().unwrap().range.start > 999_999_000);
    let h = handle.borrow().clone().unwrap();
    h.first().unwrap();
    app.flush().unwrap();
    assert!(!h.is_following().unwrap());
    assert_eq!(stats.borrow().as_ref().unwrap().range.start, 0);
    assert!(text(&app).contains("line 0"));
    h.newest().unwrap();
    app.flush().unwrap();
    assert!(h.is_following().unwrap());
    assert!(reads.get() < 250);
    assert!(stats.borrow().as_ref().unwrap().height_slots <= 32);
    app.clear().unwrap();
    assert!(h.first().is_err());
    assert!(h.newest().is_err());
    assert!(weak.upgrade().is_none());
    assert_eq!(app.stats().nodes, 0);
}
#[test]
fn invalid_indexed_updates_retain_complete_accepted_rows_and_then_recover() {
    let app = App::memory();
    let record = app.scope().cage(LogRecord::new(7, "accepted"));
    let rev = app.scope().cage(0u64);
    app.mount(
        terminal(
            "compiler",
            1usize,
            |_| 7,
            |k| (k == 7).then_some(0),
            move |_| record.get(),
        )
        .revision(rev)
        .initially_following(false),
    )
    .unwrap();
    let ids = class(&app, "sapid-output-text");
    record.revise(|r| r.payload = LogPayload::Text("updated".into()));
    app.flush().unwrap();
    assert!(text(&app).contains("updated"));
    assert_eq!(ids, class(&app, "sapid-output-text"));
    record.revise(|r| r.key = 8);
    assert!(app.flush().is_err());
    assert!(text(&app).contains("updated"));
    record.revise(|r| {
        r.key = 7;
        r.payload = LogPayload::Text("x".repeat(8193));
    });
    assert!(app.flush().is_err());
    assert!(text(&app).contains("updated"));
    record.revise(|r| r.payload = LogPayload::Redacted);
    app.flush().unwrap();
    assert!(text(&app).contains("[redacted]"));
    assert_eq!(ids, class(&app, "sapid-output-text"));
}
#[test]
fn test_results_show_application_counts_and_real_read_only_disclosures() {
    let app = App::memory();
    let counts = app.scope().cage(TestCounts {
        passed: 1,
        failed: 1,
        ..Default::default()
    });
    app.mount(
        test_results(
            "tests",
            counts,
            |i| i as u64,
            move |k| (k < counts.with(|v| v.total().unwrap()) as u64).then_some(k as usize),
            |i| TestCase {
                key: i as u64,
                name: format!("case-{i}"),
                status: if i == 0 {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                },
                elapsed_millis: Some(17),
                output: LogPayload::Text("<script>run()</script>\u{1b}[31m".into()),
            },
        )
        .initially_following(false),
    )
    .unwrap();
    assert!(text(&app).contains("1 passed · 1 failed · 0 skipped · 0 running"));
    assert!(class(&app, "sapid-output-text").is_empty());
    let trigger = button(&app, "Failed · case-1 · 17 ms");
    app.dispatch(trigger, Event::Activate).unwrap();
    assert!(text(&app).contains("<script>run()</script>\\u{1b}[31m"));
    assert_eq!(class(&app, "sapid-output-text").len(), 1);
    assert!(!all(&app).iter().any(|id| matches!(
        app.tree().node(*id).unwrap().descriptor.view().props,
        Props::Input(_)
    )));
    let labels = all(&app)
        .into_iter()
        .filter_map(
            |id| match app.tree().node(id).unwrap().descriptor.view().props {
                Props::Button(p) => Some(p.label.clone()),
                _ => None,
            },
        )
        .collect::<Vec<_>>();
    assert!(
        !labels
            .iter()
            .any(|s| s == "Run" || s == "Retry" || s == "Execute")
    );
    counts.revise(|v| *v = TestCounts::default());
    app.flush().unwrap();
    assert!(text(&app).contains("No tests supplied"));
    assert!(class(&app, "sapid-test-case").is_empty());
    app.clear().unwrap();
}
#[test]
fn source_count_and_namespaces_reject_before_any_business_record_reads() {
    let app = App::memory();
    let reads = Rc::new(Cell::new(0));
    let calls = reads.clone();
    assert!(
        app.mount(terminal(
            "bad",
            1_000_000_001usize,
            |i| i as u64,
            |_| None,
            move |i| {
                calls.set(calls.get() + 1);
                LogRecord::new(i as u64, "")
            }
        ))
        .is_err()
    );
    assert_eq!(reads.get(), 0);
    let app = App::memory();
    assert!(
        app.mount(terminal(
            "x".repeat(41),
            1usize,
            |i| i as u64,
            |_| Some(0),
            |i| LogRecord::new(i as u64, "")
        ))
        .is_err()
    );
}

#[test]
fn indexed_prepend_relocates_current_keys_before_reading_their_updated_positions() {
    let app = App::memory();
    let first = app.scope().cage(100u64);
    let count = app.scope().cage(1000usize);
    let revision = app.scope().cage(0u64);
    app.mount(
        terminal(
            "prepend",
            count,
            move |i| first.get() + i as u64,
            move |key| {
                key.checked_sub(first.get())
                    .map(|i| i as usize)
                    .filter(|i| *i < count.get())
            },
            move |i| {
                LogRecord::new(
                    first.get() + i as u64,
                    format!("record {}", first.get() + i as u64),
                )
            },
        )
        .revision(revision)
        .initially_following(false),
    )
    .unwrap();
    let before = class(&app, "sapid-output-text");
    app.scope()
        .batch(|| {
            first.revise(|v| *v -= 10);
            count.revise(|v| *v += 10);
            revision.revise(|v| *v += 1);
        })
        .unwrap();
    app.flush().unwrap();
    assert!(
        before
            .iter()
            .any(|id| class(&app, "sapid-output-text").contains(id))
    );
    assert!(text(&app).contains("record 100"));
}
#[test]
fn independent_output_namespaces_do_not_collide_and_test_source_must_match_its_key() {
    let app = App::memory();
    app.mount(view().fill((
        test_results(
            "one",
            TestCounts {
                passed: 1,
                ..Default::default()
            },
            |_| 1,
            |_| Some(0),
            |_| TestCase {
                key: 1,
                name: "one".into(),
                status: TestStatus::Passed,
                elapsed_millis: None,
                output: LogPayload::Redacted,
            },
        ),
        test_results(
            "two",
            TestCounts {
                passed: 1,
                ..Default::default()
            },
            |_| 1,
            |_| Some(0),
            |_| TestCase {
                key: 1,
                name: "two".into(),
                status: TestStatus::Passed,
                elapsed_millis: None,
                output: LogPayload::Redacted,
            },
        ),
    )))
    .unwrap();
    assert_eq!(class(&app, "sapid-test-case").len(), 2);
    let app = App::memory();
    assert!(
        app.mount(test_results(
            "mismatch",
            TestCounts {
                passed: 1,
                ..Default::default()
            },
            |_| 1,
            |_| Some(0),
            |_| TestCase {
                key: 2,
                name: "wrong".into(),
                status: TestStatus::Passed,
                elapsed_millis: None,
                output: LogPayload::Redacted
            }
        ))
        .is_err()
    );
}
