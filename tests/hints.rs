use glory::render::{
    DismissReason, Event, EventKind, LayerFocus, NodeId, PointerPhase, Props, Role,
};
use glory::ui::{App, Widget};
use sapid::{ContentPolicy, hover_card, tooltip};
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

fn nodes(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut result = Vec::new();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        result.push(id);
        pending.extend(tree.node(id).unwrap().children.iter().copied());
    }
    result
}
fn panel(app: &App, role: Role) -> NodeId {
    nodes(app)
        .into_iter()
        .find(|id| app.tree().node(*id).unwrap().descriptor.view().role == Some(role))
        .unwrap()
}
fn control(app: &App, label: &str) -> NodeId {
    nodes(app)
        .into_iter()
        .find(
            |id| match app.tree().node(*id).unwrap().descriptor.view().props {
                Props::Button(p) => p.label == label,
                Props::Link(p) => p.label == label,
                _ => false,
            },
        )
        .unwrap()
}
fn hidden(app: &App, node: NodeId) -> bool {
    app.tree().node(node).unwrap().descriptor.view().hidden()
}
fn pointer(app: &App, from: NodeId, entered: bool) {
    app.dispatch(
        from,
        Event::Pointer {
            from,
            related: None,
            phase: if entered {
                PointerPhase::Enter
            } else {
                PointerPhase::Leave
            },
        },
    )
    .unwrap();
}

#[test]
fn tooltip_is_a_passive_description_and_keeps_hover_and_focus_independent() {
    let app = App::memory();
    app.mount(
        tooltip("help")
            .label("Help")
            .text("Saving preserves your draft"),
    )
    .unwrap();
    let target = control(&app, "Help");
    let tip = panel(&app, Role::Tooltip);
    let tree = app.tree();
    let metadata = tree
        .node(target)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap();
    assert!(metadata.popup.is_none() && metadata.expanded.is_none());
    assert_eq!(
        metadata.described_by.as_ref(),
        tree.node(tip).unwrap().descriptor.view().semantic_id()
    );
    let layer = tree
        .node(tip)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .layer
        .as_ref()
        .unwrap();
    assert_eq!(layer.focus, LayerFocus::Preserve);
    assert!(!layer.modal && layer.initial_focus.is_none() && layer.return_focus.is_none());
    drop(tree);
    assert!(hidden(&app, tip));
    app.dispatch(target, Event::FocusChanged(true)).unwrap();
    assert!(!hidden(&app, tip));
    pointer(&app, target, true);
    app.dispatch(target, Event::FocusChanged(false)).unwrap();
    assert!(!hidden(&app, tip));
    pointer(&app, target, false);
    assert!(hidden(&app, tip));
    app.dispatch(target, Event::FocusChanged(true)).unwrap();
    app.dispatch(tip, Event::Dismiss(DismissReason::Escape))
        .unwrap();
    assert!(hidden(&app, tip));
    app.dispatch(target, Event::FocusChanged(false)).unwrap();
    app.dispatch(target, Event::FocusChanged(true)).unwrap();
    assert!(!hidden(&app, tip));
}

#[test]
fn controlled_rejection_and_dismiss_callbacks_keep_one_truth_and_do_not_duplicate_proposals() {
    let app = App::memory();
    let open = app.scope().cage(false);
    let order = Rc::new(RefCell::new(Vec::new()));
    let changes = order.clone();
    let dismiss = order.clone();
    let observed = order.clone();
    app.mount(
        tooltip("controlled-tip")
            .label("Info")
            .text("A description")
            .open(open)
            .on_open_change(move |value| changes.borrow_mut().push(format!("open:{value}")))
            .on_dismiss(move |_| dismiss.borrow_mut().push("dismiss".into()))
            .on(EventKind::Dismiss, move |_| {
                observed.borrow_mut().push("observer".into())
            }),
    )
    .unwrap();
    let target = control(&app, "Info");
    let tip = panel(&app, Role::Tooltip);
    pointer(&app, target, true);
    assert!(hidden(&app, tip));
    assert_eq!(*order.borrow(), ["open:true"]);
    open.revise(|value| *value = true);
    app.flush().unwrap();
    assert!(!hidden(&app, tip));
    open.revise(|value| *value = false);
    app.flush().unwrap();
    assert!(hidden(&app, tip));
    assert_eq!(
        *order.borrow(),
        ["open:true"],
        "program source writes do not propose a new hover intent"
    );
    open.revise(|value| *value = true);
    app.flush().unwrap();
    assert!(!hidden(&app, tip));
    app.dispatch(tip, Event::Dismiss(DismissReason::Escape))
        .unwrap();
    assert!(!hidden(&app, tip));
    assert_eq!(
        *order.borrow(),
        ["open:true", "open:false", "dismiss", "observer"]
    );
}

#[test]
fn actual_move_recovers_a_missed_enter_but_does_not_rearm_escape_without_leaving() {
    let app = App::memory();
    app.mount(tooltip("missed-enter").label("Help").text("Description"))
        .unwrap();
    let target = control(&app, "Help");
    let tip = panel(&app, Role::Tooltip);
    app.dispatch(
        target,
        Event::Pointer {
            from: target,
            related: None,
            phase: PointerPhase::Move,
        },
    )
    .unwrap();
    assert!(!hidden(&app, tip));
    app.dispatch(tip, Event::Dismiss(DismissReason::Escape))
        .unwrap();
    assert!(hidden(&app, tip));
    app.dispatch(
        target,
        Event::Pointer {
            from: target,
            related: None,
            phase: PointerPhase::Move,
        },
    )
    .unwrap();
    assert!(hidden(&app, tip));
    pointer(&app, target, false);
    pointer(&app, target, true);
    assert!(!hidden(&app, tip));
}

#[test]
fn hover_card_uses_preserved_dialog_focus_and_live_descendant_boundaries() {
    let app = App::memory();
    app.mount(
        hover_card("profile")
            .label("Profile")
            .name("Profile details")
            .content(|| {
                glory::ui::view().fill((
                    glory::ui::link().label("Profile link").target("/profile"),
                    glory::ui::button().label("Follow"),
                ))
            })
            .content_policy(ContentPolicy::KeepMounted),
    )
    .unwrap();
    let target = control(&app, "Profile");
    let card = panel(&app, Role::Dialog);
    let child = control(&app, "Profile link");
    assert_eq!(
        app.tree()
            .node(card)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Profile details")
    );
    assert!(hidden(&app, card));
    pointer(&app, target, true);
    assert!(!hidden(&app, card));
    pointer(&app, child, true);
    pointer(&app, target, false);
    assert!(!hidden(&app, card));
    app.dispatch(child, Event::FocusChanged(true)).unwrap();
    pointer(&app, child, false);
    assert!(!hidden(&app, card));
    app.dispatch(child, Event::FocusChanged(false)).unwrap();
    assert!(hidden(&app, card));
}

#[test]
fn typed_target_adapters_preserve_controls_reject_conflicts_and_do_not_retain_owners() {
    let app = App::memory();
    let owner = app.scope().child().unwrap();
    let held = Rc::new(RefCell::new(None));
    let save = held.clone();
    let captured = Rc::new(Cell::new(0));
    let callback = captured.clone();
    tooltip("link-tip")
        .text("Link detail")
        .target(move |target| {
            *save.borrow_mut() = Some(target.clone());
            target.link(glory::ui::link().label("Docs").target("/docs"))
        })
        .on_open_change(move |_| callback.set(callback.get() + 1))
        .show_in(&owner)
        .unwrap();
    app.flush().unwrap();
    let target = control(&app, "Docs");
    assert!(matches!(
        app.tree().node(target).unwrap().descriptor.view().props,
        Props::Link(_)
    ));
    pointer(&app, target, true);
    assert_eq!(captured.get(), 1);
    owner.dispose();
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(Rc::strong_count(&captured), 1);
    assert!(!held.borrow().as_ref().unwrap().node_ref().is_alive());
    assert!(
        app.mount(
            tooltip("conflict")
                .text("description")
                .target(|target| target.button_with(
                    glory::ui::button(),
                    glory::render::Semantics {
                        id: Some(glory::render::SemanticId::new("other-id").unwrap()),
                        ..Default::default()
                    }
                ))
        )
        .is_err()
    );
    assert_eq!(app.stats().nodes, 0);
}

#[test]
fn tooltip_text_budget_and_ssr_description_have_stable_shapes() {
    let app = App::memory();
    assert!(app.mount(tooltip("empty")).is_err());
    assert!(
        app.mount(tooltip("oversize").text("x".repeat(sapid::MAX_HINT_TEXT_BYTES + 1)))
            .is_err()
    );
    assert_eq!(app.stats().nodes, 0);
    let snapshot =
        glory::ssr::snapshot(tooltip("ssr-tip").label("Save").text("Save the draft")).unwrap();
    assert!(
        snapshot.html.contains("role=\"tooltip\"")
            && snapshot.html.contains("aria-describedby=")
    );
    assert!(
        !snapshot.html.contains("aria-haspopup")
            && !snapshot.html.contains("aria-expanded")
            && !snapshot.html.contains("aria-modal")
    );
}

struct Delay {
    ready: Rc<Cell<bool>>,
    waker: Rc<RefCell<Option<Waker>>>,
    drops: Rc<Cell<usize>>,
}
impl Future for Delay {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.ready.get() {
            Poll::Ready(())
        } else {
            *self.waker.borrow_mut() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
impl Drop for Delay {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn open_and_close_delay_share_one_owner_task_and_stale_wakes_cannot_reopen() {
    let executor = Rc::new(glory::core::executor::Executor::new(8));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let owner = app.scope().child().unwrap();
    let ready = Rc::new(Cell::new(false));
    let waker = Rc::new(RefCell::new(None::<Waker>));
    let drops = Rc::new(Cell::new(0));
    let make =
        |ready: Rc<Cell<bool>>, waker: Rc<RefCell<Option<Waker>>>, drops: Rc<Cell<usize>>| {
            move || Delay {
                ready: ready.clone(),
                waker: waker.clone(),
                drops: drops.clone(),
            }
        };
    tooltip("delay-tip")
        .label("Delayed help")
        .text("Description")
        .open_delay_with(make(ready.clone(), waker.clone(), drops.clone()))
        .close_delay_with(make(ready.clone(), waker.clone(), drops.clone()))
        .show_in(&owner)
        .unwrap();
    app.flush().unwrap();
    let target = control(&app, "Delayed help");
    let tip = panel(&app, Role::Tooltip);
    pointer(&app, target, true);
    assert_eq!(executor.len(), 1);
    assert!(waker.borrow().is_none(), "spawn never polls synchronously");
    executor.poll_one();
    let late = waker.borrow().clone().unwrap();
    pointer(&app, target, false);
    assert!(executor.is_empty());
    assert_eq!(drops.get(), 1);
    ready.set(true);
    late.wake();
    assert!(!executor.poll_one());
    app.flush().unwrap();
    assert!(hidden(&app, tip));
    pointer(&app, target, true);
    executor.poll_one();
    app.flush().unwrap();
    assert!(!hidden(&app, tip));
    ready.set(false);
    pointer(&app, target, false);
    assert_eq!(executor.len(), 1);
    executor.poll_one();
    pointer(&app, tip, true);
    assert!(executor.is_empty());
    assert!(!hidden(&app, tip));
    pointer(&app, tip, false);
    executor.poll_one();
    assert_eq!(executor.len(), 1);
    let late = waker.borrow().clone().unwrap();
    owner.dispose();
    assert!(executor.is_empty());
    assert_eq!(app.stats().nodes, 0);
    ready.set(true);
    late.wake();
    assert!(!executor.poll_one());
}
