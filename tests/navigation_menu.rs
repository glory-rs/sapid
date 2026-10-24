use glory::prelude::*;
use glory::render::{EventKind, NavigationPanelCommand, NodeId, Props, Role};
use sapid::{navigation_menu, navigation_menu_link, navigation_menu_panel};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn nodes(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut output = Vec::new();
    while let Some(id) = pending.pop() {
        pending.extend(tree.node(id).unwrap().children.iter().copied());
        output.push(id)
    }
    output
}
fn control(app: &App, label: &str) -> NodeId {
    nodes(app)
        .into_iter()
        .find(
            |id| match &app.tree().node(*id).unwrap().descriptor.view().props {
                Props::Button(p) => p.label == label,
                Props::Link(p) => p.label == label,
                _ => false,
            },
        )
        .unwrap_or_else(|| panic!("missing {label}"))
}
fn expanded(app: &App, id: NodeId) -> bool {
    app.tree()
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .expanded
        .unwrap()
}
fn fixture() -> sapid::NavigationMenu {
    navigation_menu("main").content(|ctx| {
        (
            navigation_menu_link("home")
                .context(ctx.clone())
                .label("Home")
                .href("/"),
            navigation_menu_panel("products")
                .context(ctx)
                .label("Products")
                .content(|ctx| {
                    (
                        navigation_menu_link("editor")
                            .context(ctx.clone())
                            .label("Editor")
                            .href("/editor"),
                        navigation_menu_panel("more")
                            .context(ctx)
                            .label("More")
                            .content(|ctx| {
                                navigation_menu_link("docs")
                                    .context(ctx)
                                    .label("Docs")
                                    .href("/docs")
                            }),
                    )
                }),
        )
    })
}
#[test]
fn actual_navigation_links_and_disclosures_keep_mounted_shells_and_content() {
    let app = App::memory();
    app.mount(fixture()).unwrap();
    let before = nodes(&app);
    let products = control(&app, "Products");
    assert!(!expanded(&app, products));
    app.dispatch(products, Event::Activate).unwrap();
    assert!(expanded(&app, products));
    let more = control(&app, "More");
    app.dispatch(more, Event::Activate).unwrap();
    assert!(expanded(&app, more));
    // The headless Memory renderer accepts state but explicitly has no physical
    // focus capability. Real keyboard focus is covered by the native fixture.
    assert!(
        app.dispatch(
            more,
            Event::NavigationPanel {
                from: more,
                command: NavigationPanelCommand::Close,
            },
        )
        .is_err()
    );
    assert!(!expanded(&app, products));
    assert_eq!(before, nodes(&app));
    assert!(!nodes(&app).into_iter().any(|id| matches!(
        app.tree().node(id).unwrap().descriptor.view().role,
        Some(Role::Menu)
    )));
}
#[test]
fn controlled_requests_are_ordered_and_rejection_does_not_open_or_mutate_source() {
    let app = App::memory();
    let path = app.scope().cage(Vec::<String>::new());
    let calls = Rc::new(RefCell::new(Vec::new()));
    let changed = calls.clone();
    let observed = calls.clone();
    app.mount(
        fixture()
            .open_path(path)
            .on_open_path_change(move |path| {
                changed.borrow_mut().push(format!("typed:{path:?}"))
            })
            .on(EventKind::Activate, move |_| {
                observed.borrow_mut().push("generic".into())
            }),
    )
    .unwrap();
    let products = control(&app, "Products");
    app.dispatch(products, Event::Activate).unwrap();
    assert!(path.get().is_empty());
    assert!(!expanded(&app, products));
    assert_eq!(&*calls.borrow(), &["typed:[\"products\"]"]);
    path.revise(|path| path.push("products".into()));
    app.flush().unwrap();
    assert!(expanded(&app, products));
    assert_eq!(calls.borrow().len(), 1);
    path.revise(|path| *path = vec!["missing".into()]);
    assert!(app.flush().is_err());
    assert!(expanded(&app, products));
    app.dispatch(products, Event::Activate).unwrap();
    assert_eq!(calls.borrow().last().unwrap(), "typed:[]");
}
#[test]
fn bounded_defaults_stop_consuming_lazy_input_and_invalid_initial_path_is_rejected() {
    let app = App::memory();
    let count = Rc::new(Cell::new(0));
    let visited = count.clone();
    let menu = fixture().default_open_path((0..1_000_000).map(move |_| {
        visited.set(visited.get() + 1);
        "products"
    }));
    assert_eq!(count.get(), 9);
    assert!(app.mount(menu).is_err());
    assert!(app.tree().roots().is_empty());
    assert!(app.mount(fixture().default_open_path(["missing"])).is_err());
    assert!(app.tree().roots().is_empty());
}
#[test]
fn semantic_ssr_has_navigation_and_group_layers_without_menu_or_selection_roles() {
    let html = glory::ssr::render(fixture().default_open_path(["products"])).unwrap();
    assert!(html.contains("role=\"navigation\""));
    assert!(html.contains("data-gly-navigation-panel=\"content\""));
    assert!(html.contains("aria-expanded=\"true\""));
    assert!(!html.contains("role=\"menu\""));
    assert!(!html.contains("aria-selected"));
    assert!(!html.contains("aria-haspopup"));
}

#[test]
fn link_requests_read_accepted_descriptor_and_precede_generic_observers() {
    let app = App::memory();
    let href = app.scope().cage("/accepted".to_owned());
    let calls = Rc::new(RefCell::new(Vec::new()));
    let typed = calls.clone();
    let generic = calls.clone();
    app.mount(navigation_menu("accepted").content(move |ctx| {
        navigation_menu_link("link")
            .context(ctx)
            .label("Link")
            .href(href)
            .on_navigate({
                let typed = typed.clone();
                move |request| typed.borrow_mut().push(request.href)
            })
            .on(EventKind::Activate, {
                let generic = generic.clone();
                move |_| generic.borrow_mut().push("generic".into())
            })
    }))
    .unwrap();
    let link = control(&app, "Link");
    href.revise(|value| *value = "/pending".into());
    app.dispatch(link, Event::Activate).unwrap();
    assert_eq!(&*calls.borrow(), &["/accepted", "generic"]);
}

#[test]
fn owner_retirement_drops_callbacks_and_clears_all_external_references() {
    struct DropProbe(Rc<Cell<usize>>);
    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1)
        }
    }
    let app = App::memory();
    let drops = Rc::new(Cell::new(0));
    let capture = DropProbe(drops.clone());
    let root = app.scope().cage(None);
    let trigger = app.scope().cage(None);
    let panel = app.scope().cage(None);
    app.mount(
        navigation_menu("owned")
            .node_ref(root)
            .on_open_path_change(move |_| {
                let _ = &capture;
            })
            .content(move |ctx| {
                navigation_menu_panel("panel")
                    .context(ctx)
                    .label("Panel")
                    .node_ref(trigger)
                    .panel_ref(panel)
                    .content(|ctx| {
                        navigation_menu_link("leaf")
                            .context(ctx)
                            .label("Leaf")
                            .href("/leaf")
                    })
            }),
    )
    .unwrap();
    app.dispatch(trigger.get().unwrap(), Event::Activate)
        .unwrap();
    app.clear().unwrap();
    assert_eq!(drops.get(), 1);
    assert_eq!(root.get(), None);
    assert_eq!(trigger.get(), None);
    assert_eq!(panel.get(), None);
    assert_eq!(app.stats().queries, 0);
}

#[test]
fn delayed_hover_cancels_on_leave_and_owner_retirement_rejects_late_wakes() {
    use glory::core::executor::Executor;
    use glory::render::PointerPhase;
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll, Waker},
    };
    struct Gate {
        ready: Rc<Cell<bool>>,
        waker: Rc<RefCell<Option<Waker>>>,
        drops: Rc<Cell<usize>>,
    }
    impl Future for Gate {
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
    impl Drop for Gate {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1)
        }
    }
    let executor = Rc::new(Executor::new(8));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let ready = Rc::new(Cell::new(false));
    let waker = Rc::new(RefCell::new(None::<Waker>));
    let drops = Rc::new(Cell::new(0));
    let r = ready.clone();
    let w = waker.clone();
    let d = drops.clone();
    app.mount(fixture().hover_delay_with(move || Gate {
        ready: r.clone(),
        waker: w.clone(),
        drops: d.clone(),
    }))
    .unwrap();
    let products = control(&app, "Products");
    let pointer = |phase| {
        app.dispatch(
            products,
            Event::Pointer {
                from: products,
                related: None,
                phase,
            },
        )
        .unwrap()
    };
    pointer(PointerPhase::Enter);
    pointer(PointerPhase::Move);
    assert_eq!(executor.len(), 1);
    executor.poll_one();
    let late = waker.borrow().clone().unwrap();
    pointer(PointerPhase::Move);
    assert_eq!(executor.len(), 1);
    pointer(PointerPhase::Leave);
    assert!(executor.is_empty());
    assert_eq!(drops.get(), 1);
    ready.set(true);
    late.wake();
    assert!(!executor.poll_one());
    app.flush().unwrap();
    assert!(!expanded(&app, products));
    ready.set(false);
    pointer(PointerPhase::Enter);
    pointer(PointerPhase::Move);
    executor.poll_one();
    let late = waker.borrow().clone().unwrap();
    app.clear().unwrap();
    assert!(executor.is_empty());
    late.wake();
    assert!(!executor.poll_one());
    assert_eq!(drops.get(), 2);
}

#[test]
fn a_retired_hover_target_cannot_open_a_same_key_replacement() {
    use glory::{core::executor::Executor, render::PointerPhase};
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll, Waker},
    };
    struct Gate(Rc<RefCell<Option<Waker>>>);
    impl Future for Gate {
        type Output = ();
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            *self.0.borrow_mut() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
    let executor = Rc::new(Executor::new(8));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let shown = app.scope().cage(true);
    let waker = Rc::new(RefCell::new(None::<Waker>));
    let saved = waker.clone();
    app.mount(
        navigation_menu("replacement")
            .hover_delay_with(move || Gate(saved.clone()))
            .content(move |ctx| {
                glory::ui::when(shown, move || {
                    navigation_menu_panel("products")
                        .context(ctx.clone())
                        .label("Products")
                        .content(|ctx| {
                            navigation_menu_link("editor")
                                .context(ctx)
                                .label("Editor")
                                .href("/editor")
                        })
                })
            }),
    )
    .unwrap();
    let first = control(&app, "Products");
    for phase in [PointerPhase::Enter, PointerPhase::Move] {
        app.dispatch(
            first,
            Event::Pointer {
                from: first,
                related: None,
                phase,
            },
        )
        .unwrap();
    }
    executor.poll_one();
    let late = waker.borrow().clone().unwrap();
    assert_eq!(executor.len(), 1);
    shown.revise(|value| *value = false);
    app.flush().unwrap();
    assert!(executor.is_empty());
    shown.revise(|value| *value = true);
    app.flush().unwrap();
    let second = control(&app, "Products");
    assert_ne!(first, second);
    late.wake();
    assert!(!executor.poll_one());
    assert!(!expanded(&app, second));
}

#[test]
fn focus_transfers_inside_navigation_do_not_propose_an_intermediate_close() {
    let app = App::memory();
    let path = app.scope().cage(vec!["products".to_owned()]);
    let closes = Rc::new(Cell::new(0));
    let out = closes.clone();
    app.mount(fixture().open_path(path).on_open_path_change(move |path| {
        assert!(path.is_empty());
        out.set(out.get() + 1)
    }))
    .unwrap();
    let products = control(&app, "Products");
    let editor = control(&app, "Editor");
    app.dispatch(products, Event::FocusChanged(true)).unwrap();
    app.scope()
        .batch(|| {
            app.dispatch(products, Event::FocusChanged(false)).unwrap();
            app.dispatch(editor, Event::FocusChanged(true)).unwrap();
        })
        .unwrap();
    assert_eq!(
        closes.get(),
        0,
        "a loss followed by an internal gain in one tick is not leaving navigation"
    );
    app.dispatch(editor, Event::FocusChanged(false)).unwrap();
    assert_eq!(closes.get(), 1);
}

#[test]
fn retiring_the_focused_member_closes_without_a_terminal_event_or_stale_identity() {
    let app = App::memory();
    let shown = app.scope().cage(true);
    let path = app.scope().cage(vec!["products".to_owned()]);
    let reference = app.scope().cage(None);
    app.mount(
        navigation_menu("focused-retire")
            .node_ref(reference)
            .open_path(path)
            .on_open_path_change(move |next| path.revise(|value| *value = next))
            .content(move |ctx| {
                navigation_menu_panel("products")
                    .context(ctx)
                    .label("Products")
                    .content(move |ctx| {
                        glory::ui::when(shown, move || {
                            navigation_menu_link("editor")
                                .context(ctx.clone())
                                .label("Editor")
                                .href("/editor")
                        })
                    })
            }),
    )
    .unwrap();
    let first = control(&app, "Editor");
    app.dispatch(first, Event::FocusChanged(true)).unwrap();
    shown.revise(|value| *value = false);
    app.flush().unwrap();
    assert!(path.get().is_empty());
    assert!(reference.get().is_some());
    assert!(!expanded(&app, control(&app, "Products")));
    shown.revise(|value| *value = true);
    app.flush().unwrap();
    let second = control(&app, "Editor");
    assert_ne!(first, second);
    let _ = app.dispatch(first, Event::FocusChanged(true));
    assert!(path.get().is_empty());
    assert!(!expanded(&app, control(&app, "Products")));
}

#[test]
fn closing_a_focused_nested_panel_preserves_the_parent_path() {
    let app = App::memory();
    let changes = Rc::new(RefCell::new(Vec::new()));
    let out = changes.clone();
    app.mount(
        fixture()
            .default_open_path(["products", "more"])
            .on_open_path_change(move |path| out.borrow_mut().push(path)),
    )
    .unwrap();
    let docs = control(&app, "Docs");
    let more = control(&app, "More");
    app.dispatch(docs, Event::FocusChanged(true)).unwrap();
    let panel_id = app
        .tree()
        .node(more)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .controls
        .clone();
    let panel = nodes(&app)
        .into_iter()
        .find(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .is_some_and(|semantics| semantics.id == panel_id)
        })
        .unwrap();
    let _ = app.dispatch(panel, Event::Dismiss(glory::render::DismissReason::Escape));
    assert!(expanded(&app, control(&app, "Products")));
    assert!(!expanded(&app, more));
    assert_eq!(&*changes.borrow(), &[vec!["products".to_owned()]]);
}

#[test]
fn tracked_custom_control_retirement_closes_and_replacement_rejects_old_origin() {
    let app = App::memory();
    let shown = app.scope().cage(true);
    let input = app.scope().cage(None);
    let path = app.scope().cage(vec!["products".to_owned()]);
    app.mount(
        navigation_menu("tracked-custom")
            .open_path(path)
            .on_open_path_change(move |next| path.revise(|value| *value = next))
            .content(move |ctx| {
                navigation_menu_panel("products")
                    .context(ctx)
                    .label("Products")
                    .content(move |ctx| {
                        glory::ui::when(shown, move || {
                            ctx.track_focusable(
                                input,
                                glory::ui::input().node_ref(input).value("Draft"),
                            )
                        })
                    })
            }),
    )
    .unwrap();
    let first = input.get().unwrap();
    app.dispatch(first, Event::FocusChanged(true)).unwrap();
    shown.revise(|value| *value = false);
    app.flush().unwrap();
    assert!(path.get().is_empty());
    assert_eq!(input.get(), None);
    shown.revise(|value| *value = true);
    app.flush().unwrap();
    let second = input.get().unwrap();
    assert_ne!(first, second);
    let _ = app.dispatch(first, Event::FocusChanged(true));
    assert!(path.get().is_empty());
    assert_eq!(input.get(), Some(second));
}

#[test]
fn disabled_disclosure_updates_the_intrinsic_button_and_blocks_activation() {
    let app = App::memory();
    let enabled = app.scope().cage(true);
    let requested = Rc::new(Cell::new(0));
    let out = requested.clone();
    app.mount(
        navigation_menu("disabled")
            .on_open_path_change(move |_| out.set(out.get() + 1))
            .content(move |ctx| {
                navigation_menu_panel("panel")
                    .context(ctx)
                    .label("Panel")
                    .enabled(enabled)
            }),
    )
    .unwrap();
    let button = control(&app, "Panel");
    enabled.revise(|enabled| *enabled = false);
    app.flush().unwrap();
    assert!(
        matches!(app.tree().node(button).unwrap().descriptor.view().props,Props::Button(props) if !props.enabled)
    );
    assert!(!app.dispatch(button, Event::Activate).unwrap());
    assert_eq!(requested.get(), 0);
    enabled.revise(|enabled| *enabled = true);
    app.flush().unwrap();
    assert!(app.dispatch(button, Event::Activate).unwrap());
    assert_eq!(requested.get(), 1);
    assert!(expanded(&app, button));
}
