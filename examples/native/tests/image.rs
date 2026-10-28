#[path = "../../web/src/assets.rs"]
mod assets;
#[path = "../../web/src/image_form.rs"]
mod image_form;
use glory::blitz::assets::{Bytes, NetHandler, NetProvider, Request};
use glory::blitz::{Config, shell::WindowApp};
use glory::render::{Event, NodeId, Props, Query, QueryValue, Rect};
use glory::style::Display;
use glory::ui::App;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};
struct Job {
    url: String,
    handler: Box<dyn NetHandler>,
}
#[derive(Default)]
struct Provider(Mutex<Vec<Job>>);
impl NetProvider for Provider {
    fn fetch(&self, _: usize, request: Request, handler: Box<dyn NetHandler>) {
        self.0.lock().unwrap().push(Job {
            url: request.url.to_string(),
            handler,
        });
    }
}
fn find(app: &App, predicate: impl Fn(glory::render::DescriptorView<'_>) -> bool) -> NodeId {
    let tree = app.tree();
    let mut nodes = tree.roots().to_vec();
    while let Some(id) = nodes.pop() {
        let node = tree.node(id).unwrap();
        if predicate(node.descriptor.view()) {
            return id;
        }
        nodes.extend(&node.children);
    }
    panic!("missing node")
}
fn class(app: &App, name: &str) -> NodeId {
    find(app, |node| node.classes.iter().any(|value| value == name))
}
fn content(app: &App, name: &str) -> String {
    let tree = app.tree();
    let Props::Text(value) = tree.node(class(app, name)).unwrap().descriptor.view().props
    else {
        panic!()
    };
    value.content.clone()
}
fn button(app: &App, label: &str) -> NodeId {
    find(
        app,
        |node| matches!(node.props,Props::Button(value) if value.label==label),
    )
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let output = Rc::new(RefCell::new(None));
    let reply = output.clone();
    app.request(Query::Bounds(id), move |value| {
        *reply.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = output.borrow_mut().take().unwrap() else {
        panic!()
    };
    rect
}
fn settle(host: &WindowApp, provider: &Provider) {
    for _ in 0..12 {
        let jobs = std::mem::take(&mut *provider.0.lock().unwrap());
        for job in jobs {
            let bytes: &[u8] = if job.url.ends_with("image-pattern.png") {
                include_bytes!("../../web/assets/image-pattern.png")
            } else if job.url.ends_with("image-fallback.png") {
                include_bytes!("../../web/assets/image-fallback.png")
            } else {
                include_bytes!("../../web/assets/image-invalid.png")
            };
            job.handler.bytes(job.url, Bytes::copy_from_slice(bytes));
        }
        host.app().pump().unwrap();
    }
    host.app().settle(16).unwrap();
}
#[test]
fn four_theme_native_images_load_fallback_avatar_text_recovery_and_background_content() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let provider = Arc::new(Provider::default());
        let host = WindowApp::with_config(Config {
            base_url: "https://sapid.test/".parse().unwrap(),
            provider: Some(provider.clone()),
            ..Default::default()
        });
        host.renderer().set_viewport(800, 800, 1.0).unwrap();
        host.mount(image_form::image_form(theme)).unwrap();
        settle(&host, &provider);
        let app = host.app();
        assert_eq!(content(app, "image-status"), "Loaded 1");
        assert_eq!(content(app, "image-fallback-status"), "Fallback loaded 2");
        let image = class(app, "image-primary");
        let physical = host
            .renderer()
            .engine_id(image, glory::html::Part::Main)
            .unwrap();
        let rect = bounds(app, image);
        assert!((rect.width - 200.0).abs() < 0.1 && (rect.height - 120.0).abs() < 0.1);
        let avatar = class(app, "avatar-resource");
        let children = app.tree().node(avatar).unwrap().children.to_vec();
        let avatar_image = children[0];
        let text = children[1];
        assert_eq!(
            app.tree()
                .node(text)
                .unwrap()
                .descriptor
                .view()
                .style
                .display,
            Some(Display::None)
        );
        app.dispatch(button(app, "Break image"), Event::Activate)
            .unwrap();
        settle(&host, &provider);
        assert_eq!(content(app, "image-status"), "Failed 2");
        assert_eq!(
            app.tree()
                .node(text)
                .unwrap()
                .descriptor
                .view()
                .style
                .display,
            Some(Display::Block)
        );
        app.dispatch(button(app, "Restore image"), Event::Activate)
            .unwrap();
        settle(&host, &provider);
        assert_eq!(content(app, "image-status"), "Loaded 3");
        assert_eq!(
            app.tree()
                .node(text)
                .unwrap()
                .descriptor
                .view()
                .style
                .display,
            Some(Display::None)
        );
        assert_eq!(
            host.renderer()
                .engine_id(image, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        assert_eq!(app.tree().node(avatar).unwrap().children[0], avatar_image);
        let background = bounds(app, class(app, "image-background"));
        assert!(
            (background.width - 300.0).abs() < 0.1 && (background.height - 140.0).abs() < 0.1
        );
        app.dispatch(button(app, "Background action"), Event::Activate)
            .unwrap();
        assert_eq!(content(app, "image-status"), "Background action pressed");
        assert_eq!(host.renderer().resource_stats().pending_images, 0);
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
    }
}
