use glory::style::{Display, GridPlacement};
use glory::{
    prelude::*,
    render::{Event, NodeId, Role, TreeCommand},
};
use sapid::{TreeKind, TreeNode, cascader};
use std::{cell::RefCell, collections::BTreeSet, rc::Rc};
fn nodes() -> Vec<TreeNode> {
    vec![
        TreeNode::new("one", "One").kind(TreeKind::Branch),
        TreeNode::new("a", "Alpha")
            .kind(TreeKind::Branch)
            .parent("one"),
        TreeNode::new("leaf", "Leaf").parent("a"),
        TreeNode::new("b", "Beta").parent("one"),
        TreeNode::new("two", "Two").kind(TreeKind::Lazy),
    ]
}
#[test]
fn unstyled_keeps_physical_columns_and_viewport_cap_without_cosmetic_border() {
    let app = App::memory();
    app.mount(
        cascader("unstyled-columns")
            .unstyled()
            .nodes(nodes())
            .default_open(true)
            .default_expanded(BTreeSet::from(["one".into(), "a".into()])),
    )
    .unwrap();
    let t = app.tree();
    let style = &t.node(root(&app)).unwrap().descriptor.view().style;
    assert_eq!(style.display, Some(Display::Grid));
    assert_eq!(style.max_width, Some(glory::style::Limit::Percent(95.)));
    assert!(style.border_color.is_none());
    assert!(style.border_style.is_none());
}
fn all(app: &App) -> Vec<NodeId> {
    let t = app.tree();
    let mut p = t.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = p.pop() {
        p.extend(&t.node(id).unwrap().children);
        out.push(id)
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
fn root(app: &App) -> NodeId {
    all(app)
        .into_iter()
        .find(|id| app.tree().node(*id).unwrap().descriptor.view().role == Some(Role::Tree))
        .unwrap()
}
fn go(app: &App, from: NodeId, command: TreeCommand) {
    app.dispatch(root(app), Event::TreeNavigate { from, command })
        .unwrap();
}
#[test]
fn ssr_uses_column_grid_without_replacing_tree_semantics() {
    let html = glory::ssr::snapshot(
        cascader("columns")
            .nodes(nodes())
            .default_expanded(BTreeSet::from(["one".into(), "a".into()])),
    )
    .unwrap()
    .html;
    assert!(
        html.contains("role=\"tree\"")
            && html.contains("role=\"treeitem\"")
            && !html.contains("role=\"listbox\"")
    );
    assert!(
        html.contains("grid-template-columns:180px 180px 180px")
            && html.contains("grid-column:3 / span 1")
    );
}
#[test]
fn selecting_branches_opens_one_chain_and_children_keep_real_grid_positions() {
    let app = App::memory();
    app.mount(cascader("columns").nodes(nodes()).default_open(true))
        .unwrap();
    let one = named(&app, "One");
    app.dispatch(one, Event::Activate).unwrap();
    app.flush().unwrap();
    let alpha = named(&app, "Alpha");
    let tree = app.tree();
    let d = tree.node(alpha).unwrap().descriptor.view();
    assert_eq!(
        d.style.grid_column,
        Some(GridPlacement::At { start: 2, span: 1 })
    );
    drop(tree);
    app.dispatch(alpha, Event::Activate).unwrap();
    app.flush().unwrap();
    assert!(all(&app).contains(&named(&app, "Leaf")));
    app.dispatch(named(&app, "Two"), Event::Activate).unwrap();
    app.flush().unwrap();
    assert!(app.tree().node(alpha).is_err());
    assert_eq!(
        app.tree()
            .node(root(&app))
            .unwrap()
            .descriptor
            .view()
            .style
            .display,
        Some(Display::Grid)
    );
}
#[test]
fn same_tick_open_and_right_uses_accepted_chain_and_vertical_keys_stay_in_column() {
    let app = App::memory();
    app.mount(
        cascader("keys")
            .nodes(nodes())
            .default_open(true)
            .default_expanded(BTreeSet::from(["one".into()])),
    )
    .unwrap();
    let alpha = named(&app, "Alpha");
    app.scope()
        .batch(|| {
            go(&app, alpha, TreeCommand::Open);
            go(&app, alpha, TreeCommand::Open);
            go(&app, alpha, TreeCommand::Activate);
        })
        .unwrap();
    app.flush().unwrap();
    let trigger = all(&app)
        .into_iter()
        .find(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .is_some_and(|s| s.combobox.is_some())
        })
        .unwrap();
    assert_eq!(
        app.tree()
            .node(trigger)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .expanded,
        Some(false)
    );
    // Reopen and return to siblings rather than flattened preorder descendants.
    app.dispatch(trigger, Event::Activate).unwrap();
    app.flush().unwrap();
    app.scope()
        .batch(|| {
            go(&app, alpha, TreeCommand::Next);
            go(&app, alpha, TreeCommand::Activate);
        })
        .unwrap();
    app.flush().unwrap();
    let t = app.tree();
    assert!(
        matches!(t.node(trigger).unwrap().descriptor.view().props,glory::render::Props::Button(p) if p.label=="Beta")
    );
}
#[test]
fn unrelated_expansion_and_65_siblings_reject_before_mounted_choice_state_changes() {
    let app = App::memory();
    assert!(
        app.mount(
            cascader("invalid")
                .nodes(nodes())
                .default_expanded(BTreeSet::from(["one".into(), "two".into()]))
        )
        .is_err()
    );
    let app = App::memory();
    assert!(
        app.mount(
            cascader("wide").nodes(
                (0..65)
                    .map(|i| TreeNode::new(format!("k{i}"), "Name"))
                    .collect::<Vec<_>>()
            )
        )
        .is_err()
    );
}
#[test]
fn lazy_requests_are_explicit_and_source_publication_mounts_only_current_column() {
    let app = App::memory();
    let source = app.scope().cage(nodes());
    let loads = Rc::new(RefCell::new(vec![]));
    let out = loads.clone();
    app.mount(
        cascader("lazy")
            .nodes(source)
            .default_open(true)
            .on_load_children(move |key| out.borrow_mut().push(key)),
    )
    .unwrap();
    let two = named(&app, "Two");
    app.dispatch(two, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(&*loads.borrow(), &["two"]);
    source.revise(|nodes| {
        nodes[4].kind = TreeKind::Branch;
        nodes.push(TreeNode::new("loaded", "Loaded").parent("two"));
    });
    app.flush().unwrap();
    assert_eq!(named(&app, "Two"), two);
    assert_eq!(
        app.tree()
            .node(named(&app, "Loaded"))
            .unwrap()
            .descriptor
            .view()
            .style
            .grid_column,
        Some(GridPlacement::At { start: 2, span: 1 })
    );
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
}
