use glory::prelude::*;
use glory::render::{EventKind, FocusMode, Navigation, NodeId};
use sapid::{ToggleMode, toggle, toggle_group, toggle_item};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            return id;
        }
        pending.extend(node.children.iter().copied());
    }
    panic!("missing {class}")
}
fn pressed(app: &App, class: &str) -> bool {
    app.tree()
        .node(find(app, class))
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .pressed
        .unwrap()
}
#[test]
fn local_toggle_owns_state_and_typed_callback_precedes_observer() {
    let app = App::memory();
    let trace = Rc::new(RefCell::new(Vec::new()));
    let a = trace.clone();
    let b = trace.clone();
    app.mount(
        toggle()
            .class("toggle")
            .label("Bold")
            .on_pressed_change(move |value| a.borrow_mut().push(if value { "on" } else { "off" }))
            .on(EventKind::Activate, move |_| {
                b.borrow_mut().push("observed")
            }),
    )
    .unwrap();
    let id = find(&app, "toggle");
    app.dispatch(id, Event::Activate).unwrap();
    assert!(pressed(&app, "toggle"));
    app.dispatch(id, Event::Activate).unwrap();
    assert!(!pressed(&app, "toggle"));
    assert_eq!(*trace.borrow(), vec!["on", "observed", "off", "observed"]);
}
#[test]
fn controlled_rejection_and_external_writes_are_silent() {
    let app = App::memory();
    let value = app.scope().cage(false);
    let calls = Rc::new(Cell::new(0));
    let called = calls.clone();
    app.mount(
        toggle()
            .class("toggle")
            .pressed(value)
            .default_pressed(true)
            .on_pressed_change(move |proposal| {
                assert!(proposal);
                called.set(called.get() + 1);
            }),
    )
    .unwrap();
    app.dispatch(find(&app, "toggle"), Event::Activate).unwrap();
    assert!(!pressed(&app, "toggle"));
    assert_eq!(calls.get(), 1);
    value.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(pressed(&app, "toggle"));
    assert_eq!(calls.get(), 1);
}
#[test]
fn disabled_toggle_rejects_activation_and_keeps_current_state() {
    let app = App::memory();
    let enabled = app.scope().cage(true);
    let calls = Rc::new(Cell::new(0));
    let called = calls.clone();
    app.mount(
        toggle()
            .class("toggle")
            .enabled(enabled)
            .default_pressed(true)
            .on_pressed_change(move |_| called.set(1)),
    )
    .unwrap();
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(!app.dispatch(find(&app, "toggle"), Event::Activate).unwrap());
    assert!(pressed(&app, "toggle"));
    assert_eq!(calls.get(), 0);
}
#[test]
fn single_group_replaces_and_clears_selection() {
    let app = App::memory();
    let last = Rc::new(RefCell::new(Vec::<String>::new()));
    let output = last.clone();
    app.mount(
        toggle_group("alignment")
            .name("Alignment")
            .default_value(["left"])
            .items([
                toggle_item("left").class("left"),
                toggle_item("center").class("center"),
            ])
            .on_value_change(move |value| *output.borrow_mut() = value),
    )
    .unwrap();
    assert!(pressed(&app, "left"));
    app.dispatch(find(&app, "center"), Event::Activate).unwrap();
    assert!(!pressed(&app, "left") && pressed(&app, "center"));
    assert_eq!(*last.borrow(), vec!["center"]);
    app.dispatch(find(&app, "center"), Event::Activate).unwrap();
    assert!(!pressed(&app, "center"));
    assert!(last.borrow().is_empty());
}
#[test]
fn multiple_group_preserves_disabled_selected_values_and_order() {
    let app = App::memory();
    let last = Rc::new(RefCell::new(Vec::<String>::new()));
    let output = last.clone();
    app.mount(
        toggle_group("format")
            .mode(ToggleMode::Multiple)
            .default_value(["locked"])
            .items([
                toggle_item("locked").class("locked").enabled(false),
                toggle_item("bold").class("bold"),
                toggle_item("italic").class("italic"),
            ])
            .on_value_change(move |value| *output.borrow_mut() = value),
    )
    .unwrap();
    app.dispatch(find(&app, "bold"), Event::Activate).unwrap();
    app.dispatch(find(&app, "italic"), Event::Activate).unwrap();
    assert_eq!(*last.borrow(), vec!["locked", "bold", "italic"]);
    app.dispatch(find(&app, "bold"), Event::Activate).unwrap();
    assert_eq!(*last.borrow(), vec!["locked", "italic"]);
    assert!(pressed(&app, "locked"));
}
#[test]
fn group_controlled_rejection_never_publishes_proposal() {
    let app = App::memory();
    let value = app.scope().cage(vec!["a".to_owned()]);
    let last = Rc::new(RefCell::new(Vec::<String>::new()));
    let output = last.clone();
    app.mount(
        toggle_group("controlled")
            .value(value)
            .default_value(["b"])
            .items([toggle_item("a").class("a"), toggle_item("b").class("b")])
            .on_value_change(move |value| *output.borrow_mut() = value),
    )
    .unwrap();
    app.dispatch(find(&app, "b"), Event::Activate).unwrap();
    assert!(pressed(&app, "a") && !pressed(&app, "b"));
    assert_eq!(*last.borrow(), vec!["b"]);
    value.revise(|v| *v = vec!["b".into()]);
    app.flush().unwrap();
    assert!(pressed(&app, "b"));
}
#[test]
fn roving_follows_actual_focus_and_navigation_does_not_select() {
    let app = App::memory();
    let value = app.scope().cage(vec!["a".to_owned()]);
    app.mount(toggle_group("roving").value(value).items([
        toggle_item("a").class("a"),
        toggle_item("disabled").class("disabled").enabled(false),
        toggle_item("c").class("c"),
    ]))
    .unwrap();
    let a = find(&app, "a");
    let c = find(&app, "c");
    assert_eq!(
        app.tree().node(a).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    app.dispatch(c, Event::FocusChanged(true)).unwrap();
    assert_eq!(
        app.tree().node(c).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    app.dispatch(
        a,
        Event::Navigate {
            from: a,
            command: Navigation::Next,
        },
    )
    .unwrap();
    assert_eq!(value.get(), vec!["a"]);
    assert!(pressed(&app, "a") && !pressed(&app, "c"));
}
#[test]
fn external_invalid_group_value_preserves_last_valid_ui_and_reports_error() {
    let app = App::memory();
    let value = app.scope().cage(vec!["a".to_owned()]);
    app.mount(
        toggle_group("invalid")
            .value(value)
            .items([toggle_item("a").class("a"), toggle_item("b").class("b")]),
    )
    .unwrap();
    value.revise(|v| *v = vec!["unknown".into()]);
    assert!(app.flush().is_err());
    assert!(pressed(&app, "a"));
}
#[test]
fn invalid_group_configurations_fail_without_empty_builders() {
    assert!(App::memory().mount(toggle_group("empty")).is_err());
    assert!(
        App::memory()
            .mount(toggle_group("dupe").items([toggle_item("x"), toggle_item("x")]))
            .is_err()
    );
    assert!(
        App::memory()
            .mount(
                toggle_group("many")
                    .default_value(["a", "b"])
                    .items([toggle_item("a"), toggle_item("b")])
            )
            .is_err()
    );
    assert!(
        App::memory()
            .mount(
                toggle_group("unknown")
                    .default_value(["no"])
                    .item(toggle_item("a"))
            )
            .is_err()
    );
}
#[test]
fn retirement_releases_callbacks_and_references() {
    struct DropCount(Rc<Cell<usize>>);
    impl Drop for DropCount {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1)
        }
    }
    let app = App::memory();
    let dropped = Rc::new(Cell::new(0));
    let capture = DropCount(dropped.clone());
    let reference = app.scope().cage(None);
    app.mount(
        toggle_group("retire")
            .item(toggle_item("a").node_ref(reference))
            .on_value_change(move |_| {
                let _ = &capture;
            }),
    )
    .unwrap();
    assert!(reference.get().is_some());
    app.clear().unwrap();
    assert!(reference.get().is_none());
    assert_eq!(dropped.get(), 1);
}
#[test]
fn ssr_pressed_buttons_remain_native_buttons_with_escaped_labels() {
    let html = glory::ssr::render(
        toggle_group("ssr")
            .name("Text styles")
            .mode(ToggleMode::Multiple)
            .default_value(["bold"])
            .items([
                toggle_item("bold").label("Bold <text>"),
                toggle_item("italic").label("Italic"),
            ]),
    )
    .unwrap();
    assert!(html.contains("role=\"group\""));
    assert!(html.contains("aria-pressed=\"true\""));
    assert!(html.contains("aria-pressed=\"false\""));
    assert!(html.contains("type=\"button\""));
    assert!(html.contains("Bold &lt;text&gt;"));
    assert!(!html.contains("role=\"tab\""));
}

#[test]
fn group_typed_change_precedes_item_and_group_observers() {
    let app = App::memory();
    let trace = Rc::new(RefCell::new(Vec::new()));
    let a = trace.clone();
    let b = trace.clone();
    let c = trace.clone();
    app.mount(
        toggle_group("order")
            .item(
                toggle_item("a")
                    .class("a")
                    .on(EventKind::Activate, move |_| b.borrow_mut().push("item")),
            )
            .on_value_change(move |_| a.borrow_mut().push("typed"))
            .on(EventKind::Activate, move |_| c.borrow_mut().push("group")),
    )
    .unwrap();
    app.dispatch(find(&app, "a"), Event::Activate).unwrap();
    assert_eq!(*trace.borrow(), vec!["typed", "item", "group"]);
}

#[test]
fn constructor_items_stop_at_overflow_and_drop_discarded_captures_outside_borrows() {
    struct Probe(Cage<usize>);
    impl Drop for Probe {
        fn drop(&mut self) {
            self.0.revise(|value| *value += 1);
        }
    }
    let app = App::memory();
    let dropped = app.scope().cage(0_usize);
    let consumed = Rc::new(Cell::new(0));
    let visited = consumed.clone();
    let group = toggle_group("bounded").items((0..1_000_000).map(move |index| {
        visited.set(visited.get() + 1);
        let capture = Probe(dropped);
        toggle_item(format!("item{index}")).on(EventKind::Activate, move |_| {
            let _ = &capture;
        })
    }));
    assert_eq!(consumed.get(), sapid::MAX_TOGGLE_GROUP_ITEMS + 1);
    assert_eq!(dropped.get(), 0);
    let capture = Probe(dropped);
    let group = group.item(toggle_item("extra").on(EventKind::Activate, move |_| {
        let _ = &capture;
    }));
    assert_eq!(dropped.get(), 1);
    let unused = Rc::new(Cell::new(0));
    let visited = unused.clone();
    let capture = Probe(dropped);
    let group = group.items((0..1_000_000).map(move |index| {
        let _ = &capture;
        visited.set(visited.get() + 1);
        toggle_item(format!("unused{index}"))
    }));
    assert_eq!(unused.get(), 0);
    assert_eq!(dropped.get(), 2);
    assert!(app.mount(group).is_err());
    assert!(app.tree().roots().is_empty());
    assert_eq!(dropped.get(), sapid::MAX_TOGGLE_GROUP_ITEMS + 3);
}

#[test]
fn default_selection_is_bounded_and_overflow_is_rejected_even_with_controlled_value() {
    struct Probe(Cage<usize>);
    impl Drop for Probe {
        fn drop(&mut self) {
            self.0.revise(|value| *value += 1);
        }
    }
    let app = App::memory();
    let dropped = app.scope().cage(0);
    let consumed = Rc::new(Cell::new(0));
    let visited = consumed.clone();
    let capture = Probe(dropped);
    let group = toggle_group("bounded-default")
        .mode(ToggleMode::Multiple)
        .item(toggle_item("one"))
        .default_value((0..1_000_000).map(move |index| {
            let _ = &capture;
            visited.set(visited.get() + 1);
            format!("item{index}")
        }))
        .value(Vec::<String>::new());
    assert_eq!(consumed.get(), sapid::MAX_TOGGLE_GROUP_ITEMS + 1);
    assert_eq!(dropped.get(), 1);
    assert!(app.mount(group).is_err());
    assert!(app.tree().roots().is_empty());
}

#[test]
fn repeated_single_additions_retain_only_the_overflow_sentinel() {
    struct Probe(Rc<Cell<usize>>);
    impl Drop for Probe {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let dropped = Rc::new(Cell::new(0));
    let mut group = toggle_group("repeated");
    let total = sapid::MAX_TOGGLE_GROUP_ITEMS + 17;
    for index in 0..total {
        let capture = Probe(dropped.clone());
        group = group.item(toggle_item(format!("item{index}")).on(
            EventKind::Activate,
            move |_| {
                let _ = &capture;
            },
        ));
    }
    assert_eq!(dropped.get(), 16);
    let app = App::memory();
    assert!(app.mount(group).is_err());
    assert_eq!(dropped.get(), total);
    assert!(app.tree().roots().is_empty());
}
