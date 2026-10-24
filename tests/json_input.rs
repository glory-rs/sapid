#![cfg(feature = "json")]
use glory::prelude::*;
use glory::render::{Event, EventKind, InputProps, NodeId, Props, Semantics};
use sapid::{JsonErrorKind, JsonFormat, JsonKind, JsonLimits, JsonValidity, field, json_input};
use std::{cell::RefCell, rc::Rc};

fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
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
        pending.extend(&node.children);
    }
    panic!("missing {class}")
}
fn props(app: &App, id: NodeId) -> InputProps {
    let Props::Input(value) = app.tree().node(id).unwrap().descriptor.view().props.clone()
    else {
        panic!("input")
    };
    value
}
fn semantics(app: &App, id: NodeId) -> Semantics {
    app.tree()
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_deref()
        .cloned()
        .unwrap_or_default()
}
fn edit(app: &App, id: NodeId, value: &str) {
    app.dispatch(
        id,
        Event::ValueChanged {
            value: value.into(),
            revision: props(app, id).revision,
            composing: false,
        },
    )
    .unwrap();
    app.flush().unwrap();
}

#[test]
fn malformed_draft_remains_editable_and_validation_does_not_rewrite_its_value() {
    let app = App::memory();
    let states = Rc::new(RefCell::new(Vec::new()));
    let observed = states.clone();
    app.mount(
        json_input("draft")
            .default_value("{")
            .on_validation(move |value| observed.borrow_mut().push(value)),
    )
    .unwrap();
    let editor = find(&app, "sapid-textarea");
    let error = find(&app, "sapid-json-validation");
    assert_eq!(props(&app, editor).value, "{");
    assert!(props(&app, editor).multiline);
    assert!(semantics(&app, editor).invalid);
    assert_eq!(
        semantics(&app, editor).error_message,
        semantics(&app, error).id
    );
    edit(&app, editor, "[1,true]");
    assert!(!semantics(&app, editor).invalid);
    assert!(semantics(&app, error).hidden);
    assert_eq!(
        states.borrow().last(),
        Some(&JsonValidity::Valid(JsonKind::Array))
    );
    assert_eq!(find(&app, "sapid-textarea"), editor);
    assert_eq!(props(&app, editor).value, "[1,true]");
    edit(&app, editor, "");
    assert_eq!(states.borrow().last(), Some(&JsonValidity::Empty));
}

#[test]
fn formatter_proposes_before_observation_and_respects_controlled_refusal() {
    for accepts in [false, true] {
        let app = App::memory();
        let source = app.scope().cage("{\"n\":1e999,\"n\":2}".to_string());
        let log = Rc::new(RefCell::new(Vec::new()));
        let change = log.clone();
        let formatted = log.clone();
        app.mount(
            json_input("format")
                .value(source)
                .on_value_change(move |value| {
                    change.borrow_mut().push(format!("change:{value}"));
                    if accepts {
                        source.revise(|old| *old = value)
                    }
                })
                .on_format(move |value| formatted.borrow_mut().push(format!("format:{value}"))),
        )
        .unwrap();
        let editor = find(&app, "sapid-textarea");
        let original = props(&app, editor);
        app.dispatch(find(&app, "sapid-button"), Event::Activate)
            .unwrap();
        app.flush().unwrap();
        let expected = "{\n  \"n\": 1e999,\n  \"n\": 2\n}";
        assert_eq!(
            *log.borrow(),
            [format!("change:{expected}"), format!("format:{expected}")]
        );
        assert_eq!(
            props(&app, editor).value,
            if accepts { expected } else { &original.value }
        );
        assert_eq!(source.get(), props(&app, editor).value);
        assert_eq!(find(&app, "sapid-textarea"), editor);
    }
}

#[test]
fn output_budget_keeps_valid_draft_and_next_accepted_edit_clears_format_failure() {
    let app = App::memory();
    let errors = Rc::new(RefCell::new(Vec::new()));
    let observed = errors.clone();
    app.mount(
        json_input("budget")
            .default_value("[1,2,3]")
            .format(JsonFormat::PrettyFour)
            .limits(JsonLimits { bytes: 8, depth: 8 })
            .on_format_error(move |error| observed.borrow_mut().push(error.kind)),
    )
    .unwrap();
    let editor = find(&app, "sapid-textarea");
    app.dispatch(find(&app, "sapid-button"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert_eq!(*errors.borrow(), [JsonErrorKind::OutputBudget]);
    assert!(!semantics(&app, editor).invalid);
    assert_eq!(props(&app, editor).value, "[1,2,3]");
    let text = find(&app, "sapid-json-format-error");
    assert!(
        matches!(&app.tree().node(text).unwrap().descriptor.view().props, Props::Text(value) if !value.content.is_empty())
    );
    edit(&app, editor, "[]");
    assert!(
        matches!(&app.tree().node(text).unwrap().descriptor.view().props, Props::Text(value) if value.content.is_empty())
    );
}

#[test]
fn field_error_priority_readonly_and_required_metadata_remain_independent_of_json() {
    let app = App::memory();
    let external = app.scope().cage("Server validation".to_string());
    let readonly = app.scope().cage(true);
    app.mount(
        field("payload")
            .label("Payload")
            .description("JSON document")
            .error(external)
            .readonly(readonly)
            .required(true)
            .control(move |field| json_input("field").field(field).default_value("{")),
    )
    .unwrap();
    let editor = find(&app, "sapid-textarea");
    let original = props(&app, editor);
    let own_error = semantics(&app, find(&app, "sapid-json-validation")).id;
    assert!(semantics(&app, editor).required);
    assert!(semantics(&app, editor).readonly);
    assert_ne!(semantics(&app, editor).error_message, own_error);
    let button = find(&app, "sapid-button");
    assert!(!app.tree().enabled(button));
    external.revise(String::clear);
    readonly.revise(|value| *value = false);
    app.flush().unwrap();
    assert_eq!(semantics(&app, editor).error_message, own_error);
    assert_eq!(props(&app, editor).value, original.value);
    assert_eq!(props(&app, editor).revision, original.revision);
    assert!(!semantics(&app, editor).readonly);
}

#[test]
fn composition_blocks_formatting_and_typed_edits_precede_generic_observers() {
    let app = App::memory();
    let log = Rc::new(RefCell::new(Vec::new()));
    let typed = log.clone();
    let generic = log.clone();
    app.mount(
        json_input("composition")
            .default_value("[1]")
            .on_value_change(move |_| typed.borrow_mut().push("typed"))
            .on(EventKind::ValueChanged, move |_| {
                generic.borrow_mut().push("generic")
            }),
    )
    .unwrap();
    let editor = find(&app, "sapid-textarea");
    app.dispatch(editor, Event::Composition { active: true })
        .unwrap();
    app.flush().unwrap();
    assert!(!app.tree().enabled(find(&app, "sapid-button")));
    edit(&app, editor, "[2]");
    assert_eq!(*log.borrow(), ["typed", "generic"]);
    app.dispatch(editor, Event::Composition { active: false })
        .unwrap();
    app.flush().unwrap();
    assert!(app.tree().enabled(find(&app, "sapid-button")));
}

#[test]
fn invalid_initial_configuration_is_rejected_and_oversized_source_can_recover() {
    let app = App::memory();
    assert!(
        app.mount(json_input("bad").limits(JsonLimits { bytes: 0, depth: 1 }))
            .is_err()
    );
    let app = App::memory();
    let source = app.scope().cage("null".to_string());
    app.mount(json_input("recover").value(source).limits(JsonLimits {
        bytes: 16,
        depth: 4,
    }))
    .unwrap();
    let editor = find(&app, "sapid-textarea");
    source.revise(|value| *value = " ".repeat(1_000_000));
    assert!(app.flush().is_err());
    assert_eq!(props(&app, editor).value, "null");
    source.revise(|value| *value = "true".into());
    app.flush().unwrap();
    assert_eq!(props(&app, editor).value, "true");
}

#[test]
fn ssr_escapes_draft_text_and_owner_retirement_releases_handlers_and_references() {
    let html = glory::ssr::render(
        json_input("ssr").default_value(r#"{"html":"</textarea><script>"}"#),
    )
    .unwrap();
    assert!(html.contains("<textarea"));
    assert!(html.contains("&lt;/textarea&gt;&lt;script&gt;"));
    assert!(!html.contains("<script>"));
    let app = App::memory();
    let reference = app.scope().cage(None);
    let capture = Rc::new(());
    let weak = Rc::downgrade(&capture);
    let mounted = app
        .scope()
        .mount_with(move || {
            json_input("owned")
                .node_ref(reference)
                .on_validation(move |_| {
                    let _ = &capture;
                })
        })
        .unwrap();
    assert!(reference.get().is_some());
    drop(mounted);
    app.flush().unwrap();
    assert_eq!(reference.get(), None);
    assert!(weak.upgrade().is_none());
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(app.stats().handlers, 0);
}
