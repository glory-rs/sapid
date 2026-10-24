use glory::prelude::*;
use glory::render::{memory::Memory, *};
use sapid::{MaskPattern, MaskRejection, mask_input};
use std::{cell::RefCell, rc::Rc};

struct Capture {
    tree: Memory,
    actions: Rc<RefCell<Vec<Action>>>,
}
impl Render for Capture {
    fn style_profile(&self) -> glory::style::Profile {
        glory::style::Profile::RECORD
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            input: true,
            ..Default::default()
        }
    }
    fn request(
        &mut self,
        q: Query,
        t: QueryToken,
    ) -> std::result::Result<(), glory::render::Error> {
        self.tree.request(q, t)
    }
    fn drain(&mut self, v: &mut Vec<Input>) {
        self.tree.drain(v)
    }
    fn apply(&mut self, m: &[Mutation]) -> std::result::Result<Receipt, glory::render::Error> {
        self.tree.apply(m)
    }
    fn action(&mut self, a: Action) -> std::result::Result<(), glory::render::Error> {
        a.validate_target(&self.tree.node(a.node())?.descriptor)?;
        self.actions.borrow_mut().push(a);
        Ok(())
    }
}
fn app() -> (App, Rc<RefCell<Vec<Action>>>) {
    let actions = Rc::new(RefCell::new(Vec::new()));
    let out = actions.clone();
    (
        App::new(move |runtime| Capture {
            tree: Memory::new(runtime),
            actions: out,
        }),
        actions,
    )
}
fn props(app: &App, id: NodeId) -> InputProps {
    let tree = app.tree();
    let Props::Input(p) = tree.node(id).unwrap().descriptor.view().props else {
        panic!("input")
    };
    p.clone()
}
fn edit(
    app: &App,
    id: NodeId,
    text: &str,
    after: (usize, usize),
    before: Option<(usize, usize)>,
    intent: InputEditIntent,
) {
    app.dispatch(
        id,
        Event::InputEdit(InputEdit::Value {
            value: text.into(),
            revision: props(app, id).revision,
            selection: TextSelection {
                start: after.0,
                end: after.1,
                direction: SelectionDirection::Backward,
            },
            before: before.map(|(start, end)| TextSelection {
                start,
                end,
                ..Default::default()
            }),
            intent,
            composing: false,
        }),
    )
    .unwrap();
}
#[test]
fn raw_business_value_is_distinct_from_display_and_only_one_real_input_exists() {
    let (app, actions) = app();
    let r = app.scope().cage(None);
    let source = app.scope().cage("1234".to_owned());
    let order = Rc::new(RefCell::new(Vec::new()));
    let a = order.clone();
    let b = order.clone();
    app.mount(
        mask_input(MaskPattern::parse("##-##").unwrap())
            .value(source)
            .node_ref(r)
            .on_value_change(move |raw| {
                source.revise(|v| *v = raw);
                a.borrow_mut().push("typed");
            })
            .on(EventKind::InputEdit, move |_| {
                b.borrow_mut().push("generic")
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    assert_eq!(props(&app, id).value, "12-34");
    assert_eq!(app.tree().roots(), [id]);
    edit(
        &app,
        id,
        "1954",
        (1, 3),
        Some((1, 4)),
        InputEditIntent::Insert,
    );
    assert_eq!(source.get(), "1954");
    assert_eq!(props(&app, id).value, "19-54");
    assert_eq!(order.borrow().as_slice(), ["typed", "generic"]);
    let actions = actions.borrow();
    let Action::SelectText { selection, .. } = actions.last().unwrap() else {
        panic!("selection")
    };
    assert_eq!(
        (selection.start, selection.end, selection.direction),
        (1, 4, SelectionDirection::Backward)
    );
}
#[test]
fn same_tick_literal_backspaces_use_current_physical_map_and_commit_raw_once() {
    let (app, _) = app();
    let r = app.scope().cage(None);
    let values = Rc::new(RefCell::new(Vec::new()));
    let a = values.clone();
    let b = values.clone();
    app.mount(
        mask_input(MaskPattern::parse("##-##").unwrap())
            .default_value("12")
            .node_ref(r)
            .on_value_change(move |v| a.borrow_mut().push(v))
            .on_commit(move |v| b.borrow_mut().push(format!("commit:{v}"))),
    )
    .unwrap();
    let id = r.get().unwrap();
    app.scope()
        .batch(|| {
            edit(
                &app,
                id,
                "12",
                (2, 2),
                Some((3, 3)),
                InputEditIntent::DeleteBackward,
            );
            edit(
                &app,
                id,
                "1",
                (1, 1),
                Some((2, 2)),
                InputEditIntent::DeleteBackward,
            );
            app.dispatch(
                id,
                Event::Commit {
                    value: "1".into(),
                    revision: props(&app, id).revision,
                },
            )
            .unwrap();
        })
        .unwrap();
    assert_eq!(props(&app, id).value, "");
    assert_eq!(values.borrow().as_slice(), ["1", "", "commit:"]);
}
#[test]
fn ordinary_edit_preserves_native_selection_and_only_literal_insertion_requests_a_selection() {
    let (app, actions) = app();
    let r = app.scope().cage(None);
    app.mount(mask_input(MaskPattern::parse("##-##").unwrap()).node_ref(r))
        .unwrap();
    let id = r.get().unwrap();
    edit(&app, id, "1", (1, 1), Some((0, 0)), InputEditIntent::Insert);
    assert!(actions.borrow().is_empty());
    edit(
        &app,
        id,
        "12",
        (2, 2),
        Some((1, 1)),
        InputEditIntent::Insert,
    );
    assert_eq!(props(&app, id).value, "12-");
    assert_eq!(actions.borrow().len(), 1);
}
#[test]
fn controlled_refusal_restores_accepted_display_and_invalid_paste_is_a_typed_rejection() {
    let (app, _) = app();
    let r = app.scope().cage(None);
    let source = app.scope().cage("12".to_owned());
    let proposed = Rc::new(RefCell::new(Vec::new()));
    let a = proposed.clone();
    let rejected = Rc::new(RefCell::new(Vec::new()));
    let b = rejected.clone();
    app.mount(
        mask_input(MaskPattern::parse("##-##").unwrap())
            .value(source)
            .node_ref(r)
            .on_value_change(move |v| a.borrow_mut().push(v))
            .on_reject(move |v| b.borrow_mut().push(v)),
    )
    .unwrap();
    let id = r.get().unwrap();
    edit(
        &app,
        id,
        "12-3",
        (4, 4),
        Some((3, 3)),
        InputEditIntent::Insert,
    );
    assert_eq!(proposed.borrow().as_slice(), ["123"]);
    assert_eq!(source.get(), "12");
    assert_eq!(props(&app, id).value, "12-");
    edit(&app, id, "12-x", (4, 4), None, InputEditIntent::Replace);
    assert_eq!(
        rejected.borrow().as_slice(),
        [MaskRejection::InvalidCharacter]
    );
    assert_eq!(props(&app, id).value, "12-");
    app.dispatch(
        id,
        Event::InputEdit(InputEdit::Rejected {
            revision: props(&app, id).revision,
            reason: InputEditRejection::TooLong,
        }),
    )
    .unwrap();
    assert_eq!(rejected.borrow().last(), Some(&MaskRejection::InputTooLong));
    assert!(app.flush().is_ok());
}
#[test]
fn composition_is_owned_by_the_host_until_one_terminal_mapping_and_current_echo_is_suppressed()
{
    let (app, actions) = app();
    let r = app.scope().cage(None);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let out = calls.clone();
    app.mount(
        mask_input(MaskPattern::parse("##-##").unwrap())
            .node_ref(r)
            .on_value_change(move |v| out.borrow_mut().push(v)),
    )
    .unwrap();
    let id = r.get().unwrap();
    app.dispatch(id, Event::Composition { active: true })
        .unwrap();
    app.dispatch(
        id,
        Event::InputEdit(InputEdit::Value {
            value: "中".into(),
            revision: props(&app, id).revision,
            selection: TextSelection {
                start: 1,
                end: 1,
                ..Default::default()
            },
            before: None,
            intent: InputEditIntent::Replace,
            composing: true,
        }),
    )
    .unwrap();
    assert!(calls.borrow().is_empty());
    assert!(actions.borrow().is_empty());
    app.scope()
        .batch(|| {
            app.dispatch(id, Event::Composition { active: false })
                .unwrap();
            edit(&app, id, "12", (2, 2), None, InputEditIntent::Replace);
            edit(&app, id, "12", (2, 2), None, InputEditIntent::Replace);
        })
        .unwrap();
    assert_eq!(calls.borrow().as_slice(), ["12"]);
    assert_eq!(props(&app, id).value, "12-");
}
#[test]
fn external_invalid_source_is_rejected_before_clone_and_recovers_without_business_callbacks() {
    let (app, _) = app();
    let r = app.scope().cage(None);
    let source = app.scope().cage("12".to_owned());
    let calls = Rc::new(RefCell::new(0));
    let out = calls.clone();
    app.mount(
        mask_input(MaskPattern::parse("##-##").unwrap())
            .value(source)
            .node_ref(r)
            .on_value_change(move |_| *out.borrow_mut() += 1),
    )
    .unwrap();
    let id = r.get().unwrap();
    source.revise(|v| *v = "8".repeat(1_000_000));
    assert!(app.flush().is_err());
    assert_eq!(props(&app, id).value, "12-");
    source.revise(|v| *v = "0034".into());
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "00-34");
    assert_eq!(*calls.borrow(), 0);
    app.clear().unwrap();
    assert_eq!(r.get(), None);
    assert_eq!(app.stats().queries, 0);
}

#[test]
fn newer_program_value_during_composition_rejects_the_old_terminal_and_restores_on_cleanup() {
    let (app, _) = app();
    let r = app.scope().cage(None);
    let value = app.scope().cage("12".to_owned());
    let calls = Rc::new(RefCell::new(0));
    let out = calls.clone();
    app.mount(
        mask_input(MaskPattern::parse("##-##").unwrap())
            .value(value)
            .node_ref(r)
            .on_value_change(move |_| *out.borrow_mut() += 1),
    )
    .unwrap();
    let id = r.get().unwrap();
    let old = props(&app, id).revision;
    app.dispatch(id, Event::Composition { active: true })
        .unwrap();
    value.revise(|v| *v = "34".into());
    app.flush().unwrap();
    assert!(props(&app, id).revision > old);
    app.scope()
        .batch(|| {
            app.dispatch(id, Event::Composition { active: false })
                .unwrap();
            app.dispatch(
                id,
                Event::InputEdit(InputEdit::Value {
                    value: "99".into(),
                    revision: old,
                    selection: TextSelection {
                        start: 2,
                        end: 2,
                        ..Default::default()
                    },
                    before: None,
                    intent: InputEditIntent::Replace,
                    composing: false,
                }),
            )
            .unwrap();
        })
        .unwrap();
    assert_eq!(value.get(), "34");
    assert_eq!(props(&app, id).value, "34-");
    assert_eq!(*calls.borrow(), 0);
    edit(
        &app,
        id,
        "34-5",
        (4, 4),
        Some((3, 3)),
        InputEditIntent::Insert,
    );
    assert_eq!(*calls.borrow(), 1);
}

#[test]
fn field_metadata_readonly_and_disabled_apply_to_the_real_editor() {
    let (app, _) = app();
    let r = app.scope().cage(None);
    let error = app.scope().cage(String::new());
    let readonly = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    app.mount(
        sapid::field("mask")
            .label("Reference")
            .description("ASCII reference")
            .error(error)
            .required(true)
            .readonly(readonly)
            .enabled(enabled)
            .form_name("reference")
            .control(move |f| {
                mask_input(MaskPattern::parse("##-##").unwrap())
                    .field(f)
                    .default_value("12")
                    .node_ref(r)
            }),
    )
    .unwrap();
    let id = r.get().unwrap();
    {
        let tree = app.tree();
        let m = tree
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap();
        assert!(m.required && m.labelled_by.is_some() && m.described_by.is_some());
        assert_eq!(m.form_control.as_ref().unwrap().name, "reference");
        assert!(m.input_edit.is_some());
    }
    readonly.revise(|v| *v = true);
    error.revise(|v| *v = "Complete reference".into());
    app.flush().unwrap();
    edit(&app, id, "34-", (3, 3), None, InputEditIntent::Replace);
    assert_eq!(props(&app, id).value, "12-");
    {
        let tree = app.tree();
        let m = tree
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap();
        assert!(m.readonly && m.invalid && m.error_message.is_some());
    }
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert_eq!(r.get(), Some(id));
}

#[test]
fn typed_callback_can_retire_its_owner_without_late_selection_or_generic_observation() {
    let (app, actions) = app();
    let app = Rc::new(app);
    let r = app.scope().cage(None);
    let count = Rc::new(RefCell::new(0));
    let out = count.clone();
    let weak = Rc::downgrade(&app);
    app.mount(
        mask_input(MaskPattern::parse("##-##").unwrap())
            .node_ref(r)
            .on_value_change(move |_| weak.upgrade().unwrap().clear().unwrap())
            .on(EventKind::InputEdit, move |_| *out.borrow_mut() += 1),
    )
    .unwrap();
    let id = r.get().unwrap();
    edit(&app, id, "12", (2, 2), None, InputEditIntent::Replace);
    assert!(r.get().is_none());
    assert!(actions.borrow().is_empty());
    assert_eq!(*count.borrow(), 0);
    assert_eq!(app.stats().queries, 0);
}
