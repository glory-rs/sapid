use glory::prelude::*;
use glory::render::{EventKind, NodeId, Props, RangeCommand, SpinValue};
use sapid::{Decimal, NumberLocale, NumberValue, number_input};
use std::{cell::RefCell, rc::Rc};
fn d(v: &str) -> Decimal {
    Decimal::parse(v).unwrap()
}
fn text(app: &App, node: NodeId) -> String {
    let tree = app.tree();
    let Props::Input(p) = tree.node(node).unwrap().descriptor.view().props else {
        panic!("input")
    };
    p.value.clone()
}
fn spin(app: &App, node: NodeId) -> SpinValue {
    app.tree()
        .node(node)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .spinbutton
        .clone()
        .unwrap()
}
fn key(app: &App, node: NodeId, command: RangeCommand) {
    app.dispatch(
        node,
        Event::Spin {
            from: node,
            command,
        },
    )
    .unwrap();
}
fn edit(app: &App, node: NodeId, value: &str, commit_text: bool) {
    let tree = app.tree();
    let Props::Input(p) = tree.node(node).unwrap().descriptor.view().props else {
        panic!()
    };
    let revision = p.revision;
    drop(tree);
    app.dispatch(
        node,
        if commit_text {
            Event::Commit {
                value: value.into(),
                revision,
            }
        } else {
            Event::ValueChanged {
                value: value.into(),
                revision,
                composing: false,
            }
        },
    )
    .unwrap();
}
#[test]
fn exact_local_stepping_and_same_tick_commands_never_accumulate_binary_error() {
    let app = App::memory();
    let r = app.scope().cage(None);
    app.mount(
        number_input()
            .default_value(d("0.1"))
            .step(d("0.1"))
            .min(Some(d("0")))
            .max(Some(d("1")))
            .node_ref(r),
    )
    .unwrap();
    let id = r.get().unwrap();
    app.scope()
        .batch(|| {
            key(&app, id, RangeCommand::Increase);
            key(&app, id, RangeCommand::Increase)
        })
        .unwrap();
    assert_eq!(text(&app, id), "0.3");
    assert_eq!(spin(&app, id).value_text.as_deref(), Some("0.3"));
    key(&app, id, RangeCommand::Maximum);
    assert_eq!(text(&app, id), "1");
    assert!(!spin(&app, id).can_increase);
}
#[test]
fn editing_retains_lexemes_and_empty_incomplete_invalid_are_separate() {
    let app = App::memory();
    let r = app.scope().cage(None);
    app.mount(number_input().node_ref(r)).unwrap();
    let id = r.get().unwrap();
    for value in ["1.00", "-", "1,", "wrong", ""] {
        edit(&app, id, value, false);
        assert_eq!(text(&app, id), value);
        assert_eq!(
            spin(&app, id).value,
            if value == "1.00" { Some(1.) } else { None }
        );
    }
    edit(&app, id, "1.", true);
    assert_eq!(text(&app, id), "1");
    edit(&app, id, "-", true);
    assert_eq!(text(&app, id), "-");
}
#[test]
fn range_errors_remain_visible_until_explicit_commit_and_external_changes_are_silent() {
    let app = App::memory();
    let source = app.scope().cage(NumberValue::Decimal(d("20")));
    let r = app.scope().cage(None);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let out = calls.clone();
    app.mount(
        number_input()
            .value(source)
            .min(Some(d("0")))
            .max(Some(d("10")))
            .node_ref(r)
            .on_commit(move |v| {
                out.borrow_mut().push(v.clone());
                source.revise(|old| *old = v)
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    assert_eq!(text(&app, id), "20");
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
    assert!(!spin(&app, id).can_increase);
    edit(&app, id, "20", true);
    assert_eq!(source.get(), NumberValue::Decimal(d("10")));
    assert_eq!(text(&app, id), "10");
    source.revise(|old| *old = NumberValue::Text("2.00".into()));
    app.flush().unwrap();
    assert_eq!(text(&app, id), "2.00");
    assert_eq!(calls.borrow().len(), 1);
}
#[test]
fn controlled_rejection_restores_source_and_typed_callbacks_precede_observers() {
    let app = App::memory();
    let source = app.scope().cage(NumberValue::Decimal(d("2")));
    let r = app.scope().cage(None);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let a = calls.clone();
    let b = calls.clone();
    app.mount(
        number_input()
            .value(source)
            .node_ref(r)
            .on_value_change(move |v| a.borrow_mut().push(format!("typed:{v:?}")))
            .on(EventKind::Spin, move |_| {
                b.borrow_mut().push("observed".into())
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    app.scope()
        .batch(|| {
            key(&app, id, RangeCommand::Increase);
            key(&app, id, RangeCommand::Increase)
        })
        .unwrap();
    assert_eq!(text(&app, id), "2");
    assert_eq!(source.get(), NumberValue::Decimal(d("2")));
    assert_eq!(
        calls.borrow().as_slice(),
        [
            "typed:Decimal(Decimal { coefficient: 3, scale: 0 })",
            "observed",
            "typed:Decimal(Decimal { coefficient: 3, scale: 0 })",
            "observed"
        ]
    );
}
#[test]
fn commit_only_controlled_acceptance_is_a_same_tick_cursor() {
    let app = App::memory();
    let source = app.scope().cage(NumberValue::Decimal(d("2")));
    let r = app.scope().cage(None);
    app.mount(
        number_input()
            .value(source)
            .node_ref(r)
            .on_commit(move |v| source.revise(|old| *old = v)),
    )
    .unwrap();
    let id = r.get().unwrap();
    app.scope()
        .batch(|| {
            key(&app, id, RangeCommand::Increase);
            key(&app, id, RangeCommand::Increase)
        })
        .unwrap();
    assert_eq!(source.get(), NumberValue::Decimal(d("4")));
    assert_eq!(text(&app, id), "4");
}
#[test]
fn disabled_readonly_and_locale_updates_keep_real_input_identity() {
    let app = App::memory();
    let enabled = app.scope().cage(true);
    let readonly = app.scope().cage(false);
    let r = app.scope().cage(None);
    app.mount(
        number_input()
            .locale(NumberLocale::DeDe)
            .grouping(true)
            .default_value(d("1234.5"))
            .enabled(enabled)
            .readonly(readonly)
            .node_ref(r),
    )
    .unwrap();
    let id = r.get().unwrap();
    assert_eq!(text(&app, id), "1.234,5");
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(!spin(&app, id).can_increase);
    enabled.revise(|v| *v = true);
    readonly.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(!spin(&app, id).can_decrease);
    assert!(
        app.dispatch(
            id,
            Event::Spin {
                from: id,
                command: RangeCommand::Increase
            }
        )
        .is_err()
    );
    assert_eq!(r.get(), Some(id));
}
#[test]
fn bounded_invalid_sources_reject_before_clone_and_keep_last_accepted_state() {
    let app = App::memory();
    let source = app.scope().cage(NumberValue::Decimal(d("1")));
    let r = app.scope().cage(None);
    app.mount(number_input().value(source).node_ref(r)).unwrap();
    let id = r.get().unwrap();
    source.revise(|old| *old = NumberValue::Text("9".repeat(1_000_000)));
    assert!(app.flush().is_err());
    assert_eq!(text(&app, id), "1");
    source.revise(|old| *old = NumberValue::Decimal(d("3")));
    app.flush().unwrap();
    assert_eq!(text(&app, id), "3");
}
#[test]
fn composition_retains_bounded_draft_until_terminal_event_then_publishes_once() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let out = calls.clone();
    app.mount(
        number_input()
            .node_ref(r)
            .on_value_change(move |v| out.borrow_mut().push(v)),
    )
    .unwrap();
    let id = r.get().unwrap();
    app.dispatch(id, Event::Composition { active: true })
        .unwrap();
    let revision = {
        let tree = app.tree();
        let Props::Input(p) = tree.node(id).unwrap().descriptor.view().props else {
            panic!()
        };
        p.revision
    };
    for value in ["1", "1.00"] {
        app.dispatch(
            id,
            Event::ValueChanged {
                value: value.into(),
                composing: true,
                revision,
            },
        )
        .unwrap();
    }
    assert!(calls.borrow().is_empty());
    app.dispatch(id, Event::Composition { active: false })
        .unwrap();
    assert_eq!(
        calls.borrow().as_slice(),
        [NumberValue::Text("1.00".into())]
    );
    assert_eq!(text(&app, id), "1.00");
}

#[test]
fn field_labels_successful_names_errors_and_readonly_bind_to_the_actual_input() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let error = app.scope().cage(String::new());
    let readonly = app.scope().cage(false);
    app.mount(
        sapid::field("amount")
            .label("Amount")
            .description("Exact decimal")
            .form_name("amount")
            .required(true)
            .readonly(readonly)
            .error(error)
            .control(move |context| {
                number_input()
                    .field(context)
                    .default_value(d("1.5"))
                    .node_ref(r)
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    {
        let tree = app.tree();
        let desc = tree.node(id).unwrap().descriptor.view();
        let metadata = desc.semantics.as_ref().unwrap();
        assert!(matches!(desc.props, Props::Input(_)));
        assert_eq!(desc.effective_role(), Some(glory::render::Role::SpinButton));
        assert_eq!(metadata.form_control.as_ref().unwrap().name, "amount");
        assert!(metadata.form_control.as_ref().unwrap().value.is_none());
        assert!(
            metadata.labelled_by.is_some()
                && metadata.described_by.is_some()
                && metadata.required
        );
    }
    error.revise(|v| *v = "Pick an amount".into());
    readonly.revise(|v| *v = true);
    app.flush().unwrap();
    let tree = app.tree();
    let metadata = tree
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap();
    assert!(metadata.invalid && metadata.error_message.is_some() && metadata.readonly);
    assert!(!metadata.spinbutton.as_ref().unwrap().can_increase);
}
#[test]
fn oversized_edits_reject_without_generic_delivery_and_reassert_the_bounded_source() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let rejects = Rc::new(RefCell::new(Vec::new()));
    let out = rejects.clone();
    let generic = Rc::new(RefCell::new(0));
    let seen = generic.clone();
    app.mount(
        number_input()
            .default_value(d("2"))
            .node_ref(r)
            .on_reject(move |e| out.borrow_mut().push(e))
            .on(EventKind::ValueChanged, move |_| *seen.borrow_mut() += 1),
    )
    .unwrap();
    let id = r.get().unwrap();
    edit(&app, id, &"1".repeat(1_000_000), false);
    assert_eq!(text(&app, id), "2");
    assert_eq!(
        rejects.borrow().as_slice(),
        [sapid::NumberError::InputTooLong]
    );
    assert_eq!(*generic.borrow(), 0);
}
#[test]
fn invalid_domain_source_preserves_accepted_domain_callbacks_and_recovers() {
    let app = App::memory();
    let minimum = app.scope().cage(Some(d("0")));
    let r = app.scope().cage(None);
    let proposals = Rc::new(RefCell::new(Vec::new()));
    let out = proposals.clone();
    app.mount(
        number_input()
            .min(minimum)
            .max(Some(d("10")))
            .default_value(d("2"))
            .node_ref(r)
            .on_value_change(move |v| out.borrow_mut().push(v)),
    )
    .unwrap();
    let id = r.get().unwrap();
    minimum.revise(|v| *v = Some(d("20")));
    assert!(app.flush().is_err());
    assert!(
        app.dispatch(
            id,
            Event::Spin {
                from: id,
                command: RangeCommand::Decrease
            }
        )
        .is_err()
    );
    assert_eq!(
        proposals.borrow().as_slice(),
        [NumberValue::Decimal(d("1"))]
    );
    assert_eq!(text(&app, id), "2");
    assert_eq!(spin(&app, id).min, Some(0.));
    minimum.revise(|v| *v = Some(d("-10")));
    app.flush().unwrap();
    assert_eq!(spin(&app, id).min, Some(-10.));
    assert_eq!(text(&app, id), "1");
    assert_eq!(r.get(), Some(id));
}
#[test]
fn callback_can_retire_the_owner_without_running_later_observers() {
    let app = Rc::new(App::memory());
    let r = app.scope().cage(None);
    let seen = Rc::new(RefCell::new(0));
    let out = seen.clone();
    let weak = Rc::downgrade(&app);
    app.mount(
        number_input()
            .default_value(d("2"))
            .node_ref(r)
            .on_value_change(move |_| weak.upgrade().unwrap().clear().unwrap())
            .on(EventKind::Spin, move |_| *out.borrow_mut() += 1),
    )
    .unwrap();
    let id = r.get().unwrap();
    app.dispatch(
        id,
        Event::Spin {
            from: id,
            command: RangeCommand::Increase,
        },
    )
    .unwrap();
    assert_eq!(*seen.borrow(), 0);
    assert!(r.get().is_none());
    assert_eq!(app.stats().queries, 0);
}

#[test]
fn composition_final_value_and_duplicate_echo_in_one_tick_publish_once_at_the_same_revision() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let out = calls.clone();
    app.mount(
        number_input()
            .node_ref(r)
            .on_value_change(move |v| out.borrow_mut().push(v)),
    )
    .unwrap();
    let id = r.get().unwrap();
    let revision = {
        let tree = app.tree();
        let Props::Input(p) = tree.node(id).unwrap().descriptor.view().props else {
            panic!()
        };
        p.revision
    };
    app.scope()
        .batch(|| {
            app.dispatch(id, Event::Composition { active: true })
                .unwrap();
            app.dispatch(
                id,
                Event::ValueChanged {
                    value: "1".into(),
                    composing: true,
                    revision,
                },
            )
            .unwrap();
            app.dispatch(id, Event::Composition { active: false })
                .unwrap();
            for _ in 0..2 {
                app.dispatch(
                    id,
                    Event::ValueChanged {
                        value: "1.00".into(),
                        composing: false,
                        revision,
                    },
                )
                .unwrap();
            }
        })
        .unwrap();
    assert_eq!(
        calls.borrow().as_slice(),
        [NumberValue::Text("1.00".into())]
    );
    assert_eq!(text(&app, id), "1.00");
    edit(&app, id, "1.00", false);
    assert_eq!(
        calls.borrow().len(),
        2,
        "a new host tick/revision must not inherit old composition echo history"
    );
}
#[test]
fn commit_before_form_submit_normalizes_source_without_consuming_the_form_default_action() {
    let app = App::memory();
    let source = app.scope().cage(NumberValue::Decimal(d("20")));
    let reference = app.scope().cage(None);
    let events = Rc::new(RefCell::new(Vec::new()));
    let commits = events.clone();
    let submits = events.clone();
    app.mount(
        sapid::form("number-form")
            .on_submit(move |_| submits.borrow_mut().push(("submit", source.get())))
            .fill(
                number_input()
                    .value(source)
                    .min(Some(d("0")))
                    .max(Some(d("10")))
                    .node_ref(reference)
                    .on_commit(move |v| {
                        commits.borrow_mut().push(("commit", v.clone()));
                        source.revise(|old| *old = v)
                    }),
            ),
    )
    .unwrap();
    let id = reference.get().unwrap();
    app.scope()
        .batch(|| {
            edit(&app, id, "20", true);
            app.dispatch(
                id,
                Event::Keyboard {
                    from: id,
                    command: glory::render::KeyboardCommand::Submit,
                },
            )
            .unwrap();
        })
        .unwrap();
    assert_eq!(
        events.borrow().as_slice(),
        [
            ("commit", NumberValue::Decimal(d("10"))),
            ("submit", NumberValue::Decimal(d("10")))
        ]
    );
    assert_eq!(text(&app, id), "10");
}

#[test]
fn boundary_availability_is_not_a_stale_keyboard_gate_for_same_tick_opposite_intents() {
    for local in [true, false] {
        let app = App::memory();
        let source = app.scope().cage(NumberValue::Decimal(d("0")));
        let r = app.scope().cage(None);
        let widget = number_input()
            .default_value(d("0"))
            .min(Some(d("0")))
            .max(Some(d("10")))
            .step(d("0.1"))
            .node_ref(r);
        let widget = if local {
            widget
        } else {
            widget
                .value(source)
                .on_value_change(move |v| source.revise(|old| *old = v))
        };
        app.mount(widget).unwrap();
        let id = r.get().unwrap();
        assert!(!spin(&app, id).can_decrease);
        app.scope()
            .batch(|| {
                key(&app, id, RangeCommand::Increase);
                key(&app, id, RangeCommand::Decrease);
            })
            .unwrap();
        assert_eq!(text(&app, id), "0");
        app.scope()
            .batch(|| {
                key(&app, id, RangeCommand::Decrease);
                key(&app, id, RangeCommand::Increase);
            })
            .unwrap();
        assert_eq!(text(&app, id), "0.1");
    }
}
