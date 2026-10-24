#![cfg(feature = "dates")]
use glory::prelude::*;
use glory::render::{
    EventKind, InputEdit, InputEditIntent, InputEditRejection, InputProps, NodeId, Props,
    TextSelection,
};
use sapid::prelude::*;
use std::{cell::RefCell, rc::Rc};
fn t(s: &str) -> LocalTime {
    LocalTime::parse_iso(s).unwrap()
}
fn props(app: &App, id: NodeId) -> InputProps {
    let tree = app.tree();
    let Props::Input(p) = tree.node(id).unwrap().descriptor.view().props else {
        panic!()
    };
    p.clone()
}
fn edit(app: &App, id: NodeId, value: &str, composing: bool) {
    app.dispatch(
        id,
        Event::InputEdit(InputEdit::Value {
            value: value.into(),
            revision: props(app, id).revision,
            selection: TextSelection {
                start: value.encode_utf16().count(),
                end: value.encode_utf16().count(),
                ..Default::default()
            },
            before: None,
            intent: InputEditIntent::Replace,
            composing,
        }),
    )
    .unwrap();
}
fn commit_edit(app: &App, id: NodeId, value: &str) {
    app.dispatch(
        id,
        Event::Commit {
            value: value.into(),
            revision: props(app, id).revision,
        },
    )
    .unwrap();
}
#[test]
fn empty_incomplete_invalid_and_valid_time_drafts_have_distinct_business_values() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let value = app.scope().cage(None);
    let calls = app.scope().cage(0usize);
    app.mount(
        time_input("draft")
            .node_ref(r)
            .value(value)
            .on_value_change(move |v| {
                value.revise(|old| *old = v);
                calls.revise(|n| *n += 1)
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    for draft in ["12:", "24:00"] {
        edit(&app, id, draft, false);
        app.flush().unwrap();
        assert_eq!(props(&app, id).value, draft);
        assert_eq!(value.get(), None);
        assert_eq!(calls.get(), 0);
    }
    assert!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .invalid
    );
    edit(&app, id, "23:59", false);
    app.flush().unwrap();
    assert_eq!(value.get(), Some(t("23:59")));
    assert_eq!(calls.get(), 1);
    edit(&app, id, "", false);
    app.flush().unwrap();
    assert_eq!(value.get(), None);
    assert_eq!(calls.get(), 2);
}
#[test]
fn bounds_do_not_clamp_edit_or_commit_and_recovery_is_not_an_app_fault() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let rejects = app.scope().cage(0usize);
    app.mount(
        time_input("bounds")
            .bounds(TimeBounds::new(Some(t("09:00")), Some(t("17:00"))).unwrap())
            .default_value(Some(t("10:00")))
            .node_ref(r)
            .on_reject(move |v| {
                assert_eq!(v, TimeDraftStatus::OutOfBounds(t("08:59")));
                rejects.revise(|n| *n += 1)
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    edit(&app, id, "08:59", false);
    app.flush().unwrap();
    commit_edit(&app, id, "08:59");
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "08:59");
    assert_eq!(rejects.get(), 1);
    edit(&app, id, "09:00", false);
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "09:00");
}
#[test]
fn twelve_hour_seconds_chinese_and_callback_order_use_exact_finite_format() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let value = app.scope().cage(Some(t("00:00:00")));
    let order = Rc::new(RefCell::new(Vec::new()));
    let a = order.clone();
    let b = order.clone();
    let c = order.clone();
    app.mount(
        time_input("clock")
            .format(TimeFormat::new(
                HourCycle::H12,
                TimePrecision::Second,
                DateLocale::ZhCn,
            ))
            .node_ref(r)
            .value(value)
            .on_draft_change(move |_| a.borrow_mut().push("draft"))
            .on_value_change(move |v| {
                value.revise(|old| *old = v);
                b.borrow_mut().push("value")
            })
            .on(EventKind::InputEdit, move |_| {
                c.borrow_mut().push("generic")
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    assert_eq!(props(&app, id).value, "12:00:00 上午");
    edit(&app, id, "12:13:14 下午", false);
    app.flush().unwrap();
    assert_eq!(value.get(), Some(t("12:13:14")));
    assert_eq!(order.borrow().as_slice(), ["draft", "value", "generic"]);
}
#[test]
fn controlled_refusal_external_invalid_precision_and_recovery_use_accepted_state() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let value = app.scope().cage(Some(t("10:00")));
    let calls = app.scope().cage(0usize);
    app.mount(
        time_input("controlled")
            .node_ref(r)
            .value(value)
            .on_value_change(move |_| calls.revise(|n| *n += 1)),
    )
    .unwrap();
    let id = r.get().unwrap();
    edit(&app, id, "11:00", false);
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "10:00");
    assert_eq!(calls.get(), 1);
    value.revise(|v| *v = Some(t("12:00:01")));
    assert!(app.flush().is_err());
    assert_eq!(props(&app, id).value, "10:00");
    assert_eq!(calls.get(), 1);
    value.revise(|v| *v = Some(t("13:00")));
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "13:00");
    assert_eq!(calls.get(), 1);
    let bad = App::memory();
    assert!(
        bad.mount(time_input("invalid").default_value(Some(t("10:00:01"))))
            .is_err()
    );
}
#[test]
fn same_tick_valid_then_incomplete_preserves_the_last_editor_draft() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let value = app.scope().cage(None);
    app.mount(
        time_input("batch")
            .node_ref(r)
            .value(value)
            .on_value_change(move |v| value.revise(|old| *old = v)),
    )
    .unwrap();
    let id = r.get().unwrap();
    edit(&app, id, "12:34", false);
    edit(&app, id, "12:", false);
    app.flush().unwrap();
    assert_eq!(value.get(), Some(t("12:34")));
    assert_eq!(props(&app, id).value, "12:");
}
#[test]
fn composition_does_not_format_preedit_terminal_echo_or_allow_old_revision_results() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let value = app.scope().cage(None);
    let calls = app.scope().cage(0usize);
    app.mount(
        time_input("ime")
            .node_ref(r)
            .value(value)
            .on_value_change(move |v| {
                value.revise(|old| *old = v);
                calls.revise(|n| *n += 1)
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    let revision = props(&app, id).revision;
    app.dispatch(id, Event::Composition { active: true })
        .unwrap();
    edit(&app, id, "正在输入", true);
    app.flush().unwrap();
    assert_eq!(calls.get(), 0);
    assert_eq!(value.get(), None);
    app.scope()
        .batch(|| {
            app.dispatch(id, Event::Composition { active: false })
                .unwrap();
            edit(&app, id, "12:34", false);
            edit(&app, id, "12:34", false);
        })
        .unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(value.get(), Some(t("12:34")));
    value.revise(|v| *v = Some(t("14:00")));
    app.flush().unwrap();
    app.dispatch(
        id,
        Event::Commit {
            value: "01:23".into(),
            revision,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "14:00");
}
#[test]
fn overbudget_capture_readonly_disabled_and_owner_cleanup_stay_finite() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let ro = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    let rejects = app.scope().cage(0usize);
    app.mount(
        time_input("bounded")
            .default_value(Some(t("10:00")))
            .readonly(ro)
            .enabled(enabled)
            .node_ref(r)
            .on_reject(move |reason| {
                assert_eq!(reason, TimeDraftStatus::TooLong);
                rejects.revise(|n| *n += 1)
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    app.dispatch(
        id,
        Event::InputEdit(InputEdit::Rejected {
            revision: props(&app, id).revision,
            reason: InputEditRejection::TooLong,
        }),
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(rejects.get(), 1);
    assert_eq!(props(&app, id).value, "10:00");
    ro.revise(|v| *v = true);
    app.flush().unwrap();
    edit(&app, id, "11:00", false);
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "10:00");
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(!props(&app, id).enabled);
    app.clear().unwrap();
    assert_eq!(r.get(), None);
    app.dispatch(id, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(app.stats().queries, 0);
}
#[test]
fn time_value_is_actual_text_with_empty_label_and_rejects_precision_loss() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let v = app.scope().cage(None);
    app.mount(time_value(v).node_ref(r).empty_label("No time"))
        .unwrap();
    let id = r.get().unwrap();
    assert!(
        matches!(app.tree().node(id).unwrap().descriptor.view().props,Props::Text(p) if p.content=="No time")
    );
    v.revise(|v| *v = Some(t("23:59")));
    app.flush().unwrap();
    assert!(
        matches!(app.tree().node(id).unwrap().descriptor.view().props,Props::Text(p) if p.content=="23:59")
    );
    let bad = App::memory();
    assert!(bad.mount(time_value(Some(t("12:00:01")))).is_err());
}

#[test]
fn direct_oversized_commit_is_rejected_before_copying_the_draft_and_recovers() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let rejects = app.scope().cage(0usize);
    app.mount(
        time_input("direct-budget")
            .node_ref(r)
            .default_value(Some(t("10:00")))
            .on_reject(move |v| {
                assert_eq!(v, TimeDraftStatus::TooLong);
                rejects.revise(|n| *n += 1)
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    commit_edit(&app, id, &"1".repeat(1_000_000));
    app.flush().unwrap();
    assert_eq!(rejects.get(), 1);
    assert_eq!(props(&app, id).value, "10:00");
    edit(&app, id, "11:00", false);
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "11:00");
}
