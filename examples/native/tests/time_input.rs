use glory::apace::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{
    Action, InputEdit, InputEditIntent, NodeId, Props, Query, QueryValue, SelectionDirection,
    TextSelection,
};
use sapid::prelude::*;
use std::{cell::RefCell, rc::Rc};
fn query(app: &App, q: Query) -> QueryValue {
    let reply = Rc::new(RefCell::new(None));
    let out = reply.clone();
    app.request(q, move |v| *out.borrow_mut() = Some(v.unwrap()))
        .unwrap();
    app.pump().unwrap();
    reply.borrow_mut().take().unwrap()
}
fn edit(
    app: &App,
    id: NodeId,
    value: &str,
    after: usize,
    before: usize,
    intent: InputEditIntent,
) {
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

fn time(v: &str) -> LocalTime {
    LocalTime::parse_iso(v).unwrap()
}
#[test]
fn native_time_input_four_themes_preserve_drafts_actual_editor_caret_geometry_and_pixels() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(700, 260, 1.).unwrap();
        let app = host.app();
        let r = app.scope().cage(None);
        let value = app.scope().cage(Some(time("10:30")));
        host.mount(
            time_input("native-draft")
                .theme(theme.clone())
                .name("Native wall-clock draft")
                .node_ref(r)
                .value(value)
                .on_value_change(move |v| value.revise(|old| *old = v)),
        )
        .unwrap();
        app.settle(16).unwrap();
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
        app.settle(16).unwrap();
        edit(app, id, "12:", 3, 5, InputEditIntent::Replace);
        app.settle(16).unwrap();
        assert_eq!(value.get(), Some(time("10:30")));
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
                "12:"
            )
        });
        edit(app, id, "24:00", 5, 3, InputEditIntent::Replace);
        app.settle(16).unwrap();
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
        edit(app, id, "23:59", 5, 5, InputEditIntent::Replace);
        app.settle(16).unwrap();
        assert_eq!(value.get(), Some(time("23:59")));
        app.scope()
            .set_input_selection(
                id,
                TextSelection {
                    start: 2,
                    end: 4,
                    direction: SelectionDirection::Backward,
                },
            )
            .unwrap();
        app.settle(16).unwrap();
        assert_eq!(
            query(app, Query::Selection(id)),
            QueryValue::Selection { start: 2, end: 4 }
        );
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
        let frame = raster.frame().unwrap();
        let at = ((rect.y as usize + rect.height as usize / 2) * frame.width as usize
            + rect.x as usize
            + rect.width as usize
            - 10)
            * 4;
        for (a, b) in raster.pixels()[at..at + 4].iter().zip([
            theme.input.0,
            theme.input.1,
            theme.input.2,
            theme.input.3,
        ]) {
            assert!(a.abs_diff(b) <= 2);
        }
        app.clear().unwrap();
        assert!(r.get().is_none());
        assert_eq!(app.stats().queries, 0);
    }
}

#[test]
fn native_twelve_hour_chinese_seconds_readonly_disabled_and_unstyled_keep_real_editor() {
    let host = WindowApp::new();
    host.renderer().set_viewport(700, 250, 1.).unwrap();
    let app = host.app();
    let r = app.scope().cage(None);
    let value = app.scope().cage(Some(time("00:00:01")));
    let ro = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    host.mount(
        time_input("native-twelve")
            .unstyled()
            .size(Size::ExtraSmall)
            .node_ref(r)
            .format(TimeFormat::new(
                HourCycle::H12,
                TimePrecision::Second,
                DateLocale::ZhCn,
            ))
            .value(value)
            .readonly(ro)
            .enabled(enabled)
            .on_value_change(move |v| value.revise(|old| *old = v)),
    )
    .unwrap();
    app.settle(16).unwrap();
    let id = r.get().unwrap();
    let engine = host
        .renderer()
        .engine_id(id, glory::html::Part::Main)
        .unwrap();
    let QueryValue::Bounds(rect) = query(app, Query::Bounds(id)) else {
        panic!()
    };
    assert!(rect.width >= 239. && rect.height >= 23.);
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
            "12:00:01 上午"
        )
    });
    edit(app, id, "01:02:03 下午", 11, 11, InputEditIntent::Replace);
    app.settle(16).unwrap();
    assert_eq!(value.get(), Some(time("13:02:03")));
    app.scope()
        .set_input_selection(
            id,
            TextSelection {
                start: 9,
                end: 11,
                direction: SelectionDirection::Backward,
            },
        )
        .unwrap();
    app.flush().unwrap();
    assert_eq!(
        query(app, Query::Selection(id)),
        QueryValue::Selection { start: 9, end: 11 }
    );
    ro.revise(|v| *v = true);
    app.settle(16).unwrap();
    edit(app, id, "02:03:04 下午", 11, 11, InputEditIntent::Replace);
    app.settle(16).unwrap();
    assert_eq!(value.get(), Some(time("13:02:03")));
    enabled.revise(|v| *v = false);
    app.settle(16).unwrap();
    assert!(app.scope().action(Action::Focus(id)).is_err());
    assert_eq!(
        host.renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap(),
        engine
    );
    app.clear().unwrap();
    assert!(r.get().is_none());
    assert_eq!(app.stats().queries, 0);
}
