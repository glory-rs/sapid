use glory::apace::{Raster, shell::WindowApp};
use glory::render::{
    Action, Event, KeyboardCommand, NodeId, Props, Query, QueryValue, Rect, TextSelection,
};
use glory::ui::App;
use std::{cell::RefCell, rc::Rc};
#[path = "../../web/src/prompt_form.rs"]
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
fn native_prompt_has_real_multiline_geometry_preserves_selection_and_sends_accepted_text() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(1000, 900, 1.).unwrap();
        host.mount(fixture::prompt_form_with_files(theme, false))
            .unwrap();
        observe(&host);
        let app = host.app();
        let editor = find(
            app,
            |node| matches!(node.props,Props::Input(input)if input.value=="Draft 🙂"),
        );
        let QueryValue::Bounds(Rect { width, height, .. }) = query(app, Query::Bounds(editor))
        else {
            panic!("bounds")
        };
        assert!(width > 100. && height > 50.);
        let (revision, multiline) = {
            let tree = app.tree();
            let Props::Input(value) = tree.node(editor).unwrap().descriptor.view().props else {
                panic!("input")
            };
            (value.revision, value.multiline)
        };
        assert!(multiline);
        app.dispatch(
            editor,
            Event::ValueChanged {
                value: "Accepted native text".into(),
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
            assert_eq!(value.value, "Accepted native text");
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
        assert_eq!(
            query(app, Query::Selection(editor)),
            QueryValue::Selection { start: 1, end: 5 }
        );
        app.dispatch(
            editor,
            Event::Keyboard {
                from: editor,
                command: KeyboardCommand::Submit,
            },
        )
        .unwrap();
        observe(&host);
        find(
            app,
            |node| matches!(node.props,Props::Text(value)if value.content=="Text [Accepted native text]; files 0"),
        );
        assert_eq!(
            query(app, Query::Selection(editor)),
            QueryValue::Selection { start: 1, end: 5 }
        );
        assert_eq!(query(app, Query::Focus(editor)), QueryValue::Focus(true));
        app.dispatch(editor, Event::Composition { active: true })
            .unwrap();
        let form = app
            .tree()
            .node(editor)
            .unwrap()
            .parent
            .and_then(|mut id| {
                loop {
                    let tree = app.tree();
                    let node = tree.node(id).unwrap();
                    if node.descriptor.view().field_part()
                        == Some(glory::render::FieldPart::Form)
                    {
                        break Some(id);
                    }
                    match node.parent {
                        Some(parent) => id = parent,
                        None => break None,
                    }
                }
            })
            .expect("editor belongs to the primary form");
        app.dispatch(form, Event::Submit { submitter: None })
            .unwrap();
        observe(&host);
        find(
            app,
            |node| matches!(node.props,Props::Text(value)if value.content=="Composing"),
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
#[test]
fn native_optional_files_reject_explicitly_without_claiming_native_picker_support() {
    let host = WindowApp::new();
    assert!(
        host.mount(fixture::prompt_form(sapid::Theme::blue_light()))
            .is_err()
    );
    host.app().clear().unwrap();
    assert_eq!(host.app().stats().nodes, 0);
}
