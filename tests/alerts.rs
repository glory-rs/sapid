use glory::prelude::*;
use glory::render::{Event, NodeId, Props, Role};
use sapid::{AlertSeverity, Announcement, alert};
use std::{cell::Cell, rc::Rc};

fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut ids = tree.roots().to_vec();
    while let Some(id) = ids.pop() {
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
        ids.extend(&node.children);
    }
    panic!("missing {class}");
}
#[test]
fn severity_and_announcement_are_independent_and_relations_are_stable() {
    for (announcement, role) in [
        (Announcement::Off, None),
        (Announcement::Polite, Some(Role::Status)),
        (Announcement::Assertive, Some(Role::Alert)),
    ] {
        let app = App::memory();
        app.mount(
            alert("notice")
                .title("Saved")
                .content("Workspace changes were saved.")
                .severity(AlertSeverity::Success)
                .announcement(announcement),
        )
        .unwrap();
        let root = find(&app, "sapid-alert");
        let title = find(&app, "sapid-alert-title");
        let content = find(&app, "sapid-alert-content");
        let tree = app.tree();
        let d = tree.node(root).unwrap().descriptor.view();
        assert_eq!(d.role, role);
        assert_eq!(
            d.semantics.as_ref().unwrap().labelled_by,
            tree.node(title)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .id
        );
        assert_eq!(
            d.semantics.as_ref().unwrap().described_by,
            tree.node(content)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .id
        );
    }
}
#[test]
fn local_close_hides_once_while_controlled_close_only_proposes_and_sources_are_silent() {
    let app = App::memory();
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    app.mount(
        alert("local")
            .content("Local message")
            .closable(true)
            .on_close(move || seen.set(seen.get() + 1)),
    )
    .unwrap();
    let close = find(&app, "sapid-alert-close");
    let root = find(&app, "sapid-alert");
    assert!(app.dispatch(close, Event::Activate).unwrap());
    assert!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
    assert!(!app.dispatch(close, Event::Activate).unwrap());
    assert_eq!(calls.get(), 1);
    app.clear().unwrap();
    let source = app.scope().cage(true);
    let seen = calls.clone();
    app.mount(
        alert("controlled")
            .content("Controlled message")
            .visible(source)
            .closable(true)
            .on_close(move || seen.set(seen.get() + 1)),
    )
    .unwrap();
    let root = find(&app, "sapid-alert");
    let close = find(&app, "sapid-alert-close");
    app.dispatch(close, Event::Activate).unwrap();
    assert!(source.get());
    assert!(
        !app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
    assert_eq!(calls.get(), 2);
    source.revise(|value| *value = false);
    app.flush().unwrap();
    source.revise(|value| *value = true);
    app.flush().unwrap();
    assert_eq!(find(&app, "sapid-alert"), root);
    assert_eq!(calls.get(), 2);
}
#[test]
fn invalid_reactive_messages_preserve_the_accepted_value_and_recover_in_place() {
    let app = App::memory();
    let value = app.scope().cage("Original message".to_string());
    app.mount(alert("accepted").content(value)).unwrap();
    let id = find(&app, "sapid-alert-content");
    value.revise(|value| *value = " ".into());
    assert!(app.flush().is_err());
    assert!(
        matches!(app.tree().node(id).unwrap().descriptor.view().props,Props::Text(p) if p.content=="Original message")
    );
    value.revise(|value| *value = "Recovered".into());
    app.flush().unwrap();
    assert_eq!(find(&app, "sapid-alert-content"), id);
    assert!(
        matches!(app.tree().node(id).unwrap().descriptor.view().props,Props::Text(p) if p.content=="Recovered")
    );
}
#[test]
fn invalid_configuration_leaves_no_nodes_and_title_names_do_not_include_actions() {
    for widget in [
        alert("invalid").content(""),
        alert("invalid").content("x".repeat(sapid::MAX_ALERT_TEXT_BYTES + 1)),
        alert("invalid").content("valid").title(" "),
        alert("invalid")
            .content("valid")
            .closable(true)
            .close_label(""),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert!(app.tree().is_empty());
    }
    let html = glory::ssr::render(
        alert("escape")
            .title("Saved <draft>")
            .content("A & B")
            .announcement(Announcement::Assertive)
            .closable(true),
    )
    .unwrap();
    assert!(html.contains("role=\"alert\""));
    assert!(html.contains("Saved &lt;draft&gt;"));
    assert!(html.contains("A &amp; B"));
    assert!(html.contains("aria-labelledby="));
    assert!(html.contains("aria-describedby="));
}
#[test]
fn explicit_styles_names_and_owner_retirement_preserve_the_common_contract() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    let dropped = Rc::new(Cell::new(0));
    struct Probe(Rc<Cell<usize>>);
    impl Drop for Probe {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1)
        }
    }
    let probe = Probe(dropped.clone());
    let mounted = app
        .scope()
        .mount_with(|| {
            alert("owned")
                .name("Explicit message")
                .title("Visual title")
                .content("Message")
                .unstyled()
                .class("custom")
                .style(Style::new().padding(Edges::all(7.0)))
                .node_ref(reference)
                .closable(true)
                .on_close(move || {
                    let _ = &probe;
                })
        })
        .unwrap();
    app.flush().unwrap();
    let root = reference.get().unwrap();
    {
        let tree = app.tree();
        let d = tree.node(root).unwrap().descriptor.view();
        assert_eq!(d.name.as_deref(), Some("Explicit message"));
        assert!(d.semantics.as_ref().unwrap().labelled_by.is_none());
        assert_eq!(d.style.padding, Some(Edges::all(7.0)));
        assert_eq!(d.style.background, None);
        assert!(d.classes.iter().any(|value| value == "sapid-alert"));
    }
    drop(mounted);
    app.flush().unwrap();
    assert_eq!(reference.get(), None);
    assert_eq!(dropped.get(), 1);
    assert!(app.tree().is_empty());
}
