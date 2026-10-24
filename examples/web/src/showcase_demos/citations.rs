use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::chat_evidence_form(theme, true)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::chat::*;
    pub fn explanation() -> PublicExplanation {
        PublicExplanation {
        summary: "Read public explanation".into(),
        body:
            "Application-provided public summary. <script>window.badExplanation=true</script>"
                .into(),
        steps: vec![
            ExplanationStep {
                key: "review".into(),
                label: "Review supplied records".into(),
                summary: "The application reports that its supplied sources were compared."
                    .into(),
                status: ExplanationStatus::Complete,
                elapsed_millis: Some(32),
            },
            ExplanationStep::new(
                "draft",
                "Prepare a response",
                "Only a public status summary is shown here.",
            ),
        ],
    }
    }
    pub fn source() -> SourceRecord {
        SourceRecord {
            key: "design".into(),
            title: "Design notes".into(),
            url: "#citation-target".into(),
            preview: Some(
                "Explicit application preview. <img src=x onerror=window.badCitation=true>"
                    .into(),
            ),
            trust: SourceTrust::Unverified,
        }
    }
    pub fn chat_evidence_form(theme: sapid::Theme, citations: bool) -> impl Widget {
        widget(move |scope| {
            let root = scope.cage(None);
            view()
                .class("chat-evidence-fixture")
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
            if citations {
                let usage = scope.cage(ContextUsage {
                    used: 37,
                    capacity: Some(100),
                    unit: "tokens".into(),
                });
                let source = scope.cage(source());
                let show = scope.cage(true);
                let mounted = scope.cage(true);
                let requests = scope.cage(0usize);
                for (label, value) in [
                    ("Increase context usage", 0),
                    ("Overflow context capacity", 1),
                    ("Unknown context capacity", 2),
                    ("Restore context capacity", 3),
                ] {
                    sapid::button()
                        .theme(theme.clone())
                        .label(label)
                        .on_press(move || {
                            usage.revise(|u| match value {
                                0 => u.used = 58,
                                1 => u.used = 120,
                                2 => u.capacity = None,
                                _ => {
                                    u.used = 37;
                                    u.capacity = Some(100);
                                }
                            })
                        })
                        .show_in(&area)?;
                }
                sapid::button()
                    .theme(theme.clone())
                    .label("Update supplied source")
                    .on_press(move || {
                        source.revise(|s| {
                            s.title = "Updated design notes".into();
                            s.preview = Some("Updated application preview".into());
                            s.trust = SourceTrust::Verified;
                        })
                    })
                    .show_in(&area)?;
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle source records")
                    .on_press(move || show.revise(|v| *v = !*v))
                    .show_in(&area)?;
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle evidence owner")
                    .on_press(move || mounted.revise(|v| *v = !*v))
                    .show_in(&area)?;
                let palette = theme.clone();
                glory::ui::when(mounted, move || {
                    let theme = palette.clone();
                    widget(move |scope| {
                        context()
                            .class("context-primary")
                            .theme(theme.clone())
                            .usage(usage)
                            .show_in(scope)?;
                        sources("fixture-sources")
                            .class("sources-primary")
                            .theme(theme.clone())
                            .records(scope.bond(move || {
                                if show.get() {
                                    vec![source.get()]
                                } else {
                                    vec![]
                                }
                            }))
                            .on_navigate(move |_| requests.revise(|v| *v += 1))
                            .show_in(scope)?;
                        inline_citation(source)
                            .class("citation-primary")
                            .theme(theme.clone())
                            .label("[1]")
                            .show_in(scope)?;
                        Ok(())
                    })
                })
                .show_in(&area)?;
                text()
                    .class("source-requests")
                    .content(scope.bond(move || format!("Source requests: {}", requests.get())))
                    .show_in(&area)?;
                view()
                    .semantics(Some(Box::new(glory::render::Semantics {
                        id: Some(glory::render::SemanticId::new("citation-target")?),
                        ..Default::default()
                    })))
                    .fill(text().content("Citation destination supplied by the application"))
                    .show_in(&area)?;
            } else {
                let data = scope.cage(explanation());
                let open = scope.cage(false);
                let accept = scope.cage(true);
                let enabled = scope.cage(true);
                let mounted = scope.cage(true);
                let proposals = scope.cage(0usize);
                sapid::checkbox()
                    .theme(theme.clone())
                    .label("Accept explanation proposals")
                    .checked(accept)
                    .on_checked_change(move |v| accept.revise(|o| *o = v))
                    .show_in(&area)?;
                sapid::checkbox()
                    .theme(theme.clone())
                    .label("Enable explanation controls")
                    .checked(enabled)
                    .on_checked_change(move |v| enabled.revise(|o| *o = v))
                    .show_in(&area)?;
                sapid::button()
                    .theme(theme.clone())
                    .label("Update public explanation")
                    .on_press(move || {
                        data.revise(|d| {
                            d.steps[0].summary =
                                "Updated public summary supplied by the application".into();
                            d.steps[0].status = ExplanationStatus::Active;
                        })
                    })
                    .show_in(&area)?;
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle explanation owner")
                    .on_press(move || mounted.revise(|v| *v = !*v))
                    .show_in(&area)?;
                let palette = theme.clone();
                glory::ui::when(mounted, move || {
                    reasoning("fixture-reasoning")
                        .class("reasoning-primary")
                        .theme(palette.clone())
                        .data(data)
                        .open(open)
                        .enabled(enabled)
                        .on_open_change(move |v| {
                            proposals.revise(|n| *n += 1);
                            if accept.get() {
                                open.revise(|o| *o = v);
                            }
                        })
                })
                .show_in(&area)?;
                text()
                    .class("explanation-proposals")
                    .content(
                        scope.bond(move || {
                            format!("Explanation proposals: {}", proposals.get())
                        }),
                    )
                    .show_in(&area)?;
                chain_of_thought("fixture-steps")
                    .class("chain-secondary")
                    .theme(theme.clone())
                    .data(explanation())
                    .default_open(true)
                    .show_in(&area)?;
                reasoning("unstyled-public")
                    .class("explanation-unstyled")
                    .theme(theme.clone())
                    .unstyled()
                    .data(PublicExplanation {
                        summary: "Unstyled public explanation".into(),
                        body: "Application-owned summary".into(),
                        steps: vec![],
                    })
                    .default_open(true)
                    .show_in(&area)?;
            }
            scope.retain(area)?;
            Ok(())
        })
    }
}
