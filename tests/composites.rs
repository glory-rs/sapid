use std::cell::{Cell, RefCell};
use std::rc::Rc;

use glory::render::memory::Memory;
use glory::render::{
    Action, Capabilities, Event, FocusMode, Input, Mutation, Navigation, NodeId, Props, Query,
    QueryToken, Receipt, Render, Semantics,
};
use glory::ui::App;
use sapid::{
    ContentPolicy, TabActivation, accordion, accordion_item, collapsible, tab, tabs, text_field,
};

fn find(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().iter().rev().copied().collect::<Vec<_>>();
    let mut result = Vec::new();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            result.push(id);
        }
        pending.extend(node.children.iter().rev().copied());
    }
    result
}
fn semantics(app: &App, id: NodeId) -> Semantics {
    *app.tree()
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .clone()
        .unwrap()
}
fn expanded(app: &App) -> Vec<bool> {
    find(app, "sapid-accordion-trigger")
        .iter()
        .map(|id| semantics(app, *id).expanded.unwrap())
        .collect()
}
fn selected(app: &App) -> Vec<bool> {
    find(app, "sapid-tab")
        .iter()
        .map(|id| semantics(app, *id).selected.unwrap())
        .collect()
}

#[derive(Default)]
struct Trace {
    actions: RefCell<Vec<Action>>,
    inputs: RefCell<Vec<Input>>,
}
struct Recorder {
    memory: Memory,
    trace: Rc<Trace>,
}
impl Render for Recorder {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            focus: true,
            ..Default::default()
        }
    }
    fn style_profile(&self) -> glory::style::Profile {
        glory::style::Profile::RECORD
    }
    fn apply(&mut self, mutations: &[Mutation]) -> Result<Receipt, glory::render::Error> {
        self.memory.apply(mutations)
    }
    fn request(&mut self, query: Query, token: QueryToken) -> Result<(), glory::render::Error> {
        self.memory.request(query, token)
    }
    fn action(&mut self, action: Action) -> Result<(), glory::render::Error> {
        assert!(
            self.memory.active(action.node()),
            "focus must follow the visibility commit"
        );
        self.trace.actions.borrow_mut().push(action);
        Ok(())
    }
    fn drain(&mut self, inputs: &mut Vec<Input>) {
        inputs.extend(self.trace.inputs.borrow_mut().drain(..));
    }
}
fn recording() -> (App, Rc<Trace>) {
    let trace = Rc::new(Trace::default());
    let copy = trace.clone();
    (
        App::new(move |runtime| Recorder {
            memory: Memory::new(runtime),
            trace: copy,
        }),
        trace,
    )
}

#[test]
fn collapsible_has_stable_relations_and_lazy_content_owners() {
    let app = App::memory();
    let calls = Rc::new(Cell::new(0));
    let mounts = calls.clone();
    let reference = app.scope().cage(None);
    app.mount(collapsible("details").label("Details").content(move || {
        mounts.set(mounts.get() + 1);
        text_field().node_ref(reference).default_value("fresh")
    }))
    .unwrap();
    let trigger = find(&app, "sapid-collapsible-trigger")[0];
    let panel = find(&app, "sapid-collapsible-panel")[0];
    assert_eq!(semantics(&app, trigger).controls, semantics(&app, panel).id);
    assert_eq!(
        semantics(&app, panel).labelled_by,
        semantics(&app, trigger).id
    );
    assert_eq!(calls.get(), 0);
    assert!(semantics(&app, panel).hidden);
    app.dispatch(trigger, Event::Activate).unwrap();
    let first = reference.get().unwrap();
    assert_eq!(calls.get(), 1);
    assert!(app.tree().active(first));
    app.dispatch(trigger, Event::Activate).unwrap();
    assert_eq!(reference.get(), None);
    assert!(semantics(&app, panel).hidden);
    app.dispatch(trigger, Event::Activate).unwrap();
    assert_eq!(calls.get(), 2);
    assert_ne!(reference.get().unwrap(), first);
    assert_eq!(find(&app, "sapid-collapsible-panel"), [panel]);
}

#[test]
fn keep_mounted_preserves_input_identity_but_hidden_content_cannot_receive_events() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    app.mount(
        collapsible("kept")
            .content_policy(ContentPolicy::KeepMounted)
            .content(move || text_field().node_ref(reference).default_value("draft")),
    )
    .unwrap();
    let input = reference.get().unwrap();
    assert!(!app.tree().active(input));
    assert!(!app.dispatch(input, Event::Activate).unwrap());
    let trigger = find(&app, "sapid-collapsible-trigger")[0];
    app.dispatch(trigger, Event::Activate).unwrap();
    app.dispatch(trigger, Event::Activate).unwrap();
    app.dispatch(trigger, Event::Activate).unwrap();
    assert_eq!(reference.get(), Some(input));
    assert!(
        matches!(&app.tree().node(input).unwrap().descriptor.view().props.clone(), Props::Input(value) if value.value == "draft")
    );
}

#[test]
fn controlled_disclosure_proposes_without_mutating_and_program_changes_are_silent() {
    let app = App::memory();
    let open = app.scope().cage(false);
    let proposals = Rc::new(RefCell::new(Vec::new()));
    let seen = proposals.clone();
    app.mount(
        collapsible("controlled")
            .open(open)
            .default_open(true)
            .on_open_change(move |value| seen.borrow_mut().push(value)),
    )
    .unwrap();
    let trigger = find(&app, "sapid-collapsible-trigger")[0];
    app.dispatch(trigger, Event::Activate).unwrap();
    assert_eq!(semantics(&app, trigger).expanded, Some(false));
    assert!(!open.get());
    open.revise(|value| *value = true);
    app.flush().unwrap();
    assert_eq!(semantics(&app, trigger).expanded, Some(true));
    assert_eq!(*proposals.borrow(), [true]);
}

#[test]
fn closing_a_focused_panel_queues_focus_after_removal_without_stealing_outside_focus() {
    let (app, trace) = recording();
    let open = app.scope().cage(true);
    let reference = app.scope().cage(None);
    app.mount(
        collapsible("focus")
            .open(open)
            .content(move || text_field().node_ref(reference)),
    )
    .unwrap();
    app.dispatch(reference.get().unwrap(), Event::FocusChanged(true))
        .unwrap();
    open.revise(|value| *value = false);
    app.flush().unwrap();
    let trigger = find(&app, "sapid-collapsible-trigger")[0];
    assert_eq!(*trace.actions.borrow(), [Action::Focus(trigger)]);
    trace.actions.borrow_mut().clear();
    open.revise(|value| *value = true);
    app.flush().unwrap();
    open.revise(|value| *value = false);
    app.flush().unwrap();
    assert!(trace.actions.borrow().is_empty());
}

#[test]
fn accordion_single_multiple_and_required_open_have_distinct_transitions() {
    let app = App::memory();
    app.mount(
        accordion("single")
            .item(accordion_item("a"))
            .item(accordion_item("b")),
    )
    .unwrap();
    let triggers = find(&app, "sapid-accordion-trigger");
    app.dispatch(triggers[0], Event::Activate).unwrap();
    assert_eq!(expanded(&app), [true, false]);
    app.dispatch(triggers[1], Event::Activate).unwrap();
    assert_eq!(expanded(&app), [false, true]);
    app.dispatch(triggers[1], Event::Activate).unwrap();
    assert_eq!(expanded(&app), [false, false]);
    app.clear().unwrap();
    app.mount(
        accordion("multi")
            .multiple(true)
            .item(accordion_item("a"))
            .item(accordion_item("b")),
    )
    .unwrap();
    let triggers = find(&app, "sapid-accordion-trigger");
    app.dispatch(triggers[0], Event::Activate).unwrap();
    app.dispatch(triggers[1], Event::Activate).unwrap();
    assert_eq!(expanded(&app), [true, true]);
    app.clear().unwrap();
    app.mount(
        accordion("required")
            .collapsible(false)
            .item(accordion_item("a"))
            .item(accordion_item("b")),
    )
    .unwrap();
    let triggers = find(&app, "sapid-accordion-trigger");
    assert_eq!(expanded(&app), [true, false]);
    assert!(semantics(&app, triggers[0]).disabled);
    assert!(!app.dispatch(triggers[0], Event::Activate).unwrap());
    app.dispatch(triggers[1], Event::Activate).unwrap();
    assert_eq!(expanded(&app), [false, true]);
}

#[test]
fn accordion_navigation_skips_disabled_members_and_never_opens_a_panel() {
    let (app, trace) = recording();
    app.mount(
        accordion("navigation")
            .item(accordion_item("a"))
            .item(accordion_item("b").enabled(false))
            .item(accordion_item("c")),
    )
    .unwrap();
    let triggers = find(&app, "sapid-accordion-trigger");
    app.dispatch(
        triggers[0],
        Event::Navigate {
            from: triggers[0],
            command: Navigation::Next,
        },
    )
    .unwrap();
    app.dispatch(
        triggers[2],
        Event::Navigate {
            from: triggers[2],
            command: Navigation::Next,
        },
    )
    .unwrap();
    assert_eq!(
        *trace.actions.borrow(),
        [Action::Focus(triggers[2]), Action::Focus(triggers[0])]
    );
    assert_eq!(expanded(&app), [false, false, false]);
    let root = find(&app, "sapid-accordion")[0];
    app.dispatch(
        root,
        Event::Navigate {
            from: root,
            command: Navigation::Next,
        },
    )
    .unwrap();
    assert_eq!(
        trace.actions.borrow().len(),
        2,
        "unknown origins must not fall back to the first item"
    );
    assert!(
        triggers
            .iter()
            .all(|id| app.tree().node(*id).unwrap().descriptor.view().focus == FocusMode::Auto)
    );
}

#[test]
fn invalid_controlled_accordion_state_is_reported_before_child_bindings_and_can_recover() {
    let app = App::memory();
    let source = app.scope().cage(vec!["a".to_string()]);
    app.mount(
        accordion("validation")
            .open(source)
            .item(accordion_item("a"))
            .item(accordion_item("b")),
    )
    .unwrap();
    source.revise(|value| *value = vec!["a".into(), "b".into()]);
    assert!(app.flush().is_err());
    assert_eq!(expanded(&app), [true, false]);
    source.revise(|value| *value = vec!["b".into()]);
    app.flush().unwrap();
    assert_eq!(expanded(&app), [false, true]);
    source.revise(|value| *value = vec!["missing".into()]);
    assert!(app.flush().is_err());
    assert_eq!(expanded(&app), [false, true]);
}

#[test]
fn manual_tabs_keep_focus_and_selection_separate_and_preserve_one_tab_stop() {
    let app = App::memory();
    app.mount(
        tabs("manual")
            .activation(TabActivation::Manual)
            .tab(tab("a"))
            .tab(tab("b")),
    )
    .unwrap();
    let triggers = find(&app, "sapid-tab");
    app.dispatch(triggers[1], Event::FocusChanged(true))
        .unwrap();
    assert_eq!(selected(&app), [true, false]);
    assert_eq!(
        app.tree()
            .node(triggers[0])
            .unwrap()
            .descriptor
            .view()
            .focus,
        FocusMode::Programmatic
    );
    assert_eq!(
        app.tree()
            .node(triggers[1])
            .unwrap()
            .descriptor
            .view()
            .focus,
        FocusMode::TabStop
    );
    app.dispatch(triggers[1], Event::Activate).unwrap();
    assert_eq!(selected(&app), [false, true]);
    assert_eq!(
        find(&app, "sapid-tab-panel")
            .iter()
            .map(|id| semantics(&app, *id).hidden)
            .collect::<Vec<_>>(),
        [true, false]
    );
}

#[test]
fn automatic_tabs_wait_for_actual_focus_and_skip_disabled_navigation_targets() {
    let (app, trace) = recording();
    app.mount(
        tabs("automatic")
            .tab(tab("a"))
            .tab(tab("b").enabled(false))
            .tab(tab("c")),
    )
    .unwrap();
    let triggers = find(&app, "sapid-tab");
    app.dispatch(
        triggers[0],
        Event::Navigate {
            from: triggers[0],
            command: Navigation::Next,
        },
    )
    .unwrap();
    assert_eq!(*trace.actions.borrow(), [Action::Focus(triggers[2])]);
    assert_eq!(
        selected(&app),
        [true, false, false],
        "focus intent is not observed focus"
    );
    app.dispatch(triggers[2], Event::FocusChanged(true))
        .unwrap();
    assert_eq!(selected(&app), [false, false, true]);
}

#[test]
fn rejected_tab_focus_and_activation_in_one_tick_only_propose_once() {
    let (app, trace) = recording();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let proposals = seen.clone();
    app.mount(
        tabs("readonly")
            .value("a")
            .tab(tab("a"))
            .tab(tab("b"))
            .on_value_change(move |value| proposals.borrow_mut().push(value)),
    )
    .unwrap();
    let triggers = find(&app, "sapid-tab");
    trace.inputs.borrow_mut().extend([
        Input::Event {
            node: triggers[1],
            event: Event::FocusChanged(true),
        },
        Input::Event {
            node: triggers[1],
            event: Event::Activate,
        },
    ]);
    app.pump().unwrap();
    assert_eq!(*seen.borrow(), ["b"]);
    assert_eq!(selected(&app), [true, false]);
    app.dispatch(triggers[1], Event::Activate).unwrap();
    assert_eq!(*seen.borrow(), ["b", "b"]);
}

#[test]
fn program_tab_switch_from_inside_a_panel_restores_the_new_selected_trigger() {
    let (app, trace) = recording();
    let source = app.scope().cage("a".to_string());
    let reference = app.scope().cage(None);
    app.mount(
        tabs("restore")
            .value(source)
            .tab(tab("a").content(move || text_field().node_ref(reference)))
            .tab(tab("b")),
    )
    .unwrap();
    app.dispatch(reference.get().unwrap(), Event::FocusChanged(true))
        .unwrap();
    source.revise(|value| *value = "b".into());
    app.flush().unwrap();
    let triggers = find(&app, "sapid-tab");
    assert_eq!(*trace.actions.borrow(), [Action::Focus(triggers[1])]);
    app.dispatch(triggers[1], Event::FocusChanged(true))
        .unwrap();
    assert_eq!(source.get(), "b");
}

#[test]
fn bad_keys_values_and_heading_levels_fail_without_leaving_nodes_or_handlers() {
    let app = App::memory();
    assert!(
        app.mount(tabs("duplicates").tab(tab("a")).tab(tab("a")))
            .is_err()
    );
    assert!(app.mount(tabs("empty")).is_err());
    assert!(
        app.mount(tabs("unknown").value("missing").tab(tab("a")))
            .is_err()
    );
    assert!(
        app.mount(
            accordion("heading")
                .heading_level(7)
                .item(accordion_item("a"))
        )
        .is_err()
    );
    assert!(app.mount(collapsible("bad key")).is_err());
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(app.stats().handlers, 0);
    let before = app.stats().node_slots;
    app.mount(collapsible("same")).unwrap();
    assert!(app.mount(collapsible("same")).is_err());
    assert_eq!(find(&app, "sapid-collapsible").len(), 1);
    assert!(app.stats().node_slots <= before + 4);
}

#[test]
fn composite_teardown_releases_named_members_subscriptions_and_factories() {
    let app = App::memory();
    let baseline = app.runtime_stats();
    let captured = Rc::new(Cell::new(0));
    for _ in 0..50 {
        let copy = captured.clone();
        app.mount(
            tabs("reused")
                .tab(tab("a").content(move || {
                    copy.set(copy.get() + 1);
                    text_field()
                }))
                .tab(tab("b")),
        )
        .unwrap();
        let old = find(&app, "sapid-tab")[0];
        let name = semantics(&app, old).id.unwrap();
        assert_eq!(app.tree().named(&name), Some(old));
        app.clear().unwrap();
        assert_eq!(app.tree().named(&name), None);
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(app.stats().handlers, 0);
        assert_eq!(app.runtime_stats().nodes, baseline.nodes);
        assert_eq!(app.runtime_stats().edges, baseline.edges);
        assert_eq!(app.runtime_stats().owners, baseline.owners);
        assert_eq!(Rc::strong_count(&captured), 1);
    }
    assert!(app.runtime_stats().node_slots < 80);
}

#[test]
fn ssr_relations_are_deterministic_and_namespaces_do_not_alias() {
    let build = || {
        glory::ui::col().fill((
            collapsible("details")
                .label("Details")
                .content(|| glory::ui::text().content("hidden text")),
            accordion("faq").default_open(["a"]).item(
                accordion_item("a")
                    .label("Question")
                    .content(|| glory::ui::text().content("Answer")),
            ),
            tabs("account")
                .name("Account")
                .tab(tab("profile").label("Profile"))
                .tab(tab("security").label("Security")),
        ))
    };
    let first = glory::ssr::render(build()).unwrap();
    assert_eq!(first, glory::ssr::render(build()).unwrap());
    assert!(first.contains("aria-expanded=\"false\""));
    assert!(first.contains("aria-selected=\"true\""));
    assert!(first.contains("role=\"tablist\""));
    assert!(first.contains("role=\"tabpanel\""));
    assert!(first.contains("role=\"heading\""));
    assert!(first.contains("aria-level=\"3\""));
    assert!(
        !first.contains("hidden text"),
        "closed default content should not mount"
    );
    let mut ids = std::collections::HashSet::new();
    for rest in first.split(" id=\"").skip(1) {
        assert!(ids.insert(rest.split('"').next().unwrap()));
    }
    for attribute in ["aria-controls", "aria-labelledby"] {
        for rest in first.split(&format!(" {attribute}=\"")).skip(1) {
            assert!(
                ids.contains(rest.split('"').next().unwrap()),
                "dangling SSR relation"
            );
        }
    }
}

#[test]
fn tab_unmount_policy_releases_inactive_content_and_preserves_panel_shells() {
    let app = App::memory();
    let source = app.scope().cage("a".to_string());
    let reference = app.scope().cage(None);
    app.mount(
        tabs("unmount")
            .value(source)
            .content_policy(ContentPolicy::Unmount)
            .tab(tab("a").content(move || text_field().node_ref(reference)))
            .tab(tab("b")),
    )
    .unwrap();
    let shells = find(&app, "sapid-tab-panel");
    let first = reference.get().unwrap();
    source.revise(|value| *value = "b".into());
    app.flush().unwrap();
    assert_eq!(reference.get(), None);
    source.revise(|value| *value = "a".into());
    app.flush().unwrap();
    assert_ne!(reference.get().unwrap(), first);
    assert_eq!(find(&app, "sapid-tab-panel"), shells);
}

#[test]
fn invalid_tab_program_updates_preserve_valid_selection_and_callbacks_remain_silent() {
    let app = App::memory();
    let source = app.scope().cage("a".to_string());
    app.mount(
        tabs("updates")
            .value(source)
            .tab(tab("a"))
            .tab(tab("b"))
            .on_value_change(|_| panic!("program updates must not emit proposals")),
    )
    .unwrap();
    source.revise(|value| *value = "unknown".into());
    assert!(app.flush().is_err());
    assert_eq!(selected(&app), [true, false]);
    source.revise(|value| *value = "b".into());
    app.flush().unwrap();
    assert_eq!(selected(&app), [false, true]);
}

#[test]
fn non_looping_and_fully_disabled_tabs_emit_no_focus_intents() {
    let (app, trace) = recording();
    app.mount(
        tabs("no-wrap")
            .loop_focus(false)
            .tab(tab("a"))
            .tab(tab("b")),
    )
    .unwrap();
    let triggers = find(&app, "sapid-tab");
    app.dispatch(
        triggers[1],
        Event::Navigate {
            from: triggers[1],
            command: Navigation::Next,
        },
    )
    .unwrap();
    app.dispatch(
        triggers[0],
        Event::Navigate {
            from: triggers[0],
            command: Navigation::Previous,
        },
    )
    .unwrap();
    assert!(trace.actions.borrow().is_empty());
    app.clear().unwrap();
    app.mount(tabs("disabled").enabled(false).tab(tab("a")).tab(tab("b")))
        .unwrap();
    let triggers = find(&app, "sapid-tab");
    assert!(
        triggers
            .iter()
            .all(|id| app.tree().node(*id).unwrap().descriptor.view().focus
                == FocusMode::Programmatic)
    );
    assert!(!app.dispatch(triggers[0], Event::Activate).unwrap());
    app.dispatch(
        triggers[0],
        Event::Navigate {
            from: triggers[0],
            command: Navigation::Next,
        },
    )
    .unwrap();
    assert!(trace.actions.borrow().is_empty());
}

#[test]
fn program_selecting_a_disabled_tab_restores_its_panel_without_proposing_another_selection() {
    let (app, trace) = recording();
    let source = app.scope().cage("a".to_string());
    let reference = app.scope().cage(None);
    app.mount(
        tabs("disabled-restore")
            .value(source)
            .on_value_change(|_| panic!("programmatic focus restoration must not change selection"))
            .tab(tab("a").content(move || text_field().node_ref(reference)))
            .tab(tab("b").enabled(false)),
    )
    .unwrap();
    app.dispatch(reference.get().unwrap(), Event::FocusChanged(true))
        .unwrap();
    source.revise(|value| *value = "b".into());
    app.flush().unwrap();
    let panel = find(&app, "sapid-tab-panel")[1];
    assert_eq!(*trace.actions.borrow(), [Action::Focus(panel)]);
    app.dispatch(panel, Event::FocusChanged(true)).unwrap();
    assert_eq!(source.get(), "b");
}
