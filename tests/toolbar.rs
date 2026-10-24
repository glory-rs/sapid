use glory::prelude::*;
use glory::render::{Event, EventKind, FocusMode, Navigation, NodeId, Role};
use sapid::{action_bar, button, toolbar, toolbar_group};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn nodes(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut out = Vec::new();
    while let Some(id) = pending.pop() {
        pending.extend(tree.node(id).unwrap().children.iter());
        out.push(id)
    }
    out
}
fn action(app: &App, label: &str) -> NodeId {
    let tree = app.tree();
    nodes(app).into_iter().find(|id|matches!(tree.node(*id).unwrap().descriptor.view().props,glory::render::Props::Button(p)if p.label==label)).unwrap()
}
fn toolbar_node(app: &App) -> NodeId {
    let tree = app.tree();
    nodes(app)
        .into_iter()
        .find(|id| tree.node(*id).unwrap().descriptor.view().role == Some(Role::Toolbar))
        .unwrap()
}
fn tab(app: &App, id: NodeId) -> FocusMode {
    app.tree().node(id).unwrap().descriptor.view().focus
}
#[test]
fn grouped_actions_share_one_roving_entry_and_repair_disabled_without_replacing_buttons() {
    let app = App::memory();
    let middle = app.scope().cage(false);
    app.mount(
        toolbar("tools")
            .group(
                toolbar_group("edit")
                    .name("Editing")
                    .button(button().label("Cut"))
                    .button(button().label("Copy").enabled(middle)),
            )
            .group(toolbar_group("more").button(button().label("Paste"))),
    )
    .unwrap();

    let a = action(&app, "Cut");
    let b = action(&app, "Copy");
    let c = action(&app, "Paste");
    assert_eq!(tab(&app, a), FocusMode::TabStop);
    assert_eq!(tab(&app, b), FocusMode::Programmatic);
    assert_eq!(tab(&app, c), FocusMode::Programmatic);
    app.dispatch(
        a,
        Event::Navigate {
            from: a,
            command: Navigation::Next,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(tab(&app, c), FocusMode::TabStop);
    app.dispatch(
        c,
        Event::Navigate {
            from: c,
            command: Navigation::First,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(tab(&app, a), FocusMode::TabStop);
    middle.revise(|v| *v = true);
    app.flush().unwrap();
    app.dispatch(
        a,
        Event::Navigate {
            from: a,
            command: Navigation::Next,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(tab(&app, b), FocusMode::TabStop);
    middle.revise(|v| *v = false);
    app.flush().unwrap();
    assert_eq!(tab(&app, a), FocusMode::TabStop);
    assert_eq!(action(&app, "Copy"), b);
}
#[test]
fn selection_bar_is_source_silent_and_clear_proposes_before_its_owner_retires() {
    let app = App::memory();
    let count = app.scope().cage(0usize);
    let calls = Rc::new(Cell::new(0));
    let observed = calls.clone();
    app.mount(
        action_bar("selection")
            .selected_count(count)
            .on_clear_selection(move || observed.set(observed.get() + 1))
            .group(toolbar_group("edit").button(button().label("Delete"))),
    )
    .unwrap();
    let clear = action(&app, "Clear selection");
    let delete = action(&app, "Delete");
    assert!(!app.dispatch(clear, Event::Activate).unwrap());
    count.revise(|v| *v = 3);
    app.flush().unwrap();
    assert!(app.dispatch(clear, Event::Activate).unwrap());
    assert_eq!(count.get(), 3);
    assert_eq!(calls.get(), 1);
    count.revise(|v| *v = 0);
    app.flush().unwrap();
    assert!(!app.dispatch(delete, Event::Activate).unwrap());
    assert_eq!(action(&app, "Delete"), delete);
}
#[test]
fn invalid_names_preserve_accepted_metadata_and_duplicate_or_unbounded_declarations_reject() {
    let app = App::memory();
    let name = app.scope().cage("Editing tools".to_string());
    app.mount(
        toolbar("names")
            .name(name)
            .group(toolbar_group("edit").button(button().label("A"))),
    )
    .unwrap();

    let root = toolbar_node(&app);
    name.revise(|v| *v = "x".repeat(1_000_000));
    assert!(app.flush().is_err());
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Editing tools")
    );
    name.revise(|v| *v = "Accepted tools".into());
    app.flush().unwrap();
    assert!(
        App::memory()
            .mount(
                toolbar("duplicate")
                    .group(toolbar_group("same").button(button()))
                    .group(toolbar_group("same").button(button()))
            )
            .is_err()
    );
    let consumed = Rc::new(Cell::new(0));
    let count = consumed.clone();
    let huge = (0..1_000_000).map(move |_| {
        count.set(count.get() + 1);
        button()
    });
    assert!(
        App::memory()
            .mount(toolbar("bounded").group(toolbar_group("many").buttons(huge)))
            .is_err()
    );
    assert_eq!(consumed.get(), 129);
}
#[test]
fn navigation_observer_runs_after_roving_and_retirement_drops_captures() {
    let app = App::memory();
    let log = Rc::new(RefCell::new(Vec::new()));
    let writes = log.clone();
    let dropped = Rc::new(Cell::new(0));
    struct Capture(Rc<Cell<usize>>);
    impl Drop for Capture {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1)
        }
    }
    let capture = Capture(dropped.clone());
    let mount = app
        .scope()
        .mount_with(|| {
            toolbar("observe")
                .on(EventKind::Navigate, move |_| {
                    let _ = &capture;
                    writes.borrow_mut().push("navigate")
                })
                .group(
                    toolbar_group("one")
                        .button(button().label("A"))
                        .button(button().label("B")),
                )
        })
        .unwrap();
    let a = action(&app, "A");
    let b = action(&app, "B");
    app.dispatch(
        a,
        Event::Navigate {
            from: a,
            command: Navigation::Next,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(tab(&app, b), FocusMode::TabStop);
    assert_eq!(*log.borrow(), ["navigate"]);
    drop(mount);
    app.flush().unwrap();
    assert_eq!(dropped.get(), 1);
    assert_eq!(app.stats().nodes, 0);
}
