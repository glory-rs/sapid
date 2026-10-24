#![cfg(feature = "chat")]
use glory::render::memory::Memory;
use glory::render::{
    Capabilities, Event, Input, KeyboardCommand, Mutation, NodeId, Props, Query, QueryToken,
    Receipt, Render,
};
use glory::ui::App;
use sapid::{PromptRejection, PromptSubmitMode, prompt_input};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
struct Queued {
    memory: Memory,
    queue: Rc<RefCell<Vec<Input>>>,
}
impl Render for Queued {
    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
    fn style_profile(&self) -> glory::style::Profile {
        glory::style::Profile::RECORD
    }
    fn apply(&mut self, operations: &[Mutation]) -> Result<Receipt, glory::render::Error> {
        self.memory.apply(operations)
    }
    fn request(&mut self, query: Query, token: QueryToken) -> Result<(), glory::render::Error> {
        self.memory.request(query, token)
    }
    fn drain(&mut self, output: &mut Vec<Input>) {
        output.extend(self.queue.borrow_mut().drain(..));
        self.memory.drain(output);
    }
}
fn queued() -> (
    App,
    Rc<RefCell<Vec<Input>>>,
    Rc<glory::core::executor::Executor>,
) {
    let queue = Rc::new(RefCell::new(Vec::new()));
    let input = queue.clone();
    let host = Rc::new(glory::core::executor::Executor::new(16));
    (
        App::with_host(host.clone(), move |runtime| Queued {
            memory: Memory::new(runtime),
            queue: input,
        }),
        queue,
        host,
    )
}
fn turn(app: &App, host: &glory::core::executor::Executor) {
    app.pump().unwrap();
    host.observe(std::task::Waker::noop());
    for _ in 0..16 {
        if !host.poll_one() {
            break;
        }
        app.pump().unwrap();
    }
    app.flush().unwrap();
}
fn find(app: &App, predicate: impl Fn(glory::render::DescriptorView<'_>) -> bool) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if predicate(node.descriptor.view()) {
            return id;
        }
        pending.extend(node.children.iter().copied());
    }
    panic!("missing node")
}
fn editor(app: &App) -> NodeId {
    find(app, |node| matches!(node.props, Props::Input(_)))
}
fn revision(app: &App, input: NodeId) -> u64 {
    let tree = app.tree();
    let Props::Input(input) = tree.node(input).unwrap().descriptor.view().props else {
        panic!("input")
    };
    input.revision
}
fn text(app: &App, input: NodeId) -> String {
    let tree = app.tree();
    let Props::Input(input) = tree.node(input).unwrap().descriptor.view().props else {
        panic!("input")
    };
    input.value.clone()
}

#[test]
fn submit_after_all_queued_edits_reads_the_final_accepted_text_without_clearing_it() {
    let (app, queue, host) = queued();
    let sent = Rc::new(RefCell::new(Vec::new()));
    let output = sent.clone();
    app.mount(
        prompt_input("tick")
            .default_value("old")
            .submit_mode(PromptSubmitMode::EnterToSend)
            .on_send(move |value| output.borrow_mut().push(value.text)),
    )
    .unwrap();
    let input = editor(&app);
    let revision = revision(&app, input);
    for event in [
        Event::ValueChanged {
            value: "first".into(),
            revision,
            composing: false,
        },
        Event::Keyboard {
            from: input,
            command: KeyboardCommand::Submit,
        },
        Event::ValueChanged {
            value: "last accepted 🙂".into(),
            revision,
            composing: false,
        },
    ] {
        queue.borrow_mut().push(Input::Event { node: input, event });
    }
    turn(&app, &host);
    assert_eq!(*sent.borrow(), ["last accepted 🙂"]);
    assert_eq!(text(&app, input), "last accepted 🙂");
}
#[test]
fn controlled_refusal_and_ime_readonly_busy_guards_use_current_accepted_state() {
    let (app, queue, host) = queued();
    let source = app.scope().cage("Accepted".to_owned());
    let busy = app.scope().cage(false);
    let readonly = app.scope().cage(false);
    let sent = Rc::new(RefCell::new(Vec::new()));
    let output = sent.clone();
    let rejected = Rc::new(RefCell::new(Vec::new()));
    let rejection = rejected.clone();
    app.mount(
        prompt_input("controlled")
            .value(source)
            .busy(busy)
            .readonly(readonly)
            .submit_mode(PromptSubmitMode::EnterToSend)
            .on_value_change(|_| {})
            .on_send(move |value| output.borrow_mut().push(value.text))
            .on_reject(move |value| rejection.borrow_mut().push(value)),
    )
    .unwrap();
    let input = editor(&app);
    let form = find(&app, |node| {
        node.field_part() == Some(glory::render::FieldPart::Form)
    });
    let version = revision(&app, input);
    for event in [
        Event::ValueChanged {
            value: "Refused".into(),
            revision: version,
            composing: false,
        },
        Event::Keyboard {
            from: input,
            command: KeyboardCommand::Submit,
        },
    ] {
        queue.borrow_mut().push(Input::Event { node: input, event });
    }
    turn(&app, &host);
    assert_eq!(*sent.borrow(), ["Accepted"]);
    assert_eq!(text(&app, input), "Accepted");
    app.dispatch(input, Event::Composition { active: true })
        .unwrap();
    app.dispatch(form, Event::Submit { submitter: None })
        .unwrap();
    turn(&app, &host);
    assert_eq!(sent.borrow().len(), 1);
    assert!(rejected.borrow().contains(&PromptRejection::Composing));
    app.dispatch(input, Event::Composition { active: false })
        .unwrap();
    busy.revise(|value| *value = true);
    app.flush().unwrap();
    app.dispatch(form, Event::Submit { submitter: None })
        .unwrap();
    turn(&app, &host);
    assert!(rejected.borrow().contains(&PromptRejection::Busy));
    busy.revise(|value| *value = false);
    readonly.revise(|value| *value = true);
    app.flush().unwrap();
    app.dispatch(form, Event::Submit { submitter: None })
        .unwrap();
    turn(&app, &host);
    assert!(rejected.borrow().contains(&PromptRejection::Readonly));
    assert_eq!(sent.borrow().len(), 1);
}
#[test]
fn bounded_declarations_stop_consuming_and_invalid_source_is_rejected_before_rendering() {
    let consumed = Rc::new(Cell::new(0));
    let counter = consumed.clone();
    let suggestions = std::iter::repeat_with(move || {
        counter.set(counter.get() + 1);
        sapid::PromptSuggestion {
            label: "Suggestion".into(),
            value: "Text".into(),
        }
    })
    .take(1_000_000);
    let app = App::memory();
    assert!(
        app.mount(prompt_input("budget").suggestions(suggestions))
            .is_err()
    );
    assert_eq!(consumed.get(), 17);
    assert!(app.tree().roots().is_empty());
    let app = App::memory();
    let value = app.scope().cage("x".repeat(1_000_000));
    assert!(app.mount(prompt_input("large").value(value)).is_err());
    assert!(app.tree().roots().is_empty());
    let app = App::memory();
    // Canonical NUL expansion is charged too, before building any nodes.
    assert!(
        app.mount(prompt_input("canonical-budget").default_value("\0".repeat(22_000)))
            .is_err()
    );
    assert!(app.tree().roots().is_empty());
}
#[test]
fn suggestions_only_propose_text_cancel_only_runs_the_application_callback_and_owner_cleans_up()
{
    let app = App::memory();
    let busy = app.scope().cage(false);
    let source = app.scope().cage("Original".to_owned());
    let proposals = Rc::new(RefCell::new(Vec::new()));
    let proposed = proposals.clone();
    let sends = Rc::new(Cell::new(0));
    let sent = sends.clone();
    let cancelled = Rc::new(Cell::new(0));
    let cancel = cancelled.clone();
    app.mount(
        prompt_input("actions")
            .value(source)
            .busy(busy)
            .suggestion(sapid::PromptSuggestion {
                label: "Explain this".into(),
                value: "Proposed suggestion".into(),
            })
            .on_value_change(move |value| proposed.borrow_mut().push(value))
            .on_send(move |_| sent.set(sent.get() + 1))
            .on_cancel(move || cancel.set(cancel.get() + 1)),
    )
    .unwrap();
    let suggest = find(
        &app,
        |node| matches!(node.props,Props::Button(button)if button.label=="Explain this"),
    );
    app.dispatch(suggest, Event::Activate).unwrap();
    assert_eq!(*proposals.borrow(), ["Proposed suggestion"]);
    assert_eq!(text(&app, editor(&app)), "Original");
    assert_eq!(sends.get(), 0);
    busy.revise(|value| *value = true);
    app.flush().unwrap();
    let cancel = find(
        &app,
        |node| matches!(node.props,Props::Button(button)if button.label=="Cancel request"),
    );
    app.dispatch(cancel, Event::Activate).unwrap();
    assert_eq!(cancelled.get(), 1);
    assert!(busy.get());
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(Rc::strong_count(&proposals), 1);
    assert_eq!(Rc::strong_count(&sends), 1);
    assert_eq!(Rc::strong_count(&cancelled), 1);
}

#[test]
fn deferred_send_has_a_real_query_barrier_and_pending_owner_retirement_cancels_publication() {
    let (app, _, host) = queued();
    let input = app.scope().cage(None);
    let weak = app.scope().downgrade();
    let queries = Rc::new(Cell::new(0));
    let output = queries.clone();
    app.mount(
        prompt_input("query")
            .default_value("Accepted")
            .input_ref(input)
            .on_send(move |_| {
                let scope = weak.upgrade().unwrap();
                let output = output.clone();
                scope.request(Query::Focus(input.get().unwrap()),move |result| {
            assert!(result.is_err());output.set(output.get()+1);
        }).expect("business callback runs after flush, so its query can flush a real barrier");
            }),
    )
    .unwrap();
    let form = find(&app, |node| {
        node.field_part() == Some(glory::render::FieldPart::Form)
    });
    app.dispatch(form, Event::Submit { submitter: None })
        .unwrap();
    assert_eq!(queries.get(), 0);
    assert_eq!(app.runtime_stats().tasks, 1);
    turn(&app, &host);
    assert_eq!(queries.get(), 1);
    assert_eq!(app.stats().queries, 0);
    assert_eq!(app.runtime_stats().tasks, 0);
    app.dispatch(form, Event::Submit { submitter: None })
        .unwrap();
    assert_eq!(app.runtime_stats().tasks, 1);
    app.clear().unwrap();
    turn(&app, &host);
    assert_eq!(queries.get(), 1);
    assert_eq!(app.runtime_stats().tasks, 0);
    assert_eq!(app.stats().nodes, 0);
}

#[test]
fn queued_request_rechecks_current_hidden_inert_and_disabled_ancestry() {
    for boundary in 0..3 {
        let (app, _, host) = queued();
        let blocked = app.scope().cage(false);
        let sent = Rc::new(Cell::new(0));
        let output = sent.clone();
        let prompt = prompt_input("late-boundary")
            .default_value("Accepted")
            .on_send(move |_| output.set(output.get() + 1));
        if boundary == 2 {
            app.mount(
                sapid::fieldset("late-fieldset")
                    .disabled(blocked)
                    .fill(prompt),
            )
            .unwrap();
        } else {
            let semantics = app.scope().bond(move || {
                Some(Box::new(glory::render::Semantics {
                    hidden: boundary == 0 && blocked.get(),
                    inert: boundary == 1 && blocked.get(),
                    ..Default::default()
                }))
            });
            app.mount(glory::ui::view().semantics(semantics).fill(prompt))
                .unwrap();
        }
        let form = find(&app, |node| {
            node.field_part() == Some(glory::render::FieldPart::Form)
        });
        app.dispatch(form, Event::Submit { submitter: None })
            .unwrap();
        assert_eq!(app.runtime_stats().tasks, 1);
        blocked.revise(|value| *value = true);
        app.flush().unwrap();
        turn(&app, &host);
        assert_eq!(sent.get(), 0);
        blocked.revise(|value| *value = false);
        app.flush().unwrap();
        app.dispatch(form, Event::Submit { submitter: None })
            .unwrap();
        turn(&app, &host);
        assert_eq!(sent.get(), 1);
    }
}
