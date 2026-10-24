use glory::blitz::{Raster, shell::WindowApp};
use glory::render::{Action, Event, Navigation, NodeId, Props, Query, QueryValue};
use glory::ui::App;
use std::{cell::RefCell, rc::Rc};
#[path = "../../web/model_suggestions_form.rs"]
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
        host.mount(fixture::model_suggestions_form_with_selector(theme, false))
            .unwrap();
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
fn native_model_selector_explicitly_rejects_editable_choice_preserving_an_accepted_prefix() {
    let host = WindowApp::new();
    let prefix = host.app().scope().cage(None);
    host.mount(
        glory::ui::text()
            .content("Accepted prefix")
            .node_ref(prefix),
    )
    .unwrap();
    observe(&host);
    let id = prefix.get().unwrap();
    let before = query(host.app(), Query::Bounds(id));
    let error = host
        .mount(fixture::model_suggestions_form(sapid::Theme::blue_light()))
        .unwrap_err();
    assert!(matches!(
        error,
        glory::ui::Error::Render(glory::render::Error::Unsupported(
            "native combobox controls"
        ))
    ));
    assert_eq!(query(host.app(), Query::Bounds(id)), before);
    host.app().clear().unwrap();
    assert_eq!(host.app().stats().nodes, 0);
}
