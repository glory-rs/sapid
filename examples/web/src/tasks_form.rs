use glory::prelude::*;
use sapid::chat::*;
pub fn plan_data() -> PlanData {
    let mut draft = TaskRecord::new("draft", "Prepare change");
    draft.description =
        "Public application task detail <script>window.taskExecuted=true</script>".into();
    draft.status = TaskStatus::Running;
    draft.progress = Some(37);
    draft.action = Some(TaskAction::new("cancel", "Request task cancellation"));
    let mut review = TaskRecord::new("review", "Review accepted change");
    review.status = TaskStatus::Complete;
    review.description = "The application marks this step complete.".into();
    let mut publish = TaskRecord::new("publish", "Publish when authorized");
    publish.description = "This display does not publish or schedule work.".into();
    PlanData {
        title: "Application implementation plan".into(),
        description: "Explicit public status and callback proposals.".into(),
        tasks: vec![draft, review, publish],
    }
}
pub fn tasks_form(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| {
        let data = scope.cage(plan_data());
        let standalone = scope.cage(plan_data().tasks.remove(0));
        let accept = scope.cage(false);
        let enabled = scope.cage(true);
        let mounted = scope.cage(true);
        let requests = scope.cage(0usize);
        let root = scope.cage(None);
        view()
            .class("tasks-fixture")
            .style(
                Style::new()
                    .display(glory::style::Display::Flex)
                    .flex_direction(glory::style::FlexDirection::Column)
                    .gap(12.)
                    .min_width(Length::Px(0.)),
            )
            .node_ref(root)
            .show_in(scope)?;
        let area = scope.child_in(root.get().unwrap())?;
        sapid::checkbox()
            .theme(theme.clone())
            .label("Accept task proposals")
            .checked(accept)
            .on_checked_change(move |v| accept.revise(|n| *n = v))
            .show_in(&area)?;
        sapid::checkbox()
            .theme(theme.clone())
            .label("Enable task actions")
            .checked(enabled)
            .on_checked_change(move |v| enabled.revise(|n| *n = v))
            .show_in(&area)?;
        for (label, action) in [
            ("Update current task", 0),
            ("Reverse supplied queue", 1),
            ("Empty supplied tasks", 2),
            ("Restore supplied tasks", 3),
            ("Toggle task owner", 4),
        ] {
            sapid::button()
                .theme(theme.clone())
                .label(label)
                .on_press(move || match action {
                    0 => data.revise(|p| {
                        if let Some(t) = p.tasks.iter_mut().find(|t| t.key == "draft") {
                            t.title = "Updated preparation task".into();
                            t.progress = Some(58);
                        }
                    }),
                    1 => data.revise(|p| p.tasks.reverse()),
                    2 => data.revise(|p| p.tasks.clear()),
                    3 => data.revise(|p| *p = plan_data()),
                    _ => mounted.revise(|v| *v = !*v),
                })
                .show_in(&area)?;
        }
        let palette = theme.clone();
        glory::ui::when(mounted, move || {
            let theme = palette.clone();
            widget(move |scope| {
                sapid::plan("fixture-plan")
                    .class("plan-primary")
                    .theme(theme.clone())
                    .data(data)
                    .enabled(enabled)
                    .on_action(move |request| {
                        requests.revise(|n| *n += 1);
                        if accept.get_untracked() {
                            data.revise(|p| {
                                if let Some(t) =
                                    p.tasks.iter_mut().find(|t| t.key == request.task_key)
                                {
                                    t.status = TaskStatus::Cancelled;
                                    t.progress = None;
                                    t.action = None;
                                }
                            });
                        }
                    })
                    .show_in(scope)?;
                queue("fixture-queue")
                    .class("queue-primary")
                    .theme(theme.clone())
                    .data(data)
                    .enabled(enabled)
                    .on_action(move |request| {
                        requests.revise(|n| *n += 1);
                        if accept.get_untracked() {
                            data.revise(|p| {
                                if let Some(t) =
                                    p.tasks.iter_mut().find(|t| t.key == request.task_key)
                                {
                                    t.status = TaskStatus::Cancelled;
                                    t.progress = None;
                                    t.action = None;
                                }
                            });
                        }
                    })
                    .show_in(scope)?;
                task("fixture-single", standalone)
                    .class("task-primary")
                    .theme(theme.clone())
                    .enabled(enabled)
                    .on_action(move |_| {
                        requests.revise(|n| *n += 1);
                        if accept.get_untracked() {
                            standalone.revise(|t| {
                                t.status = TaskStatus::Cancelled;
                                t.action = None;
                            });
                        }
                    })
                    .show_in(scope)?;
                Ok(())
            })
        })
        .show_in(&area)?;
        sapid::text()
            .theme(theme.clone())
            .class("task-requests")
            .content(
                scope.bond(move || format!("Application task requests: {}", requests.get())),
            )
            .show_in(&area)?;
        scope.retain(area)?;
        Ok(())
    })
}
