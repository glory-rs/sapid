use glory::apace::{Config, Raster, shell::WindowApp};
use glory::render::{Action, Event, NodeId, Query, QueryValue};
use glory::ui::App;
use std::{cell::RefCell, rc::Rc};
fn query(app: &App, q: Query) -> QueryValue {
    let value = Rc::new(RefCell::new(None));
    let out = value.clone();
    app.request(q, move |r| *out.borrow_mut() = Some(r.unwrap()))
        .unwrap();
    app.pump().unwrap();
    value.borrow_mut().take().unwrap()
}
#[test]
fn native_four_theme_copy_is_real_button_with_explicit_unavailable_and_preserved_editor() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::with_config(Config::default());
        host.renderer().set_viewport(320, 220, 1.).unwrap();
        let app = host.app();
        let node = app.scope().cage(None::<NodeId>);
        let editor = app.scope().cage(None);
        let status = app.scope().cage(sapid::CopyState::Idle);
        host.mount(
            glory::ui::col().fill((
                sapid::copy_button()
                    .theme(theme)
                    .node_ref(node)
                    .state_ref(status)
                    .source("Native private source"),
                glory::ui::input().node_ref(editor).value("Native draft"),
            )),
        )
        .unwrap();
        let id = node.get().unwrap();
        let engine = host
            .renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap();
        let QueryValue::Bounds(bounds) = query(app, Query::Bounds(id)) else {
            panic!()
        };
        assert!(bounds.width > 40. && bounds.height >= 24.);
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let before = raster.pixels().to_vec();
        app.scope()
            .action(Action::Focus(editor.get().unwrap()))
            .unwrap();
        app.flush().unwrap();
        assert!(app.scope().copy_text(id).is_err());
        app.dispatch(id, Event::Activate).unwrap();
        app.flush().unwrap();
        assert_eq!(status.get(), sapid::CopyState::Unavailable);
        assert_eq!(
            query(app, Query::Focus(editor.get().unwrap())),
            QueryValue::Focus(true)
        );
        assert_eq!(
            host.renderer()
                .engine_id(id, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        raster.draw(host.renderer()).unwrap();
        assert_ne!(before, raster.pixels());
        app.clear().unwrap();
        assert_eq!(app.runtime_stats().tasks, 0);
    }
}
