#![cfg(feature = "json")]
use glory::render::{DocumentPart, Event, NodeId, Props};
use glory::ui::App;
use sapid::*;
use std::{cell::RefCell, rc::Rc};
fn ids(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = todo.pop() {
        let n = tree.node(id).unwrap();
        if n.descriptor.view().classes.iter().any(|c| c == class) {
            out.push(id)
        }
        todo.extend(n.children.iter().rev());
    }
    out
}
fn trigger(app: &App, label: &str) -> NodeId {
    ids(app,"sapid-collapsible-trigger").into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(p)if p.label==label)).unwrap_or_else(||panic!("missing {label}"))
}
fn values(app: &App) -> Vec<String> {
    ids(app, "sapid-json-value")
        .into_iter()
        .map(|id| {
            let tree = app.tree();
            let Props::Text(text) = tree.node(id).unwrap().descriptor.view().props else {
                panic!()
            };
            text.content.clone()
        })
        .collect()
}
#[test]
fn model_preserves_numbers_escapes_pointer_identity_and_preorder() {
    let s = "{\"a/b~c\": [123456789012345678901234567890, 1e400, \"😀\\u0026\"], \"\": null}";
    let tree = parse_json_tree(s, JsonViewLimits::default()).unwrap();
    assert_eq!(tree.source(), s);
    assert_eq!(tree.nodes().len(), 6);
    assert_eq!(tree.nodes()[1].path, "/a~1b~0c");
    assert_eq!(tree.nodes()[5].path, "/");
    assert_eq!(tree.value(2), Some("123456789012345678901234567890"));
    assert_eq!(tree.value(3), Some("1e400"));
    assert_eq!(tree.nodes()[4].parent, Some(1));
    assert_eq!(tree.nodes()[4].depth, 2);
    assert_eq!(tree.search("😀").unwrap(), vec![4]);
    assert_eq!(tree.search("a/b").unwrap(), vec![1]);
    assert!(tree.search("").unwrap().is_empty());
    assert_eq!(tree.search(&"x".repeat(257)), Err(JsonViewError::Search));
    assert_eq!(
        parse_json_tree("{\"a\":1,\"\\u0061\":2}", JsonViewLimits::default()),
        Err(JsonViewError::DuplicateKey)
    );
}
#[test]
fn model_budgets_and_syntax_are_rejected_before_ui() {
    for s in ["", " ", "[1,]", "{bad}"] {
        assert!(parse_json_tree(s, JsonViewLimits::default()).is_err())
    }
    assert_eq!(
        parse_json_tree(
            "[]",
            JsonViewLimits {
                nodes: 0,
                ..Default::default()
            }
        ),
        Err(JsonViewError::Configuration)
    );
    assert_eq!(
        parse_json_tree(
            "[1,2]",
            JsonViewLimits {
                nodes: 2,
                ..Default::default()
            }
        ),
        Err(JsonViewError::Nodes)
    );
    assert!(
        parse_json_tree(
            "[1,2]",
            JsonViewLimits {
                nodes: 3,
                ..Default::default()
            }
        )
        .is_ok()
    );
    assert!(
        parse_json_tree(
            "[[[0]]]",
            JsonViewLimits {
                depth: 2,
                ..Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(
        parse_json_tree(
            &format!("{{\"{}\":0}}", "a".repeat(257)),
            JsonViewLimits::default()
        ),
        Err(JsonViewError::Key)
    );
    let mut source = "0".to_string();
    for _ in 0..10 {
        source = format!("{{\"{}\":{source}}}", "a".repeat(256));
    }
    assert_eq!(
        parse_json_tree(&source, JsonViewLimits::default()),
        Err(JsonViewError::Path)
    );
}
#[test]
fn genuine_disclosures_mount_only_open_nodes_and_controlled_rejection_is_silent() {
    let app = App::memory();
    let paths = app.scope().cage(vec![String::new()]);
    let proposals = Rc::new(RefCell::new(vec![]));
    let receive = proposals.clone();
    app.mount(
        json_viewer("j")
            .source("{\"branch\":{\"leaf\":1},\"other\":2}")
            .expanded(paths)
            .on_expanded_change(move |next| receive.borrow_mut().push(next)),
    )
    .unwrap();
    assert_eq!(values(&app), vec!["other: 2"]);
    let branch = trigger(&app, "branch: Object 1");
    app.dispatch(branch, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(
        &*proposals.borrow(),
        &[vec![String::new(), "/branch".into()]]
    );
    assert_eq!(values(&app), vec!["other: 2"]);
    paths.revise(|paths| paths.push("/branch".into()));
    app.flush().unwrap();
    assert_eq!(values(&app), vec!["leaf: 1", "other: 2"]);
    assert_eq!(proposals.borrow().len(), 1);
    let tree = app.tree();
    for id in ids(&app, "sapid-json-node") {
        assert_eq!(
            tree.node(id).unwrap().descriptor.view().document_part(),
            Some(DocumentPart::ListItem)
        );
        assert_eq!(tree.node(id).unwrap().descriptor.view().role, None);
    }
}
#[test]
fn query_reveals_ancestors_without_mutating_expanded_source_or_firing_callbacks() {
    let app = App::memory();
    let query = app.scope().cage(String::new());
    let paths = app.scope().cage(Vec::<String>::new());
    let proposals = Rc::new(RefCell::new(vec![]));
    let output = proposals.clone();
    app.mount(
        json_viewer("search")
            .source("{\"a\":{\"b\":{\"needle\":\"😀\"}},\"c\":0}")
            .expanded(paths)
            .search(query)
            .on_expanded_change(move |next| output.borrow_mut().push(next)),
    )
    .unwrap();
    assert!(values(&app).is_empty());
    query.revise(|q| *q = "needle".into());
    app.flush().unwrap();
    assert_eq!(values(&app), vec!["needle: \"😀\"", "c: 0"]);
    assert!(paths.get().is_empty());
    assert!(proposals.borrow().is_empty());
    for id in ids(&app, "sapid-collapsible-trigger") {
        assert!(
            matches!(app.tree().node(id).unwrap().descriptor.view().props,Props::Button(button) if !button.enabled)
        )
    }
    query.revise(|q| q.clear());
    app.flush().unwrap();
    assert!(values(&app).is_empty());
}
#[test]
fn reactive_json_reorders_retained_paths_and_recovers_after_invalid_source() {
    let app = App::memory();
    let source = app.scope().cage("{\"a\":1,\"b\":2}".to_string());
    app.mount(json_viewer("data").source(source)).unwrap();
    let a = ids(&app, "sapid-json-value")[0];
    source.revise(|s| *s = "{\"b\":3,\"a\":4}".into());
    app.flush().unwrap();
    assert_eq!(values(&app), vec!["b: 3", "a: 4"]);
    assert_eq!(ids(&app, "sapid-json-value")[1], a);
    source.revise(|s| *s = "{bad}".into());
    assert!(app.flush().is_err());
    assert_eq!(values(&app), vec!["b: 3", "a: 4"]);
    assert_eq!(source.get(), "{bad}");
    source.revise(|s| *s = "{\"a\":5}".into());
    app.flush().unwrap();
    assert_eq!(values(&app), vec!["a: 5"]);
    assert_eq!(ids(&app, "sapid-json-value")[0], a);
}
#[test]
fn local_expansion_retirement_and_bounded_iterators() {
    use std::cell::Cell;
    let app = App::memory();
    let mounted = app
        .scope()
        .mount_with(|| json_viewer("local").source("{\"a\":[1,2]}"))
        .unwrap();
    app.flush().unwrap();
    let baseline = app.runtime_stats().nodes;
    for _ in 0..8 {
        app.dispatch(trigger(&app, "a: Array 2"), Event::Activate)
            .unwrap();
        app.flush().unwrap();
        assert_eq!(values(&app), vec!["0: 1", "1: 2"]);
        app.dispatch(trigger(&app, "a: Array 2"), Event::Activate)
            .unwrap();
        app.flush().unwrap();
        assert!(values(&app).is_empty());
        assert_eq!(app.runtime_stats().nodes, baseline);
    }
    drop(mounted);
    app.flush().unwrap();
    assert!(ids(&app, "sapid-json-node").is_empty());
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    let invalid = json_viewer("budget").default_expanded((0..1_000_000).map(move |_| {
        count.set(count.get() + 1);
        String::new()
    }));
    assert_eq!(calls.get(), MAX_JSON_VIEW_NODES + 1);
    assert!(app.mount(invalid).is_err());
}
#[test]
fn four_theme_ssr_uses_lists_buttons_escaped_json_and_unstyled_tokens() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let html = glory::ssr::render(
            json_viewer("ssr")
                .theme(theme)
                .source("{\"a\":\"<script>😀\"}"),
        )
        .unwrap();
        assert!(html.contains("<ul"));
        assert!(html.contains("<li"));
        assert!(html.contains("<button"));
        assert!(!html.contains("role=\"tree"));
        assert!(html.contains("&lt;script&gt;"));
    }
    let app = App::memory();
    app.mount(json_viewer("unstyled").unstyled().source("[1]"))
        .unwrap();
    assert_eq!(
        app.tree()
            .node(ids(&app, "sapid-json-viewer")[0])
            .unwrap()
            .descriptor
            .view()
            .style,
        &glory::style::Style::new()
    );
}
#[test]
fn disclosure_internal_and_typed_callbacks_precede_generic_activate_observers() {
    let app = App::memory();
    let log = Rc::new(RefCell::new(Vec::new()));
    let typed = log.clone();
    let generic = log.clone();
    app.mount(
        json_viewer("observe")
            .source("{\"branch\":{\"leaf\":1}}")
            .on_expanded_change(move |_| typed.borrow_mut().push("typed"))
            .on(glory::render::EventKind::Activate, move |_| {
                generic.borrow_mut().push("generic")
            }),
    )
    .unwrap();
    app.dispatch(trigger(&app, "branch: Object 1"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert_eq!(&*log.borrow(), &["typed", "generic"]);
    assert_eq!(values(&app), vec!["leaf: 1"]);
}
#[test]
fn reusable_tree_mode_has_real_coordinates_controlled_expansion_search_and_retained_paths() {
    use glory::render::{Role, TreeCommand};
    let app = App::memory();
    let source = app
        .scope()
        .cage("{\"😀/a~\":{\"leaf\":1},\"version\":2}".to_string());
    let expanded = app
        .scope()
        .cage(vec![String::new(), "/future".into(), "/version".into()]);
    let query = app.scope().cage(String::new());
    let proposals = Rc::new(RefCell::new(vec![]));
    let output = proposals.clone();
    app.mount(
        json_viewer("tree-json")
            .source(source)
            .tree_navigation(true)
            .expanded(expanded)
            .search(query)
            .on_expanded_change(move |paths| output.borrow_mut().push(paths)),
    )
    .unwrap();
    let find = |name: &str| {
        ids(&app, "sapid-tree-item")
            .into_iter()
            .find(|id| {
                app.tree()
                    .node(*id)
                    .unwrap()
                    .descriptor
                    .view()
                    .name
                    .as_deref()
                    == Some(name)
            })
            .unwrap()
    };
    let root = ids(&app, "sapid-json-tree")[0];
    let branch = find("😀/a~: Object");
    assert_eq!(
        app.tree().node(root).unwrap().descriptor.view().role,
        Some(Role::Tree)
    );
    assert_eq!(
        app.tree()
            .node(branch)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .tree_item
            .unwrap()
            .level,
        2
    );
    app.dispatch(
        root,
        Event::TreeNavigate {
            from: branch,
            command: TreeCommand::Open,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(proposals.borrow().len(), 1);
    assert!(proposals.borrow()[0].contains(&"/😀~1a~0".into()));
    assert!(proposals.borrow()[0].contains(&"/future".into()));
    assert!(proposals.borrow()[0].contains(&"/version".into()));
    assert!(!ids(&app,"sapid-json-tree-value").into_iter().any(|id|matches!(app.tree().node(id).unwrap().descriptor.view().props,Props::Text(t)if t.content=="leaf: 1")));
    expanded.revise(|paths| *paths = proposals.borrow()[0].clone());
    app.flush().unwrap();
    let leaf = find("leaf: Number");
    let leaf_before = leaf;
    assert_eq!(
        app.tree()
            .node(leaf)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .tree_item
            .unwrap()
            .level,
        3
    );
    source.revise(|value| *value = "{\"version\":2,\"😀/a~\":{\"leaf\":1}}".into());
    app.flush().unwrap();
    assert_eq!(find("leaf: Number"), leaf_before);
    assert_eq!(find("😀/a~: Object"), branch);
    expanded.revise(|v| *v = vec![String::new()]);
    app.flush().unwrap();
    query.revise(|v| *v = "leaf".into());
    app.flush().unwrap();
    assert!(app.tree().node(find("leaf: Number")).is_ok());
    let calls = proposals.borrow().len();
    app.dispatch(
        root,
        Event::TreeNavigate {
            from: branch,
            command: TreeCommand::Close,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(proposals.borrow().len(), calls);
    assert_eq!(expanded.get(), vec![String::new()]);
    query.revise(String::clear);
    app.flush().unwrap();
    assert_eq!(ids(&app, "sapid-tree-item").len(), 3);
    app.clear().unwrap();
    assert_eq!(app.runtime_stats().tasks, 0);
    assert_eq!(app.stats().queries, 0);
}
#[test]
fn tree_mode_full_leaf_and_finite_depth_reject_atomically_without_changing_copy() {
    let app = App::memory();
    let source = app
        .scope()
        .cage(format!("{{\"long\":\"{}\"}}", "x".repeat(5000)));
    let copy = app.scope().cage(None);
    app.mount(
        json_viewer("long-tree")
            .source(source)
            .tree_navigation(true)
            .limits(JsonViewLimits {
                depth: 64,
                ..Default::default()
            })
            .copy_button(copy_button().node_ref(copy)),
    )
    .unwrap();
    let label = ids(&app, "sapid-json-tree-value")
        .into_iter()
        .find_map(
            |id| match app.tree().node(id).unwrap().descriptor.view().props {
                Props::Text(text) if text.content.starts_with("long:") => {
                    Some(text.content.clone())
                }
                _ => None,
            },
        )
        .unwrap();
    assert!(label.len() > 5000);
    let accepted = source.get();
    source.revise(|v| *v = format!("{}0{}", "[".repeat(64), "]".repeat(64)));
    assert!(app.flush().is_err());
    assert_eq!(
        app.tree()
            .node(copy.get().unwrap())
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .clipboard
            .as_ref()
            .unwrap()
            .text,
        accepted
    );
    let html = glory::ssr::render(
        json_viewer("ssr-tree")
            .tree_navigation(true)
            .source("{\"a\":1}"),
    )
    .unwrap();
    assert!(html.contains("role=\"tree\""));
    assert!(html.contains("role=\"treeitem\""));
    assert!(html.contains("aria-level=\"2\""));
}
