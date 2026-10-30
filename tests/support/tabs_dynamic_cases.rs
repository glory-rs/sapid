use glory::render::{
    Action, Capabilities, Direction, Event, EventKind, FocusMode, Input, Mutation, NodeId,
    Orientation, Props, Query, QueryToken, QueryValue, Receipt, Render,
};
use glory::ui::App;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

fn nodes(app: &App, class: &str) -> Vec<NodeId> {
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
fn selected(app: &App) -> Vec<bool> {
    nodes(app, "sapid-tab")
        .into_iter()
        .map(|id| {
            app.tree()
                .node(id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .selected
                .unwrap()
        })
        .collect()
}
fn keys() -> Vec<String> {
    vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
}

#[test]
fn keyed_tabs_share_one_factory_and_preserve_editors_on_reorder_without_history() {
    let app = App::memory();
    let members = app.scope().cage(keys());
    let calls = Rc::new(RefCell::new(HashMap::<String, usize>::new()));
    let factories = calls.clone();
    app.mount(sapid::tabs("dynamic").tabs(members, move |key| {
        *factories.borrow_mut().entry(key.to_owned()).or_default() += 1;
        let class = format!("editor-{key}");
        sapid::tab(key)
            .trigger_content(|| glory::ui::text().content("Rich tab"))
            .content(move || glory::ui::input().class(class.clone()).value("draft"))
    }))
    .unwrap();
    let triggers = nodes(&app, "sapid-tab");
    let editor = nodes(&app, "editor-a")[0];
    let baseline = app.runtime_stats();
    assert_eq!(selected(&app), [true, false, false]);
    members.revise(|keys| keys.reverse());
    app.flush().unwrap();
    assert_eq!(
        nodes(&app, "sapid-tab"),
        [triggers[2], triggers[1], triggers[0]]
    );
    assert_eq!(nodes(&app, "editor-a"), [editor]);
    assert_eq!(selected(&app), [false, false, true]);
    assert!(calls.borrow().values().all(|calls| *calls == 1));
    for index in 0..40 {
        members.revise(|keys| keys.push(format!("temporary-{index}")));
        app.flush().unwrap();
        members.revise(|keys| {
            keys.pop();
        });
        app.flush().unwrap();
        assert_eq!(app.runtime_stats().owners, baseline.owners);
        assert_eq!(app.runtime_stats().nodes, baseline.nodes);
    }
    members.revise(|keys| keys.retain(|key| key != "a"));
    app.flush().unwrap();
    assert!(nodes(&app, "editor-a").is_empty());
    assert_eq!(selected(&app), [true, false]);
    members.revise(|keys| keys.push("a".to_owned()));
    app.flush().unwrap();
    assert_ne!(nodes(&app, "editor-a"), [editor]);
    assert_eq!(calls.borrow()["a"], 2);
    app.clear().unwrap();
    assert!(app.tree().is_empty());
}

#[test]
fn controlled_absence_preserves_intent_and_local_repair_is_silent() {
    let app = App::memory();
    let members = app.scope().cage(keys());
    let value = app.scope().cage("a".to_owned());
    let proposals = app.scope().cage(Vec::<String>::new());
    app.mount(
        sapid::tabs("controlled")
            .value(value)
            .on_value_change(move |value| proposals.revise(|values| values.push(value)))
            .tabs(members, |key| sapid::tab(key)),
    )
    .unwrap();
    members.revise(|keys| {
        keys.remove(0);
    });
    app.flush().unwrap();
    assert_eq!(selected(&app), [false, false]);
    assert_eq!(value.get(), "a");
    assert!(proposals.get().is_empty());
    value.revise(|value| *value = "not-mounted".to_owned());
    app.flush().unwrap();
    assert_eq!(selected(&app), [false, false]);
    value.revise(|value| *value = "b".to_owned());
    app.flush().unwrap();
    assert_eq!(selected(&app), [true, false]);
    let before = nodes(&app, "sapid-tab");
    members.revise(|keys| keys.push("b".to_owned()));
    assert!(app.flush().is_err());
    assert_eq!(nodes(&app, "sapid-tab"), before);
    members.revise(|keys| {
        keys.pop();
    });
    app.flush().unwrap();
    value.revise(|value| *value = "x".repeat(65));
    assert!(app.flush().is_err());
    assert_eq!(selected(&app), [true, false]);
    value.revise(|value| *value = "a".to_owned());
    app.flush().unwrap();
    members.revise(|keys| keys.push("a".to_owned()));
    app.flush().unwrap();
    assert_eq!(selected(&app), [false, false, true]);
    assert!(proposals.get().is_empty());
    members.revise(Vec::clear);
    app.flush().unwrap();
    assert!(nodes(&app, "sapid-tab").is_empty());
    assert!(nodes(&app, "sapid-tab-panel").is_empty());
    members.revise(|members| *members = keys());
    app.flush().unwrap();
    assert_eq!(selected(&app), [true, false, false]);
}

#[test]
fn localized_names_direction_and_current_order_keep_real_roles_and_identity() {
    let app = App::memory();
    let members = app.scope().cage(keys());
    let language = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    let item_enabled = app.scope().cage(false);
    app.mount(
        sapid::tabs("localized")
            .name("文档标签")
            .direction(Direction::Rtl)
            .orientation(Orientation::Vertical)
            .enabled(enabled)
            .tabs(members, move |key| {
                let word = key.to_owned();
                let label = language.map(move |chinese| {
                    if *chinese {
                        format!("章节 {word}")
                    } else {
                        format!("Section {word}")
                    }
                });
                let tab = sapid::tab(key)
                    .label(label)
                    .trigger_content(move || glory::ui::text().content(label));
                if key == "b" {
                    tab.enabled(item_enabled)
                } else {
                    tab
                }
            }),
    )
    .unwrap();
    let triggers = nodes(&app, "sapid-tab");
    let list = nodes(&app, "sapid-tab-list")[0];
    let semantics = app
        .tree()
        .node(list)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .clone()
        .unwrap();
    assert_eq!(semantics.navigation, Some(Orientation::Vertical));
    assert_eq!(semantics.navigation_direction, Direction::Rtl);
    assert_eq!(
        app.tree()
            .node(list)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("文档标签")
    );
    language.revise(|value| *value = true);
    item_enabled.revise(|value| *value = true);
    app.flush().unwrap();
    assert_eq!(nodes(&app, "sapid-tab"), triggers);
    for (index, node) in triggers.iter().enumerate() {
        let tree = app.tree();
        let descriptor = tree.node(*node).unwrap().descriptor.view();
        assert_eq!(
            descriptor.name.as_deref(),
            Some(format!("章节 {}", ["a", "b", "c"][index]).as_str())
        );
        assert!(matches!(descriptor.props, Props::Button(props) if props.label.is_empty()));
        assert!(descriptor.semantics.as_ref().unwrap().tab);
    }
    enabled.revise(|value| *value = false);
    app.flush().unwrap();
    assert!(triggers.iter().all(|id| !app.tree().enabled(*id)));
    let snapshot = glory::ssr::snapshot(sapid::tabs("ssr").direction(Direction::Rtl).tabs(
        vec!["a".to_owned()],
        |key| {
            sapid::tab(key)
                .label("标签 <One>")
                .content(|| glory::ui::text().content("Body"))
        },
    ))
    .unwrap();
    let html = snapshot.html;
    assert!(html.contains("role=\"tab\""));
    assert!(html.contains("role=\"tabpanel\""));
    assert!(html.contains("标签 &lt;One&gt;"));
}

#[test]
fn invalid_member_sources_never_call_factories_or_replace_the_accepted_set() {
    for invalid in [
        vec!["a".to_owned(); 257],
        vec!["a".to_owned(); 2],
        vec!["x".repeat(65)],
    ] {
        let app = App::memory();
        let calls = Rc::new(Cell::new(0));
        let count = calls.clone();
        assert!(
            app.mount(sapid::tabs("invalid").tabs(invalid, move |key| {
                count.set(count.get() + 1);
                sapid::tab(key)
            }))
            .is_err()
        );
        assert_eq!(calls.get(), 0);
        assert!(app.tree().is_empty());
    }
    let app = App::memory();
    let members = app.scope().cage(vec!["a".to_owned()]);
    app.mount(sapid::tabs("failure").tabs(members, |key| {
        sapid::tab(if key == "bad" { "wrong" } else { key })
    }))
    .unwrap();
    let baseline = app.runtime_stats();
    let old = nodes(&app, "sapid-tab");
    members.revise(|keys| keys.extend(["new".to_owned(), "bad".to_owned()]));
    assert!(app.flush().is_err());
    assert_eq!(nodes(&app, "sapid-tab"), old);
    assert_eq!(app.runtime_stats().owners, baseline.owners);
    assert_eq!(app.runtime_stats().nodes, baseline.nodes);
    members.revise(|keys| keys.truncate(1));
    app.flush().unwrap();
}

#[test]
fn manual_automatic_and_generic_callbacks_keep_proposal_order_and_source_silence() {
    for automatic in [false, true] {
        let app = App::memory();
        let value = app.scope().cage("a".to_owned());
        let log = app.scope().cage(Vec::<String>::new());
        app.mount(
            sapid::tabs("activation")
                .value(value)
                .activation(if automatic {
                    sapid::TabActivation::Automatic
                } else {
                    sapid::TabActivation::Manual
                })
                .on_value_change(move |next| log.revise(|log| log.push(format!("typed:{next}"))))
                .on(EventKind::Activate, move |_| {
                    log.revise(|log| log.push("generic".to_owned()))
                })
                .tabs(keys(), |key| sapid::tab(key)),
        )
        .unwrap();
        let tab = nodes(&app, "sapid-tab")[1];
        app.dispatch(tab, Event::FocusChanged(true)).unwrap();
        assert_eq!(log.get(), if automatic { vec!["typed:b"] } else { vec![] });
        log.revise(Vec::clear);
        app.dispatch(tab, Event::Activate).unwrap();
        assert_eq!(log.get(), ["typed:b", "generic"]);
        assert_eq!(selected(&app), [true, false, false]);
        log.revise(Vec::clear);
        value.revise(|value| *value = "c".to_owned());
        app.flush().unwrap();
        assert_eq!(selected(&app), [false, false, true]);
        assert!(log.get().is_empty());
        assert_eq!(
            app.tree().node(tab).unwrap().descriptor.view().focus,
            FocusMode::TabStop
        );
    }
}

#[derive(Default)]
struct Trace {
    inputs: RefCell<Vec<Input>>,
    focus: Cell<Option<NodeId>>,
    accept: Cell<bool>,
}
struct HostRender {
    memory: glory::render::memory::Memory,
    trace: Rc<Trace>,
}
impl Render for HostRender {
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
    fn action(&mut self, action: Action) -> Result<(), glory::render::Error> {
        if let Action::Focus(node) = action
            && self.trace.accept.get()
        {
            if let Some(old) = self.trace.focus.replace(Some(node)) {
                self.trace.inputs.borrow_mut().push(Input::Event {
                    node: old,
                    event: Event::FocusChanged(false),
                });
            }
            self.trace.inputs.borrow_mut().push(Input::Event {
                node,
                event: Event::FocusChanged(true),
            });
        }
        Ok(())
    }
    fn request(&mut self, query: Query, token: QueryToken) -> Result<(), glory::render::Error> {
        self.trace.inputs.borrow_mut().push(Input::Response {
            token,
            applied: self.memory.receipt().applied,
            result: Ok(QueryValue::Focus(
                self.trace.focus.get() == Some(query.node()),
            )),
        });
        Ok(())
    }
    fn drain(&mut self, out: &mut Vec<Input>) {
        out.extend(self.trace.inputs.take());
    }
}

#[test]
fn failed_focus_restore_expires_at_host_observation_and_later_user_focus_activates_normally() {
    for accept in [false, true] {
        let executor = Rc::new(glory::core::executor::Executor::new(8));
        let trace = Rc::new(Trace::default());
        trace.accept.set(accept);
        let output = trace.clone();
        let app = App::with_host(executor.clone(), move |runtime| HostRender {
            memory: glory::render::memory::Memory::new(runtime),
            trace: output,
        });
        let members = app.scope().cage(keys());
        let value = app.scope().cage("a".to_owned());
        let proposals = app.scope().cage(Vec::<String>::new());
        app.mount(
            sapid::tabs("focus")
                .value(value)
                .on_value_change(move |next| proposals.revise(|values| values.push(next)))
                .tabs(members, |key| sapid::tab(key)),
        )
        .unwrap();
        let old = nodes(&app, "sapid-tab")[0];
        trace.focus.set(Some(old));
        app.dispatch(old, Event::FocusChanged(true)).unwrap();
        proposals.revise(Vec::clear);
        members.revise(|members| {
            members.remove(0);
        });
        app.flush().unwrap();
        assert!(proposals.get().is_empty());
        while executor.poll_one() {}
        app.pump().unwrap();
        assert_eq!(app.stats().queries, 0);
        assert!(
            proposals.get().is_empty(),
            "membership restoration must not auto-select a new controlled value"
        );
        assert_eq!(value.get(), "a");
        let next = nodes(&app, "sapid-tab")[0];
        app.dispatch(next, Event::FocusChanged(false)).unwrap();
        app.dispatch(next, Event::FocusChanged(true)).unwrap();
        assert_eq!(proposals.get(), ["b"]);
        assert_eq!(selected(&app), [false, false]);
        assert!(executor.is_empty());
        app.clear().unwrap();
        assert_eq!(app.stats().queries, 0);
    }
}

#[test]
fn focus_confirmation_is_current_only_and_cancels_on_other_focus_target_retirement_or_owner() {
    for scenario in ["other-focus", "target-retirement", "owner-retirement"] {
        let executor = Rc::new(glory::core::executor::Executor::new(8));
        let trace = Rc::new(Trace::default());
        let output = trace.clone();
        let app = App::with_host(executor.clone(), move |runtime| HostRender {
            memory: glory::render::memory::Memory::new(runtime),
            trace: output,
        });
        let members = app.scope().cage(keys());
        let source = app.scope().cage("a".to_owned());
        let proposals = app.scope().cage(Vec::<String>::new());
        app.mount(
            sapid::tabs("cancel-focus")
                .value(source)
                .on_value_change(move |next| proposals.revise(|values| values.push(next)))
                .tabs(members, |key| sapid::tab(key)),
        )
        .unwrap();
        let old = nodes(&app, "sapid-tab")[0];
        trace.focus.set(Some(old));
        app.dispatch(old, Event::FocusChanged(true)).unwrap();
        members.revise(|members| {
            members.remove(0);
        });
        app.flush().unwrap();
        assert_eq!(app.runtime_stats().tasks, 1);
        while executor.poll_one() {}
        assert_eq!(app.stats().queries, 1);
        assert_eq!(app.runtime_stats().tasks, 0);
        match scenario {
            "other-focus" => {
                app.dispatch(nodes(&app, "sapid-tab")[1], Event::FocusChanged(true))
                    .unwrap();
                assert_eq!(proposals.get(), ["c"]);
            }
            "target-retirement" => {
                members.revise(|members| {
                    members.remove(0);
                });
                app.flush().unwrap();
                assert!(proposals.get().is_empty());
            }
            "owner-retirement" => app.clear().unwrap(),
            _ => unreachable!(),
        }
        assert_eq!(app.stats().queries, 0);
        app.pump().unwrap();
        assert_eq!(app.stats().queries, 0);
        assert_eq!(app.runtime_stats().tasks, 0);
        assert!(executor.is_empty());
        assert_eq!(source.get(), "a");
    }
}

#[path = "../../examples/web/src/tabs_dynamic_form.rs"]
mod fixture;

#[test]
fn dynamic_tabs_fixture_has_deterministic_four_theme_semantics_and_rich_labels() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let first = glory::ssr::snapshot(fixture::tabs_dynamic_form(theme.clone())).unwrap();
        let second = glory::ssr::snapshot(fixture::tabs_dynamic_form(theme)).unwrap();
        assert_eq!(first.html, second.html);
        assert_eq!(first.html.matches("role=\"tab\"").count(), 6);
        assert_eq!(first.html.matches("role=\"tablist\"").count(), 2);
        assert_eq!(first.html.matches("role=\"tabpanel\"").count(), 6);
        assert!(
            first.html.contains("Document sections") && first.html.contains("dynamic-tab-caption")
        );
    }
}

#[test]
fn unstyled_dynamic_tabs_keep_structural_layout_roles_and_explicit_overrides() {
    use glory::style::{Display, FlexDirection, Style};
    let app = App::memory();
    app.mount(
        sapid::tabs("unstyled")
            .unstyled()
            .class("custom-tabs")
            .style(Style::new().gap(23.))
            .orientation(Orientation::Vertical)
            .tabs(vec!["a".to_owned()], |key| {
                sapid::tab(key).content(|| glory::ui::text().content("Body"))
            }),
    )
    .unwrap();
    let root = nodes(&app, "sapid-tabs")[0];
    let list = nodes(&app, "sapid-tab-list")[0];
    let trigger = nodes(&app, "sapid-tab")[0];
    let panel = nodes(&app, "sapid-tab-panel")[0];
    let tree = app.tree();
    let root = tree.node(root).unwrap().descriptor.view();
    assert!(root.classes.iter().any(|class| class == "custom-tabs"));
    assert_eq!(root.style.gap, Some(23.));
    assert_eq!(root.style.display, None);
    let list = tree.node(list).unwrap().descriptor.view();
    assert_eq!(list.style.display, Some(Display::Flex));
    assert_eq!(list.style.flex_direction, Some(FlexDirection::Column));
    assert_eq!(
        tree.node(trigger).unwrap().descriptor.view().style,
        &Style::new()
    );
    assert_eq!(
        tree.node(panel).unwrap().descriptor.view().style,
        &Style::new()
    );
}

#[test]
fn unmount_policy_retires_only_body_owners_and_keeps_current_panel_shell_relations() {
    let app = App::memory();
    let members = app.scope().cage(keys());
    let source = app.scope().cage("a".to_owned());
    app.mount(
        sapid::tabs("unmount")
            .value(source)
            .content_policy(sapid::ContentPolicy::Unmount)
            .tabs(members, |key| {
                let class = format!("body-{key}");
                sapid::tab(key).content(move || glory::ui::input().class(class.clone()))
            }),
    )
    .unwrap();
    let shells = nodes(&app, "sapid-tab-panel");
    let old = nodes(&app, "body-a")[0];
    assert!(nodes(&app, "body-b").is_empty());
    source.revise(|value| *value = "b".to_owned());
    app.flush().unwrap();
    assert_eq!(nodes(&app, "sapid-tab-panel"), shells);
    assert!(nodes(&app, "body-a").is_empty());
    assert_eq!(nodes(&app, "body-b").len(), 1);
    source.revise(|value| *value = "a".to_owned());
    app.flush().unwrap();
    assert_ne!(nodes(&app, "body-a"), [old]);
    source.revise(|value| *value = "absent".to_owned());
    app.flush().unwrap();
    assert_eq!(nodes(&app, "sapid-tab-panel"), shells);
    assert!(nodes(&app, "body-a").is_empty() && nodes(&app, "body-b").is_empty());
    assert_eq!(selected(&app), [false, false, false]);
}
