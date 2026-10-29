use glory::apace::shell::WindowApp;
use glory::prelude::*;
use glory::render::{Event, NodeId, Query, QueryValue, Role};
use sapid::{ShellLayout, app_shell, sidebar};
use std::{cell::RefCell, rc::Rc};
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
        .unwrap()
}
fn query(app: &App, query: Query) -> QueryValue {
    let value = Rc::new(RefCell::new(None));
    let out = value.clone();
    app.request(query, move |result| {
        *out.borrow_mut() = Some(result.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    value.borrow_mut().take().unwrap()
}
#[test]
fn native_shell_has_real_landmarks_dock_resize_identity_and_typed_skip_focus_in_four_themes() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let nav = app.scope().cage(None);
        let input = app.scope().cage(None);
        let main = app.scope().cage(None);
        host.mount(
            app_shell("native-shell")
                .theme(theme)
                .breakpoint(600.0)
                .style(Style::new().width(Length::Px(700.0)))
                .main_ref(main)
                .sidebar(sidebar("primary").nav_ref(nav).content(move |_| {
                    glory::ui::input()
                        .node_ref(input)
                        .name("Navigation draft")
                        .value("Keep this draft")
                }))
                .header(|_| glory::ui::text().content("Application header"))
                .main(|_| glory::ui::text().content("Actual main content"))
                .footer(|_| glory::ui::text().content("Application footer")),
        )
        .unwrap();
        app.settle(16).unwrap();
        let nav = nav.get().unwrap();
        let input = input.get().unwrap();
        let engine = host
            .renderer()
            .engine_id(input, glory::html::Part::Main)
            .unwrap();
        for role in [
            Role::Banner,
            Role::Main,
            Role::ContentInfo,
            Role::Navigation,
        ] {
            let id = nodes(app)
                .into_iter()
                .find(|id| app.tree().node(*id).unwrap().descriptor.view().role == Some(role))
                .unwrap();
            let QueryValue::Bounds(rect) = query(app, Query::Bounds(id)) else {
                panic!("bounds")
            };
            assert!(rect.width > 0.0 && rect.height > 0.0, "{role:?}: {rect:?}");
        }
        app.dispatch(class(app, "sapid-sidebar-trigger"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert_eq!(
            host.renderer()
                .engine_id(input, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        assert_eq!(nav, class(app, "sapid-sidebar-nav"));
        app.dispatch(class(app, "sapid-shell-skip"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert_eq!(
            query(app, Query::Focus(main.get().unwrap())),
            QueryValue::Focus(true)
        );
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
}
#[test]
fn native_mobile_layer_rejection_keeps_desktop_navigation_parent_and_accepted_state() {
    let host = WindowApp::new();
    let app = host.app();
    let layout = ShellLayout::new(app.scope(), 1024.0, 600.0).unwrap();
    let nav = app.scope().cage(None);
    let drawer = app.scope().cage(None);
    let handle = Rc::new(RefCell::new(None::<sapid::SidebarHandle>));
    let saved = handle.clone();
    host.mount(
        app_shell("native-reject")
            .layout(layout.clone())
            .style(Style::new().width(Length::Px(700.0)))
            .sidebar(sidebar("primary").nav_ref(nav).drawer_ref(drawer))
            .header(move |context| {
                *saved.borrow_mut() = context.sidebar;
                glory::ui::text().content("Header")
            }),
    )
    .unwrap();
    app.settle(16).unwrap();
    let nav = nav.get().unwrap();
    let parent = app.tree().node(nav).unwrap().parent;
    let engine = host
        .renderer()
        .engine_id(nav, glory::html::Part::Main)
        .unwrap();
    layout.observe_width(500.0).unwrap();
    assert!(app.flush().is_err());
    assert_eq!(app.tree().node(nav).unwrap().parent, parent);
    assert_eq!(
        host.renderer()
            .engine_id(nav, glory::html::Part::Main)
            .unwrap(),
        engine
    );
    assert!(!handle.borrow().as_ref().unwrap().mobile().get());
    assert!(
        app.tree()
            .node(drawer.get().unwrap())
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .layer
            .is_none()
    );
    layout.observe_width(700.0).unwrap();
    app.flush().unwrap();
    app.clear().unwrap();
    assert!(app.tree().is_empty());
}
