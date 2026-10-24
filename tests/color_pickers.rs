use glory::prelude::*;
use glory::render::{
    Event, EventKind, InputProps, NodeId, Props, RangeCommand, Role, Semantics,
};
use sapid::{Hsva, color_input, color_picker};
use std::{cell::RefCell, rc::Rc};
fn nodes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut out = Vec::new();
    while let Some(id) = todo.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            out.push(id);
        }
        todo.extend(&node.children);
    }
    out
}
fn slider(app: &App, name: &str) -> NodeId {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    while let Some(id) = todo.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().role == Some(Role::Slider)
            && node.descriptor.view().name.as_deref() == Some(name)
        {
            return id;
        }
        todo.extend(&node.children);
    }
    panic!("missing {name}")
}
fn metadata(app: &App, id: NodeId) -> Semantics {
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
fn number(app: &App, id: NodeId) -> f64 {
    metadata(app, id).slider.unwrap().range.value
}
fn key(app: &App, id: NodeId, command: RangeCommand) {
    app.dispatch(id, Event::Range { from: id, command })
        .unwrap();
}
fn input(app: &App, id: NodeId) -> InputProps {
    let Props::Input(value) = app.tree().node(id).unwrap().descriptor.view().props.clone()
    else {
        panic!("input")
    };
    value
}
fn edit(app: &App, id: NodeId, value: &str) {
    app.dispatch(
        id,
        Event::ValueChanged {
            value: value.into(),
            revision: input(app, id).revision,
            composing: false,
        },
    )
    .unwrap();
    app.flush().unwrap();
}
#[test]
fn local_picker_has_four_true_keyboard_sliders_and_bounded_passive_plane() {
    let app = App::memory();
    let root = app.scope().cage(None);
    app.mount(
        color_picker("local")
            .name("Precise color")
            .node_ref(root)
            .default_value(Hsva {
                hue: 120.123456789,
                saturation: 0.5,
                value: 0.75,
                alpha: 0.4,
            }),
    )
    .unwrap();
    assert_eq!(
        app.tree()
            .node(root.get().unwrap())
            .unwrap()
            .descriptor
            .view()
            .role,
        Some(Role::Group)
    );
    assert_eq!(nodes(&app, "sapid-color-plane-cell").len(), 256);
    assert_eq!(
        metadata(&app, nodes(&app, "sapid-color-plane")[0]).drag_capture,
        Some(glory::render::DragCapture::Press)
    );
    let saturation = slider(&app, "Saturation");
    app.scope()
        .batch(|| {
            key(&app, saturation, RangeCommand::Increase);
            key(&app, saturation, RangeCommand::Increase);
        })
        .unwrap();
    assert!((number(&app, saturation) - 0.52).abs() < 1e-12);
    assert_eq!(number(&app, slider(&app, "Hue")), 120.123456789);
    key(&app, slider(&app, "Brightness"), RangeCommand::Maximum);
    assert_eq!(number(&app, slider(&app, "Brightness")), 1.);
}
#[test]
fn controlled_refusal_emits_one_change_then_commit_and_never_mutates_source() {
    let app = App::memory();
    let source = app.scope().cage(Hsva::default());
    let log = Rc::new(RefCell::new(Vec::new()));
    let a = log.clone();
    let b = log.clone();
    app.mount(
        color_picker("controlled")
            .value(source)
            .on_value_change(move |v| a.borrow_mut().push(("change", v)))
            .on_value_commit(move |v| b.borrow_mut().push(("commit", v))),
    )
    .unwrap();
    let sat = slider(&app, "Saturation");
    key(&app, sat, RangeCommand::Decrease);
    assert_eq!(number(&app, sat), 1.);
    assert_eq!(source.get(), Hsva::default());
    let log = log.borrow();
    assert_eq!(log.len(), 2);
    assert_eq!(log[0].0, "change");
    assert_eq!(log[1].0, "commit");
    assert_eq!(log[0].1, log[1].1);
}
#[test]
fn commit_only_accepted_changes_accumulate_across_channels_in_one_tick() {
    let app = App::memory();
    let source = app.scope().cage(Hsva {
        hue: 80.,
        saturation: 0.5,
        value: 0.75,
        alpha: 0.4,
    });
    app.mount(
        color_picker("accepted")
            .value(source)
            .on_value_commit(move |next| source.revise(|value| *value = next)),
    )
    .unwrap();
    let sat = slider(&app, "Saturation");
    let value = slider(&app, "Brightness");
    app.scope()
        .batch(|| {
            key(&app, sat, RangeCommand::Increase);
            key(&app, sat, RangeCommand::Increase);
            key(&app, value, RangeCommand::Decrease);
        })
        .unwrap();
    assert!((source.get().saturation - 0.52).abs() < 1e-12);
    assert!((source.get().value - 0.74).abs() < 1e-12);
    assert_eq!(source.get().alpha, 0.4);
}
#[test]
fn readonly_and_disabled_preserve_values_and_refuse_generic_drag_and_range_observers() {
    for readonly in [true, false] {
        let app = App::memory();
        let observed = Rc::new(RefCell::new(0));
        let seen = observed.clone();
        app.mount(
            color_picker("blocked")
                .readonly(readonly)
                .enabled(readonly)
                .on_value_change(move |_| *seen.borrow_mut() += 1),
        )
        .unwrap();
        let sat = slider(&app, "Saturation");
        let result = app.dispatch(
            sat,
            Event::Range {
                from: sat,
                command: RangeCommand::Decrease,
            },
        );
        if readonly {
            result.unwrap();
        } else {
            assert!(result.is_err());
        }
        assert_eq!(number(&app, sat), 1.);
        assert_eq!(*observed.borrow(), 0);
        assert!(metadata(&app, nodes(&app, "sapid-color-plane")[0]).disabled);
    }
}
#[test]
fn invalid_program_source_keeps_accepted_tree_and_later_recovers_without_callbacks() {
    let app = App::memory();
    let source = app.scope().cage(Hsva::default());
    app.mount(color_picker("invalid").value(source)).unwrap();
    let hue = slider(&app, "Hue");
    source.revise(|value| value.hue = f64::NAN);
    assert!(app.flush().is_err());
    assert_eq!(number(&app, hue), 0.);
    source.revise(|value| value.hue = 240.);
    app.flush().unwrap();
    assert_eq!(number(&app, hue), 240.);
    assert_eq!(slider(&app, "Hue"), hue);
}
#[test]
fn malformed_hex_remains_editable_and_error_link_recovers_with_same_input_identity() {
    let app = App::memory();
    app.mount(color_input("draft").default_value("#g")).unwrap();
    let id = nodes(&app, "sapid-color-editor")[0];
    let error = nodes(&app, "sapid-color-validation")[0];
    assert_eq!(input(&app, id).value, "#g");
    assert!(metadata(&app, id).invalid);
    assert_eq!(metadata(&app, id).error_message, metadata(&app, error).id);
    edit(&app, id, "#1234");
    assert!(!metadata(&app, id).invalid);
    assert!(metadata(&app, error).hidden);
    assert_eq!(nodes(&app, "sapid-color-editor")[0], id);
    edit(&app, id, "");
    assert_eq!(input(&app, id).value, "");
    assert!(!metadata(&app, id).invalid);
}
#[test]
fn hex_controlled_refusal_keeps_source_and_typed_change_precedes_generic_observer() {
    let app = App::memory();
    let source = app.scope().cage("#ff0000".to_owned());
    let log = Rc::new(RefCell::new(Vec::new()));
    let a = log.clone();
    let b = log.clone();
    app.mount(
        color_input("refuse")
            .value(source)
            .on_value_change(move |_| a.borrow_mut().push("change"))
            .on(EventKind::ValueChanged, move |_| {
                b.borrow_mut().push("observer")
            }),
    )
    .unwrap();
    let id = nodes(&app, "sapid-color-editor")[0];
    edit(&app, id, "#00ff00");
    assert_eq!(source.get(), "#ff0000");
    assert_eq!(input(&app, id).value, "#ff0000");
    assert_eq!(*log.borrow(), vec!["change", "observer"]);
}
#[test]
fn finite_keys_dimensions_and_drafts_are_rejected_before_unbounded_construction() {
    for side in [f64::NAN, 0., 127., 513.] {
        assert!(
            App::memory()
                .mount(color_picker("size").plane_size(side))
                .is_err()
        );
    }
    assert!(App::memory().mount(color_picker("x".repeat(49))).is_err());
    assert!(App::memory().mount(color_input("x".repeat(47))).is_err());
    assert!(
        App::memory()
            .mount(color_input("budget").default_value("x".repeat(257)))
            .is_err()
    );
    let app = App::memory();
    app.mount(color_picker("no-alpha").show_alpha(false).unstyled())
        .unwrap();
    assert_eq!(nodes(&app, "sapid-alpha-slider").len(), 0);
    assert_eq!(nodes(&app, "sapid-color-plane-cell").len(), 256);
}
#[test]
fn disabled_capture_still_delivers_terminal_cancel_cleanup_to_generic_observers() {
    let app = App::memory();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let observe = seen.clone();
    app.mount(
        color_picker("cancel")
            .enabled(false)
            .on(EventKind::Drag, move |event| {
                observe.borrow_mut().push(event)
            }),
    )
    .unwrap();
    let id = nodes(&app, "sapid-color-plane")[0];
    app.dispatch(
        id,
        Event::Drag {
            from: id,
            phase: glory::render::DragPhase::Cancel,
            x: 3.,
            y: 4.,
        },
    )
    .unwrap();
    assert!(matches!(
        &seen.borrow()[0],
        Event::Drag {
            phase: glory::render::DragPhase::Cancel,
            ..
        }
    ));
}
#[test]
fn hex_only_mode_keeps_field_contract_and_rejects_conflicting_popup_configuration() {
    let app = App::memory();
    app.mount(
        sapid::field("hex")
            .label("Accent")
            .description("A color")
            .required(true)
            .readonly(true)
            .form_name("accent")
            .control(|field| color_input("hex").show_picker(false).field(field)),
    )
    .unwrap();
    let id = nodes(&app, "sapid-color-editor")[0];
    let state = metadata(&app, id);
    assert!(
        state.required
            && state.readonly
            && state.labelled_by.is_some()
            && state.described_by.is_some()
    );
    assert_eq!(
        state.form_control.as_ref().map(|value| value.name.as_str()),
        Some("accent")
    );
    assert!(nodes(&app, "sapid-popover").is_empty());
    assert!(
        App::memory()
            .mount(
                color_input("bad-popup")
                    .show_picker(false)
                    .default_open(true)
            )
            .is_err()
    );
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
}
#[test]
fn composition_and_field_readonly_disable_the_live_popup_picker_channels() {
    let app = App::memory();
    app.mount(color_input("compose").default_open(true))
        .unwrap();
    let editor = nodes(&app, "sapid-color-editor")[0];
    let sat = slider(&app, "Saturation");
    assert!(!metadata(&app, sat).disabled);
    app.dispatch(editor, Event::Composition { active: true })
        .unwrap();
    app.flush().unwrap();
    assert!(metadata(&app, sat).disabled);
    app.dispatch(editor, Event::Composition { active: false })
        .unwrap();
    app.flush().unwrap();
    assert!(!metadata(&app, sat).disabled);
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
}
