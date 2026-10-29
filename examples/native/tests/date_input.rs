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

fn date(v: &str) -> CivilDate {
    CivilDate::parse_iso(v).unwrap()
}
#[test]
fn native_date_input_four_themes_preserve_drafts_actual_editor_caret_geometry_and_pixels() {
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
        let value = app.scope().cage(Some(date("2024-02-26")));
        host.mount(
            date_input("native-draft", DatesProvider::new(date("2024-02-29")))
                .with_picker(false)
                .theme(theme.clone())
                .name("Native civil draft")
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
        edit(app, id, "2024-", 5, 10, InputEditIntent::Replace);
        app.settle(16).unwrap();
        assert_eq!(value.get(), Some(date("2024-02-26")));
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
                "2024-"
            )
        });
        edit(app, id, "2023-02-29", 10, 5, InputEditIntent::Replace);
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
        edit(app, id, "2024-02-29", 10, 10, InputEditIntent::Replace);
        app.settle(16).unwrap();
        assert_eq!(value.get(), Some(date("2024-02-29")));
        app.scope()
            .set_input_selection(
                id,
                TextSelection {
                    start: 2,
                    end: 8,
                    direction: SelectionDirection::Backward,
                },
            )
            .unwrap();
        app.settle(16).unwrap();
        assert_eq!(
            query(app, Query::Selection(id)),
            QueryValue::Selection { start: 2, end: 8 }
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
fn native_date_input_default_dialog_updates_same_editor_and_readonly_disabled_unstyled_are_real()
 {
    let host = WindowApp::new();
    host.renderer().set_viewport(800, 650, 1.).unwrap();
    let app = host.app();
    let reference = app.scope().cage(None);
    let readonly = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    host.mount(
        date_input("native-popup", DatesProvider::new(date("2024-02-29")))
            .unstyled()
            .size(Size::ExtraSmall)
            .node_ref(reference)
            .readonly(readonly)
            .enabled(enabled)
            .picker_label("Choose native civil date")
            .picker_title("Native date input"),
    )
    .unwrap();
    app.settle(16).unwrap();
    let id = reference.get().unwrap();
    let physical = host
        .renderer()
        .engine_id(id, glory::html::Part::Main)
        .unwrap();
    let QueryValue::Bounds(rect) = query(app, Query::Bounds(id)) else {
        panic!()
    };
    assert!(rect.width >= 239. && rect.height >= 23.);
    let mut todo = app.tree().roots().to_vec();
    let mut trigger = None;
    let mut day = None;
    while let Some(n) = todo.pop() {
        let t = app.tree();
        let node = t.node(n).unwrap();
        todo.extend(node.children.iter().copied());
        if matches!(node.descriptor.view().props,Props::Button(p) if p.label=="Choose native civil date")
        {
            trigger = Some(n);
        }
        if node.descriptor.view().name.as_deref() == Some("Thursday, February 29, 2024") {
            day = Some(n);
        }
    }
    let trigger = trigger.unwrap();
    app.scope().action(Action::Focus(trigger)).unwrap();
    app.settle(16).unwrap();
    app.dispatch(trigger, Event::Activate).unwrap();
    app.settle(16).unwrap();
    assert_eq!(host.renderer().inspect(|doc| doc.top_layers().len()), 1);
    app.dispatch(day.unwrap(), Event::Activate).unwrap();
    app.settle(16).unwrap();
    assert_eq!(host.renderer().inspect(|doc| doc.top_layers().len()), 0);
    assert_eq!(query(app, Query::Focus(trigger)), QueryValue::Focus(true));
    host.renderer().inspect(|dom| {
        assert_eq!(
            dom.get_node(physical)
                .unwrap()
                .element_data()
                .unwrap()
                .text_input_data()
                .unwrap()
                .editor
                .raw_text(),
            "2024-02-29"
        )
    });
    readonly.revise(|v| *v = true);
    app.settle(16).unwrap();
    edit(app, id, "2024-02-28", 10, 10, InputEditIntent::Replace);
    app.settle(16).unwrap();
    assert_eq!(
        host.renderer()
            .engine_id(id, glory::html::Part::Main)
            .unwrap(),
        physical
    );
    enabled.revise(|v| *v = false);
    app.settle(16).unwrap();
    assert!(app.scope().action(Action::Focus(id)).is_err());
    app.clear().unwrap();
    assert_eq!(app.stats().queries, 0);
}
