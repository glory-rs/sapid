use glory::prelude::*;
use glory::render::{EventKind, InputAutofill, InputMode, NodeId, Props};
use sapid::{PinAlphabet, PinRejection, input_otp, pin_input};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn text(app: &App, id: NodeId) -> String {
    let tree = app.tree();
    let Props::Input(p) = tree.node(id).unwrap().descriptor.view().props else {
        panic!("input")
    };
    p.value.clone()
}
fn revision(app: &App, id: NodeId) -> u64 {
    let tree = app.tree();
    let Props::Input(p) = tree.node(id).unwrap().descriptor.view().props else {
        panic!("input")
    };
    p.revision
}
fn edit(app: &App, id: NodeId, value: &str, committed: bool) {
    let revision = revision(app, id);
    app.dispatch(
        id,
        if committed {
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
fn one_real_input_and_bounded_passive_cells_preserve_whole_string_and_public_hints() {
    let app = App::memory();
    let r = app.scope().cage(None);
    app.mount(input_otp().length(6).default_value("000001").node_ref(r))
        .unwrap();
    let id = r.get().unwrap();
    assert_eq!(text(&app, id).len(), 6);
    let tree = app.tree();
    let all = tree
        .roots()
        .iter()
        .flat_map(|root| {
            let mut pending = vec![*root];
            let mut found = vec![];
            while let Some(id) = pending.pop() {
                let node = tree.node(id).unwrap();
                found.push(id);
                pending.extend(node.children.iter());
            }
            found
        })
        .collect::<Vec<_>>();
    assert_eq!(
        all.iter()
            .filter(|id| matches!(
                tree.node(**id).unwrap().descriptor.view().props,
                Props::Input(_)
            ))
            .count(),
        1
    );
    assert_eq!(
        all.iter()
            .filter(|id| tree
                .node(**id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|c| c == "sapid-pin-cell"))
            .count(),
        6
    );
    let metadata = tree
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap();
    let assist = metadata.input_assist.unwrap();
    assert_eq!(assist.autofill, Some(InputAutofill::OneTimeCode));
    assert_eq!(assist.mode, Some(InputMode::Numeric));
    drop(tree);
    edit(&app, id, "000002", false);
    assert_eq!(text(&app, id), "000002");
    edit(&app, id, "00002", false);
    assert_eq!(text(&app, id), "00002");
    edit(&app, id, "", false);
    assert!(text(&app, id).is_empty());
}
#[test]
fn completion_only_follows_a_different_accepted_user_value_and_typed_callbacks_run_first() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let source = app.scope().cage(String::new());
    let order = Rc::new(RefCell::new(Vec::new()));
    let a = order.clone();
    let b = order.clone();
    let c = order.clone();
    app.mount(
        pin_input()
            .length(4)
            .value(source)
            .node_ref(r)
            .on_value_change(move |v| {
                source.revise(|s| *s = v);
                a.borrow_mut().push("change");
            })
            .on_complete(move |_| b.borrow_mut().push("complete"))
            .on(EventKind::ValueChanged, move |_| {
                c.borrow_mut().push("observe")
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    edit(&app, id, "0001", false);
    assert_eq!(order.borrow().as_slice(), ["change", "complete", "observe"]);
    edit(&app, id, "0001", false);
    assert_eq!(
        order.borrow().iter().filter(|v| **v == "complete").count(),
        1
    );
    source.revise(|s| *s = "0002".into());
    app.flush().unwrap();
    assert_eq!(
        order.borrow().iter().filter(|v| **v == "complete").count(),
        1
    );
    edit(&app, id, "0002", true);
    assert_eq!(
        order.borrow().iter().filter(|v| **v == "complete").count(),
        1
    );
    app.scope()
        .batch(|| {
            edit(&app, id, "000", false);
            edit(&app, id, "0003", false)
        })
        .unwrap();
    assert_eq!(
        order.borrow().iter().filter(|v| **v == "complete").count(),
        2
    );
}
#[test]
fn controlled_refusal_keeps_source_and_never_emits_completion() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let source = app.scope().cage("000".to_owned());
    let changes = Rc::new(Cell::new(0));
    let complete = Rc::new(Cell::new(0));
    let a = changes.clone();
    let b = complete.clone();
    app.mount(
        pin_input()
            .length(4)
            .value(source)
            .node_ref(r)
            .on_value_change(move |_| a.set(a.get() + 1))
            .on_complete(move |_| b.set(b.get() + 1)),
    )
    .unwrap();
    let id = r.get().unwrap();
    edit(&app, id, "0001", false);
    assert_eq!(changes.get(), 1);
    assert_eq!(complete.get(), 0);
    assert_eq!(source.get(), "000");
    assert_eq!(text(&app, id), "000");
}
#[test]
fn invalid_paste_and_million_byte_host_edits_reject_whole_proposals_without_faulting_or_logging_values()
 {
    let app = App::memory();
    let r = app.scope().cage(None);
    let rejects = Rc::new(RefCell::new(Vec::new()));
    let a = rejects.clone();
    let generic = Rc::new(Cell::new(0));
    let b = generic.clone();
    app.mount(
        pin_input()
            .length(4)
            .default_value("000")
            .node_ref(r)
            .on_reject(move |e| a.borrow_mut().push(e))
            .on(EventKind::ValueChanged, move |_| b.set(b.get() + 1)),
    )
    .unwrap();
    let id = r.get().unwrap();
    for value in ["00001", "00x1", "１２"] {
        edit(&app, id, value, false);
        assert_eq!(text(&app, id), "000");
    }
    let before = generic.get();
    edit(&app, id, &"7".repeat(1_000_000), false);
    assert_eq!(generic.get(), before);
    assert_eq!(text(&app, id), "000");
    assert_eq!(
        rejects.borrow().as_slice(),
        [
            PinRejection::TooLong,
            PinRejection::InvalidCharacter,
            PinRejection::TooLong,
            PinRejection::TooLong
        ]
    );
    edit(&app, id, "0001", false);
    assert_eq!(text(&app, id).len(), 4);
    app.flush().unwrap();
}
#[test]
fn bounded_initial_and_external_sources_reject_before_clone_and_recover_the_same_input() {
    for length in [0, 17, usize::MAX] {
        let app = App::memory();
        assert!(app.mount(pin_input().length(length)).is_err());
    }
    let app = App::memory();
    assert!(
        app.mount(pin_input().default_value("9".repeat(1_000_000)))
            .is_err()
    );
    let app = App::memory();
    let source = app.scope().cage("000".to_owned());
    let r = app.scope().cage(None);
    app.mount(pin_input().value(source).node_ref(r)).unwrap();
    let id = r.get().unwrap();
    source.revise(|s| *s = "9".repeat(1_000_000));
    assert!(app.flush().is_err());
    assert_eq!(text(&app, id), "000");
    source.revise(|s| *s = "0001".into());
    app.flush().unwrap();
    assert_eq!(text(&app, id).len(), 4);
    assert_eq!(r.get(), Some(id));
}
#[test]
fn composition_final_value_replaces_preedit_and_current_revision_echo_does_not_duplicate_completion()
 {
    let app = App::memory();
    let r = app.scope().cage(None);
    let changes = Rc::new(Cell::new(0));
    let complete = Rc::new(Cell::new(0));
    let a = changes.clone();
    let b = complete.clone();
    app.mount(
        pin_input()
            .length(4)
            .node_ref(r)
            .on_value_change(move |_| a.set(a.get() + 1))
            .on_complete(move |_| b.set(b.get() + 1)),
    )
    .unwrap();
    let id = r.get().unwrap();
    let rev = revision(&app, id);
    app.scope()
        .batch(|| {
            app.dispatch(id, Event::Composition { active: true })
                .unwrap();
            app.dispatch(
                id,
                Event::ValueChanged {
                    value: "preedit".into(),
                    revision: rev,
                    composing: true,
                },
            )
            .unwrap();
            app.dispatch(id, Event::Composition { active: false })
                .unwrap();
            for _ in 0..2 {
                app.dispatch(
                    id,
                    Event::ValueChanged {
                        value: "0001".into(),
                        revision: rev,
                        composing: false,
                    },
                )
                .unwrap();
            }
        })
        .unwrap();
    assert_eq!(text(&app, id).len(), 4);
    assert_eq!(changes.get(), 1);
    assert_eq!(complete.get(), 1);
    edit(&app, id, "0001", false);
    assert_eq!(changes.get(), 2);
    assert_eq!(complete.get(), 1);
}
#[test]
fn unsupported_composition_final_rejects_to_source_and_preedit_has_a_finite_budget() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let rejects = Rc::new(Cell::new(0));
    let a = rejects.clone();
    app.mount(
        pin_input()
            .default_value("00")
            .node_ref(r)
            .on_reject(move |_| a.set(a.get() + 1)),
    )
    .unwrap();
    let id = r.get().unwrap();
    let rev = revision(&app, id);
    app.scope()
        .batch(|| {
            app.dispatch(id, Event::Composition { active: true })
                .unwrap();
            app.dispatch(
                id,
                Event::ValueChanged {
                    value: "9".repeat(1_000_000),
                    revision: rev,
                    composing: true,
                },
            )
            .unwrap();
            app.dispatch(id, Event::Composition { active: false })
                .unwrap();
            app.dispatch(
                id,
                Event::ValueChanged {
                    value: "１２".into(),
                    revision: rev,
                    composing: false,
                },
            )
            .unwrap();
        })
        .unwrap();
    assert_eq!(rejects.get(), 2);
    assert_eq!(text(&app, id), "00");
    app.flush().unwrap();
}
#[test]
fn field_names_errors_required_readonly_and_disabled_belong_to_the_real_editor() {
    let app = App::memory();
    let r = app.scope().cage(None);
    let error = app.scope().cage(String::new());
    let readonly = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    app.mount(
        sapid::field("public-pin")
            .label("Code")
            .description("Public fixture")
            .error(error)
            .required(true)
            .readonly(readonly)
            .enabled(enabled)
            .form_name("otp")
            .control(move |field| pin_input().field(field).node_ref(r).masked(true)),
    )
    .unwrap();
    let id = r.get().unwrap();
    {
        let tree = app.tree();
        let node = tree.node(id).unwrap().descriptor.view();
        let Props::Input(p) = node.props else {
            panic!()
        };
        assert_eq!(p.purpose, Purpose::Password);
        let m = node.semantics.as_ref().unwrap();
        assert!(m.required && m.labelled_by.is_some() && m.described_by.is_some());
        assert_eq!(m.form_control.as_ref().unwrap().name, "otp");
    }
    error.revise(|e| *e = "Use the complete code".into());
    readonly.revise(|v| *v = true);
    app.flush().unwrap();
    edit(&app, id, "000001", false);
    assert!(text(&app, id).is_empty());
    {
        let tree = app.tree();
        let metadata = tree
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap();
        assert!(metadata.readonly && metadata.invalid && metadata.error_message.is_some());
    }
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert_eq!(r.get(), Some(id));
}
#[test]
fn alphanumeric_and_unstyled_keep_structural_mono_geometry_without_a_fake_editor() {
    let app = App::memory();
    let r = app.scope().cage(None);
    app.mount(
        pin_input()
            .length(4)
            .alphabet(PinAlphabet::AsciiAlphanumeric)
            .unstyled()
            .node_ref(r),
    )
    .unwrap();
    let id = r.get().unwrap();
    edit(&app, id, "aZ09", false);
    assert_eq!(text(&app, id), "aZ09");
    let tree = app.tree();
    let node = tree.node(id).unwrap().descriptor.view();
    assert_eq!(node.style.width, Some(Length::Ch(4.)));
    assert_eq!(
        node.style.font_family.as_deref(),
        Some([Family::Mono].as_slice())
    );
    assert_eq!(
        node.semantics.as_ref().unwrap().input_assist.unwrap().mode,
        Some(InputMode::Text)
    );
}
#[test]
fn callback_owner_retirement_prevents_completion_observers_and_releases_references() {
    let app = Rc::new(App::memory());
    let r = app.scope().cage(None);
    let complete = Rc::new(Cell::new(0));
    let observed = Rc::new(Cell::new(0));
    let weak = Rc::downgrade(&app);
    let a = complete.clone();
    let b = observed.clone();
    app.mount(
        pin_input()
            .length(4)
            .node_ref(r)
            .on_value_change(move |_| weak.upgrade().unwrap().clear().unwrap())
            .on_complete(move |_| a.set(a.get() + 1))
            .on(EventKind::ValueChanged, move |_| b.set(b.get() + 1)),
    )
    .unwrap();
    let id = r.get().unwrap();
    edit(&app, id, "0001", false);
    assert_eq!(complete.get(), 0);
    assert_eq!(observed.get(), 0);
    assert!(r.get().is_none());
    assert_eq!(app.stats().queries, 0);
}
