#![cfg(feature = "chat")]
use glory::render::{Event, NodeId, Props};
use glory::ui::App;
use sapid::{ModelRecord, ModelSelector, model_selector};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn model(key: &str, provider: &str, name: &str) -> ModelRecord {
    let mut model = ModelRecord::new(key, provider, name);
    model.value = format!("{provider}/{key}");
    model
}
fn selector() -> ModelSelector {
    let mut disabled = model("disabled", "Second provider", "Unavailable");
    disabled.available = false;
    model_selector("catalog")
        .models([
            model("alpha", "First provider", "Alpha"),
            model("beta", "Second provider", "Beta"),
            disabled,
        ])
        .form_name("model")
        .default_value(Some("alpha".into()))
        .default_open(true)
}
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
fn option(app: &App, label: &str) -> NodeId {
    find(
        app,
        |node| matches!(node.props,Props::Button(value)if value.label==label),
    )
}
fn input(app: &App) -> String {
    let node = find(app, |node| matches!(node.props, Props::Input(_)));
    let tree = app.tree();
    let Props::Input(value) = tree.node(node).unwrap().descriptor.view().props else {
        panic!("input")
    };
    value.value.clone()
}
#[test]
fn grouped_catalog_uses_real_combobox_and_business_values_and_returns_only_the_chosen_record() {
    let app = App::memory();
    let changes = Rc::new(RefCell::new(Vec::new()));
    let output = changes.clone();
    app.mount(selector().on_change(move |value| output.borrow_mut().push(value)))
        .unwrap();
    assert_eq!(input(&app), "Alpha");
    find(&app, |node| node.name.as_deref() == Some("First provider"));
    find(&app, |node| node.name.as_deref() == Some("Second provider"));
    app.dispatch(option(&app, "Beta"), Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(input(&app), "Beta");
    let chosen = changes.borrow()[0].clone().unwrap();
    assert_eq!(chosen.key, "beta");
    assert_eq!(chosen.value, "Second provider/beta");
    let select = find(&app, |node| matches!(node.props, Props::Select(_)));
    let tree = app.tree();
    let node = tree.node(select).unwrap();
    let Props::Select(select) = node.descriptor.view().props else {
        panic!("select")
    };
    assert_eq!(select.selection.keys[0].as_str(), "beta");
    assert_eq!(
        node.descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .form_control
            .as_ref()
            .unwrap()
            .name,
        "model"
    );
    drop(tree);
    app.clear().unwrap();
    assert_eq!(Rc::strong_count(&changes), 1);
}
#[test]
fn controlled_selection_refusal_preserves_accepted_label_and_disabled_model_cannot_propose() {
    let app = App::memory();
    let value = app.scope().cage(Some("alpha".to_owned()));
    let changes = Rc::new(RefCell::new(Vec::new()));
    let output = changes.clone();
    app.mount(
        selector()
            .value(value)
            .on_change(move |value| output.borrow_mut().push(value)),
    )
    .unwrap();
    assert!(
        !app.dispatch(option(&app, "Unavailable"), Event::Activate)
            .unwrap()
    );
    assert!(changes.borrow().is_empty());
    app.dispatch(option(&app, "Beta"), Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(value.get(), Some("alpha".into()));
    assert_eq!(
        input(&app),
        "Beta",
        "the accepted local search query is distinct from the controlled model key"
    );
    find(
        &app,
        |node| matches!(node.props,Props::Text(value)if value.content=="Selected model: Alpha (First provider)"),
    );
    assert_eq!(changes.borrow()[0].as_ref().unwrap().key, "beta");
    assert_eq!(
        changes.borrow().len(),
        1,
        "refusing one chosen key must not publish an additional clear proposal"
    );
}
#[test]
fn model_and_byte_budgets_reject_before_any_controls_are_created() {
    let app = App::memory();
    let consumed = Rc::new(Cell::new(0));
    let output = consumed.clone();
    let config = model_selector("bounded").models(
        std::iter::repeat_with(move || {
            let at = output.get();
            output.set(at + 1);
            model(&format!("key-{at}"), "provider", "Model")
        })
        .take(1_000_000),
    );
    assert_eq!(consumed.get(), 1025);
    assert!(app.mount(config).is_err());
    assert!(app.tree().roots().is_empty());
    let mut value = model("large", "provider", "Model");
    value.name = "x".repeat(1_000_000);
    assert!(app.mount(model_selector("bytes").model(value)).is_err());
    assert!(app.tree().roots().is_empty());
    assert!(
        app.mount(selector().default_value(Some("unknown".into())))
            .is_err()
    );
    assert!(app.tree().roots().is_empty());
}
