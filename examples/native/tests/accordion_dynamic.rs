use glory::apace::shell::WindowApp;
use glory::render::{Action, Event, Navigation, NodeId, Props, Query, QueryValue, Rect};
use std::{cell::RefCell, rc::Rc};

fn button(app: &glory::App, label: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if matches!(node.descriptor.view().props,Props::Button(button) if button.label==label) {
            return id;
        }
        pending.extend(&node.children);
    }
    panic!("missing {label}")
}
fn query(app: &glory::App, query: Query) -> QueryValue {
    let result = Rc::new(RefCell::new(None));
    let out = result.clone();
    app.request(query, move |value| *out.borrow_mut() = Some(value.unwrap()))
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
fn four_theme_native_dynamic_accordion_preserves_layout_editor_and_actual_navigation_on_reorder()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(480, 480, 1.).unwrap();
        let app = host.app();
        let keys = app
            .scope()
            .cage(vec!["alpha".to_owned(), "beta".to_owned()]);
        let editor = app.scope().cage(None);
        host.mount(
            sapid::accordion("native-dynamic")
                .theme(theme)
                .content_policy(sapid::ContentPolicy::KeepMounted)
                .items(keys, move |key| {
                    let alpha = key == "alpha";
                    sapid::accordion_item(key).content(move || {
                        let input = glory::ui::input().value("Retained editor");
                        if alpha { input.node_ref(editor) } else { input }
                    })
                }),
        )
        .unwrap();
        app.settle(16).unwrap();
        let alpha = button(app, "alpha");
        let beta = button(app, "beta");
        app.dispatch(alpha, Event::Activate).unwrap();
        let input = editor.get().unwrap();
        let physical = host
            .renderer()
            .engine_id(input, glory::html::Part::Main)
            .unwrap();
        assert!(bounds(app, input).height > 0.);
        let initial = bounds(app, alpha);
        assert!(initial.width > 20. && initial.height >= 30.);
        keys.revise(|keys| keys.reverse());
        app.flush().unwrap();
        assert_eq!(button(app, "alpha"), alpha);
        assert_eq!(editor.get(), Some(input));
        assert_eq!(
            host.renderer()
                .engine_id(input, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        assert!(bounds(app, beta).y < bounds(app, alpha).y);
        app.dispatch(
            beta,
            Event::Navigate {
                from: beta,
                command: Navigation::Next,
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        assert_eq!(query(app, Query::Focus(alpha)), QueryValue::Focus(true));
        keys.revise(|keys| keys.retain(|key| key == "beta"));
        app.flush().unwrap();
        assert!(editor.get().is_none());
        assert_eq!(query(app, Query::Focus(beta)), QueryValue::Focus(true));
        assert!(
            host.renderer()
                .engine_id(input, glory::html::Part::Main)
                .is_err()
        );
        keys.revise(|keys| keys.push("alpha".to_owned()));
        app.flush().unwrap();
        assert_ne!(editor.get(), Some(input));
        let outside = app.scope().cage(None);
        app.mount(
            glory::ui::button()
                .label("Outside action")
                .node_ref(outside),
        )
        .unwrap();
        let outside = outside.get().unwrap();
        app.scope().action(Action::Focus(outside)).unwrap();
        app.flush().unwrap();
        app.settle(16).unwrap();
        keys.revise(|keys| keys.retain(|key| key != "beta"));
        app.flush().unwrap();
        assert_eq!(query(app, Query::Focus(outside)), QueryValue::Focus(true));
        app.clear().unwrap();
        assert!(editor.get().is_none());
    }
}
