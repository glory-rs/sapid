use glory::prelude::*;
use glory::render::{EventKind, LinkOpen, NodeId, Props};
use sapid::{AnchorRequest, anchor};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
fn find(app: &App) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if matches!(node.descriptor.view().props.clone(), Props::Link(_)) {
            return id;
        }
        pending.extend(node.children.iter().copied());
    }
    panic!("missing anchor")
}
#[test]
fn anchor_keeps_href_label_and_accessible_name_separate() {
    let app = App::memory();
    app.mount(
        anchor()
            .href("/docs")
            .label("Read")
            .name("Read documentation"),
    )
    .unwrap();
    let tree = app.tree();
    let d = tree.node(find(&app)).unwrap().descriptor.view();
    assert_eq!(d.name.as_deref(), Some("Read documentation"));
    assert!(matches!(&d.props,Props::Link(p)if p.target=="/docs"&&p.label=="Read"));
}
#[test]
fn navigation_request_precedes_generic_observers_and_does_not_navigate_implicitly() {
    let app = App::memory();
    let trace = Rc::new(RefCell::new(Vec::new()));
    let a = trace.clone();
    let b = trace.clone();
    app.mount(
        anchor()
            .href("https://example.test/docs")
            .open_in(LinkOpen::NewContext)
            .label("Docs")
            .on_navigate(move |request| {
                assert_eq!(
                    request,
                    AnchorRequest {
                        href: "https://example.test/docs".into(),
                        open: LinkOpen::NewContext
                    }
                );
                a.borrow_mut().push("typed")
            })
            .on(EventKind::Activate, move |_| {
                b.borrow_mut().push("observed")
            }),
    )
    .unwrap();
    app.dispatch(find(&app), Event::Activate).unwrap();
    assert_eq!(*trace.borrow(), vec!["typed", "observed"]);
}
#[test]
fn external_destination_and_open_context_updates_are_silent_and_preserve_identity() {
    let app = App::memory();
    let href = app.scope().cage("/old".to_owned());
    let open = app.scope().cage(LinkOpen::NewContext);
    let calls = Rc::new(Cell::new(0));
    let called = calls.clone();
    app.mount(
        anchor()
            .href(href)
            .open_in(open)
            .label("Docs")
            .on_navigate(move |_| called.set(1)),
    )
    .unwrap();
    let id = find(&app);
    href.revise(|v| *v = "/new".into());
    open.revise(|v| *v = LinkOpen::Current);
    app.flush().unwrap();
    assert_eq!(find(&app), id);
    assert_eq!(calls.get(), 0);
    let tree = app.tree();
    let d = tree.node(id).unwrap().descriptor.view();
    assert!(matches!(&d.props,Props::Link(p)if p.target=="/new"));
    assert_eq!(
        d.semantics.as_ref().unwrap().link_open,
        Some(LinkOpen::Current)
    );
}
#[test]
fn ssr_new_context_has_safe_rel_and_current_does_not_emit_context_attributes() {
    let html = glory::ssr::render(
        anchor()
            .href("https://example.test/?a=1&b=\"2\"")
            .label("Read <more>")
            .open_in(LinkOpen::NewContext),
    )
    .unwrap();
    assert!(html.contains("target=\"_blank\""));
    assert!(html.contains("rel=\"noopener noreferrer\""));
    assert!(html.contains("Read &lt;more&gt;"));
    assert!(html.contains("&amp;b=&quot;2&quot;"));
    let html = glory::ssr::render(anchor().href("/docs").label("Docs")).unwrap();
    assert!(!html.contains("target=\"_blank\""));
    assert!(!html.contains("rel="));
}
#[test]
fn owner_retirement_drops_navigation_capture_and_reference() {
    struct Count(Rc<Cell<usize>>);
    impl Drop for Count {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1)
        }
    }
    let app = App::memory();
    let count = Rc::new(Cell::new(0));
    let capture = Count(count.clone());
    let reference = app.scope().cage(None);
    app.mount(
        anchor()
            .href("/docs")
            .node_ref(reference)
            .on_navigate(move |_| {
                let _ = &capture;
            }),
    )
    .unwrap();
    app.clear().unwrap();
    assert_eq!(count.get(), 1);
    assert!(reference.get().is_none());
}
#[test]
fn unstyled_anchor_retains_custom_style_and_navigation_semantics() {
    let app = App::memory();
    app.mount(anchor().href("/docs").unstyled().style(Style {
        font_size: Some(18.0),
        ..Style::new()
    }))
    .unwrap();
    let tree = app.tree();
    let d = tree.node(find(&app)).unwrap().descriptor.view();
    assert_eq!(d.style.font_size, Some(18.0));
    assert_eq!(d.style.color, None);
    assert_eq!(
        d.semantics.as_ref().unwrap().link_open,
        Some(LinkOpen::Current)
    );
}

#[test]
fn navigation_capture_metadata_is_present_only_for_typed_requests() {
    let active =
        glory::ssr::render(anchor().href("/docs").label("Docs").on_navigate(|_| {})).unwrap();
    assert!(active.contains("data-gly-link-request"));
    let passive = glory::ssr::render(anchor().href("/docs").label("Docs")).unwrap();
    assert!(!passive.contains("data-gly-link-request"));
}

#[test]
fn navigation_reads_accepted_descriptor_before_pending_source_updates_and_explicit_patches() {
    let app = App::memory();
    let href = app.scope().cage("/accepted".to_owned());
    let last = Rc::new(RefCell::new(None));
    let output = last.clone();
    app.mount(
        anchor()
            .href(href)
            .label("Link")
            .on_navigate(move |request| *output.borrow_mut() = Some(request)),
    )
    .unwrap();
    let node = find(&app);
    href.revise(|v| *v = "/pending".into());
    app.dispatch(node, Event::Activate).unwrap();
    assert_eq!(last.borrow().as_ref().unwrap().href, "/accepted");
    app.scope()
        .patch(
            node,
            glory::render::Patch::Target(glory::render::Change::Set("/explicit".into())),
        )
        .unwrap();
    app.dispatch(node, Event::Activate).unwrap();
    assert_eq!(last.borrow().as_ref().unwrap().href, "/explicit");
}
