#![cfg(feature = "files")]
use glory::render::{memory::Memory, *};
use glory::ui::App;
use sapid::{FileStatus, dropzone, file_button, file_input};
use std::{cell::RefCell, rc::Rc};
struct Wire {
    document: FileDocumentId,
    events: RefCell<Vec<Input>>,
    actions: RefCell<Vec<Action>>,
}
struct Host {
    tree: Memory,
    wire: Rc<Wire>,
}
impl Render for Host {
    fn style_profile(&self) -> glory::style::Profile {
        glory::style::Profile::RECORD
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            files: Some(FileCapabilities {
                document: self.wire.document,
                picker: true,
                drop: true,
                read: false,
                preview: false,
            }),
            ..Default::default()
        }
    }
    fn apply(&mut self, m: &[Mutation]) -> Result<Receipt, Error> {
        self.tree.apply(m)
    }
    fn request(&mut self, q: Query, t: QueryToken) -> Result<(), Error> {
        self.tree.request(q, t)
    }
    fn action(&mut self, a: Action) -> Result<(), Error> {
        self.tree.admit_file_action(a)?;
        self.wire.actions.borrow_mut().push(a);
        Ok(())
    }
    fn drain(&mut self, v: &mut Vec<Input>) {
        for input in self.wire.events.borrow_mut().drain(..) {
            if let Input::Event {
                node,
                event: Event::Files(event),
            } = &input
            {
                self.tree
                    .finish_files(*node, self.wire.document, event)
                    .unwrap();
            }
            v.push(input);
        }
        self.tree.drain(v);
    }
}
fn app() -> (App, Rc<Wire>) {
    let wire = Rc::new(Wire {
        document: FileDocumentId::new().unwrap(),
        events: RefCell::default(),
        actions: RefCell::default(),
    });
    let host = wire.clone();
    (
        App::new(move |runtime| Host {
            tree: Memory::new(runtime),
            wire: host,
        }),
        wire,
    )
}
fn selected(wire: &Wire, node: NodeId, request: u64, batch: u64, revision: u64, count: usize) {
    wire.events.borrow_mut().push(Input::Event {
        node,
        event: Event::Files(Box::new(FileEvent {
            document: wire.document,
            revision,
            request: Some(request),
            batch,
            origin: FileOrigin::Picker,
            outcome: FileOutcome::Selected(
                (0..count)
                    .map(|i| FileInfo {
                        token: FileToken {
                            document: wire.document,
                            node,
                            revision,
                            serial: batch * 32 + i as u64,
                        },
                        name: format!("file-{i}.txt"),
                        media_type: "text/plain".into(),
                        size: 5,
                    })
                    .collect(),
            ),
        })),
    });
}
fn classes(app: &App, class: &str) -> Vec<NodeId> {
    fn visit(tree: &Memory, id: NodeId, class: &str, out: &mut Vec<NodeId>) {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            out.push(id);
        }
        for child in &node.children {
            visit(tree, *child, class, out);
        }
    }
    let tree = app.tree();
    let mut out = Vec::new();
    for root in tree.roots() {
        visit(&tree, *root, class, &mut out);
    }
    out
}
fn descendant(app: &App, node: NodeId, ancestor: NodeId) -> bool {
    let tree = app.tree();
    let mut cursor = Some(node);
    while let Some(node) = cursor {
        if node == ancestor {
            return true;
        }
        cursor = tree.node(node).unwrap().parent;
    }
    false
}

#[test]
fn selected_private_state_typed_order_external_output_is_not_input_and_clear_is_silent() {
    let (app, wire) = app();
    let scope = app.scope();
    let node = scope.cage(None);
    let output = scope.cage(Vec::new());
    let status = scope.cage(FileStatus::Idle);
    let controller = scope.cage(None::<sapid::FileController>);
    let log = Rc::new(RefCell::new(vec![]));
    let typed = log.clone();
    let generic = log.clone();
    app.mount(
        file_button()
            .node_ref(node)
            .files_ref(output)
            .status_ref(status)
            .controller_ref(controller)
            .on_files(move |files| {
                typed
                    .borrow_mut()
                    .push(format!("typed:{}:{:?}", files.len(), status.get()))
            })
            .on(EventKind::Files, move |_| {
                generic
                    .borrow_mut()
                    .push(format!("generic:{:?}", status.get()))
            }),
    )
    .unwrap();
    let id = node.get().unwrap();
    app.dispatch(id, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(status.get(), FileStatus::Choosing);
    selected(&wire, id, 1, 1, 0, 1);
    app.pump().unwrap();
    assert_eq!(&*log.borrow(), &["typed:1:Selected", "generic:Selected"]);
    let handle = controller.get().unwrap();
    assert_eq!(handle.files().get().len(), 1);
    output.revise(Vec::clear);
    app.flush().unwrap();
    assert_eq!(handle.files().get().len(), 1);
    handle.clear().unwrap();
    app.flush().unwrap();
    assert!(handle.files().get().is_empty());
    assert_eq!(log.borrow().len(), 2);
    assert_eq!(status.get(), FileStatus::Idle);
    app.clear().unwrap();
    assert!(controller.get().is_none());
    assert!(handle.clear().is_err());
}
#[test]
fn input_attachment_owners_are_stable_removal_is_named_and_not_inside_main_button() {
    let (app, wire) = app();
    let node = app.scope().cage(None);
    let config = FileConfig {
        multiple: true,
        max_files: 3,
        ..Default::default()
    };
    let builds = Rc::new(RefCell::new(vec![]));
    let built = builds.clone();
    app.mount(
        file_input()
            .node_ref(node)
            .configuration(config)
            .attachment(move |info, control| {
                built.borrow_mut().push(info.token);
                sapid::file_attachment(info, control)
            }),
    )
    .unwrap();
    let id = node.get().unwrap();
    app.dispatch(id, Event::Activate).unwrap();
    app.flush().unwrap();
    selected(&wire, id, 1, 1, 0, 2);
    app.pump().unwrap();
    let first = classes(&app, "sapid-file-attachment");
    assert_eq!(first.len(), 2);
    assert_eq!(builds.borrow().len(), 2);
    let removes = classes(&app, "sapid-file-remove");
    assert_eq!(removes.len(), 2);
    assert_eq!(
        app.tree()
            .node(removes[0])
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Remove file-0.txt")
    );
    assert!(!descendant(&app, removes[0], id));
    app.dispatch(removes[0], Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(classes(&app, "sapid-file-attachment"), vec![first[1]]);
    assert_eq!(builds.borrow().len(), 2);
}
#[test]
fn accepted_configuration_change_revokes_selection_silently_invalid_update_rolls_back() {
    let (app, wire) = app();
    let scope = app.scope();
    let node = scope.cage(None);
    let config = scope.cage(FileConfig::default());
    let output = scope.cage(Vec::new());
    let controller = scope.cage(None::<sapid::FileController>);
    let calls = scope.cage(0);
    app.mount(
        dropzone()
            .node_ref(node)
            .configuration(config)
            .files_ref(output)
            .controller_ref(controller)
            .on_files(move |_| calls.revise(|v| *v += 1)),
    )
    .unwrap();
    let id = node.get().unwrap();
    assert!(
        matches!(&app.tree().node(id).unwrap().descriptor.view().props,Props::File(p) if p.configuration.drop)
    );
    app.dispatch(id, Event::Activate).unwrap();
    app.flush().unwrap();
    selected(&wire, id, 1, 1, 0, 1);
    app.pump().unwrap();
    let old = output.get()[0].token;
    config.revise(|v| v.max_file_bytes = 10);
    app.flush().unwrap();
    assert!(output.get().is_empty());
    assert_eq!(calls.get(), 1);
    assert!(controller.get().unwrap().remove(old).is_err());
    assert!(
        matches!(&app.tree().node(id).unwrap().descriptor.view().props,Props::File(p) if p.revision==1)
    );
    config.revise(|v| v.max_files = 0);
    assert!(app.flush().is_err());
    assert!(
        matches!(&app.tree().node(id).unwrap().descriptor.view().props,Props::File(p) if p.configuration.max_files==1&&p.revision==1)
    );
}
#[test]
fn themes_unstyled_slots_and_native_unavailable_are_explicit() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let app = App::memory();
        let node = app.scope().cage(None);
        let status = app.scope().cage(FileStatus::Idle);
        app.mount(
            file_input()
                .theme(theme.clone())
                .node_ref(node)
                .status_ref(status)
                .description(|| glory::ui::text().content("Local uploads only")),
        )
        .unwrap();
        let id = node.get().unwrap();
        assert_eq!(
            app.tree()
                .node(id)
                .unwrap()
                .descriptor
                .view()
                .style
                .background,
            Some(theme.input)
        );
        app.dispatch(id, Event::Activate).unwrap();
        app.flush().unwrap();
        assert_eq!(
            status.get(),
            FileStatus::Rejected(FileRejection::Unavailable)
        );
    }
    let app = App::memory();
    let node = app.scope().cage(None);
    app.mount(file_button().unstyled().node_ref(node)).unwrap();
    assert_eq!(
        app.tree()
            .node(node.get().unwrap())
            .unwrap()
            .descriptor
            .view()
            .style,
        &glory::style::Style::new()
    );
}
#[test]
fn invalid_initial_configuration_does_not_call_user_slots() {
    let app = App::memory();
    let calls = Rc::new(RefCell::new(0));
    let invoked = calls.clone();
    assert!(
        app.mount(
            file_input()
                .configuration(FileConfig {
                    max_files: 0,
                    ..Default::default()
                })
                .description(move || {
                    *invoked.borrow_mut() += 1;
                    glory::ui::text()
                })
        )
        .is_err()
    );
    assert_eq!(*calls.borrow(), 0);
}
#[test]
fn picker_cancel_and_rejection_preserve_previous_files_and_disabled_cannot_start() {
    let (app, wire) = app();
    let scope = app.scope();
    let node = scope.cage(None);
    let enabled = scope.cage(true);
    let controller = scope.cage(None::<sapid::FileController>);
    let status = scope.cage(FileStatus::Idle);
    let calls = scope.cage(0);
    let results = Rc::new(RefCell::new(vec![]));
    let output = results.clone();
    app.mount(
        file_button()
            .node_ref(node)
            .enabled(enabled)
            .controller_ref(controller)
            .status_ref(status)
            .on_files(move |_| calls.revise(|value| *value += 1))
            .on_result(move |outcome| output.borrow_mut().push(outcome)),
    )
    .unwrap();
    let id = node.get().unwrap();
    app.dispatch(id, Event::Activate).unwrap();
    app.flush().unwrap();
    selected(&wire, id, 1, 1, 0, 1);
    app.pump().unwrap();
    let previous = controller.get().unwrap().files().get();
    for (request, outcome) in [
        (2, FileOutcome::Cancelled),
        (3, FileOutcome::Rejected(FileRejection::Type)),
    ] {
        app.dispatch(id, Event::Activate).unwrap();
        app.flush().unwrap();
        wire.events.borrow_mut().push(Input::Event {
            node: id,
            event: Event::Files(Box::new(FileEvent {
                document: wire.document,
                revision: 0,
                request: Some(request),
                batch: 1,
                origin: FileOrigin::Picker,
                outcome,
            })),
        });
        app.pump().unwrap();
        assert_eq!(controller.get().unwrap().files().get(), previous);
    }
    assert_eq!(status.get(), FileStatus::Rejected(FileRejection::Type));
    assert_eq!(calls.get(), 1);
    assert_eq!(results.borrow().len(), 3);
    enabled.revise(|value| *value = false);
    app.flush().unwrap();
    let before = wire.actions.borrow().len();
    app.dispatch(id, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(wire.actions.borrow().len(), before);
}
