use glory::render::{memory::Memory, *};
use glory::ui::App;
use sapid::{OverflowHandle, overflow_item, overflow_list};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

fn nodes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut found = Vec::new();
    while let Some(id) = todo.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            found.push(id)
        }
        todo.extend(node.children.iter().rev());
    }
    found
}
fn entries() -> Vec<String> {
    ["a", "b", "c"].map(str::to_owned).to_vec()
}
fn item(key: &str) -> sapid::OverflowItem {
    let class = format!("editor-{key}");
    overflow_item(key)
        .width(60.)
        .content(move || glory::ui::input().class(class.clone()).value("draft"))
}
#[test]
fn finite_iterators_stop_at_the_sentinel_before_any_item_factory() {
    let consumed = Rc::new(Cell::new(0));
    let count = consumed.clone();
    let input = (0..1_000_000).map(move |index| {
        count.set(count.get() + 1);
        overflow_item(format!("item-{index}"))
    });
    let widget = overflow_list("bounded").items(input);
    assert_eq!(consumed.get(), 129);
    let app = App::memory();
    assert!(app.mount(widget).is_err());
    assert!(
        App::memory()
            .mount(overflow_list("bad-width").item(overflow_item("a").width(f64::NAN)))
            .is_err()
    );
    assert!(
        App::memory()
            .mount(overflow_list("bad-gap").gap(f64::INFINITY))
            .is_err()
    );
}
#[test]
fn current_keys_keep_single_owner_and_editor_identity_across_moves_reorders_and_reinsert() {
    let app = App::memory();
    let source = app.scope().cage(entries());
    let visible = app.scope().cage(1);
    let builds = Rc::new(Cell::new(0));
    let count = builds.clone();
    app.mount(
        overflow_list("keys")
            .visible_count(visible)
            .dynamic_items(source, move |key| {
                count.set(count.get() + 1);
                item(key)
            }),
    )
    .unwrap();
    assert_eq!(builds.get(), 3);
    let old = nodes(&app, "editor-a")[0];
    let row = nodes(&app, "sapid-overflow-item");
    visible.revise(|value| *value = 3);
    app.flush().unwrap();
    assert_eq!(nodes(&app, "editor-a"), [old]);
    assert!(
        row.iter().all(|id| app.tree().node(*id).unwrap().parent
            == Some(nodes(&app, "sapid-overflow-strip")[0]))
    );
    source.revise(|keys| keys.reverse());
    app.flush().unwrap();
    assert_eq!(builds.get(), 3);
    assert_eq!(nodes(&app, "editor-a"), [old]);
    source.revise(|keys| keys.retain(|key| key != "a"));
    app.flush().unwrap();
    assert!(nodes(&app, "editor-a").is_empty());
    source.revise(|keys| keys.push("a".into()));
    app.flush().unwrap();
    assert_eq!(builds.get(), 4);
    assert_ne!(nodes(&app, "editor-a"), [old]);
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(app.stats().queries, 0);
}
#[derive(Default)]
struct Wire {
    width: Cell<f64>,
    focus: Cell<Option<NodeId>>,
    accept_focus: Cell<bool>,
    defer: Cell<bool>,
    inputs: RefCell<Vec<Input>>,
    requests: RefCell<Vec<(Query, QueryToken, u64)>>,
}
struct Host {
    memory: Memory,
    wire: Rc<Wire>,
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
    fn apply(&mut self, mutations: &[Mutation]) -> Result<Receipt, Error> {
        self.memory.apply(mutations)
    }
    fn request(&mut self, query: Query, token: QueryToken) -> Result<(), Error> {
        let node = self.memory.node(query.node())?;
        let applied = self.memory.receipt().applied;
        self.wire
            .requests
            .borrow_mut()
            .push((query, token, applied));
        if self.wire.defer.get() {
            return Ok(());
        }
        let value = match query {
            Query::Focus(node) => QueryValue::Focus(self.wire.focus.get() == Some(node)),
            Query::Bounds(_) => {
                let width = if node
                    .descriptor
                    .view()
                    .classes
                    .iter()
                    .any(|class| class == "sapid-overflow-line")
                {
                    self.wire.width.get()
                } else if let Some(glory::style::Length::Px(width)) =
                    node.descriptor.view().style.width
                {
                    width
                } else {
                    40.
                };
                QueryValue::Bounds(Rect {
                    x: 0.,
                    y: 0.,
                    width,
                    height: 30.,
                })
            }
            _ => unreachable!(),
        };
        self.wire.inputs.borrow_mut().push(Input::Response {
            token,
            applied,
            result: Ok(value),
        });
        Ok(())
    }
    fn action(&mut self, action: Action) -> Result<(), Error> {
        if let Action::Focus(node) = action
            && self.wire.accept_focus.get()
        {
            if let Some(old) = self.wire.focus.replace(Some(node)) {
                self.wire.inputs.borrow_mut().push(Input::Event {
                    node: old,
                    event: Event::FocusChanged(false),
                })
            }
            self.wire.inputs.borrow_mut().push(Input::Event {
                node,
                event: Event::FocusChanged(true),
            });
        }
        Ok(())
    }
    fn drain(&mut self, out: &mut Vec<Input>) {
        out.extend(self.wire.inputs.take())
    }
}
type Fixture = (
    App,
    Rc<glory::core::executor::Executor>,
    Rc<Wire>,
    OverflowHandle,
    glory::Cage<usize>,
    glory::Cage<Vec<usize>>,
);
fn fixture(controlled: bool) -> Fixture {
    let executor = Rc::new(glory::core::executor::Executor::new(8));
    let wire = Rc::new(Wire::default());
    wire.width.set(226.);
    wire.accept_focus.set(true);
    let output = wire.clone();
    let app = App::with_host(executor.clone(), move |runtime| Host {
        memory: Memory::new(runtime),
        wire: output,
    });
    let handle = app.scope().cage(None);
    let count = app.scope().cage(3usize);
    let proposals = app.scope().cage(Vec::new());
    let mut widget = overflow_list("measure")
        .initial_visible(3)
        .handle_ref(handle)
        .items(entries().iter().map(|key| item(key)))
        .on_visible_count_change(move |next| proposals.revise(|values| values.push(next)));
    if controlled {
        widget = widget.visible_count(count)
    }
    app.mount(widget).unwrap();
    (app, executor, wire, handle.get().unwrap(), count, proposals)
}
fn settle(app: &App, executor: &glory::core::executor::Executor) {
    for _ in 0..12 {
        for _ in 0..32 {
            if !executor.poll_one() {
                break;
            }
        }
        app.pump().unwrap();
        if executor.is_empty() && app.stats().queries == 0 {
            return;
        }
    }
    panic!("overflow did not quiesce")
}
#[test]
fn real_query_barriers_measure_a_bounded_current_prefix_and_controlled_refusals_stay_silent() {
    for controlled in [false, true] {
        let (app, executor, wire, handle, count, proposals) = fixture(controlled);
        settle(&app, &executor);
        assert_eq!(handle.visible_count().get(), 3);
        wire.width.set(120.);
        handle.refresh().unwrap();
        settle(&app, &executor);
        assert_eq!(handle.visible_count().get(), if controlled { 3 } else { 1 });
        assert_eq!(proposals.get().last(), Some(&1));
        assert_eq!(handle.pending_queries().get(), 0);
        assert_eq!(app.stats().queries, 0);
        proposals.revise(Vec::clear);
        if controlled {
            count.revise(|value| *value = 1);
            app.flush().unwrap();
            settle(&app, &executor);
            assert_eq!(handle.visible_count().get(), 1);
            assert!(proposals.get().is_empty());
        }
        app.clear().unwrap();
        assert!(handle.refresh().is_err());
        assert_eq!(app.runtime_stats().tasks, 0);
    }
}
#[test]
fn pending_measurements_coalesce_and_owner_retirement_cancels_every_current_query() {
    let (app, executor, wire, handle, _, _) = fixture(false);
    settle(&app, &executor);
    wire.defer.set(true);
    wire.requests.borrow_mut().clear();
    handle.refresh().unwrap();
    for _ in 0..10 {
        handle.refresh().unwrap()
    }
    while executor.poll_one() {}
    assert_eq!(app.stats().queries, 5);
    assert_eq!(handle.pending_queries().get(), 5);
    assert_eq!(wire.requests.borrow().len(), 5);
    app.clear().unwrap();
    assert_eq!(app.stats().queries, 0);
    assert_eq!(app.runtime_stats().tasks, 0);
    for (_, token, applied) in wire.requests.take() {
        wire.inputs.borrow_mut().push(Input::Response {
            token,
            applied,
            result: Ok(QueryValue::Bounds(Rect {
                x: 0.,
                y: 0.,
                width: 60.,
                height: 30.,
            })),
        })
    }
    app.pump().unwrap();
    assert_eq!(app.stats().nodes, 0);
}

#[test]
fn focused_item_moving_into_closed_overflow_uses_actual_focus_and_refusal_does_not_stick() {
    for accept in [false, true] {
        let (app, executor, wire, handle, _, _) = fixture(false);
        settle(&app, &executor);
        wire.accept_focus.set(accept);
        let editor = nodes(&app, "editor-c")[0];
        wire.focus.set(Some(editor));
        app.dispatch(editor, Event::FocusChanged(true)).unwrap();
        wire.width.set(120.);
        handle.refresh().unwrap();
        settle(&app, &executor);
        let trigger = nodes(&app, "sapid-popover-trigger")[0];
        assert_eq!(
            wire.focus.get(),
            Some(if accept { trigger } else { editor })
        );
        assert_eq!(app.stats().queries, 0);
        assert_eq!(app.runtime_stats().tasks, 0);
        assert_eq!(handle.visible_count().get(), 1);
        assert_eq!(nodes(&app, "editor-c"), [editor]);
        // A later real focus transition is never interpreted as an old queued
        // restoration. Returning the same widget to the strip keeps identity.
        wire.focus.set(Some(nodes(&app, "editor-a")[0]));
        app.dispatch(nodes(&app, "editor-a")[0], Event::FocusChanged(true))
            .unwrap();
        wire.width.set(300.);
        handle.refresh().unwrap();
        settle(&app, &executor);
        assert_eq!(nodes(&app, "editor-c"), [editor]);
        assert_eq!(handle.visible_count().get(), 3);
        app.clear().unwrap();
        assert_eq!(app.stats().queries, 0);
    }
}

#[test]
fn maximum_membership_never_has_more_than_128_plus_container_and_trigger_queries() {
    let executor = Rc::new(glory::core::executor::Executor::new(8));
    let wire = Rc::new(Wire::default());
    wire.width.set(300.);
    wire.defer.set(true);
    let output = wire.clone();
    let app = App::with_host(executor.clone(), move |runtime| Host {
        memory: Memory::new(runtime),
        wire: output,
    });
    let reference = app.scope().cage(None);
    app.mount(
        overflow_list("maximum")
            .handle_ref(reference)
            .items((0..128).map(|index| overflow_item(format!("item-{index}")).width(60.))),
    )
    .unwrap();
    while executor.poll_one() {}
    assert_eq!(app.stats().queries, sapid::MAX_OVERFLOW_QUERIES);
    assert_eq!(reference.get().unwrap().pending_queries().get(), 130);
    app.clear().unwrap();
    assert_eq!(app.stats().queries, 0);
    assert_eq!(app.runtime_stats().tasks, 0);
}

#[test]
fn closing_cancels_the_old_observation_before_a_new_current_measurement_can_begin() {
    let (app, executor, wire, handle, _, _) = fixture(false);
    settle(&app, &executor);
    wire.width.set(120.);
    handle.refresh().unwrap();
    settle(&app, &executor);
    handle.set_open(true).unwrap();
    app.flush().unwrap();
    settle(&app, &executor);
    wire.defer.set(true);
    handle.refresh().unwrap();
    while executor.poll_one() {}
    assert_eq!(app.stats().queries, 5);
    handle.set_open(false).unwrap();
    app.flush().unwrap();
    assert_eq!(app.stats().queries, 0);
    app.clear().unwrap();
    assert!(executor.is_empty());
}

#[test]
fn unstyled_keeps_measurement_structure_and_independent_explicit_root_overrides() {
    let app = App::memory();
    app.mount(
        overflow_list("plain")
            .unstyled()
            .class("custom-overflow")
            .style(
                glory::style::Style::new()
                    .width(glory::style::Length::Px(320.))
                    .gap(17.),
            )
            .item(item("a")),
    )
    .unwrap();
    let tree = app.tree();
    let root = tree
        .node(nodes(&app, "sapid-overflow-list")[0])
        .unwrap()
        .descriptor
        .view();
    assert!(root.classes.iter().any(|value| value == "custom-overflow"));
    assert_eq!(root.style.width, Some(glory::style::Length::Px(320.)));
    assert_eq!(root.style.gap, Some(17.));
    assert_eq!(root.style.background, None);
    let line = tree
        .node(nodes(&app, "sapid-overflow-line")[0])
        .unwrap()
        .descriptor
        .view();
    assert_eq!(line.style.display, Some(glory::style::Display::Flex));
    let lane = tree
        .node(nodes(&app, "sapid-overflow-measure")[0])
        .unwrap()
        .descriptor
        .view();
    assert!(lane.semantics.as_ref().unwrap().inert);
    assert!(!lane.semantics.as_ref().unwrap().hidden);
    assert_eq!(lane.style.opacity, Some(0.));
}

#[test]
fn rejected_dimensions_and_memberships_retain_the_accepted_widget_without_rewriting_business_sources()
 {
    let app = App::memory();
    let width = app.scope().cage(60.);
    let members = app.scope().cage(vec!["a".to_owned()]);
    app.mount(
        overflow_list("invalid-updates")
            .overflow(false)
            .dynamic_items(members, move |key| item(key).width(width)),
    )
    .unwrap();
    let row = nodes(&app, "sapid-overflow-item")[0];
    let editor = nodes(&app, "editor-a")[0];
    width.revise(|value| *value = f64::NAN);
    assert!(app.flush().is_err());
    assert!(width.get().is_nan());
    assert_eq!(
        app.tree().node(row).unwrap().descriptor.view().style.width,
        Some(glory::style::Length::Px(60.))
    );
    width.revise(|value| *value = 90.);
    app.flush().unwrap();
    assert_eq!(
        app.tree().node(row).unwrap().descriptor.view().style.width,
        Some(glory::style::Length::Px(90.))
    );
    members.revise(|keys| keys.push("a".into()));
    assert!(app.flush().is_err());
    assert_eq!(nodes(&app, "editor-a"), [editor]);
    members.revise(|keys| keys.pop());
    app.flush().unwrap();
    assert_eq!(nodes(&app, "editor-a"), [editor]);
}

#[path = "../examples/web/src/overflow_list_form.rs"]
mod fixture_page;
#[test]
fn deterministic_four_theme_ssr_starts_at_the_same_explicit_prefix_without_physical_queries() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let first = glory::ssr::snapshot(fixture_page::overflow_list_form(theme.clone())).unwrap();
        let second = glory::ssr::snapshot(fixture_page::overflow_list_form(theme)).unwrap();
        assert_eq!(first.html, second.html);
        assert!(first.html.contains("More actions (3 hidden)"));
        assert!(
            first
                .html
                .contains("Visible: 2; hidden: 3; pending: 0; factories: 5")
        );
        assert_eq!(
            first.html.matches("class=\"sapid-overflow-item\"").count(),
            5
        );
        assert!(
            first.html.contains("role=\"dialog\"") && !first.html.contains("role=\"menuitem\"")
        );
    }
}
