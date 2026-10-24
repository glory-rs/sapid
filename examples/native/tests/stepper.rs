use glory::blitz::shell::WindowApp;
use glory::prelude::*;
use glory::render::{Action, Current, Event, NodeId, Props, Query, QueryValue};
use sapid::{ContentPolicy, step, stepper};
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
fn button(app: &App, label: &str) -> NodeId {
    nodes(app).into_iter().find(|id|matches!(&app.tree().node(*id).unwrap().descriptor.view().props.clone(),Props::Button(p)if p.label==label)).unwrap()
}
fn query(app: &App, query: Query) -> QueryValue {
    let result = Rc::new(RefCell::new(None));
    let out = result.clone();
    app.request(query, move |value| *out.borrow_mut() = Some(value.unwrap()))
        .unwrap();
    app.pump().unwrap();
    result.borrow_mut().take().unwrap()
}
#[test]
fn stepper_native_four_themes_preserve_kept_inputs_and_restore_actual_focus_to_new_panel() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let active = app.scope().cage(0);
        let first = app.scope().cage(None);
        let second = app.scope().cage(None);
        let panel = app.scope().cage(None);
        let request = app.scope().cage(usize::MAX);
        host.mount(
            stepper("native")
                .theme(theme)
                .style(Style::new().width(Length::Px(720.0)))
                .active(active)
                .selectable(true)
                .on_step_change(move |value| request.revise(|old| *old = value))
                .steps([
                    step("account").label("Account").content(move || {
                        glory::ui::input()
                            .node_ref(first)
                            .name("Account name")
                            .value("First draft")
                    }),
                    step("review")
                        .label("Review")
                        .panel_ref(panel)
                        .content(move || {
                            glory::ui::input()
                                .node_ref(second)
                                .name("Review note")
                                .value("Second draft")
                        }),
                ]),
        )
        .unwrap();
        app.settle(16).unwrap();
        let first = first.get().unwrap();
        let second = second.get().unwrap();
        let engine = host
            .renderer()
            .engine_id(first, glory::html::Part::Main)
            .unwrap();
        let QueryValue::Bounds(rect) = query(app, Query::Bounds(first)) else {
            panic!("bounds")
        };
        assert!(rect.width > 0.0 && rect.height > 0.0);
        app.dispatch(button(app, "2. Review"), Event::Activate)
            .unwrap();
        assert_eq!(request.get(), 1);
        assert_eq!(active.get(), 0);
        app.scope().action(Action::Focus(first)).unwrap();
        app.settle(16).unwrap();
        assert_eq!(query(app, Query::Focus(first)), QueryValue::Focus(true));
        active.revise(|value| *value = 1);
        app.settle(16).unwrap();
        assert_eq!(
            query(app, Query::Focus(panel.get().unwrap())),
            QueryValue::Focus(true)
        );
        assert_eq!(
            host.renderer()
                .engine_id(first, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        let QueryValue::Bounds(rect) = query(app, Query::Bounds(second)) else {
            panic!("bounds")
        };
        assert!(rect.width > 0.0 && rect.height > 0.0);
        assert_eq!(
            app.tree()
                .node(button(app, "2. Review"))
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .current,
            Some(Current::Step)
        );
        active.revise(|value| *value = 2);
        app.settle(16).unwrap();
        assert!(nodes(app).iter().all(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .is_none_or(|s| s.current.is_none())
        }));
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
}
#[test]
fn stepper_unmount_retires_native_inputs_on_hidden_panels_and_recreates_clean_owners() {
    let host = WindowApp::new();
    let app = host.app();
    let active = app.scope().cage(0);
    let created = Rc::new(Cell::new(0));
    let retired = Rc::new(Cell::new(0));
    let a = created.clone();
    let b = retired.clone();
    let content = move || {
        let a = a.clone();
        let b = b.clone();
        glory::ui::component(move |scope| {
            a.set(a.get() + 1);
            let b = b.clone();
            scope.on_cleanup(move || b.set(b.get() + 1))?;
            glory::ui::input().value("New local draft").show_in(scope)
        })
    };
    let first = content.clone();
    host.mount(
        stepper("native-unmount")
            .active(active)
            .content_policy(ContentPolicy::Unmount)
            .steps([step("one").content(first), step("two").content(content)]),
    )
    .unwrap();
    app.settle(16).unwrap();
    assert_eq!(created.get(), 1);
    active.revise(|value| *value = 1);
    app.settle(16).unwrap();
    assert_eq!((created.get(), retired.get()), (2, 1));
    active.revise(|value| *value = 0);
    app.settle(16).unwrap();
    assert_eq!((created.get(), retired.get()), (3, 2));
    app.clear().unwrap();
    assert_eq!(created.get(), retired.get());
}
