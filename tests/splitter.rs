use glory::prelude::*;
use glory::render::memory::Memory;
use glory::render::{
    Action, Capabilities, DragPhase, Error, Input, Mutation, NodeId, Query, QueryToken, QueryValue,
    RangeCommand, Receipt, Rect, Render,
};
use sapid::splitter;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn class(app: &App, name: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == name)
        {
            return id;
        }
        pending.extend(node.children.iter().copied());
    }
    panic!("missing {name}")
}
fn value(app: &App, node: NodeId) -> f64 {
    app.tree()
        .node(node)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .splitter
        .unwrap()
        .value
}
#[derive(Default)]
struct Wire {
    requests: Vec<(QueryToken, u64)>,
    inputs: Vec<Input>,
}
struct Host {
    memory: Memory,
    wire: Rc<RefCell<Wire>>,
    applied: u64,
}
impl Render for Host {
    fn style_profile(&self) -> glory::style::Profile {
        glory::style::Profile::RECORD
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            measure: true,
            focus: true,
            ..Default::default()
        }
    }
    fn apply(&mut self, mutations: &[Mutation]) -> std::result::Result<Receipt, Error> {
        let receipt = self.memory.apply(mutations)?;
        self.applied = receipt.applied;
        Ok(receipt)
    }
    fn request(&mut self, query: Query, token: QueryToken) -> std::result::Result<(), Error> {
        query.validate_target(&self.memory.node(query.node())?.descriptor)?;
        self.wire.borrow_mut().requests.push((token, self.applied));
        Ok(())
    }
    fn action(&mut self, _: Action) -> std::result::Result<(), Error> {
        Ok(())
    }
    fn drain(&mut self, inputs: &mut Vec<Input>) {
        inputs.append(&mut self.wire.borrow_mut().inputs);
    }
}
fn fixture() -> (App, Rc<RefCell<Wire>>) {
    let wire = Rc::new(RefCell::new(Wire::default()));
    let source = wire.clone();
    (
        App::new(move |runtime| Host {
            memory: Memory::new(runtime),
            wire: source,
            applied: 0,
        }),
        wire,
    )
}
fn pointer(app: &App, handle: NodeId, phase: DragPhase, x: f64, y: f64) {
    app.dispatch(
        handle,
        Event::Drag {
            from: handle,
            phase,
            x,
            y,
        },
    )
    .unwrap();
}
fn respond(wire: &RefCell<Wire>, index: usize, result: std::result::Result<QueryValue, Error>) {
    let (token, applied) = wire.borrow().requests[index];
    wire.borrow_mut().inputs.push(Input::Response {
        token,
        applied,
        result,
    });
}
#[test]
fn local_same_tick_keys_accumulate_and_typed_callbacks_precede_generic_observers() {
    let (app, wire) = fixture();
    let trace = Rc::new(RefCell::new(Vec::new()));
    let changed = trace.clone();
    let committed = trace.clone();
    let generic = trace.clone();
    app.mount(
        splitter("local")
            .on_value_change(move |value| changed.borrow_mut().push(format!("change:{value}")))
            .on_value_commit(move |value| committed.borrow_mut().push(format!("commit:{value}")))
            .on(glory::render::EventKind::Range, move |_| {
                generic.borrow_mut().push("generic".into())
            }),
    )
    .unwrap();
    let handle = class(&app, "sapid-splitter-handle");
    for _ in 0..2 {
        wire.borrow_mut().inputs.push(Input::Event {
            node: handle,
            event: Event::Range {
                from: handle,
                command: RangeCommand::Increase,
            },
        });
    }
    app.pump().unwrap();
    assert_eq!(value(&app, handle), 52.0);
    assert_eq!(
        *trace.borrow(),
        [
            "change:51",
            "commit:51",
            "generic",
            "change:52",
            "commit:52",
            "generic"
        ]
    );
    let first = class(&app, "sapid-splitter-first");
    let second = class(&app, "sapid-splitter-second");
    assert_eq!(
        app.tree().node(first).unwrap().descriptor.view().style.grow,
        Some(52.0)
    );
    assert_eq!(
        app.tree()
            .node(second)
            .unwrap()
            .descriptor
            .view()
            .style
            .grow,
        Some(48.0)
    );
}
#[test]
fn controlled_sources_are_readonly_silent_and_rejected_values_keep_accepted_callbacks() {
    let app = App::memory();
    let source = app.scope().cage(61.0);
    let requests = Rc::new(RefCell::new(Vec::new()));
    let output = requests.clone();
    app.mount(
        splitter("controlled")
            .default_value(22.0)
            .value(source)
            .on_value_change(move |value| output.borrow_mut().push(value)),
    )
    .unwrap();
    let handle = class(&app, "sapid-splitter-handle");
    app.dispatch(
        handle,
        Event::Range {
            from: handle,
            command: RangeCommand::Increase,
        },
    )
    .unwrap();
    assert_eq!(source.get(), 61.0);
    assert_eq!(value(&app, handle), 61.0);
    assert_eq!(*requests.borrow(), [62.0]);
    source.revise(|value| *value = 75.0);
    app.flush().unwrap();
    assert_eq!(requests.borrow().len(), 1);
    assert_eq!(value(&app, handle), 75.0);
    source.revise(|value| *value = f64::NAN);
    assert!(app.flush().is_err());
    assert_eq!(value(&app, handle), 75.0);
    app.dispatch(
        handle,
        Event::Range {
            from: handle,
            command: RangeCommand::Increase,
        },
    )
    .unwrap();
    assert_eq!(*requests.borrow(), [62.0, 76.0]);
    assert!(source.get().is_nan());
    source.revise(|value| *value = 70.0);
    app.flush().unwrap();
    assert_eq!(value(&app, handle), 70.0);
}
#[test]
fn bounds_and_step_reject_before_nodes_and_disabled_handle_rejects_edits() {
    let app = App::memory();
    for widget in [
        splitter("nan").default_value(f64::NAN),
        splitter("bounds").min(70.0).max(60.0),
        splitter("step").step(0.0),
        splitter("width").thickness(f64::INFINITY),
    ] {
        assert!(app.mount(widget).is_err());
        assert!(app.tree().is_empty());
    }
    let requests = Rc::new(Cell::new(0));
    let output = requests.clone();
    app.mount(
        splitter("disabled")
            .enabled(false)
            .on_value_change(move |_| output.set(output.get() + 1)),
    )
    .unwrap();
    let handle = class(&app, "sapid-splitter-handle");
    assert!(!app.tree().enabled(handle));
    assert_eq!(
        app.tree().node(handle).unwrap().descriptor.view().focus,
        glory::render::FocusMode::Programmatic
    );
    assert!(
        app.dispatch(
            handle,
            Event::Range {
                from: handle,
                command: RangeCommand::Increase
            }
        )
        .is_err()
    );
    assert_eq!(requests.get(), 0);
}
#[test]
fn panes_remain_mounted_with_stable_refs_and_factories_at_collapsed_endpoints() {
    let app = App::memory();
    let source = app.scope().cage(50.0);
    let enabled = app.scope().cage(true);
    let input = app.scope().cage(None);
    let handle = app.scope().cage(None);
    let count = Rc::new(Cell::new(0));
    let built = count.clone();
    app.mount(
        splitter("stable")
            .enabled(enabled)
            .min(0.0)
            .max(100.0)
            .value(source)
            .name("Resize editor panes")
            .handle_ref(handle)
            .first(move || {
                built.set(built.get() + 1);
                glory::ui::input().value("Kept draft").node_ref(input)
            })
            .second(|| glory::ui::text().content("Second")),
    )
    .unwrap();
    let identity = input.get().unwrap();
    let handle_id = handle.get().unwrap();
    assert_eq!(
        app.tree()
            .node(handle_id)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Resize editor panes")
    );
    for next in [0.0, 100.0, 25.0, 75.0] {
        source.revise(|value| *value = next);
        app.flush().unwrap();
        assert_eq!(input.get(), Some(identity));
        assert_eq!(count.get(), 1);
    }
    let pane = class(&app, "sapid-splitter-first");
    app.dispatch(
        pane,
        Event::FocusWithin {
            node: identity,
            focused: true,
        },
    )
    .unwrap();
    app.scope()
        .batch(|| {
            enabled.revise(|value| *value = false);
            source.revise(|value| *value = 0.0);
        })
        .unwrap();
    app.flush().unwrap();
    assert_eq!(input.get(), Some(identity));
    app.clear().unwrap();
    assert!(input.get().is_none());
    assert!(handle.get().is_none());
}
#[test]
fn pending_drag_coalesces_absolute_points_and_commits_before_its_generic_drop() {
    let (app, wire) = fixture();
    let trace = Rc::new(RefCell::new(Vec::new()));
    let change = trace.clone();
    let commit_observer = trace.clone();
    let observer = trace.clone();
    app.mount(
        splitter("drag")
            .on_value_change(move |value| change.borrow_mut().push(format!("change:{value:.2}")))
            .on_value_commit(move |value| {
                commit_observer
                    .borrow_mut()
                    .push(format!("commit:{value:.2}"))
            })
            .on(glory::render::EventKind::Drag, move |event| {
                if let Event::Drag { phase, x, y, .. } = event {
                    observer.borrow_mut().push(format!("{phase:?}:{x}:{y}"));
                }
            }),
    )
    .unwrap();
    let handle = class(&app, "sapid-splitter-handle");
    pointer(&app, handle, DragPhase::Start, 200.0, 12.0);
    pointer(&app, handle, DragPhase::Move, 260.0, 15.0);
    pointer(&app, handle, DragPhase::Drop, 300.0, 17.0);
    assert_eq!(wire.borrow().requests.len(), 1);
    assert_eq!(*trace.borrow(), ["Start:200:12"]);
    respond(
        &wire,
        0,
        Ok(QueryValue::Bounds(Rect {
            width: 606.0,
            height: 300.0,
            ..Default::default()
        })),
    );
    app.pump().unwrap();
    assert_eq!(
        *trace.borrow(),
        [
            "Start:200:12",
            "change:66.67",
            "commit:66.67",
            "Drop:300:17"
        ]
    );
    assert!((value(&app, handle) - 66.6666666667).abs() < 0.0001);
}
#[test]
fn cancel_and_owner_retirement_remove_pending_geometry_and_ignore_late_errors() {
    let (app, wire) = fixture();
    let count = Rc::new(Cell::new(0));
    let output = count.clone();
    app.mount(splitter("cancel").on_value_change(move |_| output.set(output.get() + 1)))
        .unwrap();
    let handle = class(&app, "sapid-splitter-handle");
    for _ in 0..128 {
        pointer(&app, handle, DragPhase::Start, 100.0, 0.0);
        assert_eq!(app.stats().queries, 1);
        pointer(&app, handle, DragPhase::Cancel, 120.0, 0.0);
        assert_eq!(
            (
                app.stats().queries,
                app.stats().query_owners,
                app.stats().query_targets
            ),
            (0, 0, 0)
        );
    }
    for index in 0..128 {
        respond(&wire, index, Err(Error::Unsupported("obsolete bounds")));
    }
    app.pump().unwrap();
    assert_eq!(count.get(), 0);
    pointer(&app, handle, DragPhase::Start, 100.0, 0.0);
    app.clear().unwrap();
    assert_eq!(app.stats().queries, 0);
    respond(
        &wire,
        128,
        Ok(QueryValue::Bounds(Rect {
            width: 606.0,
            height: 300.0,
            ..Default::default()
        })),
    );
    app.pump().unwrap();
    assert_eq!(count.get(), 0);
}
