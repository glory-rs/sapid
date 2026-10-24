use glory::render::{NodeId, Role};
use glory::ui::{App, Widget};
use sapid::{MAX_TOUR_STEPS, TourHandle, tour, tour_context, tour_step, tour_target};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[path = "../examples/web/src/tour_form.rs"]
mod fixture;
fn panel(app: &App) -> NodeId {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    while let Some(id) = todo.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|c| c == "sapid-tour")
        {
            return id;
        }
        todo.extend(&node.children);
    }
    panic!("Tour panel")
}
#[test]
fn real_dialog_has_stable_named_anchor_relations_and_measureless_initial_shape() {
    let app = App::memory();
    let ctx = tour_context(app.scope(), "guide").unwrap();
    app.mount(glory::ui::view().fill((
        tour_target(&ctx, "editor").fill(glory::ui::input().name("Draft")),
        tour(&ctx, "intro").default_open(true).step(tour_step(
            "editor",
            "Edit",
            "Type a draft",
        )),
    )))
    .unwrap();
    assert_eq!(ctx.len(), 1);
    let tree = app.tree();
    let d = tree.node(panel(&app)).unwrap().descriptor.view();
    assert_eq!(d.role, Some(Role::Dialog));
    let s = d.semantics.as_ref().unwrap();
    assert!(s.hidden, "SSR/headless cannot infer a measured anchor");
    assert!(s.labelled_by.is_some() && s.described_by.is_some());
    let layer = s.layer.as_ref().unwrap();
    assert!(!layer.modal && !layer.outside);
    assert!(layer.anchor.is_some() && layer.anchor == layer.return_focus);
    assert_eq!(app.stats().queries, 0);
}
#[test]
fn missing_targets_are_valid_late_relations_without_invented_rows_or_queries() {
    let app = App::memory();
    let ctx = tour_context(app.scope(), "late").unwrap();
    app.mount(tour(&ctx, "intro").default_open(true).step(tour_step(
        "not-mounted",
        "Waiting",
        "A real target is required",
    )))
    .unwrap();
    assert_eq!(ctx.len(), 0);
    assert_eq!(app.stats().queries, 0);
    assert!(
        app.tree()
            .node(panel(&app))
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
}
#[test]
fn local_commands_accumulate_in_one_tick_and_retired_handles_cannot_reopen() {
    let app = App::memory();
    let ctx = tour_context(app.scope(), "local").unwrap();
    let out = Rc::new(RefCell::new(None::<TourHandle>));
    let sink = out.clone();
    let changes = Rc::new(RefCell::new(Vec::new()));
    let changed = changes.clone();
    let completed = Rc::new(Cell::new(0));
    let complete = completed.clone();
    app.mount(
        tour(&ctx, "intro")
            .default_open(true)
            .steps([
                tour_step("a", "A", "First"),
                tour_step("b", "B", "Second"),
                tour_step("c", "C", "Third"),
            ])
            .on_current_change(move |i| changed.borrow_mut().push(i))
            .on_complete(move || complete.set(complete.get() + 1))
            .on_ready(move |handle| *sink.borrow_mut() = Some(handle)),
    )
    .unwrap();
    let handle = out.borrow().clone().unwrap();
    let node = panel(&app);
    handle.next().unwrap();
    handle.next().unwrap();
    app.flush().unwrap();
    assert_eq!(*changes.borrow(), [1, 2]);
    assert_eq!(panel(&app), node);
    handle.next().unwrap();
    app.flush().unwrap();
    assert_eq!(completed.get(), 1);
    app.clear().unwrap();
    assert!(handle.next().is_err());
    assert!(handle.refresh().is_err());
    assert!(ctx.is_empty());
}
#[test]
fn controlled_step_and_close_refusal_remain_proposals_without_partial_navigation() {
    let app = App::memory();
    let ctx = tour_context(app.scope(), "controlled").unwrap();
    let current = app.scope().cage(0);
    let open = app.scope().cage(true);
    let out = Rc::new(RefCell::new(None::<TourHandle>));
    let sink = out.clone();
    let proposed = Rc::new(RefCell::new(Vec::new()));
    let changes = proposed.clone();
    let closes = Rc::new(Cell::new(0));
    let closing = closes.clone();
    app.mount(
        tour(&ctx, "intro")
            .open(open)
            .current(current)
            .steps([tour_step("a", "First", "A"), tour_step("b", "Second", "B")])
            .on_current_change(move |i| changes.borrow_mut().push(i))
            .on_open_change(move |_| closing.set(closing.get() + 1))
            .on_ready(move |handle| *sink.borrow_mut() = Some(handle)),
    )
    .unwrap();
    let handle = out.borrow().clone().unwrap();
    handle.next().unwrap();
    handle.next().unwrap();
    handle.cancel().unwrap();
    app.flush().unwrap();
    assert_eq!(current.get(), 0);
    assert!(open.get());
    assert_eq!(*proposed.borrow(), [1, 1]);
    assert_eq!(closes.get(), 1);
    current.revise(|value| *value = 1);
    app.flush().unwrap();
    handle.previous().unwrap();
    assert_eq!(proposed.borrow().last(), Some(&0));
    assert!(handle.go_to(2).is_err());
}
#[test]
fn declarations_reject_empty_duplicate_invalid_and_overbudget_before_panel_nodes() {
    for mode in 0..5 {
        let app = App::memory();
        let ctx = tour_context(app.scope(), "reject").unwrap();
        let guide = match mode {
            0 => tour(&ctx, "intro"),
            1 => tour(&ctx, "intro")
                .steps([tour_step("a", "A", "A"), tour_step("a", "Again", "A")]),
            2 => tour(&ctx, "intro").step(tour_step("a", " ", "A")),
            3 => tour(&ctx, "intro")
                .default_current(1)
                .step(tour_step("a", "A", "A")),
            _ => tour(&ctx, "intro")
                .steps((0..=MAX_TOUR_STEPS).map(|i| tour_step(format!("s{i}"), "A", "A"))),
        };
        assert!(app.mount(guide).is_err());
        assert!(app.tree().is_empty());
    }
}
#[test]
fn million_item_iterator_is_consumed_only_through_the_first_rejected_declaration() {
    let app = App::memory();
    let ctx = tour_context(app.scope(), "bounded").unwrap();
    let count = Rc::new(Cell::new(0));
    let consumed = count.clone();
    let guide = tour(&ctx, "intro").steps((0..1_000_000).map(move |i| {
        consumed.set(consumed.get() + 1);
        tour_step(format!("s{i}"), "Title", "Body")
    }));
    assert_eq!(count.get(), MAX_TOUR_STEPS + 1);
    assert!(app.mount(guide).is_err());
    assert!(app.tree().is_empty());
}
#[test]
fn target_reservations_bound_nested_construction_and_cleanup_current_registry_only() {
    let app = App::memory();
    let ctx = tour_context(app.scope(), "targets").unwrap();
    let target_context = ctx.clone();
    app.mount(glory::ui::widget(move |scope| {
        for i in 0..MAX_TOUR_STEPS {
            tour_target(&target_context, format!("t{i}"))
                .fill(glory::ui::text().content("Target"))
                .show_in(scope)?;
        }
        Ok(())
    }))
    .unwrap();
    assert_eq!(ctx.len(), MAX_TOUR_STEPS);
    assert!(app.mount(tour_target(&ctx, "overflow")).is_err());
    app.clear().unwrap();
    assert!(ctx.is_empty());
}
#[test]
fn four_theme_ssr_is_repeatable_with_real_hidden_dialog_and_finite_anchor_policy() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let first = glory::ssr::snapshot(fixture::tour_form(theme.clone())).unwrap();
        let second = glory::ssr::snapshot(fixture::tour_form(theme)).unwrap();
        assert_eq!(first.html, second.html);
        assert!(first.html.contains("role=\"dialog\""));
        assert!(first.html.contains("data-gly-hidden=\"\""));
        assert!(first.html.contains("data-gly-anchor-side=\"bottom\""));
        assert!(first.html.contains("sapid-tour-target"));
        assert!(
            first.html.contains("aria-labelledby=") && first.html.contains("aria-describedby=")
        );
    }
}
