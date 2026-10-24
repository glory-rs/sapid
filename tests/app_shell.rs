use glory::prelude::*;
use glory::render::{DismissReason, EventKind, NodeId, Props, Rect, Role};
use sapid::{ShellLayout, SidebarHandle, app_shell, sidebar};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn nodes(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut result = Vec::new();
    while let Some(id) = pending.pop() {
        pending.extend(tree.node(id).unwrap().children.iter().copied());
        result.push(id)
    }
    result
}
fn class(app: &App, class: &str) -> NodeId {
    nodes(app)
        .into_iter()
        .find(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|value| value == class)
        })
        .unwrap_or_else(|| panic!("missing class {class}"))
}
fn resize(width: f64) -> Event {
    Event::Resize {
        size: Rect {
            x: 0.0,
            y: 0.0,
            width,
            height: 600.0,
        },
        viewport: Rect {
            x: 0.0,
            y: 0.0,
            width,
            height: 600.0,
        },
    }
}
#[test]
fn local_layout_is_finite_explicit_readonly_and_driven_by_observations() {
    let app = App::memory();
    for (width, breakpoint) in [
        (f64::NAN, 768.0),
        (1024.0, 0.0),
        (1024.0, f64::INFINITY),
        (-1.0, 768.0),
        (1_000_001.0, 768.0),
    ] {
        assert!(ShellLayout::new(app.scope(), width, breakpoint).is_err());
    }
    let layout = ShellLayout::new(app.scope(), 1024.0, 768.0).unwrap();
    assert_eq!(layout.width().get(), 1024.0);
    assert!(!layout.mobile().get());
    assert!(!layout.observe_width(1024.0).unwrap());
    assert!(layout.observe_width(500.0).unwrap());
    assert!(layout.mobile().get());
    assert!(layout.observe_width(f64::NAN).is_err());
    assert_eq!(layout.width().get(), 500.0);
}
#[test]
fn shell_has_real_landmarks_and_a_captured_skip_link_to_programmatic_main() {
    let app = App::memory();
    app.mount(
        app_shell("landmarks")
            .sidebar(
                sidebar("primary")
                    .name("Workspace navigation")
                    .content(|_| sapid::anchor().href("/docs").label("Docs")),
            )
            .header(|_| glory::ui::text().content("Header"))
            .main(|_| glory::ui::text().content("Main"))
            .footer(|_| glory::ui::text().content("Footer")),
    )
    .unwrap();
    for role in [
        Role::Banner,
        Role::ContentInfo,
        Role::Navigation,
        Role::Main,
    ] {
        assert_eq!(nodes(&app).iter().filter(|id|app.tree().node(**id).unwrap().descriptor.view().role==Some(role)).count(),1);
    }
    let main = class(&app, "sapid-shell-main");
    assert_eq!(
        app.tree()
            .node(class(&app, "sapid-sidebar-nav"))
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Workspace navigation")
    );
    assert_eq!(
        app.tree()
            .node(class(&app, "sapid-sidebar-drawer"))
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Workspace navigation")
    );
    assert_eq!(
        app.tree().node(main).unwrap().descriptor.view().focus,
        glory::render::FocusMode::Programmatic
    );
    let skip = class(&app, "sapid-shell-skip");
    let tree = app.tree();
    let descriptor = tree.node(skip).unwrap().descriptor.view();
    assert!(descriptor.semantics.as_ref().unwrap().link_request);
    assert!(
        matches!(descriptor.props,Props::Link(props)if props.target.starts_with("#sapid-shell-"))
    );
}
#[test]
fn resize_moves_one_navigation_tree_without_rebuilding_owners_refs_or_factories() {
    let app = App::memory();
    let layout = ShellLayout::new(app.scope(), 1024.0, 768.0).unwrap();
    let nav = app.scope().cage(None);
    let drawer = app.scope().cage(None);
    let input = app.scope().cage(None);
    let count = Rc::new(Cell::new(0));
    let built = count.clone();
    app.mount(
        app_shell("stable").layout(layout.clone()).sidebar(
            sidebar("primary")
                .nav_ref(nav)
                .drawer_ref(drawer)
                .content(move |_| {
                    built.set(built.get() + 1);
                    glory::ui::input().node_ref(input).value("Persistent draft")
                }),
        ),
    )
    .unwrap();
    let nav = nav.get().unwrap();
    let input = input.get().unwrap();
    let dock = app.tree().node(nav).unwrap().parent;
    layout.observe_width(500.0).unwrap();
    app.flush().unwrap();
    assert_eq!(app.tree().node(nav).unwrap().parent, drawer.get());
    assert_eq!(count.get(), 1);
    assert!(app.tree().node(input).is_ok());
    layout.observe_width(1200.0).unwrap();
    app.flush().unwrap();
    assert_eq!(app.tree().node(nav).unwrap().parent, dock);
    assert_eq!(count.get(), 1);
    assert!(app.tree().node(input).is_ok());
    assert_eq!(
        nodes(&app)
            .iter()
            .filter(|id| app.tree().node(**id).unwrap().descriptor.view().role
                == Some(Role::Navigation))
            .count(),
        1
    );
}
#[test]
fn controlled_collapse_proposes_without_writing_and_external_changes_are_silent() {
    let app = App::memory();
    let collapsed = app.scope().cage(false);
    let requests = Rc::new(Cell::new(0));
    let out = requests.clone();
    let external = Rc::new(RefCell::new(None));
    let saved = external.clone();
    app.mount(
        app_shell("controlled")
            .sidebar(
                sidebar("primary")
                    .collapsed(collapsed)
                    .default_collapsed(true)
                    .on_collapsed_change(move |next| {
                        assert!(next);
                        out.set(out.get() + 1);
                    }),
            )
            .header(move |context| {
                *saved.borrow_mut() = context.sidebar;
                glory::ui::text().content("Header")
            }),
    )
    .unwrap();
    let handle: SidebarHandle = external.borrow().as_ref().unwrap().clone();
    app.dispatch(class(&app, "sapid-sidebar-trigger"), Event::Activate)
        .unwrap();
    assert!(!collapsed.get());
    assert!(!handle.collapsed().get());
    assert_eq!(requests.get(), 1);
    collapsed.revise(|value| *value = true);
    app.flush().unwrap();
    assert!(handle.collapsed().get());
    assert_eq!(requests.get(), 1);
    let tree = app.tree();
    assert_eq!(
        tree.node(class(&app, "sapid-sidebar"))
            .unwrap()
            .descriptor
            .view()
            .style
            .width,
        Some(Length::Px(72.0))
    );
}
#[test]
fn mobile_drawer_uses_host_layer_close_focus_target_and_typed_callbacks_before_observers() {
    let app = App::memory();
    let trace = Rc::new(RefCell::new(Vec::new()));
    let a = trace.clone();
    let b = trace.clone();
    let c = trace.clone();
    app.mount(
        app_shell("mobile").initial_width(500.0).sidebar(
            sidebar("primary")
                .on_mobile_open_change(move |open| {
                    a.borrow_mut().push(if open { "open" } else { "close" })
                })
                .on(EventKind::Activate, move |_| {
                    b.borrow_mut().push("activate")
                })
                .on(EventKind::Dismiss, move |_| c.borrow_mut().push("dismiss")),
        ),
    )
    .unwrap();
    let dialog = class(&app, "sapid-sidebar-drawer");
    {
        let tree = app.tree();
        let metadata = tree
            .node(dialog)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap();
        let layer = metadata.layer.as_ref().unwrap();
        assert!(layer.modal);
        assert!(
            layer
                .initial_focus
                .as_ref()
                .unwrap()
                .as_str()
                .contains("close")
        );
        assert!(metadata.hidden);
    }
    app.dispatch(class(&app, "sapid-sidebar-trigger"), Event::Activate)
        .unwrap();
    assert_eq!(*trace.borrow(), vec!["open", "activate"]);
    assert!(
        !app.tree()
            .node(dialog)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
    app.dispatch(dialog, Event::Dismiss(DismissReason::Escape))
        .unwrap();
    assert_eq!(
        *trace.borrow(),
        vec!["open", "activate", "close", "dismiss"]
    );
    assert!(
        app.tree()
            .node(dialog)
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
fn resize_transition_and_width_callback_precede_generic_observer() {
    let app = App::memory();
    let root = app.scope().cage(None);
    let trace = Rc::new(RefCell::new(Vec::new()));
    let a = trace.clone();
    let b = trace.clone();
    let handle = Rc::new(RefCell::new(None::<SidebarHandle>));
    let saved = handle.clone();
    let state = handle.clone();
    app.mount(
        app_shell("resize")
            .node_ref(root)
            .sidebar(sidebar("primary"))
            .header(move |context| {
                *saved.borrow_mut() = context.sidebar;
                glory::ui::text().content("Header")
            })
            .on_width_change(move |width| {
                assert_eq!(width, 500.0);
                assert!(state.borrow().as_ref().unwrap().mobile().get());
                a.borrow_mut().push("typed");
            })
            .on(EventKind::Resize, move |_| b.borrow_mut().push("generic")),
    )
    .unwrap();
    app.dispatch(root.get().unwrap(), resize(500.0)).unwrap();
    assert_eq!(*trace.borrow(), vec!["typed", "generic"]);
    app.dispatch(root.get().unwrap(), resize(500.0)).unwrap();
    assert_eq!(*trace.borrow(), vec!["typed", "generic", "generic"]);
}
#[test]
fn external_handle_cannot_keep_retired_callbacks_or_node_references_live() {
    struct Probe(Cage<usize>);
    impl Drop for Probe {
        fn drop(&mut self) {
            self.0.revise(|value| *value += 1);
        }
    }
    let app = App::memory();
    let dropped = app.scope().cage(0);
    let probe = Probe(dropped);
    let reference = app.scope().cage(None);
    let handle = Rc::new(RefCell::new(None::<SidebarHandle>));
    let saved = handle.clone();
    app.mount(
        app_shell("retire")
            .sidebar(
                sidebar("primary")
                    .node_ref(reference)
                    .on_collapsed_change(move |_| {
                        let _ = &probe;
                    }),
            )
            .header(move |context| {
                *saved.borrow_mut() = context.sidebar;
                glory::ui::text().content("Header")
            }),
    )
    .unwrap();
    assert!(reference.get().is_some());
    app.clear().unwrap();
    assert!(reference.get().is_none());
    assert_eq!(dropped.get(), 1);
    assert!(handle.borrow().as_ref().unwrap().toggle().is_err());
}
#[test]
fn standalone_sidebar_requires_explicit_layout_and_invalid_widths_reject_before_nodes() {
    let app = App::memory();
    assert!(app.mount(sidebar("missing")).is_err());
    let layout = ShellLayout::new(app.scope(), 1024.0, 768.0).unwrap();
    for width in [0.0, -1.0, f64::INFINITY] {
        assert!(
            app.mount(sidebar("invalid").layout(layout.clone()).width(width))
                .is_err()
        );
        assert!(app.tree().roots().is_empty());
    }
    assert!(
        app.mount(
            sidebar("invalid")
                .layout(layout)
                .width(64.0)
                .rail_width(72.0)
        )
        .is_err()
    );
    assert!(app.tree().roots().is_empty());
}
#[test]
fn foreign_or_retired_layout_rejects_before_creating_the_shell() {
    let first = App::memory();
    let second = App::memory();
    let foreign = ShellLayout::new(first.scope(), 1024.0, 768.0).unwrap();
    assert!(
        second
            .mount(app_shell("foreign").layout(foreign.clone()))
            .is_err()
    );
    assert!(second.tree().roots().is_empty());
    assert!(second.mount(sidebar("foreign").layout(foreign)).is_err());
    assert!(second.tree().roots().is_empty());
    let owner = second.scope().child().unwrap();
    let retired = ShellLayout::new(&owner, 1024.0, 768.0).unwrap();
    drop(owner);
    assert!(second.mount(app_shell("retired").layout(retired)).is_err());
    assert!(second.tree().roots().is_empty());
}
#[test]
fn ssr_has_header_footer_main_single_nav_and_explicit_initial_mobile_layer() {
    let html = glory::ssr::render(
        app_shell("ssr")
            .initial_width(500.0)
            .sidebar(
                sidebar("primary")
                    .default_mobile_open(true)
                    .content(|_| sapid::anchor().href("/home").label("Home")),
            )
            .header(|_| glory::ui::text().content("Site header"))
            .main(|_| glory::ui::text().content("Main"))
            .footer(|_| glory::ui::text().content("Site footer")),
    )
    .unwrap();
    assert!(html.contains("<header"));
    assert!(html.contains("<footer"));
    assert!(html.contains("<main"));
    assert_eq!(html.matches("<nav").count(), 1);
    assert!(html.contains("data-gly-layer=\"left\""));
    assert!(html.contains("aria-modal=\"true\""));
    assert!(html.contains("data-gly-link-request"));
}
