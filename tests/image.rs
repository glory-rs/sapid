use glory::render::{Event, EventKind, ImageStatus, NodeId, Props};
use glory::style::{Length, ObjectFit, Position, Style};
use glory::ui::App;
use sapid::{Theme, background_image, image};
use std::{cell::RefCell, rc::Rc};
fn photo(app: &App, id: NodeId) -> glory::render::ImageProps {
    let tree = app.tree();
    let Props::Image(image) = tree.node(id).unwrap().descriptor.view().props else {
        panic!("image")
    };
    image.clone()
}
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            return id;
        }
        pending.extend(&node.children);
    }
    panic!("missing {class}")
}
#[test]
fn image_is_a_real_resource_node_with_explicit_alt_and_escaped_ssr() {
    let app = App::memory();
    app.mount(image("photo.png").alt("A photo")).unwrap();
    let root = app.tree().roots()[0];
    assert_eq!(
        (photo(&app, root).source, photo(&app, root).alternative),
        ("photo.png".into(), Some("A photo".into()))
    );
    let html = glory::ssr::render(image("a<&>.png").alt("A <&> photo")).unwrap();
    assert!(html.contains("<img"));
    assert!(html.contains("src=\"a&lt;&amp;&gt;.png\""));
    assert!(html.contains("alt=\"A &lt;&amp;&gt; photo\""));
    let html = glory::ssr::render(image("photo.png").decorative()).unwrap();
    assert!(html.contains("aria-hidden=\"true\""));
    assert!(html.contains("alt=\"\""));
}
#[test]
fn invalid_initial_sources_alternatives_and_fallbacks_fail_before_visible_creation() {
    for widget in [
        image("photo.png"),
        image(" ").alt("Photo"),
        image("x".repeat(4097)).alt("Photo"),
        image("photo.png").alt("Photo").fallback_src(""),
        image("photo.png").decorative().name("Invalid"),
        image("photo.png")
            .decorative()
            .focus(glory::render::FocusMode::TabStop),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert_eq!(app.tree().len(), 0);
    }
}
#[test]
fn fallback_is_one_real_request_attempt_and_typed_callbacks_precede_observers() {
    let app = App::memory();
    let source = app.scope().cage("primary.png".to_string());
    let log = Rc::new(RefCell::new(Vec::new()));
    let typed = log.clone();
    let generic = log.clone();
    app.mount(
        image(source)
            .alt("Photo")
            .fallback_src("fallback.png")
            .on_error(move |revision| typed.borrow_mut().push(format!("typed{revision}")))
            .on(EventKind::ImageStatus, move |_| {
                generic.borrow_mut().push("observer".into())
            }),
    )
    .unwrap();
    let id = app.tree().roots()[0];
    app.dispatch(
        id,
        Event::ImageStatus {
            revision: 1,
            status: ImageStatus::Failed,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(
        (photo(&app, id).source, photo(&app, id).revision),
        ("fallback.png".into(), 2)
    );
    assert_eq!(&*log.borrow(), &["typed1", "observer"]);
    app.dispatch(
        id,
        Event::ImageStatus {
            revision: 2,
            status: ImageStatus::Failed,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(photo(&app, id).revision, 2);
    assert_eq!(source.get(), "primary.png");
    source.revise(|value| *value = "next.png".into());
    app.flush().unwrap();
    assert_eq!(
        (photo(&app, id).source, photo(&app, id).revision),
        ("next.png".into(), 3)
    );
    assert_eq!(log.borrow().len(), 4);
}
#[test]
fn selecting_the_existing_fallback_as_primary_starts_a_new_revision() {
    let app = App::memory();
    let source = app.scope().cage("primary.png".to_string());
    app.mount(image(source).alt("Photo").fallback_src("fallback.png"))
        .unwrap();
    let id = app.tree().roots()[0];
    app.dispatch(
        id,
        Event::ImageStatus {
            revision: 1,
            status: ImageStatus::Failed,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(photo(&app, id).revision, 2);
    source.revise(|value| *value = "fallback.png".into());
    app.flush().unwrap();
    assert_eq!(
        (photo(&app, id).source, photo(&app, id).revision),
        ("fallback.png".into(), 3)
    );
}
#[test]
fn invalid_external_source_preserves_accepted_resource_and_recovers_silently() {
    let app = App::memory();
    let source = app.scope().cage("a.png".to_string());
    let called = Rc::new(RefCell::new(Vec::new()));
    let load = called.clone();
    let error = called.clone();
    app.mount(
        image(source)
            .alt("Photo")
            .on_load(move |revision| load.borrow_mut().push(revision))
            .on_error(move |revision| error.borrow_mut().push(revision)),
    )
    .unwrap();
    let id = app.tree().roots()[0];
    source.revise(|value| *value = " ".into());
    assert!(app.flush().is_err());
    assert_eq!(photo(&app, id).source, "a.png");
    assert_eq!(source.get(), " ");
    source.revise(|value| *value = "b.png".into());
    app.flush().unwrap();
    assert_eq!(photo(&app, id).source, "b.png");
    assert!(called.borrow().is_empty());
    assert!(
        !app.dispatch(
            id,
            Event::ImageStatus {
                revision: 1,
                status: ImageStatus::Loaded
            }
        )
        .unwrap()
    );
    assert!(called.borrow().is_empty());
    app.dispatch(
        id,
        Event::ImageStatus {
            revision: 2,
            status: ImageStatus::Loaded,
        },
    )
    .unwrap();
    assert_eq!(&*called.borrow(), &[2]);
}
#[test]
fn four_themes_custom_overrides_and_unstyled_keep_explicit_fit_and_dimensions() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let app = App::memory();
        app.mount(
            image("a.png")
                .alt("Photo")
                .theme(theme.clone())
                .fit(ObjectFit::Contain)
                .width(Length::Px(80.0))
                .height(Length::Px(40.0)),
        )
        .unwrap();
        let tree = app.tree();
        let style = tree.node(tree.roots()[0]).unwrap().descriptor.view().style;
        assert_eq!(style.background, Some(theme.surface));
        assert_eq!(style.object_fit, Some(ObjectFit::Contain));
        assert_eq!(style.width, Some(Length::Px(80.0)));
    }
    let app = App::memory();
    app.mount(
        image("a.png")
            .alt("Photo")
            .unstyled()
            .fit(ObjectFit::Cover)
            .width(Length::Px(80.0))
            .style(Style::new().object_fit(ObjectFit::None)),
    )
    .unwrap();
    let tree = app.tree();
    assert_eq!(
        *tree.node(tree.roots()[0]).unwrap().descriptor.view().style,
        Style::new()
            .width(Length::Px(80.0))
            .object_fit(ObjectFit::None)
    );
}
#[test]
fn background_is_a_decorative_real_image_behind_an_independent_content_slot() {
    let app = App::memory();
    app.mount(
        background_image("photo.png")
            .width(Length::Px(240.0))
            .height(Length::Px(100.0))
            .fill(sapid::button().label("Read article")),
    )
    .unwrap();
    let tree = app.tree();
    let root = tree.roots()[0];
    assert_eq!(tree.node(root).unwrap().descriptor.view().role, None);
    assert_eq!(
        tree.node(root).unwrap().descriptor.view().style.position,
        Some(Position::Relative)
    );
    let media = find(&app, "sapid-background-image-media");
    assert_eq!(photo(&app, media).alternative, None);
    assert_eq!(
        tree.node(media).unwrap().descriptor.view().style.position,
        Some(Position::Absolute)
    );
    let content = find(&app, "sapid-background-image-content");
    assert_eq!(
        tree.node(content).unwrap().descriptor.view().style.position,
        Some(Position::Relative)
    );
    let button = tree.node(content).unwrap().children[0];
    assert!(matches!(
        tree.node(button).unwrap().descriptor.view().props,
        Props::Button(_)
    ));
}
#[test]
fn source_drivers_node_refs_and_captures_retire_with_owner() {
    let app = App::memory();
    let source = app.scope().cage("a.png".to_string());
    let node = app.scope().cage(None);
    let baseline = app.runtime_stats();
    let mounted = app
        .scope()
        .mount_with(|| image(source).alt("Photo").node_ref(node))
        .unwrap();
    app.flush().unwrap();
    assert!(node.get().is_some());
    drop(mounted);
    app.flush().unwrap();
    assert_eq!(node.get(), None);
    assert_eq!(app.runtime_stats().edges, baseline.edges);
    source.revise(|value| *value = "b.png".into());
    app.flush().unwrap();
    assert_eq!(app.stats().nodes, 0);
}
