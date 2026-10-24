use glory::render::{DismissReason, Event, LayerPlacement, NodeId, Semantics};
use glory::ui::App;
use sapid::{
    AnchorAlign, AnchorPlacement, AnchorSide, ContentPolicy, DrawerSide, dialog, drawer,
    popover,
};
use std::cell::RefCell;
use std::rc::Rc;

fn nodes(app: &App, class: &str) -> Vec<NodeId> {
    fn visit(app: &App, id: NodeId, class: &str, result: &mut Vec<NodeId>) {
        let tree = app.tree();
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|s| s == class) {
            result.push(id);
        }
        for child in &node.children {
            visit(app, *child, class, result);
        }
    }
    let mut result = Vec::new();
    for id in app.tree().roots() {
        visit(app, *id, class, &mut result);
    }
    result
}
fn metadata(app: &App, id: NodeId) -> Semantics {
    *app.tree()
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .clone()
        .unwrap()
}
#[test]
fn shared_layer_state_relations_and_local_dismissals() {
    let app = App::memory();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let copy = calls.clone();
    app.mount(
        dialog("settings")
            .label("Settings")
            .title("Preferences")
            .description("Configure this app")
            .on_open_change(move |v| copy.borrow_mut().push(v))
            .content(glory::ui::input),
    )
    .unwrap();
    let trigger = nodes(&app, "sapid-dialog-trigger")[0];
    let panel = nodes(&app, "sapid-dialog")[0];
    assert!(metadata(&app, panel).hidden);
    assert!(nodes(&app, "sapid-layer-close").len() == 1);
    app.dispatch(trigger, Event::Activate).unwrap();
    app.flush().unwrap();
    let m = metadata(&app, panel);
    assert!(!m.hidden && m.layer.as_ref().unwrap().modal);
    assert_eq!(metadata(&app, trigger).controls, m.id);
    assert_eq!(m.layer.unwrap().return_focus, metadata(&app, trigger).id);
    app.dispatch(panel, Event::Dismiss(DismissReason::Escape))
        .unwrap();
    app.flush().unwrap();
    assert!(metadata(&app, panel).hidden);
    assert_eq!(*calls.borrow(), [true, false]);
    assert_eq!(nodes(&app, "sapid-dialog")[0], panel);
}
#[test]
fn controlled_rejection_and_external_state_changes_are_silent() {
    let app = App::memory();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let copy = calls.clone();
    let state = app.scope().cage(true);
    app.mount(
        popover("controlled")
            .open(state)
            .on_open_change(move |v| copy.borrow_mut().push(v)),
    )
    .unwrap();
    let panel = nodes(&app, "sapid-popover")[0];
    app.dispatch(panel, Event::Dismiss(DismissReason::Outside))
        .unwrap();
    app.flush().unwrap();
    assert!(!metadata(&app, panel).hidden);
    assert_eq!(*calls.borrow(), [false]);
    state.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(metadata(&app, panel).hidden);
    assert_eq!(*calls.borrow(), [false]);
}
#[test]
fn popover_slots_keep_intrinsic_buttons_accessible_names_controlled_refusal_and_owner_cleanup()
{
    use glory::render::Props;
    let app = App::memory();
    let state = app.scope().cage(true);
    let capture = Rc::new(());
    let slot = capture.clone();
    app.mount(
        popover("slots")
            .label("Details")
            .close_label("Dismiss details")
            .open(state)
            .trigger_content(move || {
                let _ = &slot;
                glory::ui::text().content("Custom trigger")
            })
            .close_content(|| glory::ui::text().content("Custom close")),
    )
    .unwrap();
    for (class, name) in [
        ("sapid-popover-trigger", "Details"),
        ("sapid-layer-close", "Dismiss details"),
    ] {
        let id = nodes(&app, class)[0];
        let tree = app.tree();
        let d = tree.node(id).unwrap().descriptor.view();
        assert!(matches!(d.props, Props::Button(_)));
        assert_eq!(d.name.as_deref(), Some(name));
    }
    let close = nodes(&app, "sapid-layer-close")[0];
    app.dispatch(close, Event::Activate).unwrap();
    assert!(!metadata(&app, nodes(&app, "sapid-popover")[0]).hidden);
    assert!(state.get());
    app.clear().unwrap();
    assert_eq!(Rc::strong_count(&capture), 1);
}

#[test]
fn button_slots_reject_interactive_descendants_through_transparent_groups() {
    for close in [false, true] {
        let app = App::memory();
        let p = popover("invalid-slot");
        let p = if close {
            p.close_content(|| glory::ui::group().fill(glory::ui::input()))
        } else {
            p.trigger_content(|| glory::ui::group().fill(glory::ui::button()))
        };
        assert!(app.mount(p).is_err());
    }
}

#[test]
fn popover_anchor_loss_is_verified_by_the_shared_model_before_request_callbacks() {
    let app = App::memory();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let out = calls.clone();
    app.mount(
        popover("anchor-loss")
            .default_open(true)
            .on_dismiss(move |reason| out.borrow_mut().push(reason)),
    )
    .unwrap();
    let panel = nodes(&app, "sapid-popover")[0];
    let trigger = nodes(&app, "sapid-popover-trigger")[0];
    app.dispatch(panel, Event::Dismiss(DismissReason::AnchorLost))
        .unwrap();
    assert!(calls.borrow().is_empty());
    app.scope()
        .patch(
            trigger,
            glory::render::Patch::Style(glory::render::Change::Set(Box::new(
                glory::style::Style::new().display(glory::style::Display::None),
            ))),
        )
        .unwrap();
    app.dispatch(panel, Event::Dismiss(DismissReason::AnchorLost))
        .unwrap();
    assert_eq!(*calls.borrow(), [DismissReason::AnchorLost]);
    assert!(metadata(&app, panel).hidden);
}
#[test]
fn drawer_edges_popover_alignment_and_dismiss_policy_use_typed_metadata() {
    for side in [
        DrawerSide::Left,
        DrawerSide::Right,
        DrawerSide::Top,
        DrawerSide::Bottom,
    ] {
        let app = App::memory();
        app.mount(
            drawer("edge")
                .side(side)
                .default_open(true)
                .dismiss_on_escape(false),
        )
        .unwrap();
        let panel = nodes(&app, "sapid-drawer")[0];
        let l = metadata(&app, panel).layer.unwrap();
        assert!(l.modal && !l.escape && l.outside);
        assert_eq!(
            l.placement,
            match side {
                DrawerSide::Left => LayerPlacement::Left,
                DrawerSide::Right => LayerPlacement::Right,
                DrawerSide::Top => LayerPlacement::Top,
                DrawerSide::Bottom => LayerPlacement::Bottom,
            }
        );
        assert!(
            app.dispatch(panel, Event::Dismiss(DismissReason::Escape))
                .is_err()
        );
    }
    let app = App::memory();
    app.mount(
        popover("align").placement(AnchorPlacement::new(AnchorSide::Bottom, AnchorAlign::End)),
    )
    .unwrap();
    let l = metadata(&app, nodes(&app, "sapid-popover")[0])
        .layer
        .unwrap();
    assert!(!l.modal);
    assert_eq!(
        l.placement,
        LayerPlacement::Anchor(glory::render::AnchorPlacement::new(
            glory::render::AnchorSide::Bottom,
            glory::render::AnchorAlign::End
        ))
    );
    assert!(l.anchor.is_some());
}
#[test]
fn content_policy_retires_or_preserves_child_owners_and_hidden_events() {
    for policy in [ContentPolicy::Unmount, ContentPolicy::KeepMounted] {
        let app = App::memory();
        let value = app.scope().cage(None);
        app.mount(
            dialog("life")
                .content_policy(policy)
                .content(move || glory::ui::input().node_ref(value)),
        )
        .unwrap();
        assert_eq!(value.get().is_some(), policy == ContentPolicy::KeepMounted);
        let trigger = nodes(&app, "sapid-dialog-trigger")[0];
        app.dispatch(trigger, Event::Activate).unwrap();
        app.flush().unwrap();
        let first = value.get().unwrap();
        let panel = nodes(&app, "sapid-dialog")[0];
        app.dispatch(panel, Event::Dismiss(DismissReason::Outside))
            .unwrap();
        app.flush().unwrap();
        assert_eq!(value.get().is_some(), policy == ContentPolicy::KeepMounted);
        app.dispatch(trigger, Event::Activate).unwrap();
        app.flush().unwrap();
        assert_eq!(
            value.get() == Some(first),
            policy == ContentPolicy::KeepMounted
        );
    }
}
#[test]
fn ssr_layer_names_escaping_and_open_state_are_deterministic() {
    let a = glory::ssr::snapshot(
        dialog("ssr")
            .title("<unsafe>")
            .description("details")
            .default_open(true),
    )
    .unwrap();
    let b = glory::ssr::snapshot(
        dialog("ssr")
            .title("<unsafe>")
            .description("details")
            .default_open(true),
    )
    .unwrap();
    assert_eq!(a.html, b.html);
    for text in [
        "popover=\"manual\"",
        "aria-modal=\"true\"",
        "aria-haspopup=\"dialog\"",
        "aria-describedby=",
        "&lt;unsafe&gt;",
    ] {
        assert!(a.html.contains(text), "{text}: {}", a.html);
    }
}

#[test]
fn dismiss_callbacks_precede_observers_and_hidden_requests_do_not_escape() {
    use glory::render::EventKind;
    let app = App::memory();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let dismiss = calls.clone();
    let change = calls.clone();
    let observed = calls.clone();
    app.mount(
        dialog("order")
            .default_open(true)
            .on_dismiss(move |reason| {
                assert_eq!(reason, DismissReason::Outside);
                dismiss.borrow_mut().push("dismiss");
            })
            .on_open_change(move |_| change.borrow_mut().push("change"))
            .on(EventKind::Dismiss, move |_| {
                observed.borrow_mut().push("observer")
            }),
    )
    .unwrap();
    let panel = nodes(&app, "sapid-dialog")[0];
    app.dispatch(panel, Event::Dismiss(DismissReason::Outside))
        .unwrap();
    assert_eq!(*calls.borrow(), ["dismiss", "change", "observer"]);
    app.dispatch(panel, Event::Dismiss(DismissReason::Outside))
        .unwrap();
    assert_eq!(calls.borrow().len(), 3);
    app.clear().unwrap();
    assert_eq!(app.tree().len(), 0);
}
