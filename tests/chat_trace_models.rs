#![cfg(feature = "chat")]
use sapid::prelude::*;
use std::{cell::Cell, rc::Rc};
fn key(v: &str) -> TraceKey {
    TraceKey::new(v).unwrap()
}
fn record(v: &str) -> TraceRecord {
    TraceRecord::new(key(v), TraceKind::Span, v).unwrap()
}
#[test]
fn records_stop_at_limit_plus_sentinel_without_consuming_a_million_source() {
    let count = Rc::new(Cell::new(0));
    let seen = count.clone();
    let result = TraceData::new((0..1_000_000).map(move |_| {
        seen.set(seen.get() + 1);
        record("span")
    }));
    assert_eq!(result.unwrap_err(), TraceError::TooManyRecords);
    assert_eq!(count.get(), MAX_TRACE_RECORDS + 1);
}
#[test]
fn utf8_prefix_is_explicit_bounded_and_redacted_payload_contains_no_original() {
    let text = "🙂 Hello";
    assert_eq!(TraceText::prefix(text, 3).unwrap().as_str(), "");
    let prefix = TraceText::prefix(text, 4).unwrap();
    assert_eq!(prefix.as_str(), "🙂");
    assert!(prefix.is_truncated());
    let prefix = TraceText::prefix(text, 100).unwrap();
    assert_eq!(prefix.as_str(), text);
    assert!(!prefix.is_truncated());
    assert!(TraceText::prefix(text, MAX_TRACE_PAYLOAD_BYTES + 1).is_err());
    assert!(TraceText::new(&"x".repeat(MAX_TRACE_PAYLOAD_BYTES + 1)).is_err());
    let raw = "x".repeat(1_000_000);
    assert_eq!(TraceText::prefix(&raw, 32).unwrap().as_str().len(), 32);
    let data = record("tool")
        .input(TracePayload::Redacted)
        .output(TracePayload::Public(
            TraceText::new("<script>literal</script>").unwrap(),
        ));
    assert!(matches!(data.input_payload(), Some(TracePayload::Redacted)));
    assert!(
        matches!(data.output_payload(),Some(TracePayload::Public(v))if v.as_str()=="<script>literal</script>")
    );
}
#[test]
fn whole_forest_accepts_out_of_order_parents_and_rejects_duplicates_missing_and_cycles() {
    let data = TraceData::new([record("child").parent(key("root")), record("root")]).unwrap();
    assert_eq!(data.depth(0), Some(1));
    assert_eq!(data.depth(1), Some(0));
    assert_eq!(data.depth(2), None);
    assert_eq!(
        TraceData::new([record("same"), record("same")]).unwrap_err(),
        TraceError::DuplicateKey
    );
    assert_eq!(
        TraceData::new([record("a").parent(key("missing"))]).unwrap_err(),
        TraceError::MissingParent
    );
    assert_eq!(
        TraceData::new([record("a").parent(key("a"))]).unwrap_err(),
        TraceError::Cycle
    );
    assert_eq!(
        TraceData::new([record("a").parent(key("b")), record("b").parent(key("a"))]).unwrap_err(),
        TraceError::Cycle
    );
}
#[test]
fn maximum_hierarchy_depth_is_finite_and_not_a_recursive_stack_walk() {
    let make = |n| {
        (0..n).map(|i| {
            if i == 0 {
                record("k0")
            } else {
                record(&format!("k{i}")).parent(key(&format!("k{}", i - 1)))
            }
        })
    };
    let data = TraceData::new(make(17)).unwrap();
    assert_eq!(data.depth(16), Some(16));
    assert_eq!(TraceData::new(make(18)).unwrap_err(), TraceError::TooDeep);
}
#[test]
fn complete_text_budget_counts_both_payloads_keys_titles_and_parents() {
    let payload =
        TracePayload::Public(TraceText::new(&"x".repeat(MAX_TRACE_PAYLOAD_BYTES)).unwrap());
    let make = |n| {
        (0..n).map(|i| {
            record(&format!("s{i}"))
                .input(payload.clone())
                .output(payload.clone())
        })
    };
    assert_eq!(TraceData::new(make(3)).unwrap().records().len(), 3);
    assert_eq!(
        TraceData::new(make(4)).unwrap_err(),
        TraceError::TooManyBytes
    );
    assert_eq!(
        TraceData::new((0..MAX_TRACE_RECORDS).map(|i| record(&format!("s{i}"))))
            .unwrap()
            .records()
            .len(),
        MAX_TRACE_RECORDS
    );
}
#[test]
fn names_identifiers_literal_text_and_elapsed_values_have_explicit_boundaries() {
    for key in ["", "with space", "汉字", &"x".repeat(65)] {
        assert!(TraceKey::new(key).is_err());
    }
    for title in ["", " ", "line\nline", "nul\0", "cr\r"] {
        assert!(TraceRecord::new(key("a"), TraceKind::Tool, title).is_err());
    }
    assert!(TraceText::new("line\n\tmore").is_ok());
    assert!(TraceText::new("cr\r").is_err());
    assert!(TraceText::new("\u{1b}[31m").is_err());
    assert!(TraceText::new("bell\u{7}").is_err());
    assert!(record("duration").elapsed_millis(Some(604_800_000)).is_ok());
    assert!(
        record("duration")
            .elapsed_millis(Some(604_800_001))
            .is_err()
    );
    let r = TraceRecord::new(key("worker"), TraceKind::Subagent, "Application worker")
        .unwrap()
        .status(TraceStatus::Running)
        .elapsed_millis(Some(123))
        .unwrap();
    assert_eq!(r.kind().label(), "Subagent");
    assert_eq!(r.state().label(), "Running");
    assert_eq!(r.elapsed(), Some(123));
}
#[test]
fn current_indices_preserve_application_sibling_order_and_do_not_retain_removed_records() {
    let data = TraceData::new([
        record("b").parent(key("root")),
        record("root"),
        record("a").parent(key("root")),
        record("other"),
    ])
    .unwrap();
    assert_eq!(
        data.root_keys().map(|k| k.as_str()).collect::<Vec<_>>(),
        ["root", "other"]
    );
    assert_eq!(
        data.child_keys(&key("root"))
            .map(|k| k.as_str())
            .collect::<Vec<_>>(),
        ["b", "a"]
    );
    assert_eq!(data.record(&key("a")).unwrap().title(), "a");
    assert!(data.record(&key("absent")).is_none());
    assert_eq!(data.child_keys(&key("absent")).count(), 0);
    let next = TraceData::new([record("next")]).unwrap();
    assert!(next.record(&key("root")).is_none());
    assert_eq!(next.root_keys().count(), 1);
}
#[test]
fn stack_frames_are_literal_bounded_keyed_locations_without_implicit_urls() {
    let frame = StackFrame::new(key("f1"), "parse<script>")
        .unwrap()
        .location(StackLocation::new("D:/private/src.rs", 18, Some(4)).unwrap());
    let data = StackTraceData::new(
        TracePayload::Redacted,
        [
            frame.clone(),
            StackFrame::new(key("f2"), "parse<script>").unwrap(),
        ],
    )
    .unwrap();
    assert_eq!(
        data.frames()[0].source().unwrap().file(),
        "D:/private/src.rs"
    );
    assert_eq!(data.frames()[0].source().unwrap().line(), 18);
    assert_eq!(data.frames()[0].source().unwrap().column(), Some(4));
    assert_eq!(data.frames()[0].function(), "parse<script>");
    assert_eq!(
        StackTraceData::new(TracePayload::Redacted, [frame.clone(), frame]).unwrap_err(),
        TraceError::DuplicateKey
    );
    let seen = Rc::new(Cell::new(0));
    let count = seen.clone();
    assert_eq!(
        StackTraceData::new(
            TracePayload::Redacted,
            (0..1_000_000).map(move |_| {
                count.set(count.get() + 1);
                StackFrame::new(key("f"), "function").unwrap()
            })
        )
        .unwrap_err(),
        TraceError::TooManyFrames
    );
    assert_eq!(seen.get(), MAX_STACK_FRAMES + 1);
    assert!(StackLocation::new("source", 0, None).is_err());
    assert!(StackLocation::new("source", 1, Some(0)).is_err());
    assert!(StackLocation::new("with\nnewline", 1, None).is_err());
}
