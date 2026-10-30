use glory::apace::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{
    Action, InputEdit, InputEditIntent, NodeId, Props, Query, QueryValue, SelectionDirection,
    TextSelection,
};
use sapid::{MaskPattern, mask_input};
use std::{cell::RefCell, rc::Rc};
fn query(app: &App, q: Query) -> QueryValue {
    let reply = Rc::new(RefCell::new(None));
    let out = reply.clone();
    app.request(q, move |v| *out.borrow_mut() = Some(v.unwrap()))
        .unwrap();
    app.pump().unwrap();
    reply.borrow_mut().take().unwrap()
}
fn edit(app: &App, id: NodeId, value: &str, after: usize, before: usize, intent: InputEditIntent) {
    let revision = {
        let tree = app.tree();
        let Props::Input(p) = tree.node(id).unwrap().descriptor.view().props else {
            panic!()
        };
        p.revision
    };
    app.dispatch(
        id,
        Event::InputEdit(InputEdit::Value {
            value: value.into(),
            revision,
            selection: TextSelection {
                start: after,
                end: after,
                direction: SelectionDirection::None,
            },
            before: Some(TextSelection {
                start: before,
                end: before,
                ..Default::default()
            }),
            intent,
            composing: false,
        }),
    )
    .unwrap();
}
#[test]
fn native_mask_four_themes_format_one_real_editor_and_apply_actual_selection_with_stable_identity()
{
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(700, 220, 1.).unwrap();
        let app = host.app();
        let r = app.scope().cage(None);
        let source = app.scope().cage("12".to_owned());
        host.mount(
            mask_input(MaskPattern::parse("##-##").unwrap())
                .theme(theme.clone())
                .name("Public masked reference")
                .value(source)
                .node_ref(r)
                .on_value_change(move |v| source.revise(|s| *s = v)),
        )
        .unwrap();
        app.settle(12).unwrap();
        let id = r.get().unwrap();
        let engine = host
            .renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap();
        let QueryValue::Bounds(rect) = query(app, Query::Bounds(id)) else {
            panic!()
        };
        assert!(rect.width >= 239. && rect.height >= 35.);
        app.scope().action(Action::Focus(id)).unwrap();
        app.flush().unwrap();
        app.scope()
            .set_input_selection(
                id,
                TextSelection {
                    start: 3,
                    end: 3,
                    ..Default::default()
                },
            )
            .unwrap();
        app.flush().unwrap();
        // The native InputEdit driver itself is covered in glory-apace's actual
        // key/IME tests; this exercises the widget protocol and physical action.
        edit(app, id, "12", 2, 3, InputEditIntent::DeleteBackward);
        app.settle(12).unwrap();
        assert_eq!(source.get(), "1");
        assert_eq!(
            query(app, Query::Selection(id)),
            QueryValue::Selection { start: 1, end: 1 }
        );
        host.renderer().inspect(|dom| {
            assert_eq!(
                dom.get_node(engine)
                    .unwrap()
                    .element_data()
                    .unwrap()
                    .text_input_data()
                    .unwrap()
                    .editor
                    .raw_text(),
                "1"
            )
        });
        edit(app, id, "12", 2, 1, InputEditIntent::Insert);
        app.settle(12).unwrap();
        assert_eq!(source.get(), "12");
        assert_eq!(
            query(app, Query::Selection(id)),
            QueryValue::Selection { start: 3, end: 3 }
        );
        host.renderer().inspect(|dom| {
            assert_eq!(
                dom.get_node(engine)
                    .unwrap()
                    .element_data()
                    .unwrap()
                    .text_input_data()
                    .unwrap()
                    .editor
                    .raw_text(),
                "12-"
            )
        });
        assert_eq!(
            host.renderer()
                .engine_id(id, glory::html::Part::Main)
                .unwrap(),
            engine
        );
        let hit = host.renderer().inspect(|dom| {
            dom.hit(
                (rect.x + rect.width / 2.) as f32,
                (rect.y + rect.height / 2.) as f32,
            )
            .unwrap()
            .node_id
        });
        assert_eq!(hit, engine);
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let offset = (((rect.y + rect.height / 2.).round() as usize)
            * raster.frame().unwrap().width as usize
            + (rect.x + rect.width - 10.).round() as usize)
            * 4;
        for (actual, expected) in raster.pixels()[offset..offset + 4].iter().zip([
            theme.input.0,
            theme.input.1,
            theme.input.2,
            theme.input.3,
        ]) {
            assert!(actual.abs_diff(expected) <= 2);
        }
        app.clear().unwrap();
        assert!(r.get().is_none());
        assert_eq!(app.stats().queries, 0);
    }
}
#[test]
fn native_mask_controlled_refusal_readonly_and_unstyled_dimensions_preserve_actual_input() {
    let host = WindowApp::new();
    let app = host.app();
    let source = app.scope().cage("1234".to_owned());
    let readonly = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    let r = app.scope().cage(None);
    host.mount(
        mask_input(MaskPattern::parse("##-##").unwrap())
            .value(source)
            .readonly(readonly)
            .enabled(enabled)
            .unstyled()
            .node_ref(r),
    )
    .unwrap();
    let id = r.get().unwrap();
    let engine = host
        .renderer()
        .engine_id(id, glory::html::Part::Main)
        .unwrap();
    let QueryValue::Bounds(rect) = query(app, Query::Bounds(id)) else {
        panic!()
    };
    assert!(rect.width >= 239. && rect.height >= 35.);
    edit(app, id, "12-3", 4, 5, InputEditIntent::DeleteBackward);
    app.settle(12).unwrap();
    assert_eq!(source.get(), "1234");
    host.renderer().inspect(|dom| {
        assert_eq!(
            dom.get_node(engine)
                .unwrap()
                .element_data()
                .unwrap()
                .text_input_data()
                .unwrap()
                .editor
                .raw_text(),
            "12-34"
        )
    });
    readonly.revise(|v| *v = true);
    app.flush().unwrap();
    app.scope().action(Action::Focus(id)).unwrap();
    app.flush().unwrap();
    assert_eq!(query(app, Query::Focus(id)), QueryValue::Focus(true));
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(app.scope().action(Action::Focus(id)).is_err());
    assert_eq!(
        host.renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap(),
        engine
    );
    app.clear().unwrap();
    assert_eq!(app.stats().queries, 0);
}
