use glory::blitz::{Raster, shell::WindowApp};
use glory::render::{Action, Event, Navigation, NodeId, Props, Query, QueryValue};
use glory::ui::App;
use std::{cell::RefCell, rc::Rc};
#[path = "../../web/src/model_suggestions_form.rs"]
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
fn native_suggestion_toolbar_has_real_focus_navigation_payload_source_and_theme_raster() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(1000, 850, 1.).unwrap();
        host.mount(fixture::model_suggestions_form(theme)).unwrap();
        observe(&host);
        let app = host.app();
        let first = button(app, "Explain · Application notes");
        let last = button(app, "Summarize");
        let disabled = button(app, "Unavailable suggestion");
        let QueryValue::Bounds(bounds) = query(app, Query::Bounds(first)) else {
            panic!("bounds")
        };
        assert!(bounds.width > 100. && bounds.height > 30.);
        app.scope().action(Action::Focus(first)).unwrap();
        app.flush().unwrap();
        app.dispatch(
            first,
            Event::Navigate {
                from: first,
                command: Navigation::Next,
            },
        )
        .unwrap();
        observe(&host);
        assert_eq!(query(app, Query::Focus(last)), QueryValue::Focus(true));
        assert!(!app.dispatch(disabled, Event::Activate).unwrap());
        app.dispatch(last, Event::Activate).unwrap();
        observe(&host);
        find(
            app,
            |node| matches!(node.props,Props::Text(value)if value.content=="Chosen summarize: Application summary request [no source]"),
        );
        app.dispatch(first, Event::Activate).unwrap();
        observe(&host);
        find(
            app,
            |node| matches!(node.props,Props::Text(value)if value.content=="Chosen explain: Application explanation request [Application notes]"),
        );
        app.dispatch(button(app, "Toggle suggestions enabled"), Event::Activate)
            .unwrap();
        observe(&host);
        assert!(!app.tree().enabled(first));
        assert!(!app.dispatch(first, Event::Activate).unwrap());
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
    }
}
#[test]
fn native_model_selector_keeps_editing_identity_controlled_refusal_and_business_values() {
    use sapid::{ModelRecord, model_selector};
    use std::cell::Cell;
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let selected = app.scope().cage(Some("atlas".to_owned()));
        let accept = Rc::new(Cell::new(false));
        let acceptance = accept.clone();
        let changes = Rc::new(RefCell::new(Vec::new()));
        let output = changes.clone();
        let mut boreal = ModelRecord::new("boreal", "South", "Boreal");
        boreal.value = "south/boreal-v2".into();
        let mut disabled = ModelRecord::new("retired", "South", "Retired");
        disabled.available = false;
        host.mount(
            model_selector("native-models")
                .theme(theme)
                .models([
                    ModelRecord::new("atlas", "North", "Atlas"),
                    boreal,
                    disabled,
                ])
                .form_name("model")
                .default_open(true)
                .value(selected)
                .on_change(move |proposal| {
                    output.borrow_mut().push(proposal.clone());
                    if acceptance.get() {
                        selected.revise(|value| *value = proposal.map(|model| model.key));
                    }
                }),
        )
        .unwrap();
        observe(&host);
        let editor = find(app, |node| matches!(node.props, Props::Input(_)));
        let physical = host
            .renderer()
            .engine_id(editor, glory::html::Part::Main)
            .unwrap();
        app.scope().action(Action::Focus(editor)).unwrap();
        app.flush().unwrap();
        assert!(
            !app.dispatch(button(app, "Retired"), Event::Activate)
                .unwrap()
        );
        app.dispatch(button(app, "Boreal"), Event::Activate)
            .unwrap();
        observe(&host);
        assert_eq!(selected.get().as_deref(), Some("atlas"));
        assert_eq!(changes.borrow().len(), 1);
        assert_eq!(
            changes.borrow()[0].as_ref().unwrap().value,
            "south/boreal-v2"
        );
        find(
            app,
            |node| matches!(node.props, Props::Text(value) if value.content == "Selected model: Atlas (North)"),
        );
        accept.set(true);
        app.dispatch(
            editor,
            Event::Choice {
                from: editor,
                command: glory::render::ChoiceCommand::OpenFirst,
            },
        )
        .unwrap();
        observe(&host);
        app.dispatch(button(app, "Boreal"), Event::Activate)
            .unwrap();
        observe(&host);
        assert_eq!(selected.get().as_deref(), Some("boreal"));
        let select = find(app, |node| matches!(node.props, Props::Select(_)));
        let tree = app.tree();
        let Props::Select(value) = tree.node(select).unwrap().descriptor.view().props else {
            panic!("select");
        };
        assert_eq!(value.selection.keys[0].as_str(), "boreal");
        drop(tree);
        assert_eq!(query(app, Query::Focus(editor)), QueryValue::Focus(true));
        assert_eq!(
            host.renderer()
                .engine_id(editor, glory::html::Part::Main)
                .unwrap(),
            physical
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
        assert_eq!(Rc::strong_count(&changes), 1);
    }
}
