use glory::ui::App;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};
#[path = "../examples/web/src/accordion_form.rs"]
mod fixture;

fn nodes(app: &App, class: &str) -> Vec<glory::render::NodeId> {
    let tree = app.tree();
    let mut pending: Vec<_> = tree.roots().iter().rev().copied().collect();
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
fn labels(app: &App) -> Vec<String> {
    nodes(app, "sapid-accordion-trigger")
        .into_iter()
        .map(
            |id| match app.tree().node(id).unwrap().descriptor.view().props {
                glory::render::Props::Button(button) => button.label.clone(),
                _ => unreachable!(),
            },
        )
        .collect()
}

struct Capture {
    drops: Rc<Cell<usize>>,
    source: glory::Cage<usize>,
}
impl Drop for Capture {
    fn drop(&mut self) {
        // User captures may reenter the graph when an excess item is dropped.
        assert_eq!(self.source.try_get().unwrap(), 7);
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn static_composite_factories_are_bounded_and_excess_captures_drop_outside_borrows() {
    let app = App::memory();
    let source = app.scope().cage(7usize);
    let tab_drops = Rc::new(Cell::new(0));
    let accordion_drops = Rc::new(Cell::new(0));
    let mut tabs = sapid::tabs("bounded-tabs");
    let mut accordion = sapid::accordion("bounded-accordion");
    for index in 0..1000 {
        let capture = Capture {
            drops: tab_drops.clone(),
            source,
        };
        tabs = tabs.tab(sapid::tab(format!("item-{index}")).content(move || {
            let _capture = &capture;
            glory::ui::text().content("Panel")
        }));
        let capture = Capture {
            drops: accordion_drops.clone(),
            source,
        };
        accordion = accordion.item(sapid::accordion_item(format!("item-{index}")).content(
            move || {
                let _capture = &capture;
                glory::ui::text().content("Panel")
            },
        ));
    }
    assert_eq!(tab_drops.get(), 743);
    assert_eq!(accordion_drops.get(), 743);
    assert!(app.mount(tabs).is_err());
    assert!(app.mount(accordion).is_err());
    assert_eq!(tab_drops.get(), 1000);
    assert_eq!(accordion_drops.get(), 1000);
    assert!(app.tree().is_empty());
}

#[test]
fn accordion_default_open_consumes_only_the_budget_plus_one_sentinel() {
    let consumed = Rc::new(Cell::new(0usize));
    let count = consumed.clone();
    let keys = (0..1_000_000).map(move |index| {
        count.set(count.get() + 1);
        format!("item-{index}")
    });
    let widget = sapid::accordion("bounded-open")
        .item(sapid::accordion_item("item-0"))
        .default_open(keys);
    assert_eq!(consumed.get(), 257);
    let app = App::memory();
    assert!(app.mount(widget).is_err());
    assert!(app.tree().is_empty());
}

#[test]
fn dynamic_accordion_reorders_live_members_without_rebuilding_their_owners_or_editors() {
    let app = App::memory();
    let baseline_owners = app.runtime_stats().owners;
    let keys = app.scope().cage(vec!["a".to_owned(), "b".to_owned()]);
    let calls = Rc::new(RefCell::new(HashMap::<String, usize>::new()));
    let count = calls.clone();
    app.mount(
        sapid::accordion("dynamic")
            .content_policy(sapid::ContentPolicy::KeepMounted)
            .items(keys, move |key| {
                *count.borrow_mut().entry(key.to_owned()).or_default() += 1;
                let class = format!("editor-{key}");
                sapid::accordion_item(key)
                    .label(key.to_uppercase())
                    .content(move || glory::ui::input().class(class.clone()))
            }),
    )
    .unwrap();
    let a = nodes(&app, "editor-a")[0];
    let b = nodes(&app, "editor-b")[0];
    assert_eq!(labels(&app), ["A", "B"]);
    keys.revise(|keys| keys.reverse());
    app.flush().unwrap();
    assert_eq!(labels(&app), ["B", "A"]);
    assert_eq!(nodes(&app, "editor-a"), [a]);
    assert_eq!(nodes(&app, "editor-b"), [b]);
    assert_eq!(calls.borrow().values().sum::<usize>(), 2);
    keys.revise(|keys| keys.retain(|key| key == "b"));
    app.flush().unwrap();
    assert!(app.tree().node(a).is_err());
    let owners = app.runtime_stats().owners;
    let graph_nodes = app.runtime_stats().nodes;
    for index in 0..40 {
        keys.revise(|keys| keys.push(format!("temporary-{index}")));
        app.flush().unwrap();
        keys.revise(|keys| {
            keys.pop();
        });
        app.flush().unwrap();
        assert_eq!(app.runtime_stats().owners, owners);
        assert_eq!(app.runtime_stats().nodes, graph_nodes);
    }
    keys.revise(|keys| keys.push("a".to_owned()));
    app.flush().unwrap();
    assert_ne!(nodes(&app, "editor-a"), [a]);
    assert_eq!(nodes(&app, "editor-b"), [b]);
    assert_eq!(calls.borrow()["a"], 2);
    app.clear().unwrap();
    assert_eq!(app.runtime_stats().owners, baseline_owners);
    assert_eq!(app.stats().handlers, 0);
}

#[test]
fn dynamic_accordion_failed_configuration_and_reentrant_factory_publish_no_partial_members() {
    let app = App::memory();
    let keys = app.scope().cage(vec!["a".to_owned()]);
    let drops = Rc::new(Cell::new(0));
    let drop_count = drops.clone();
    let source = app.scope().cage(7usize);
    app.mount(
        sapid::accordion("transactional")
            .content_policy(sapid::ContentPolicy::KeepMounted)
            .items(keys, move |key| {
                if key == "bad" {
                    return sapid::accordion_item("mismatch");
                }
                if key == "reenter" {
                    keys.revise(|keys| keys.push("late".to_owned()));
                }
                let capture = Capture {
                    drops: drop_count.clone(),
                    source,
                };
                sapid::accordion_item(key).content(move || {
                    let _capture = &capture;
                    glory::ui::text().content("Content")
                })
            }),
    )
    .unwrap();
    let owners = app.runtime_stats().owners;
    let mounted = app.stats().nodes;
    keys.revise(|keys| keys.extend(["fresh".to_owned(), "bad".to_owned()]));
    assert!(app.flush().is_err());
    assert_eq!(labels(&app), ["a"]);
    assert_eq!(app.runtime_stats().owners, owners);
    assert_eq!(app.stats().nodes, mounted);
    assert_eq!(drops.get(), 1);
    keys.revise(|keys| *keys = vec!["a".to_owned(), "reenter".to_owned()]);
    app.flush().unwrap();
    assert_eq!(labels(&app), ["a", "reenter", "late"]);
    app.clear().unwrap();
    assert_eq!(drops.get(), 4);
}

fn open_states(app: &App) -> Vec<bool> {
    nodes(app, "sapid-accordion-trigger")
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
                .expanded
                .unwrap()
        })
        .collect()
}

#[test]
fn dynamic_selection_repairs_local_removal_and_preserves_controlled_absent_intent_silently() {
    use glory::render::Event;
    let app = App::memory();
    let keys = app.scope().cage(vec!["a".to_owned(), "b".to_owned()]);
    let calls = app.scope().cage(0usize);
    app.mount(
        sapid::accordion("required-dynamic")
            .collapsible(false)
            .on_open_change(move |_| calls.revise(|calls| *calls += 1))
            .items(keys, |key| sapid::accordion_item(key.to_owned())),
    )
    .unwrap();
    assert_eq!(open_states(&app), [true, false]);
    let b = nodes(&app, "sapid-accordion-trigger")[1];
    app.dispatch(b, Event::Activate).unwrap();
    assert_eq!(open_states(&app), [false, true]);
    keys.revise(|keys| {
        keys.pop();
    });
    app.flush().unwrap();
    assert_eq!(open_states(&app), [true]);
    assert_eq!(calls.get(), 1);
    app.clear().unwrap();

    let open = app.scope().cage(vec!["a".to_owned()]);
    app.mount(
        sapid::accordion("controlled-dynamic")
            .open(open)
            .on_open_change(move |_| calls.revise(|calls| *calls += 1))
            .items(keys, |key| sapid::accordion_item(key.to_owned())),
    )
    .unwrap();
    assert_eq!(open_states(&app), [true]);
    keys.revise(|keys| *keys = vec!["b".to_owned()]);
    app.flush().unwrap();
    assert_eq!(open_states(&app), [false]);
    assert_eq!(open.get(), ["a"]);
    keys.revise(|keys| keys.push("a".to_owned()));
    app.flush().unwrap();
    assert_eq!(open_states(&app), [false, true]);
    assert_eq!(calls.get(), 1);
    let before = nodes(&app, "sapid-accordion-trigger");
    keys.revise(|keys| keys.push("a".to_owned()));
    assert!(app.flush().is_err());
    assert_eq!(nodes(&app, "sapid-accordion-trigger"), before);
    keys.revise(|keys| {
        keys.pop();
    });
    app.flush().unwrap();
    open.revise(|open| *open = vec!["x".to_owned(); 257]);
    assert!(app.flush().is_err());
    assert_eq!(open_states(&app), [false, true]);
}

#[test]
fn ordered_collection_traverses_current_indices_and_does_not_recycle_retired_registration() {
    use glory::ui::Widget;
    let app = App::memory();
    let collection =
        sapid::Collection::new(app.scope(), sapid::CollectionLimits::default()).unwrap();
    let mut scopes = Vec::new();
    let mut tokens = Vec::new();
    let mut positions = Vec::new();
    for (index, key) in ["a", "b", "c"].into_iter().enumerate() {
        let scope = app.scope().child().unwrap();
        let reference = scope.cage(None);
        let position = scope.cage(index);
        glory::ui::button()
            .node_ref(reference)
            .show_in(&scope)
            .unwrap();
        tokens.push(
            collection
                .register_ordered(&scope, key, reference, true, key, position)
                .unwrap(),
        );
        positions.push(position);
        scopes.push(scope);
    }
    assert_eq!(
        collection
            .mounted()
            .unwrap()
            .into_iter()
            .map(|item| item.key)
            .collect::<Vec<_>>(),
        ["a", "b", "c"]
    );
    positions[0].revise(|value| *value = 2);
    positions[1].revise(|value| *value = 0);
    positions[2].revise(|value| *value = 1);
    let mounted = collection.mounted().unwrap();
    assert_eq!(
        mounted
            .iter()
            .map(|item| item.key.as_str())
            .collect::<Vec<_>>(),
        ["b", "c", "a"]
    );
    assert_eq!(mounted[0].member, tokens[1]);
    let roving = sapid::RovingFocus::new(app.scope(), collection.clone(), true).unwrap();
    assert_eq!(
        roving
            .destination(mounted[0].node, glory::render::Navigation::Next)
            .unwrap()
            .unwrap()
            .key,
        "c"
    );
    scopes.remove(1).dispose();
    assert_eq!(collection.registered_len().unwrap(), 2);
    assert!(
        !collection
            .mounted()
            .unwrap()
            .iter()
            .any(|item| item.member == tokens[1])
    );
}

#[test]
fn rich_trigger_slots_keep_intrinsic_semantics_names_and_independent_body_lifetimes() {
    use glory::render::{Event, Props};
    let app = App::memory();
    let caption = app.scope().cage("Visible summary".to_owned());
    let trigger = app.scope().cage(None);
    app.mount(
        sapid::collapsible("rich-collapse")
            .label("Localized disclosure name")
            .trigger_ref(trigger)
            .trigger_content(move || glory::ui::text().class("rich-caption").content(caption))
            .content(|| glory::ui::input().class("independent-body")),
    )
    .unwrap();
    let button = trigger.get().unwrap();
    let text = nodes(&app, "rich-caption")[0];
    assert!(
        matches!(app.tree().node(button).unwrap().descriptor.view().props, Props::Button(props) if props.label.is_empty())
    );
    assert_eq!(
        app.tree()
            .node(button)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Localized disclosure name")
    );
    assert!(nodes(&app, "independent-body").is_empty());
    caption.revise(|caption| *caption = "修改摘要".to_owned());
    app.flush().unwrap();
    assert_eq!(nodes(&app, "rich-caption"), [text]);
    app.dispatch(button, Event::Activate).unwrap();
    let first = nodes(&app, "independent-body")[0];
    app.dispatch(button, Event::Activate).unwrap();
    assert!(app.tree().node(first).is_err());
    assert_eq!(trigger.get(), Some(button));
    assert_eq!(nodes(&app, "rich-caption"), [text]);
    app.clear().unwrap();
    assert!(trigger.get().is_none());

    let snapshot = glory::ssr::snapshot(
        glory::ui::view().fill((
            sapid::accordion("rich-accordion").item(
                sapid::accordion_item("one")
                    .label("Section name")
                    .trigger_content(|| glory::ui::text().content("<Summary>")),
            ),
            sapid::tabs("rich-tabs").tab(
                sapid::tab("one")
                    .label("Tab name")
                    .trigger_content(|| glory::ui::text().content("<Tab>")),
            ),
        )),
    )
    .unwrap();
    assert!(snapshot.html.contains("aria-label=\"Section name\""));
    assert!(snapshot.html.contains("aria-label=\"Tab name\""));
    assert!(snapshot.html.contains("&lt;Summary&gt;") && snapshot.html.contains("&lt;Tab&gt;"));
    assert!(snapshot.html.contains("role=\"tab\""));
    assert!(
        app.mount(sapid::collapsible("invalid-rich").trigger_content(glory::ui::input))
            .is_err()
    );
}

#[test]
fn dynamic_accordion_fixture_has_deterministic_rich_relations_and_four_theme_ssr() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let first = glory::ssr::snapshot(fixture::accordion_form(theme.clone())).unwrap();
        let second = glory::ssr::snapshot(fixture::accordion_form(theme)).unwrap();
        assert_eq!(first.html, second.html);
        for expected in [
            "Section alpha",
            "Section beta",
            "Section gamma",
            "Details: ",
            "aria-expanded=\"true\"",
            "aria-controls=",
            "aria-labelledby=",
            "role=\"region\"",
            "Proposals: 0; factories: 3; order:",
        ] {
            assert!(first.html.contains(expected), "missing {expected}");
        }
    }
}
