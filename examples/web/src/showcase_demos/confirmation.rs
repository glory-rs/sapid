use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::confirmation_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::chat::*;
    pub fn confirmation_data() -> ConfirmationData {
        ConfirmationData {
            title: "Approve deployment request".into(),
            description: "Only the application can authorize the displayed action.".into(),
            details: "Command preview: <script>window.confirmationExecuted=true</script>"
                .into(),
            revision: 3,
            ..Default::default()
        }
    }
    pub fn checkpoint_data() -> CheckpointData {
        CheckpointData {
        title: "Saved draft checkpoint".into(),
        description:
            "An explicit application snapshot. Restore is a request, not an automatic rollback."
                .into(),
        details: "Draft text <literal> & preserved context".into(),
        revision: 8,
        ..Default::default()
    }
    }
    pub fn confirmation_form(theme: sapid::Theme) -> impl Widget {
        widget(move |scope| {
            let data = scope.cage(confirmation_data());
            let saved = scope.cage(checkpoint_data());
            let accept = scope.cage(false);
            let enabled = scope.cage(true);
            let visible = scope.cage(true);
            let requests = scope.cage(0usize);
            let last = scope.cage("No application requests".to_owned());
            let root = scope.cage(None);
            view()
                .class("confirmation-fixture")
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
                .label("Application accepts requests")
                .checked(accept)
                .on_checked_change(move |value| accept.revise(|current| *current = value))
                .show_in(&area)?;
            sapid::checkbox()
                .theme(theme.clone())
                .label("Enable confirmation actions")
                .checked(enabled)
                .on_checked_change(move |value| enabled.revise(|current| *current = value))
                .show_in(&area)?;
            for (label, kind) in [
                ("Reset confirmation data", 0),
                ("Program cancel confirmation", 1),
                ("Reset checkpoint data", 2),
                ("Mark restore complete", 3),
                ("Toggle confirmation owner", 4),
                ("Mark checkpoint current", 5),
                ("Mark checkpoint unavailable", 6),
            ] {
                sapid::button()
                    .theme(theme.clone())
                    .label(label)
                    .on_press(move || match kind {
                        0 => data.revise(|value| {
                            let revision = value.revision.saturating_add(1);
                            *value = ConfirmationData {
                                revision,
                                ..confirmation_data()
                            };
                        }),
                        1 => data.revise(|value| value.status = ConfirmationStatus::Cancelled),
                        2 => saved.revise(|value| {
                            let revision = value.revision.saturating_add(1);
                            *value = CheckpointData {
                                revision,
                                ..checkpoint_data()
                            };
                        }),
                        3 => saved.revise(|value| value.status = CheckpointStatus::Restored),
                        4 => visible.revise(|value| *value = !*value),
                        5 => saved.revise(|value| value.status = CheckpointStatus::Current),
                        _ => saved.revise(|value| value.status = CheckpointStatus::Unavailable),
                    })
                    .show_in(&area)?;
            }
            let palette = theme.clone();
            when(visible, move || {
                let theme = palette.clone();
                widget(move |scope| {
                    confirmation("deployment-v1")
                        .name("Deployment confirmation")
                        .class("confirm-primary")
                        .theme(theme.clone())
                        .data(data)
                        .enabled(enabled)
                        .on_decision(move |request| {
                            requests.revise(|value| *value += 1);
                            last.revise(|value| {
                                *value = format!(
                                    "{}@{}:{:?}",
                                    request.key, request.revision, request.decision
                                )
                            });
                            if accept.get_untracked() {
                                data.revise(|value| {
                                    if value.revision == request.revision
                                        && value.status == ConfirmationStatus::Pending
                                    {
                                        value.status = match request.decision {
                                            ConfirmationDecision::Approve => {
                                                ConfirmationStatus::Approved
                                            }
                                            ConfirmationDecision::Reject => {
                                                ConfirmationStatus::Rejected
                                            }
                                        };
                                    }
                                });
                            }
                        })
                        .show_in(scope)?;
                    checkpoint("saved-draft")
                        .name("Draft checkpoint")
                        .class("checkpoint-primary")
                        .theme(theme.clone())
                        .data(saved)
                        .enabled(enabled)
                        .on_restore(move |request| {
                            requests.revise(|value| *value += 1);
                            last.revise(|value| {
                                *value = format!("{}@{}:Restore", request.key, request.revision)
                            });
                            if accept.get_untracked() {
                                saved.revise(|value| {
                                    if value.revision == request.revision
                                        && value.status == CheckpointStatus::Available
                                    {
                                        value.status = CheckpointStatus::Restoring;
                                    }
                                });
                            }
                        })
                        .show_in(scope)?;
                    Ok(())
                })
            })
            .show_in(&area)?;
            text()
                .class("confirmation-feedback")
                .content(
                    scope.bond(move || {
                        format!("Requests:{}; Last:{}", requests.get(), last.get())
                    }),
                )
                .style(Style::new().white_space(glory::style::WhiteSpace::PreWrap))
                .show_in(&area)?;
            confirmation("plain")
                .class("confirmation-unstyled")
                .unstyled()
                .labels(ConfirmationLabels {
                    approve: "允许请求".into(),
                    reject: "拒绝请求".into(),
                    ..Default::default()
                })
                .data(ConfirmationData {
                    title: "Unstyled confirmation".into(),
                    details: "Literal details survive without theme fill".into(),
                    ..Default::default()
                })
                .show_in(&area)?;
            checkpoint("no-handler")
                .class("checkpoint-no-handler")
                .theme(theme.clone())
                .data(CheckpointData {
                    title: "Display-only checkpoint".into(),
                    ..Default::default()
                })
                .show_in(&area)?;
            scope.retain(area)?;
            Ok(())
        })
    }
}
