use glory::apace::{Raster, shell::WindowApp};
use glory::render::{Action, Event, NodeId, Props, Query, QueryValue, TextSelection};
use glory::ui::App;
use std::{cell::RefCell, rc::Rc};
#[path = "../../web/src/questionnaire_form.rs"]
mod fixture;
fn find(app: &App, predicate: impl Fn(glory::render::DescriptorView<'_>) -> bool) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if predicate(node.descriptor.view()) {
            return id;
        }
        pending.extend(node.children.iter().copied());
    }
    panic!("missing node")
}
fn button(app: &App, label: &str) -> NodeId {
    find(
        app,
        |node| matches!(node.props,Props::Button(value)if value.label==label),
    )
}
fn query(app: &App, query: Query) -> QueryValue {
    let result = Rc::new(RefCell::new(None));
    let output = result.clone();
    app.request(query, move |value| {
        *output.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    result.borrow_mut().take().unwrap()
}
fn observe(host: &WindowApp) {
    let context = std::task::Context::from_waker(std::task::Waker::noop());
    for _ in 0..8 {
        host.observe(&context).unwrap();
    }
}
#[test]
fn native_typed_questionnaire_validates_real_text_and_checkbox_preserving_hidden_editor_identity()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(900, 850, 1.).unwrap();
        host.mount(fixture::questionnaire_form(theme)).unwrap();
        observe(&host);
        let app = host.app();
        let editor = find(app, |node| matches!(node.props, Props::Input(_)));
        let physical = host
            .renderer()
            .engine_id(editor, glory::html::Part::Main)
            .unwrap();
        let QueryValue::Bounds(bounds) = query(app, Query::Bounds(editor)) else {
            panic!("bounds")
        };
        assert!(bounds.width > 100. && bounds.height > 30.);
        app.dispatch(button(app, "Next question step"), Event::Activate)
            .unwrap();
        observe(&host);
        find(
            app,
            |node| matches!(node.props,Props::Text(value)if value.content=="Name is required"),
        );
        let revision = {
            let tree = app.tree();
            let Props::Input(value) = tree.node(editor).unwrap().descriptor.view().props else {
                panic!("input")
            };
            value.revision
        };
        app.dispatch(
            editor,
            Event::ValueChanged {
                value: "Native answer".into(),
                revision,
                composing: false,
            },
        )
        .unwrap();
        observe(&host);
        let revision = {
            let tree = app.tree();
            let Props::Input(value) = tree.node(editor).unwrap().descriptor.view().props else {
                panic!("input")
            };
            value.revision
        };
        app.scope().action(Action::Focus(editor)).unwrap();
        app.scope()
            .action(Action::SelectText {
                node: editor,
                revision,
                selection: TextSelection {
                    start: 1,
                    end: 5,
                    direction: Default::default(),
                },
            })
            .unwrap();
        app.flush().unwrap();
        app.dispatch(button(app, "Next question step"), Event::Activate)
            .unwrap();
        observe(&host);
        assert!(!app.tree().active(editor));
        assert!(!app.tree().enabled(editor));
        let checkbox = find(app, |node| matches!(node.props, Props::Check(_)));
        assert!(app.tree().active(checkbox));
        let QueryValue::Bounds(bounds) = query(app, Query::Bounds(checkbox)) else {
            panic!("bounds")
        };
        assert!(bounds.width > 20. && bounds.height > 10.);
        app.dispatch(checkbox, Event::CheckedChanged(true)).unwrap();
        app.dispatch(button(app, "Submit answers"), Event::Activate)
            .unwrap();
        observe(&host);
        find(
            app,
            |node| matches!(node.props,Props::Text(value)if value.content=="Accepted [Native answer]; permission true"),
        );
        app.dispatch(button(app, "Previous question step"), Event::Activate)
            .unwrap();
        observe(&host);
        assert_eq!(
            host.renderer()
                .engine_id(editor, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        assert_eq!(
            query(app, Query::Selection(editor)),
            QueryValue::Selection { start: 1, end: 5 }
        );
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        assert!(
            raster
                .pixels()
                .as_chunks::<4>()
                .0
                .iter()
                .any(|pixel| pixel[3] > 0)
        );
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(app.runtime_stats().tasks, 0);
    }
}
