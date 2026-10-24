use glory::render::{DismissReason, Event, LayerPlacement, NodeId, Props, Role};
use glory::ui::App;
use sapid::{FloatingWindowHandle, WindowBounds, WindowCommand, WindowLimits, floating_window};
use std::{cell::RefCell, rc::Rc};

fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    while let Some(id) = todo.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            return id;
        }
        todo.extend(&node.children)
    }
    panic!("missing {class}")
}
fn geometry(app: &App, panel: NodeId) -> WindowBounds {
    let tree = app.tree();
    let d = tree.node(panel).unwrap().descriptor.view();
    let LayerPlacement::Viewport(p) = d
        .semantics
        .as_ref()
        .unwrap()
        .layer
        .as_ref()
        .unwrap()
        .placement
    else {
        panic!("not viewport")
    };
    let (Some(glory::style::Length::Px(width)), Some(glory::style::Length::Px(height))) =
        (d.style.width, d.style.height)
    else {
        panic!("not dimensions")
    };
    WindowBounds {
        x: p.x,
        y: p.y,
        width,
        height,
    }
}
#[test]
fn window_is_a_named_nonmodal_dialog_with_true_buttons_and_pointer_regions() {
    let app = App::memory();
    app.mount(
        floating_window("one")
            .title("Editor window")
            .content(|| glory::ui::input().name("Window draft")),
    )
    .unwrap();
    let panel = find(&app, "sapid-floating-window");
    let tree = app.tree();
    let d = tree.node(panel).unwrap().descriptor.view();
    assert_eq!(d.role, Some(Role::Dialog));
    let semantics = d.semantics.as_ref().unwrap();
    let layer = semantics.layer.as_ref().unwrap();
    assert!(!layer.modal && !layer.outside && !semantics.hidden);
    assert!(semantics.labelled_by.is_some() && layer.anchor.is_none());
    for class in ["sapid-window-caption", "sapid-window-resize-grip"] {
        let id = find(&app, class);
        assert!(
            tree.node(id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .drag_capture
                .is_some()
        );
    }
    assert!(matches!(
        tree.node(find(&app, "sapid-window-close"))
            .unwrap()
            .descriptor
            .view()
            .props,
        Props::Button(_)
    ));
    drop(tree);
    assert_eq!(geometry(&app, panel), WindowBounds::default());
}
#[test]
fn finite_initial_configuration_rejects_before_any_ui_nodes() {
    for widget in [
        floating_window("bad").title(" "),
        floating_window("bad").title("x".repeat(1_000_000)),
        floating_window("bad").default_bounds(WindowBounds {
            x: f64::NAN,
            ..Default::default()
        }),
        floating_window("bad").limits(WindowLimits {
            max_width: 100.,
            ..Default::default()
        }),
        floating_window("bad").keyboard_step(0.),
        floating_window("bad").viewport_padding(4097.),
        floating_window("bad")
            .style(glory::style::Style::new().width(glory::style::Length::Px(3.))),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert!(app.tree().is_empty());
    }
}
#[test]
fn keyboard_button_commands_apply_local_bounds_and_observe_after_internal_behavior() {
    let app = App::memory();
    let handle = Rc::new(RefCell::new(None::<FloatingWindowHandle>));
    let output = handle.clone();
    let proposals = Rc::new(RefCell::new(Vec::new()));
    let observed = proposals.clone();
    app.mount(
        floating_window("local")
            .on_ready(move |value| *output.borrow_mut() = Some(value))
            .on_bounds_change(move |next| observed.borrow_mut().push(next)),
    )
    .unwrap();
    let handle = handle.borrow().clone().unwrap();
    let panel = find(&app, "sapid-floating-window");
    handle.command(WindowCommand::MoveRight).unwrap();
    app.flush().unwrap();
    assert_eq!(geometry(&app, panel).x, 58.);
    handle.command(WindowCommand::Taller).unwrap();
    app.flush().unwrap();
    assert_eq!(geometry(&app, panel).height, 290.);
    assert_eq!(proposals.borrow().len(), 2);
    handle.close().unwrap();
    app.flush().unwrap();
    assert!(
        app.tree()
            .node(panel)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
}

#[test]
fn commands_in_one_tick_accumulate_against_the_current_local_or_accepted_controlled_source() {
    for controlled in [false, true] {
        let app = App::memory();
        let source = app.scope().cage(WindowBounds::default());
        let handle = Rc::new(RefCell::new(None::<FloatingWindowHandle>));
        let output = handle.clone();
        let widget =
            floating_window("tick").on_ready(move |value| *output.borrow_mut() = Some(value));
        let widget = if controlled {
            widget
                .bounds(source)
                .on_bounds_change(move |next| source.revise(|value| *value = next))
        } else {
            widget
        };
        app.mount(widget).unwrap();
        let handle = handle.borrow().clone().unwrap();
        handle.command(WindowCommand::MoveRight).unwrap();
        handle.command(WindowCommand::MoveRight).unwrap();
        handle.command(WindowCommand::Wider).unwrap();
        app.flush().unwrap();
        let bounds = geometry(&app, find(&app, "sapid-floating-window"));
        assert_eq!((bounds.x, bounds.width), (68., 410.));
    }
}
#[test]
fn controlled_geometry_and_close_are_atomic_proposals_and_refusal_retains_content() {
    let app = App::memory();
    let bounds = app.scope().cage(WindowBounds::default());
    let open = app.scope().cage(true);
    let handle = Rc::new(RefCell::new(None::<FloatingWindowHandle>));
    let output = handle.clone();
    let proposals = Rc::new(RefCell::new(Vec::new()));
    let observed = proposals.clone();
    app.mount(
        floating_window("controlled")
            .bounds(bounds)
            .open(open)
            .on_ready(move |value| *output.borrow_mut() = Some(value))
            .on_bounds_change(move |next| observed.borrow_mut().push(next))
            .content(|| glory::ui::input().class("persistent-draft")),
    )
    .unwrap();
    let handle = handle.borrow().clone().unwrap();
    let panel = find(&app, "sapid-floating-window");
    let editor = find(&app, "persistent-draft");
    handle.command(WindowCommand::Wider).unwrap();
    app.flush().unwrap();
    assert_eq!(geometry(&app, panel), WindowBounds::default());
    assert_eq!(proposals.borrow()[0].width, 410.);
    bounds.revise(|value| {
        *value = WindowBounds {
            x: 90.,
            y: 130.,
            width: 420.,
            height: 320.,
        }
    });
    app.flush().unwrap();
    assert_eq!(geometry(&app, panel), bounds.get());
    assert_eq!(proposals.borrow().len(), 1);
    app.dispatch(panel, Event::Dismiss(DismissReason::Escape))
        .unwrap();
    assert!(open.get());
    assert!(
        !app.tree()
            .node(panel)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
    open.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(app.tree().node(editor).is_ok());
    open.revise(|v| *v = true);
    app.flush().unwrap();
    assert_eq!(find(&app, "persistent-draft"), editor);
}
#[test]
fn disabled_and_retired_window_handles_never_mutate_or_resurrect_the_owner() {
    let app = App::memory();
    let enabled = app.scope().cage(false);
    let handle = Rc::new(RefCell::new(None::<FloatingWindowHandle>));
    let output = handle.clone();
    app.mount(
        floating_window("owner")
            .enabled(enabled)
            .on_ready(move |value| *output.borrow_mut() = Some(value)),
    )
    .unwrap();
    let handle = handle.borrow().clone().unwrap();
    let panel = find(&app, "sapid-floating-window");
    handle.command(WindowCommand::MoveRight).unwrap();
    app.flush().unwrap();
    assert_eq!(geometry(&app, panel), WindowBounds::default());
    enabled.revise(|v| *v = true);
    app.flush().unwrap();
    handle.command(WindowCommand::MoveRight).unwrap();
    app.flush().unwrap();
    assert_eq!(geometry(&app, panel).x, 58.);
    app.clear().unwrap();
    assert!(handle.command(WindowCommand::MoveLeft).is_err());
    assert!(handle.close().is_err());
    assert!(app.tree().is_empty());
}
#[test]
fn unmount_content_policy_retires_the_draft_while_preserving_the_window_shell() {
    let app = App::memory();
    let open = app.scope().cage(true);
    app.mount(
        floating_window("lifetime")
            .open(open)
            .content_policy(sapid::ContentPolicy::Unmount)
            .content(|| glory::ui::input().class("retired-draft")),
    )
    .unwrap();
    let panel = find(&app, "sapid-floating-window");
    let editor = find(&app, "retired-draft");
    open.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(app.tree().node(editor).is_err());
    assert!(app.tree().node(panel).is_ok());
    open.revise(|v| *v = true);
    app.flush().unwrap();
    assert_ne!(find(&app, "retired-draft"), editor);
    assert_eq!(find(&app, "sapid-floating-window"), panel);
}
