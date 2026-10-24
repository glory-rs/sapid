use glory::prelude::*;
use glory::render::{NodeId, Props};
use glory::style::{Length, Position, Style};
use sapid::{
    AffixContext, AffixHorizontal, AffixOffset, AffixVertical, affix, affix_container,
};
use std::{cell::Cell, rc::Rc};
fn style(app: &App, node: NodeId) -> Style {
    app.tree()
        .node(node)
        .unwrap()
        .descriptor
        .view()
        .style
        .clone()
}
fn offset(horizontal: AffixHorizontal, vertical: AffixVertical) -> AffixOffset {
    AffixOffset {
        horizontal,
        vertical,
    }
}
#[test]
fn viewport_offsets_clear_opposite_sides_and_invalid_sources_keep_the_accepted_snapshot() {
    let app = App::memory();
    let value = app.scope().cage(AffixOffset::default());
    let visible = app.scope().cage(true);
    let reference = app.scope().cage(None);
    app.mount(
        affix("floating")
            .offset(value)
            .visible(visible)
            .node_ref(reference),
    )
    .unwrap();
    let id = reference.get().unwrap();
    assert_eq!(style(&app, id).position, Some(Position::Fixed));
    assert_eq!(style(&app, id).right, Some(Length::Px(16.0)));
    value.revise(|v| *v = offset(AffixHorizontal::Left(12.0), AffixVertical::Top(-3.0)));
    app.flush().unwrap();
    let accepted = style(&app, id);
    assert_eq!(accepted.left, Some(Length::Px(12.0)));
    assert_eq!(accepted.top, Some(Length::Px(-3.0)));
    assert!(accepted.right.is_none() && accepted.bottom.is_none());
    value.revise(|v| v.horizontal = AffixHorizontal::Right(f64::NAN));
    visible.revise(|value| *value = false);
    assert!(app.flush().is_err());
    assert_eq!(style(&app, id), accepted);
    assert!(
        !app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
    value.revise(|v| *v = AffixOffset::default());
    app.flush().unwrap();
    assert!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
    assert_eq!(style(&app, id).bottom, Some(Length::Px(16.0)));
    assert_eq!(app.stats().queries, 0);
}
#[test]
fn visibility_keeps_factory_children_refs_and_state_until_owner_retirement() {
    let app = App::memory();
    let visible = app.scope().cage(true);
    let input = app.scope().cage(None);
    let wrapper = app.scope().cage(None);
    let builds = Rc::new(Cell::new(0));
    let count = builds.clone();
    app.mount(
        affix("visible")
            .visible(visible)
            .node_ref(wrapper)
            .fill(glory::ui::widget(move |scope| {
                count.set(count.get() + 1);
                glory::ui::input()
                    .value("Draft")
                    .node_ref(input)
                    .show_in(scope)
            })),
    )
    .unwrap();
    let id = input.get().unwrap();
    let root = wrapper.get().unwrap();
    for next in [false, true, false, true] {
        visible.revise(|v| *v = next);
        app.flush().unwrap();
        assert_eq!(input.get(), Some(id));
        assert_eq!(builds.get(), 1);
        assert_eq!(
            app.tree()
                .node(root)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .hidden,
            !next
        );
    }
    app.clear().unwrap();
    assert!(wrapper.get().is_none() && input.get().is_none());
}
#[test]
fn container_context_requires_a_live_same_runtime_ancestor_and_preserves_child_owners() {
    let app = App::memory();
    let saved = Rc::new(Cell::new(None::<AffixContext>));
    let output = saved.clone();
    let absolute = app.scope().cage(None);
    let sticky = app.scope().cage(None);
    app.mount(affix_container("region").content(move |context| {
        output.set(Some(context));
        glory::ui::view().fill((
            affix("absolute").within(context).node_ref(absolute),
            affix("sticky").sticky(context).node_ref(sticky),
        ))
    }))
    .unwrap();
    assert_eq!(
        style(&app, absolute.get().unwrap()).position,
        Some(Position::Absolute)
    );
    let sticky_style = style(&app, sticky.get().unwrap());
    assert_eq!(sticky_style.position, Some(Position::Sticky));
    assert_eq!(sticky_style.top, Some(Length::Px(0.0)));
    assert!(sticky_style.bottom.is_none());
    let context = saved.get().unwrap();
    let before = app.tree().stats();
    assert!(affix("wrong").within(context).show_in(app.scope()).is_err());
    assert_eq!(app.tree().stats(), before);
    let foreign = App::memory();
    assert!(foreign.mount(affix("foreign").within(context)).is_err());
    assert!(foreign.tree().roots().is_empty());
    app.clear().unwrap();
    assert!(absolute.get().is_none() && sticky.get().is_none());
    assert!(app.mount(affix("retired").within(context)).is_err());
}
#[test]
fn explicit_styles_and_unstyled_preserve_structural_positioning_and_widget_classes() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    app.mount(
        affix("custom")
            .unstyled()
            .class("custom")
            .node_ref(reference)
            .style(
                Style::new()
                    .position(Position::Static)
                    .width(Length::Px(123.0))
                    .left(Length::Px(999.0)),
            ),
    )
    .unwrap();
    let node = reference.get().unwrap();
    let tree = app.tree();
    let descriptor = tree.node(node).unwrap().descriptor.view();
    assert!(matches!(descriptor.props, Props::View));
    assert!(descriptor.classes.iter().any(|v| v == "custom"));
    assert!(descriptor.classes.iter().any(|v| v == "sapid-affix"));
    assert_eq!(descriptor.style.width, Some(Length::Px(123.0)));
    assert_eq!(descriptor.style.position, Some(Position::Fixed));
    assert!(descriptor.style.left.is_none());
}
#[test]
fn invalid_initial_offsets_are_rejected_before_creating_a_node() {
    for value in [f64::NAN, f64::INFINITY, 1_000_001.0] {
        let app = App::memory();
        assert!(
            app.mount(affix("invalid").offset(offset(
                AffixHorizontal::Right(value),
                AffixVertical::Bottom(0.0)
            )))
            .is_err()
        );
        assert!(app.tree().roots().is_empty());
    }
    // A finite signed offset remains intentional and valid.
    let app = App::memory();
    app.mount(affix("negative").offset(offset(
        AffixHorizontal::Left(-5.0),
        AffixVertical::Top(-10.0),
    )))
    .unwrap();
}
