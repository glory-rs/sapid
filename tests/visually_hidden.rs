use glory::prelude::*;
use glory::render::{Props, SemanticId};
use glory::style::{Overflow, Position};
use sapid::{MAX_VISUALLY_HIDDEN_BYTES, visually_hidden};

#[test]
fn semantic_text_remains_accessible_without_an_in_flow_box_or_hidden_flags() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    let id = SemanticId::new("extra-description").unwrap();
    app.mount(
        visually_hidden()
            .content("Details")
            .semantic_id(id.clone())
            .node_ref(reference),
    )
    .unwrap();
    let tree = app.tree();
    let node = tree.node(reference.get().unwrap()).unwrap();
    assert!(
        matches!(&node.descriptor.view().props.clone(),Props::Text(text) if text.content=="Details")
    );
    let semantics = node.descriptor.view().semantics.as_ref().unwrap();
    assert_eq!(semantics.id.as_ref(), Some(&id));
    assert!(!semantics.hidden);
    assert_eq!(
        node.descriptor.view().style.position,
        Some(Position::Absolute)
    );
    assert_eq!(node.descriptor.view().style.width, Some(Length::Px(0.0)));
    assert_eq!(node.descriptor.view().style.height, Some(Length::Px(0.0)));
    assert_eq!(
        node.descriptor.view().style.overflow_x,
        Some(Overflow::Hidden)
    );
    assert_eq!(
        node.descriptor.view().style.overflow_y,
        Some(Overflow::Hidden)
    );
}

#[test]
fn reactive_text_rejects_over_budget_preserves_identity_and_recovers() {
    let app = App::memory();
    let source = app.scope().cage("Initial".to_string());
    let reference = app.scope().cage(None);
    app.mount(visually_hidden().content(source).node_ref(reference))
        .unwrap();
    let id = reference.get().unwrap();
    source.revise(|text| *text = "x".repeat(MAX_VISUALLY_HIDDEN_BYTES + 1));
    assert!(app.flush().is_err());
    assert!(
        matches!(&app.tree().node(id).unwrap().descriptor.view().props.clone(),Props::Text(text) if text.content=="Initial")
    );
    source.revise(|text| *text = "Recovered".into());
    app.flush().unwrap();
    assert_eq!(reference.get(), Some(id));
    assert!(
        matches!(&app.tree().node(id).unwrap().descriptor.view().props.clone(),Props::Text(text) if text.content=="Recovered")
    );
    app.clear().unwrap();
    assert_eq!(reference.get(), None);
    source.revise(|text| *text = "Late".into());
    app.flush().unwrap();
    assert!(app.tree().is_empty());
}

#[test]
fn over_budget_creation_and_expired_content_fail_before_allocating_nodes() {
    let app = App::memory();
    assert!(
        app.mount(visually_hidden().content("x".repeat(MAX_VISUALLY_HIDDEN_BYTES + 1)))
            .is_err()
    );
    assert_eq!(app.stats().nodes, 0);
    let runtime = Runtime::new();
    let owner = runtime.owner();
    let source = owner.cage("Retired".to_string());
    owner.dispose();
    assert!(app.mount(visually_hidden().content(source)).is_err());
    assert_eq!(app.stats().nodes, 0);
}

#[test]
fn classes_are_composed_and_ssr_keeps_escaped_accessible_content() {
    let html = glory::ssr::render(
        visually_hidden()
            .class("custom")
            .content("Extra <detail> & text"),
    )
    .unwrap();
    assert!(html.contains("sapid-visually-hidden custom"));
    assert!(html.contains("Extra &lt;detail&gt; &amp; text"));
    assert!(html.contains("position:absolute") && html.contains("overflow-x:hidden"));
    assert!(
        !html.contains("aria-hidden")
            && !html.contains(" hidden")
            && !html.contains("display:none")
    );
}
