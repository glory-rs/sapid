#![cfg(feature = "chat")]
use glory::Cage;
use glory::render::{Event, NodeId, Props};
use glory::ui::App;
use sapid::{FieldOptions, Questionnaire, question, question_step, questionnaire};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn form(name: Cage<String>, consent: Cage<bool>) -> Questionnaire {
    questionnaire("answers")
        .step(
            question_step("identity", "Identity").question(
                question(
                    "name",
                    name,
                    FieldOptions::default().validate(|name: &String| {
                        if name.trim().is_empty() {
                            vec!["Name is required".into()]
                        } else {
                            Vec::new()
                        }
                    }),
                    |field, control| {
                        let value = field.value();
                        sapid::text_field()
                            .field(control)
                            .value(value)
                            .on_value_change(move |proposal| {
                                value.revise(|value| *value = proposal);
                                field.changed().unwrap();
                            })
                    },
                )
                .label("Your name")
                .required(true),
            ),
        )
        .step(
            question_step("permission", "Permission").question(
                question(
                    "consent",
                    consent,
                    FieldOptions::default().validate(|consent: &bool| {
                        if *consent {
                            Vec::new()
                        } else {
                            vec!["Consent is required".into()]
                        }
                    }),
                    |field, control| {
                        let value = field.value();
                        sapid::checkbox()
                            .field(control)
                            .checked(value)
                            .on_checked_change(move |proposal| {
                                value.revise(|value| *value = proposal);
                                field.changed().unwrap();
                            })
                    },
                )
                .label("Consent"),
            ),
        )
}
fn button(app: &App, label: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if matches!(node.descriptor.view().props,Props::Button(value)if value.label==label) {
            return id;
        }
        pending.extend(node.children.iter().copied());
    }
    panic!("missing button {label}")
}
fn text(app: &App, content: &str) -> bool {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if matches!(node.descriptor.view().props,Props::Text(value)if value.content==content) {
            return true;
        }
        pending.extend(node.children.iter().copied());
    }
    false
}
fn app() -> (App, Rc<glory::core::executor::Executor>) {
    let host = Rc::new(glory::core::executor::Executor::new(16));
    (
        App::with_host(host.clone(), glory::render::memory::Memory::new),
        host,
    )
}
fn turn(app: &App, host: &glory::core::executor::Executor) {
    for _ in 0..16 {
        app.pump().unwrap();
        host.observe(std::task::Waker::noop());
        if !host.poll_one() {
            break;
        }
    }
    app.flush().unwrap();
}
#[test]
fn typed_steps_validate_before_advancing_and_fresh_final_validation_returns_to_earlier_error() {
    let (app, host) = app();
    let name = app.scope().cage(String::new());
    let consent = app.scope().cage(false);
    let complete = Rc::new(Cell::new(0));
    let output = complete.clone();
    app.mount(form(name, consent).on_complete(move |token| {
        output.set(output.get() + 1);
        assert!(token.complete(Ok(())));
    }))
    .unwrap();
    app.dispatch(button(&app, "Next question step"), Event::Activate)
        .unwrap();
    turn(&app, &host);
    assert!(text(&app, "Name is required"));
    assert_eq!(complete.get(), 0);
    name.revise(|value| *value = "Accepted name".into());
    app.flush().unwrap();
    app.dispatch(button(&app, "Next question step"), Event::Activate)
        .unwrap();
    turn(&app, &host);
    assert!(app.tree().active(button(&app, "Submit answers")));
    app.dispatch(button(&app, "Submit answers"), Event::Activate)
        .unwrap();
    turn(&app, &host);
    assert!(text(&app, "Consent is required"));
    assert_eq!(complete.get(), 0);
    consent.revise(|value| *value = true);
    name.revise(String::clear);
    app.flush().unwrap();
    app.dispatch(button(&app, "Submit answers"), Event::Activate)
        .unwrap();
    turn(&app, &host);
    assert!(text(&app, "Name is required"));
    assert_eq!(complete.get(), 0);
    assert!(app.tree().active(button(&app, "Next question step")));
    name.revise(|value| *value = "Fixed name".into());
    app.flush().unwrap();
    app.dispatch(button(&app, "Next question step"), Event::Activate)
        .unwrap();
    turn(&app, &host);
    assert!(consent.get());
    app.dispatch(button(&app, "Submit answers"), Event::Activate)
        .unwrap();
    turn(&app, &host);
    assert_eq!(complete.get(), 1);
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(app.runtime_stats().tasks, 0);
}
#[test]
fn refused_controlled_step_proposes_without_switching_the_accepted_panel() {
    let (app, host) = app();
    let name = app.scope().cage("Accepted".to_owned());
    let consent = app.scope().cage(true);
    let active = app.scope().cage(0usize);
    let proposals = Rc::new(RefCell::new(Vec::new()));
    let output = proposals.clone();
    app.mount(
        form(name, consent)
            .active(active)
            .on_step_change(move |value| output.borrow_mut().push(value)),
    )
    .unwrap();
    app.dispatch(button(&app, "Next question step"), Event::Activate)
        .unwrap();
    turn(&app, &host);
    assert_eq!(*proposals.borrow(), [1]);
    assert_eq!(active.get(), 0);
    assert!(app.tree().active(button(&app, "Next question step")));
    app.clear().unwrap();
    assert_eq!(Rc::strong_count(&proposals), 1);
}
#[test]
fn step_intake_is_bounded_and_invalid_schema_never_calls_question_factories() {
    let app = App::memory();
    let calls = Rc::new(Cell::new(0));
    let output = calls.clone();
    let invalid = questionnaire("bad")
        .step(question_step("duplicate", "One").question(question(
            "a",
            app.scope().cage(false),
            FieldOptions::default(),
            move |_, _| {
                output.set(output.get() + 1);
                glory::ui::text()
            },
        )))
        .step(question_step("duplicate", "Two"));
    assert!(app.mount(invalid).is_err());
    assert_eq!(calls.get(), 0);
    assert!(app.tree().roots().is_empty());
    let consumed = Rc::new(Cell::new(0));
    let output = consumed.clone();
    let schema = questionnaire("million").steps(
        std::iter::repeat_with(move || {
            output.set(output.get() + 1);
            question_step("step", "Step")
        })
        .take(1_000_000),
    );
    assert_eq!(consumed.get(), 17);
    assert!(app.mount(schema).is_err());
}

#[test]
fn asynchronous_final_validation_is_revoked_by_back_and_owner_retirement() {
    let (app, host) = app();
    let ready = Rc::new(Cell::new(false));
    let wait = ready.clone();
    let wake = Rc::new(RefCell::new(None::<std::task::Waker>));
    let pending = wake.clone();
    let done = Rc::new(Cell::new(0));
    let output = done.clone();
    let value = app.scope().cage(true);
    let field = move || {
        question(
            "consent",
            value,
            FieldOptions::default(),
            |field, control| sapid::checkbox().field(control).checked(field.value()),
        )
    };
    let schema = questionnaire("async")
        .default_active(1)
        .step(question_step("one", "One").question(field()))
        .step(question_step("two", "Two").question(question(
            "confirm",
            value,
            FieldOptions::default().validate_async(move |_| {
                let wait = wait.clone();
                let pending = pending.clone();
                Box::pin(std::future::poll_fn(move |context| {
                    if wait.get() {
                        std::task::Poll::Ready(Vec::new())
                    } else {
                        *pending.borrow_mut() = Some(context.waker().clone());
                        std::task::Poll::Pending
                    }
                }))
            }),
            |field, control| sapid::checkbox().field(control).checked(field.value()),
        )))
        .on_complete(move |token| {
            output.set(output.get() + 1);
            token.complete(Ok(()));
        });
    app.mount(schema).unwrap();
    app.dispatch(button(&app, "Submit answers"), Event::Activate)
        .unwrap();
    turn(&app, &host);
    assert!(wake.borrow().is_some());
    assert_eq!(done.get(), 0);
    app.dispatch(button(&app, "Previous question step"), Event::Activate)
        .unwrap();
    turn(&app, &host);
    ready.set(true);
    if let Some(waker) = wake.borrow_mut().take() {
        waker.wake()
    }
    turn(&app, &host);
    assert_eq!(done.get(), 0);
    assert!(app.tree().active(button(&app, "Next question step")));
    app.dispatch(button(&app, "Next question step"), Event::Activate)
        .unwrap();
    turn(&app, &host);
    app.dispatch(button(&app, "Submit answers"), Event::Activate)
        .unwrap();
    app.clear().unwrap();
    turn(&app, &host);
    assert_eq!(done.get(), 0);
    assert_eq!(app.runtime_stats().tasks, 0);
    assert_eq!(Rc::strong_count(&done), 1);
}
