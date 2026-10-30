use glory::prelude::*;
use glory::render::memory::Memory;
use glory::render::{Capabilities, Input, Mutation, Query, QueryToken, Receipt, Render};
use glory::render::{EventKind, InputProps, NodeId, Props};
use glory::style::FieldSizing;
use sapid::{
    button, field, fieldset, input_addon, input_group, input_section, password_input, text_field,
    textarea,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            return id;
        }
        pending.extend(node.children.iter().copied());
    }
    panic!("missing {class}")
}
fn props(app: &App, id: NodeId) -> InputProps {
    let Props::Input(p) = app.tree().node(id).unwrap().descriptor.view().props.clone() else {
        panic!("expected input")
    };
    p
}
fn edit(app: &App, id: NodeId, value: &str, composing: bool) {
    app.dispatch(
        id,
        Event::ValueChanged {
            value: value.into(),
            revision: props(app, id).revision,
            composing,
        },
    )
    .unwrap();
}

#[test]
fn textarea_reuses_controlled_tick_order_field_metadata_and_composition() {
    struct Queued {
        memory: Memory,
        queue: Rc<RefCell<Vec<Input>>>,
    }
    impl Render for Queued {
        fn style_profile(&self) -> glory::style::Profile {
            glory::style::Profile::RECORD
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities::default()
        }
        fn apply(
            &mut self,
            mutations: &[Mutation],
        ) -> std::result::Result<Receipt, glory::render::Error> {
            self.memory.apply(mutations)
        }
        fn request(
            &mut self,
            query: Query,
            token: QueryToken,
        ) -> std::result::Result<(), glory::render::Error> {
            self.memory.request(query, token)
        }
        fn drain(&mut self, inputs: &mut Vec<Input>) {
            inputs.extend(self.queue.borrow_mut().drain(..));
        }
    }
    let queue = Rc::new(RefCell::new(Vec::new()));
    let queued = queue.clone();
    let app = App::new(move |runtime| Queued {
        memory: Memory::new(runtime),
        queue: queued,
    });
    let value = app.scope().cage("original\ntext".to_string());
    let order = Rc::new(RefCell::new(Vec::new()));
    let typed = order.clone();
    let observer = order.clone();
    app.mount(
        field("bio")
            .label("Biography")
            .required(true)
            .control(move |context| {
                textarea()
                    .field(context)
                    .value(value)
                    .on_value_change({
                        let typed = typed.clone();
                        move |v| typed.borrow_mut().push(v)
                    })
                    .on(EventKind::ValueChanged, {
                        let observer = observer.clone();
                        move |_| observer.borrow_mut().push("observer".into())
                    })
            }),
    )
    .unwrap();
    let id = find(&app, "sapid-textarea");
    assert!(props(&app, id).multiline);
    assert!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .required
    );
    let revision = props(&app, id).revision;
    for (value, composing) in [("first\n行", true), ("second\n行", false)] {
        queue.borrow_mut().push(Input::Event {
            node: id,
            event: Event::ValueChanged {
                value: value.into(),
                composing,
                revision,
            },
        });
    }
    assert_eq!(app.pump().unwrap(), 2);
    assert_eq!(
        *order.borrow(),
        ["first\n行", "observer", "second\n行", "observer"]
    );
    assert_eq!(props(&app, id).value, value.get());
    assert_eq!(props(&app, id).revision, revision + 1);
    value.revise(|v| *v = "external\r\ntext\0".into());
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "external\ntext�");
    assert_eq!(order.borrow().len(), 4);
    assert_eq!(find(&app, "sapid-textarea"), id);
}

#[test]
fn textarea_row_constraints_follow_effective_style_and_reject_invalid_configuration() {
    let app = App::memory();
    let style = app.scope().cage(Style {
        line_height: Some(20.0),
        ..Style::new()
            .padding(Edges::all(2.0))
            .border_width(Edges::all(1.0))
    });
    app.mount(
        textarea()
            .autosize(true)
            .min_rows(2)
            .max_rows(5)
            .style(style),
    )
    .unwrap();
    let id = find(&app, "sapid-textarea");
    let sizing = || app.tree().node(id).unwrap().descriptor.view().style.clone();
    assert_eq!(sizing().field_sizing, Some(FieldSizing::Content));
    assert_eq!(sizing().height, Some(Length::Auto));
    assert_eq!(sizing().min_height, Some(Length::Px(46.0)));
    assert_eq!(sizing().max_height, Some(Limit::Px(106.0)));
    style.revise(|s| {
        s.line_height = Some(30.0);
        s.height = Some(Length::Px(75.0));
    });
    app.flush().unwrap();
    assert_eq!(sizing().min_height, Some(Length::Px(66.0)));
    assert_eq!(sizing().height, Some(Length::Px(75.0)));
    for widget in [
        textarea().rows(0),
        textarea().min_rows(0),
        textarea().max_rows(0),
        textarea().min_rows(5).max_rows(2),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert!(app.tree().is_empty());
    }
}

#[test]
fn password_visibility_is_independent_of_value_revision_and_field_disabling() {
    let app = App::memory();
    let disabled = app.scope().cage(false);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let seen = calls.clone();
    app.mount(
        field("secret")
            .label("Password")
            .enabled(disabled.map(|v| !v))
            .control(move |context| {
                password_input()
                    .field(context)
                    .default_value("秘密🦀")
                    .on_visibility_change({
                        let seen = seen.clone();
                        move |v| seen.borrow_mut().push(v)
                    })
            }),
    )
    .unwrap();
    let id = find(&app, "sapid-password-input");
    let toggle = find(&app, "sapid-password-toggle");
    let before = props(&app, id);
    app.dispatch(toggle, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(props(&app, id).purpose, Purpose::Text);
    assert_eq!(
        (props(&app, id).value, props(&app, id).revision),
        (before.value.clone(), before.revision)
    );
    assert!(
        matches!(&app.tree().node(toggle).unwrap().descriptor.view().props.clone(),Props::Button(p) if p.label=="Hide password")
    );
    disabled.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(!app.tree().enabled(id));
    assert!(!app.tree().enabled(toggle));
    assert!(!app.dispatch(toggle, Event::Activate).unwrap());
    disabled.revise(|v| *v = false);
    app.flush().unwrap();
    app.dispatch(toggle, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(props(&app, id), before);
    assert_eq!(*calls.borrow(), [true, false]);
}

#[test]
fn visibility_controlled_rejection_external_updates_and_owner_cleanup() {
    let app = App::memory();
    let visible = app.scope().cage(false);
    let reference = app.scope().cage(None);
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    app.mount(
        password_input()
            .name("Secret")
            .visible(visible)
            .default_visible(true)
            .node_ref(reference)
            .value("fixed")
            .on_visibility_change(move |_| seen.set(seen.get() + 1)),
    )
    .unwrap();
    let id = reference.get().unwrap();
    let toggle = find(&app, "sapid-password-toggle");
    app.dispatch(toggle, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(props(&app, id).purpose, Purpose::Password);
    assert!(!visible.get());
    edit(&app, id, "rejected", true);
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "fixed");
    visible.revise(|v| *v = true);
    app.flush().unwrap();
    assert_eq!(props(&app, id).purpose, Purpose::Text);
    assert_eq!(calls.get(), 1);
    app.clear().unwrap();
    assert_eq!(reference.get(), None);
    assert!(app.tree().is_empty());
    assert!(!app.dispatch(toggle, Event::Activate).unwrap());
    assert_eq!(calls.get(), 1);
    assert!(
        App::memory()
            .mount(password_input().visibility_labels("", "Hide"))
            .is_err()
    );
}

#[test]
fn input_group_decorations_do_not_replace_control_label_or_intrinsic_controls() {
    let app = App::memory();
    let clicks = Rc::new(Cell::new(0));
    let seen = clicks.clone();
    app.mount(
        fieldset("account")
            .legend("Account")
            .fill(field("domain").label("Site").control(move |context| {
                input_group()
                    .start(input_addon().content("https://"))
                    .input(text_field().field(context).default_value("example"))
                    .end(input_section().fill(button().label("Visit site").on_press({
                        let seen = seen.clone();
                        move || seen.set(seen.get() + 1)
                    })))
            })),
    )
    .unwrap();
    let input = find(&app, "sapid-text-field");
    let addon = find(&app, "sapid-input-addon");
    assert!(
        app.tree()
            .node(input)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .labelled_by
            .is_some()
    );
    assert!(
        app.tree()
            .node(addon)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .is_none()
    );
    assert!(matches!(
        app.tree()
            .node(addon)
            .unwrap()
            .descriptor
            .view()
            .props
            .clone(),
        Props::Text(_)
    ));
    app.dispatch(find(&app, "sapid-button"), Event::Activate)
        .unwrap();
    assert_eq!(clicks.get(), 1);
    app.clear().unwrap();
    assert!(app.tree().is_empty());
    let empty = App::memory();
    assert!(empty.mount(input_group()).is_err());
    assert!(empty.tree().is_empty());
}

#[test]
fn specialized_inputs_ssr_escape_and_keep_stable_native_shapes() {
    let snapshot = glory::ssr::snapshot(
        col().fill((
            field("bio").label("Biography").control(|context| {
                textarea()
                    .field(context)
                    .autosize(true)
                    .min_rows(2)
                    .max_rows(4)
                    .default_value("<private>\n第二行")
            }),
            field("secret")
                .label("Password")
                .control(|context| password_input().field(context).default_value("<&\"")),
            input_group()
                .start(input_addon().content("<prefix>"))
                .input(text_field().name("Address")),
        )),
    )
    .unwrap();
    for token in [
        "<textarea",
        "field-sizing:content",
        "&lt;private&gt;",
        "type=\"password\"",
        "Show password",
        "&lt;prefix&gt;",
    ] {
        assert!(snapshot.html.contains(token), "{token}: {}", snapshot.html);
    }
    let plain = glory::ssr::snapshot(textarea().unstyled().class("custom").rows(2)).unwrap();
    assert!(
        plain.html.contains("sapid-textarea")
            && plain.html.contains("sapid-unstyled")
            && plain.html.contains("custom")
    );
    assert!(!plain.html.contains("background-color:"));
}
