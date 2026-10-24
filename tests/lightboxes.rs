use glory::{
    prelude::*,
    render::{DismissReason, Event, Navigation, NodeId, Props, Semantics},
};
use sapid::{
    LightboxImage, LightboxView, LightboxZoom, MAX_LIGHTBOX_IMAGES, lightbox, lightbox_image,
};
use std::{cell::RefCell, rc::Rc};
fn images() -> Vec<LightboxImage> {
    vec![
        lightbox_image("one", "https://example.invalid/one.png", "First image")
            .caption("First caption"),
        lightbox_image("two", "https://example.invalid/two.png", "Second image")
            .caption("Second caption"),
    ]
}
fn all(app: &App) -> Vec<NodeId> {
    let t = app.tree();
    let mut pending = t.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = pending.pop() {
        pending.extend(&t.node(id).unwrap().children);
        out.push(id);
    }
    out
}
fn class(app: &App, name: &str) -> Vec<NodeId> {
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
                .any(|c| c == name)
        })
        .collect()
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
        .unwrap_or_else(|| panic!("missing {name}"))
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
fn metadata(app: &App, id: NodeId) -> Semantics {
    *app.tree()
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .clone()
        .unwrap()
}
#[test]
fn closed_gallery_allocates_no_images_and_open_uses_one_real_managed_modal() {
    let app = App::memory();
    app.mount(lightbox("gallery").images(images())).unwrap();
    assert!(class(&app, "sapid-lightbox-image").is_empty());
    let trigger = named(&app, "Open image gallery");
    let panel = class(&app, "sapid-lightbox")[0];
    assert!(metadata(&app, panel).hidden);
    app.dispatch(trigger, Event::Activate).unwrap();
    assert!(!metadata(&app, panel).hidden);
    assert!(metadata(&app, panel).layer.unwrap().modal);
    assert_eq!(class(&app, "sapid-lightbox-image").len(), 1);
    assert!(text(&app).contains("First caption"));
    app.dispatch(panel, Event::Dismiss(DismissReason::Escape))
        .unwrap();
    assert!(class(&app, "sapid-lightbox-image").is_empty());
    assert_eq!(panel, class(&app, "sapid-lightbox")[0]);
}
#[test]
fn navigation_zoom_and_fit_keep_one_image_identity_and_exact_alt() {
    let app = App::memory();
    app.mount(lightbox("gallery").images(images()).default_open(true))
        .unwrap();
    let id = class(&app, "sapid-lightbox-image")[0];
    app.dispatch(named(&app, "Next image"), Event::Activate)
        .unwrap();
    assert_eq!(class(&app, "sapid-lightbox-image"), vec![id]);
    assert!(text(&app).contains("Second caption"));
    let viewport = class(&app, "sapid-lightbox-viewport")[0];
    app.dispatch(
        viewport,
        Event::Navigate {
            from: viewport,
            command: Navigation::First,
        },
    )
    .unwrap();
    assert!(text(&app).contains("First caption"));
    for _ in 0..3 {
        app.dispatch(named(&app, "Zoom in"), Event::Activate)
            .unwrap();
    }
    assert!(text(&app).contains("400%"));
    app.dispatch(named(&app, "Fit image"), Event::Activate)
        .unwrap();
    assert!(text(&app).contains("100%"));
    assert!(
        matches!(app.tree().node(id).unwrap().descriptor.view().props,Props::Image(p)if p.alternative.as_deref()==Some("First image"))
    );
}
#[test]
fn controlled_refusal_keeps_open_and_view_while_emitting_typed_proposals() {
    let app = App::memory();
    let changes = Rc::new(RefCell::new(vec![]));
    let out = changes.clone();
    let opens = Rc::new(RefCell::new(vec![]));
    let open_out = opens.clone();
    app.mount(
        lightbox("gallery")
            .images(images())
            .open(true)
            .view(LightboxView::default())
            .on_view_change(move |v| out.borrow_mut().push(v))
            .on_open_change(move |v| open_out.borrow_mut().push(v)),
    )
    .unwrap();
    app.dispatch(named(&app, "Next image"), Event::Activate)
        .unwrap();
    assert_eq!(changes.borrow()[0].current.as_deref(), Some("two"));
    assert!(text(&app).contains("First caption"));
    app.dispatch(named(&app, "Zoom in"), Event::Activate)
        .unwrap();
    assert_eq!(changes.borrow()[1].zoom, LightboxZoom::Two);
    assert!(text(&app).contains("100%"));
    let panel = class(&app, "sapid-lightbox")[0];
    app.dispatch(panel, Event::Dismiss(DismissReason::Escape))
        .unwrap();
    assert_eq!(*opens.borrow(), [false]);
    assert!(!metadata(&app, panel).hidden);
}
#[test]
fn paired_snapshot_admission_rejects_invalid_updates_before_replacing_the_current_image() {
    let app = App::memory();
    let data = app.scope().cage(images());
    let view = app.scope().cage(LightboxView::default());
    app.mount(
        lightbox("gallery")
            .images(data)
            .view(view)
            .default_open(true),
    )
    .unwrap();
    let id = class(&app, "sapid-lightbox-image")[0];
    view.revise(|v| v.current = Some("missing".into()));
    assert!(app.flush().is_err());
    assert_eq!(class(&app, "sapid-lightbox-image"), vec![id]);
    assert!(text(&app).contains("First caption"));
    view.revise(|v| v.current = None);
    data.revise(|d| d[0].alternative.clear());
    assert!(app.flush().is_err());
    assert!(text(&app).contains("First caption"));
    data.revise(|d| *d = vec![]);
    app.flush().unwrap();
    assert!(class(&app, "sapid-lightbox-image").is_empty());
    assert!(text(&app).contains("No images"));
}
#[test]
fn data_changes_are_silent_and_repeated_navigation_has_constant_live_nodes() {
    let app = App::memory();
    let data = app.scope().cage(images());
    let changes = Rc::new(RefCell::new(vec![]));
    let out = changes.clone();
    app.mount(
        lightbox("gallery")
            .images(data)
            .default_open(true)
            .on_view_change(move |v| out.borrow_mut().push(v)),
    )
    .unwrap();
    let count = app.stats().nodes;
    data.revise(|d| d[0].caption = "Updated caption".into());
    app.flush().unwrap();
    assert!(changes.borrow().is_empty());
    for _ in 0..40 {
        app.dispatch(named(&app, "Next image"), Event::Activate)
            .unwrap();
        app.dispatch(named(&app, "Previous image"), Event::Activate)
            .unwrap();
        assert_eq!(app.stats().nodes, count);
    }
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
}
#[test]
fn bounded_metadata_keys_alternatives_and_current_selection_fail_before_mount() {
    for data in [
        vec![lightbox_image("a", "x", "")],
        vec![lightbox_image("a", "x", "A"), lightbox_image("a", "y", "B")],
        (0..=MAX_LIGHTBOX_IMAGES)
            .map(|i| lightbox_image(i.to_string(), "x", "A"))
            .collect(),
    ] {
        let app = App::memory();
        assert!(
            app.mount(lightbox("bad").images(data).default_open(true))
                .is_err()
        );
        assert_eq!(app.stats().nodes, 0);
    }
    let app = App::memory();
    assert!(
        app.mount(lightbox("bad").images(images()).default_view(LightboxView {
            current: Some("unknown".into()),
            zoom: LightboxZoom::Fit
        }))
        .is_err()
    );
}
