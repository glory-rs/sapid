use glory::prelude::*;
use sapid::{NavigationProgressHandle, navigation_progress};
use std::{cell::RefCell, rc::Rc};

fn fixture() -> (
    App,
    NavigationProgressHandle,
    Cage<Option<glory::render::NodeId>>,
) {
    let app = App::memory();
    let saved = Rc::new(RefCell::new(None));
    let out = saved.clone();
    let reference = app.scope().cage(None);
    app.mount(
        navigation_progress()
            .node_ref(reference)
            .on_ready(move |handle| *out.borrow_mut() = Some(handle)),
    )
    .unwrap();
    let handle = saved.borrow_mut().take().unwrap();
    (app, handle, reference)
}
#[test]
fn application_commands_keep_identity_and_publish_actual_progress_semantics() {
    let (app, handle, reference) = fixture();
    let root = reference.get().unwrap();
    let bar = app.tree().node(root).unwrap().children[0];
    assert!(!handle.snapshot().unwrap().active);
    let token = handle.start().unwrap();
    app.flush().unwrap();
    assert!(handle.snapshot().unwrap().active);
    {
        let tree = app.tree();
        assert!(
            !tree
                .node(root)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .hidden
        );
        let props = tree.node(bar).unwrap().descriptor.view();
        assert_eq!(props.role, Some(glory::render::Role::ProgressBar));
        assert_eq!(
            props
                .semantics
                .as_ref()
                .unwrap()
                .progress
                .as_ref()
                .unwrap()
                .current,
            None
        );
    }
    assert!(handle.set(&token, 45.).unwrap());
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(bar)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .progress
            .as_ref()
            .unwrap()
            .current,
        Some(45.)
    );
    assert!(handle.indeterminate(&token).unwrap());
    assert!(handle.finish(&token).unwrap());
    app.flush().unwrap();
    assert_eq!(
        handle.snapshot().unwrap(),
        sapid::NavigationProgressSnapshot {
            active: false,
            value: Some(100.)
        }
    );
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
    assert_eq!(app.tree().node(root).unwrap().children[0], bar);
    assert_eq!(app.stats().queries, 0);
    assert_eq!(app.stats().handlers, 0);
}
#[test]
fn replacement_revokes_stale_permits_and_foreign_tokens_cannot_publish() {
    let (app, handle, _) = fixture();
    let (_foreign_app, foreign, _) = fixture();
    let old = handle.start().unwrap();
    let token = handle.start().unwrap();
    assert!(!handle.set(&old, 98.).unwrap());
    assert!(!handle.finish(&old).unwrap());
    assert!(foreign.set(&token, 20.).is_err());
    for value in [f64::NAN, f64::INFINITY, -1., 101.] {
        assert!(handle.set(&token, value).is_err());
        assert_eq!(handle.snapshot().unwrap().value, None);
    }
    assert!(handle.cancel(&token).unwrap());
    assert!(!handle.cancel(&token).unwrap());
    assert!(!handle.finish(&token).unwrap());
    assert!(!handle.set(&token, 20.).unwrap());
    app.flush().unwrap();
    assert!(!handle.snapshot().unwrap().active);
}
#[test]
fn retirement_rejects_handles_and_tokens_without_retaining_scope_or_nodes() {
    let app = App::memory();
    let saved = Rc::new(RefCell::new(None));
    let out = saved.clone();
    let mounted = app
        .scope()
        .mount_with(|| {
            navigation_progress().on_ready(move |handle| *out.borrow_mut() = Some(handle))
        })
        .unwrap();
    app.flush().unwrap();
    let handle = saved.borrow_mut().take().unwrap();
    let token = handle.start().unwrap();
    drop(mounted);
    app.flush().unwrap();
    assert!(app.tree().is_empty());
    assert!(handle.start().is_err());
    assert!(handle.set(&token, 50.).is_err());
    assert!(handle.snapshot().is_err());
}
#[test]
fn independent_controllers_and_repeated_cycles_keep_only_current_operation() {
    let (app, handle, reference) = fixture();
    let root = reference.get().unwrap();
    let initial_nodes = app.tree().len();
    for _ in 0..512 {
        let token = handle.start().unwrap();
        handle.set(&token, 75.).unwrap();
        handle.cancel(&token).unwrap();
    }
    app.flush().unwrap();
    assert_eq!(app.tree().len(), initial_nodes);
    assert_eq!(reference.get(), Some(root));
    assert_eq!(app.stats().queries, 0);
}
