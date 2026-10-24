use glory::render::{Event, EventKind, NodeId, Props, SelectProps, SemanticId};
use glory::ui::{App, Widget};
use sapid::{native_select, select_group, select_option};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

fn node(app: &App) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let n = tree.node(id).unwrap();
        if matches!(n.descriptor.view().props, Props::Select(_)) {
            return id;
        }
        pending.extend(n.children.iter().copied());
    }
    panic!()
}
fn props(app: &App, id: NodeId) -> SelectProps {
    let Props::Select(p) = app.tree().node(id).unwrap().descriptor.view().props.clone() else {
        panic!()
    };
    p
}
fn change(app: &App, id: NodeId, keys: &[&str]) {
    let revision = props(app, id).selection.revision;
    app.dispatch(
        id,
        Event::SelectChanged {
            keys: keys
                .iter()
                .map(|key| SemanticId::new(*key).unwrap())
                .collect(),
            revision,
        },
    )
    .unwrap();
}

#[test]
fn controlled_rejection_echo_and_generic_order_preserve_true_native_selection() {
    let app = App::memory();
    let source = app.scope().cage(Some("a".to_owned()));
    let log = Rc::new(RefCell::new(Vec::new()));
    let typed = log.clone();
    let generic = log.clone();
    app.mount(
        native_select("controlled")
            .label("Plan")
            .value(source)
            .option(select_option("a", "Starter").value("same"))
            .option(select_option("b", "Pro").value("same"))
            .on_value_change(move |value| typed.borrow_mut().push(format!("typed:{value:?}")))
            .on(EventKind::SelectChanged, move |_| {
                generic.borrow_mut().push("generic".into())
            }),
    )
    .unwrap();
    let id = node(&app);
    let entries = props(&app, id).entries;
    change(&app, id, &["b"]);
    assert_eq!(source.get(), Some("a".into()));
    assert_eq!(props(&app, id).selection.keys[0].as_str(), "a");
    assert_eq!(*log.borrow(), ["typed:Some(\"b\")", "generic"]);
    source.revise(|value| *value = Some("b".into()));
    app.flush().unwrap();
    assert_eq!(props(&app, id).selection.keys[0].as_str(), "b");
    assert!(Rc::ptr_eq(&entries, &props(&app, id).entries));
    let before = log.borrow().len();
    app.dispatch(
        id,
        Event::SelectChanged {
            keys: vec![SemanticId::new("a").unwrap()],
            revision: 0,
        },
    )
    .unwrap();
    assert_eq!(log.borrow().len(), before);
}

#[test]
fn multiple_sources_canonicalize_order_and_local_state_tracks_whole_tick() {
    let app = App::memory();
    let source = app.scope().cage(vec!["c".into(), "a".into()]);
    app.mount(native_select("multiple").values(source).options([
        select_option("a", "A"),
        select_option("b", "B"),
        select_option("c", "C"),
    ]))
    .unwrap();
    let id = node(&app);
    assert_eq!(
        props(&app, id)
            .selection
            .keys
            .iter()
            .map(|key| key.as_str())
            .collect::<Vec<_>>(),
        ["a", "c"]
    );
    change(&app, id, &["b"]);
    assert_eq!(props(&app, id).selection.keys.len(), 2);
    assert_eq!(source.get(), ["c", "a"]);
    let app = App::memory();
    app.mount(native_select("local").option(select_option("a", "A")))
        .unwrap();
    let id = node(&app);
    assert!(props(&app, id).selection.keys.is_empty());
    change(&app, id, &["a"]);
    assert_eq!(props(&app, id).selection.keys[0].as_str(), "a");
    change(&app, id, &[]);
    assert!(props(&app, id).selection.keys.is_empty());
}

#[test]
fn field_associations_native_form_values_and_readonly_rejection_are_shared() {
    let snapshot = glory::ssr::snapshot(
        sapid::field("native-field")
            .label("Country")
            .description("Delivery destination")
            .required(true)
            .form_name("country")
            .control(|field| {
                native_select("country")
                    .field(field)
                    .placeholder("Choose")
                    .option(select_option("uk", "United Kingdom").value("GB"))
            }),
    )
    .unwrap();
    let html = snapshot.html;
    assert!(
        html.contains("<select")
            && html.contains("aria-labelledby")
            && html.contains("aria-describedby")
            && html.contains("required=")
            && html.contains("name=\"country\"")
    );
    assert!(
        html.contains("value=\"GB\"")
            && html.contains("data-gly-placeholder=\"\" value=\"\" disabled hidden selected")
    );
    let app = App::memory();
    assert!(app.mount(native_select("readonly").readonly(true)).is_err());
    assert_eq!(app.stats().nodes, 0);
}

#[test]
fn groups_disabled_program_selection_and_owner_retirement_are_exact() {
    let app = App::memory();
    let calls = Rc::new(Cell::new(0));
    let observed = calls.clone();
    let mounted = native_select("groups")
        .default_value(Some("b".into()))
        .option(select_option("a", "A"))
        .group(
            select_group("Unavailable")
                .disabled(true)
                .option(select_option("b", "B")),
        )
        .on_value_change(move |_| observed.set(observed.get() + 1))
        .mount(app.scope())
        .unwrap();
    app.flush().unwrap();
    let id = node(&app);
    change(&app, id, &["a"]);
    assert_eq!(calls.get(), 1);
    assert!(
        app.dispatch(
            id,
            Event::SelectChanged {
                keys: vec![SemanticId::new("b").unwrap()],
                revision: props(&app, id).selection.revision
            }
        )
        .is_err()
    );
    drop(mounted);
    app.flush().unwrap();
    assert_eq!(app.stats().nodes, 0);
}

#[test]
fn source_and_configuration_budgets_reject_before_copying_unbounded_data() {
    let app = App::memory();
    let reads = Rc::new(Cell::new(0));
    let count = reads.clone();
    let widget = native_select("bounded").options((0..1_000_000).map(move |index| {
        count.set(count.get() + 1);
        select_option(format!("k{index}"), "Label")
    }));
    assert_eq!(reads.get(), 1025);
    assert!(app.mount(widget).is_err());
    assert_eq!(app.stats().nodes, 0);
    assert!(
        app.mount(native_select("huge-vector").values(vec![String::new(); 1_000_000]))
            .is_err()
    );
    assert!(
        app.mount(native_select("huge-string").value(Some("x".repeat(1_000_000))))
            .is_err()
    );
    assert_eq!(app.stats().nodes, 0);
    assert!(
        app.mount(
            native_select("duplicate")
                .options([select_option("a", "A"), select_option("a", "Again")])
        )
        .is_err()
    );
}

#[test]
fn invalid_program_updates_keep_last_valid_selection_and_recover() {
    let app = App::memory();
    let source = app.scope().cage(Some("a".to_owned()));
    app.mount(
        native_select("recover")
            .value(source)
            .option(select_option("a", "A")),
    )
    .unwrap();
    let id = node(&app);
    source.revise(|value| *value = Some("x".repeat(1_000_000)));
    assert!(app.flush().is_err());
    assert_eq!(props(&app, id).selection.keys[0].as_str(), "a");
    source.revise(|value| *value = None);
    app.flush().unwrap();
    assert!(props(&app, id).selection.keys.is_empty());
}

struct Host {
    memory: glory::render::memory::Memory,
    queued: Rc<RefCell<Vec<glory::render::Input>>>,
}
impl glory::render::Render for Host {
    fn style_profile(&self) -> glory::style::Profile {
        glory::style::Profile::RECORD
    }
    fn capabilities(&self) -> glory::render::Capabilities {
        glory::render::Capabilities::default()
    }
    fn request(
        &mut self,
        query: glory::render::Query,
        token: glory::render::QueryToken,
    ) -> std::result::Result<(), glory::render::Error> {
        glory::render::Render::request(&mut self.memory, query, token)
    }
    fn apply(
        &mut self,
        commands: &[glory::render::Mutation],
    ) -> std::result::Result<glory::render::Receipt, glory::render::Error> {
        glory::render::Render::apply(&mut self.memory, commands)
    }
    fn drain(&mut self, output: &mut Vec<glory::render::Input>) {
        output.append(&mut self.queued.borrow_mut());
        glory::render::Render::drain(&mut self.memory, output);
    }
}
#[test]
fn queued_changes_share_the_original_revision_until_the_entire_host_tick_finishes() {
    let queued = Rc::new(RefCell::new(Vec::new()));
    let inputs = queued.clone();
    let app = App::new(move |runtime| Host {
        memory: glory::render::memory::Memory::new(runtime),
        queued: inputs,
    });
    let source = app.scope().cage(None);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let observed = calls.clone();
    app.mount(
        native_select("tick")
            .value(source)
            .options([select_option("a", "A"), select_option("b", "B")])
            .on_value_change(move |value| {
                observed.borrow_mut().push(value.clone());
                source.revise(|old| *old = value);
            }),
    )
    .unwrap();
    let id = node(&app);
    let revision = props(&app, id).selection.revision;
    queued.borrow_mut().extend(
        ["a", "b"]
            .into_iter()
            .map(|key| glory::render::Input::Event {
                node: id,
                event: Event::SelectChanged {
                    keys: vec![SemanticId::new(key).unwrap()],
                    revision,
                },
            }),
    );
    app.pump().unwrap();
    assert_eq!(*calls.borrow(), [Some("a".into()), Some("b".into())]);
    assert_eq!(source.get(), Some("b".into()));
    assert_eq!(props(&app, id).selection.keys[0].as_str(), "b");
    assert_eq!(props(&app, id).selection.revision, revision + 1);
}
