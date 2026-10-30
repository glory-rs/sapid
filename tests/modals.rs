use glory::prelude::*;
use glory::render::{Event, NodeId, Role};
use sapid::{ModalOutcome, ModalsHandle, confirmation_request, modal_request, modals_manager};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn fixture(capacity: usize) -> (App, ModalsHandle) {
    let app = App::memory();
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    app.mount(
        modals_manager("queue")
            .capacity(capacity)
            .on_ready(move |handle| *saved.borrow_mut() = Some(handle)),
    )
    .unwrap();
    let handle = out.borrow_mut().take().unwrap();
    (app, handle)
}
fn active(app: &App, role: Role) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        let d = node.descriptor.view();
        if d.role == Some(role) && !d.semantics.as_ref().is_some_and(|s| s.hidden) {
            return id;
        }
        pending.extend(&node.children)
    }
    panic!("missing role {role:?}")
}
fn button(app: &App, label: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let n = tree.node(id).unwrap();
        if matches!(n.descriptor.view().props,glory::render::Props::Button(p) if p.label==label) {
            return id;
        }
        pending.extend(&n.children)
    }
    panic!("missing button {label}")
}
#[test]
fn fifo_mounts_only_current_content_and_updates_metadata_without_rebuilding_it() {
    let (app, handle) = fixture(3);
    let built = Rc::new(Cell::new(0));
    let first = built.clone();
    let second = built.clone();
    let a = handle
        .open(modal_request("First", "First details").content(move || {
            first.set(first.get() + 1);
            glory::ui::text().content("First content")
        }))
        .unwrap();
    let b = handle
        .open(
            confirmation_request("Second", "Second details").content(move || {
                second.set(second.get() + 1);
                glory::ui::text().content("Second content")
            }),
        )
        .unwrap();
    assert_eq!(built.get(), 0);
    app.flush().unwrap();
    assert_eq!(built.get(), 1);
    let panel = active(&app, Role::Dialog);
    assert_eq!(handle.pending().unwrap(), 2);
    assert!(
        handle
            .update(&a, "Updated first", "Updated details")
            .unwrap()
    );
    app.flush().unwrap();
    assert_eq!(active(&app, Role::Dialog), panel);
    assert_eq!(built.get(), 1);
    {
        let tree = app.tree();
        let mut pending = tree.roots().to_vec();
        let mut found = false;
        while let Some(id) = pending.pop() {
            let node = tree.node(id).unwrap();
            if matches!(node.descriptor.view().props,glory::render::Props::Text(p) if p.content=="Updated first")
            {
                found = true
            }
            pending.extend(&node.children)
        }
        assert!(found, "updated modal title did not reach retained content");
    }
    assert!(handle.close(&a).unwrap());
    app.flush().unwrap();
    assert_eq!(built.get(), 2);
    active(&app, Role::AlertDialog);
    assert!(!handle.close(&a).unwrap());
    assert!(handle.close(&b).unwrap());
    app.flush().unwrap();
    assert_eq!(handle.pending().unwrap(), 0);
}
#[test]
fn confirmation_results_are_once_and_queued_close_does_not_construct_content() {
    let (app, handle) = fixture(3);
    let log = Rc::new(RefCell::new(Vec::new()));
    let saved = log.clone();
    let a = handle
        .open(
            confirmation_request("Delete", "A destructive action")
                .on_result(move |r| saved.borrow_mut().push(r)),
        )
        .unwrap();
    let built = Rc::new(Cell::new(0));
    let saved = built.clone();
    let results = log.clone();
    let b = handle
        .open(
            modal_request("Queued", "Never mounted")
                .content(move || {
                    saved.set(saved.get() + 1);
                    glory::ui::text().content("never")
                })
                .on_result(move |r| results.borrow_mut().push(r)),
        )
        .unwrap();
    assert!(handle.close(&b).unwrap());
    assert_eq!(built.get(), 0);
    app.flush().unwrap();
    app.dispatch(button(&app, "Confirm"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert_eq!(
        *log.borrow(),
        vec![ModalOutcome::Closed, ModalOutcome::Confirmed]
    );
    assert!(!handle.close(&a).unwrap());
    assert_eq!(handle.pending().unwrap(), 0);
    let saved = log.clone();
    handle
        .open(
            confirmation_request("Continue", "Cancel this operation")
                .on_result(move |r| saved.borrow_mut().push(r)),
        )
        .unwrap();
    app.flush().unwrap();
    app.dispatch(button(&app, "Cancel"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert_eq!(log.borrow().last(), Some(&ModalOutcome::Cancelled));
}
#[test]
fn admission_and_foreign_tokens_validate_before_queue_changes() {
    let (app, handle) = fixture(1);
    let (_other, foreign) = fixture(1);
    for request in [
        modal_request("", "details"),
        modal_request("Title", "x".repeat(16_385)),
        confirmation_request("Title", "details").confirm_label(""),
    ] {
        assert!(handle.open(request).is_err());
        assert_eq!(handle.pending().unwrap(), 0)
    }
    let token = handle.open(modal_request("One", "Details")).unwrap();
    assert!(handle.open(modal_request("Two", "Details")).is_err());
    assert!(foreign.close(&token).is_err());
    assert!(foreign.update(&token, "Wrong", "Wrong").is_err());
    assert!(handle.update(&token, "", "Bad").is_err());
    assert_eq!(handle.pending().unwrap(), 1);
    app.flush().unwrap();
    active(&app, Role::Dialog);
}
#[test]
fn result_callbacks_can_reenter_after_removal_and_close_all_preserves_new_requests() {
    let (app, handle) = fixture(3);
    let command = handle.clone();
    handle
        .open(modal_request("First", "Details").on_result(move |_| {
            assert_eq!(command.pending().unwrap(), 0);
            command
                .open(modal_request("New", "After old queue removal"))
                .unwrap();
        }))
        .unwrap();
    assert_eq!(handle.close_all().unwrap(), 1);
    assert_eq!(handle.pending().unwrap(), 1);
    app.flush().unwrap();
    active(&app, Role::Dialog);
    assert_eq!(handle.close_all().unwrap(), 1);
    assert_eq!(handle.close_all().unwrap(), 0);
}
#[test]
fn owner_retirement_drops_callbacks_without_emission_and_invalidates_all_handles() {
    let app = App::memory();
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    let log = Rc::new(Cell::new(0));
    let count = log.clone();
    let mounted = app
        .scope()
        .mount_with(|| modals_manager("retire").on_ready(move |h| *saved.borrow_mut() = Some(h)))
        .unwrap();
    let handle = out.borrow_mut().take().unwrap();
    let token = handle
        .open(modal_request("Retire", "Details").on_result(move |_| count.set(count.get() + 1)))
        .unwrap();
    app.flush().unwrap();
    drop(mounted);
    app.flush().unwrap();
    assert!(app.tree().is_empty());
    assert_eq!(log.get(), 0);
    assert!(handle.close(&token).is_err());
    assert!(handle.pending().is_err());
}
#[test]
fn bounded_cycles_release_all_request_nodes_and_do_not_keep_history() {
    let (app, handle) = fixture(2);
    let initial = app.tree().len();
    for _ in 0..32 {
        let token = handle.open(modal_request("Transient", "Details")).unwrap();
        app.flush().unwrap();
        handle.close(&token).unwrap();
        app.flush().unwrap();
        assert_eq!(app.tree().len(), initial)
    }
    assert_eq!(handle.pending().unwrap(), 0);
    assert_eq!(app.stats().queries, 0);
}
#[test]
fn imperative_result_callbacks_construct_under_the_manager_owner() {
    let app = App::memory();
    let source = app.scope().cage(4usize);
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    let mounted = app
        .scope()
        .mount_with(|| {
            modals_manager("callback-owner").on_ready(move |h| *saved.borrow_mut() = Some(h))
        })
        .unwrap();
    let handle = out.borrow_mut().take().unwrap();
    let derived = Rc::new(RefCell::new(None));
    let saved = derived.clone();
    let token = handle
        .open(
            modal_request("Callback", "Owner context")
                .on_result(move |_| *saved.borrow_mut() = Some(source.map(|value| value * 2))),
        )
        .unwrap();
    // Called directly from application code rather than an event listener.
    handle.close(&token).unwrap();
    let bond = derived.borrow_mut().take().unwrap();
    assert_eq!(bond.get(), 8);
    drop(mounted);
    app.flush().unwrap();
    assert!(bond.try_get().is_err());
    assert_eq!(source.get(), 4);
}
#[test]
fn triggerless_dialog_requires_an_independent_name_and_retains_valid_reactive_title() {
    for widget in [
        sapid::dialog("missing").show_trigger(false),
        sapid::dialog("empty").show_trigger(false).title(""),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert!(app.tree().is_empty())
    }
    let app = App::memory();
    let title = app.scope().cage("Independent title".to_string());
    app.mount(
        sapid::dialog("named")
            .show_trigger(false)
            .title(title)
            .open(true),
    )
    .unwrap();
    let panel = active(&app, Role::Dialog);
    title.revise(|value| value.clear());
    assert!(app.flush().is_err());
    title.revise(|value| *value = "Recovered".into());
    app.flush().unwrap();
    assert_eq!(active(&app, Role::Dialog), panel);
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        assert!(
            !node
                .descriptor
                .view()
                .classes
                .iter()
                .any(|class| class == "sapid-dialog-trigger")
        );
        pending.extend(&node.children)
    }
}
