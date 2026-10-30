use super::*;
use glory::render::{InputProps, Props};
use glory::ui::App;
use std::cell::RefCell;
use std::task::{Context, Poll, Waker};

fn nodes(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut nodes = Vec::new();
    while let Some(id) = pending.pop() {
        nodes.push(id);
        pending.extend(tree.node(id).unwrap().children.iter().copied())
    }
    nodes
}
fn editor(app: &App) -> NodeId {
    nodes(app)
        .into_iter()
        .find(|id| {
            matches!(
                app.tree().node(*id).unwrap().descriptor.view().props,
                Props::Input(_)
            )
        })
        .unwrap()
}
fn input(app: &App) -> InputProps {
    let tree = app.tree();
    let Props::Input(input) = tree.node(editor(app)).unwrap().descriptor.view().props else {
        panic!()
    };
    input.clone()
}
fn options(app: &App) -> Vec<NodeId> {
    nodes(app)
        .into_iter()
        .filter(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .is_some_and(|s| s.option.is_some())
        })
        .collect()
}
fn panel(app: &App) -> NodeId {
    nodes(app)
        .into_iter()
        .find(|id| app.tree().node(*id).unwrap().descriptor.view().role == Some(Role::Dialog))
        .unwrap()
}
fn open(app: &App) -> bool {
    !app.tree()
        .node(panel(app))
        .unwrap()
        .descriptor
        .view()
        .hidden()
}
fn edit(app: &App, value: &str, composing: bool) {
    app.dispatch(
        editor(app),
        Event::ValueChanged {
            value: value.into(),
            composing,
            revision: input(app).revision,
        },
    )
    .unwrap();
}
fn choice(app: &App, command: ChoiceCommand) {
    let from = editor(app);
    app.dispatch(from, Event::Choice { from, command }).unwrap();
}
fn sample() -> Command {
    command("palette")
        .default_open(true)
        .group(command_group("files", "Files"))
        .actions([
            command_action("new", "New document")
                .group("files")
                .keywords("create fresh"),
            command_action("blocked", "Unavailable")
                .group("files")
                .disabled(true),
            command_action("open", "Open document").group("files"),
            command_action("settings", "Settings").description("Application settings"),
        ])
}

#[test]
fn command_is_real_dialog_editor_grouped_listbox_and_application_actions_without_successful_value_proxy()
 {
    let app = App::memory();
    let actions = Rc::new(RefCell::new(Vec::new()));
    let output = actions.clone();
    app.mount(sample().on_action(move |key| output.borrow_mut().push(key)))
        .unwrap();
    assert!(open(&app));
    assert_eq!(options(&app).len(), 4);
    assert!(nodes(&app).iter().all(|id| !matches!(
        app.tree().node(*id).unwrap().descriptor.view().props,
        Props::Select(_)
    )));
    let tree = app.tree();
    let input = tree.node(editor(&app)).unwrap().descriptor.view();
    assert_eq!(
        input.semantics.as_ref().unwrap().combobox,
        Some(ComboboxKind::Editable)
    );
    assert!(
        input
            .semantics
            .as_ref()
            .unwrap()
            .active_descendant
            .is_some()
    );
    drop(tree);
    edit(&app, "fresh", false);
    assert_eq!(options(&app).len(), 1);
    choice(&app, ChoiceCommand::Commit);
    assert_eq!(*actions.borrow(), ["new"]);
    assert!(!open(&app));
}
#[test]
fn command_controlled_search_and_open_refusal_echo_atomically_and_composition_blocks_execution() {
    let app = App::memory();
    let query = app.scope().cage(String::new());
    let visible = app.scope().cage(true);
    let actions = Rc::new(RefCell::new(Vec::new()));
    let output = actions.clone();
    let proposals = Rc::new(RefCell::new(Vec::new()));
    let changes = proposals.clone();
    app.mount(
        sample()
            .search(query)
            .open(visible)
            .on_search_change(move |value| changes.borrow_mut().push(value))
            .on_action(move |key| output.borrow_mut().push(key)),
    )
    .unwrap();
    edit(&app, "No match", false);
    assert_eq!(input(&app).value, "");
    assert_eq!(query.get(), "");
    assert_eq!(options(&app).len(), 4);
    assert_eq!(*proposals.borrow(), ["No match"]);
    edit(&app, &"x".repeat(1_000_000), false);
    assert_eq!(input(&app).value, "");
    assert_eq!(*proposals.borrow(), ["No match"]);
    edit(&app, "预编辑", true);
    choice(&app, ChoiceCommand::Commit);
    assert!(actions.borrow().is_empty());
    assert!(open(&app));
    app.dispatch(editor(&app), Event::Composition { active: false })
        .unwrap();
    assert_eq!(input(&app).value, "");
    choice(&app, ChoiceCommand::Commit);
    assert_eq!(actions.borrow().len(), 1);
    assert!(open(&app));
    assert!(visible.get());
}
#[test]
fn stamped_controlled_results_remove_old_actions_before_new_results_arrive_and_reject_invalid_sources()
 {
    let app = App::memory();
    let query = app.scope().cage(String::new());
    let results = app.scope().cage(CommandResponse {
        query: String::new(),
        results: CommandResults::new(vec![command_action("old", "Old action")]),
    });
    let actions = Rc::new(Cell::new(0));
    let output = actions.clone();
    app.mount(
        command("controlled")
            .default_open(true)
            .search(query)
            .on_search_change(move |value| query.revise(|old| *old = value))
            .results(results)
            .on_action(move |_| output.set(output.get() + 1)),
    )
    .unwrap();
    let old = options(&app)[0];
    query.revise(|value| *value = "new".into());
    app.dispatch(old, Event::Activate).unwrap();
    assert_eq!(actions.get(), 0);
    assert!(options(&app).is_empty());
    results.revise(|value| {
        *value = CommandResponse {
            query: "new".into(),
            results: CommandResults::new(vec![command_action("new", "Ranked action")]),
        }
    });
    app.flush().unwrap();
    assert_eq!(options(&app).len(), 1);
    results.revise(|value| value.results.actions[0].label = "x".repeat(1_000_000));
    assert!(app.flush().is_err());
    assert_eq!(options(&app).len(), 1);
    results.revise(|value| value.results.actions[0].label = "Ranked action".into());
    app.flush().unwrap();
    choice(&app, ChoiceCommand::Commit);
    assert_eq!(actions.get(), 1);
}
#[test]
fn command_labels_reject_blank_palette_action_and_group_names_before_ui_allocation() {
    for blank in ["", " \t\n", "\u{2003}"] {
        for builder in [
            command("palette").label(blank),
            command("action").action(command_action("one", blank)),
            command("group").group(command_group("files", blank)),
            command("response").results(CommandResponse {
                query: String::new(),
                results: CommandResults::new(vec![command_action("one", blank)]),
            }),
            command("group-response").results(CommandResponse {
                query: String::new(),
                results: CommandResults {
                    actions: Vec::new(),
                    groups: vec![command_group("files", blank)],
                },
            }),
        ] {
            let app = App::memory();
            assert!(app.mount(builder).is_err());
            assert_eq!(app.stats().nodes, 0);
        }
    }
}

#[test]
fn command_admission_is_bounded_and_mutually_exclusive_sources_fail_before_ui_nodes() {
    let app = App::memory();
    let consumed = Rc::new(Cell::new(0));
    let count = consumed.clone();
    let builder = command("many").actions((0..1_000_000).map(move |index| {
        count.set(count.get() + 1);
        command_action(format!("item-{index}"), "Action")
    }));
    assert_eq!(consumed.get(), 257);
    assert!(app.mount(builder).is_err());
    assert_eq!(app.stats().nodes, 0);
    let app = App::memory();
    assert!(
        app.mount(
            command("sources")
                .action(command_action("one", "One"))
                .results(CommandResponse::default())
        )
        .is_err()
    );
    assert_eq!(app.stats().nodes, 0);
}
#[test]
fn command_handle_has_one_explicit_shortcut_scope_and_drops_application_captures_on_owner_retirement()
 {
    let app = App::memory();
    let owner = app.scope().child().unwrap();
    let handle = Rc::new(RefCell::new(None));
    let ready = handle.clone();
    let capture = Rc::new(());
    let inside = capture.clone();
    let target = app.scope().cage(None);
    glory::ui::view()
        .node_ref(target)
        .show_in(app.scope())
        .unwrap();
    sample()
        .default_open(false)
        .on_ready(move |value| *ready.borrow_mut() = Some(value))
        .on_action(move |_| {
            let _ = &inside;
        })
        .show_in(&owner)
        .unwrap();
    app.flush().unwrap();
    let handle = handle.borrow().clone().unwrap();
    let target = target.get().unwrap();
    handle.register_shortcut(target).unwrap();
    handle.register_shortcut(target).unwrap();
    assert!(handle.register_shortcut(editor(&app)).is_err());
    app.dispatch(target, Event::Palette { from: target })
        .unwrap();
    assert!(open(&app));
    owner.dispose();
    assert_eq!(Rc::strong_count(&capture), 1);
    assert!(handle.open().is_err());
    assert!(handle.register_shortcut(target).is_err());
}

struct Gate {
    ready: Rc<Cell<bool>>,
    waker: Rc<RefCell<Option<Waker>>>,
    drops: Rc<Cell<usize>>,
    label: String,
}
impl Future for Gate {
    type Output = std::result::Result<CommandResults, String>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.ready.get() {
            Poll::Ready(Ok(CommandResults::new(vec![command_action(
                "loaded",
                self.label.clone(),
            )])))
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
#[test]
fn asynchronous_command_refresh_query_close_cancel_and_retirement_revoke_old_actions_and_late_wakes()
 {
    let executor = Rc::new(glory::core::executor::Executor::new(8));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let owner = app.scope().child().unwrap();
    let query = owner.cage(String::new());
    let disabled = owner.cage(false);
    let ready = Rc::new(Cell::new(false));
    let waker = Rc::new(RefCell::new(None::<Waker>));
    let drops = Rc::new(Cell::new(0));
    let starts = Rc::new(Cell::new(0));
    let r = ready.clone();
    let w = waker.clone();
    let d = drops.clone();
    let s = starts.clone();
    let handle = Rc::new(RefCell::new(None));
    let h = handle.clone();
    let actions = Rc::new(Cell::new(0));
    let a = actions.clone();
    command("async")
        .default_open(true)
        .search(query)
        .enabled(owner.bond(move || !disabled.get()))
        .on_search_change(move |value| query.revise(|old| *old = value))
        .on_ready(move |value| *h.borrow_mut() = Some(value))
        .on_action(move |_| a.set(a.get() + 1))
        .load_with(move |query| {
            s.set(s.get() + 1);
            Gate {
                ready: r.clone(),
                waker: w.clone(),
                drops: d.clone(),
                label: format!("Loaded {query}"),
            }
        })
        .show_in(&owner)
        .unwrap();
    app.flush().unwrap();
    let handle = handle.borrow().clone().unwrap();
    assert!(options(&app).is_empty());
    assert_eq!(starts.get(), 0);
    assert_eq!(executor.len(), 1);
    assert!(executor.poll_one());
    assert_eq!(starts.get(), 1);
    let old_waker = waker.borrow().clone().unwrap();
    query.revise(|value| *value = "next".into());
    app.flush().unwrap();
    assert_eq!(drops.get(), 1);
    assert!(options(&app).is_empty());
    old_waker.wake_by_ref();
    assert!(executor.poll_one());
    assert_eq!(starts.get(), 2);
    ready.set(true);
    waker.borrow().as_ref().unwrap().wake_by_ref();
    assert!(executor.poll_one());
    app.flush().unwrap();
    assert_eq!(options(&app).len(), 1);
    let stale = options(&app)[0];
    ready.set(false);
    handle.refresh().unwrap();
    app.dispatch(stale, Event::Activate).unwrap();
    assert_eq!(actions.get(), 0);
    assert!(options(&app).is_empty());
    assert!(executor.poll_one());
    assert_eq!(starts.get(), 3);
    let cancelled = waker.borrow().clone().unwrap();
    handle.cancel().unwrap();
    assert_eq!(drops.get(), 3);
    app.flush().unwrap();
    cancelled.wake_by_ref();
    assert!(!executor.poll_one());
    assert!(options(&app).is_empty());
    handle.refresh().unwrap();
    app.flush().unwrap();
    assert!(executor.poll_one());
    assert_eq!(starts.get(), 4);
    disabled.revise(|value| *value = true);
    app.flush().unwrap();
    assert_eq!(drops.get(), 4);
    while executor.poll_one() {}
    assert_eq!(starts.get(), 4);
    disabled.revise(|value| *value = false);
    app.flush().unwrap();
    assert!(executor.poll_one());
    assert_eq!(starts.get(), 5);
    handle.close().unwrap();
    app.flush().unwrap();
    assert_eq!(drops.get(), 5);
    while executor.poll_one() {}
    assert_eq!(starts.get(), 5);
    handle.open().unwrap();
    app.flush().unwrap();
    assert!(executor.poll_one());
    let late = waker.borrow().clone().unwrap();
    owner.dispose();
    assert_eq!(drops.get(), 6);
    assert!(executor.is_empty());
    late.wake_by_ref();
    assert!(!executor.poll_one());
    assert_eq!(app.stats().nodes, 0);
    assert!(handle.refresh().is_err());
}
#[test]
fn asynchronous_command_invalid_response_and_error_budgets_do_not_install_unbounded_rows() {
    let executor = Rc::new(glory::core::executor::Executor::new(8));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    app.mount(
        command("invalid-load")
            .default_open(true)
            .load_with(|_| async {
                Ok(CommandResults::new(vec![command_action(
                    "huge",
                    "x".repeat(1_000_000),
                )]))
            }),
    )
    .unwrap();
    assert!(executor.poll_one());
    app.flush().unwrap();
    assert!(options(&app).is_empty());
    assert!(nodes(&app).iter().any(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Text(text)if text.content.contains("budget"))));
}

#[test]
fn asynchronous_command_cold_ssr_and_browser_pending_shapes_agree_when_closed_or_initially_open() {
    fn shapes(app: &App) -> Vec<glory::html::Shape> {
        let tree = app.tree();
        nodes(app)
            .into_iter()
            .map(|id| glory::html::shape(tree.node(id).unwrap().descriptor.view()).unwrap())
            .collect()
    }
    for open in [false, true] {
        let cold = App::memory();
        cold.mount(
            command("cold")
                .default_open(open)
                .load_with(|_| std::future::pending()),
        )
        .unwrap();
        let executor = Rc::new(glory::core::executor::Executor::new(8));
        let ready = App::with_host(executor, glory::render::memory::Memory::new);
        ready
            .mount(
                command("cold")
                    .default_open(open)
                    .load_with(|_| std::future::pending()),
            )
            .unwrap();
        assert_eq!(shapes(&cold), shapes(&ready));
    }
}

#[test]
fn palette_scope_routes_only_the_nearest_live_owner_and_retirement_removes_external_registration() {
    let app = App::memory();
    let outer = app.scope().cage(None);
    let inner = app.scope().cage(None);
    let origin = app.scope().cage(None);
    glory::ui::view()
        .node_ref(outer)
        .show_in(app.scope())
        .unwrap();
    app.flush().unwrap();
    let outer_scope = app.scope().child_in(outer.get().unwrap()).unwrap();
    glory::ui::view()
        .node_ref(inner)
        .show_in(&outer_scope)
        .unwrap();
    app.flush().unwrap();
    let inner_scope = outer_scope.child_in(inner.get().unwrap()).unwrap();
    glory::ui::button()
        .label("Shortcut origin")
        .node_ref(origin)
        .show_in(&inner_scope)
        .unwrap();
    app.flush().unwrap();
    let outer_open = app.scope().cage(false);
    let inner_open = app.scope().cage(false);
    let h1 = Rc::new(RefCell::new(None));
    let h2 = Rc::new(RefCell::new(None));
    let r1 = h1.clone();
    let r2 = h2.clone();
    command("outer")
        .open(outer_open)
        .on_open_change(move |value| outer_open.revise(|old| *old = value))
        .on_ready(move |handle| *r1.borrow_mut() = Some(handle))
        .show_in(&outer_scope)
        .unwrap();
    command("inner")
        .open(inner_open)
        .on_open_change(move |value| inner_open.revise(|old| *old = value))
        .on_ready(move |handle| *r2.borrow_mut() = Some(handle))
        .show_in(&inner_scope)
        .unwrap();
    app.flush().unwrap();
    h1.borrow()
        .as_ref()
        .unwrap()
        .register_shortcut(outer.get().unwrap())
        .unwrap();
    h2.borrow()
        .as_ref()
        .unwrap()
        .register_shortcut(inner.get().unwrap())
        .unwrap();
    let from = origin.get().unwrap();
    app.dispatch(from, Event::Palette { from }).unwrap();
    assert!(inner_open.get());
    assert!(!outer_open.get());
    inner_scope.dispose();
    assert!(h2.borrow().as_ref().unwrap().open().is_err());
    assert!(!outer_open.get());
    let from = inner.get().unwrap();
    app.dispatch(from, Event::Palette { from }).unwrap();
    assert!(outer_open.get());
    outer_scope.dispose();
    assert!(h1.borrow().as_ref().unwrap().open().is_err());
}

#[test]
fn maximum_command_results_stay_below_live_subtree_budget_and_use_a_scrollable_listbox() {
    let app = App::memory();
    app.mount(command("max").default_open(true).actions(
        (0..256).map(|index| command_action(format!("item-{index}"), format!("Item {index}"))),
    ))
    .unwrap();
    assert_eq!(options(&app).len(), 256);
    assert!(app.stats().nodes < 4096);
    let list = nodes(&app)
        .into_iter()
        .find(|id| app.tree().node(*id).unwrap().descriptor.view().role == Some(Role::Listbox))
        .unwrap();
    let tree = app.tree();
    let style = tree.node(list).unwrap().descriptor.view().style;
    assert_eq!(style.max_height, Some(Limit::Px(320.0)));
    assert_eq!(style.overflow_y, Some(Overflow::Auto));
    drop(tree);
    choice(&app, ChoiceCommand::Previous);
    let tree = app.tree();
    let metadata = tree
        .node(editor(&app))
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap();
    assert!(
        metadata
            .active_descendant
            .as_ref()
            .unwrap()
            .as_str()
            .ends_with("item-255")
    );
}
