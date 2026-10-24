#![cfg(feature = "chat")]
use glory::{
    prelude::*,
    render::{NodeId, Props},
};
use sapid::chat::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn all(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = pending.pop() {
        pending.extend(&tree.node(id).unwrap().children);
        out.push(id)
    }
    out
}
fn class(app: &App, name: &str) -> NodeId {
    all(app)
        .into_iter()
        .find(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|class| class == name)
        })
        .unwrap()
}
fn content(app: &App) -> String {
    all(app)
        .into_iter()
        .filter_map(
            |id| match app.tree().node(id).unwrap().descriptor.view().props {
                Props::Text(p) => Some(p.content.clone()),
                _ => None,
            },
        )
        .collect::<Vec<_>>()
        .join("|")
}
fn enabled(app: &App, id: NodeId) -> bool {
    matches!(app.tree().node(id).unwrap().descriptor.view().props,Props::Button(p)if p.enabled)
}
#[test]
fn confirmation_only_proposes_complete_key_revision_and_refusal_keeps_current_status() {
    let app = App::memory();
    let data = app.scope().cage(ConfirmationData {
        revision: 17,
        ..Default::default()
    });
    let out = Rc::new(RefCell::new(vec![]));
    let requests = out.clone();
    app.mount(
        confirmation("delete-draft")
            .data(data)
            .on_decision(move |request| requests.borrow_mut().push(request)),
    )
    .unwrap();
    let root = class(&app, "sapid-confirmation");
    let approve = class(&app, "sapid-confirmation-approve");
    let reject = class(&app, "sapid-confirmation-reject");
    let before = data.get();
    app.dispatch(approve, Event::Activate).unwrap();
    app.dispatch(reject, Event::Activate).unwrap();
    assert_eq!(data.get(), before);
    assert!(content(&app).contains("Awaiting confirmation"));
    assert_eq!(
        *out.borrow(),
        [
            ConfirmationRequest {
                key: "delete-draft".into(),
                revision: 17,
                decision: ConfirmationDecision::Approve
            },
            ConfirmationRequest {
                key: "delete-draft".into(),
                revision: 17,
                decision: ConfirmationDecision::Reject
            }
        ]
    );
    data.revise(|value| {
        value.status = ConfirmationStatus::Approved;
        value.revision = 18;
    });
    app.flush().unwrap();
    assert_eq!(class(&app, "sapid-confirmation"), root);
    assert_eq!(class(&app, "sapid-confirmation-approve"), approve);
    assert!(!enabled(&app, approve) && !enabled(&app, reject));
    assert!(content(&app).contains("Approved by application"));
    app.dispatch(approve, Event::Activate).unwrap();
    assert_eq!(out.borrow().len(), 2);
}
#[test]
fn application_alone_accepts_decision_and_source_changes_are_silent() {
    let app = App::memory();
    let data = app.scope().cage(ConfirmationData::default());
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    app.mount(
        confirmation("release")
            .data(data)
            .on_decision(move |request| {
                count.set(count.get() + 1);
                data.revise(|value| {
                    value.status = match request.decision {
                        ConfirmationDecision::Approve => ConfirmationStatus::Approved,
                        ConfirmationDecision::Reject => ConfirmationStatus::Rejected,
                    }
                });
            }),
    )
    .unwrap();
    app.dispatch(class(&app, "sapid-confirmation-reject"), Event::Activate)
        .unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(data.get().status, ConfirmationStatus::Rejected);
    data.revise(|value| value.status = ConfirmationStatus::Cancelled);
    app.flush().unwrap();
    assert_eq!(calls.get(), 1);
    assert!(content(&app).contains("Cancelled by application"));
}
#[test]
fn checkpoint_restore_is_a_proposal_with_explicit_application_state_and_no_fake_rollback() {
    let app = App::memory();
    let data = app.scope().cage(CheckpointData {
        revision: 8,
        description: "Preserved draft <script>literal</script>".into(),
        ..Default::default()
    });
    let out = Rc::new(RefCell::new(vec![]));
    let requests = out.clone();
    app.mount(
        checkpoint("draft-saved")
            .data(data)
            .on_restore(move |request| requests.borrow_mut().push(request)),
    )
    .unwrap();
    let restore = class(&app, "sapid-checkpoint-restore");
    let initial = data.get();
    app.dispatch(restore, Event::Activate).unwrap();
    assert_eq!(data.get(), initial);
    assert_eq!(
        *out.borrow(),
        [CheckpointRequest {
            key: "draft-saved".into(),
            revision: 8
        }]
    );
    for status in [
        CheckpointStatus::Current,
        CheckpointStatus::Restoring,
        CheckpointStatus::Restored,
        CheckpointStatus::Unavailable,
    ] {
        data.revise(|value| value.status = status);
        app.flush().unwrap();
        assert_eq!(class(&app, "sapid-checkpoint-restore"), restore);
        assert!(!enabled(&app, restore));
        app.dispatch(restore, Event::Activate).unwrap();
        assert_eq!(out.borrow().len(), 1);
        assert!(content(&app).contains(status.label()));
    }
    data.revise(|value| {
        value.status = CheckpointStatus::Available;
        value.revision = 9;
    });
    app.flush().unwrap();
    app.dispatch(restore, Event::Activate).unwrap();
    assert_eq!(out.borrow()[1].revision, 9);
}
#[test]
fn explicit_disable_absent_callback_and_owner_cleanup_prevent_requests() {
    let app = App::memory();
    let active = app.scope().cage(false);
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    let sentinel = Rc::new(());
    let weak = Rc::downgrade(&sentinel);
    app.mount(
        group().fill((
            confirmation("disabled")
                .enabled(active)
                .on_decision(move |_| {
                    let _ = &sentinel;
                    count.set(count.get() + 1);
                }),
            checkpoint("no-handler"),
        )),
    )
    .unwrap();
    let button = class(&app, "sapid-confirmation-approve");
    assert!(!enabled(&app, button));
    assert!(!enabled(&app, class(&app, "sapid-checkpoint-restore")));
    app.dispatch(button, Event::Activate).unwrap();
    assert_eq!(calls.get(), 0);
    active.revise(|value| *value = true);
    app.flush().unwrap();
    app.dispatch(button, Event::Activate).unwrap();
    assert_eq!(calls.get(), 1);
    app.clear().unwrap();
    assert!(weak.upgrade().is_none());
    assert_eq!(app.stats().nodes, 0);
    app.dispatch(button, Event::Activate).unwrap();
    assert_eq!(calls.get(), 1);
}
#[test]
fn finite_keys_labels_text_and_total_budget_reject_before_allocating_or_cloning_large_values() {
    for key in ["", "contains space", &"a".repeat(65)] {
        let app = App::memory();
        assert!(app.mount(confirmation(key)).is_err());
        assert_eq!(app.stats().nodes, 0);
    }
    let mut data = ConfirmationData {
        title: "x".repeat(257),
        ..Default::default()
    };
    assert!(data.validate().is_err());
    data.title = "Valid".into();
    data.description = "x".repeat(8193);
    assert!(data.validate().is_err());
    data.description = "x".repeat(8192);
    data.details = "x".repeat(24576);
    assert!(data.validate().is_err());
    data.details.truncate(24000);
    data.validate().unwrap();
    data.details = "bad\rtext".into();
    assert!(data.validate().is_err());
    let mut labels = ConfirmationLabels {
        approve: "Accept\nrequest".into(),
        ..Default::default()
    };
    assert!(labels.validate().is_err());
    labels.approve = "Accept request".into();
    labels.validate().unwrap();
    let app = App::memory();
    assert!(app.mount(confirmation("invalid").data(data)).is_err());
    assert_eq!(app.stats().nodes, 0);
    let invalid = CheckpointData {
        details: "x".repeat(24577),
        ..Default::default()
    };
    assert!(invalid.validate().is_err());
    let labels = CheckpointLabels {
        restore: " ".into(),
        ..Default::default()
    };
    assert!(labels.validate().is_err());
}
#[test]
fn invalid_external_data_retains_complete_accepted_view_and_recovers_without_proposals() {
    let app = App::memory();
    let data = app.scope().cage(ConfirmationData::default());
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    app.mount(
        confirmation("live")
            .data(data)
            .on_decision(move |_| count.set(count.get() + 1)),
    )
    .unwrap();
    let id = class(&app, "sapid-confirmation");
    data.revise(|value| value.title = "Accepted updated title".into());
    app.flush().unwrap();
    data.revise(|value| value.details = "x".repeat(24577));
    assert!(app.flush().is_err());
    assert_eq!(class(&app, "sapid-confirmation"), id);
    assert!(content(&app).contains("Accepted updated title"));
    data.revise(|value| value.details = "Recovered".into());
    app.flush().unwrap();
    assert!(content(&app).contains("Recovered"));
    assert_eq!(calls.get(), 0);
}
#[test]
fn four_theme_ssr_has_real_group_heading_button_and_literal_details_unstyled_keeps_geometry() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let html = glory::ssr::render(
            group().fill((
                confirmation("confirm")
                    .theme(theme.clone())
                    .data(ConfirmationData {
                        details: "<script>alert('literal')</script>".into(),
                        ..Default::default()
                    }),
                checkpoint("checkpoint").theme(theme),
            )),
        )
        .unwrap();
        assert!(
            html.contains("role=\"group\"") && html.contains("<h3") && html.contains("<button")
        );
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>alert"));
        assert!(!html.contains("role=\"alertdialog\""));
    }
    let app = App::memory();
    app.mount(group().fill((
        confirmation("plain").unstyled(),
        checkpoint("plain-checkpoint").unstyled(),
    )))
    .unwrap();
    for name in ["sapid-confirmation", "sapid-checkpoint"] {
        let tree = app.tree();
        let node = tree.node(class(&app, name)).unwrap().descriptor.view();
        assert!(node.style.background.is_none());
        assert_eq!(node.style.min_width, Some(glory::style::Length::Px(0.)));
    }
}
