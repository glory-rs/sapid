#[path = "../../web/lightbox_form.rs"]
mod fixture;
use glory::{
    blitz::{
        Config, Raster,
        assets::{Bytes, NetHandler, NetProvider, Request},
        shell::WindowApp,
    },
    prelude::*,
    render::{Action, DismissReason, Navigation, NodeId, Props, Query, QueryValue, Rect},
};
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
fn button(app: &App, name: &str) -> NodeId {
    all(app).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(p)if p.label==name)).unwrap()
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let out = Rc::new(RefCell::new(None));
    let r = out.clone();
    app.request(Query::Bounds(id), move |v| {
        *r.borrow_mut() = Some(v.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(v) = out.borrow_mut().take().unwrap() else {
        panic!()
    };
    v
}
#[test]
fn four_theme_native_lightbox_has_real_modal_image_pixels_zoom_navigation_failure_and_cleanup()
{
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
        host.renderer().set_viewport(800, 900, 1.).unwrap();
        host.mount(fixture::lightbox_form(theme)).unwrap();
        let app = host.app();
        app.settle(16).unwrap();
        assert!(provider.0.lock().unwrap().is_empty());
        assert!(class(app, "sapid-lightbox-image").is_empty());
        app.dispatch(button(app, "Open image gallery"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        let image = class(app, "sapid-lightbox-image")[0];
        let physical = host
            .renderer()
            .engine_id(image, glory::html::Part::Main)
            .unwrap();
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let before = raster.pixels().to_vec();
        settle(&host, &provider);
        raster.draw(host.renderer()).unwrap();
        assert!(
            before
                .iter()
                .zip(raster.pixels())
                .filter(|(a, b)| a != b)
                .count()
                > 1000
        );
        let viewport = class(app, "sapid-lightbox-viewport")[0];
        let r = bounds(app, viewport);
        assert!(r.width > 400. && (r.height - 320.).abs() < 0.5, "{r:?}");
        app.scope().action(Action::Focus(viewport)).unwrap();
        app.settle(16).unwrap();
        app.dispatch(
            viewport,
            Event::Navigate {
                from: viewport,
                command: Navigation::Next,
            },
        )
        .unwrap();
        settle(&host, &provider);
        assert_eq!(
            host.renderer().engine_id(image, glory::html::Part::Main),
            Ok(physical)
        );
        assert!(
            matches!(app.tree().node(image).unwrap().descriptor.view().props,Props::Image(p)if p.alternative.as_deref()==Some("Fallback pattern"))
        );
        app.dispatch(button(app, "Zoom in"), Event::Activate)
            .unwrap();
        app.settle(16).unwrap();
        let zoom = bounds(app, image);
        assert!(
            zoom.width > r.width * 1.8 && zoom.height > 600.,
            "{zoom:?} {r:?}"
        );
        app.dispatch(button(app, "Fit image"), Event::Activate)
            .unwrap();
        app.dispatch(button(app, "Next image"), Event::Activate)
            .unwrap();
        settle(&host, &provider);
        assert!(all(app).iter().any(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Text(p)if p.content=="Image could not be loaded")));
        let panel = class(app, "sapid-lightbox")[0];
        app.dispatch(panel, Event::Dismiss(DismissReason::Escape))
            .unwrap();
        app.settle(16).unwrap();
        assert!(class(app, "sapid-lightbox-image").is_empty());
        assert_eq!(host.renderer().resource_stats().pending_images, 0);
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(host.renderer().stats().nodes, 0);
    }
}
