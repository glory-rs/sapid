use glory::blitz::shell::WindowApp;
use glory::prelude::*;
use glory::render::{Current, Event, NodeId, Props, Query, QueryValue};
use sapid::{breadcrumb_item, breadcrumbs, nav_link, pagination};
use std::{cell::RefCell, rc::Rc};
fn all(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut result = Vec::new();
    while let Some(id) = pending.pop() {
        pending.extend(tree.node(id).unwrap().children.iter().copied());
        result.push(id)
    }
    result
}
fn button(app: &App, label: &str) -> NodeId {
    all(app).into_iter().find(|id|matches!(&app.tree().node(*id).unwrap().descriptor.view().props.clone(),Props::Button(p)if p.label==label)).unwrap()
}
fn bounds(app: &App, node: NodeId) -> glory::render::Rect {
    let reply = Rc::new(RefCell::new(None));
    let out = reply.clone();
    app.request(Query::Bounds(node), move |value| {
        *out.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = reply.borrow_mut().take().unwrap() else {
        panic!("bounds")
    };
    rect
}
fn dump(host: &WindowApp, label: &str) {
    if std::env::var_os("SAPID_NAVIGATION_DUMP").is_none() {
        return;
    }
    host.renderer().inspect(|doc| {
        println!("=== {label} ===");
        let mut pending=vec![doc.root_node().id];let mut seen=std::collections::HashSet::new();
        while let Some(id)=pending.pop() {
            if !seen.insert(id) { continue; }
            let Some(node)=doc.get_node(id) else { println!("MISSING {id:?}");continue; };
            let layout=node.layout_children.borrow().clone();
            println!("{id:?} kind={:?} tag={:?} attrs={:?} dom_parent={:?} layout_parent={:?} children={:?} layout={:?} anonymous={:?} paint={:?} flags={:?} display={:?}",node.data.kind(),node.element_data().map(|e|e.name.local.to_string()),node.element_data().map(|e|e.attrs.iter().map(|a|(a.name.local.to_string(),a.value.clone())).collect::<Vec<_>>()),node.parent,node.layout_parent.get(),node.children,layout,node.anonymous_blocks,node.paint_children.borrow(),node.flags.bits(),node.primary_styles().map(|style|style.get_box().display));
            pending.extend(node.children.iter().copied());
            pending.extend(node.anonymous_blocks.iter().copied());
            if let Some(layout)=layout {pending.extend(layout);}
        }
    });
}
#[test]
fn nav_link_unmount_after_hidden_transition_releases_native_layout_children() {
    let host = WindowApp::new();
    host.mount(
        nav_link("minimal")
            .label("Docs")
            .children(|| glory::ui::input().value("Child")),
    )
    .unwrap();
    let app = host.app();
    app.settle(16).unwrap();
    app.dispatch(button(app, "Expand Docs"), Event::Activate)
        .unwrap();
    app.settle(16).unwrap();
    dump(&host, "open settled");
    app.dispatch(button(app, "Collapse Docs"), Event::Activate)
        .unwrap();
    dump(&host, "closed committed before settle");
    app.settle(16).unwrap();
    assert!(!all(app).iter().any(|id| matches!(
        app.tree().node(*id).unwrap().descriptor.view().props.clone(),
        Props::Input(_)
    )));
}
#[test]
fn navigation_uses_real_native_controls_current_metadata_and_stable_engine_nodes_in_four_themes()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let page = app.scope().cage(5);
        let enabled = app.scope().cage(true);
        let proposed = app.scope().cage(0_u64);
        host.mount(
            col().style(Style::new().width(Length::Px(1050.0))).fill((
                breadcrumbs("native-trail").theme(theme.clone()).items([
                    breadcrumb_item("home", "Home").href("/home"),
                    breadcrumb_item("current", "Current"),
                ]),
                pagination("native-pages")
                    .theme(theme.clone())
                    .page(page)
                    .total(Some(1_000_000_000_000))
                    .enabled(enabled)
                    .on_page_change(move |value| proposed.revise(|old| *old = value)),
                nav_link("native-docs")
                    .theme(theme)
                    .label("Documentation")
                    .href("/docs")
                    .current(true)
                    .children(|| glory::ui::input().name("Nested input").value("Nested")),
            )),
        )
        .unwrap();
        app.settle(16).unwrap();
        let next = button(app, "Next page");
        let engine = host
            .renderer()
            .engine_id(next, glory::html::Part::Main)
            .unwrap();
        let rect = bounds(app, next);
        assert!(rect.width > 0.0 && rect.height > 0.0);
        app.dispatch(next, Event::Activate).unwrap();
        assert_eq!(proposed.get(), 6);
        assert_eq!(page.get(), 5);
        let ids = all(app);
        page.revise(|old| *old = 999);
        app.settle(16).unwrap();
        assert_eq!(all(app), ids);
        assert_eq!(
            host.renderer()
                .engine_id(next, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        assert_eq!(
            app.tree()
                .node(button(app, "Page 999"))
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .current,
            Some(Current::Page)
        );
        enabled.revise(|old| *old = false);
        app.settle(16).unwrap();
        assert!(!app.dispatch(next, Event::Activate).unwrap());
        assert_eq!(proposed.get(), 6);
        app.dispatch(button(app, "Expand Documentation"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert!(all(app).iter().any(|id| matches!(
            app.tree().node(*id).unwrap().descriptor.view().props.clone(),
            Props::Input(_)
        )));
        app.dispatch(button(app, "Collapse Documentation"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        assert!(!all(app).iter().any(|id| matches!(
            app.tree().node(*id).unwrap().descriptor.view().props.clone(),
            Props::Input(_)
        )));
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
}
