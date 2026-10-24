use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, Props, Query, QueryValue, SelectionDirection, TextSelection};
use sapid::{portal, portal_context, portal_target};
use std::{cell::RefCell, rc::Rc};
fn read(app: &App, query: Query) -> QueryValue {
    let reply = Rc::new(RefCell::new(None));
    let result = reply.clone();
    app.request(query, move |value| {
        *result.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    reply.borrow_mut().take().unwrap()
}
#[test]
fn native_portal_moves_real_engine_ranges_without_replacing_editor_or_selection_and_recovers_retired_target()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(800, 650, 1.).unwrap();
        let app = host.app();
        let context = portal_context(app.scope(), "native").unwrap();
        let target_a = context.clone();
        let chosen = app.scope().cage("a".to_string());
        let mounted = app.scope().cage(true);
        let reference = app.scope().cage(None);
        let current = app.scope().cage("Keep selected draft".to_owned());
        let input_theme = theme.clone();
        host.mount(
            view().fill((
                view()
                    .style(Style::new().width(Length::Px(350.)))
                    .fill(when(mounted, move || {
                        portal_target(&target_a, "a").fill(text().content("Destination A"))
                    })),
                view().style(Style::new().height(Length::Px(100.))),
                view()
                    .style(Style::new().width(Length::Px(350.)))
                    .fill(portal_target(&context, "b").fill(text().content("Destination B"))),
                portal(&context, chosen).content(move || {
                    sapid::text_field()
                        .theme(input_theme.clone())
                        .name("Portaled native draft")
                        .value(current)
                        .node_ref(reference)
                }),
            )),
        )
        .unwrap();
        app.settle(16).unwrap();
        let editor = reference.get().unwrap();
        let physical = host
            .renderer()
            .engine_id(editor, glory::html::Part::Main)
            .unwrap();
        let revision = {
            let tree = app.tree();
            let Props::Input(input) = tree.node(editor).unwrap().descriptor.view().props else {
                panic!("Input")
            };
            input.revision
        };
        app.scope().action(Action::Focus(editor)).unwrap();
        app.scope()
            .action(Action::SelectText {
                node: editor,
                revision,
                selection: TextSelection {
                    start: 3,
                    end: 8,
                    direction: SelectionDirection::Forward,
                },
            })
            .unwrap();
        app.flush().unwrap();
        let QueryValue::Bounds(first) = read(app, Query::Bounds(editor)) else {
            panic!("Bounds")
        };
        chosen.revise(|value| *value = "b".to_string());
        app.flush().unwrap();
        app.settle(16).unwrap();
        let QueryValue::Bounds(second) = read(app, Query::Bounds(editor)) else {
            panic!("Bounds")
        };
        assert!(
            second.y - first.y > 80.,
            "real parent layout moved: {first:?} -> {second:?}"
        );
        assert_eq!(
            host.renderer()
                .engine_id(editor, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        assert_eq!(
            read(app, Query::Selection(editor)),
            QueryValue::Selection { start: 3, end: 8 }
        );
        assert_eq!(read(app, Query::Focus(editor)), QueryValue::Focus(true));
        chosen.revise(|value| *value = "a".to_string());
        app.flush().unwrap();
        mounted.revise(|value| *value = false);
        app.flush().unwrap();
        assert!(app.tree().node(editor).is_ok());
        assert!(!app.scope().is_active(editor).unwrap());
        mounted.revise(|value| *value = true);
        app.flush().unwrap();
        app.settle(16).unwrap();
        assert!(app.scope().is_active(editor).unwrap());
        assert_eq!(
            host.renderer()
                .engine_id(editor, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        assert_eq!(
            read(app, Query::Selection(editor)),
            QueryValue::Selection { start: 3, end: 8 }
        );
        assert_eq!(current.get(), "Keep selected draft");
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        assert!(raster.pixels().as_chunks::<4>().0.iter().any(|p| p[3] != 0));
        app.clear().unwrap();
        assert_eq!(context.sources(), 0);
        assert_eq!(context.targets(), 0);
        assert_eq!(app.stats().disposal_boundaries, 0);
        assert_eq!(app.stats().queries, 0);
    }
}
