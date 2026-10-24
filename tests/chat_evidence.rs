#![cfg(feature = "chat")]
use glory::{
    prelude::*,
    render::{DocumentPart, Event, NodeId, Props},
};
use sapid::chat::*;
use std::{cell::RefCell, rc::Rc};
fn explanation() -> PublicExplanation {
    PublicExplanation {
        summary: "How this result was prepared".into(),
        body: "Public summary supplied by the application <script>alert(1)</script>".into(),
        steps: vec![
            ExplanationStep {
                key: "review".into(),
                label: "Review supplied sources".into(),
                summary: "Compared the two application-provided records.".into(),
                status: ExplanationStatus::Complete,
                elapsed_millis: Some(32),
            },
            ExplanationStep::new("draft", "Write response", "A short public status summary."),
        ],
    }
}
fn record() -> SourceRecord {
    let mut s = SourceRecord::new("paper", "Design notes", "https://example.test/paper");
    s.preview = Some("Application-provided preview; no network fetch.".into());
    s.trust = SourceTrust::Verified;
    s
}
fn all(app: &App) -> Vec<NodeId> {
    let t = app.tree();
    let mut p = t.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = p.pop() {
        p.extend(&t.node(id).unwrap().children);
        out.push(id);
    }
    out
}
fn class(app: &App, c: &str) -> Vec<NodeId> {
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
                .any(|v| v == c)
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
fn named(app: &App, name: &str) -> NodeId {
    all(app)
        .into_iter()
        .find(|id| {
            let t = app.tree();
            let d = t.node(*id).unwrap().descriptor.view();
            d.name.as_deref() == Some(name)
                || matches!(d.props,Props::Button(p)if p.label==name)
        })
        .unwrap()
}
#[test]
fn public_disclosures_are_collapsed_by_default_and_use_real_ordered_or_unordered_steps() {
    for ordered in [false, true] {
        let app = App::memory();
        app.mount(
            if ordered {
                chain_of_thought("public")
            } else {
                reasoning("public")
            }
            .data(explanation()),
        )
        .unwrap();
        assert!(class(&app, "sapid-explanation-body").is_empty());
        app.dispatch(named(&app, "How this result was prepared"), Event::Activate)
            .unwrap();
        assert_eq!(class(&app, "sapid-explanation-step").len(), 2);
        assert!(text(&app).contains("<script>alert(1)</script>"));
        assert!(text(&app).contains("Complete · 32 ms"));
        assert!(all(&app).iter().any(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .is_some_and(|s| {
                    s.document_part
                        == Some(if ordered {
                            DocumentPart::OrderedList
                        } else {
                            DocumentPart::UnorderedList
                        })
                })
        }));
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
    }
}
#[test]
fn public_disclosure_controlled_refusal_and_keyed_updates_preserve_the_accepted_content() {
    let app = App::memory();
    let source = app.scope().cage(explanation());
    let proposals = Rc::new(RefCell::new(vec![]));
    let out = proposals.clone();
    app.mount(
        chain_of_thought("public")
            .data(source)
            .open(true)
            .on_open_change(move |v| out.borrow_mut().push(v)),
    )
    .unwrap();
    let ids = class(&app, "sapid-explanation-step");
    app.dispatch(named(&app, "How this result was prepared"), Event::Activate)
        .unwrap();
    assert_eq!(*proposals.borrow(), [false]);
    assert_eq!(ids, class(&app, "sapid-explanation-step"));
    source.revise(|v| v.steps[0].summary = "Updated public summary".into());
    app.flush().unwrap();
    assert_eq!(ids, class(&app, "sapid-explanation-step"));
    assert!(text(&app).contains("Updated public summary"));
    source.revise(|v| v.steps[0].label.clear());
    assert!(app.flush().is_err());
    assert!(text(&app).contains("Updated public summary"));
    source.revise(|v| v.steps.clear());
    app.flush().unwrap();
    assert!(class(&app, "sapid-explanation-step").is_empty());
}
#[test]
fn explanation_budgets_and_duplicate_keys_fail_without_mounting_nodes() {
    let mut cases = vec![];
    let mut d = explanation();
    d.steps[1].key = d.steps[0].key.clone();
    cases.push(d);
    let mut d = explanation();
    d.body = "x".repeat(16385);
    cases.push(d);
    let mut d = explanation();
    d.steps = (0..65)
        .map(|i| ExplanationStep::new(i.to_string(), "Step", ""))
        .collect();
    cases.push(d);
    let mut d = explanation();
    d.steps[0].elapsed_millis = Some(604_800_001);
    cases.push(d);
    for data in cases {
        let app = App::memory();
        assert!(app.mount(reasoning("invalid").data(data)).is_err());
        assert_eq!(app.stats().nodes, 0);
    }
}
#[test]
fn sources_keep_real_links_previews_application_trust_and_stable_current_keys() {
    let app = App::memory();
    let source = app.scope().cage(vec![record()]);
    app.mount(sources("docs").records(source)).unwrap();
    let link = class(&app, "sapid-source-link")[0];
    assert!(
        matches!(app.tree().node(link).unwrap().descriptor.view().props,Props::Link(p)if p.target=="https://example.test/paper")
    );
    assert!(
        !app.tree()
            .node(link)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .link_request
    );
    assert!(text(&app).contains("Application trust: verified"));
    source.revise(|v| {
        v[0].title = "Updated notes".into();
        v[0].preview = Some("Updated preview".into());
    });
    app.flush().unwrap();
    assert_eq!(class(&app, "sapid-source-link"), vec![link]);
    assert!(text(&app).contains("Updated preview"));
    source.revise(|v| v[0].url = "javascript:alert(1)".into());
    assert!(app.flush().is_err());
    assert!(
        matches!(app.tree().node(link).unwrap().descriptor.view().props,Props::Link(p)if p.target=="https://example.test/paper")
    );
}
#[test]
fn citations_offer_exact_requests_only_when_the_application_installs_a_handler() {
    let app = App::memory();
    let out = Rc::new(RefCell::new(vec![]));
    let result = out.clone();
    app.mount(
        inline_citation(record())
            .label("[2]")
            .on_navigate(move |v| result.borrow_mut().push(v)),
    )
    .unwrap();
    let link = class(&app, "sapid-inline-citation")[0];
    assert!(
        app.tree()
            .node(link)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .link_request
    );
    app.dispatch(link, Event::Activate).unwrap();
    assert_eq!(out.borrow()[0].key, "paper");
    assert_eq!(out.borrow()[0].url, "https://example.test/paper");
    assert!(
        app.tree()
            .node(link)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref()
            .unwrap()
            .contains("[2]")
    );
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
}
#[test]
fn destinations_and_metadata_reject_executable_ambient_or_unbounded_inputs() {
    for url in [
        "javascript:alert(1)",
        "data:text/html,x",
        "file:///secret",
        "https://user:pass@example.test/",
        "https://example.test:0/",
        "https:///path",
        "https://a..b/",
        "https://example.test/\n",
    ] {
        let mut source = record();
        source.url = url.into();
        assert!(source.validate().is_err(), "{url}");
    }
    for url in [
        "https://example.test/a?q=x#part",
        "http://localhost:8080/",
        "#local-record",
    ] {
        let mut source = record();
        source.url = url.into();
        source.validate().unwrap();
    }
    let app = App::memory();
    assert!(
        app.mount(sources("bad").records(vec![record(); 129]))
            .is_err()
    );
    assert_eq!(app.stats().nodes, 0);
    let app = App::memory();
    assert!(
        app.mount(sources("bad").records(vec![record(), record()]))
            .is_err()
    );
}
#[test]
fn context_has_exact_text_readonly_ranges_unknown_capacity_and_explicit_overflow() {
    let app = App::memory();
    let usage = app.scope().cage(ContextUsage {
        used: 120,
        capacity: Some(100),
        unit: "tokens".into(),
    });
    app.mount(context().usage(usage)).unwrap();
    assert!(text(&app).contains("120 of 100 tokens; 20 over capacity"));
    let progress = class(&app, "sapid-context-progress")[0];
    let t = app.tree();
    let p = t
        .node(progress)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .progress
        .as_ref()
        .unwrap();
    assert_eq!(p.current, Some(100.));
    drop(t);
    usage.revise(|v| v.capacity = None);
    app.flush().unwrap();
    assert!(text(&app).contains("capacity unknown"));
    assert!(
        app.tree()
            .node(progress)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .progress
            .as_ref()
            .unwrap()
            .current
            .is_none()
    );
    usage.revise(|v| v.capacity = Some(0));
    assert!(app.flush().is_err());
    assert!(text(&app).contains("capacity unknown"));
}
