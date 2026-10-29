use glory::apace::{Config, Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{CollapseConfig, Event, MotionPreference, NodeId};
use std::{
    cell::Cell,
    rc::Rc,
    task::{Context, Waker},
};

fn nodes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut found = Vec::new();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|name| name == class)
        {
            found.push(id);
        }
        pending.extend(&node.children);
    }
    found
}
fn config(app: &App, id: NodeId) -> CollapseConfig {
    app.tree()
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .collapse
        .unwrap()
}
fn observe(host: &WindowApp) {
    host.observe(&Context::from_waker(Waker::noop())).unwrap();
}
fn height(host: &WindowApp, node: NodeId) -> f64 {
    Raster::new().draw(host.renderer()).unwrap();
    let physical = host
        .renderer()
        .engine_id(node, glory::html::Part::Main)
        .unwrap();
    host.renderer()
        .inspect(|doc| f64::from(doc.get_node(physical).unwrap().final_layout().size.height))
}
fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.1,
        "actual={actual},expected={expected}"
    );
}

#[test]
fn actual_collapsible_waits_for_native_close_preserves_reopened_editor_and_obeys_content_policy_in_four_themes()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        for policy in [
            sapid::ContentPolicy::Unmount,
            sapid::ContentPolicy::KeepMounted,
        ] {
            let host = WindowApp::with_config(Config {
                motion: MotionPreference::NoPreference,
                ..Default::default()
            });
            host.renderer().set_viewport(640, 480, 1.).unwrap();
            let app = host.app();
            let scope = app.scope();
            let open = scope.cage(true);
            let duration = scope.cage(200u16);
            let reference = scope.cage(None);
            let trigger = scope.cage(None);
            let draft = scope.cage("中文😀".to_owned());
            let cleaned = Rc::new(Cell::new(0));
            let retired = cleaned.clone();
            host.mount(
                sapid::collapsible("native-height")
                    .theme(theme.clone())
                    .open(open)
                    .height_transition(duration)
                    .content_policy(policy)
                    .panel_ref(reference)
                    .trigger_ref(trigger)
                    .content(move || {
                        let retired = retired.clone();
                        widget(move |scope| {
                            let retired = retired.clone();
                            scope.on_cleanup(move || retired.set(retired.get() + 1))?;
                            view()
                                .style(Style::new().height(Length::Px(80.)))
                                .fill(input().class("height-editor").value(draft))
                                .show_in(scope)
                        })
                    }),
            )
            .unwrap();
            observe(&host);
            let panel = reference.get().unwrap();
            let editor = nodes(app, "height-editor")[0];
            let physical = host
                .renderer()
                .engine_id(editor, glory::html::Part::Main)
                .unwrap();
            let full = height(&host, panel);
            assert!(full >= 80.);
            let control = physical;
            scope.action(glory::render::Action::Focus(editor)).unwrap();
            app.flush().unwrap();
            observe(&host);
            assert_eq!(
                host.renderer().inspect(|doc| doc.get_focussed_node_id()),
                Some(control)
            );
            draft.revise(|value| *value = "保留😀".into());
            app.flush().unwrap();
            open.revise(|value| *value = false);
            app.flush().unwrap();
            observe(&host);
            let old = config(app, panel).revision;
            assert!(!app.tree().active(editor));
            assert!(app.tree().inert(editor));
            assert_eq!(cleaned.get(), 0);
            let trigger = host
                .renderer()
                .engine_id(trigger.get().unwrap(), glory::html::Part::Main)
                .unwrap();
            assert_eq!(
                host.renderer().inspect(|doc| doc.get_focussed_node_id()),
                Some(trigger)
            );
            host.renderer().advance_time(0.1).unwrap();
            observe(&host);
            near(height(&host, panel), full / 2.);
            assert_eq!(nodes(app, "height-editor"), [editor]);
            open.revise(|value| *value = true);
            app.flush().unwrap();
            observe(&host);
            assert!(
                !app.dispatch(panel, Event::CollapseComplete { revision: old })
                    .unwrap()
            );
            assert_eq!(cleaned.get(), 0);
            host.renderer().advance_time(0.3).unwrap();
            observe(&host);
            near(height(&host, panel), full);
            assert_eq!(nodes(app, "height-editor"), [editor]);
            assert_eq!(
                host.renderer()
                    .engine_id(editor, glory::html::Part::Main)
                    .unwrap(),
                physical
            );
            open.revise(|value| *value = false);
            app.flush().unwrap();
            host.renderer().advance_time(0.4).unwrap();
            observe(&host);
            near(height(&host, panel), full / 2.);
            duration.revise(|value| *value = 0);
            app.flush().unwrap();
            observe(&host);
            near(height(&host, panel), 0.);
            assert_eq!(
                cleaned.get(),
                usize::from(policy == sapid::ContentPolicy::Unmount)
            );
            assert_eq!(
                nodes(app, "height-editor").is_empty(),
                policy == sapid::ContentPolicy::Unmount
            );
            assert!(!host.renderer().inspect(|doc| doc.is_animating()));
            duration.revise(|value| *value = 200);
            app.flush().unwrap();
            observe(&host);
            open.revise(|value| *value = true);
            app.flush().unwrap();
            observe(&host);
            host.renderer().advance_time(0.5).unwrap();
            observe(&host);
            near(height(&host, panel), full / 2.);
            let next = nodes(app, "height-editor")[0];
            assert_eq!(next == editor, policy == sapid::ContentPolicy::KeepMounted);
            host.renderer().advance_time(0.61).unwrap();
            observe(&host);
            near(height(&host, panel), full);
            app.clear().unwrap();
            assert_eq!(
                cleaned.get(),
                if policy == sapid::ContentPolicy::Unmount {
                    2
                } else {
                    1
                }
            );
            assert!(app.tree().is_empty());
            assert_eq!(host.renderer().stats().nodes, 0);
        }
    }
}
