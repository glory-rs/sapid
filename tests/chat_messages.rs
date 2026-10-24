#![cfg(feature = "chat")]
use glory::render::{NodeId, Props, SemanticId};
use glory::ui::App;
use sapid::{MessageData, MessageGroup, MessageQuote, MessageStatus, message};
#[path = "../examples/web/src/messages_form.rs"]
mod fixture;

fn nodes(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut result = Vec::new();
    while let Some(id) = pending.pop() {
        result.push(id);
        pending.extend(tree.node(id).unwrap().children.iter().copied());
    }
    result
}
fn contents(app: &App) -> Vec<String> {
    let ids = nodes(app);
    let tree = app.tree();
    ids.iter()
        .filter_map(|id| match tree.node(*id).unwrap().descriptor.view().props {
            Props::Text(text) => Some(text.content.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn display_data_is_bounded_and_key_namespaces_are_unambiguous() {
    assert_ne!(
        sapid::message_id("a_b").unwrap(),
        sapid::message_id("ab").unwrap()
    );
    for key in ["".to_owned(), "bad key".into(), "a".repeat(65)] {
        assert!(sapid::message_id(key).is_err());
    }
    let mut value = MessageData::new("A", "x".repeat(sapid::MAX_MESSAGE_BODY_BYTES));
    assert!(value.validate().is_ok());
    value.body.push('x');
    assert!(value.validate().is_err());
    value.body.clear();
    value.author = " ".into();
    assert!(value.validate().is_err());
    value.author = "A".into();
    value.quote = Some(MessageQuote {
        source: "Source".into(),
        body: "x".repeat(4097),
        target: None,
    });
    assert!(value.validate().is_err());
}

#[test]
fn caller_actions_and_fixture_build_in_all_four_themes() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let app = App::memory();
        app.mount(fixture::messages_form(theme)).unwrap();
        assert!(contents(&app).contains(&"Retries 0".into()));
        let ids = nodes(&app);
        let retry = {
            let tree = app.tree();
            ids.iter().find(|id|matches!(tree.node(**id).unwrap().descriptor.view().props,Props::Button(button) if button.label=="Application retry")).copied().unwrap()
        };
        app.dispatch(retry, glory::render::Event::Activate).unwrap();
        app.flush().unwrap();
        assert!(contents(&app).contains(&"Retries 1".into()));
    }
}

#[test]
fn atomic_rejection_preserves_body_metadata_and_existing_nodes_then_recovers() {
    let app = App::memory();
    let data = app
        .scope()
        .cage(MessageData::new("Alice", "First\r\nbody\0"));
    app.mount(message("first", data)).unwrap();
    let initial = nodes(&app);
    assert!(contents(&app).contains(&"First\nbody�".into()));
    data.revise(|value| {
        value.body = "Must not publish".into();
        value.author = "\n".into();
    });
    assert!(app.flush().is_err());
    assert_eq!(nodes(&app), initial);
    assert!(!contents(&app).contains(&"Must not publish".into()));
    data.revise(|value| {
        value.author = "Bob".into();
        value.body = "Recovered".into();
    });
    app.flush().unwrap();
    assert_eq!(nodes(&app), initial);
    assert!(contents(&app).contains(&"Recovered".into()));
    app.clear().unwrap();
    assert!(app.tree().roots().is_empty());
}

#[test]
fn quote_is_real_blockquote_and_marker_is_a_real_link_without_a_fake_action() {
    let mut data = MessageData::new("Caller", "<script>never execute</script>");
    data.timestamp = Some("10:00 UTC".into());
    data.status = MessageStatus::Failed;
    data.quote = Some(MessageQuote {
        source: "Original".into(),
        body: "Quoted".into(),
        target: Some(SemanticId::new("original").unwrap()),
    });
    let snapshot = glory::ssr::snapshot(message("reply", data)).unwrap();
    assert!(snapshot.html.contains("<blockquote"));
    assert!(snapshot.html.contains("href=\"#original\""));
    assert!(
        snapshot
            .html
            .contains("&lt;script&gt;never execute&lt;/script&gt;")
    );
    assert!(snapshot.html.contains("Failed"));
    assert!(!snapshot.html.contains("role=\"alert\""));
}

#[test]
fn grouping_hides_only_metadata_and_preserves_body_and_identity() {
    let app = App::memory();
    let data = app
        .scope()
        .cage(MessageData::new("Alice", "Always visible"));
    app.mount(message("group", data)).unwrap();
    let before = nodes(&app);
    data.revise(|value| {
        value.group = MessageGroup::Middle;
        value.status = MessageStatus::Sending;
    });
    app.flush().unwrap();
    assert_eq!(before, nodes(&app));
    let tree = app.tree();
    let header = before.iter().find_map(|id| {
        let node = tree.node(*id).unwrap();
        let view = node.descriptor.view();
        view.classes
            .iter()
            .any(|name| name == "sapid-message-header")
            .then(|| view.semantics.as_ref().unwrap().hidden)
    });
    assert_eq!(header, Some(true));
}
