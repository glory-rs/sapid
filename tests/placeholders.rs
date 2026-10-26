use glory::prelude::*;
use glory::render::{NodeId, Props, Semantics};

fn editor(app: &App) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if matches!(node.descriptor.view().props, Props::Input(_)) {
            return id;
        }
        pending.extend(node.children.iter().copied());
    }
    panic!("input missing");
}

#[test]
fn reactive_placeholder_preserves_value_revision_name_and_rejects_oversized_sources() {
    let app = App::memory();
    let hint = app.scope().cage("Search projects".to_owned());
    app.mount(
        sapid::text_field()
            .name("Project")
            .default_value("Rust")
            .placeholder(hint),
    )
    .unwrap();
    let id = editor(&app);
    let before = app.tree().node(id).unwrap().descriptor.view().to_owned();
    hint.revise(|value| *value = "Find a project".into());
    app.flush().unwrap();
    assert_eq!(
        app.tree().node(id).unwrap().descriptor.view().props,
        &before.props
    );
    assert_eq!(
        app.tree().node(id).unwrap().descriptor.view().name,
        &before.name
    );
    hint.revise(|value| *value = "x".repeat(1_000_000));
    assert!(app.flush().is_err());
    assert_eq!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .placeholder
            .as_deref(),
        Some("Find a project")
    );
    hint.revise(String::clear);
    app.flush().unwrap();
    assert_eq!(editor(&app), id);
}

#[test]
fn input_families_lower_real_hints_and_multiline_is_not_a_business_value() {
    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(sapid::text_field().placeholder("Hint <&>")),
        Box::new(sapid::password_input().placeholder("Hint <&>")),
        Box::new(sapid::textarea().placeholder("Hint <&>\nSecond line")),
        Box::new(sapid::combobox("hint-combo").placeholder("Hint <&>")),
        Box::new(sapid::autocomplete("hint-auto").placeholder("Hint <&>")),
        Box::new(sapid::multi_select("hint-multi").placeholder("Hint <&>")),
        Box::new(sapid::tags_input("hint-tags").placeholder("Hint <&>")),
        Box::new(sapid::pills_input("hint-pills").placeholder("Hint <&>")),
    ];
    for widget in widgets {
        let html = glory::ssr::snapshot(widget).unwrap().html;
        assert!(html.contains("placeholder=\"Hint &lt;&amp;&gt;"));
        assert!(!html.contains("value=\"Hint"));
    }
    let app = App::memory();
    assert!(
        app.mount(sapid::text_field().placeholder("bad\nhint"))
            .is_err()
    );
    assert!(app.tree().is_empty());
    let invalid = Semantics {
        placeholder: Some("x".repeat(4097)),
        ..Default::default()
    };
    assert!(
        app.mount(input().semantics(Some(Box::new(invalid))))
            .is_err()
    );
}

#[cfg(feature = "json")]
#[test]
fn json_input_forwards_multiline_hint_without_supplying_a_document_value() {
    let html =
        glory::ssr::snapshot(sapid::json_input("json-hint").placeholder("Paste JSON\n{}"))
            .unwrap()
            .html;
    assert!(html.contains("placeholder=\"Paste JSON\n{}\""));
    assert!(!html.contains("value=\"Paste JSON"));
}
