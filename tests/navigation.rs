use glory::prelude::*;
use glory::render::{Current, EventKind, NodeId, Props, Role};
use sapid::{
    ContentPolicy, PageItem, breadcrumb_item, breadcrumbs, nav_link, pagination,
    pagination_window,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
fn all(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut result = Vec::new();
    while let Some(id) = pending.pop() {
        pending.extend(tree.node(id).unwrap().children.iter().rev().copied());
        result.push(id);
    }
    result
}
fn button(app: &App, label: &str) -> NodeId {
    all(app).into_iter().find(|id|matches!(&app.tree().node(*id).unwrap().descriptor.view().props.clone(),Props::Button(p) if p.label==label)).unwrap_or_else(||panic!("missing button {label}"))
}
fn current(app: &App) -> Vec<NodeId> {
    all(app)
        .into_iter()
        .filter(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .is_some_and(|s| s.current == Some(Current::Page))
        })
        .collect()
}
#[test]
fn window_handles_limits_gaps_dedup_and_unknown_total_without_scanning_total() {
    assert_eq!(
        pagination_window(5, Some(10), 1, 1).unwrap(),
        vec![
            PageItem::Page(1),
            PageItem::Gap,
            PageItem::Page(4),
            PageItem::Page(5),
            PageItem::Page(6),
            PageItem::Gap,
            PageItem::Page(10)
        ]
    );
    assert_eq!(
        pagination_window(2, Some(3), 8, 8).unwrap(),
        vec![PageItem::Page(1), PageItem::Page(2), PageItem::Page(3)]
    );
    assert_eq!(
        pagination_window(u64::MAX, None, 8, 8).unwrap(),
        vec![PageItem::Page(u64::MAX)]
    );
    assert_eq!(
        pagination_window(1, Some(1), 0, 0).unwrap(),
        vec![PageItem::Page(1)]
    );
    for page in [1, 2, 9, u64::MAX / 2, u64::MAX - 1, u64::MAX] {
        let window = pagination_window(page, Some(u64::MAX), 8, 8).unwrap();
        assert!(window.len() <= 35);
        assert!(window.contains(&PageItem::Page(page)));
        assert!(
            !window
                .windows(2)
                .any(|pair| pair == [PageItem::Gap, PageItem::Gap])
        );
    }
    for (page, total, siblings, boundaries) in [
        (0, None, 1, 1),
        (1, Some(0), 1, 1),
        (3, Some(2), 1, 1),
        (1, None, 9, 1),
        (1, None, 1, 9),
    ] {
        assert!(pagination_window(page, total, siblings, boundaries).is_err());
    }
}
#[test]
fn local_pages_change_with_boundaries_and_callbacks_before_observers() {
    let app = App::memory();
    let trace = Rc::new(RefCell::new(Vec::new()));
    let a = trace.clone();
    let b = trace.clone();
    app.mount(
        pagination("pages")
            .total(Some(3))
            .on_page_change(move |page| a.borrow_mut().push(page.to_string()))
            .on(EventKind::Activate, move |_| {
                b.borrow_mut().push("observe".into())
            }),
    )
    .unwrap();
    assert!(
        !app.dispatch(button(&app, "First page"), Event::Activate)
            .unwrap()
    );
    assert!(
        !app.dispatch(button(&app, "Previous page"), Event::Activate)
            .unwrap()
    );
    app.dispatch(button(&app, "Next page"), Event::Activate)
        .unwrap();
    assert_eq!(*trace.borrow(), vec!["2", "observe"]);
    assert_eq!(current(&app), vec![button(&app, "Page 2")]);
    app.dispatch(button(&app, "Last page"), Event::Activate)
        .unwrap();
    assert_eq!(current(&app), vec![button(&app, "Page 3")]);
    assert!(
        !app.dispatch(button(&app, "Next page"), Event::Activate)
            .unwrap()
    );
    app.dispatch(button(&app, "Page 3"), Event::Activate)
        .unwrap();
    assert_eq!(trace.borrow().len(), 5); // current is a no-op but an observable activation
}
#[test]
fn controlled_pages_propose_only_and_external_changes_are_silent() {
    let app = App::memory();
    let page = app.scope().cage(2);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = calls.clone();
    app.mount(
        pagination("pages")
            .page(page)
            .default_page(3)
            .total(Some(4))
            .on_page_change(move |page| output.borrow_mut().push(page)),
    )
    .unwrap();
    app.dispatch(button(&app, "Next page"), Event::Activate)
        .unwrap();
    assert_eq!(page.get(), 2);
    assert_eq!(*calls.borrow(), vec![3]);
    let ids = all(&app);
    page.revise(|old| *old = 4);
    app.flush().unwrap();
    assert_eq!(ids, all(&app));
    assert_eq!(current(&app), vec![button(&app, "Page 4")]);
    assert_eq!(calls.borrow().len(), 1);
}
#[test]
fn invalid_dynamic_page_or_total_preserves_accepted_ui_and_callback_requests() {
    let app = App::memory();
    let page = app.scope().cage(4);
    let total = app.scope().cage(Some(8));
    let calls = Rc::new(RefCell::new(Vec::new()));
    let out = calls.clone();
    app.mount(
        pagination("pages")
            .page(page)
            .total(total)
            .on_page_change(move |page| out.borrow_mut().push(page)),
    )
    .unwrap();
    page.revise(|old| *old = 9);
    assert!(app.flush().is_err());
    assert_eq!(current(&app), vec![button(&app, "Page 4")]);
    app.dispatch(button(&app, "Next page"), Event::Activate)
        .unwrap();
    assert_eq!(*calls.borrow(), vec![5]);
    total.revise(|old| *old = Some(0));
    assert!(app.flush().is_err());
    app.dispatch(button(&app, "Last page"), Event::Activate)
        .unwrap();
    assert_eq!(*calls.borrow(), vec![5, 8]);
    app.scope()
        .batch(|| {
            page.revise(|old| *old = 2);
            total.revise(|old| *old = Some(3));
        })
        .unwrap();
    assert_eq!(current(&app), vec![button(&app, "Page 2")]);
}
#[test]
fn pending_source_change_uses_previously_displayed_page_for_request() {
    let app = App::memory();
    let page = app.scope().cage(2);
    let proposal = Rc::new(Cell::new(0));
    let out = proposal.clone();
    app.mount(
        pagination("pages")
            .page(page)
            .total(Some(10))
            .on_page_change(move |page| out.set(page)),
    )
    .unwrap();
    page.revise(|old| *old = 8);
    app.dispatch(button(&app, "Next page"), Event::Activate)
        .unwrap();
    assert_eq!(proposal.get(), 3);
}
#[test]
fn unknown_total_uses_explicit_has_next_and_guards_u64_overflow() {
    let app = App::memory();
    let page = app.scope().cage(10);
    let has_next = app.scope().cage(false);
    app.mount(pagination("unknown").page(page).has_next(has_next))
        .unwrap();
    assert!(
        !app.dispatch(button(&app, "Next page"), Event::Activate)
            .unwrap()
    );
    assert!(
        !app.dispatch(button(&app, "Last page"), Event::Activate)
            .unwrap()
    );
    has_next.revise(|old| *old = true);
    app.flush().unwrap();
    assert!(
        app.dispatch(button(&app, "Next page"), Event::Activate)
            .unwrap()
    );
    page.revise(|old| *old = u64::MAX);
    app.flush().unwrap();
    assert!(
        !app.dispatch(button(&app, "Next page"), Event::Activate)
            .unwrap()
    );
    assert_eq!(
        current(&app),
        vec![button(&app, &format!("Page {}", u64::MAX))]
    );
}
#[test]
fn disabled_pagination_native_controls_reject_every_request() {
    let app = App::memory();
    app.mount(
        pagination("disabled")
            .default_page(2)
            .total(Some(3))
            .enabled(false)
            .on_page_change(|_| panic!("disabled")),
    )
    .unwrap();
    for id in all(&app) {
        if matches!(
            app.tree().node(id).unwrap().descriptor.view().props.clone(),
            Props::Button(_)
        ) {
            assert!(!app.dispatch(id, Event::Activate).unwrap());
        }
    }
}
fn crumbs() -> Vec<sapid::BreadcrumbItem> {
    (0..7)
        .map(|i| {
            breadcrumb_item(format!("item{i}"), format!("Item {i}")).href(format!("/item{i}"))
        })
        .collect()
}
#[test]
fn breadcrumb_navigation_has_list_current_links_and_stable_folded_shells() {
    let app = App::memory();
    app.mount(breadcrumbs("trail").items(crumbs()).max_items(4))
        .unwrap();
    let ids = all(&app);
    assert_eq!(current(&app).len(), 1);
    assert!(all(&app).iter().any(
        |id| app.tree().node(*id).unwrap().descriptor.view().role == Some(Role::Navigation)
    ));
    let hidden = all(&app)
        .iter()
        .filter(|id| {
            app.tree()
                .node(**id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .is_some_and(|s| s.hidden)
        })
        .count();
    assert_eq!(hidden, 4);
    app.dispatch(button(&app, "Expand breadcrumb path"), Event::Activate)
        .unwrap();
    assert_eq!(ids, all(&app));
    assert!(!all(&app).iter().any(|id| {
        app.tree()
            .node(*id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .is_some_and(|s| s.hidden)
    }));
    app.dispatch(button(&app, "Collapse breadcrumb path"), Event::Activate)
        .unwrap();
    assert_eq!(ids, all(&app));
}
#[test]
fn breadcrumb_current_middle_stays_visible_and_invalid_current_is_rejected() {
    let app = App::memory();
    let value = app.scope().cage("item3".to_owned());
    app.mount(
        breadcrumbs("trail")
            .items(crumbs())
            .max_items(3)
            .current(value),
    )
    .unwrap();
    let link = current(&app)[0];
    let parent = app.tree().node(link).unwrap().parent.unwrap();
    assert!(
        !app.tree()
            .node(parent)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
    value.revise(|old| *old = "missing".into());
    assert!(app.flush().is_err());
    assert_eq!(current(&app), vec![link]);
}
#[test]
fn breadcrumb_controlled_expansion_rejects_proposal_without_writing_input() {
    let app = App::memory();
    let open = app.scope().cage(false);
    let calls = Rc::new(Cell::new(0));
    let out = calls.clone();
    app.mount(
        breadcrumbs("trail")
            .items(crumbs())
            .expanded(open)
            .on_expanded_change(move |next| {
                assert!(next);
                out.set(out.get() + 1)
            }),
    )
    .unwrap();
    app.dispatch(button(&app, "Expand breadcrumb path"), Event::Activate)
        .unwrap();
    assert!(!open.get());
    assert_eq!(calls.get(), 1);
    open.revise(|old| *old = true);
    app.flush().unwrap();
    assert_eq!(calls.get(), 1);
}
#[test]
fn nav_link_keeps_navigation_and_expansion_distinct_without_selected_claims() {
    let app = App::memory();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let a = calls.clone();
    let b = calls.clone();
    let c = calls.clone();
    app.mount(
        nav_link("docs")
            .label("Docs")
            .href("/docs")
            .current(true)
            .highlighted(true)
            .children(|| glory::ui::text().content("Child"))
            .on_navigate(move |request| a.borrow_mut().push(request.href))
            .on_expanded_change(move |open| b.borrow_mut().push(open.to_string()))
            .on(EventKind::Activate, move |_| {
                c.borrow_mut().push("observer".into())
            }),
    )
    .unwrap();
    let link = current(&app)[0];
    app.dispatch(link, Event::Activate).unwrap();
    assert_eq!(*calls.borrow(), vec!["/docs", "observer"]);
    app.dispatch(button(&app, "Expand Docs"), Event::Activate)
        .unwrap();
    assert_eq!(
        *calls.borrow(),
        vec!["/docs", "observer", "true", "observer"]
    );
    assert!(all(&app).iter().all(|id| {
        app.tree()
            .node(*id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .is_none_or(|s| s.selected.is_none())
    }));
}
#[test]
fn nav_children_owner_policy_retires_on_unmount_and_keeps_mounted_state_explicitly() {
    for (policy, expected) in [(ContentPolicy::Unmount, 1), (ContentPolicy::KeepMounted, 0)] {
        let app = App::memory();
        let drops = Rc::new(Cell::new(0));
        let out = drops.clone();
        app.mount(
            nav_link("docs")
                .label("Docs")
                .children(move || {
                    let out = out.clone();
                    glory::ui::component(move |scope| {
                        let out = out.clone();
                        scope.on_cleanup(move || out.set(out.get() + 1))?;
                        glory::ui::text().content("Child").show_in(scope)
                    })
                })
                .content_policy(policy),
        )
        .unwrap();
        app.dispatch(button(&app, "Expand Docs"), Event::Activate)
            .unwrap();
        app.dispatch(button(&app, "Collapse Docs"), Event::Activate)
            .unwrap();
        assert_eq!(drops.get(), expected);
        app.clear().unwrap();
        assert_eq!(drops.get(), 1);
    }
}
#[test]
fn semantic_ssr_current_is_page_or_step_and_absence_clears_metadata() {
    let html = glory::ssr::render(glory::ui::group().fill((
        breadcrumbs("trail").items([
            breadcrumb_item("home", "Home").href("/"),
            breadcrumb_item("docs", "Docs"),
        ]),
        pagination("pages").total(Some(5)),
        nav_link("docs").label("Docs").href("/docs").current(true),
    )))
    .unwrap();
    assert_eq!(html.matches("aria-current=\"page\"").count(), 3);
    assert!(html.contains("role=\"navigation\""));
    assert!(html.contains("role=\"listitem\""));
    assert!(!html.contains("aria-selected"));
    assert!(!html.contains("role=\"menu\""));
    let html = glory::ssr::render(
        sapid::anchor()
            .label("Step")
            .href("/step")
            .current(Some(Current::Step)),
    )
    .unwrap();
    assert!(html.contains("aria-current=\"step\""));
}
#[test]
fn invalid_initial_navigation_configs_create_no_nodes() {
    let app = App::memory();
    assert!(app.mount(pagination("p").default_page(0)).is_err());
    assert!(app.tree().roots().is_empty());
    assert!(
        app.mount(breadcrumbs("b").items([
            breadcrumb_item("same", "One"),
            breadcrumb_item("same", "Two")
        ]))
        .is_err()
    );
    assert!(app.tree().roots().is_empty());
    let visited = Rc::new(Cell::new(0));
    let count = visited.clone();
    let oversized = breadcrumbs("bounded").items((0..1_000_000).map(move |index| {
        count.set(count.get() + 1);
        breadcrumb_item(format!("item{index}"), "Item")
    }));
    assert_eq!(visited.get(), sapid::MAX_BREADCRUMB_ITEMS + 1);
    assert!(app.mount(oversized).is_err());
    assert!(app.tree().roots().is_empty());
}

#[test]
fn navigation_retirement_releases_callbacks_and_resets_external_root_reference() {
    struct Retired(Rc<Cell<usize>>);
    impl Drop for Retired {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let app = App::memory();
    let drops = Rc::new(Cell::new(0));
    let page = Retired(drops.clone());
    let link = Retired(drops.clone());
    let path = Retired(drops.clone());
    let reference = app.scope().cage(None);
    app.mount(
        glory::ui::group().fill((
            pagination("retired-pages")
                .node_ref(reference)
                .total(Some(10))
                .on_page_change(move |_| {
                    let _ = &page;
                }),
            nav_link("retired-link")
                .href("/docs")
                .on_navigate(move |_| {
                    let _ = &link;
                }),
            breadcrumbs("retired-path")
                .items(crumbs())
                .on_expanded_change(move |_| {
                    let _ = &path;
                }),
        )),
    )
    .unwrap();
    assert!(reference.get().is_some());
    app.clear().unwrap();
    assert_eq!(drops.get(), 3);
    assert!(reference.get().is_none());
}
