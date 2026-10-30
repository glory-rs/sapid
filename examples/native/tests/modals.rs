use glory::{
    apace::{Raster, shell::WindowApp},
    prelude::*,
    render::{
        Action, NodeId, Props, Query, QueryValue, Rect, Role, SelectionDirection, TextSelection,
    },
};
use sapid::{ModalOutcome, confirmation_request, modal_request, modals_manager};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn nodes(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = pending.pop() {
        pending.extend(&tree.node(id).unwrap().children);
        out.push(id)
    }
    out
}
fn active(app: &App, role: Role) -> Option<NodeId> {
    nodes(app).into_iter().find(|id| {
        let tree = app.tree();
        let node = tree.node(*id).unwrap().descriptor.view();
        node.role == Some(role) && !node.hidden()
    })
}
fn button(app: &App, label: &str) -> NodeId {
    nodes(app).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(value)if value.label==label)).unwrap()
}
fn query(app: &App, query: Query) -> QueryValue {
    let out = Rc::new(RefCell::new(None));
    let saved = out.clone();
    app.request(query, move |value| {
        *saved.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    out.borrow_mut().take().unwrap()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let QueryValue::Bounds(value) = query(app, Query::Bounds(id)) else {
        panic!()
    };
    value
}
fn focused(app: &App, id: NodeId) -> bool {
    query(app, Query::Focus(id)) == QueryValue::Focus(true)
}
#[test]
fn native_modal_manager_uses_real_fifo_layers_retains_editors_and_retires_owner_in_four_themes() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(800, 600, 1.).unwrap();
        let app = host.app();
        let opener = app.scope().cage(None);
        let editor = app.scope().cage(None);
        let value = app.scope().cage("Initial retained input".to_owned());
        let ready = Rc::new(RefCell::new(None));
        let saved = ready.clone();
        host.mount(
            view().fill((
                sapid::button().label("Independent opener").node_ref(opener),
                modals_manager("native-queue")
                    .theme(theme.clone())
                    .capacity(3)
                    .on_ready(move |handle| *saved.borrow_mut() = Some(handle)),
            )),
        )
        .unwrap();
        app.settle(16).unwrap();
        let handle = ready
            .borrow_mut()
            .take()
            .expect("supported closed Layer admits manager and ready handle");
        assert_eq!(handle.pending().unwrap(), 0);
        assert!(active(app, Role::Dialog).is_none());
        let opener = opener.get().unwrap();
        app.scope().action(Action::Focus(opener)).unwrap();
        app.settle(16).unwrap();
        assert!(focused(app, opener));
        let built = Rc::new(Cell::new(0));
        let first = built.clone();
        let second = built.clone();
        let outcomes = Rc::new(RefCell::new(vec![]));
        let log = outcomes.clone();
        let a = handle
            .open(
                modal_request("First native modal", "Retained finite content")
                    .content(move || {
                        first.set(first.get() + 1);
                        input()
                            .name("Retained queue input")
                            .value(value)
                            .node_ref(editor)
                    })
                    .on_result(move |result| log.borrow_mut().push(result)),
            )
            .unwrap();
        let log = outcomes.clone();
        let b = handle
            .open(
                confirmation_request(
                    "Second native confirmation",
                    "Safe cancel is the initial focus",
                )
                .content(move || {
                    second.set(second.get() + 1);
                    text().content("Second finite content")
                })
                .on_result(move |result| log.borrow_mut().push(result)),
            )
            .unwrap();
        assert_eq!(built.get(), 0);
        app.settle(24).unwrap();
        assert_eq!(built.get(), 1);
        assert_eq!(handle.pending().unwrap(), 2);
        let panel = active(app, Role::Dialog).unwrap();
        assert!(active(app, Role::AlertDialog).is_none());
        let rect = bounds(app, panel);
        assert!(
            rect.width > 200.
                && rect.height > 80.
                && rect.x >= 0.
                && rect.y >= 0.
                && rect.x + rect.width <= 800.1
                && rect.y + rect.height <= 600.1,
            "{rect:?}"
        );
        let editor = editor.get().unwrap();
        let physical = host
            .renderer()
            .engine_id(editor, glory::html::Part::Main)
            .unwrap();
        value.revise(|value| *value = "Retained selected draft".into());
        app.settle(16).unwrap();
        app.scope().action(Action::Focus(editor)).unwrap();
        let revision = match app.tree().node(editor).unwrap().descriptor.view().props {
            Props::Input(value) => value.revision,
            _ => panic!(),
        };
        app.scope()
            .action(Action::SelectText {
                node: editor,
                revision,
                selection: TextSelection {
                    start: 2,
                    end: 9,
                    direction: SelectionDirection::Backward,
                },
            })
            .unwrap();
        app.settle(16).unwrap();
        assert_eq!(
            query(app, Query::Selection(editor)),
            QueryValue::Selection { start: 2, end: 9 }
        );
        assert!(
            handle
                .update(
                    &a,
                    "Updated native modal",
                    "Metadata changed without replacing the editor"
                )
                .unwrap()
        );
        app.settle(16).unwrap();
        assert_eq!(active(app, Role::Dialog), Some(panel));
        assert_eq!(
            host.renderer()
                .engine_id(editor, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        assert_eq!(built.get(), 1);
        assert!(focused(app, editor));
        assert_eq!(
            query(app, Query::Selection(editor)),
            QueryValue::Selection { start: 2, end: 9 }
        );
        let close = button(app, "Close");
        let close_rect = bounds(app, close);
        let close_physical = host
            .renderer()
            .engine_id(close, glory::html::Part::Main)
            .unwrap();
        let hit = host.renderer().inspect(|dom| {
            dom.hit(
                (close_rect.x + close_rect.width / 2.) as f32,
                (close_rect.y + close_rect.height / 2.) as f32,
            )
            .unwrap()
            .node_id
        });
        assert_eq!(hit, close_physical);
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let c = theme.surface;
        assert!(
            raster
                .pixels()
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|pixel| **pixel == [c.0, c.1, c.2, c.3])
                .count()
                > 1000
        );
        app.dispatch(close, Event::Activate).unwrap();
        app.settle(24).unwrap();
        assert_eq!(built.get(), 2);
        assert_eq!(handle.pending().unwrap(), 1);
        assert!(active(app, Role::Dialog).is_none());
        let confirmation = active(app, Role::AlertDialog).unwrap();
        assert!(bounds(app, confirmation).height > 80.);
        assert!(focused(app, button(app, "Cancel")));
        assert!(
            host.renderer()
                .engine_id(editor, glory::html::Part::Main)
                .is_err()
        );
        assert_eq!(*outcomes.borrow(), [ModalOutcome::Closed]);
        assert!(!handle.close(&a).unwrap());
        app.dispatch(button(app, "Confirm"), Event::Activate)
            .unwrap();
        app.settle(24).unwrap();
        assert_eq!(handle.pending().unwrap(), 0);
        assert_eq!(
            *outcomes.borrow(),
            [ModalOutcome::Closed, ModalOutcome::Confirmed]
        );
        assert!(!handle.close(&b).unwrap());
        assert!(
            focused(app, opener),
            "FIFO queue must restore its original independent opener"
        );
        let sentinel = Rc::new(());
        let weak = Rc::downgrade(&sentinel);
        let log = outcomes.clone();
        handle
            .open(
                modal_request(
                    "Retire pending modal",
                    "Owner cleanup is not a user outcome",
                )
                .on_result(move |result| {
                    let _ = &sentinel;
                    log.borrow_mut().push(result);
                }),
            )
            .unwrap();
        app.settle(16).unwrap();
        app.clear().unwrap();
        assert!(weak.upgrade().is_none());
        assert_eq!(outcomes.borrow().len(), 2);
        assert!(handle.pending().is_err());
        assert!(
            handle
                .open(modal_request("Retired", "Must not reopen the native owner"))
                .is_err()
        );
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(app.stats().queries, 0);
        assert_eq!(host.renderer().stats().nodes, 0);
    }
}
