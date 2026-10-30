#![cfg(feature = "dates")]
use glory::prelude::*;
use glory::render::{
    EventKind, InputEdit, InputEditIntent, InputEditRejection, InputProps, NodeId, Props,
    TextSelection,
};
use sapid::prelude::*;
use std::{cell::RefCell, rc::Rc};
fn d(s: &str) -> CivilDate {
    CivilDate::parse_iso(s).unwrap()
}
fn p() -> DatesProvider {
    DatesProvider::new(d("2024-02-29"))
}
fn props(app: &App, id: NodeId) -> InputProps {
    let t = app.tree();
    let Props::Input(p) = t.node(id).unwrap().descriptor.view().props else {
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
fn incomplete_invalid_empty_and_valid_drafts_are_distinct_from_committed_civil_dates() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let value = app.scope().cage(None);
    let calls = app.scope().cage(0usize);
    app.mount(
        date_input("draft", p())
            .with_picker(false)
            .node_ref(r)
            .value(value)
            .on_value_change(move |v| {
                value.revise(|old| *old = v);
                calls.revise(|n| *n += 1);
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    for draft in ["2024-", "2023-02-29"] {
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
    edit(&app, id, "2024-02-29", false);
    app.flush().unwrap();
    assert_eq!(value.get(), Some(d("2024-02-29")));
    assert_eq!(calls.get(), 1);
    assert!(
        !app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .invalid
    );
    edit(&app, id, "", false);
    app.flush().unwrap();
    assert_eq!(value.get(), None);
    assert_eq!(calls.get(), 2);
}
#[test]
fn outside_bounds_and_invalid_commit_preserve_text_and_business_value_without_faulting() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let rejected = app.scope().cage(0usize);
    let bounds = DateBounds::new(Some(d("2024-02-01")), Some(d("2024-02-29"))).unwrap();
    app.mount(
        date_input("bounds", p().bounds(bounds))
            .with_picker(false)
            .node_ref(r)
            .default_value(Some(d("2024-02-26")))
            .on_reject(move |s| {
                assert!(matches!(s, DateDraftStatus::OutOfBounds(_)));
                rejected.revise(|n| *n += 1);
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    edit(&app, id, "2024-03-01", false);
    app.flush().unwrap();
    commit_edit(&app, id, "2024-03-01");
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "2024-03-01");
    assert_eq!(rejected.get(), 1);
    edit(&app, id, "2024-02-29", false);
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "2024-02-29");
}
#[test]
fn custom_parser_keeps_native_spelling_until_explicit_commit_and_callbacks_precede_generic() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let order = Rc::new(RefCell::new(Vec::new()));
    let a = order.clone();
    let b = order.clone();
    let c = order.clone();
    app.mount(
        date_input("custom", p())
            .with_picker(false)
            .node_ref(r)
            .parser(|v| {
                if v == "29/02/2024" {
                    DateParse::Date(d("2024-02-29"))
                } else {
                    parse_iso_date_draft(v)
                }
            })
            .on_value_change(move |_| a.borrow_mut().push("change"))
            .on_commit(move |_| b.borrow_mut().push("commit"))
            .on(EventKind::InputEdit, move |_| {
                c.borrow_mut().push("generic")
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    edit(&app, id, "29/02/2024", false);
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "29/02/2024");
    assert_eq!(order.borrow().as_slice(), ["change", "generic"]);
    commit_edit(&app, id, "29/02/2024");
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "2024-02-29");
    assert_eq!(order.borrow().as_slice(), ["change", "generic", "commit"]);
}
#[test]
fn controlled_refusal_reconciles_valid_proposals_after_the_whole_tick_and_all_edits_are_observed() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let value = app.scope().cage(Some(d("2024-02-26")));
    let proposals = Rc::new(RefCell::new(Vec::new()));
    let out = proposals.clone();
    app.mount(
        date_input("refused", p())
            .with_picker(false)
            .node_ref(r)
            .value(value)
            .on_value_change(move |v| out.borrow_mut().push(v)),
    )
    .unwrap();
    let id = r.get().unwrap();
    app.scope()
        .batch(|| {
            edit(&app, id, "2024-02-27", false);
            edit(&app, id, "2024-02-29", false);
        })
        .unwrap();
    assert_eq!(
        proposals.borrow().as_slice(),
        [Some(d("2024-02-27")), Some(d("2024-02-29"))]
    );
    assert_eq!(props(&app, id).value, "2024-02-26");
    edit(&app, id, "2024-", false);
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "2024-");
    assert_eq!(value.get(), Some(d("2024-02-26")));
}
#[test]
fn oversized_host_capture_never_invokes_parser_and_recovers_with_small_typed_diagnostic() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let parses = app.scope().cage(0usize);
    let rejections = app.scope().cage(0usize);
    app.mount(
        date_input("budget", p())
            .with_picker(false)
            .node_ref(r)
            .default_value(Some(d("2024-02-29")))
            .parser(move |v| {
                parses.revise(|n| *n += 1);
                parse_iso_date_draft(v)
            })
            .on_reject(move |s| {
                assert_eq!(s, DateDraftStatus::TooLong);
                rejections.revise(|n| *n += 1);
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
    assert_eq!(parses.get(), 0);
    assert_eq!(rejections.get(), 1);
    assert_eq!(props(&app, id).value, "2024-02-29");
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
    edit(&app, id, "2024-02-28", false);
    app.flush().unwrap();
    assert_eq!(parses.get(), 1);
}
#[test]
fn composition_is_host_owned_terminal_maps_once_and_superseded_revision_cannot_overwrite_program_value()
 {
    let app = App::memory();
    let r = app.scope().cage(None);
    let value = app.scope().cage(None);
    let calls = app.scope().cage(0usize);
    app.mount(
        date_input("ime", p())
            .with_picker(false)
            .node_ref(r)
            .value(value)
            .on_value_change(move |v| {
                calls.revise(|n| *n += 1);
                value.revise(|old| *old = v);
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    app.dispatch(id, Event::Composition { active: true })
        .unwrap();
    edit(&app, id, "2024-", true);
    app.flush().unwrap();
    assert_eq!(calls.get(), 0);
    app.scope()
        .batch(|| {
            app.dispatch(id, Event::Composition { active: false })
                .unwrap();
            edit(&app, id, "2024-02-29", false);
            edit(&app, id, "2024-02-29", false);
        })
        .unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(props(&app, id).value, "2024-02-29");
    app.dispatch(id, Event::Composition { active: true })
        .unwrap();
    let revision = props(&app, id).revision;
    value.revise(|v| *v = Some(d("2024-03-01")));
    app.flush().unwrap();
    assert!(
        !app.dispatch(
            id,
            Event::InputEdit(InputEdit::Value {
                value: "2024-02-28".into(),
                revision,
                selection: TextSelection {
                    start: 10,
                    end: 10,
                    ..Default::default()
                },
                before: None,
                intent: InputEditIntent::Replace,
                composing: false
            })
        )
        .unwrap()
    );
    app.dispatch(id, Event::Composition { active: false })
        .unwrap();
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "2024-03-01");
    assert_eq!(calls.get(), 1);
}
#[test]
fn external_invalid_recovery_is_silent_and_readonly_disabled_retirement_drop_edits() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let value = app.scope().cage(Some(d("2024-02-26")));
    let readonly = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    let visible = app.scope().cage(true);
    let calls = app.scope().cage(0usize);
    app.mount(glory::ui::when(visible, move || {
        date_input(
            "owned",
            p().bounds(DateBounds::new(Some(d("2024-02-01")), Some(d("2024-02-29"))).unwrap()),
        )
        .with_picker(false)
        .node_ref(r)
        .value(value)
        .readonly(readonly)
        .enabled(enabled)
        .on_value_change(move |_| calls.revise(|n| *n += 1))
    }))
    .unwrap();
    let id = r.get().unwrap();
    value.revise(|v| *v = Some(d("2024-03-01")));
    assert!(app.flush().is_err());
    assert_eq!(props(&app, id).value, "2024-02-26");
    value.revise(|v| *v = Some(d("2024-02-29")));
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "2024-02-29");
    assert_eq!(calls.get(), 0);
    readonly.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(
        !app.dispatch(
            id,
            Event::InputEdit(InputEdit::Rejected {
                revision: props(&app, id).revision,
                reason: InputEditRejection::TooLong
            })
        )
        .unwrap()
    );
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    visible.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(r.get().is_none());
    assert!(
        !app.dispatch(
            id,
            Event::Commit {
                value: "2024-02-28".into(),
                revision: 0
            }
        )
        .unwrap()
    );
}
#[test]
fn field_relations_and_default_calendar_popup_share_one_real_editable_input() {
    let app = App::memory();
    let r = app.scope().cage(None);
    app.mount(
        field("civil")
            .label("Civil date")
            .form_name("civil")
            .description("ISO date")
            .required(true)
            .control(move |f| date_input("field-date", p()).field(f).node_ref(r)),
    )
    .unwrap();
    let id = r.get().unwrap();
    let t = app.tree();
    let v = t.node(id).unwrap().descriptor.view();
    assert!(!v.readonly());
    let s = v.semantics.as_ref().unwrap();
    assert!(s.required && s.labelled_by.is_some() && s.described_by.is_some());
    assert_eq!(s.form_control.as_ref().unwrap().name, "civil");
    drop(t);
    let mut pending = app.tree().roots().to_vec();
    let mut trigger = None;
    let mut count = 0;
    while let Some(n) = pending.pop() {
        let t = app.tree();
        let node = t.node(n).unwrap();
        pending.extend(node.children.iter().copied());
        if matches!(node.descriptor.view().props, Props::Input(_)) {
            count += 1;
        }
        if matches!(node.descriptor.view().props,Props::Button(p) if p.label=="Choose date") {
            trigger = Some(n);
        }
    }
    assert_eq!(count, 1);
    app.dispatch(trigger.unwrap(), Event::Activate).unwrap();
    app.flush().unwrap();
    let mut pending = app.tree().roots().to_vec();
    let mut day = None;
    while let Some(n) = pending.pop() {
        let t = app.tree();
        let node = t.node(n).unwrap();
        pending.extend(node.children.iter().copied());
        if node.descriptor.view().name.as_deref() == Some("Thursday, February 29, 2024") {
            day = Some(n);
        }
    }
    app.dispatch(day.unwrap(), Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "2024-02-29");
}

#[test]
fn later_incomplete_edit_in_the_same_tick_retains_the_draft_after_an_earlier_valid_publish() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    let value = app.scope().cage(Some(d("2024-02-26")));
    app.mount(
        date_input("cursor", p())
            .with_picker(false)
            .node_ref(reference)
            .value(value)
            .on_value_change(move |v| value.revise(|old| *old = v)),
    )
    .unwrap();
    let id = reference.get().unwrap();
    app.scope()
        .batch(|| {
            edit(&app, id, "2024-02-27", false);
            edit(&app, id, "2024-", false);
        })
        .unwrap();
    assert_eq!(value.get(), Some(d("2024-02-27")));
    assert_eq!(props(&app, id).value, "2024-");
    edit(&app, id, "2024-0", false);
    value.revise(|v| *v = Some(d("2024-02-28")));
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "2024-02-28");
}

#[test]
fn calendar_selection_clears_the_published_draft_diagnostic_before_the_civil_value_callback() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    let draft = app.scope().cage(DateDraftStatus::Empty);
    let order = app.scope().cage(true);
    app.mount(
        date_input("calendar-draft", p())
            .node_ref(reference)
            .on_draft_change(move |v| draft.revise(|old| *old = v.status))
            .on_value_change(move |v| {
                order.revise(|ok| {
                    *ok &= draft.get_untracked()
                        == v.map_or(DateDraftStatus::Empty, DateDraftStatus::Valid)
                })
            }),
    )
    .unwrap();
    let id = reference.get().unwrap();
    edit(&app, id, "bad", false);
    app.flush().unwrap();
    assert_eq!(draft.get(), DateDraftStatus::Invalid);
    let mut pending = app.tree().roots().to_vec();
    let mut trigger = None;
    let mut day = None;
    while let Some(n) = pending.pop() {
        let t = app.tree();
        let node = t.node(n).unwrap();
        pending.extend(node.children.iter().copied());
        if matches!(node.descriptor.view().props,Props::Button(p) if p.label=="Choose date") {
            trigger = Some(n);
        }
        if node.descriptor.view().name.as_deref() == Some("Thursday, February 29, 2024") {
            day = Some(n);
        }
    }
    app.dispatch(trigger.unwrap(), Event::Activate).unwrap();
    app.flush().unwrap();
    app.dispatch(day.unwrap(), Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(draft.get(), DateDraftStatus::Valid(d("2024-02-29")));
    assert!(order.get());
    assert_eq!(props(&app, id).value, "2024-02-29");
}

#[test]
fn direct_oversized_commit_uses_a_finite_rejection_and_restores_accepted_date() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let rejects = app.scope().cage(0usize);
    app.mount(
        date_input("direct-budget", p())
            .with_picker(false)
            .node_ref(r)
            .default_value(Some(d("2024-02-29")))
            .on_reject(move |v| {
                assert_eq!(v, DateDraftStatus::TooLong);
                rejects.revise(|n| *n += 1)
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    commit_edit(&app, id, &"1".repeat(1_000_000));
    app.flush().unwrap();
    assert_eq!(rejects.get(), 1);
    assert_eq!(props(&app, id).value, "2024-02-29");
    edit(&app, id, "2024-03-01", false);
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "2024-03-01");
}
