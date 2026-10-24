use glory::render::{NodeId, Props};
use glory::ui::{App, Widget};
use sapid::{portal, portal_context, portal_target};
use std::{cell::Cell, rc::Rc};
#[path = "../examples/web/portal_form.rs"]
mod fixture;
fn target(app: &App, key: &str) -> NodeId {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    while let Some(id) = todo.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .semantics
            .as_ref()
            .and_then(|s| s.id.as_ref())
            .is_some_and(|id| id.as_str().ends_with(key))
        {
            return id;
        }
        todo.extend(&node.children);
    }
    panic!("missing target {key}")
}
fn inside(app: &App, parent: NodeId, child: NodeId) -> bool {
    app.scope().contains_node(parent, child).unwrap()
}
#[test]
fn source_order_is_independent_and_content_is_a_boxless_semantic_child_of_the_target() {
    for source_first in [true, false] {
        let app = App::memory();
        let ctx = portal_context(app.scope(), "ordered").unwrap();
        let editor = app.scope().cage(None);
        let context = ctx.clone();
        app.mount(glory::ui::component(move |scope| {
            let source = portal(&context, "destination").content(move || {
                glory::ui::input()
                    .name("Portaled editor")
                    .node_ref(editor)
                    .value("Draft")
            });
            let destination = portal_target(&context, "destination")
                .fill(glory::ui::text().content("Target content"));
            if source_first {
                source.show_in(scope)?;
                destination.show_in(scope)?;
            } else {
                destination.show_in(scope)?;
                source.show_in(scope)?;
            }
            Ok(())
        }))
        .unwrap();
        let destination = target(&app, "destination");
        let editor = editor.get().unwrap();
        assert!(inside(&app, destination, editor));
        let group = app.tree().node(editor).unwrap().parent.unwrap();
        assert!(matches!(
            app.tree().node(group).unwrap().descriptor.view().props,
            Props::Group
        ));
        assert_eq!(ctx.targets(), 1);
        assert_eq!(ctx.sources(), 1);
        app.clear().unwrap();
        assert_eq!(ctx.targets(), 0);
        assert_eq!(ctx.sources(), 0);
    }
}
#[test]
fn target_retirement_moves_source_owned_content_before_dispose_and_recovers_the_same_nodes() {
    let app = App::memory();
    let ctx = portal_context(app.scope(), "lifetime").unwrap();
    let present = app.scope().cage(true);
    let editor = app.scope().cage(None);
    let calls = Rc::new(Cell::new(0));
    let invoked = calls.clone();
    let lost = Rc::new(Cell::new(0));
    let loss = lost.clone();
    let destination = ctx.clone();
    app.mount(
        glory::ui::view().fill((
            glory::ui::when(present, move || portal_target(&destination, "slot")),
            portal(&ctx, "slot")
                .content(move || {
                    invoked.set(invoked.get() + 1);
                    glory::ui::input().node_ref(editor).value("Keep")
                })
                .on_target_lost(move |_| loss.set(loss.get() + 1)),
        )),
    )
    .unwrap();
    let editor = editor.get().unwrap();
    let old = target(&app, "slot");
    assert!(inside(&app, old, editor));
    present.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(app.tree().node(old).is_err());
    assert!(app.tree().node(editor).is_ok());
    assert!(!app.scope().is_active(editor).unwrap());
    assert_eq!(ctx.targets(), 0);
    assert_eq!(ctx.sources(), 1);
    assert_eq!(lost.get(), 1);
    present.revise(|v| *v = true);
    app.flush().unwrap();
    let new = target(&app, "slot");
    assert_ne!(old, new);
    assert!(inside(&app, new, editor));
    assert!(app.scope().is_active(editor).unwrap());
    assert_eq!(calls.get(), 1);
    app.clear().unwrap();
    assert_eq!(ctx.targets(), 0);
    assert_eq!(ctx.sources(), 0);
}
#[test]
fn local_disable_and_dynamic_destination_preserve_the_mounted_factory_and_old_target_children()
{
    let app = App::memory();
    let ctx = portal_context(app.scope(), "move").unwrap();
    let enabled = app.scope().cage(true);
    let selected = app.scope().cage("a".to_string());
    let editor = app.scope().cage(None);
    app.mount(
        glory::ui::view().fill((
            portal_target(&ctx, "a"),
            portal_target(&ctx, "b"),
            portal(&ctx, selected)
                .enabled(enabled)
                .content(move || glory::ui::input().node_ref(editor).value("Keep")),
        )),
    )
    .unwrap();
    let editor = editor.get().unwrap();
    let a = target(&app, "a");
    let b = target(&app, "b");
    assert!(inside(&app, a, editor));
    selected.revise(|v| *v = "b".into());
    app.flush().unwrap();
    assert!(inside(&app, b, editor));
    assert!(!inside(&app, a, editor));
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(!app.scope().is_active(editor).unwrap());
    enabled.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(inside(&app, b, editor));
    assert!(app.scope().is_active(editor).unwrap());
}
#[test]
fn source_retirement_removes_moved_descendants_without_disposing_the_target() {
    let app = App::memory();
    let ctx = portal_context(app.scope(), "source").unwrap();
    let mounted = app.scope().cage(true);
    let reference = app.scope().cage(None);
    let sources = ctx.clone();
    app.mount(glory::ui::view().fill((
        portal_target(&ctx, "destination"),
        glory::ui::when(mounted, move || {
            portal(&sources, "destination")
                .content(move || glory::ui::input().node_ref(reference))
        }),
    )))
    .unwrap();
    let input = reference.get().unwrap();
    let destination = target(&app, "destination");
    mounted.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(app.tree().node(input).is_err());
    assert!(app.tree().node(destination).is_ok());
    assert_eq!(ctx.sources(), 0);
    mounted.revise(|v| *v = true);
    app.flush().unwrap();
    assert_ne!(reference.get().unwrap(), input);
    assert_eq!(ctx.sources(), 1);
}
#[test]
fn cross_runtime_context_and_empty_content_reject_before_creating_nodes() {
    let owner = App::memory();
    let ctx = portal_context(owner.scope(), "foreign").unwrap();
    let other = App::memory();
    assert!(other.mount(portal_target(&ctx, "a")).is_err());
    assert!(other.tree().is_empty());
    assert!(owner.mount(portal(&ctx, "a")).is_err());
    assert!(owner.tree().is_empty());
    assert_eq!(ctx.sources(), 0);
}
#[test]
fn content_cannot_move_into_its_own_descendant_and_refusal_preserves_the_accepted_prefix() {
    let app = App::memory();
    let ctx = portal_context(app.scope(), "cycle").unwrap();
    let selected = app.scope().cage("outside".to_string());
    let reference = app.scope().cage(None);
    let nested = ctx.clone();
    app.mount(glory::ui::view().fill((
        portal_target(&ctx, "outside"),
        portal(&ctx, selected).content(move || {
            glory::ui::view().fill((
                glory::ui::input().node_ref(reference),
                portal_target(&nested, "inside"),
            ))
        }),
    )))
    .unwrap();
    let reference = reference.get().unwrap();
    let outside = target(&app, "outside");
    selected.revise(|v| *v = "inside".into());
    assert!(app.flush().is_err());
    assert!(inside(&app, outside, reference));
    assert!(app.tree().node(reference).is_ok());
}
// Scoped targets may be built from a different live Owner than their logical
// parent. Losing the parent must park the source range outside that subtree.
#[test]
fn foreign_owned_source_inside_a_retiring_target_is_parked_without_being_disposed() {
    let app = App::memory();
    let ctx = portal_context(app.scope(), "foreign-owner").unwrap();
    let parent = app.scope().child().unwrap();
    let destination = parent.mount_with(|| portal_target(&ctx, "a")).unwrap();
    let target = target(&app, "a");
    let source = app.scope().child_in(target).unwrap();
    let reference = app.scope().cage(None);
    portal(&ctx, "a")
        .content(move || glory::ui::input().node_ref(reference).value("Keep"))
        .show_in(&source)
        .unwrap();
    app.flush().unwrap();
    let editor = reference.get().unwrap();
    drop(destination);
    app.flush().unwrap();
    assert!(app.tree().node(editor).is_ok());
    assert!(!app.scope().is_active(editor).unwrap());
    portal_target(&ctx, "a").show_in(app.scope()).unwrap();
    app.flush().unwrap();
    assert!(app.scope().is_active(editor).unwrap());
    drop(source);
    drop(parent);
}
#[test]
fn current_target_and_source_budgets_reject_before_extra_factories_and_reclaim_every_marker() {
    for targets in [true, false] {
        let app = App::memory();
        let ctx = portal_context(app.scope(), "budget").unwrap();
        let used = ctx.clone();
        let calls = Rc::new(Cell::new(0usize));
        let count = calls.clone();
        let result = app.mount(glory::ui::component(move |scope| {
            let limit = if targets {
                sapid::MAX_PORTAL_TARGETS
            } else {
                sapid::MAX_PORTALS
            };
            for i in 0..=limit {
                if targets {
                    portal_target(&used, format!("t{i}")).show_in(scope)?;
                } else {
                    let count = count.clone();
                    portal(&used, "missing")
                        .content(move || {
                            count.set(count.get() + 1);
                            glory::ui::text().content("Current source")
                        })
                        .show_in(scope)?;
                }
            }
            Ok(())
        }));
        assert!(result.is_err());
        assert!(app.tree().is_empty());
        assert_eq!(ctx.sources(), 0);
        assert_eq!(ctx.targets(), 0);
        assert_eq!(app.stats().disposal_boundaries, 0);
        assert_eq!(calls.get(), if targets { 0 } else { sapid::MAX_PORTALS });
    }
}
#[test]
fn source_and_target_retire_together_without_running_a_loss_callback_under_a_dead_owner() {
    let app = App::memory();
    let ctx = portal_context(app.scope(), "together").unwrap();
    let lost = Rc::new(Cell::new(0));
    let callback = lost.clone();
    app.mount(
        glory::ui::view().fill((
            portal_target(&ctx, "a").fill(glory::ui::text().content("Own child")),
            portal(&ctx, "a")
                .content(glory::ui::input)
                .on_target_lost(move |_| callback.set(callback.get() + 1)),
        )),
    )
    .unwrap();
    app.clear().unwrap();
    assert_eq!(lost.get(), 0);
    assert_eq!(ctx.sources(), 0);
    assert_eq!(ctx.targets(), 0);
    assert_eq!(app.stats().disposal_boundaries, 0);
    assert_eq!(app.stats().handlers, 0);
    assert_eq!(app.stats().queries, 0);
    assert_eq!(app.runtime_stats().queued, 0);
    assert_eq!(app.runtime_stats().queue_entries, 0);
}
#[test]
fn four_theme_ssr_uses_final_semantic_parent_order_and_deterministic_suspended_missing_ranges()
{
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let first = glory::ssr::snapshot(fixture::portal_form(theme.clone())).unwrap();
        let second = glory::ssr::snapshot(fixture::portal_form(theme.clone())).unwrap();
        assert_eq!(first.html, second.html);
        assert_eq!(
            first.html.matches("aria-label=\"Portaled draft\"").count(),
            1
        );
        let destination = first.html.find("Destination A").unwrap();
        let input = first.html.find("aria-label=\"Portaled draft\"").unwrap();
        let source = first.html.find("portal-source-location").unwrap();
        assert!(
            destination < input && input < source,
            "SSR serializes the actual semantic target child, not source location"
        );
        let first = glory::ssr::snapshot(fixture::portal_missing_form(theme.clone())).unwrap();
        let second = glory::ssr::snapshot(fixture::portal_missing_form(theme)).unwrap();
        assert_eq!(first.html, second.html);
        assert!(
            !first.html.contains("aria-label=\"Portaled draft\""),
            "suspended content is absent from serialized DOM"
        );
    }
}
#[test]
fn failing_target_child_construction_still_parks_a_live_foreign_source_before_target_disposal()
{
    let app = Rc::new(App::memory());
    let context = portal_context(app.scope(), "failed-target").unwrap();
    let source = app.scope().child().unwrap();
    let independent = source.clone();
    let reference = app.scope().cage(None);
    let during_build = app.clone();
    let targets = context.clone();
    let result = app.mount(portal_target(&context, "a").fill(glory::ui::component(
        move |_| {
            let name =
                glory::render::SemanticId::new("sapid-portal-13_failed-target-target-1_a")
                    .unwrap();
            let target = during_build.tree().named(&name).unwrap();
            let owner = independent.child_in(target)?;
            portal(&targets, "a")
                .content(move || glory::ui::input().node_ref(reference).value("Keep"))
                .show_in(&owner)?;
            independent.retain(owner)?;
            Err(glory::render::Error::InvalidProps("target child failed").into())
        },
    )));
    assert!(result.is_err());
    let editor = reference.get().unwrap();
    assert!(
        app.tree().node(editor).is_ok(),
        "independent source survives failed destination construction"
    );
    assert!(!app.scope().is_active(editor).unwrap());
    assert_eq!(context.targets(), 0);
    assert_eq!(context.sources(), 1);
    portal_target(&context, "a").show_in(app.scope()).unwrap();
    app.flush().unwrap();
    assert!(app.scope().is_active(editor).unwrap());
    source.owner().dispose();
    app.clear().unwrap();
    assert_eq!(context.sources(), 0);
    assert_eq!(app.stats().disposal_boundaries, 0);
}
