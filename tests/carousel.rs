use glory::render::{Event, EventKind, Navigation, NodeId, Props};
use glory::ui::{App, Widget};
use sapid::carousel;
use std::{cell::Cell, rc::Rc};
fn nodes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut found = vec![];
    while let Some(id) = todo.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            found.push(id);
        }
        todo.extend(node.children.iter().copied());
    }
    found
}
fn position(app: &App) -> String {
    let id = nodes(app, "sapid-carousel-position")[0];
    let tree = app.tree();
    let Props::Text(text) = tree.node(id).unwrap().descriptor.view().props else {
        panic!()
    };
    text.content.clone()
}
#[test]
fn million_source_calls_only_window_factories_and_retains_current_editor_owner() {
    let app = App::memory();
    let current = app.scope().cage(500_000usize);
    let builds = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let built = builds.clone();
    let retired = drops.clone();
    app.mount(
        carousel()
            .count(1_000_000usize)
            .current(current)
            .overscan(3)
            .render_slide(move |index| {
                built.set(built.get() + 1);
                let retired = retired.clone();
                glory::ui::component(move |scope| {
                    let retired = retired.clone();
                    scope.on_cleanup(move || retired.set(retired.get() + 1))?;
                    glory::ui::input()
                        .class(format!("editor-{index}"))
                        .value("Draft")
                        .show_in(scope)
                })
            }),
    )
    .unwrap();
    assert_eq!(builds.get(), 7);
    assert_eq!(nodes(&app, "sapid-carousel-slide").len(), 7);
    let editor = nodes(&app, "editor-500001")[0];
    current.revise(|value| *value = 500_001);
    app.flush().unwrap();
    assert_eq!(builds.get(), 8);
    assert_eq!(drops.get(), 1);
    assert_eq!(nodes(&app, "editor-500001")[0], editor);
    let mounted = nodes(&app, "sapid-carousel-slide");
    assert_eq!(mounted.len(), 7);
    let tree = app.tree();
    assert_eq!(
        mounted
            .iter()
            .filter(|id| !tree.node(**id).unwrap().descriptor.view().hidden())
            .count(),
        1
    );
    drop(tree);
    drop(app);
    assert_eq!(drops.get(), 8);
}
#[test]
fn controlled_proposals_precede_generic_and_external_sources_are_silent() {
    let app = App::memory();
    let current = app.scope().cage(1usize);
    let proposals = app.scope().cage(Vec::<String>::new());
    app.mount(
        carousel()
            .count(4usize)
            .current(current)
            .on_current_change(move |next| {
                proposals.revise(|log| log.push(format!("typed:{next}")))
            })
            .on(EventKind::Navigate, move |_| {
                proposals.revise(|log| log.push("generic".into()))
            })
            .render_slide(|index| glory::ui::text().content(format!("Slide {index}"))),
    )
    .unwrap();
    let root = nodes(&app, "sapid-carousel")[0];
    app.dispatch(
        root,
        Event::Navigate {
            from: root,
            command: Navigation::Next,
        },
    )
    .unwrap();
    assert_eq!(proposals.get(), vec!["typed:2", "generic"]);
    assert_eq!(position(&app), "2 / 4");
    assert_eq!(current.get(), 1);
    current.revise(|value| *value = 3);
    app.flush().unwrap();
    assert_eq!(position(&app), "4 / 4");
    assert_eq!(proposals.get().len(), 2);
    app.dispatch(
        root,
        Event::Navigate {
            from: root,
            command: Navigation::Next,
        },
    )
    .unwrap();
    assert_eq!(proposals.get().len(), 3);
}
#[test]
fn local_buttons_keyboard_edges_empty_and_count_shrink() {
    let app = App::memory();
    let count = app.scope().cage(3usize);
    app.mount(
        carousel()
            .count(count)
            .default_current(1)
            .render_slide(|index| glory::ui::text().content(index.to_string())),
    )
    .unwrap();
    let next = nodes(&app, "sapid-carousel-next")[0];
    app.dispatch(next, Event::Activate).unwrap();
    assert_eq!(position(&app), "3 / 3");
    count.revise(|value| *value = 1);
    app.flush().unwrap();
    assert_eq!(position(&app), "1 / 1");
    count.revise(|value| *value = 0);
    app.flush().unwrap();
    assert_eq!(position(&app), "No slides");
    assert!(nodes(&app, "sapid-carousel-slide").is_empty());
    assert!(!app.tree().enabled(next));
}
#[test]
fn invalid_inputs_reject_before_factories_and_unstyled_preserves_geometry_policy() {
    for widget in [
        carousel().count(1_000_001usize),
        carousel().count(2usize).default_current(2),
        carousel().count(2usize).overscan(4),
    ] {
        let calls = Rc::new(Cell::new(0));
        let called = calls.clone();
        assert!(
            App::memory()
                .mount(widget.render_slide(move |_| {
                    called.set(called.get() + 1);
                    glory::ui::text().content("unexpected")
                }))
                .is_err()
        );
        assert_eq!(calls.get(), 0);
    }
    let app = App::memory();
    app.mount(
        carousel()
            .unstyled()
            .count(1usize)
            .style(glory::style::Style::new().width(glory::style::Length::Px(270.)))
            .render_slide(|_| glory::ui::text().content("Only")),
    )
    .unwrap();
    let id = nodes(&app, "sapid-carousel")[0];
    let tree = app.tree();
    let view = tree.node(id).unwrap().descriptor.view();
    assert!(view.style.background.is_none());
    assert_eq!(view.style.width, Some(glory::style::Length::Px(270.)));
    assert_eq!(
        view.style.max_width,
        Some(glory::style::Limit::Percent(100.))
    );
}

#[test]
fn autoplay_is_one_revocable_owner_delay_with_hover_focus_and_explicit_pause() {
    use std::{
        cell::RefCell,
        future::Future,
        pin::Pin,
        task::{Context, Poll, Waker},
    };
    struct Clock {
        ready: Rc<Cell<bool>>,
        wake: Rc<RefCell<Option<Waker>>>,
    }
    impl Future for Clock {
        type Output = ();
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            if self.ready.replace(false) {
                Poll::Ready(())
            } else {
                *self.wake.borrow_mut() = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
    let executor = Rc::new(glory::core::executor::Executor::new(8));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let owner = app.scope().child().unwrap();
    let ready = Rc::new(Cell::new(false));
    let wake = Rc::new(RefCell::new(None::<Waker>));
    let timer_ready = ready.clone();
    let timer_wake = wake.clone();
    carousel()
        .count(4usize)
        .looping(true)
        .autoplay(move || {
            Box::pin(Clock {
                ready: timer_ready.clone(),
                wake: timer_wake.clone(),
            })
        })
        .render_slide(|index| glory::ui::text().content(index.to_string()))
        .show_in(&owner)
        .unwrap();
    app.flush().unwrap();
    assert_eq!(executor.len(), 1);
    assert!(wake.borrow().is_none());
    executor.poll_one();
    let stale = wake.borrow().clone().unwrap();
    let root = nodes(&app, "sapid-carousel")[0];
    app.dispatch(
        root,
        Event::Pointer {
            from: root,
            related: None,
            phase: glory::render::PointerPhase::Enter,
        },
    )
    .unwrap();
    assert!(executor.is_empty());
    app.dispatch(
        root,
        Event::Pointer {
            from: root,
            related: None,
            phase: glory::render::PointerPhase::Move,
        },
    )
    .unwrap();
    assert!(
        executor.is_empty(),
        "movement inside an already hovered boundary must keep autoplay paused"
    );
    ready.set(true);
    stale.wake();
    assert!(!executor.poll_one());
    assert_eq!(position(&app), "1 / 4");
    app.dispatch(
        root,
        Event::Pointer {
            from: root,
            related: None,
            phase: glory::render::PointerPhase::Leave,
        },
    )
    .unwrap();
    executor.poll_one();
    app.flush().unwrap();
    assert_eq!(position(&app), "2 / 4");
    assert_eq!(executor.len(), 1);
    app.dispatch(root, Event::FocusChanged(true)).unwrap();
    assert!(executor.is_empty());
    app.dispatch(root, Event::FocusChanged(false)).unwrap();
    assert_eq!(executor.len(), 1);
    let pause = nodes(&app, "sapid-carousel-pause")[0];
    app.dispatch(pause, Event::Activate).unwrap();
    assert!(executor.is_empty());
    app.dispatch(pause, Event::Activate).unwrap();
    assert_eq!(executor.len(), 1);
    owner.dispose();
    assert!(executor.is_empty());
    assert_eq!(app.stats().nodes, 0);
}

#[test]
fn synchronous_ssr_never_constructs_autoplay_future_and_keeps_first_shape() {
    let calls = Rc::new(Cell::new(0));
    let factory = calls.clone();
    let snapshot = glory::ssr::snapshot(
        carousel()
            .count(3usize)
            .autoplay(move || {
                factory.set(factory.get() + 1);
                Box::pin(async {})
            })
            .render_slide(|index| glory::ui::text().content(index.to_string())),
    )
    .unwrap();
    assert_eq!(calls.get(), 0);
    assert!(snapshot.html.contains("1 / 3") && snapshot.html.contains("Pause autoplay"));
}
