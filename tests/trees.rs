use glory::{
    prelude::*,
    render::{Event, EventKind, NodeId, Props, Role, TreeCommand},
};
use sapid::{TreeKind, TreeNode, TreeSelectionMode, tree};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
    rc::Rc,
};
fn nodes() -> Vec<TreeNode> {
    vec![
        TreeNode::new("src", "Source").kind(TreeKind::Branch),
        TreeNode::new("main", "Main").parent("src"),
        TreeNode::new("skip", "Disabled")
            .parent("src")
            .disabled(true),
        TreeNode::new("lib", "Library").parent("src"),
        TreeNode::new("tests", "Tests").kind(TreeKind::Lazy),
    ]
}
fn all(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut stack = tree.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = stack.pop() {
        stack.extend(&tree.node(id).unwrap().children);
        out.push(id);
    }
    out
}
fn named(app: &App, name: &str) -> NodeId {
    all(app)
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
        .unwrap_or_else(|| panic!("missing {name}"))
}
fn items(app: &App) -> Vec<NodeId> {
    all(app)
        .into_iter()
        .filter(|id| app.tree().node(*id).unwrap().descriptor.view().role == Some(Role::TreeItem))
        .collect()
}
fn navigate(app: &App, root: NodeId, from: NodeId, command: TreeCommand) {
    app.dispatch(root, Event::TreeNavigate { from, command })
        .unwrap();
}
fn tab(app: &App) -> NodeId {
    items(app)
        .into_iter()
        .find(|id| app.tree().node(*id).unwrap().descriptor.view().focus == FocusMode::TabStop)
        .unwrap()
}
#[test]
fn four_theme_ssr_uses_real_tree_items_finite_coordinates_and_no_collapsed_owners() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let html = glory::ssr::snapshot(
            tree("files")
                .theme(theme)
                .nodes(nodes())
                .default_selected(BTreeSet::from(["src".into()])),
        )
        .unwrap()
        .html;
        assert!(
            html.contains("role=\"tree\"")
                && html.contains("role=\"treeitem\"")
                && html.contains("aria-level=\"1\"")
                && html.contains("aria-setsize=\"2\"")
                && html.contains("aria-posinset=\"2\"")
        );
        assert!(
            html.contains("aria-expanded=\"false\"") && html.contains("aria-selected=\"true\"")
        );
        assert!(!html.contains("Main") && !html.contains("role=\"button\""));
    }
}
#[test]
fn same_tick_controlled_expansion_and_selection_propose_then_reconcile_refusal() {
    let app = App::memory();
    let expanded = app.scope().cage(BTreeSet::new());
    let selected = app.scope().cage(BTreeSet::new());
    let proposals = Rc::new(RefCell::new(vec![]));
    let out = proposals.clone();
    let selections = Rc::new(RefCell::new(vec![]));
    let out_selected = selections.clone();
    let root = app.scope().cage(None);
    app.mount(
        tree("controlled")
            .nodes(nodes())
            .node_ref(root)
            .expanded(expanded)
            .selected(selected)
            .selection_mode(TreeSelectionMode::Multiple)
            .on_expanded_change(move |value| out.borrow_mut().push(value))
            .on_selection_change(move |value| out_selected.borrow_mut().push(value)),
    )
    .unwrap();
    let disclosure = named(&app, "Toggle Source");
    app.scope()
        .batch(|| {
            app.dispatch(disclosure, Event::Activate).unwrap();
            app.dispatch(disclosure, Event::Activate).unwrap();
        })
        .unwrap();
    app.flush().unwrap();
    assert_eq!(
        *proposals.borrow(),
        [BTreeSet::from(["src".into()]), BTreeSet::new()]
    );
    assert_eq!(items(&app).len(), 2);
    let source = named(&app, "Source");
    app.scope()
        .batch(|| {
            navigate(&app, root.get().unwrap(), source, TreeCommand::Select);
            navigate(&app, root.get().unwrap(), source, TreeCommand::Select);
        })
        .unwrap();
    app.flush().unwrap();
    assert_eq!(
        *selections.borrow(),
        [BTreeSet::from(["src".into()]), BTreeSet::new()]
    );
    assert!(selected.get().is_empty());
    expanded.revise(|set| {
        set.insert("src".into());
    });
    app.flush().unwrap();
    assert_eq!(items(&app).len(), 5);
    assert_eq!(proposals.borrow().len(), 2);
}
#[test]
fn keyboard_focus_and_selection_are_independent_skip_disabled_and_compose_within_tick() {
    let app = App::memory();
    let root = app.scope().cage(None);
    let activated = Rc::new(RefCell::new(vec![]));
    let out = activated.clone();
    app.mount(
        tree("keyboard")
            .nodes(nodes())
            .node_ref(root)
            .default_expanded(BTreeSet::from(["src".into()]))
            .on_activate(move |key| out.borrow_mut().push(key)),
    )
    .unwrap();
    let source = named(&app, "Source");
    let main = named(&app, "Main");
    let library = named(&app, "Library");
    let r = root.get().unwrap();
    assert_eq!(tab(&app), source);
    navigate(&app, r, source, TreeCommand::Open);
    assert_eq!(tab(&app), main);
    app.scope()
        .batch(|| {
            navigate(&app, r, main, TreeCommand::Next);
            navigate(&app, r, main, TreeCommand::Activate);
        })
        .unwrap();
    app.flush().unwrap();
    assert_eq!(tab(&app), library);
    assert_eq!(*activated.borrow(), ["lib"]);
    let view = app.tree();
    assert!(
        view.node(library)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .selected
            .unwrap()
    );
    drop(view);
    navigate(&app, r, library, TreeCommand::Close);
    assert_eq!(tab(&app), source);
    navigate(&app, r, source, TreeCommand::Close);
    assert_eq!(items(&app).len(), 2);
}
#[test]
fn lazy_children_are_explicit_requests_and_current_model_replacement_preserves_parent_identity() {
    let app = App::memory();
    let data = app.scope().cage(nodes());
    let root = app.scope().cage(None);
    let calls = Rc::new(RefCell::new(vec![]));
    let out = calls.clone();
    app.mount(
        tree("lazy")
            .nodes(data)
            .node_ref(root)
            .on_load_children(move |key| out.borrow_mut().push(key)),
    )
    .unwrap();
    let tests = named(&app, "Tests");
    let r = root.get().unwrap();
    navigate(&app, r, tests, TreeCommand::Open);
    navigate(&app, r, tests, TreeCommand::Open);
    assert_eq!(*calls.borrow(), ["tests"]);
    data.revise(|nodes| {
        nodes[4].kind = TreeKind::Branch;
        nodes.push(TreeNode::new("case", "A test").parent("tests"));
    });
    app.flush().unwrap();
    assert_eq!(named(&app, "Tests"), tests);
    assert!(items(&app).contains(&named(&app, "A test")));
    navigate(&app, r, tests, TreeCommand::Open);
    assert_eq!(tab(&app), named(&app, "A test"));
    assert_eq!(calls.borrow().len(), 1);
}
#[test]
fn keyed_custom_editors_survive_source_reorder_and_closed_rows_release_owners() {
    let app = App::memory();
    let data = app.scope().cage(nodes());
    let expanded = app.scope().cage(BTreeSet::from(["src".into()]));
    let created = Rc::new(Cell::new(0));
    let built = created.clone();
    let dropped = Rc::new(Cell::new(0));
    let retired = dropped.clone();
    app.mount(
        tree("editors")
            .nodes(data)
            .expanded(expanded)
            .render_label(move |node| {
                built.set(built.get() + 1);
                let retired = retired.clone();
                widget(move |scope| {
                    let retired = retired.clone();
                    scope.on_cleanup(move || retired.set(retired.get() + 1))?;
                    input()
                        .value("draft")
                        .name(node.with(|node| format!("Editor {}", node.key)))
                        .show_in(scope)
                })
            }),
    )
    .unwrap();
    let main = named(&app, "Main");
    let editor = named(&app, "Editor main");
    data.revise(|nodes| nodes.swap(1, 3));
    app.flush().unwrap();
    assert_eq!(named(&app, "Main"), main);
    assert_eq!(named(&app, "Editor main"), editor);
    assert_eq!(created.get(), 5);
    expanded.revise(BTreeSet::clear);
    app.flush().unwrap();
    assert_eq!(items(&app).len(), 2);
    assert_eq!(dropped.get(), 3);
    assert!(app.tree().node(editor).is_err());
    app.clear().unwrap();
    assert_eq!(dropped.get(), 5);
    assert_eq!(app.stats().nodes, 0);
}
#[test]
fn invalid_external_data_is_rejected_before_factories_and_can_recover() {
    let app = App::memory();
    let data = app.scope().cage(nodes());
    let factories = Rc::new(Cell::new(0));
    let out = factories.clone();
    app.mount(tree("admission").nodes(data).render_label(move |node| {
        out.set(out.get() + 1);
        text().content(node.with(|node| node.label.clone()))
    }))
    .unwrap();
    let source = named(&app, "Source");
    data.revise(|nodes| nodes[0].parent = Some("src".into()));
    assert!(app.flush().is_err());
    assert_eq!(factories.get(), 2);
    assert_eq!(named(&app, "Source"), source);
    data.revise(|data| *data = nodes());
    app.flush().unwrap();
    assert_eq!(factories.get(), 2);
    assert_eq!(named(&app, "Source"), source);
    let bad = App::memory();
    assert!(
        bad.mount(tree("bad").nodes(vec![TreeNode::new("key", "x".repeat(4097))]))
            .is_err()
    );
}
#[test]
fn bounded_typeahead_uses_current_visible_labels_and_selection_none_stays_unselected() {
    let app = App::memory();
    let root = app.scope().cage(None);
    app.mount(
        tree("search")
            .nodes(nodes())
            .node_ref(root)
            .selection_mode(TreeSelectionMode::None)
            .default_expanded(BTreeSet::from(["src".into()])),
    )
    .unwrap();
    let source = named(&app, "Source");
    navigate(
        &app,
        root.get().unwrap(),
        source,
        TreeCommand::Search {
            text: "l".into(),
            at_millis: 10,
        },
    );
    assert_eq!(tab(&app), named(&app, "Library"));
    navigate(
        &app,
        root.get().unwrap(),
        named(&app, "Library"),
        TreeCommand::Select,
    );
    assert!(items(&app).iter().all(|id| {
        app.tree()
            .node(*id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .selected
            .is_none()
    }));
    assert!(all(&app).iter().all(|id| !matches!(
        app.tree().node(*id).unwrap().descriptor.view().props,
        Props::Check(_)
    )));
}

#[test]
fn opening_then_navigation_in_one_tick_uses_only_an_accepted_expansion() {
    let app = App::memory();
    let root = app.scope().cage(None);
    app.mount(tree("tick").nodes(nodes()).node_ref(root))
        .unwrap();
    let source = named(&app, "Source");
    let r = root.get().unwrap();
    app.scope()
        .batch(|| {
            navigate(&app, r, source, TreeCommand::Open);
            navigate(&app, r, source, TreeCommand::Next);
            navigate(&app, r, source, TreeCommand::Select);
        })
        .unwrap();
    app.flush().unwrap();
    let main = named(&app, "Main");
    assert_eq!(tab(&app), main);
    assert_eq!(
        app.tree()
            .node(main)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .selected,
        Some(true)
    );
}

#[test]
fn nested_editors_keep_activation_and_generic_focus_observers_cannot_replace_internal_state() {
    let app = App::memory();
    let focused = Rc::new(Cell::new(0));
    let out = focused.clone();
    app.mount(
        tree("nested")
            .nodes(nodes())
            .on(EventKind::FocusWithin, move |_| out.set(out.get() + 1))
            .render_label(|node| {
                input()
                    .name(node.with(|node| format!("Edit {}", node.key)))
                    .value("draft")
            }),
    )
    .unwrap();
    let source = named(&app, "Source");
    let editor = named(&app, "Edit tests");
    app.dispatch(editor, Event::Activate).unwrap();
    assert_eq!(tab(&app), source);
    let tests = named(&app, "Tests");
    app.dispatch(tests, Event::FocusChanged(true)).unwrap();
    assert_eq!(tab(&app), tests);
    assert_eq!(focused.get(), 1);
}

#[test]
fn disabled_tree_uses_a_real_fieldset_for_custom_inputs_and_excludes_item_tab_stops() {
    let app = App::memory();
    app.mount(
        tree("disabled")
            .nodes(nodes())
            .enabled(false)
            .render_label(|_| input().name("Custom input").value("draft")),
    )
    .unwrap();
    assert!(all(&app).iter().any(|id| {
        app.tree()
            .node(*id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .is_some_and(|s| s.field_part == Some(glory::render::FieldPart::Fieldset) && s.disabled)
    }));
    assert!(
        items(&app)
            .iter()
            .all(|id| app.tree().node(*id).unwrap().descriptor.view().focus != FocusMode::TabStop)
    );
}
