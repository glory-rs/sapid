use glory::prelude::*;
use glory::render::{NodeId, Props};
use glory::style::{Align, Display, FlexDirection, FlexWrap, Justify, Length, Limit};
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            return id;
        }
        pending.extend(node.children.iter().copied());
    }
    panic!("missing {class}")
}
#[test]
fn layout_group_is_a_box_while_glory_group_stays_boxless() {
    let app = App::memory();
    app.mount(sapid::group().fill(glory::ui::group().fill(text().content("child"))))
        .unwrap();
    let id = find(&app, "sapid-group");
    let tree = app.tree();
    let root = tree.node(id).unwrap().descriptor.view();
    assert!(matches!(root.props, Props::View));
    assert_eq!(root.style.display, Some(Display::Flex));
    assert_eq!(root.style.flex_wrap, Some(FlexWrap::Wrap));
    assert_eq!(root.style.align, Some(Align::Center));
    assert_eq!(root.effective_role(), None);
    assert!(matches!(
        tree.node(tree.node(id).unwrap().children[0])
            .unwrap()
            .descriptor
            .view()
            .props
            .clone(),
        Props::Group
    ));
}
#[test]
fn flex_stack_gap_direction_and_explicit_overrides_are_independent() {
    let app = App::memory();
    let custom = app.scope().cage(Style::new().padding(Edges::all(4.0)));
    let reference = app.scope().cage(None);
    app.mount(
        sapid::flex()
            .direction(FlexDirection::Column)
            .gap(7.0)
            .align(Align::End)
            .justify(Justify::Between)
            .wrap(FlexWrap::Wrap)
            .style(custom)
            .node_ref(reference)
            .fill(sapid::stack().class("inside")),
    )
    .unwrap();
    let id = reference.get().unwrap();
    let descriptor = app.tree().node(id).unwrap().descriptor.view().to_owned();
    assert_eq!(descriptor.style.gap, Some(7.0));
    assert_eq!(descriptor.style.justify, Some(Justify::Between));
    assert_eq!(descriptor.style.flex_direction, Some(FlexDirection::Column));
    assert_eq!(descriptor.style.padding, Some(Edges::all(4.0)));
    custom.revise(|style| *style = Style::new().height(Length::Px(40.0)));
    app.flush().unwrap();
    let style = app.tree().node(id).unwrap().descriptor.view().style.clone();
    assert_eq!(style.gap, Some(7.0));
    assert_eq!(style.padding, None);
    assert_eq!(style.height, Some(Length::Px(40.0)));
    assert_eq!(reference.get(), Some(id));
    assert_eq!(
        app.tree()
            .node(find(&app, "inside"))
            .unwrap()
            .descriptor
            .view()
            .style
            .flex_direction,
        Some(FlexDirection::Column)
    );
    app.clear().unwrap();
    assert_eq!(reference.get(), None);
}
#[test]
fn unstyled_preserves_layout_and_explicit_padding_removes_container_cosmetics() {
    let app = App::memory();
    app.mount(
        sapid::container()
            .unstyled()
            .max_width(Limit::Px(480.0))
            .fill(sapid::group().unstyled().gap(9.0)),
    )
    .unwrap();
    let tree = app.tree();
    let outer = tree
        .node(find(&app, "sapid-container"))
        .unwrap()
        .descriptor
        .view()
        .to_owned();
    assert_eq!(outer.style.padding, None);
    assert_eq!(outer.style.display, Some(Display::Flex));
    let inner = tree
        .node(find(&app, "sapid-container-content"))
        .unwrap()
        .descriptor
        .view()
        .to_owned();
    assert_eq!(inner.style.max_width, Some(Limit::Px(480.0)));
    assert_eq!(
        tree.node(find(&app, "sapid-group"))
            .unwrap()
            .descriptor
            .view()
            .style
            .gap,
        Some(9.0)
    );
}
#[test]
fn invalid_layout_configuration_rejects_before_allocating_any_node() {
    for invalid in [sapid::flex().gap(f64::NAN), sapid::flex().gap(-1.0)] {
        let app = App::memory();
        assert!(app.mount(invalid).is_err());
        assert!(app.tree().is_empty());
    }
    let app = App::memory();
    assert!(
        app.mount(sapid::container().max_width(Limit::Px(-1.0)))
            .is_err()
    );
    assert!(app.tree().is_empty());
}
#[test]
fn center_container_space_and_box_serialize_portable_structure_with_no_implicit_roles() {
    let html = glory::ssr::render(
        sapid::container().max_width(Limit::Percent(80.0)).fill(
            sapid::center().height(Length::Px(80.0)).fill((
                sapid::box_().fill(text().content("<&>")),
                sapid::space()
                    .width(Length::Percent(10.0))
                    .height(Length::Px(20.0)),
            )),
        ),
    )
    .unwrap();
    assert!(html.contains("max-width:80%"));
    assert!(html.contains("justify-content:center"));
    assert!(html.contains("&lt;&amp;&gt;"));
    assert!(!html.contains("role="));
}
