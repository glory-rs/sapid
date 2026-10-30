use glory::apace::shell::WindowApp;
use glory::render::{Action, Event, Navigation, NodeId, Props, Query, QueryValue, Rect};
use std::{cell::RefCell, rc::Rc};

fn button(app: &glory::App, label: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if matches!(node.descriptor.view().props, Props::Button(props) if props.label == label) {
            return id;
        }
        pending.extend(&node.children);
    }
    panic!("missing tab {label}")
}
fn query(app: &glory::App, query: Query) -> QueryValue {
    let result = Rc::new(RefCell::new(None));
    let output = result.clone();
    app.request(query, move |value| {
        *output.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    result.borrow_mut().take().unwrap()
}
fn bounds(app: &glory::App, node: NodeId) -> Rect {
    let QueryValue::Bounds(rect) = query(app, Query::Bounds(node)) else {
        panic!("bounds")
    };
    rect
}

#[test]
fn four_theme_dynamic_tabs_keep_engine_editor_identity_and_actual_focus_on_member_changes() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(480, 480, 1.).unwrap();
        let app = host.app();
        let members = app
            .scope()
            .cage(vec!["alpha".to_owned(), "beta".to_owned()]);
        let source = app.scope().cage("alpha".to_owned());
        let proposals = app.scope().cage(0usize);
        let editor = app.scope().cage(None);
        host.mount(
            sapid::tabs("native-tabs")
                .theme(theme)
                .value(source)
                .on_value_change(move |_| proposals.revise(|value| *value += 1))
                .tabs(members, move |key| {
                    let alpha = key == "alpha";
                    sapid::tab(key).content(move || {
                        let input = glory::ui::input().value("Retained draft");
                        if alpha { input.node_ref(editor) } else { input }
                    })
                }),
        )
        .unwrap();
        app.settle(16).unwrap();
        let alpha = button(app, "alpha");
        let beta = button(app, "beta");
        let input = editor.get().unwrap();
        let physical = host
            .renderer()
            .engine_id(input, glory::html::Part::Main)
            .unwrap();
        assert!(bounds(app, alpha).height >= 30.);
        assert!(bounds(app, input).width > 20.);
        members.revise(|keys| keys.reverse());
        app.flush().unwrap();
        assert_eq!(button(app, "alpha"), alpha);
        assert_eq!(
            host.renderer()
                .engine_id(input, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        assert!(bounds(app, beta).x < bounds(app, alpha).x);
        app.scope().action(Action::Focus(input)).unwrap();
        app.flush().unwrap();
        app.settle(16).unwrap();
        members.revise(|keys| keys.retain(|key| key != "alpha"));
        app.flush().unwrap();
        app.settle(16).unwrap();
        assert_eq!(query(app, Query::Focus(beta)), QueryValue::Focus(true));
        assert_eq!(
            proposals.get(),
            0,
            "membership handoff must preserve controlled absent intent"
        );
        assert_eq!(source.get(), "alpha");
        assert!(editor.get().is_none());
        assert!(
            host.renderer()
                .engine_id(input, glory::html::Part::Main)
                .is_err()
        );
        members.revise(|keys| keys.push("alpha".to_owned()));
        app.flush().unwrap();
        assert_ne!(editor.get(), Some(input));
        assert_eq!(source.get(), "alpha");
        assert_eq!(proposals.get(), 0);
        let next = button(app, "alpha");
        app.dispatch(
            beta,
            Event::Navigate {
                from: beta,
                command: Navigation::Next,
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        assert_eq!(query(app, Query::Focus(next)), QueryValue::Focus(true));
        assert_eq!(
            proposals.get(),
            0,
            "focusing already controlled selected tab is silent"
        );
        app.clear().unwrap();
        assert_eq!(app.stats().queries, 0);
        assert_eq!(app.runtime_stats().tasks, 0);
    }
}
