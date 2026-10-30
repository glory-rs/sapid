use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

use glory::core::executor::Executor;
use glory::render::{
    Event, FocusMode, MenuCommand, MenuItemKind, Navigation, NodeId, PointerPhase, Props, Role,
};
use glory::ui::{App, Widget};
use sapid::{
    dropdown_menu, menu, menu_checkbox, menu_item, menu_radio_group, menu_radio_item, menu_submenu,
};

fn nodes(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().iter().rev().copied().collect::<Vec<_>>();
    let mut nodes = Vec::new();
    while let Some(id) = pending.pop() {
        nodes.push(id);
        pending.extend(tree.node(id).unwrap().children.iter().rev().copied());
    }
    nodes
}
fn control(app: &App, label: &str) -> NodeId {
    nodes(app)
        .into_iter()
        .find(
            |id| match &app.tree().node(*id).unwrap().descriptor.view().props {
                Props::Button(button) => button.label == label,
                Props::Check(check) => check.label == label,
                _ => false,
            },
        )
        .unwrap()
}
fn panels(app: &App) -> Vec<NodeId> {
    nodes(app)
        .into_iter()
        .filter(|id| app.tree().node(*id).unwrap().descriptor.view().role == Some(Role::Menu))
        .collect()
}
fn hidden(app: &App, id: NodeId) -> bool {
    app.tree()
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .hidden
}
fn command(app: &App, from: NodeId, command: MenuCommand) {
    app.dispatch(from, Event::Menu { from, command }).unwrap();
}
fn pointer(app: &App, from: NodeId, phase: PointerPhase) {
    app.dispatch(
        from,
        Event::Pointer {
            from,
            related: None,
            phase,
        },
    )
    .unwrap();
    if phase == PointerPhase::Enter {
        app.dispatch(
            from,
            Event::Pointer {
                from,
                related: None,
                phase: PointerPhase::Move,
            },
        )
        .unwrap();
    }
}

#[test]
fn menu_has_intrinsic_items_shared_navigation_typeahead_hierarchy_and_close_policy() {
    let app = App::memory();
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    app.mount(
        dropdown_menu("file")
            .label("File")
            .item(
                menu_item("new")
                    .label("New")
                    .on_activate(move || seen.set(seen.get() + 1)),
            )
            .item(menu_item("disabled").label("Disabled").enabled(false))
            .item(menu_checkbox("ruler").label("Ruler"))
            .radio_group(menu_radio_group("layout").default_value("wide").items([
                menu_radio_item("wide", "Wide"),
                menu_radio_item("compact", "Compact"),
            ]))
            .separator("break")
            .submenu(
                menu_submenu("export")
                    .label("Export")
                    .items([menu_item("pdf").label("PDF"), menu_item("png").label("PNG")]),
            ),
    )
    .unwrap();
    let panels = panels(&app);
    assert_eq!(panels.len(), 2);
    assert!(panels.iter().all(|id| hidden(&app, *id)));
    let trigger = control(&app, "File");
    command(&app, trigger, MenuCommand::Last);
    let export = control(&app, "Export");
    assert_eq!(
        app.tree().node(export).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    assert!(!hidden(&app, panels[0]));
    command(&app, export, MenuCommand::Open);
    assert!(!hidden(&app, panels[1]));
    let pdf = control(&app, "PDF");
    command(&app, pdf, MenuCommand::Close);
    assert!(hidden(&app, panels[1]));
    assert!(!hidden(&app, panels[0]));
    command(&app, trigger, MenuCommand::First);
    let new = control(&app, "New");
    app.dispatch(
        new,
        Event::Navigate {
            from: new,
            command: Navigation::Next,
        },
    )
    .unwrap();
    let ruler = control(&app, "Ruler");
    assert_eq!(
        app.tree().node(ruler).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    app.dispatch(ruler, Event::CheckedChanged(true)).unwrap();
    assert!(!hidden(&app, panels[0]));
    assert!(
        matches!(&app.tree().node(ruler).unwrap().descriptor.view().props.clone(),Props::Check(check) if check.checked)
    );
    app.dispatch(
        ruler,
        Event::Typeahead {
            from: ruler,
            text: "e".into(),
            at_millis: 1,
        },
    )
    .unwrap();
    assert_eq!(
        app.tree().node(export).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    assert_eq!(
        app.tree()
            .node(ruler)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .menu_item
            .as_ref()
            .unwrap()
            .kind,
        MenuItemKind::Checkbox
    );
    let compact = control(&app, "Compact");
    app.dispatch(compact, Event::CheckedChanged(true)).unwrap();
    assert!(
        matches!(&app.tree().node(compact).unwrap().descriptor.view().props.clone(),Props::Check(check) if check.checked)
    );
    let wide = control(&app, "Wide");
    assert!(
        matches!(&app.tree().node(wide).unwrap().descriptor.view().props.clone(),Props::Check(check) if !check.checked)
    );
    app.dispatch(new, Event::Activate).unwrap();
    assert_eq!(calls.get(), 1);
    assert!(hidden(&app, panels[0]));
    app.dispatch(new, Event::Activate).unwrap();
    assert_eq!(calls.get(), 1, "hidden retained menu items do not activate");
}

#[test]
fn controlled_open_check_radio_and_submenu_rejections_preserve_their_sources() {
    let app = App::memory();
    let open = app.scope().cage(false);
    let checked = app.scope().cage(false);
    let value = app.scope().cage("one".to_string());
    let proposals = Rc::new(RefCell::new(Vec::new()));
    let seen = proposals.clone();
    app.mount(
        dropdown_menu("controlled-menu")
            .label("Controlled menu")
            .open(open)
            .on_open_change(move |value| seen.borrow_mut().push(value))
            .item(menu_checkbox("check").label("Check").checked(checked))
            .radio_group(
                menu_radio_group("radio")
                    .value(value)
                    .items([menu_radio_item("one", "One"), menu_radio_item("two", "Two")]),
            ),
    )
    .unwrap();
    let panel = panels(&app)[0];
    app.dispatch(control(&app, "Controlled menu"), Event::Activate)
        .unwrap();
    assert!(hidden(&app, panel));
    assert_eq!(*proposals.borrow(), [true]);
    assert!(!open.get());
    open.revise(|value| *value = true);
    app.flush().unwrap();
    app.dispatch(control(&app, "Check"), Event::CheckedChanged(true))
        .unwrap();
    app.dispatch(control(&app, "Two"), Event::CheckedChanged(true))
        .unwrap();
    assert!(!checked.get());
    assert_eq!(value.get(), "one");
    assert!(
        matches!(&app.tree().node(control(&app,"Check")).unwrap().descriptor.view().props.clone(),Props::Check(check) if !check.checked)
    );
    assert!(
        matches!(&app.tree().node(control(&app,"One")).unwrap().descriptor.view().props.clone(),Props::Check(check) if check.checked)
    );
    command(
        &app,
        control(&app, "One"),
        MenuCommand::Exit { backwards: false },
    );
    assert!(!hidden(&app, panel));
    assert_eq!(*proposals.borrow(), [true, false]);
}

#[test]
fn invalid_program_submenu_sets_retain_last_valid_visibility_and_owners_release_captures() {
    let app = App::memory();
    let one = app.scope().cage(true);
    let two = app.scope().cage(false);
    let capture = Rc::new(());
    let inside = capture.clone();
    let owner = app.scope().child().unwrap();
    dropdown_menu("siblings")
        .label("Siblings")
        .default_open(true)
        .submenu(menu_submenu("one").label("First submenu").open(one).item(
            menu_item("a").label("A").on_activate(move || {
                let _ = &inside;
            }),
        ))
        .submenu(
            menu_submenu("two")
                .label("Second submenu")
                .open(two)
                .item(menu_item("b").label("B")),
        )
        .show_in(&owner)
        .unwrap();
    app.flush().unwrap();
    let panels = panels(&app);
    assert!(!hidden(&app, panels[1]));
    assert!(hidden(&app, panels[2]));
    two.revise(|value| *value = true);
    assert!(app.flush().is_err());
    assert!(!hidden(&app, panels[1]));
    assert!(hidden(&app, panels[2]));
    two.revise(|value| *value = false);
    app.flush().unwrap();
    command(&app, control(&app, "Second submenu"), MenuCommand::Open);
    assert!(one.get());
    assert!(!two.get());
    assert!(!hidden(&app, panels[1]));
    assert!(hidden(&app, panels[2]));
    owner.dispose();
    assert_eq!(Rc::strong_count(&capture), 1);
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(app.stats().handlers, 0);
}

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
        self.drops.set(self.drops.get() + 1);
    }
}
#[test]
fn hover_delay_has_one_owner_task_cancels_on_leave_sibling_close_and_ignores_late_wakes() {
    let executor = Rc::new(Executor::new(8));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let ready = Rc::new(Cell::new(false));
    let waker = Rc::new(RefCell::new(None::<Waker>));
    let drops = Rc::new(Cell::new(0));
    let owner = app.scope().child().unwrap();
    let r = ready.clone();
    let w = waker.clone();
    let d = drops.clone();
    dropdown_menu("hover")
        .label("Hover")
        .default_open(true)
        .hover_delay_with(move || Gate {
            ready: r.clone(),
            waker: w.clone(),
            drops: d.clone(),
        })
        .item(menu_item("other").label("Other").close_on_select(false))
        .submenu(
            menu_submenu("child")
                .label("Child")
                .item(menu_item("leaf").label("Leaf")),
        )
        .show_in(&owner)
        .unwrap();
    app.flush().unwrap();
    let child = control(&app, "Child");
    let other = control(&app, "Other");
    let panel = panels(&app)[1];
    pointer(&app, child, PointerPhase::Enter);
    assert_eq!(executor.len(), 1);
    executor.poll_one();
    assert!(hidden(&app, panel));
    let late = waker.borrow().clone().unwrap();
    pointer(&app, child, PointerPhase::Leave);
    assert!(executor.is_empty());
    assert_eq!(drops.get(), 1);
    ready.set(true);
    late.wake();
    assert!(!executor.poll_one());
    app.flush().unwrap();
    assert!(hidden(&app, panel));
    ready.set(false);
    pointer(&app, child, PointerPhase::Enter);
    executor.poll_one();
    pointer(&app, other, PointerPhase::Enter);
    assert!(executor.is_empty());
    assert_eq!(drops.get(), 2);
    pointer(&app, child, PointerPhase::Enter);
    executor.poll_one();
    ready.set(true);
    waker.borrow().as_ref().unwrap().wake_by_ref();
    executor.poll_one();
    app.flush().unwrap();
    assert!(!hidden(&app, panel));
    ready.set(false);
    pointer(&app, child, PointerPhase::Leave);
    pointer(&app, child, PointerPhase::Enter);
    assert_eq!(executor.len(), 1);
    owner.dispose();
    assert!(executor.is_empty());
    assert_eq!(app.stats().nodes, 0);
}

#[test]
fn layout_boundary_entry_cannot_steal_keyboard_focus_without_actual_pointer_movement() {
    let app = App::memory();
    app.mount(
        dropdown_menu("stationary-pointer")
            .label("Actions")
            .item(menu_item("first").label("First"))
            .submenu(
                menu_submenu("share")
                    .label("Share")
                    .item(menu_item("child").label("Child")),
            )
            .item(menu_item("last").label("Last")),
    )
    .unwrap();
    let trigger = control(&app, "Actions");
    let share = control(&app, "Share");
    let last = control(&app, "Last");
    command(&app, trigger, MenuCommand::Last);
    app.dispatch(
        share,
        Event::Pointer {
            from: share,
            related: None,
            phase: PointerPhase::Enter,
        },
    )
    .unwrap();
    assert_eq!(
        app.tree().node(last).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    assert!(hidden(&app, panels(&app)[1]));
    app.dispatch(
        share,
        Event::Pointer {
            from: share,
            related: None,
            phase: PointerPhase::Move,
        },
    )
    .unwrap();
    assert_eq!(
        app.tree().node(share).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    assert!(!hidden(&app, panels(&app)[1]));
}

#[test]
fn menu_configuration_stops_iterators_at_the_budget_and_rejects_before_nodes() {
    let app = App::memory();
    let read = Rc::new(Cell::new(0));
    let seen = read.clone();
    let widget = dropdown_menu("bounded").items((0..1_000_000).map(move |index| {
        seen.set(seen.get() + 1);
        menu_item(format!("item-{index}")).label("Item")
    }));
    assert_eq!(read.get(), 257);
    assert!(app.mount(widget).is_err());
    assert_eq!(app.stats().nodes, 0);
    let mut widget = dropdown_menu("repeated");
    for index in 0..10_000 {
        widget = widget.item(menu_item(format!("item-{index}")));
    }
    assert!(app.mount(widget).is_err());
    assert_eq!(app.stats().nodes, 0);
    assert!(
        app.mount(dropdown_menu("duplicate").items([menu_item("same"), menu_item("same")]))
            .is_err()
    );
    assert!(
        app.mount(dropdown_menu("wrong").item(menu_item("action").checked(true)))
            .is_err()
    );
    assert!(app.mount(menu("no-anchor").item(menu_item("one"))).is_err());
}

#[test]
fn menu_ssr_has_real_menu_checks_radio_grouping_and_stable_hidden_shells() {
    let snapshot = glory::ssr::snapshot_with(|_| {
        dropdown_menu("ssr-menu")
            .label("Tools")
            .item(menu_checkbox("check").label("Grid"))
            .radio_group(
                menu_radio_group("modes")
                    .default_value("one")
                    .items([menu_radio_item("one", "One"), menu_radio_item("two", "Two")]),
            )
            .submenu(
                menu_submenu("export")
                    .label("Export")
                    .item(menu_item("pdf").label("PDF")),
            )
    })
    .unwrap();
    assert_eq!(snapshot.html.matches("role=\"menu\"").count(), 2);
    assert!(snapshot.html.contains("aria-haspopup=\"menu\""));
    assert!(snapshot.html.contains("role=\"menuitemcheckbox\""));
    assert_eq!(snapshot.html.matches("role=\"menuitemradio\"").count(), 2);
    assert_eq!(snapshot.html.matches("type=\"radio\"").count(), 2);
    assert!(!snapshot.html.contains("aria-modal"));
    assert_eq!(snapshot.html.matches("popover=\"manual\"").count(), 2);
    assert!(snapshot.html.contains("hidden=\"\""));
}

#[test]
fn keyboard_focus_and_reveal_follow_visibility_and_exit_retires_older_focus_intents() {
    use glory::render::{
        Action, Capabilities, Input, Mutation, Query, QueryToken, Receipt, Render,
    };
    struct Host {
        memory: glory::render::memory::Memory,
        actions: Rc<RefCell<Vec<Action>>>,
        queued: Rc<RefCell<Vec<Input>>>,
    }
    impl Render for Host {
        fn style_profile(&self) -> glory::style::Profile {
            glory::style::Profile::RECORD
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities {
                focus: true,
                scroll: true,
                ..Default::default()
            }
        }
        fn apply(&mut self, m: &[Mutation]) -> std::result::Result<Receipt, glory::render::Error> {
            self.memory.apply(m)
        }
        fn request(
            &mut self,
            q: Query,
            t: QueryToken,
        ) -> std::result::Result<(), glory::render::Error> {
            self.memory.request(q, t)
        }
        fn action(&mut self, a: Action) -> std::result::Result<(), glory::render::Error> {
            if matches!(a, Action::Focus(_) | Action::ScrollIntoView(_)) {
                assert!(
                    self.memory.active(a.node()),
                    "focus/reveal must follow the visibility commit"
                );
            }
            self.actions.borrow_mut().push(a);
            Ok(())
        }
        fn drain(&mut self, out: &mut Vec<Input>) {
            out.append(&mut self.queued.borrow_mut());
        }
    }
    let actions = Rc::new(RefCell::new(Vec::new()));
    let output = actions.clone();
    let queued = Rc::new(RefCell::new(Vec::<Input>::new()));
    let queue = queued.clone();
    let app = App::new(move |runtime| Host {
        memory: glory::render::memory::Memory::new(runtime),
        actions: output,
        queued: queue,
    });
    let open = app.scope().cage(false);
    let accept = app.scope().cage(true);
    app.mount(
        dropdown_menu("reveal")
            .label("Actions")
            .open(open)
            .on_open_change(move |next| {
                if accept.get() {
                    open.revise(|value| *value = next)
                }
            })
            .items([
                menu_item("alpha").label("Alpha"),
                menu_item("beta").label("Beta"),
            ]),
    )
    .unwrap();
    let trigger = control(&app, "Actions");
    let alpha = control(&app, "Alpha");
    let beta = control(&app, "Beta");
    command(&app, trigger, MenuCommand::Last);
    app.settle(16).unwrap();
    assert_eq!(
        *actions.borrow(),
        [Action::Focus(beta), Action::ScrollIntoView(beta)]
    );
    actions.borrow_mut().clear();
    app.dispatch(
        beta,
        Event::Navigate {
            from: beta,
            command: Navigation::First,
        },
    )
    .unwrap();
    app.settle(16).unwrap();
    assert_eq!(
        *actions.borrow(),
        [Action::Focus(alpha), Action::ScrollIntoView(alpha)]
    );
    actions.borrow_mut().clear();
    accept.revise(|value| *value = false);
    queued.borrow_mut().extend([
        Input::Event {
            node: trigger,
            event: Event::Menu {
                from: trigger,
                command: MenuCommand::Last,
            },
        },
        Input::Event {
            node: alpha,
            event: Event::Menu {
                from: alpha,
                command: MenuCommand::Exit { backwards: false },
            },
        },
    ]);
    app.pump().unwrap();
    app.settle(16).unwrap();
    assert!(open.get(), "the controlled close was refused");
    assert_eq!(
        *actions.borrow(),
        [Action::ExitMenu {
            node: panels(&app)[0],
            backwards: false
        }],
        "an older focus/reveal request must not reclaim focus after Tab"
    );
    actions.borrow_mut().clear();
    app.clear().unwrap();
    app.settle(16).unwrap();
    assert!(actions.borrow().is_empty());
}
