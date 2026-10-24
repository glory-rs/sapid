#![cfg(feature = "chat")]
use glory::{
    prelude::*,
    render::{DocumentPart, EventKind, NodeId, Props},
};
use sapid::chat::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn key(raw: &str) -> TraceKey {
    TraceKey::new(raw).unwrap()
}
fn record(raw: &str, kind: TraceKind) -> TraceRecord {
    TraceRecord::new(key(raw), kind, raw)
        .unwrap()
        .status(TraceStatus::Running)
        .input(TracePayload::Public(
            TraceText::new("literal <script>x</script>\ninput").unwrap(),
        ))
        .output(TracePayload::Redacted)
}
fn data() -> TraceData {
    TraceData::new([
        record("root", TraceKind::Span),
        record("tool", TraceKind::Tool).parent(key("root")),
        record("agent", TraceKind::Subagent).parent(key("root")),
    ])
    .unwrap()
}
fn expansion(values: &[&str]) -> TraceExpansion {
    TraceExpansion::new(values.iter().map(|v| key(v))).unwrap()
}
fn all(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut queue = tree.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = queue.pop() {
        queue.extend(&tree.node(id).unwrap().children);
        out.push(id);
    }
    out
}
fn class(app: &App, name: &str) -> Vec<NodeId> {
    all(app)
        .into_iter()
        .filter(|&id| {
            app.tree()
                .node(id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|v| v == name)
        })
        .collect()
}
fn button(app: &App, prefix: &str) -> NodeId {
    all(app).into_iter().find(|&id| matches!(app.tree().node(id).unwrap().descriptor.view().props, Props::Button(p) if p.label.starts_with(prefix))).unwrap()
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
#[test]
fn real_hierarchy_literal_payload_and_redaction_use_lists_and_independent_disclosures() {
    let app = App::memory();
    app.mount(trace("main", data()).default_expanded(expansion(&["root", "tool"])))
        .unwrap();
    assert_eq!(class(&app, "sapid-trace-record").len(), 3);
    assert_eq!(class(&app, "sapid-trace-payload").len(), 4);
    assert!(text(&app).contains("<script>x</script>"));
    assert!(text(&app).contains("[redacted]"));
    for id in class(&app, "sapid-trace-list") {
        assert_eq!(
            app.tree()
                .node(id)
                .unwrap()
                .descriptor
                .view()
                .document_part(),
            Some(DocumentPart::OrderedList)
        );
    }
    for id in class(&app, "sapid-trace-record") {
        assert_eq!(
            app.tree()
                .node(id)
                .unwrap()
                .descriptor
                .view()
                .document_part(),
            Some(DocumentPart::ListItem)
        );
    }
    app.dispatch(button(&app, "Tool · tool"), Event::Activate)
        .unwrap();
    assert_eq!(class(&app, "sapid-trace-payload").len(), 2);
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
}
#[test]
fn controlled_refusal_keeps_payload_closed_and_typed_precedes_generic() {
    let app = App::memory();
    let expanded = app.scope().cage(TraceExpansion::default());
    let seen = Rc::new(RefCell::new(vec![]));
    let typed = seen.clone();
    let generic = seen.clone();
    app.mount(
        trace("refusal", data())
            .expanded(expanded)
            .on_expand_change(move |p| {
                assert!(p.open);
                assert!(p.expanded.contains(&key("root")));
                typed.borrow_mut().push("typed");
            })
            .on(EventKind::Activate, move |_| {
                generic.borrow_mut().push("generic")
            }),
    )
    .unwrap();
    let trigger = button(&app, "Span · root");
    app.dispatch(trigger, Event::Activate).unwrap();
    assert_eq!(*seen.borrow(), ["typed", "generic"]);
    assert!(class(&app, "sapid-trace-payload").is_empty());
    expanded.revise(|v| *v = expansion(&["root"]));
    app.flush().unwrap();
    assert_eq!(class(&app, "sapid-trace-payload").len(), 2);
    assert_eq!(seen.borrow().len(), 2);
}
#[test]
fn keyed_title_updates_keep_live_shell_and_removed_expansion_is_not_visited_history() {
    let app = App::memory();
    let source = app.scope().cage(data());
    app.mount(trace("keys", source).default_expanded(expansion(&["root", "tool"])))
        .unwrap();
    let trigger = button(&app, "Tool · tool");
    source.revise(|d| {
        *d = TraceData::new([
            record("root", TraceKind::Span),
            TraceRecord::new(key("tool"), TraceKind::Tool, "Updated tool")
                .unwrap()
                .parent(key("root")),
        ])
        .unwrap()
    });
    app.flush().unwrap();
    assert_eq!(trigger, button(&app, "Tool · Updated tool"));
    source.revise(|d| *d = TraceData::new([record("root", TraceKind::Span)]).unwrap());
    app.flush().unwrap();
    assert!(app.tree().node(trigger).is_err());
    source.revise(|d| *d = data());
    app.flush().unwrap();
    let new = button(&app, "Tool · tool");
    assert_ne!(trigger, new);
    assert!(
        !app.tree()
            .node(new)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .expanded
            .unwrap()
    );
}
#[test]
fn maximum_record_keys_do_not_overflow_internal_relations() {
    let app = App::memory();
    let row = record(&"a".repeat(64), TraceKind::Span);
    app.mount(trace("b".repeat(32), TraceData::new([row]).unwrap()))
        .unwrap();
    assert_eq!(class(&app, "sapid-trace-record").len(), 1);
    assert!(app.mount(trace("x".repeat(33), data())).is_err());
    assert!(
        app.mount(tool("x".repeat(1_000_000), record("t", TraceKind::Tool)))
            .is_err()
    );
    assert!(
        app.mount(subagent("x".repeat(33), record("a", TraceKind::Subagent)))
            .is_err()
    );
}
#[test]
fn tool_and_subagent_validate_kind_and_recover_accepted_record_without_callbacks() {
    let app = App::memory();
    let source = app.scope().cage(record("call", TraceKind::Tool));
    let calls = Rc::new(Cell::new(0));
    let observed = calls.clone();
    app.mount(
        tool("entry", source)
            .default_open(true)
            .on_open_change(move |_| observed.set(observed.get() + 1)),
    )
    .unwrap();
    let trigger = button(&app, "Tool · call");
    source.revise(|r| *r = record("agent", TraceKind::Subagent));
    assert!(app.flush().is_err());
    assert_eq!(trigger, button(&app, "Tool · call"));
    assert_eq!(calls.get(), 0);
    source
        .revise(|r| *r = TraceRecord::new(key("call"), TraceKind::Tool, "Recovered").unwrap());
    app.flush().unwrap();
    assert_eq!(trigger, button(&app, "Tool · Recovered"));
    let rejected = App::memory();
    assert!(
        rejected
            .mount(subagent("wrong", record("bad", TraceKind::Tool)))
            .is_err()
    );
}
#[test]
fn action_is_explicit_disabled_is_actual_and_retirement_releases_captures() {
    let app = App::memory();
    let enabled = app.scope().cage(true);
    let calls = Rc::new(Cell::new(0));
    let observed = calls.clone();
    let sentinel = Rc::new(());
    let weak = Rc::downgrade(&sentinel);
    app.mount(
        tool("action", record("tool", TraceKind::Tool))
            .default_open(true)
            .enabled(enabled)
            .on_action(move |a| {
                let _ = &sentinel;
                assert_eq!(a.key, key("tool"));
                observed.set(observed.get() + 1);
            }),
    )
    .unwrap();
    let action = button(&app, "Inspect record");
    app.dispatch(action, Event::Activate).unwrap();
    assert_eq!(calls.get(), 1);
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(
        !app.tree()
            .node(action)
            .unwrap()
            .descriptor
            .view()
            .props
            .enabled()
    );
    app.dispatch(action, Event::Activate).unwrap();
    assert_eq!(calls.get(), 1);
    app.clear().unwrap();
    assert!(weak.upgrade().is_none());
    assert_eq!(app.stats().nodes, 0);
}
fn stack() -> StackTraceData {
    StackTraceData::new(
        TracePayload::Public(TraceText::new("Error <tag>").unwrap()),
        [
            StackFrame::new(key("one"), "parse<script>")
                .unwrap()
                .location(StackLocation::new("D:/secret/file.rs", 12, Some(3)).unwrap()),
            StackFrame::new(key("two"), "caller").unwrap(),
        ],
    )
    .unwrap()
}
#[test]
fn stack_is_literal_ordered_display_without_inferred_links_and_current_location_action() {
    let app = App::memory();
    let source = app.scope().cage(stack());
    let calls = Rc::new(RefCell::new(vec![]));
    let typed = calls.clone();
    let generic = calls.clone();
    app.mount(
        stack_trace(source)
            .on_location(move |r| typed.borrow_mut().push(r.location.line().to_string()))
            .on(EventKind::Activate, move |_| {
                generic.borrow_mut().push("generic".into())
            }),
    )
    .unwrap();
    assert_eq!(class(&app, "sapid-stack-frame").len(), 2);
    assert!(text(&app).contains("parse<script>"));
    assert!(!all(&app).iter().any(|&id| matches!(
        app.tree().node(id).unwrap().descriptor.view().props,
        Props::Link(_)
    )));
    let location = button(&app, "D:/secret/file.rs:12:3");
    source.revise(|d| {
        *d = StackTraceData::new(
            TracePayload::Redacted,
            [StackFrame::new(key("one"), "new")
                .unwrap()
                .location(StackLocation::new("D:/secret/file.rs", 20, None).unwrap())],
        )
        .unwrap()
    });
    app.flush().unwrap();
    assert_eq!(location, button(&app, "D:/secret/file.rs:20"));
    app.dispatch(location, Event::Activate).unwrap();
    assert_eq!(*calls.borrow(), ["20", "generic"]);
    assert!(text(&app).contains("[redacted]"));
}
#[test]
fn passive_stack_has_no_actions_and_unstyled_preserves_payload_scrolling() {
    let app = App::memory();
    app.mount(stack_trace(stack()).unstyled()).unwrap();
    assert!(!all(&app).iter().any(|&id| matches!(
        app.tree().node(id).unwrap().descriptor.view().props,
        Props::Button(_)
    )));
    let id = class(&app, "sapid-stack-message")[0];
    let tree = app.tree();
    let style = &tree.node(id).unwrap().descriptor.view().style;
    assert_eq!(style.background, None);
    assert_eq!(style.max_height, Some(glory::style::Limit::Px(240.)));
    assert_eq!(style.overflow_y, Some(glory::style::Overflow::Auto));
    for id in class(&app, "sapid-stack-function")
        .into_iter()
        .chain(class(&app, "sapid-stack-location"))
    {
        assert_eq!(
            app.tree().node(id).unwrap().descriptor.view().style.display,
            Some(glory::style::Display::Block),
            "passive function and location must remain distinct lines"
        );
    }
}
#[test]
fn expansion_builder_consumes_only_bounded_sentinel_and_invalid_presentation_is_rejected() {
    let seen = Cell::new(0);
    assert_eq!(
        TraceExpansion::new((0..1_000_000).map(|_| {
            seen.set(seen.get() + 1);
            key("repeat")
        }))
        .unwrap_err(),
        TraceError::TooManyRecords
    );
    assert_eq!(seen.get(), 257);
    let app = App::memory();
    assert!(
        app.mount(trace("height", data()).payload_height(f64::NAN))
            .is_err()
    );
    assert!(
        app.mount(stack_trace(stack()).payload_height(641.))
            .is_err()
    );
}
#[test]
fn keep_mounted_disclosure_preserves_body_identity_but_hidden_actions_are_inactive() {
    let app = App::memory();
    app.mount(
        tool("keep", record("tool", TraceKind::Tool))
            .default_open(true)
            .content_policy(sapid::ContentPolicy::KeepMounted)
            .on_action(|_| {}),
    )
    .unwrap();
    let action = button(&app, "Inspect record");
    let trigger = button(&app, "Tool · tool");
    app.dispatch(trigger, Event::Activate).unwrap();
    assert!(app.tree().node(action).is_ok());
    assert!(!app.dispatch(action, Event::Activate).unwrap());
    app.dispatch(trigger, Event::Activate).unwrap();
    assert_eq!(action, button(&app, "Inspect record"));
}
#[test]
fn action_can_clear_its_owner_and_drop_captures_that_write_without_stale_generic() {
    struct WriteOnDrop(Cage<usize>);
    impl Drop for WriteOnDrop {
        fn drop(&mut self) {
            self.0.revise(|v| *v += 1);
        }
    }
    let app = Rc::new(App::memory());
    let drops = app.scope().cage(0usize);
    let sentinel = WriteOnDrop(drops);
    let weak = Rc::downgrade(&app);
    let generic = Rc::new(Cell::new(0));
    let observed = generic.clone();
    app.mount(
        tool("reentrant", record("tool", TraceKind::Tool))
            .default_open(true)
            .on_action(move |_| {
                let _ = &sentinel;
                weak.upgrade().unwrap().clear().unwrap();
            })
            .on(EventKind::Activate, move |_| {
                observed.set(observed.get() + 1)
            }),
    )
    .unwrap();
    app.dispatch(button(&app, "Inspect record"), Event::Activate)
        .unwrap();
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(drops.get(), 1);
    assert_eq!(generic.get(), 0);
}
