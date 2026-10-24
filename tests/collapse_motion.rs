use glory::render::{memory::Memory, *};
use glory::ui::{App, Widget};
use std::{cell::Cell, rc::Rc};
struct MotionHost(Memory);
impl Render for MotionHost {
    fn style_profile(&self) -> glory::style::Profile {
        glory::style::Profile::RECORD
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            collapse_completion: true,
            ..Default::default()
        }
    }
    fn apply(&mut self, mutations: &[Mutation]) -> Result<Receipt, Error> {
        self.0.apply(mutations)
    }
    fn request(&mut self, query: Query, token: QueryToken) -> Result<(), Error> {
        self.0.request(query, token)
    }
    fn drain(&mut self, out: &mut Vec<Input>) {
        self.0.drain(out)
    }
}
fn nodes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut out = Vec::new();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|name| name == class)
        {
            out.push(id)
        }
        pending.extend(&node.children);
    }
    out
}
fn config(app: &App, node: NodeId) -> CollapseConfig {
    app.tree()
        .node(node)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .collapse
        .unwrap()
}
#[test]
fn closing_unmount_waits_for_matching_terminal_and_old_complete_cannot_retire_reopened_owner() {
    let app = App::new(MotionHost::from);
    let open = app.scope().cage(true);
    let cleaned = app.scope().cage(0usize);
    app.mount(
        sapid::collapsible("height")
            .open(open)
            .height_transition(240)
            .content(move || {
                glory::ui::component(move |scope| {
                    scope.on_cleanup(move || {
                        assert!(cleaned.try_get().is_ok());
                        cleaned.revise(|value| *value += 1);
                    })?;
                    glory::ui::input().class("animated-editor").show_in(scope)
                })
            }),
    )
    .unwrap();
    let panel = nodes(&app, "sapid-collapsible-panel")[0];
    let input = nodes(&app, "animated-editor")[0];
    open.revise(|open| *open = false);
    app.flush().unwrap();
    let closing = config(&app, panel);
    assert!(!closing.open && closing.revision == 1);
    assert!(!app.tree().active(input));
    assert!(app.tree().inert(input));
    assert_eq!(nodes(&app, "animated-editor"), [input]);
    assert_eq!(cleaned.get(), 0);
    open.revise(|open| *open = true);
    app.flush().unwrap();
    assert_eq!(config(&app, panel).revision, 2);
    assert!(
        !app.dispatch(
            panel,
            Event::CollapseComplete {
                revision: closing.revision
            }
        )
        .unwrap()
    );
    assert_eq!(nodes(&app, "animated-editor"), [input]);
    assert_eq!(cleaned.get(), 0);
    let future = config(&app, panel).revision + 1;
    assert!(
        app.dispatch(panel, Event::CollapseComplete { revision: future })
            .is_err()
    );
    open.revise(|open| *open = false);
    app.flush().unwrap();
    let revision = config(&app, panel).revision;
    app.dispatch(panel, Event::CollapseComplete { revision })
        .unwrap();
    assert!(nodes(&app, "animated-editor").is_empty());
    assert_eq!(cleaned.get(), 1);
    app.dispatch(panel, Event::CollapseComplete { revision })
        .unwrap();
    assert_eq!(cleaned.get(), 1);
    open.revise(|open| *open = true);
    app.flush().unwrap();
    assert_ne!(nodes(&app, "animated-editor"), [input]);
    app.clear().unwrap();
    assert_eq!(cleaned.get(), 2);
}
impl From<RuntimeId> for MotionHost {
    fn from(runtime: RuntimeId) -> Self {
        Self(Memory::new(runtime))
    }
}
#[test]
fn clear_zero_and_duration_validation_use_accepted_revisions_and_keepmounted_retains_body() {
    for policy in [
        sapid::ContentPolicy::Unmount,
        sapid::ContentPolicy::KeepMounted,
    ] {
        let app = App::new(MotionHost::from);
        let open = app.scope().cage(true);
        let duration = app.scope().cage(240u16);
        let animated = app.scope().cage(true);
        app.mount(
            sapid::collapsible("clear")
                .open(open)
                .height_transition(duration)
                .animated(animated)
                .content_policy(policy)
                .content(|| glory::ui::input().class("body")),
        )
        .unwrap();
        let panel = nodes(&app, "sapid-collapsible-panel")[0];
        let input = nodes(&app, "body")[0];
        open.revise(|open| *open = false);
        app.flush().unwrap();
        let old = config(&app, panel);
        animated.revise(|animated| *animated = false);
        app.flush().unwrap();
        let immediate = config(&app, panel);
        assert_eq!(immediate.duration_millis, 0);
        assert_eq!(immediate.revision, old.revision + 1);
        app.dispatch(
            panel,
            Event::CollapseComplete {
                revision: old.revision,
            },
        )
        .unwrap();
        assert_eq!(nodes(&app, "body"), [input]);
        app.dispatch(
            panel,
            Event::CollapseComplete {
                revision: immediate.revision,
            },
        )
        .unwrap();
        assert_eq!(
            nodes(&app, "body").is_empty(),
            policy == sapid::ContentPolicy::Unmount
        );
        let before = config(&app, panel);
        duration.revise(|value| *value = 501);
        assert!(app.flush().is_err());
        assert_eq!(config(&app, panel), before);
        duration.revise(|value| *value = 0);
        animated.revise(|value| *value = true);
        open.revise(|value| *value = true);
        app.flush().unwrap();
        assert_eq!(config(&app, panel).duration_millis, 0);
        let next = config(&app, panel);
        open.revise(|value| *value = true);
        app.flush().unwrap();
        assert_eq!(config(&app, panel), next);
    }
}
#[test]
fn no_completion_driver_ssr_and_memory_close_unmount_immediately_and_controlled_refusal_is_silent()
 {
    let app = App::memory();
    let open = app.scope().cage(true);
    let proposals = Rc::new(Cell::new(0));
    let count = proposals.clone();
    app.mount(
        sapid::collapsible("no-driver")
            .open(open)
            .height_transition(240)
            .on_open_change(move |_| count.set(count.get() + 1))
            .content(|| glory::ui::input().class("body")),
    )
    .unwrap();
    let trigger = nodes(&app, "sapid-collapsible-trigger")[0];
    app.dispatch(trigger, Event::Activate).unwrap();
    assert_eq!(proposals.get(), 1);
    assert!(!nodes(&app, "body").is_empty());
    open.revise(|open| *open = false);
    app.flush().unwrap();
    assert!(nodes(&app, "body").is_empty());
    assert_eq!(proposals.get(), 1);
    let snapshot = glory::ssr::snapshot(
        sapid::collapsible("server")
            .height_transition(240)
            .content(|| glory::ui::text().content("Not mounted")),
    )
    .unwrap();
    assert!(snapshot.html.contains("data-gly-collapse=\"closed\""));
    assert!(!snapshot.html.contains("Not mounted"));
    assert!(
        app.mount(sapid::collapsible("invalid").height_transition(501))
            .is_err()
    );
}

#[path = "../examples/web/collapse_motion_form.rs"]
mod fixture;
#[test]
fn four_theme_fixture_is_deterministic_and_whole_owner_retirement_does_not_write_disposed_counters()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let first = glory::ssr::snapshot(fixture::collapse_motion_form(theme.clone())).unwrap();
        let second =
            glory::ssr::snapshot(fixture::collapse_motion_form(theme.clone())).unwrap();
        assert_eq!(first.html, second.html);
        let app = App::new(MotionHost::from);
        app.mount(fixture::collapse_motion_form(theme)).unwrap();
        let action = |label: &str| {
            let tree = app.tree();
            let mut pending = tree.roots().to_vec();
            while let Some(id) = pending.pop() {
                let node = tree.node(id).unwrap();
                if matches!(node.descriptor.view().props,Props::Button(props)if props.label==label)
                {
                    return id;
                }
                pending.extend(&node.children);
            }
            panic!("missing {label}")
        };
        app.dispatch(action("Open from source"), Event::Activate)
            .unwrap();
        assert_eq!(nodes(&app, "animated-collapse-editor").len(), 1);
        app.dispatch(action("Close from source"), Event::Activate)
            .unwrap();
        assert_eq!(nodes(&app, "animated-collapse-editor").len(), 1);
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
}
