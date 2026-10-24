use glory::render::{memory::Memory, *};
use glory::ui::App;
use sapid::{CopyState, copy_button};
use std::{cell::RefCell, rc::Rc};
#[derive(Default)]
struct Wire {
    events: RefCell<Vec<Input>>,
    actions: RefCell<Vec<Action>>,
}
struct Driver {
    tree: Memory,
    wire: Rc<Wire>,
}
impl Render for Driver {
    fn style_profile(&self) -> glory::style::Profile {
        glory::style::Profile::RECORD
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            clipboard_write: true,
            files: None,
            ..Default::default()
        }
    }
    fn apply(&mut self, m: &[Mutation]) -> Result<Receipt, Error> {
        self.tree.apply(m)
    }
    fn request(&mut self, q: Query, t: QueryToken) -> Result<(), Error> {
        self.tree.request(q, t)
    }
    fn action(&mut self, a: Action) -> Result<(), Error> {
        self.tree.admit_clipboard(a)?;
        self.wire.actions.borrow_mut().push(a);
        Ok(())
    }
    fn drain(&mut self, v: &mut Vec<Input>) {
        v.extend(self.wire.events.borrow_mut().drain(..));
        self.tree.drain(v)
    }
}
fn app() -> (App, Rc<Wire>) {
    let wire = Rc::new(Wire::default());
    let w = wire.clone();
    (
        App::new(move |r| Driver {
            tree: Memory::new(r),
            wire: w,
        }),
        wire,
    )
}
fn complete(wire: &Wire, id: NodeId, request: u64, revision: u64, outcome: ClipboardOutcome) {
    wire.events.borrow_mut().push(Input::Event {
        node: id,
        event: Event::Clipboard {
            request,
            revision,
            outcome,
        },
    })
}
#[test]
fn pending_result_order_duplicate_source_cancellation_and_retirement() {
    let (app, wire) = app();
    let scope = app.scope();
    let node = scope.cage(None);
    let source = scope.cage("one\r\n😀\0".to_string());
    let state = scope.cage(CopyState::Idle);
    let log = Rc::new(RefCell::new(vec![]));
    let typed = log.clone();
    let generic = log.clone();
    let activate = log.clone();
    app.mount(
        copy_button()
            .node_ref(node)
            .source(source)
            .state_ref(state)
            .on_result(move |r| {
                typed
                    .borrow_mut()
                    .push(format!("typed:{r:?}:{:?}", state.get()))
            })
            .on(EventKind::Clipboard, move |_| {
                generic
                    .borrow_mut()
                    .push(format!("generic:{:?}", state.get()))
            })
            .on(EventKind::Activate, move |_| {
                activate
                    .borrow_mut()
                    .push(format!("activate:{:?}", state.get()))
            }),
    )
    .unwrap();
    let id = node.get().unwrap();
    let text = app
        .tree()
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .clipboard
        .clone()
        .unwrap();
    assert_eq!(text.text, "one\n😀�");
    assert_eq!(source.get(), "one\r\n😀\0");
    app.dispatch(id, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(state.get(), CopyState::Pending);
    assert_eq!(wire.actions.borrow().len(), 1);
    assert_eq!(log.borrow()[0], "activate:Pending");
    app.dispatch(id, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(wire.actions.borrow().len(), 1);
    complete(&wire, id, 1, 0, ClipboardOutcome::Written);
    complete(&wire, id, 1, 0, ClipboardOutcome::Written);
    app.pump().unwrap();
    assert_eq!(state.get(), CopyState::Written);
    assert_eq!(
        &log.borrow()[2..],
        &["typed:Written:Written", "generic:Written"]
    );
    app.dispatch(id, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(wire.actions.borrow().len(), 2);
    source.revise(|v| *v = "two".into());
    app.flush().unwrap();
    assert_eq!(state.get(), CopyState::Idle);
    complete(&wire, id, 2, 0, ClipboardOutcome::Written);
    app.pump().unwrap();
    assert_eq!(state.get(), CopyState::Idle);
    app.dispatch(id, Event::Activate).unwrap();
    app.flush().unwrap();
    assert!(matches!(
        wire.actions.borrow().last(),
        Some(Action::CopyText {
            request: 3,
            revision: 1,
            ..
        })
    ));
    complete(&wire, id, 3, 1, ClipboardOutcome::Denied);
    app.pump().unwrap();
    assert_eq!(state.get(), CopyState::Denied);
    app.dispatch(id, Event::Activate).unwrap();
    app.flush().unwrap();
    app.clear().unwrap();
    complete(&wire, id, 4, 1, ClipboardOutcome::Written);
    app.pump().unwrap();
    assert_eq!(app.runtime_stats().tasks, 0);
}
#[test]
fn unavailable_is_explicit_disabled_is_inert_and_invalid_source_preserves_accepted() {
    let app = App::memory();
    let node = app.scope().cage(None);
    let state = app.scope().cage(CopyState::Idle);
    let enabled = app.scope().cage(false);
    let source = app.scope().cage("old".to_string());
    app.mount(
        copy_button()
            .source(source)
            .node_ref(node)
            .state_ref(state)
            .enabled(enabled),
    )
    .unwrap();
    let id = node.get().unwrap();
    app.dispatch(id, Event::Activate).unwrap();
    assert_eq!(state.get(), CopyState::Idle);
    enabled.revise(|v| *v = true);
    app.flush().unwrap();
    app.dispatch(id, Event::Activate).unwrap();
    assert_eq!(state.get(), CopyState::Unavailable);
    source.revise(|v| *v = "x".repeat(65537));
    assert!(app.flush().is_err());
    assert_eq!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .clipboard
            .as_ref()
            .unwrap()
            .text,
        "old"
    );
    source.revise(|v| *v = "\0".repeat(65536));
    assert!(app.flush().is_err());
    assert_eq!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .clipboard
            .as_ref()
            .unwrap()
            .text,
        "old"
    );
    source.revise(|v| *v = "new".into());
    app.flush().unwrap();
    assert_eq!(state.get(), CopyState::Idle);
}
#[test]
fn explicit_timer_owns_one_task_and_cancels_on_source_and_owner() {
    use glory::core::executor::Executor;
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll, Waker},
    };
    struct Wait(Rc<std::cell::Cell<bool>>);
    impl Future for Wait {
        type Output = ();
        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
            if self.0.get() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }
    let exec = Rc::new(Executor::new(8));
    let app = App::with_host(exec.clone(), Memory::new);
    let node = app.scope().cage(None);
    let source = app.scope().cage("x".to_string());
    let status = app.scope().cage(CopyState::Idle);
    let done = Rc::new(std::cell::Cell::new(false));
    let d = done.clone();
    app.mount(
        copy_button()
            .node_ref(node)
            .state_ref(status)
            .source(source)
            .feedback_delay(move || Box::pin(Wait(d.clone()))),
    )
    .unwrap();
    let id = node.get().unwrap();
    app.dispatch(id, Event::Activate).unwrap();
    assert_eq!(status.get(), CopyState::Unavailable);
    assert_eq!(app.runtime_stats().tasks, 1);
    done.set(true);
    exec.observe(Waker::noop());
    while exec.poll_one() {}
    app.flush().unwrap();
    assert_eq!(status.get(), CopyState::Idle);
    done.set(false);
    app.dispatch(id, Event::Activate).unwrap();
    assert_eq!(app.runtime_stats().tasks, 1);
    source.revise(|v| *v = "next".into());
    app.flush().unwrap();
    assert_eq!(app.runtime_stats().tasks, 0);
    assert_eq!(status.get(), CopyState::Idle);
    app.dispatch(id, Event::Activate).unwrap();
    assert_eq!(app.runtime_stats().tasks, 1);
    app.clear().unwrap();
    assert_eq!(app.runtime_stats().tasks, 0);
}
#[test]
fn four_themes_unstyled_explicit_overrides_and_ssr_no_clipboard_leak() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let app = App::memory();
        let node = app.scope().cage(None);
        app.mount(
            copy_button()
                .theme(theme)
                .node_ref(node)
                .class("custom")
                .style(glory::style::Style::new().width(glory::style::Length::Px(111.)))
                .source("private"),
        )
        .unwrap();
        let tree = app.tree();
        let d = tree.node(node.get().unwrap()).unwrap().descriptor.view();
        assert!(d.classes.iter().any(|v| v == "sapid-copy-button"));
        assert!(d.classes.iter().any(|v| v == "custom"));
        assert_eq!(d.style.width, Some(glory::style::Length::Px(111.)));
        assert!(d.style.background.is_some());
    }
    let html = glory::ssr::render(copy_button().source("private-secret").unstyled()).unwrap();
    assert!(!html.contains("private-secret"));
    assert!(html.contains("sapid-unstyled"));
    assert!(html.contains(">Copy<"));
}
#[cfg(all(feature = "code", feature = "json"))]
#[test]
fn code_and_json_copy_slots_share_only_the_accepted_complete_source() {
    let (app, wire) = app();
    let code = app.scope().cage("fn main() {}\n".to_string());
    let json = app.scope().cage("{\"branch\":{\"hidden\":1}}".to_string());
    let c = app.scope().cage(None);
    let j = app.scope().cage(None);
    app.mount(
        glory::ui::col().fill((
            sapid::code_highlight()
                .source(code)
                .copy_button(copy_button().node_ref(c).label("Copy code")),
            sapid::json_viewer("copy-json")
                .source(json)
                .copy_button(copy_button().node_ref(j).label("Copy JSON")),
        )),
    )
    .unwrap();
    let text = |id| {
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .clipboard
            .as_ref()
            .unwrap()
            .text
            .clone()
    };
    assert_eq!(text(c.get().unwrap()), code.get());
    assert_eq!(text(j.get().unwrap()), json.get());
    app.dispatch(j.get().unwrap(), Event::Activate).unwrap();
    app.flush().unwrap();
    assert!(matches!(
        wire.actions.borrow().last(),
        Some(Action::CopyText { revision: 0, .. })
    ));
    json.revise(|v| *v = "{bad}".into());
    assert!(app.flush().is_err());
    assert_eq!(text(j.get().unwrap()), "{\"branch\":{\"hidden\":1}}");
    code.revise(|v| *v = "let next = 42;\r\n".into());
    json.revise(|v| *v = "{\"new\":2}".into());
    app.flush().unwrap();
    assert_eq!(text(c.get().unwrap()), "let next = 42;\n");
    assert_eq!(text(j.get().unwrap()), "{\"new\":2}");
}
