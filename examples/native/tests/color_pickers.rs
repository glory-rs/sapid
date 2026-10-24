use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, DragPhase, Event, NodeId, Query, QueryValue, RangeCommand, Rect};
use glory::style::Color;
use std::{cell::RefCell, rc::Rc};
fn bounds(app: &App, id: NodeId) -> Rect {
    let out = Rc::new(RefCell::new(None));
    let seen = out.clone();
    app.request(Query::Bounds(id), move |value| {
        *seen.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = out.borrow_mut().take().unwrap() else {
        panic!("bounds")
    };
    rect
}
fn nodes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut out = Vec::new();
    while let Some(id) = todo.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            out.push(id);
        }
        todo.extend(&node.children);
    }
    out
}
fn pixel(raster: &Raster, x: f64, y: f64) -> [u8; 4] {
    let offset = (y as usize * raster.frame().unwrap().width as usize + x as usize) * 4;
    raster.pixels()[offset..offset + 4].try_into().unwrap()
}
#[test]
fn native_picker_four_themes_has_real_plane_pixels_two_axis_mapping_focus_and_stable_identity()
{
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let source = app.scope().cage(sapid::Hsva {
            hue: 120.,
            saturation: 0.5,
            value: 0.5,
            alpha: 0.4,
        });
        let editor = app.scope().cage(None);
        host.mount(
            sapid::stack().fill((
                sapid::text_field()
                    .theme(theme.clone())
                    .name("Preserved notes")
                    .node_ref(editor)
                    .default_value("caret"),
                sapid::color_picker("native")
                    .theme(theme)
                    .value(source)
                    .on_value_change(move |next| source.revise(|value| *value = next)),
            )),
        )
        .unwrap();
        app.settle(16).unwrap();
        let plane = nodes(app, "sapid-color-plane")[0];
        let identity = host
            .renderer()
            .engine_id(plane, glory::html::Part::Main)
            .unwrap();
        let rect = bounds(app, plane);
        assert_eq!((rect.width, rect.height), (240., 240.));
        let indicator = bounds(app, nodes(app, "sapid-color-plane-indicator")[0]);
        assert!((indicator.x + indicator.width / 2. - (rect.x + rect.width * 0.5)).abs() < 1.);
        assert!(
            (indicator.y + indicator.height / 2. - (rect.y + rect.height * 0.5)).abs() < 1.
        );
        let cells = nodes(app, "sapid-color-plane-cell");
        assert_eq!(cells.len(), 256);
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let cell = *cells
            .iter()
            .find(|id| {
                let r = bounds(app, **id);
                r.x > rect.x + 200. && r.y < rect.y + 2.
            })
            .unwrap();
        let cell_rect = bounds(app, cell);
        let Some(Color(r, g, b, a)) = app
            .tree()
            .node(cell)
            .unwrap()
            .descriptor
            .view()
            .style
            .background
        else {
            panic!("color")
        };
        let actual = pixel(
            &raster,
            cell_rect.x + cell_rect.width / 2.,
            cell_rect.y + cell_rect.height / 2.,
        );
        for (actual, want) in actual.into_iter().zip([r, g, b, a]) {
            assert!(actual.abs_diff(want) <= 2);
        }
        let editor = editor.get().unwrap();
        app.scope().action(Action::Focus(editor)).unwrap();
        app.flush().unwrap();
        app.dispatch(
            plane,
            Event::Drag {
                from: plane,
                phase: DragPhase::Start,
                x: rect.x + rect.width * 0.25,
                y: rect.y + rect.height * 0.75,
            },
        )
        .unwrap();
        app.dispatch(
            plane,
            Event::Drag {
                from: plane,
                phase: DragPhase::Drop,
                x: rect.x + rect.width + 100.,
                y: rect.y - 100.,
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        assert_eq!(source.get().saturation, 1.);
        assert_eq!(source.get().value, 1.);
        assert_eq!(source.get().alpha, 0.4);
        assert_eq!(app.stats().queries, 0);
        let hue = nodes(app, "sapid-hue-slider")[0];
        app.dispatch(
            hue,
            Event::Range {
                from: hue,
                command: RangeCommand::Set(240.),
            },
        )
        .unwrap();
        app.settle(16).unwrap();
        raster.draw(host.renderer()).unwrap();
        let now = app
            .tree()
            .node(cell)
            .unwrap()
            .descriptor
            .view()
            .style
            .background
            .unwrap();
        assert_ne!(now, Color(r, g, b, a));
        assert_eq!(
            host.renderer()
                .engine_id(plane, glory::html::Part::Main)
                .unwrap(),
            identity
        );
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(app.stats().queries, 0);
        host.mount(sapid::color_picker("small").plane_size(128.))
            .unwrap();
        app.settle(16).unwrap();
        for id in nodes(app, "sapid-slider") {
            assert!((bounds(app, id).width - 128.).abs() < 1.);
        }
        app.clear().unwrap();
    }
}
#[test]
fn native_color_input_preserves_hex_editor_identity_and_error_metadata() {
    let host = WindowApp::new();
    let app = host.app();
    let value = app.scope().cage("#bad?".to_owned());
    host.mount(
        sapid::field("color")
            .label("Accent color")
            .required(true)
            .form_name("accent")
            .control(move |field| {
                sapid::color_input("hex")
                    .show_picker(false)
                    .field(field)
                    .value(value)
            }),
    )
    .unwrap();
    app.settle(16).unwrap();
    let id = nodes(app, "sapid-color-editor")[0];
    let identity = host
        .renderer()
        .engine_id(id, glory::html::Part::Main)
        .unwrap();
    assert!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .invalid
    );
    assert!(bounds(app, id).height > 20.);
    value.revise(|value| *value = "#1234".into());
    app.settle(16).unwrap();
    assert!(
        !app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .invalid
    );
    assert_eq!(
        host.renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap(),
        identity
    );
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
}
#[test]
fn native_default_color_popup_is_explicitly_unsupported_without_silent_fallback() {
    let host = WindowApp::new();
    let error = host.mount(sapid::color_input("popup")).unwrap_err();
    assert!(matches!(
        error,
        glory::ui::Error::Render(glory::render::Error::Unsupported("native top layers"))
    ));
}
