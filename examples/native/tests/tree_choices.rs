use glory::blitz::{Renderer, shell::WindowApp};
use glory::render::*;
#[test]
fn native_tree_select_and_cascader_keep_explicit_combobox_rejection_in_all_themes() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        for columns in [false, true] {
            let host = WindowApp::new();
            let node = sapid::TreeNode::new("leaf", "Leaf");
            let error = if columns {
                host.mount(
                    sapid::cascader("native-columns")
                        .theme(theme.clone())
                        .nodes(vec![node]),
                )
            } else {
                host.mount(
                    sapid::tree_select("native-tree-choice")
                        .theme(theme.clone())
                        .nodes(vec![node]),
                )
            }
            .unwrap_err();
            assert!(
                matches!(
                    error,
                    glory::ui::Error::Render(Error::Unsupported("native combobox controls"))
                ),
                "{error:?}"
            );
            host.app().clear().unwrap();
            assert_eq!(host.app().stats().nodes, 0);
        }
    }
}
#[test]
fn native_tree_popup_preflight_and_direct_apply_retain_the_successful_prefix_and_query_watermark()
 {
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
    assert_eq!(
        renderer.preflight(d.view(), None),
        Err(Error::Unsupported("native combobox controls"))
    );
    assert_eq!(
        renderer.apply(&[
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
        ]),
        Err(Error::Unsupported("native combobox controls"))
    );
    assert_eq!(renderer.stats().nodes, 1);
    assert_eq!(renderer.stats().applied, 2);
    renderer
        .request(Query::Bounds(first), QueryToken(8009))
        .unwrap();
    let mut inputs = vec![];
    renderer.drain(&mut inputs);
    assert!(matches!(
        inputs.as_slice(),
        [Input::Response {
            applied: 2,
            token: QueryToken(8009),
            result: Ok(QueryValue::Bounds(_))
        }]
    ));
}
