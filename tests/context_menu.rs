use glory::render::*;
use glory::ui::{App, Widget};
use sapid::menu::{context_menu, menu_item, menu_submenu};
use std::{cell::Cell, rc::Rc};
fn nodes(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut result = Vec::new();
    while let Some(id) = pending.pop() {
        pending.extend(tree.node(id).unwrap().children.iter().copied());
        result.push(id);
    }
    result
}
fn region(app: &App) -> NodeId {
    nodes(app)
        .into_iter()
        .find(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .is_some_and(|s| s.context_menu.is_some())
        })
        .unwrap()
}
fn panel(app: &App) -> NodeId {
    nodes(app)
        .into_iter()
        .find(|id| {
            app.tree().node(*id).unwrap().descriptor.view().role == Some(Role::Menu)
                && app
                    .tree()
                    .node(*id)
                    .unwrap()
                    .descriptor
                    .view()
                    .semantics
                    .as_ref()
                    .unwrap()
                    .layer
                    .as_ref()
                    .unwrap()
                    .placement
                    .as_str()
                    == "context-point"
        })
        .unwrap()
}
fn request(app: &App, target: NodeId, from: NodeId, point: ContextPoint) {
    app.dispatch(
        target,
        Event::ContextMenu {
            from,
            point,
            source: ContextSource::Pointer,
        },
    )
    .unwrap();
}
fn metadata(app: &App, id: NodeId) -> Semantics {
    app.tree()
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .as_ref()
        .clone()
}
#[test]
fn context_request_uses_real_menu_tree_and_accepted_coordinate_without_popup_trigger() {
    let app = App::memory();
    app.mount(
        context_menu("document")
            .target(glory::ui::button().label("Document"))
            .item(menu_item("copy").label("Copy"))
            .submenu(
                menu_submenu("export")
                    .label("Export")
                    .item(menu_item("pdf").label("PDF")),
            ),
    )
    .unwrap();
    let target = region(&app);
    let panel = panel(&app);
    let child=nodes(&app).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(button)if button.label=="Document")).unwrap();
    assert!(metadata(&app, panel).hidden);
    assert!(metadata(&app, target).popup.is_none());
    let point = ContextPoint { x: 180.25, y: 90.0 };
    request(&app, target, child, point);
    let semantics = metadata(&app, panel);
    assert!(!semantics.hidden);
    let layer = semantics.layer.unwrap();
    assert_eq!(layer.placement, LayerPlacement::ContextPoint(point));
    assert_eq!(layer.return_focus, None);
    assert!(!layer.modal);
    assert!(nodes(&app).into_iter().any(|id| {
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .is_some_and(|s| {
                s.menu_item
                    .as_ref()
                    .is_some_and(|m| m.kind == MenuItemKind::Action)
            })
    }));
    app.dispatch(panel, Event::Dismiss(DismissReason::Escape))
        .unwrap();
    assert!(metadata(&app, panel).hidden);
}
#[test]
fn controlled_context_refusal_disabled_and_owner_retirement_do_not_publish_or_resurrect() {
    let app = App::memory();
    let open = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let owner = app.scope().child().unwrap();
    context_menu("controlled-context")
        .target(glory::ui::text().content("Surface"))
        .open(open)
        .enabled(enabled)
        .on_open_change(move |_| seen.set(seen.get() + 1))
        .item(menu_item("action").label("Action"))
        .show_in(&owner)
        .unwrap();
    app.flush().unwrap();
    let target = region(&app);
    let panel = panel(&app);
    request(&app, target, target, ContextPoint { x: 400.0, y: 350.0 });
    assert!(metadata(&app, panel).hidden);
    assert_eq!(
        metadata(&app, panel).layer.unwrap().placement,
        LayerPlacement::ContextPoint(ContextPoint::default())
    );
    assert_eq!(calls.get(), 1);
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    request(&app, target, target, ContextPoint { x: 500.0, y: 350.0 });
    assert_eq!(calls.get(), 1);
    owner.dispose();
    app.flush().unwrap();
    assert!(app.tree().node(target).is_err());
    assert!(
        !app.dispatch(
            target,
            Event::ContextMenu {
                from: target,
                point: ContextPoint::default(),
                source: ContextSource::Keyboard
            }
        )
        .unwrap()
    );
    assert_eq!(calls.get(), 1);
}
#[test]
fn context_requires_a_target_and_retains_menu_configuration_budgets() {
    let app = App::memory();
    assert!(
        app.mount(context_menu("missing").item(menu_item("a").label("A")))
            .is_err()
    );
    let consumed = Rc::new(Cell::new(0));
    let counter = consumed.clone();
    let app = App::memory();
    let config = context_menu("bounded")
        .target(glory::ui::text().content("Area"))
        .items((0..1_000_000).map(move |i| {
            counter.set(counter.get() + 1);
            menu_item(format!("item-{i}")).label("Item")
        }));
    assert!(app.mount(config).is_err());
    assert_eq!(consumed.get(), 257);
}

#[test]
fn context_internal_callbacks_precede_observers_and_retirement_skips_later_callbacks() {
    let app = App::memory();
    let enabled = app.scope().cage(true);
    let order = Rc::new(std::cell::RefCell::new(Vec::new()));
    let first = order.clone();
    let second = order.clone();
    app.mount(
        context_menu("ordered-context")
            .target(glory::ui::button().label("Document"))
            .enabled(enabled)
            .on_open_change(move |_| first.borrow_mut().push("internal"))
            .on(EventKind::ContextMenu, move |_| {
                second.borrow_mut().push("observer")
            })
            .item(menu_item("copy").label("Copy")),
    )
    .unwrap();
    let target = region(&app);
    request(&app, target, target, ContextPoint { x: 10.0, y: 10.0 });
    assert_eq!(*order.borrow(), ["internal", "observer"]);
    enabled.revise(|value| *value = false);
    app.flush().unwrap();
    request(&app, target, target, ContextPoint { x: 20.0, y: 20.0 });
    assert_eq!(
        *order.borrow(),
        ["internal", "observer"],
        "disabled context skips both typed handlers and generic observers"
    );
    let app = App::memory();
    let owner = app.scope().child().unwrap();
    let weak = owner.downgrade();
    let seen = Rc::new(Cell::new(0));
    let calls = seen.clone();
    context_menu("retiring-context")
        .target(glory::ui::button().label("Document"))
        .on_open_change(move |_| {
            if let Some(owner) = weak.upgrade() {
                owner.dispose();
            }
        })
        .on(EventKind::ContextMenu, move |_| calls.set(calls.get() + 1))
        .item(menu_item("copy").label("Copy"))
        .show_in(&owner)
        .unwrap();
    app.flush().unwrap();
    let target = region(&app);
    request(&app, target, target, ContextPoint { x: 100.0, y: 100.0 });
    assert_eq!(seen.get(), 0);
    assert_eq!(app.stats().nodes, 0);
}
