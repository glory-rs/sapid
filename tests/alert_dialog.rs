use glory::prelude::*;
use glory::render::{DismissReason, Event, NodeId, Role};
use sapid::{AlertCancelReason, alert_dialog};
use std::{cell::RefCell, rc::Rc};
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let n = tree.node(id).unwrap();
        if n.descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            return id;
        }
        pending.extend(&n.children)
    }
    panic!("missing {class}")
}
fn hidden(app: &App, id: NodeId) -> bool {
    app.tree()
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .hidden
}
#[test]
fn safe_decisions_require_named_description_and_cancel_before_creating_nodes() {
    for widget in [
        alert_dialog("invalid"),
        alert_dialog("invalid").title("Title"),
        alert_dialog("invalid").title(" ").description("Message"),
        alert_dialog("invalid")
            .title("Title")
            .description("Message")
            .show_close(false),
        alert_dialog("invalid")
            .title("Title")
            .description("Message")
            .confirm_label(""),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert!(app.tree().is_empty());
    }
    let app = App::memory();
    app.mount(
        alert_dialog("decision")
            .title("Remove draft?")
            .description("This action removes the local draft."),
    )
    .unwrap();
    let panel = find(&app, "sapid-alert-dialog");
    let cancel = find(&app, "sapid-layer-close");
    let tree = app.tree();
    let d = tree.node(panel).unwrap().descriptor.view();
    assert_eq!(d.role, Some(Role::AlertDialog));
    let layer = d.semantics.as_ref().unwrap().layer.as_ref().unwrap();
    assert!(layer.modal && !layer.outside);
    assert_eq!(
        layer.initial_focus,
        tree.node(cancel)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .id
    );
}
#[test]
fn confirm_cancel_and_escape_have_distinct_ordered_callbacks_and_controlled_refusal() {
    let app = App::memory();
    let open = app.scope().cage(true);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let confirmed = calls.clone();
    let cancelled = calls.clone();
    let changes = calls.clone();
    let dismisses = calls.clone();
    app.mount(
        alert_dialog("controlled")
            .title("Remove draft?")
            .description("Draft will be removed.")
            .open(open)
            .on_confirm(move || confirmed.borrow_mut().push("confirm".into()))
            .on_cancel(move |why| cancelled.borrow_mut().push(format!("cancel:{why:?}")))
            .on_dismiss(move |why| dismisses.borrow_mut().push(format!("dismiss:{why:?}")))
            .on_open_change(move |value| changes.borrow_mut().push(format!("change:{value}"))),
    )
    .unwrap();
    let panel = find(&app, "sapid-alert-dialog");
    app.dispatch(find(&app, "sapid-alert-dialog-confirm"), Event::Activate)
        .unwrap();
    assert!(!hidden(&app, panel));
    assert_eq!(*calls.borrow(), ["confirm", "change:false"]);
    calls.borrow_mut().clear();
    app.dispatch(find(&app, "sapid-layer-close"), Event::Activate)
        .unwrap();
    assert_eq!(*calls.borrow(), ["cancel:Cancel", "change:false"]);
    calls.borrow_mut().clear();
    app.dispatch(panel, Event::Dismiss(DismissReason::Escape))
        .unwrap();
    assert_eq!(
        *calls.borrow(),
        ["cancel:Dismiss(Escape)", "dismiss:Escape", "change:false"]
    );
    assert!(
        app.dispatch(panel, Event::Dismiss(DismissReason::Outside))
            .is_err()
    );
    let count = calls.borrow().len();
    open.revise(|value| *value = false);
    app.flush().unwrap();
    assert!(hidden(&app, panel));
    assert_eq!(calls.borrow().len(), count);
}
#[test]
fn local_decision_closes_and_confirm_enabled_does_not_disable_the_safe_action() {
    let app = App::memory();
    let enabled = app.scope().cage(false);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let confirmed = calls.clone();
    let cancelled = calls.clone();
    app.mount(
        alert_dialog("local")
            .title("Proceed?")
            .description("Confirm the proposed change.")
            .default_open(true)
            .confirm_enabled(enabled)
            .on_confirm(move || confirmed.borrow_mut().push("confirm"))
            .on_cancel(move |why| {
                assert_eq!(why, AlertCancelReason::Cancel);
                cancelled.borrow_mut().push("cancel")
            }),
    )
    .unwrap();
    let panel = find(&app, "sapid-alert-dialog");
    let confirm = find(&app, "sapid-alert-dialog-confirm");
    assert!(!app.dispatch(confirm, Event::Activate).unwrap());
    assert!(!hidden(&app, panel));
    enabled.revise(|value| *value = true);
    app.flush().unwrap();
    assert!(calls.borrow().is_empty());
    app.dispatch(confirm, Event::Activate).unwrap();
    assert!(hidden(&app, panel));
    assert_eq!(*calls.borrow(), ["confirm"]);
}
#[test]
fn callback_retirement_skips_later_change_and_dismiss_handlers() {
    let app = App::memory();
    let mounted = Rc::new(RefCell::new(None::<glory::ui::Mounted>));
    let retire = mounted.clone();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let cancelled = calls.clone();
    let changed = calls.clone();
    let dismissed = calls.clone();
    let handle = app
        .scope()
        .mount_with(|| {
            alert_dialog("retire")
                .title("Proceed?")
                .description("Retire while handling cancellation.")
                .default_open(true)
                .on_cancel(move |_| {
                    cancelled.borrow_mut().push("cancel");
                    let handle = retire.borrow_mut().take();
                    drop(handle);
                })
                .on_dismiss(move |_| dismissed.borrow_mut().push("dismiss"))
                .on_open_change(move |_| changed.borrow_mut().push("change"))
        })
        .unwrap();
    *mounted.borrow_mut() = Some(handle);
    app.flush().unwrap();
    let panel = find(&app, "sapid-alert-dialog");
    app.dispatch(panel, Event::Dismiss(DismissReason::Escape))
        .unwrap();
    assert_eq!(*calls.borrow(), ["cancel"]);
    assert!(app.tree().is_empty());
}
#[test]
fn ssr_uses_alertdialog_modality_safe_focus_and_escaped_description() {
    let html = glory::ssr::render(
        alert_dialog("ssr")
            .title("Remove <draft>?")
            .description("A & B")
            .confirm_variant(sapid::ButtonVariant::Danger)
            .default_open(true),
    )
    .unwrap();
    assert!(html.contains("role=\"alertdialog\""));
    assert!(html.contains("aria-modal=\"true\""));
    assert!(html.contains("aria-describedby="));
    assert!(html.contains("Remove &lt;draft&gt;?"));
    assert!(html.contains("A &amp; B"));
    assert!(html.contains("sapid-button-danger"));
}
