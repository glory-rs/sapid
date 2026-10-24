#![cfg(feature = "chat")]
use glory::{
    prelude::*,
    render::{DocumentPart, NodeId, Props},
};
use sapid::chat::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[path = "../examples/web/src/tasks_form.rs"]
mod fixture;
#[test]
fn task_fixture_builds_all_three_widgets_in_four_themes_and_unstyled_has_no_theme_fill() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let app = App::memory();
        app.mount(fixture::tasks_form(theme)).unwrap();
        assert_eq!(class(&app, "sapid-plan").len(), 1);
        assert_eq!(class(&app, "sapid-queue").len(), 1);
        assert_eq!(class(&app, "sapid-task").len(), 7);
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
    }
    let app = App::memory();
    app.mount(task("plain", TaskRecord::new("item", "Plain task")).unstyled())
        .unwrap();
    assert_eq!(
        app.tree()
            .node(class(&app, "sapid-task")[0])
            .unwrap()
            .descriptor
            .view()
            .style
            .background,
        None
    );
}
fn data() -> PlanData {
    let mut first = TaskRecord::new("draft", "Draft change");
    first.description = "Application public task detail <script>run()</script>".into();
    first.status = TaskStatus::Running;
    first.progress = Some(37);
    first.action = Some(TaskAction::new("cancel", "Request cancel"));
    let mut done = TaskRecord::new("review", "Review change");
    done.status = TaskStatus::Complete;
    PlanData {
        title: "Application plan".into(),
        description: "Application-supplied work and status".into(),
        tasks: vec![first, done],
    }
}
fn all(app: &App) -> Vec<NodeId> {
    let t = app.tree();
    let mut pending = t.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = pending.pop() {
        pending.extend(&t.node(id).unwrap().children);
        out.push(id)
    }
    out
}
fn class(app: &App, name: &str) -> Vec<NodeId> {
    all(app)
        .into_iter()
        .filter(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|c| c == name)
        })
        .collect()
}
fn text(app: &App) -> String {
    all(app)
        .into_iter()
        .filter_map(
            |id| match app.tree().node(id).unwrap().descriptor.view().props {
                Props::Text(p) => Some(p.content.clone()),
                _ => None,
            },
        )
        .collect::<Vec<_>>()
        .join("|")
}
fn button(app: &App, name: &str) -> NodeId {
    all(app).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(p)if p.label==name)).unwrap()
}
#[test]
fn plan_progress_counts_terminal_work_without_claiming_failed_work_succeeded() {
    let app = App::memory();
    let mut d = data();
    d.tasks[1].status = TaskStatus::Failed;
    app.mount(sapid::plan("work").data(d)).unwrap();
    assert!(text(&app).contains("Processed 1 of 2 tasks"));
    assert!(text(&app).contains("Failed · Review change"));
    assert!(text(&app).contains("<script>run()</script>"));
    assert_eq!(class(&app, "sapid-task-list-item").len(), 2);
    assert!(all(&app).iter().any(|id| {
        app.tree()
            .node(*id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .is_some_and(|s| s.document_part == Some(DocumentPart::OrderedList))
    }));
    assert!(all(&app).iter().any(|id| {
        app.tree()
            .node(*id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .is_some_and(|s| s.progress.as_ref().is_some_and(|p| p.current == Some(50.)))
    }));
    // An action without an installed application callback cannot execute.
    assert!(
        matches!(app.tree().node(button(&app,"Request cancel")).unwrap().descriptor.view().props,Props::Button(p)if !p.enabled)
    );
}
#[test]
fn actions_only_propose_full_keys_and_refusal_keeps_state() {
    let app = App::memory();
    let source = app.scope().cage(data());
    let requests = Rc::new(RefCell::new(vec![]));
    let out = requests.clone();
    app.mount(
        sapid::plan("controlled")
            .data(source)
            .on_action(move |r| out.borrow_mut().push(r)),
    )
    .unwrap();
    let before = source.get();
    app.dispatch(button(&app, "Request cancel"), Event::Activate)
        .unwrap();
    assert_eq!(source.get(), before);
    assert_eq!(
        *requests.borrow(),
        [TaskRequest {
            task_key: "draft".into(),
            action_key: "cancel".into()
        }]
    );
    source.revise(|p| {
        p.tasks[0].action.as_mut().unwrap().enabled = false;
    });
    app.flush().unwrap();
    assert!(
        matches!(app.tree().node(button(&app,"Request cancel")).unwrap().descriptor.view().props,Props::Button(p)if !p.enabled)
    );
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
}
#[test]
fn queue_order_and_current_record_updates_preserve_full_key_identity() {
    let app = App::memory();
    let source = app.scope().cage(data());
    app.mount(queue("queue").data(source)).unwrap();
    let ids = class(&app, "sapid-task");
    let semantics = ids
        .iter()
        .map(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .id
                .clone()
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_ne!(semantics[0], semantics[1]);
    source.revise(|p| {
        p.tasks.reverse();
        p.tasks[0].description = "New public review detail".into();
    });
    app.flush().unwrap();
    let after = class(&app, "sapid-task");
    assert!(ids.iter().all(|id| after.contains(id)));
    assert!(text(&app).contains("New public review detail"));
    source.revise(|p| p.tasks.clear());
    app.flush().unwrap();
    assert!(text(&app).contains("No tasks supplied"));
    assert!(class(&app, "sapid-task").is_empty());
    assert!(class(&app, "sapid-plan-progress").is_empty());
}
#[test]
fn finite_model_rejects_duplicate_keys_counts_bytes_and_invalid_percentages() {
    let mut d = data();
    d.tasks[1].key = d.tasks[0].key.clone();
    assert!(d.validate().is_err());
    let mut d = data();
    d.tasks[0].progress = Some(101);
    assert!(d.validate().is_err());
    let mut d = data();
    d.tasks = (0..129)
        .map(|i| TaskRecord::new(i.to_string(), "Task"))
        .collect();
    assert!(d.validate().is_err());
    d.tasks.truncate(128);
    for t in &mut d.tasks {
        t.description = "x".repeat(8192);
    }
    assert!(d.validate().is_err());
    let mut d = data();
    d.tasks[0].action.as_mut().unwrap().label = "bad\nlabel".into();
    assert!(d.validate().is_err());
}
#[test]
fn invalid_current_plan_retains_complete_accepted_rows_and_then_recovers() {
    let app = App::memory();
    let source = app.scope().cage(data());
    app.mount(sapid::plan("admit").data(source)).unwrap();
    let ids = class(&app, "sapid-task");
    source.revise(|p| p.tasks[0].title = "Updated title".into());
    app.flush().unwrap();
    source.revise(|p| p.tasks[0].description = "x".repeat(8193));
    assert!(app.flush().is_err());
    assert!(text(&app).contains("Updated title"));
    assert_eq!(ids, class(&app, "sapid-task"));
    source.revise(|p| p.tasks[0].description = "Recovered".into());
    app.flush().unwrap();
    assert!(text(&app).contains("Recovered"));
}
#[test]
fn standalone_task_keeps_identity_and_retires_callback_captures() {
    let app = App::memory();
    let source = app.scope().cage(data().tasks.remove(0));
    let calls = Rc::new(Cell::new(0));
    let counter = calls.clone();
    let sentinel = Rc::new(());
    let weak = Rc::downgrade(&sentinel);
    app.mount(task("single", source).on_action(move |_| {
        let _ = &sentinel;
        counter.set(counter.get() + 1);
    }))
    .unwrap();
    let id = class(&app, "sapid-task")[0];
    app.dispatch(button(&app, "Request cancel"), Event::Activate)
        .unwrap();
    assert_eq!(calls.get(), 1);
    source.revise(|t| t.key = "replacement".into());
    assert!(app.flush().is_err());
    assert_eq!(id, class(&app, "sapid-task")[0]);
    app.clear().unwrap();
    assert!(weak.upgrade().is_none());
}
