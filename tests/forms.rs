use glory::prelude::*;
use glory::render::{EventKind, FieldPart, FormAction, KeyboardCommand, NodeId};
use sapid::{field, form, reset_button, submit_button, text_field, textarea};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut nodes = tree.roots().to_vec();
    while let Some(id) = nodes.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            return id;
        }
        nodes.extend(node.children.iter().copied());
    }
    panic!("missing {class}")
}
#[test]
fn form_requests_run_typed_callbacks_before_generic_observers() {
    let app = App::memory();
    let trace = Rc::new(RefCell::new(Vec::new()));
    let a = trace.clone();
    let b = trace.clone();
    let c = trace.clone();
    let d = trace.clone();
    let submitter = Rc::new(Cell::new(None));
    let captured = submitter.clone();
    app.mount(
        form("profile")
            .on_submit(move |id| {
                captured.set(id);
                a.borrow_mut().push("typed submit");
            })
            .on(EventKind::Submit, move |_| {
                b.borrow_mut().push("observed submit")
            })
            .on_reset(move || c.borrow_mut().push("typed reset"))
            .on(EventKind::Reset, move |_| {
                d.borrow_mut().push("observed reset")
            })
            .fill((submit_button().class("save"), reset_button().class("reset"))),
    )
    .unwrap();
    let save = find(&app, "save");
    let reset = find(&app, "reset");
    app.dispatch(save, Event::Activate).unwrap();
    app.dispatch(reset, Event::Activate).unwrap();
    assert_eq!(submitter.get(), Some(save));
    assert_eq!(
        *trace.borrow(),
        vec![
            "typed submit",
            "observed submit",
            "typed reset",
            "observed reset"
        ]
    );
}
#[test]
fn single_line_enter_submits_but_multiline_enter_does_not() {
    let app = App::memory();
    let requests = Rc::new(Cell::new(0));
    let count = requests.clone();
    app.mount(
        form("enter")
            .on_submit(move |id| {
                assert!(id.is_none());
                count.set(count.get() + 1);
            })
            .fill((text_field().class("single"), textarea().class("multi"))),
    )
    .unwrap();
    let single = find(&app, "single");
    let multi = find(&app, "multi");
    app.dispatch(
        single,
        Event::Keyboard {
            from: single,
            command: KeyboardCommand::Submit,
        },
    )
    .unwrap();
    app.dispatch(
        multi,
        Event::Keyboard {
            from: multi,
            command: KeyboardCommand::Submit,
        },
    )
    .unwrap();
    assert_eq!(requests.get(), 1);
}
#[test]
fn disabled_form_has_stable_native_fieldset_and_blocks_requests() {
    let app = App::memory();
    let disabled = app.scope().cage(false);
    let requests = Rc::new(Cell::new(0));
    let count = requests.clone();
    app.mount(
        form("disabled")
            .enabled(app.scope().bond(move || !disabled.get()))
            .on_submit(move |_| count.set(count.get() + 1))
            .fill((text_field().class("input"), submit_button().class("save"))),
    )
    .unwrap();
    let form_id = find(&app, "sapid-form");
    let controls = find(&app, "sapid-form-controls");
    let save = find(&app, "save");
    disabled.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(
        app.tree()
            .node(controls)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .disabled
    );
    assert_eq!(
        app.tree()
            .node(controls)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .field_part,
        Some(FieldPart::Fieldset)
    );
    assert!(!app.dispatch(save, Event::Activate).unwrap());
    assert!(
        !app.dispatch(form_id, Event::Submit { submitter: None })
            .unwrap()
    );
    assert_eq!(requests.get(), 0);
    disabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert_eq!(find(&app, "sapid-form-controls"), controls);
    app.dispatch(save, Event::Activate).unwrap();
    assert_eq!(requests.get(), 1);
}
#[test]
fn name_metadata_and_button_entry_are_separate_from_accessible_names() {
    let app = App::memory();
    app.mount(
        form("names").name("Profile").fill((
            field("email")
                .label("Public email")
                .form_name("account_email")
                .control(|ctx| text_field().field(ctx).class("input").value("a@b")),
            submit_button()
                .class("save")
                .label("Save")
                .name("Save profile")
                .entry("intent", "save"),
        )),
    )
    .unwrap();
    let input = app
        .tree()
        .node(find(&app, "input"))
        .unwrap()
        .descriptor
        .view()
        .to_owned();
    let save = app
        .tree()
        .node(find(&app, "save"))
        .unwrap()
        .descriptor
        .view()
        .to_owned();
    assert_eq!(
        input
            .semantics
            .as_ref()
            .unwrap()
            .form_control
            .as_ref()
            .unwrap()
            .name,
        "account_email"
    );
    assert_eq!(
        input
            .semantics
            .as_ref()
            .unwrap()
            .form_control
            .as_ref()
            .unwrap()
            .value,
        None
    );
    assert_eq!(save.name.as_deref(), Some("Save profile"));
    assert_eq!(
        save.semantics.as_ref().unwrap().form_action,
        Some(FormAction::Submit)
    );
    assert_eq!(
        save.semantics
            .as_ref()
            .unwrap()
            .form_control
            .as_ref()
            .unwrap()
            .value
            .as_deref(),
        Some("save")
    );
}
#[test]
fn changing_form_name_preserves_input_revision_and_identity() {
    let app = App::memory();
    let name = app.scope().cage("before".to_owned());
    app.mount(
        form("dynamic").fill(
            field("email")
                .label("Email")
                .form_name(name)
                .control(|ctx| text_field().field(ctx).class("input").value("draft")),
        ),
    )
    .unwrap();
    let id = find(&app, "input");
    let props = app.tree().node(id).unwrap().descriptor.view().props.clone();
    name.revise(|v| *v = "after".into());
    app.flush().unwrap();
    assert_eq!(find(&app, "input"), id);
    assert_eq!(
        app.tree().node(id).unwrap().descriptor.view().props.clone(),
        props
    );
    assert_eq!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .form_control
            .as_ref()
            .unwrap()
            .name,
        "after"
    );
}
#[test]
fn form_reset_does_not_mutate_controlled_values_without_application_policy() {
    let app = App::memory();
    let value = app.scope().cage("owned draft".to_owned());
    app.mount(form("explicit-reset").fill((
        text_field().value(value).class("input"),
        reset_button().class("reset"),
    )))
    .unwrap();
    app.dispatch(find(&app, "reset"), Event::Activate).unwrap();
    assert_eq!(value.get(), "owned draft");
}
#[test]
fn owner_retirement_drops_request_captures_and_clears_node_references() {
    struct DropCount(Rc<Cell<usize>>);
    impl Drop for DropCount {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let app = App::memory();
    let dropped = Rc::new(Cell::new(0));
    let capture = DropCount(dropped.clone());
    let reference = app.scope().cage(None);
    app.mount(
        form("retiring")
            .on_submit(move |_| {
                let _ = &capture;
            })
            .fill(submit_button().node_ref(reference)),
    )
    .unwrap();
    assert!(reference.get().is_some());
    app.clear().unwrap();
    assert_eq!(dropped.get(), 1);
    assert!(reference.get().is_none());
}
#[test]
fn ssr_uses_real_form_and_escapes_successful_control_metadata() {
    let html = glory::ssr::render(
        form("escaped").name("Profile").fill((
            field("name")
                .label("Name")
                .form_name("name\"<&")
                .control(|ctx| text_field().field(ctx).value("draft<&")),
            submit_button().entry("intent", "save\"<&"),
            reset_button(),
        )),
    )
    .unwrap();
    assert!(html.contains("<form"));
    assert!(html.contains("novalidate"));
    assert!(html.contains("data-gly-form"));
    assert!(html.contains("name=\"name&quot;&lt;&amp;\""));
    assert!(html.contains("value=\"save&quot;&lt;&amp;\""));
    assert!(!html.contains("<script"));
}
#[test]
fn invalid_field_successful_values_fail_explicitly() {
    assert!(
        glory::ssr::render(
            field("invalid")
                .form_value("value")
                .control(|ctx| text_field().field(ctx))
        )
        .is_err()
    );
    assert!(
        glory::ssr::render(
            field("invalid-input-value")
                .form_name("name")
                .form_value("value")
                .control(|ctx| text_field().field(ctx))
        )
        .is_err()
    );
}

#[test]
fn invalid_form_names_are_borrow_checked_before_the_control_factory_runs() {
    for name in [
        "x".repeat(1_000_000),
        "bad\nname".into(),
        "bad\0name".into(),
        "bad\rname".into(),
    ] {
        let app = App::memory();
        let called = std::rc::Rc::new(std::cell::Cell::new(false));
        let out = called.clone();
        assert!(
            app.mount(field("bounded-name").form_name(name).control(move |ctx| {
                out.set(true);
                text_field().field(ctx)
            }))
            .is_err()
        );
        assert!(!called.get());
        assert!(app.tree().roots().is_empty());
    }
}

#[test]
fn successful_name_and_value_validate_as_one_pair_without_an_invented_value_limit() {
    let app = App::memory();
    let name = app.scope().cage("accepted".to_owned());
    let value = app.scope().cage("v".repeat(5000));
    let reference = app.scope().cage(None);
    app.mount(
        field("pair")
            .form_name(name)
            .form_value(value)
            .control(move |ctx| sapid::checkbox().field(ctx).node_ref(reference)),
    )
    .unwrap();
    let node = reference.get().unwrap();
    let metadata = || {
        app.tree()
            .node(node)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .form_control
            .clone()
            .unwrap()
    };
    assert_eq!(metadata().value.as_ref().unwrap().len(), 5000);
    app.scope()
        .batch(|| {
            name.revise(|name| *name = "next".into());
            value.revise(|value| *value = "bad\rvalue".into());
        })
        .unwrap();
    assert!(app.flush().is_err());
    assert_eq!(metadata().name, "accepted");
    assert_eq!(metadata().value.as_ref().unwrap().len(), 5000);
    value.revise(|value| *value = "valid\nvalue".into());
    app.flush().unwrap();
    assert_eq!(metadata().name, "next");
    assert_eq!(metadata().value.as_deref(), Some("valid\nvalue"));
}
