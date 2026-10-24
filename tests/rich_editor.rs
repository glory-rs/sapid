#![cfg(feature = "editor")]
use glory::{
    render::{self, Event, EventKind, NodeId, Props, RichSnapshot},
    ui::App,
};
use sapid::*;
use std::boxed::Box;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn doc(text: &str) -> Rc<RichDocument> {
    Rc::new(
        RichDocument::new(vec![RichBlock {
            kind: RichBlockKind::Paragraph,
            runs: vec![RichRun {
                text: text.into(),
                marks: RichMarks::default(),
            }],
        }])
        .unwrap(),
    )
}
fn node(app: &App) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let current = tree.node(id).unwrap();
        if matches!(current.descriptor.view().props, Props::RichEditor(_)) {
            return id;
        }
        pending.extend(current.children.iter().copied());
    }
    panic!("rich node")
}
fn revision(app: &App, id: NodeId) -> u64 {
    app.scope().rich_editor_value(id).unwrap().1
}
fn snapshot(document: Rc<RichDocument>, revision: u64, composing: bool) -> Event {
    let selection = RichSelection::collapsed(RichPoint {
        block: 0,
        offset: document.blocks[0].utf16_len(),
    });
    Event::RichEdit(Box::new(RichSnapshot {
        document,
        selection,
        revision,
        composing,
    }))
}
#[test]
fn real_document_props_user_acceptance_typed_before_generic_and_stable_identity() {
    let app = App::memory();
    let control = app.scope().cage(None);
    let log = Rc::new(RefCell::new(Vec::new()));
    let typed = log.clone();
    let generic = log.clone();
    app.mount(
        rich_text_editor()
            .default_document(doc("Start"))
            .controller_ref(control)
            .on_document_change(move |value| {
                typed
                    .borrow_mut()
                    .push(format!("typed:{}", value.plain_text()))
            })
            .on(EventKind::RichEdit, move |_| {
                generic.borrow_mut().push("generic".into())
            }),
    )
    .unwrap();
    let id = node(&app);
    let controller = control.get().unwrap();
    let old = revision(&app, id);
    app.dispatch(id, snapshot(doc("Actual😀"), old, false))
        .unwrap();
    assert_eq!(*log.borrow(), ["typed:Actual😀", "generic"]);
    assert_eq!(controller.document().get().plain_text(), "Actual😀");
    assert_eq!(controller.history().get().undo, 1);
    assert_eq!(node(&app), id);
    assert!(revision(&app, id) > old);
    assert!(controller.undo().unwrap());
    app.flush().unwrap();
    assert_eq!(controller.document().get().plain_text(), "Start");
    assert_eq!(controller.history().get().redo, 1);
    assert!(controller.redo().unwrap());
    app.flush().unwrap();
    assert_eq!(controller.document().get().plain_text(), "Actual😀");
}
#[test]
fn controlled_refusal_restores_new_revision_without_history_and_external_source_is_silent() {
    let app = App::memory();
    let source = app.scope().cage(doc("Accepted"));
    let accept = app.scope().cage(false);
    let output = app.scope().cage(None);
    let calls = Rc::new(Cell::new(0));
    let called = calls.clone();
    app.mount(
        rich_text_editor()
            .document(source)
            .controller_ref(output)
            .on_document_change(move |next| {
                called.set(called.get() + 1);
                if accept.get() {
                    source.revise(|value| *value = next);
                }
            }),
    )
    .unwrap();
    let id = node(&app);
    let controller = output.get().unwrap();
    let before = revision(&app, id);
    app.dispatch(id, snapshot(doc("Refused"), before, false))
        .unwrap();
    assert_eq!(source.get().plain_text(), "Accepted");
    assert_eq!(controller.history().get().undo, 0);
    assert!(revision(&app, id) > before);
    app.dispatch(id, snapshot(doc("Late"), before, false))
        .unwrap();
    assert_eq!(calls.get(), 1);
    accept.revise(|value| *value = true);
    app.dispatch(id, snapshot(doc("New"), revision(&app, id), false))
        .unwrap();
    assert_eq!(source.get().plain_text(), "New");
    assert_eq!(controller.history().get().undo, 1);
    source.revise(|value| *value = doc("Program"));
    app.flush().unwrap();
    assert_eq!(controller.document().get().plain_text(), "Program");
    assert_eq!(controller.history().get(), EditorHistory::default());
    assert_eq!(calls.get(), 2);
}
#[test]
fn composition_preedit_is_not_history_and_external_replacement_rejects_late_draft() {
    let app = App::memory();
    let output = app.scope().cage(None);
    let source = app.scope().cage(doc("Start"));
    app.mount(
        rich_text_editor()
            .document(source)
            .controller_ref(output)
            .on_document_change(move |next| source.revise(|value| *value = next)),
    )
    .unwrap();
    let id = node(&app);
    let controller = output.get().unwrap();
    let rev = revision(&app, id);
    app.dispatch(id, Event::Composition { active: true })
        .unwrap();
    for draft in ["中", "中文"] {
        app.dispatch(id, snapshot(doc(draft), rev, true)).unwrap();
        assert_eq!(controller.history().get().undo, 0);
        assert_eq!(source.get().plain_text(), "Start");
    }
    app.dispatch(id, snapshot(doc("中文"), rev, false)).unwrap();
    app.dispatch(id, Event::Composition { active: false })
        .unwrap();
    assert_eq!(source.get().plain_text(), "中文");
    assert_eq!(controller.history().get().undo, 1);
    let rev = revision(&app, id);
    app.dispatch(id, Event::Composition { active: true })
        .unwrap();
    source.revise(|value| *value = doc("Program wins"));
    app.flush().unwrap();
    app.dispatch(id, snapshot(doc("late"), rev, false)).unwrap();
    app.dispatch(id, Event::Composition { active: false })
        .unwrap();
    assert_eq!(source.get().plain_text(), "Program wins");
    assert_eq!(controller.history().get().undo, 0);
    assert!(!controller.composing().get());
}
#[test]
fn formatting_unicode_safe_links_undo_and_readonly_inert_policy_are_real() {
    let app = App::memory();
    let output = app.scope().cage(None);
    let readonly = app.scope().cage(false);
    app.mount(
        rich_text_editor()
            .default_document(doc("a😀z"))
            .readonly(readonly)
            .controller_ref(output),
    )
    .unwrap();
    let id = node(&app);
    let controller = output.get().unwrap();
    let selection = RichSelection {
        anchor: RichPoint {
            block: 0,
            offset: 1,
        },
        focus: RichPoint {
            block: 0,
            offset: 3,
        },
    };
    assert!(
        controller
            .format_at(selection, RichFormat::Bold(true))
            .unwrap()
    );
    app.flush().unwrap();
    let value = controller.document().get();
    assert_eq!(value.blocks[0].runs.len(), 3);
    assert_eq!(value.blocks[0].runs[1].text, "😀");
    assert!(value.blocks[0].runs[1].marks.bold);
    assert!(
        controller
            .format_at(
                selection,
                RichFormat::Link(Some("javascript:alert(1)".into()))
            )
            .is_err()
    );
    assert!(
        controller
            .format_at(
                RichSelection::collapsed(RichPoint {
                    block: 0,
                    offset: 2
                }),
                RichFormat::Bold(true)
            )
            .is_err()
    );
    assert!(
        controller
            .format_at(
                selection,
                RichFormat::Link(Some("https://example.test/".into()))
            )
            .unwrap()
    );
    app.flush().unwrap();
    assert!(controller.undo().unwrap());
    app.flush().unwrap();
    assert!(
        controller.document().get().blocks[0].runs[1]
            .marks
            .link
            .is_none()
    );
    readonly.revise(|value| *value = true);
    app.flush().unwrap();
    assert!(
        !controller
            .format_at(selection, RichFormat::Italic(true))
            .unwrap()
    );
    readonly.revise(|value| *value = false);
    app.flush().unwrap();
    let shell = app.tree().roots()[0];
    app.scope()
        .patch(
            shell,
            render::Patch::Semantics(render::Change::Set(Some(Box::new(render::Semantics {
                inert: true,
                ..Default::default()
            })))),
        )
        .unwrap();
    app.flush().unwrap();
    assert!(
        !controller
            .format_at(selection, RichFormat::Italic(true))
            .unwrap()
    );
    assert!(!app.scope().rich_editor_editable(id).unwrap());
}
#[test]
fn history_is_bounded_by_snapshot_count_and_independent_payload_budget() {
    let app = App::memory();
    let output = app.scope().cage(None);
    app.mount(rich_text_editor().toolbar(false).controller_ref(output))
        .unwrap();
    let id = node(&app);
    let controller = output.get().unwrap();
    for index in 0..45 {
        let text = format!("{index}{}", "x".repeat(65_500));
        app.dispatch(id, snapshot(doc(&text), revision(&app, id), false))
            .unwrap();
        let history = controller.history().get();
        assert!(history.undo + history.redo <= MAX_EDITOR_HISTORY);
        assert!(history.charged_bytes <= MAX_EDITOR_HISTORY_BYTES);
    }
    let count = controller.history().get().undo;
    assert!(count > 0 && count <= 32);
    for _ in 0..count {
        assert!(controller.undo().unwrap());
        app.flush().unwrap();
    }
    assert!(!controller.undo().unwrap());
    assert_eq!(controller.history().get().redo, count);
    controller.reset(doc("Reset")).unwrap();
    app.flush().unwrap();
    assert_eq!(controller.history().get(), EditorHistory::default());
    drop(app);
    assert!(controller.undo().is_err());
}
#[test]
fn four_themes_ssr_finite_document_unstyled_and_owner_cleanup() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let html = glory::ssr::render(
            rich_text_editor()
                .theme(theme)
                .default_document(doc("<literal> & 😀")),
        )
        .unwrap();
        assert!(
            html.contains("contenteditable=\"true\"")
                && html.contains("role=\"textbox\"")
                && html.contains("&lt;literal&gt; &amp; 😀")
                && html.contains("Rich text formatting")
        );
        assert!(!html.contains("<textarea"));
    }
    let app = App::memory();
    let output = app.scope().cage(None);
    app.mount(rich_text_editor().unstyled().controller_ref(output))
        .unwrap();
    let id = node(&app);
    let tree = app.tree();
    let node = tree.node(id).unwrap().descriptor.view();
    assert!(node.style.background.is_none() && node.style.border_color.is_none());
    assert_eq!(node.style.min_height, Some(glory::style::Length::Px(120.)));
    drop(tree);
    let controller = output.get().unwrap();
    drop(app);
    assert!(controller.reset(doc("dead")).is_err());
}
