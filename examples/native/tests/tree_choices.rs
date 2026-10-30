use glory::apace::{Renderer, shell::WindowApp};
use glory::render::*;
fn named(app: &glory::App, name: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().name.as_deref() == Some(name) {
            return id;
        }
        pending.extend(&node.children);
    }
    panic!("missing {name}")
}
#[test]
fn native_tree_select_and_cascader_keep_real_popup_focus_selection_and_identity_in_all_themes() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        for columns in [false, true] {
            let host = WindowApp::new();
            let app = host.app();
            let value = app.scope().cage(sapid::TreeChoices::default());
            let open = app.scope().cage(false);
            let widget: Box<dyn glory::Widget> = if columns {
                Box::new(
                    sapid::cascader("native-columns")
                        .theme(theme.clone())
                        .name("Choose native")
                        .nodes(vec![sapid::TreeNode::new("leaf", "Leaf")])
                        .value(value)
                        .open(open)
                        .on_value_change(move |next| value.revise(|old| *old = next))
                        .on_open_change(move |next| open.revise(|old| *old = next)),
                )
            } else {
                Box::new(
                    sapid::tree_select("native-tree-choice")
                        .theme(theme.clone())
                        .name("Choose native")
                        .nodes(vec![sapid::TreeNode::new("leaf", "Leaf")])
                        .value(value)
                        .open(open)
                        .on_value_change(move |next| value.revise(|old| *old = next))
                        .on_open_change(move |next| open.revise(|old| *old = next)),
                )
            };
            host.mount(widget).unwrap();
            app.settle(16).unwrap();
            let trigger = named(app, "Choose native");
            let identity = host
                .renderer()
                .engine_id(trigger, glory::html::Part::Main)
                .unwrap();
            app.scope().action(Action::Focus(trigger)).unwrap();
            app.flush().unwrap();
            app.dispatch(trigger, Event::Activate).unwrap();
            app.settle(16).unwrap();
            assert!(open.get());
            let leaf = named(app, "Leaf");
            let tree = named(app, "Choices");
            app.request(Query::Bounds(leaf), |reply| {
                let QueryValue::Bounds(bounds) = reply.unwrap() else {
                    panic!("bounds")
                };
                assert!(bounds.width > 0. && bounds.height > 0.);
            })
            .unwrap();
            app.pump().unwrap();
            app.dispatch(
                tree,
                Event::TreeNavigate {
                    from: leaf,
                    command: TreeCommand::Activate,
                },
            )
            .unwrap();
            app.settle(16).unwrap();
            assert_eq!(value.get().paths()[0].keys(), &["leaf".to_owned()]);
            assert!(!open.get());
            app.request(Query::Focus(trigger), |reply| {
                assert_eq!(reply.unwrap(), QueryValue::Focus(true))
            })
            .unwrap();
            app.pump().unwrap();
            assert_eq!(
                host.renderer()
                    .engine_id(trigger, glory::html::Part::Main)
                    .unwrap(),
                identity
            );
            app.clear().unwrap();
            assert_eq!(app.stats().nodes, 0);
        }
    }
}
#[test]
fn native_tree_popup_preflight_and_direct_apply_admit_the_same_shape_and_query_watermark() {
    let runtime = RuntimeId::new();
    let mut ids = NodeIds::new(runtime);
    let first = ids.allocate();
    let choice = ids.allocate();
    let mut renderer = Renderer::new(runtime);
    let mut d = Descriptor::new(Props::Button(Default::default()));
    d.semantics = Some(Box::new(Semantics {
        id: Some(SemanticId::new("choice").unwrap()),
        controls: Some(SemanticId::new("tree").unwrap()),
        combobox: Some(ComboboxKind::Select),
        popup: Some(PopupKind::Tree),
        expanded: Some(false),
        ..Default::default()
    }));
    assert_eq!(renderer.preflight(d.view(), None), Ok(()));
    assert_eq!(
        renderer
            .apply(&[
                Mutation::Create {
                    id: first,
                    node: Box::default()
                },
                Mutation::Place {
                    id: first,
                    parent: None,
                    before: None
                },
                Mutation::Create {
                    id: choice,
                    node: Box::new(d)
                }
            ])
            .map(|_| ()),
        Ok(())
    );
    assert_eq!(renderer.stats().nodes, 2);
    assert_eq!(renderer.stats().applied, 3);
    renderer
        .request(Query::Bounds(first), QueryToken(8009))
        .unwrap();
    let mut inputs = vec![];
    renderer.drain(&mut inputs);
    assert!(matches!(
        inputs.as_slice(),
        [Input::Response {
            applied: 3,
            token: QueryToken(8009),
            result: Ok(QueryValue::Bounds(_))
        }]
    ));
}
